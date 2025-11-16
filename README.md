# B.I.T. - Bit Information Tool

A desktop application for binary data visualization and manipulation. Built with Rust and egui.

## Features

- View Modes: Bit, Byte (hex), and ASCII visualization
- Operations: Take/Skip sequences, Invert, Truncate, Interleaving (Block/Convolutional/Symbol), Multi-Worksheet Load
- Pattern Search: Find patterns with fuzzy matching
- Worksheets: Multiple files with independent operation pipelines
- Sessions: Auto-save and restore

## Installation

```
cargo build --release
cargo run --release
```

Binary: `target/release/bit.exe` (Windows) or `target/release/bit` (Unix)

## Contributing

```
git clone https://github.com/Strange-Dust/bit.git
cd bit
cargo test
```

Add operations by:
1. Add variant to `BitOperation` in `src/processing/operations.rs`
2. Implement `apply()` logic
3. Add UI in `src/ui/windows.rs`
4. Write tests in `tests/operations_tests.rs`

## License

MIT
