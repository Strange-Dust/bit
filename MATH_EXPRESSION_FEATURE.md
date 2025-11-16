# Math Expression Support for Truncate Bits - Feature Summary

## Overview
Added math expression evaluation to the Truncate Bits operation's Start and End input fields, allowing users to calculate values directly in the UI.

## Usage

### How to Use
1. Open the Truncate Bits operation editor
2. In the "Start" or "End" field, type a math expression (e.g., `8*8`)
3. Press **Enter** to evaluate the expression
4. The field will automatically update with the calculated result

### Supported Operations
- **Addition**: `100+50` → `150`
- **Subtraction**: `200-10` → `190`
- **Multiplication**: `8*8` → `64`
- **Division**: `64/2` → `32`

### Order of Operations
The evaluator follows standard mathematical order of operations (multiplication and division before addition and subtraction):
- `2+3*4` → `14` (not 20)
- `100+20*3-10/2` → `155`
- `8*8+16` → `80`

### Examples
| Expression | Result |
|------------|--------|
| `8*8` | `64` |
| `100+50` | `150` |
| `200-10` | `190` |
| `64/2` | `32` |
| `2+3*4` | `14` |
| `100/5/2` | `10` |
| `8 * 8` | `64` (spaces are ignored) |

### Features
- **Whitespace Handling**: Spaces are automatically ignored
- **Direct Numbers**: If you just type a number (e.g., `250`), it works as before
- **Error Handling**: Invalid expressions show error messages
- **Division by Zero Protection**: Prevents divide-by-zero errors
- **Negative Result Protection**: Prevents negative results (usize cannot be negative)

## Implementation Details

### Files Modified
1. **src/utils/math_eval.rs** (NEW) - Math expression evaluator with comprehensive tests
2. **src/utils/mod.rs** (NEW) - Utils module exports
3. **src/lib.rs** - Added utils module export
4. **src/main.rs** - Added utils module declaration
5. **src/ui/windows.rs** - Integrated math evaluation on Enter key press

### Technical Implementation
- **Tokenizer**: Parses expressions into tokens (numbers and operators)
- **Evaluator**: Two-pass evaluation:
  1. First pass: Process `*` and `/` operations (higher precedence)
  2. Second pass: Process `+` and `-` operations (lower precedence)
- **UI Integration**: Uses egui's `lost_focus()` and `key_pressed()` to detect Enter key

### Testing
11 comprehensive tests covering:
- ✓ Simple numbers
- ✓ All four operations (+, -, *, /)
- ✓ Order of operations
- ✓ Complex expressions with multiple operators
- ✓ Whitespace handling
- ✓ Division by zero error handling
- ✓ Negative result error handling
- ✓ Invalid expression error handling

All 115 total tests pass successfully.

## UI Updates
- Added new tip in the Truncate Bits editor: "• You can use math: 8*8, 100+50, 200-10, 64/2"
- Expression evaluation happens automatically when Enter is pressed
- No need to click any button - just type and press Enter

## Build Status
- **Compilation**: ✓ Success (4.84s)
- **Warnings**: 0
- **Tests**: 115 passed, 0 failed
- **Release Build**: ✓ Ready

## Benefits
- **Faster Workflow**: Calculate bit ranges directly without using a calculator
- **Fewer Errors**: Automatic calculation reduces manual entry mistakes
- **Intuitive**: Works exactly as expected - type math, press Enter, get result
- **Flexible**: Supports complex expressions with multiple operations

## Example Use Cases
1. **Calculate byte boundaries**: `8*8` for 64 bits (8 bytes)
2. **Add ranges**: `100+150` to calculate end position
3. **Split data**: `1024/2` to find midpoint
4. **Offset calculations**: `256-10` for adjusted positions
