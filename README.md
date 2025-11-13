# B.I.T. - Bit Information Tool

A desktop application for manipulating bits in files, built with Rust and egui.

## Features

- **File Reading**: Open any file type and view its contents as bits
- **Bit Visualization**: Display bits as squares or circles with customizable size
- **Scrollable Viewer**: Efficiently render millions of bits with horizontal and vertical scrolling
- **Configurable Frame Length**: Adjust how many bits display per row
- **Bit Operations**: Apply sequences of bit manipulation operations:
  - `t` - Take: Include the next N bits in output
  - `r` - Reverse: Reverse the order of the next N bits
  - `i` - Invert: Flip the next N bits (1→0, 0→1)
  - `s` - Skip: Drop the next N bits from output
- **Operation Sequences**: Chain multiple operations and reorder them
- **File Export**: Save the modified bits to a new file
- **Zoom Controls**: Zoom in/out on the bit viewer

## Operation Syntax

Operations are specified using a compact syntax. Example: `t4r3i8s1`

This means:
- `t4` - Take 4 bits
- `r3` - Reverse 3 bits
- `i8` - Invert 8 bits
- `s1` - Skip 1 bit

The sequence repeats until the end of the file.

### Example

If the input bits are: `1011 001 01011110`

Applying `t4r3i8s1`:
1. `t4`: Take first 4 bits → `1011`
2. `r3`: Reverse next 3 bits (001) → `100`
3. `i8`: Invert next 8 bits - but only 2 remain (01) → `10`
4. `s1`: Skip 1 bit

Output: `1011 100 10` (then repeat on remaining bits)

## Building

### Prerequisites

- Rust (latest stable version)
- Cargo

### Build Instructions

```bash
# Build in release mode for optimal performance
cargo build --release

# Run the application
cargo run --release
```

## Usage

1. **Open a File**: Click the "📂 Open File" button to load any file
2. **View Bits**: The bit viewer will display the file's bits (black=1, white=0)
3. **Adjust Frame Length**: Use the slider to change how many bits per row
4. **Zoom**: Use ➕/➖ buttons to zoom in/out, 🔄 to reset
5. **Toggle Shape**: Switch between square (⬛) and circle (⚫) display
6. **Add Operations**: 
   - Type operation syntax in the input field (e.g., `t4r3i8s1`)
   - Click "➕ Add" or press Enter
   - Operations are applied in sequence
7. **Reorder Operations**: Use ⬆ and ⬇ buttons to change operation order
8. **View Results**: Toggle between "Original" and "Processed" to see the effect
9. **Save File**: Click "💾 Save File" to export the processed bits

## Performance

The application uses viewport culling to efficiently render millions of bits. Only visible bits are drawn, allowing smooth scrolling even with very large files.

## Testing

Run the test suite:

```bash
cargo test
```

## Architecture

- `main.rs` - Application entry point and UI logic
- `operations.rs` - Operation parsing and execution engine
- `file_io.rs` - File reading and writing utilities
- `bit_viewer.rs` - Bit visualization component with scrolling support

## License

MIT
