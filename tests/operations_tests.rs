use bit::processing::{Operation, OperationSequence, BitOperation};
use bitvec::prelude::*;

#[cfg(test)]
mod operation_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_single_operation() {
        let seq = OperationSequence::from_string("t10").unwrap();
        assert_eq!(seq.operations.len(), 1);
        assert_eq!(seq.operations[0], Operation::Take(10));
    }

    #[test]
    fn test_parse_multiple_operations() {
        let seq = OperationSequence::from_string("t4s2r3i1").unwrap();
        assert_eq!(seq.operations.len(), 4);
    }

    #[test]
    fn test_parse_large_numbers() {
        let seq = OperationSequence::from_string("t1234s5678").unwrap();
        assert_eq!(seq.operations.len(), 2);
        assert_eq!(seq.operations[0], Operation::Take(1234));
        assert_eq!(seq.operations[1], Operation::Skip(5678));
    }

    #[test]
    fn test_parse_uppercase() {
        let seq = OperationSequence::from_string("T10S5R3I2").unwrap();
        assert_eq!(seq.operations.len(), 4);
    }

    #[test]
    fn test_parse_mixed_case() {
        let seq = OperationSequence::from_string("t10S5r3I2").unwrap();
        assert_eq!(seq.operations.len(), 4);
    }

    #[test]
    fn test_parse_error_no_number() {
        let result = OperationSequence::from_string("t");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected number"));
    }

    #[test]
    fn test_parse_error_invalid_operation() {
        let result = OperationSequence::from_string("x10");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown operation"));
    }

    #[test]
    fn test_parse_error_invalid_number() {
        let result = OperationSequence::from_string("t99999999999999999999");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_string() {
        let seq = OperationSequence::from_string("").unwrap();
        assert_eq!(seq.operations.len(), 0);
    }
}

#[cfg(test)]
mod take_operation_tests {
    use super::*;

    #[test]
    fn test_take_all() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let seq = OperationSequence::from_string("t4").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_take_partial() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1];
        let seq = OperationSequence::from_string("t3").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1]);
    }

    #[test]
    fn test_take_more_than_available() {
        let input = bitvec![u8, Msb0; 1, 0];
        let seq = OperationSequence::from_string("t10").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 0]);
    }

    #[test]
    fn test_take_zero() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let seq = OperationSequence::from_string("t0").unwrap();
        let result = seq.apply(&input);
        // Zero-sized operations should be skipped to prevent infinite loops
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_take_repeating() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1, 0, 0];
        let seq = OperationSequence::from_string("t2").unwrap();
        let result = seq.apply(&input);
        // Takes 2, then 2, then 2, then 2
        assert_eq!(result, input);
    }
}

#[cfg(test)]
mod skip_operation_tests {
    use super::*;

    #[test]
    fn test_skip_some() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let seq = OperationSequence::from_string("s2t2").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 0]);
    }

    #[test]
    fn test_skip_all() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let seq = OperationSequence::from_string("s10").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_alternating_take_skip() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0, 1, 0];
        let seq = OperationSequence::from_string("t1s1").unwrap();
        let result = seq.apply(&input);
        // Takes bit 0, skips bit 1, takes bit 2, skips bit 3...
        assert_eq!(result, bitvec![u8, Msb0; 1, 1, 1, 1]);
    }
}

#[cfg(test)]
mod reverse_operation_tests {
    use super::*;

    #[test]
    fn test_reverse_all() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 1];
        let seq = OperationSequence::from_string("r4").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 1, 0, 1]);
    }

    #[test]
    fn test_reverse_partial() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0];
        let seq = OperationSequence::from_string("r3t3").unwrap();
        let result = seq.apply(&input);
        // Reverse first 3: 101 -> 101
        // Take next 3: 100
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0]);
    }

    #[test]
    fn test_reverse_single_bit() {
        let input = bitvec![u8, Msb0; 1];
        let seq = OperationSequence::from_string("r1").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1]);
    }
}

#[cfg(test)]
mod invert_operation_tests {
    use super::*;

    #[test]
    fn test_invert_all() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let seq = OperationSequence::from_string("i4").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 0, 1, 0, 1]);
    }

    #[test]
    fn test_invert_partial() {
        let input = bitvec![u8, Msb0; 1, 1, 0, 0];
        let seq = OperationSequence::from_string("i2t2").unwrap();
        let result = seq.apply(&input);
        // Invert first 2: 11 -> 00
        // Take next 2: 00
        assert_eq!(result, bitvec![u8, Msb0; 0, 0, 0, 0]);
    }

    #[test]
    fn test_double_invert() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1, 0, 0];
        let seq = OperationSequence::from_string("i4i4").unwrap();
        let result = seq.apply(&input);
        // i4: invert first 4 bits (1010 -> 0101)
        // i4: invert next 4 bits (1100 -> 0011)
        // Result: 0101 0011
        assert_eq!(result, bitvec![u8, Msb0; 0, 1, 0, 1, 0, 0, 1, 1]);
    }
}

#[cfg(test)]
mod complex_sequence_tests {
    use super::*;

    #[test]
    fn test_extract_every_other_bit() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0, 1, 0];
        let seq = OperationSequence::from_string("t1s1").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 1, 1, 1]);
    }

    #[test]
    fn test_skip_every_other_bit() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0, 1, 0];
        let seq = OperationSequence::from_string("s1t1").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 0, 0, 0, 0]);
    }

    #[test]
    fn test_take_reverse_pattern() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0, 1, 0];
        let seq = OperationSequence::from_string("t4r4").unwrap();
        let result = seq.apply(&input);
        // Take 4: 1011, Reverse 4: 0010 -> 0100
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 1, 0, 1, 0, 0]);
    }

    #[test]
    fn test_manchester_encoding_extraction() {
        // Simulate extracting data from Manchester encoding (take first half of each pair)
        let input = bitvec![u8, Msb0; 1, 0, 0, 1, 1, 0, 0, 1];
        let seq = OperationSequence::from_string("t1s1").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 0]);
    }

    #[test]
    fn test_empty_input() {
        let input = BitVec::<u8, Msb0>::new();
        let seq = OperationSequence::from_string("t4s2r1").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_all_operations_combined() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0];
        let seq = OperationSequence::from_string("t2i2r2s1").unwrap();
        let result = seq.apply(&input);
        
        // First iteration:
        // t2: 10
        // i2: 01 (inverted) -> result: 1001
        // r2: 10 (from 01, reversed) -> result: 100110
        // s1: skip 1
        // Second iteration (position 7):
        // t2: 01
        // i2: 11 (from 11, inverted to 00) -> result: 10011001
        // r2: 10 (from last 10 reversed to 01) -> would need more bits
        
        assert!(result.len() > 0);
    }
}

#[cfg(test)]
mod bit_operation_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_invert_bits_operation() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let op = BitOperation::InvertBits {
            name: "test".to_string(),
            enabled: true,
        };
        let result = op.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 0, 1, 0, 1]);
    }

    #[test]
    fn test_take_skip_sequence_operation() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1];
        let seq = OperationSequence::from_string("t2s1").unwrap();
        let op = BitOperation::TakeSkipSequence {
            name: "test".to_string(),
            sequence: seq,
            enabled: true,
        };
        let result = op.apply(&input);
        // t2: 10, s1: skip, t2: 01, s1: skip (end)
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 0, 1]);
    }

    #[test]
    fn test_operation_name() {
        let op = BitOperation::InvertBits {
            name: "MyInvert".to_string(),
            enabled: true,
        };
        assert_eq!(op.name(), "MyInvert");
    }

    #[test]
    fn test_operation_description() {
        let op = BitOperation::InvertBits {
            name: "test".to_string(),
            enabled: true,
        };
        assert_eq!(op.description(), "Inverts all bits");
    }

    #[test]
    fn test_load_file_description() {
        let op = BitOperation::LoadFile {
            name: "test".to_string(),
            file_path: PathBuf::from("test.bin"),
            enabled: true,
        };
        assert!(op.description().contains("test.bin"));
    }

    #[test]
    fn test_truncate_bits_basic() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1, 0, 0, 1, 1];
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 0,
            end: 5,
            enabled: true,
        };
        let result = op.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 0, 1]);
    }

    #[test]
    fn test_truncate_bits_middle_range() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1, 0, 0, 1, 1];
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 3,
            end: 7,
            enabled: true,
        };
        let result = op.apply(&input);
        // Bits 3-6 (indices 3,4,5,6)
        assert_eq!(result, bitvec![u8, Msb0; 0, 1, 1, 0]);
    }

    #[test]
    fn test_truncate_bits_to_end() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 1, 0, 0, 1, 1];
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 5,
            end: usize::MAX,
            enabled: true,
        };
        let result = op.apply(&input);
        // From index 5 to end
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 0, 1, 1]);
    }

    #[test]
    fn test_truncate_bits_beyond_length() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 0,
            end: 100,
            enabled: true,
        };
        let result = op.apply(&input);
        // Should clamp to actual length
        assert_eq!(result, input);
    }

    #[test]
    fn test_truncate_bits_start_beyond_length() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 10,
            end: 20,
            enabled: true,
        };
        let result = op.apply(&input);
        // start >= end after clamping, should return empty
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_truncate_bits_start_equals_end() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 2,
            end: 2,
            enabled: true,
        };
        let result = op.apply(&input);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_truncate_bits_single_bit() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1];
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 2,
            end: 3,
            enabled: true,
        };
        let result = op.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1]);
    }

    #[test]
    fn test_truncate_bits_description() {
        let op = BitOperation::TruncateBits {
            name: "test".to_string(),
            start: 100,
            end: 250,
            enabled: true,
        };
        assert_eq!(op.description(), "Keep bits 100-250");
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_operation_beyond_input_length() {
        let input = bitvec![u8, Msb0; 1, 0];
        let seq = OperationSequence::from_string("t100").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_skip_beyond_input_length() {
        let input = bitvec![u8, Msb0; 1, 0];
        let seq = OperationSequence::from_string("s100t1").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_sequence_on_single_bit() {
        let input = bitvec![u8, Msb0; 1];
        let seq = OperationSequence::from_string("t1").unwrap();
        let result = seq.apply(&input);
        // t1 on single bit: takes 1 bit, then no more bits left
        assert_eq!(result, bitvec![u8, Msb0; 1]);
    }

    #[test]
    fn test_repeating_pattern_extraction() {
        // Create repeating pattern: 10101010
        let input = bitvec![u8, Msb0; 1, 0, 1, 0, 1, 0, 1, 0];
        let seq = OperationSequence::from_string("t1s1").unwrap();
        let result = seq.apply(&input);
        // Should extract all 1s
        assert_eq!(result, bitvec![u8, Msb0; 1, 1, 1, 1]);
    }
}
