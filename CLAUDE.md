# B.I.T. - Bit Information Tool

Rust + egui desktop app for binary data visualization and manipulation.

## Build & Test

```bash
cargo build --release     # Build optimized binary
cargo test                # Run all tests
cargo clippy              # Lint check
cargo fmt -- --check      # Format check
```

CI (`.github/workflows/rust.yml`) runs build + test on push/PR to main.

## Architecture

```
src/
  main.rs              # Entry point (eframe::run_native)
  app.rs               # BitApp - main application struct, egui::App impl
  app_state/           # Application state management
  lib.rs               # Library exports and re-exports
  core/                # Core types (ViewMode, etc.)
  operations/          # Bit manipulation operations (see below)
  ui/                  # UI panels (top_panel, operations_panel, worksheets_panel, etc.)
  storage/             # File I/O, sessions, settings, templates, worksheets
  viewers/             # BitViewer, ByteViewer - data display widgets
  analysis/            # Pattern locator, frame width analysis
  utils/               # Shared utilities
tests/
  operations_tests.rs  # Integration tests for operations
  export_tests.rs      # Export functionality tests
  pattern_locator_tests.rs
```

## Operations System

Operations are the core abstraction. Each operation is **self-contained in its own file** with both logic and editor UI.

### Key files

- `src/operations/traits.rs` - `OperationEditor` trait (the contract every editor implements)
- `src/operations/mod.rs` - `register_operations!` macro, `BitOperation` enum, `impl_bit_operation_delegating!` macro
- `src/operations/editor.rs` - `OperationEditorState` enum (thin dispatch layer)
- `src/operations/base.rs` - `OperationSequence` (ordered list of operations)

### Adding a new operation

Use `src/operations/invert.rs` as the simplest reference implementation. The process:

1. **Create `src/operations/your_op.rs`** containing:
   - Operation struct (with `name: String`, `enabled: bool`, serde derives)
   - `description()`, `apply()`, `is_source_operation()` methods
   - Editor struct implementing `OperationEditor` trait (`from_operation`, `try_build`, `render`)

2. **Update `src/operations/mod.rs`** (~10 lines):
   - Add `pub mod your_op;` declaration
   - Add `use your_op::YourOperation;` import
   - Add entry to `register_operations!` macro (display name, icon, description, category)
   - Add variant to `BitOperation` enum
   - Add variant to `impl_bit_operation_delegating!` macro invocation

3. **Update `src/operations/editor.rs`** (~5 lines):
   - Import your editor type
   - Add variant to `OperationEditorState` enum
   - Add match arms in `new_for_type`, `from_operation`, `try_build_operation`, `render`

4. **Add tests** in `tests/operations_tests.rs`

### Operation categories

- `Loading` - Operations that load data (blue in UI)
- `Transformation` - Operations that transform bits (purple in UI)
- `Analysis` - Operations that analyze data (green in UI)

## Key Conventions

- **No emojis in UI strings** - egui's default font doesn't render them; icons are empty strings in `register_operations!`
- **Operations are self-contained** - logic + editor live in the same file
- **`OperationEditor` trait** in `traits.rs` defines the editor contract: `from_operation`, `try_build`, `render`
- **`register_operations!` macro** in `mod.rs` is the single source of truth for operation metadata
- **`impl_bit_operation_delegating!` macro** auto-generates method dispatch for `BitOperation` enum
- **Source operations** return `true` from `is_source_operation()` (e.g., LoadFile); transforms return `false`

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| eframe | 0.33 | Application framework |
| egui | 0.33 | Immediate-mode GUI |
| egui_plot | 0.34 | Plot widgets |
| egui_extras | 0.33 | Extra egui widgets |
| bitvec | 1.0 | Bit-level data structures |
| serde / serde_json | 1.0 | Serialization (sessions, templates) |
| rfd | 0.15 | Native file dialogs |
| dirs | 5.0 | Platform config directories |
| base64 | 0.22 | Base64 encoding |

Dev: `tempfile 3.15` for test fixtures.

## Testing

- Unit tests: Inside modules (`#[cfg(test)]` blocks)
- Integration tests: `tests/` directory
- Run specific test: `cargo test test_name`
- Run with output: `cargo test -- --nocapture`
