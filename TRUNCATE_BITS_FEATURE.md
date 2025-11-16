# Truncate Bits Operation - Feature Summary

## Overview
A new "Truncate Bits" operation has been added to the B.I.T. application, allowing users to extract specific bit ranges from their data and discard the rest.

## Usage

### Creating a Truncate Bits Operation
1. Click the "✂️ Truncate bits" button in the operations panel
2. Configure the operation:
   - **Name**: (Optional) Give the operation a descriptive name
   - **Start**: The starting bit index (default: 0)
   - **End**: The ending bit index (optional - leave empty to keep to end of file)

### Syntax Examples

| Input | Start | End | Result |
|-------|-------|-----|--------|
| 1000 bits | 0 | 250 | Keep bits 0-249, drop 250-999 |
| 1000 bits | 100 | 250 | Drop bits 0-99, keep bits 100-249, drop 250-999 |
| 1000 bits | 0 | (empty) | Keep all bits 0-999 |
| 1000 bits | 300 | (empty) | Drop bits 0-299, keep bits 300-999 |

## Implementation Details

### Files Modified
1. **core/types.rs** - Added `TruncateBits` to `OperationType` enum
2. **processing/operations.rs** - Added `TruncateBits` variant to `BitOperation` enum with apply logic
3. **app.rs** - Added UI state management and operation handlers
4. **ui/windows.rs** - Added `render_truncate_editor()` UI window
5. **main.rs** - Added to operations list

### Technical Implementation
- Uses BitVec slicing: `input[start..end].to_bitvec()`
- Bounds checking: Values are clamped to actual data length
- Edge cases handled:
  - If `start >= end` after clamping: returns empty BitVec
  - If end is empty/not provided: uses `usize::MAX` (keeps to end)
  - If start is beyond data length: returns empty BitVec

### Testing
8 comprehensive tests were added covering:
- ✓ Basic truncation (0-5)
- ✓ Middle range extraction (3-7)
- ✓ Truncate to end (start to EOF)
- ✓ Range beyond length (clamping)
- ✓ Start beyond length (empty result)
- ✓ Start equals end (empty result)
- ✓ Single bit extraction
- ✓ Description generation

All 104 tests pass successfully.

## Build Status
- **Compilation**: ✓ Success (3.95s)
- **Warnings**: 0
- **Tests**: 104 passed, 0 failed
- **Release Build**: ✓ Ready

## Icon
The operation uses the scissors emoji (✂️) for intuitive recognition.

## Serialization Support
The operation fully supports saving/loading with worksheets through serde serialization.
