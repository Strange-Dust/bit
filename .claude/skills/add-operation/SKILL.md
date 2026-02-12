# Add Operation

Step-by-step guide for adding a new bit operation to B.I.T. Takes the operation name as an argument.

## Instructions

When the user invokes this skill with an operation name (e.g., `/add-operation ReverseBits`):

### 1. Create the operation file

Create `src/operations/<snake_case_name>.rs` using `src/operations/invert.rs` as the template. The file must contain:

- **Operation struct** with `name: String`, `enabled: bool`, `#[derive(Debug, Clone, Serialize, Deserialize)]`
- **Methods**: `description() -> String`, `apply(input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0>`, `is_source_operation() -> bool`
- **Editor struct** implementing `Default`
- **`OperationEditor` trait impl** with `from_operation`, `try_build`, `render`
- Use `render_save_cancel_buttons` from `super::traits` for the editor footer

### 2. Register in `src/operations/mod.rs`

- Add `pub mod <snake_case_name>;` to module declarations
- Add `use <snake_case_name>::<OperationStruct>;` to imports
- Add entry to `register_operations!` with display_name, icon (empty string `""`), description, and category
- Add variant to `BitOperation` enum: `VariantName { #[serde(flatten)] inner: OperationStruct }`
- Add variant name to `impl_bit_operation_delegating!` invocation

### 3. Register editor in `src/operations/editor.rs`

- Add `use super::<snake_case_name>::<EditorStruct>;` import
- Add variant to `OperationEditorState` enum
- Add match arms in: `new_for_type`, `from_operation`, `try_build_operation`, `render`

### 4. Add tests

Add tests in `tests/operations_tests.rs` covering:
- Basic apply behavior
- Edge cases (empty input, single bit, etc.)
- Round-trip if applicable

### 5. Verify

Run `cargo build --release && cargo test && cargo clippy -- -D warnings` to confirm everything compiles and passes.

## Categories

Choose the appropriate category for the operation:
- `Loading` - Loads data from external sources
- `Transformation` - Transforms bit data
- `Analysis` - Analyzes or inspects data without modifying it
