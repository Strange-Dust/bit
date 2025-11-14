# Operation System Architecture

## Overview

The B.I.T. application now uses a scalable operation system that allows for easy addition of new operation types.

## Structure

### Two-Column Layout

The operations panel is divided into two columns:

1. **Available Operations (Left)**: Lists all available operation types that can be added
2. **Active Operations (Right)**: Shows operations that have been added and are being applied to the bits

### Operation Types

Each operation type is defined in the `OperationType` enum and has:
- A name (displayed in the UI)
- An icon (emoji for visual identification)
- A description (shown on hover)
- A specific configuration menu

### Current Operation Types

1. **Take/Skip Sequence** (📝)
   - Pattern-based bit extraction using syntax like `t4r3i8s1`
   - Operations: take (t), reverse (r), invert (i), skip (s)
   - Configuration: Text input for the sequence

### Adding New Operation Types

To add a new operation type (e.g., "Find Pattern"):

1. **Add to `OperationType` enum** (`src/main.rs`):
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum OperationType {
    TakeSkipSequence,
    FindPattern,  // New operation type
}
```

2. **Implement the helper methods**:
```rust
impl OperationType {
    fn name(&self) -> &str {
        match self {
            OperationType::TakeSkipSequence => "Take/Skip Sequence",
            OperationType::FindPattern => "Find Pattern",
        }
    }

    fn icon(&self) -> &str {
        match self {
            OperationType::TakeSkipSequence => "📝",
            OperationType::FindPattern => "🔍",
        }
    }

    fn description(&self) -> &str {
        match self {
            OperationType::TakeSkipSequence => "Pattern-based bit extraction (t4r3i8s1)",
            OperationType::FindPattern => "Find and highlight bit patterns",
        }
    }
}
```

3. **Add to `BitOperation` enum** (`src/operations.rs`):
```rust
#[derive(Debug, Clone)]
pub enum BitOperation {
    TakeSkipSequence(OperationSequence),
    FindPattern { pattern: String, highlight: bool },
}
```

4. **Implement the methods**:
```rust
impl BitOperation {
    pub fn name(&self) -> &str {
        match self {
            BitOperation::TakeSkipSequence(_) => "Take/Skip Sequence",
            BitOperation::FindPattern { .. } => "Find Pattern",
        }
    }

    pub fn description(&self) -> String {
        match self {
            BitOperation::TakeSkipSequence(seq) => seq.to_string(),
            BitOperation::FindPattern { pattern, .. } => format!("Pattern: {}", pattern),
        }
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self {
            BitOperation::TakeSkipSequence(seq) => seq.apply(input),
            BitOperation::FindPattern { .. } => {
                // Implement pattern finding logic
                input.clone()  // For now, just return input unchanged
            }
        }
    }
}
```

5. **Add state variables** in `BitApp`:
```rust
struct BitApp {
    // ... existing fields ...
    
    // FindPattern editor state
    findpattern_input: String,
    findpattern_highlight: bool,
}
```

6. **Add to configuration window** (`src/main.rs` in `update()` method):
```rust
match op_type {
    OperationType::TakeSkipSequence => {
        // Existing UI code
    }
    OperationType::FindPattern => {
        ui.heading("Find Pattern");
        ui.separator();
        
        ui.label("Enter a bit pattern to find:");
        ui.text_edit_singleline(&mut self.findpattern_input);
        
        ui.checkbox(&mut self.findpattern_highlight, "Highlight matches");
        
        ui.horizontal(|ui| {
            if ui.button("✓ Save").clicked() {
                self.save_current_operation();
            }
            if ui.button("✗ Cancel").clicked() {
                self.cancel_operation_edit();
            }
        });
    }
}
```

7. **Update `save_current_operation()`**:
```rust
fn save_current_operation(&mut self) {
    if let Some(op_type) = self.show_operation_menu {
        let new_operation = match op_type {
            OperationType::TakeSkipSequence => { /* ... */ }
            OperationType::FindPattern => {
                BitOperation::FindPattern {
                    pattern: self.findpattern_input.clone(),
                    highlight: self.findpattern_highlight,
                }
            }
        };
        // ... rest of the method
    }
}
```

8. **Update `open_operation_editor()`**:
```rust
fn open_operation_editor(&mut self, index: usize) {
    if let Some(op) = self.operations.get(index) {
        match op {
            BitOperation::TakeSkipSequence(seq) => { /* ... */ }
            BitOperation::FindPattern { pattern, highlight } => {
                self.show_operation_menu = Some(OperationType::FindPattern);
                self.editing_operation_index = Some(index);
                self.findpattern_input = pattern.clone();
                self.findpattern_highlight = *highlight;
            }
        }
    }
}
```

## Features

- **Editable Operations**: Each added operation has an "Edit" button that reopens its configuration menu with current settings
- **Reorderable**: Operations can be moved up/down to change execution order
- **Deletable**: Operations can be removed individually or all cleared at once
- **Persistent**: Operation order and settings are maintained during the session
