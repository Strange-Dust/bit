use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Take(usize),
    Reverse(usize),
    Invert(usize),
    Skip(usize),
}

impl Operation {
    pub fn to_string(&self) -> String {
        match self {
            Operation::Take(n) => format!("t{}", n),
            Operation::Reverse(n) => format!("r{}", n),
            Operation::Invert(n) => format!("i{}", n),
            Operation::Skip(n) => format!("s{}", n),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSequence {
    pub operations: Vec<Operation>,
}

// Represents a complete operation that can be applied to bits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BitOperation {
    TakeSkipSequence {
        name: String,
        sequence: OperationSequence,
    },
    InvertBits {
        name: String,
    },
    MultiWorksheetLoad {
        name: String,
        worksheet_operations: Vec<WorksheetOperation>,
    },
    // Future operations:
    // FindPattern { name: String, pattern: String, highlight: bool },
    // Replace { name: String, from_pattern: String, to_pattern: String },
    // etc.
}

/// Represents a take/skip operation to apply to a specific worksheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetOperation {
    pub worksheet_index: usize,
    pub sequence: OperationSequence,
}

impl BitOperation {
    pub fn name(&self) -> &str {
        match self {
            BitOperation::TakeSkipSequence { name, .. } => name,
            BitOperation::InvertBits { name } => name,
            BitOperation::MultiWorksheetLoad { name, .. } => name,
        }
    }

    pub fn description(&self) -> String {
        match self {
            BitOperation::TakeSkipSequence { sequence, .. } => sequence.to_string(),
            BitOperation::InvertBits { .. } => "Inverts all bits".to_string(),
            BitOperation::MultiWorksheetLoad { worksheet_operations, .. } => {
                format!("Load from {} worksheet(s)", worksheet_operations.len())
            }
        }
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self {
            BitOperation::TakeSkipSequence { sequence, .. } => sequence.apply(input),
            BitOperation::InvertBits { .. } => {
                let mut result = input.clone();
                result.iter_mut().for_each(|mut bit| *bit = !*bit);
                result
            }
            BitOperation::MultiWorksheetLoad { .. } => {
                // This operation type requires worksheet data, so it should be handled
                // differently in the main application. For now, return empty.
                BitVec::new()
            }
        }
    }
}

impl OperationSequence {
    pub fn from_string(s: &str) -> Result<Self, String> {
        let mut operations = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            let op_type = c;
            let mut num_str = String::new();

            // Read the number following the operation character
            while let Some(&next_char) = chars.peek() {
                if next_char.is_numeric() {
                    num_str.push(next_char);
                    chars.next();
                } else {
                    break;
                }
            }

            if num_str.is_empty() {
                return Err(format!("Expected number after '{}'", op_type));
            }

            let num: usize = num_str
                .parse()
                .map_err(|_| format!("Invalid number: {}", num_str))?;

            let operation = match op_type {
                't' | 'T' => Operation::Take(num),
                'r' | 'R' => Operation::Reverse(num),
                'i' | 'I' => Operation::Invert(num),
                's' | 'S' => Operation::Skip(num),
                _ => return Err(format!("Unknown operation: {}", op_type)),
            };

            operations.push(operation);
        }

        Ok(Self { operations })
    }

    pub fn to_string(&self) -> String {
        self.operations
            .iter()
            .map(|op| op.to_string())
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let mut result = BitVec::new();
        let mut pos = 0;

        while pos < input.len() {
            for operation in &self.operations {
                if pos >= input.len() {
                    break;
                }

                match operation {
                    Operation::Take(n) => {
                        let end = (pos + n).min(input.len());
                        result.extend_from_bitslice(&input[pos..end]);
                        pos = end;
                    }
                    Operation::Reverse(n) => {
                        let end = (pos + n).min(input.len());
                        let mut reversed: BitVec<u8, Msb0> = input[pos..end].iter().collect();
                        reversed.reverse();
                        result.extend_from_bitslice(&reversed);
                        pos = end;
                    }
                    Operation::Invert(n) => {
                        let end = (pos + n).min(input.len());
                        for bit in &input[pos..end] {
                            result.push(!*bit);
                        }
                        pos = end;
                    }
                    Operation::Skip(n) => {
                        pos = (pos + n).min(input.len());
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_operation_sequence() {
        let seq = OperationSequence::from_string("t4r3i8s1").unwrap();
        assert_eq!(seq.operations.len(), 4);
        assert_eq!(seq.operations[0], Operation::Take(4));
        assert_eq!(seq.operations[1], Operation::Reverse(3));
        assert_eq!(seq.operations[2], Operation::Invert(8));
        assert_eq!(seq.operations[3], Operation::Skip(1));
    }

    #[test]
    fn test_operation_sequence_to_string() {
        let seq = OperationSequence::from_string("t4r3i8s1").unwrap();
        assert_eq!(seq.to_string(), "t4r3i8s1");
    }

    #[test]
    fn test_take_operation() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0, 1, 0];
        let seq = OperationSequence::from_string("t4").unwrap();
        let result = seq.apply(&input);
        // t4 repeats: take 4, then take 4 again
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0, 1, 0]);
    }

    #[test]
    fn test_reverse_operation() {
        let input = bitvec![u8, Msb0; 1, 1, 0];
        let seq = OperationSequence::from_string("r3").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 0, 1, 1]);
    }

    #[test]
    fn test_invert_operation() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 0];
        let seq = OperationSequence::from_string("i4").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 0, 1, 0, 1]);
    }

    #[test]
    fn test_skip_operation() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0];
        let seq = OperationSequence::from_string("t2s2t2").unwrap();
        let result = seq.apply(&input);
        assert_eq!(result, bitvec![u8, Msb0; 1, 0, 0, 0]);
    }

    #[test]
    fn test_complex_sequence() {
        let input = bitvec![u8, Msb0; 1, 0, 1, 1, 0, 0, 1, 0, 1, 1, 1, 0];
        let seq = OperationSequence::from_string("t4r3i2s1").unwrap();
        let result = seq.apply(&input);
        
        // t4: take first 4 bits -> 1011
        // r3: reverse next 3 bits (001) -> 100
        // i2: invert next 2 bits (01) -> 10
        // s1: skip 1 bit (skip 1)
        // Then repeat...
        // t4: take next 4 bits (10) -> only 2 bits left, so 10
        
        let expected = bitvec![u8, Msb0; 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0];
        assert_eq!(result, expected);
    }
}
