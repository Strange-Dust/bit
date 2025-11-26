# Adding a New Operation: Complete Example

This guide walks through adding a **"Pad To Alignment"** operation that pads bits to align to a boundary.

**Files to modify:** 2 (`mod.rs` and `editor.rs`)

---

## Step 1: Create the Operation Logic File

Create `src/operations/pad_alignment.rs`:

```rust
use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

/// Pads bits to align to a specified boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PadAlignmentOperation {
    pub name: String,
    pub alignment: usize,
    pub pad_value: bool,  // false = pad with 0s, true = pad with 1s
    pub enabled: bool,
}

impl PadAlignmentOperation {
    pub fn description(&self) -> String {
        format!(
            "Pad to {} bits with {}s",
            self.alignment,
            if self.pad_value { "1" } else { "0" }
        )
    }

    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        if self.alignment == 0 || input.is_empty() {
            return input;
        }

        let remainder = input.len() % self.alignment;
        if remainder == 0 {
            return input;
        }

        let padding_needed = self.alignment - remainder;
        let mut result = input;
        result.resize(result.len() + padding_needed, self.pad_value);
        result
    }
}
```

---

## Step 2: Register in `mod.rs`

Edit `src/operations/mod.rs`:

### 2a. Add module declaration (top of file):

```rust
pub mod base;
pub mod editor;
pub mod interleaver;
pub mod load_file;
pub mod take_skip_sequence;
pub mod invert;
pub mod multi_worksheet;
pub mod truncate;
pub mod pad_alignment;  // <-- ADD THIS
```

### 2b. Import the operation struct:

```rust
use load_file::LoadFileOperation;
use take_skip_sequence::TakeSkipSequenceOperation;
use invert::InvertBitsOperation;
use multi_worksheet::MultiWorksheetLoadOperation;
use truncate::TruncateBitsOperation;
use interleaver::InterleaveBitsOperation;
use pad_alignment::PadAlignmentOperation;  // <-- ADD THIS
```

### 2c. Add to `register_operations!` macro:

```rust
register_operations! {
    LoadFile { ... },
    TakeSkipSequence { ... },
    InvertBits { ... },
    MultiWorksheetLoad { ... },
    TruncateBits { ... },
    InterleaveBits { ... },

    // ADD THIS:
    PadAlignment {
        display_name: "Pad to Alignment",
        icon: "📐",
        description: "Pad bits to align to a boundary",
        category: Transformation,
    },
}
```

### 2d. Add to `BitOperation` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BitOperation {
    LoadFile { ... },
    TakeSkipSequence { ... },
    InvertBits { ... },
    MultiWorksheetLoad { ... },
    TruncateBits { ... },
    InterleaveBits { ... },

    // ADD THIS:
    PadAlignment {
        name: String,
        alignment: usize,
        pad_value: bool,
        enabled: bool,
    },
}
```

### 2e. Add to `impl_bit_operation_common!` macro:

```rust
impl_bit_operation_common!(
    LoadFile,
    TakeSkipSequence,
    InvertBits,
    MultiWorksheetLoad,
    TruncateBits,
    InterleaveBits,
    PadAlignment,  // <-- ADD THIS
);
```

### 2f. Add `description()` match arm:

```rust
pub fn description(&self) -> String {
    match self {
        // ... existing arms ...

        BitOperation::PadAlignment { alignment, pad_value, .. } => {
            PadAlignmentOperation {
                name: String::new(),
                alignment: *alignment,
                pad_value: *pad_value,
                enabled: true,
            }.description()
        }
    }
}
```

### 2g. Add `apply()` match arm:

```rust
pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
    match self {
        // ... existing arms ...

        BitOperation::PadAlignment { alignment, pad_value, .. } => {
            PadAlignmentOperation {
                name: String::new(),
                alignment: *alignment,
                pad_value: *pad_value,
                enabled: true,
            }.apply(input)
        }
    }
}
```

---

## Step 3: Add Editor State and UI in `editor.rs`

Edit `src/operations/editor.rs`:

### 3a. Add fields to `OperationEditorState`:

```rust
pub struct OperationEditorState {
    // Common
    pub name: String,

    // ... existing fields ...

    // PadAlignment - ADD THESE:
    pub alignment_str: String,
    pub pad_value: bool,
}
```

### 3b. Update `Default` impl if needed:

```rust
impl Default for OperationEditorState {
    fn default() -> Self {
        Self {
            name: String::new(),
            // ... existing defaults ...
            alignment_str: "8".to_string(),
            pad_value: false,
        }
    }
}
```

### 3c. Add to `new_for_type()`:

```rust
pub fn new_for_type(op_type: OperationType) -> Self {
    let mut state = Self::default();

    match op_type {
        // ... existing arms ...

        OperationType::PadAlignment => {
            state.alignment_str = "8".to_string();
            state.pad_value = false;
        }
    }

    state
}
```

### 3d. Add to `from_operation()`:

```rust
pub fn from_operation(op: &BitOperation) -> Self {
    match op {
        // ... existing arms ...

        BitOperation::PadAlignment { name, alignment, pad_value, .. } => Self {
            name: name.clone(),
            alignment_str: alignment.to_string(),
            pad_value: *pad_value,
            ..Self::default()
        },
    }
}
```

### 3e. Add to `try_build_operation()`:

```rust
pub fn try_build_operation(&self, op_type: OperationType) -> Result<BitOperation, String> {
    match op_type {
        // ... existing arms ...

        OperationType::PadAlignment => {
            let alignment = self.alignment_str.trim().parse::<usize>()
                .map_err(|_| "Alignment must be a positive number".to_string())?;

            if alignment == 0 {
                return Err("Alignment must be greater than 0".to_string());
            }

            let name = if self.name.trim().is_empty() {
                format!("Pad to {} bits", alignment)
            } else {
                self.name.clone()
            };

            Ok(BitOperation::PadAlignment {
                name,
                alignment,
                pad_value: self.pad_value,
                enabled: true,
            })
        }
    }
}
```

### 3f. Add editor render function:

```rust
fn render_pad_alignment_editor(state: &mut OperationEditorState, ui: &mut egui::Ui) -> EditorAction {
    ui.heading("Pad to Alignment");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Alignment (bits):");
        ui.text_edit_singleline(&mut state.alignment_str);
    });

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("Pad with:");
        ui.radio_value(&mut state.pad_value, false, "0s");
        ui.radio_value(&mut state.pad_value, true, "1s");
    });

    ui.add_space(8.0);

    // Preview
    if let Ok(alignment) = state.alignment_str.trim().parse::<usize>() {
        if alignment > 0 {
            ui.label(format!("Example: 13 bits → {} bits (adds {} padding bits)",
                ((13 + alignment - 1) / alignment) * alignment,
                alignment - (13 % alignment)
            ));
        }
    }

    ui.add_space(4.0);
    ui.label("💡 Common alignments: 8 (byte), 16 (word), 32 (dword)");

    ui.add_space(8.0);

    let is_valid = state.alignment_str.trim().parse::<usize>().map_or(false, |v| v > 0);
    render_save_cancel_buttons(ui, is_valid)
}
```

### 3g. Add to `render_operation_editor()` dispatch:

```rust
pub fn render_operation_editor(
    op_type: OperationType,
    state: &mut OperationEditorState,
    ctx: &EditorContext,
    ui: &mut egui::Ui,
) -> EditorAction {
    match op_type {
        OperationType::LoadFile => render_loadfile_editor(state, ui),
        OperationType::TakeSkipSequence => render_takeskip_editor(state, ui),
        OperationType::InvertBits => render_invert_editor(state, ui),
        OperationType::TruncateBits => render_truncate_editor(state, ui),
        OperationType::InterleaveBits => render_interleave_editor(state, ui),
        OperationType::MultiWorksheetLoad => render_multiworksheet_editor(state, ctx, ui),
        OperationType::PadAlignment => render_pad_alignment_editor(state, ui),  // <-- ADD THIS
    }
}
```

---

## Step 4: Verify

```bash
cargo check
cargo run
```

The operation will now appear in the "Add Operation" menu under the Transformation category.

---

## Summary

| File | What to add |
|------|-------------|
| `src/operations/pad_alignment.rs` | **NEW:** Operation struct + `apply()` logic |
| `src/operations/mod.rs` | Module declaration, import, register in macro, add to `BitOperation` enum |
| `src/operations/editor.rs` | Editor state fields, `new_for_type`, `from_operation`, `try_build_operation`, render function, dispatch |

**Total: ~50 lines in mod.rs + ~50 lines in editor.rs + operation logic file**
