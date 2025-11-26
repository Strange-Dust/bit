# Operation System Refactoring Options

This document shows how adding a new "Line Up" operation would look with different architectural approaches, compared to the current 5-file approach.

---

## Current Approach (5 Files)

**Files to edit:** 5
**Lines of boilerplate:** ~100+

See [adding-new-operation.md](adding-new-operation.md) for the full process.

---

## Option 1: Derive Macro

**Files to edit:** 1 (just the new operation file)
**Lines of boilerplate:** ~0
**Implementation effort:** High (requires separate proc-macro crate)

### What You'd Write

Create `src/operations/line_up.rs`:

```rust
use crate::operations::prelude::*;

#[derive(BitOperation)]
#[operation(
    name = "Line Up",
    icon = "📏",
    category = "Transformation",
    description = "Lines up bits according to specified alignment"
)]
pub struct LineUp {
    #[field(label = "Name", default = "Line Up")]
    pub name: String,

    #[field(label = "Alignment", kind = "number", default = "8")]
    pub alignment: usize,

    #[field(hidden)]
    pub enabled: bool,
}

impl LineUp {
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        // Your operation logic here
        let padding = (self.alignment - (input.len() % self.alignment)) % self.alignment;
        let mut result = input;
        result.resize(result.len() + padding, false);
        result
    }
}
```

### That's It!

The derive macro automatically generates:
- `BitOperation` enum variant
- `OperationType` enum variant
- All trait implementations (`name()`, `is_enabled()`, `description()`, etc.)
- UI editor with proper field types
- Serde serialization
- App state fields and save/load logic

### Required Infrastructure

You'd need to create a proc-macro crate (`bit-macros/`):

```
bit/
├── Cargo.toml
├── src/
│   └── ...
└── bit-macros/           # New proc-macro crate
    ├── Cargo.toml
    └── src/
        └── lib.rs        # ~300-500 lines of macro code
```

**bit-macros/Cargo.toml:**
```toml
[package]
name = "bit-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

**bit-macros/src/lib.rs** (simplified):
```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BitOperation, attributes(operation, field))]
pub fn derive_bit_operation(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract #[operation(...)] attributes
    // Generate enum variant
    // Generate trait impls
    // Generate UI editor
    // Generate app state handling

    let expanded = quote! {
        // Generated code here
    };

    TokenStream::from(expanded)
}
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Single file per operation | Complex macro implementation |
| Zero boilerplate | Harder to debug |
| Compile-time validation | Separate crate needed |
| Self-documenting | Learning curve for contributors |

---

## Option 2: Registration Macro (Recommended)

**Files to edit:** 2 (operation logic + registration)
**Lines of boilerplate:** ~10
**Implementation effort:** Medium

### What You'd Write

#### File 1: `src/operations/line_up.rs`

```rust
use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineUpOperation {
    pub name: String,
    pub alignment: usize,
    pub enabled: bool,
}

impl LineUpOperation {
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let padding = (self.alignment - (input.len() % self.alignment)) % self.alignment;
        let mut result = input;
        result.resize(result.len() + padding, false);
        result
    }
}
```

#### File 2: `src/operations/registry.rs` (add one entry)

```rust
register_operations! {
    // Existing operations...
    InvertBits {
        icon: "🔄",
        category: Transformation,
        description: "Inverts all bits (0→1, 1→0)",
        fields: {
            name: String = "Invert Bits",
        },
    },

    TruncateBits {
        icon: "✂️",
        category: Transformation,
        description: "Keep only bits within a range",
        fields: {
            name: String = "Truncate",
            start: usize = 0,
            end: usize = 0,
        },
    },

    // ADD YOUR NEW OPERATION HERE:
    LineUp {
        icon: "📏",
        category: Transformation,
        description: "Lines up bits to specified alignment",
        fields: {
            name: String = "Line Up",
            alignment: usize = 8,
        },
    },
}
```

### Required Infrastructure

Add the macro to `src/operations/mod.rs`:

```rust
macro_rules! register_operations {
    (
        $(
            $variant:ident {
                icon: $icon:literal,
                category: $category:ident,
                description: $desc:literal,
                fields: {
                    $( $field:ident : $type:ty = $default:expr ),* $(,)?
                } $(,)?
            }
        ),* $(,)?
    ) => {
        // Generate OperationType enum
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum OperationType {
            $( $variant ),*
        }

        impl OperationType {
            pub fn name(&self) -> &'static str {
                match self {
                    $( OperationType::$variant => stringify!($variant) ),*
                }
            }

            pub fn icon(&self) -> &'static str {
                match self {
                    $( OperationType::$variant => $icon ),*
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    $( OperationType::$variant => $desc ),*
                }
            }

            pub fn category(&self) -> OperationCategory {
                match self {
                    $( OperationType::$variant => OperationCategory::$category ),*
                }
            }

            pub fn all() -> &'static [OperationType] {
                &[ $( OperationType::$variant ),* ]
            }
        }

        // Generate BitOperation enum
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "type")]
        pub enum BitOperation {
            $(
                $variant {
                    $( $field: $type, )*
                    enabled: bool,
                }
            ),*
        }

        impl BitOperation {
            pub fn name(&self) -> &str {
                match self {
                    $( BitOperation::$variant { name, .. } => name ),*
                }
            }

            pub fn is_enabled(&self) -> bool {
                match self {
                    $( BitOperation::$variant { enabled, .. } => *enabled ),*
                }
            }

            pub fn set_enabled(&mut self, value: bool) {
                match self {
                    $( BitOperation::$variant { enabled, .. } => *enabled = value ),*
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    $( BitOperation::$variant { .. } => $desc ),*
                }
            }

            pub fn operation_type(&self) -> OperationType {
                match self {
                    $( BitOperation::$variant { .. } => OperationType::$variant ),*
                }
            }

            pub fn default_for(op_type: OperationType) -> Self {
                match op_type {
                    $(
                        OperationType::$variant => BitOperation::$variant {
                            $( $field: $default.into(), )*
                            enabled: true,
                        }
                    ),*
                }
            }
        }
    };
}
```

### UI Would Be Simplified Too

With the registry, the UI could use reflection-like patterns:

```rust
// In ui/windows.rs - generic editor that works for all operations
fn render_operation_editor(app: &mut BitApp, ui: &mut egui::Ui) {
    let op_type = app.show_operation_menu.unwrap();

    ui.heading(op_type.name());
    ui.label(op_type.description());
    ui.separator();

    // Use the editing_operation or create default
    let operation = app.editing_operation
        .get_or_insert_with(|| BitOperation::default_for(op_type));

    // Render fields based on operation type
    render_operation_fields(operation, ui);

    ui.add_space(16.0);

    if ui.button("Save").clicked() {
        app.save_current_operation();
    }
    if ui.button("Cancel").clicked() {
        app.cancel_operation_edit();
    }
}
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Single registration point | Still need operation logic file |
| No external crate needed | Macro syntax learning curve |
| Easy to see all operations | Complex macro to write initially |
| Compile-time checked | Limited UI customization |

---

## Option 3: Trait + Inventory Pattern

**Files to edit:** 1 (just the new operation file)
**Lines of boilerplate:** ~5
**Implementation effort:** Medium

### What You'd Write

Create `src/operations/line_up.rs`:

```rust
use crate::operations::prelude::*;
use inventory;

pub struct LineUpOperation;

impl OperationMeta for LineUpOperation {
    const NAME: &'static str = "Line Up";
    const ICON: &'static str = "📏";
    const DESCRIPTION: &'static str = "Lines up bits to specified alignment";
    const CATEGORY: OperationCategory = OperationCategory::Transformation;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineUpData {
    pub name: String,
    pub alignment: usize,
    pub enabled: bool,
}

impl Default for LineUpData {
    fn default() -> Self {
        Self {
            name: "Line Up".to_string(),
            alignment: 8,
            enabled: true,
        }
    }
}

impl Operation for LineUpData {
    fn name(&self) -> &str { &self.name }
    fn is_enabled(&self) -> bool { self.enabled }
    fn set_enabled(&mut self, v: bool) { self.enabled = v; }

    fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let padding = (self.alignment - (input.len() % self.alignment)) % self.alignment;
        let mut result = input;
        result.resize(result.len() + padding, false);
        result
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });
        ui.horizontal(|ui| {
            ui.label("Alignment:");
            ui.add(egui::DragValue::new(&mut self.alignment).range(1..=64));
        });
    }
}

// Auto-register at compile time!
inventory::submit! {
    OperationRegistration::new::<LineUpOperation, LineUpData>()
}
```

### Required Infrastructure

**Cargo.toml:**
```toml
[dependencies]
inventory = "0.3"
```

**src/operations/traits.rs:**
```rust
use bitvec::prelude::*;
use egui::Ui;

pub trait OperationMeta {
    const NAME: &'static str;
    const ICON: &'static str;
    const DESCRIPTION: &'static str;
    const CATEGORY: OperationCategory;
}

pub trait Operation: Send + Sync + Clone + serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn name(&self) -> &str;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0>;
    fn render_editor(&mut self, ui: &mut Ui);
}

pub struct OperationRegistration {
    pub name: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub category: OperationCategory,
    pub create_default: fn() -> Box<dyn Operation>,
}

impl OperationRegistration {
    pub fn new<M: OperationMeta, D: Operation + Default + 'static>() -> Self {
        Self {
            name: M::NAME,
            icon: M::ICON,
            description: M::DESCRIPTION,
            category: M::CATEGORY,
            create_default: || Box::new(D::default()),
        }
    }
}

inventory::collect!(OperationRegistration);
```

**src/operations/mod.rs:**
```rust
pub fn all_operations() -> impl Iterator<Item = &'static OperationRegistration> {
    inventory::iter::<OperationRegistration>()
}

pub fn find_operation(name: &str) -> Option<&'static OperationRegistration> {
    all_operations().find(|r| r.name == name)
}
```

### Pros & Cons

| Pros | Cons |
|------|------|
| True single-file operations | Requires `inventory` crate |
| No central registration | Dynamic dispatch (trait objects) |
| Plugin-friendly architecture | Slightly more complex serialization |
| Operations are self-contained | Each op needs more boilerplate |

---

## Option 4: Config File + Build Script

**Files to edit:** 2 (operation logic + config entry)
**Lines of boilerplate:** ~5
**Implementation effort:** Medium

### What You'd Write

#### File 1: `src/operations/line_up.rs`

```rust
use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineUpOperation {
    pub name: String,
    pub alignment: usize,
    pub enabled: bool,
}

impl LineUpOperation {
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let padding = (self.alignment - (input.len() % self.alignment)) % self.alignment;
        let mut result = input;
        result.resize(result.len() + padding, false);
        result
    }
}
```

#### File 2: `operations.toml` (add entry)

```toml
# Existing operations...

[[operations]]
id = "InvertBits"
name = "Invert Bits"
icon = "🔄"
category = "Transformation"
description = "Inverts all bits (0→1, 1→0)"
struct = "InvertBitsOperation"
fields = [
    { name = "name", type = "String", default = "Invert Bits", label = "Name" },
]

[[operations]]
id = "TruncateBits"
name = "Truncate Bits"
icon = "✂️"
category = "Transformation"
description = "Keep only bits within a range"
struct = "TruncateBitsOperation"
fields = [
    { name = "name", type = "String", default = "Truncate", label = "Name" },
    { name = "start", type = "usize", default = "0", label = "Start" },
    { name = "end", type = "usize", default = "0", label = "End" },
]

# ADD YOUR NEW OPERATION:
[[operations]]
id = "LineUp"
name = "Line Up"
icon = "📏"
category = "Transformation"
description = "Lines up bits to specified alignment"
struct = "LineUpOperation"
fields = [
    { name = "name", type = "String", default = "Line Up", label = "Name" },
    { name = "alignment", type = "usize", default = "8", label = "Alignment" },
]
```

### Required Infrastructure

**build.rs:**
```rust
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=operations.toml");

    let config = fs::read_to_string("operations.toml")
        .expect("Failed to read operations.toml");
    let config: OperationsConfig = toml::from_str(&config)
        .expect("Failed to parse operations.toml");

    let generated = generate_code(&config);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("operations_generated.rs");
    fs::write(&dest_path, generated).unwrap();
}

fn generate_code(config: &OperationsConfig) -> String {
    // Generate OperationType enum
    // Generate BitOperation enum
    // Generate all match arms
    // Generate UI field renderers
    format!(r#"
        // Auto-generated from operations.toml

        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum OperationType {{
            {variants}
        }}

        // ... rest of generated code
    "#, variants = config.operations.iter()
        .map(|o| &o.id)
        .collect::<Vec<_>>()
        .join(",\n            "))
}
```

**src/operations/mod.rs:**
```rust
// Include the generated code
include!(concat!(env!("OUT_DIR"), "/operations_generated.rs"));
```

### Pros & Cons

| Pros | Cons |
|------|------|
| Config is human-readable | Build script complexity |
| Non-Rust devs can add ops | Generated code harder to debug |
| Easy to validate/lint | IDE support may be limited |
| Could generate docs too | Adds build-time dependency |

---

## Comparison Summary

| Approach | Files to Edit | Boilerplate | Effort | Best For |
|----------|---------------|-------------|--------|----------|
| **Current** | 5 | ~100 lines | - | Small projects |
| **Derive Macro** | 1 | ~0 lines | High | Many operations, long-term |
| **Registration Macro** | 2 | ~10 lines | Medium | Balance of simplicity/DRY |
| **Trait + Inventory** | 1 | ~20 lines | Medium | Plugin architecture |
| **Config + Build** | 2 | ~5 lines | Medium | Team with non-Rust devs |

---

## Recommendation

**Option 2 (Registration Macro) has been implemented!**

See [adding-new-operation.md](adding-new-operation.md) for the current process.

The implementation provides:

1. **No external dependencies** - Pure Rust declarative macros
2. **Single source of truth** - All operations visible in `operations/mod.rs`
3. **Centralized editor state** - `OperationEditorState` in `operations/editor.rs`
4. **Easy to understand** - Contributors can see the pattern immediately
5. **Compile-time safety** - All operations validated at build time
6. **2-file editing** - Only modify `mod.rs` and `editor.rs` for new operations
