# B.I.T. - Bit Information Tool

A desktop application for manipulating bits in files, built with Rust and egui.

## Features

### Core Functionality
- **File Reading**: Open any file type and view its contents as bits
- **Bit Visualization**: Display bits as squares or circles with customizable size
- **Scrollable Viewer**: Efficiently render millions of bits with horizontal and vertical scrolling
- **Configurable Frame Length**: Adjust how many bits display per row
- **Zoom Controls**: Zoom in/out on the bit viewer
- **File Export**: Save the modified bits to a new file

### Bit Operations
Apply sequences of bit manipulation operations:
- `t` - **Take**: Include the next N bits in output
- `r` - **Reverse**: Reverse the order of the next N bits
- `i` - **Invert**: Flip the next N bits (1→0, 0→1)
- `s` - **Skip**: Drop the next N bits from output
- **Custom Invert**: Create named invert operations with custom bit patterns
- **Multi-Worksheet Load**: Combine bits from multiple worksheets with individual operation sequences

### Pattern Locator
- **Pattern Search**: Search for bit patterns in three formats:
  - **Hexadecimal**: e.g., `DEADBEEF`, `A1B2C3`
  - **ASCII**: Search for text strings as bit patterns
  - **Binary**: Direct bit pattern search, e.g., `10110011`
- **Garble Tolerance**: Fuzzy matching with configurable tolerance (0-8 bits)
- **Visual Highlighting**: Matched patterns highlighted with semi-transparent yellow overlay
- **Compact Results**: Shows position, delta from previous match, and mismatch count
- **Click to Jump**: Click any match to jump directly to that bit position

### Worksheet Management
- **Multiple Worksheets**: Work with multiple files simultaneously in separate worksheets
- **Worksheet Switching**: Easily switch between different worksheets
- **Per-Worksheet Operations**: Each worksheet maintains its own operation sequence
- **Cross-Worksheet Operations**: Load and combine bits from multiple worksheets

### Session Persistence
- **Auto-Save**: Your work is automatically saved when closing the application
- **Session Restore**: On startup, choose to restore your previous session or start fresh
- **Persistent State**: All worksheets, operations, and current position are preserved

## Operation Syntax

### Basic Operations
Operations are specified using a compact syntax. Example: `t4r3i8s1`

This means:
- `t4` - Take 4 bits
- `r3` - Reverse 3 bits
- `i8` - Invert 8 bits
- `s1` - Skip 1 bit

The sequence repeats until the end of the file.

### Advanced Operations

#### Custom Invert Operations
Create named invert operations for reusable bit transformations:
- Give it a descriptive name (e.g., "Scramble", "Flip Header")
- Define the operation sequence (e.g., `t4i8t4`)
- Use it like any other operation in your workflow

#### Multi-Worksheet Load
Combine bits from multiple worksheets with individual processing:
- Select multiple source worksheets
- Each worksheet can have its own operation sequence
- Bits are combined in the order specified
- Perfect for merging processed data from different files

### Example

If the input bits are: `1011 001 01011110`

Applying `t4r3i8s1`:
1. `t4`: Take first 4 bits → `1011`
2. `r3`: Reverse next 3 bits (001) → `100`
3. `i8`: Invert next 8 bits - but only 2 remain (01) → `10`
4. `s1`: Skip 1 bit

Output: `1011 100 10` (then repeat on remaining bits)

### Pattern Search Example

Searching for hex pattern `DEAD` with 2-bit garble tolerance:
- Finds exact match: `1101 1110 1010 1101`
- Also finds near-matches like: `1101 1110 1011 1101` (1 bit different)
- Results displayed as: `#1 @1024 Δ0 ~0` (match #1 at position 1024, 0 bits from start, 0 mismatches)

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

### Getting Started
1. **Open a File**: Click the "📂 Open File" button to load any file
2. **View Bits**: The bit viewer will display the file's bits (black=1, white=0)
3. **Adjust Frame Length**: Use the slider to change how many bits per row
4. **Zoom**: Use ➕/➖ buttons to zoom in/out, 🔄 to reset
5. **Toggle Shape**: Switch between square (⬛) and circle (⚫) display

### Working with Operations
1. **Add Basic Operations**: 
   - Type operation syntax in the input field (e.g., `t4r3i8s1`)
   - Click "➕ Add" or press Enter
   - Operations are applied in sequence
2. **Reorder Operations**: Use ⬆ and ⬇ buttons to change operation order
3. **Custom Invert Operations**:
   - Select "Invert Bits" from the operation type dropdown
   - Enter a name for the operation (e.g., "Scramble")
   - Input the operation sequence
   - Click "➕ Add"
4. **Multi-Worksheet Load**:
   - Select "Multi-Worksheet Load" from the operation type dropdown
   - Choose which worksheets to include
   - Specify individual operation sequences for each worksheet
   - Bits are combined in the order specified

### Pattern Locator
1. **Open Pattern Locator**: Click the "🔍 Pattern Locator" button
2. **Select Format**: Choose Hex, ASCII, or Bits
3. **Enter Pattern**: Type the pattern to search for
4. **Set Garble Tolerance**: Adjust slider for fuzzy matching (0 = exact match)
5. **Search**: Click "🔍 Search" to find all matches
6. **Navigate Results**: Click on any match to jump to that position
   - Results show: `#N @position ΔN ~N` (match number, position, delta, mismatches)

### Multiple Worksheets
1. **Create Worksheet**: Click "➕ New Worksheet" button
2. **Switch Worksheets**: Click on worksheet tabs to switch between them
3. **Load Files**: Each worksheet can load a different file
4. **Independent Operations**: Each worksheet has its own operation sequence
5. **Combine Worksheets**: Use Multi-Worksheet Load to merge bits from multiple worksheets

### Session Management
- **Automatic Save**: Your session is automatically saved when you close the application
- **Restore on Startup**: When reopening, you'll be prompted to:
  - **🔄 Restore Session**: Continue where you left off
  - **🆕 Start Fresh**: Begin with a clean default worksheet
- **All State Preserved**: Worksheets, files, operations, and viewer position are all saved

### Viewing Results
1. **Toggle View**: Switch between "Original" and "Processed" to see the effect of operations
2. **Pattern Highlights**: Matched patterns are shown with yellow overlay
3. **Scroll & Navigate**: Use scrollbars or click pattern matches to navigate
4. **Save File**: Click "💾 Save File" to export the processed bits

## Performance

The application uses viewport culling to efficiently render millions of bits. Only visible bits are drawn, allowing smooth scrolling even with very large files.

## Testing

Run the test suite:

```bash
cargo test
```

## Architecture

- `main.rs` - Application entry point and UI logic
- `operations.rs` - Operation parsing and execution engine (includes Multi-Worksheet Load)
- `file_io.rs` - File reading and writing utilities
- `bit_viewer.rs` - Bit visualization component with scrolling support and pattern highlighting
- `pattern_locator.rs` - Pattern search with fuzzy matching using Hamming distance
- `session.rs` - Session persistence (save/load application state)

## Technical Details

### Pattern Matching
- Uses **Hamming distance** algorithm for fuzzy matching
- Configurable tolerance allows finding patterns with up to 8 bit differences
- Supports multiple pattern formats (Hex, ASCII, Binary) with automatic conversion

### Session Storage
- Sessions saved to platform-specific config directory:
  - Linux/Mac: `~/.config/bit/last_session.json`
  - Windows: `%APPDATA%\bit\last_session.json`
- Includes all worksheets, operation sequences, and current state
- Automatic cleanup when starting fresh

## License

MIT
