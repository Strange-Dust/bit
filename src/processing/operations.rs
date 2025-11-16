use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::interleaver::{BlockInterleaverConfig, ConvolutionalInterleaverConfig, SymbolInterleaverConfig, InterleaverType};

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
    LoadFile {
        name: String,
        file_path: PathBuf,
        enabled: bool,
    },
    TakeSkipSequence {
        name: String,
        sequence: OperationSequence,
        enabled: bool,
    },
    InvertBits {
        name: String,
        enabled: bool,
    },
    MultiWorksheetLoad {
        name: String,
        worksheet_operations: Vec<WorksheetOperation>,
        enabled: bool,
    },
    TruncateBits {
        name: String,
        start: usize,
        end: usize,
        enabled: bool,
    },
    InterleaveBits {
        name: String,
        interleaver_type: InterleaverType,
        block_config: Option<BlockInterleaverConfig>,
        convolutional_config: Option<ConvolutionalInterleaverConfig>,
        symbol_config: Option<SymbolInterleaverConfig>,
        enabled: bool,
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
            BitOperation::LoadFile { name, .. } => name,
            BitOperation::TakeSkipSequence { name, .. } => name,
            BitOperation::InvertBits { name, .. } => name,
            BitOperation::MultiWorksheetLoad { name, .. } => name,
            BitOperation::TruncateBits { name, .. } => name,
            BitOperation::InterleaveBits { name, .. } => name,
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        match self {
            BitOperation::LoadFile { enabled, .. } => *enabled,
            BitOperation::TakeSkipSequence { enabled, .. } => *enabled,
            BitOperation::InvertBits { enabled, .. } => *enabled,
            BitOperation::MultiWorksheetLoad { enabled, .. } => *enabled,
            BitOperation::TruncateBits { enabled, .. } => *enabled,
            BitOperation::InterleaveBits { enabled, .. } => *enabled,
        }
    }
    
    pub fn set_enabled(&mut self, new_enabled: bool) {
        match self {
            BitOperation::LoadFile { enabled, .. } => *enabled = new_enabled,
            BitOperation::TakeSkipSequence { enabled, .. } => *enabled = new_enabled,
            BitOperation::InvertBits { enabled, .. } => *enabled = new_enabled,
            BitOperation::MultiWorksheetLoad { enabled, .. } => *enabled = new_enabled,
            BitOperation::TruncateBits { enabled, .. } => *enabled = new_enabled,
            BitOperation::InterleaveBits { enabled, .. } => *enabled = new_enabled,
        }
    }

    pub fn description(&self) -> String {
        match self {
            BitOperation::LoadFile { file_path, .. } => {
                format!("Load: {}", file_path.file_name().unwrap_or_default().to_string_lossy())
            }
            BitOperation::TakeSkipSequence { sequence, .. } => sequence.to_string(),
            BitOperation::InvertBits { .. } => "Inverts all bits".to_string(),
            BitOperation::MultiWorksheetLoad { worksheet_operations, .. } => {
                format!("Load from {} worksheet(s)", worksheet_operations.len())
            }
            BitOperation::TruncateBits { start, end, .. } => {
                format!("Keep bits {}-{}", start, end)
            }
            BitOperation::InterleaveBits { interleaver_type, block_config, convolutional_config, symbol_config, .. } => {
                match interleaver_type {
                    InterleaverType::Block => {
                        if let Some(cfg) = block_config {
                            let dir = match cfg.direction {
                                crate::processing::InterleaverDirection::Interleave => "Interleave",
                                crate::processing::InterleaverDirection::Deinterleave => "Deinterleave",
                            };
                            format!("Block {}×{} {}", cfg.block_size, cfg.depth, dir)
                        } else {
                            "Block interleaver".to_string()
                        }
                    }
                    InterleaverType::Convolutional => {
                        if let Some(cfg) = convolutional_config {
                            let dir = match cfg.direction {
                                crate::processing::InterleaverDirection::Interleave => "Interleave",
                                crate::processing::InterleaverDirection::Deinterleave => "Deinterleave",
                            };
                            format!("Conv B={} M={} {}", cfg.branches, cfg.delay_increment, dir)
                        } else {
                            "Convolutional interleaver".to_string()
                        }
                    }
                    InterleaverType::Symbol => {
                        if let Some(cfg) = symbol_config {
                            let dir = match cfg.direction {
                                crate::processing::InterleaverDirection::Interleave => "Interleave",
                                crate::processing::InterleaverDirection::Deinterleave => "Deinterleave",
                            };
                            format!("Symbol {}×{} ({}bit) {}", cfg.block_size, cfg.depth, cfg.symbol_size, dir)
                        } else {
                            "Symbol interleaver".to_string()
                        }
                    }
                }
            }
        }
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self {
            BitOperation::LoadFile { .. } => {
                // LoadFile operations are handled specially in the main application
                // since they need file I/O. Return the input unchanged here.
                input.clone()
            }
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
            BitOperation::TruncateBits { start, end, .. } => {
                let len = input.len();
                let actual_start = (*start).min(len);
                let actual_end = (*end).min(len);
                
                if actual_start >= actual_end {
                    return BitVec::new();
                }
                
                input[actual_start..actual_end].to_bitvec()
            }
            BitOperation::InterleaveBits { interleaver_type, block_config, convolutional_config, symbol_config, .. } => {
                match interleaver_type {
                    InterleaverType::Block => {
                        if let Some(cfg) = block_config {
                            cfg.apply(input)
                        } else {
                            input.clone()
                        }
                    }
                    InterleaverType::Convolutional => {
                        if let Some(cfg) = convolutional_config {
                            cfg.apply(input)
                        } else {
                            input.clone()
                        }
                    }
                    InterleaverType::Symbol => {
                        if let Some(cfg) = symbol_config {
                            cfg.apply(input)
                        } else {
                            input.clone()
                        }
                    }
                }
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
            let start_pos = pos;
            
            for operation in &self.operations {
                if pos >= input.len() {
                    break;
                }

                match operation {
                    Operation::Take(n) => {
                        if *n == 0 {
                            // Skip zero-sized operations to prevent infinite loops
                            continue;
                        }
                        let end = (pos + n).min(input.len());
                        result.extend_from_bitslice(&input[pos..end]);
                        pos = end;
                    }
                    Operation::Reverse(n) => {
                        if *n == 0 {
                            continue;
                        }
                        let end = (pos + n).min(input.len());
                        let mut reversed: BitVec<u8, Msb0> = input[pos..end].iter().collect();
                        reversed.reverse();
                        result.extend_from_bitslice(&reversed);
                        pos = end;
                    }
                    Operation::Invert(n) => {
                        if *n == 0 {
                            continue;
                        }
                        let end = (pos + n).min(input.len());
                        for bit in &input[pos..end] {
                            result.push(!*bit);
                        }
                        pos = end;
                    }
                    Operation::Skip(n) => {
                        if *n == 0 {
                            continue;
                        }
                        pos = (pos + n).min(input.len());
                    }
                }
            }
            
            // Prevent infinite loop if no progress was made
            if pos == start_pos {
                break;
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
