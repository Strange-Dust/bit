# B.I.T. Usage Examples

## Quick Start

1. **Launch the Application**
   ```bash
   cargo run --release
   ```

2. **Open a File**
   - Click "📂 Open File" button
   - Select any file (text, image, binary, etc.)
   - The bit viewer will display the file's bits

3. **Apply Operations**

   Example: `t4r3i8s1`
   
   This operation:
   - Takes 4 bits
   - Reverses the next 3 bits
   - Inverts the next 8 bits
   - Skips 1 bit
   - Repeats until end of file

## Example Operations

### Simple Take and Skip
```
t8s8
```
Takes every other byte (takes 8 bits, skips 8 bits, repeats)

### Reverse Bytes
```
r8
```
Reverses each byte

### Invert All Bits
```
i1
```
Inverts every single bit (NOT operation on entire file)

### Complex Pattern
```
t16r8i4s2
```
- Take 16 bits (2 bytes)
- Reverse next 8 bits (1 byte)
- Invert next 4 bits (half byte)
- Skip 2 bits

### Scramble Pattern
```
t1s1t1s1t1s1t1s1
```
Takes alternating bits (creates a decimated version of the file)

## Tips

- **Frame Length**: Adjust to powers of 2 for clean visualization (8, 16, 32, 64, 128, 256)
- **Zoom**: Use for large files - zoom out to see patterns, zoom in for detail
- **Square vs Circle**: Circles may look better at small sizes, squares at large sizes
- **Original vs Processed**: Toggle to compare before/after
- **Multiple Operations**: Stack operations - they apply in sequence

## Example Workflow

1. Open an image file (e.g., PNG)
2. Set frame length to 128 (likely close to image width)
3. Apply `i1` to invert all colors
4. Save to new file
5. Open the new file in image viewer to see inverted image

## Creating Custom Patterns

You can create interesting visual patterns by combining operations:

```
t3r3i3s3  # Creates a repeating pattern
t1i1      # Alternates between original and inverted bits
r4        # Groups of 4 bits reversed (nibble swap)
```

## Performance Notes

- The viewer uses viewport culling for efficiency
- Files with millions of bits render smoothly
- Zoom out to see overview, zoom in for detail
- Operations are applied lazily when you toggle to "Processed" view
