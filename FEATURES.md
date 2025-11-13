# B.I.T. Application Features

## Main Interface Components

### Top Panel (Toolbar)
- **📂 Open File** - Load any file for bit viewing
- **💾 Save File** - Export processed bits to a new file
- **Frame Length Slider** - Adjust bits per row (8-512)
- **Zoom Controls** - ➕ Zoom In | ➖ Zoom Out | 🔄 Reset
- **Shape Selection** - ⬛ Square | ⚫ Circle
- **View Toggle** - Original | Processed

### Left Panel (Operations)
- **Operation Input** - Text field for entering operation syntax
- **➕ Add Button** - Add operation to sequence
- **Operation List** - Shows all active operations with:
  - ☰ Drag handle (for future reordering)
  - Operation display (e.g., "1. t4r3i8s1")
  - 🗑 Delete button
  - ⬆ Move up
  - ⬇ Move down
- **Batch Actions**:
  - 🔄 Reapply All Operations
  - 🗑 Clear All Operations
- **File Info** - Current file path and bit counts

### Center Panel (Bit Viewer)
- Scrollable viewport (horizontal and vertical)
- Virtualized rendering (only visible bits drawn)
- Color coding:
  - Black = 1
  - White = 0
- Grid with outlines for clarity

## Keyboard Shortcuts
- **Enter** (in operation input) - Add operation
- **Mouse Wheel** (in viewer) - Scroll vertically
- **Shift + Mouse Wheel** (in viewer) - Scroll horizontally

## Operation Syntax Reference

| Operation | Syntax | Description | Example |
|-----------|--------|-------------|---------|
| Take | `tN` | Include next N bits | `t8` takes 8 bits |
| Reverse | `rN` | Reverse next N bits | `r4` reverses 4 bits |
| Invert | `iN` | Flip next N bits | `i16` inverts 16 bits |
| Skip | `sN` | Drop next N bits | `s2` skips 2 bits |

### Chaining Operations
Combine multiple operations: `t4r3i8s1`

This creates a sequence that repeats:
1. Take 4 bits
2. Reverse 3 bits
3. Invert 8 bits
4. Skip 1 bit
5. Loop back to step 1 until EOF

## Common Use Cases

### 1. File Inspection
- Open unknown file
- Set appropriate frame length
- Visually identify patterns, headers, structure

### 2. Data Transformation
- Apply inversion, reversal, or selective filtering
- Save transformed data
- Useful for simple encryption/obfuscation

### 3. Visual Analysis
- Find repeating patterns
- Identify compression artifacts
- Analyze file structure

### 4. Educational
- Learn about file formats
- Understand binary data
- Experiment with bit manipulation

## Performance Characteristics

- **Viewport Culling**: Only visible bits are rendered
- **Lazy Evaluation**: Operations applied on-demand
- **Scalability**: Tested with files up to several GB
- **Zoom Levels**: From 2px to 100px per bit
- **Frame Widths**: 8 to 512 bits per row

## File Format Support

B.I.T. works with **any file type**:
- Text files (.txt, .md, .json, .xml, etc.)
- Images (.png, .jpg, .gif, .bmp, etc.)
- Executables (.exe, .dll, .so, etc.)
- Archives (.zip, .tar, .gz, etc.)
- Audio/Video (.mp3, .mp4, .avi, etc.)
- Documents (.pdf, .docx, etc.)
- Raw binary files

All files are read as pure binary and displayed at the bit level.

## Advanced Tips

1. **Finding Patterns**: Use zoom and frame length to align with expected data structures
2. **Byte Boundaries**: Set frame length to multiples of 8 for byte-aligned viewing
3. **Quick Inversion**: Use `i1` to invert entire file (creates bitwise NOT)
4. **Data Filtering**: Use take/skip patterns to extract specific bit patterns
5. **Operation Stacking**: Build complex transformations by adding multiple operation sequences

## Troubleshooting

- **File won't open**: Check file permissions
- **Viewer is slow**: Zoom out less, reduce visible area
- **Operations seem wrong**: Check syntax, remember operations repeat
- **Can't see changes**: Make sure you're viewing "Processed" not "Original"
