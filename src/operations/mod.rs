// Operations module - bit manipulation operations organized by type

pub mod base;
pub mod interleaver;
pub mod load_file;
pub mod take_skip_sequence;
pub mod invert;
pub mod multi_worksheet;
pub mod truncate;
pub mod interleave;

// Re-export base types
pub use base::OperationSequence;

// Re-export interleaver types
pub use interleaver::{
    BlockInterleaverConfig, ConvolutionalInterleaverConfig,
    InterleaverDirection, InterleaverType, SymbolInterleaverConfig,
};

// Re-export worksheet operation
pub use multi_worksheet::WorksheetOperation;

use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

/// Unified operation enum - maintains backward compatibility with named fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BitOperation {
    LoadFile {
        name: String,
        file_path: std::path::PathBuf,
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
                                InterleaverDirection::Interleave => "Interleave",
                                InterleaverDirection::Deinterleave => "Deinterleave",
                            };
                            format!("Block {}×{} {}", cfg.block_size, cfg.depth, dir)
                        } else {
                            "Block interleaver".to_string()
                        }
                    }
                    InterleaverType::Convolutional => {
                        if let Some(cfg) = convolutional_config {
                            let dir = match cfg.direction {
                                InterleaverDirection::Interleave => "Interleave",
                                InterleaverDirection::Deinterleave => "Deinterleave",
                            };
                            format!("Conv B={} M={} {}", cfg.branches, cfg.delay_increment, dir)
                        } else {
                            "Convolutional interleaver".to_string()
                        }
                    }
                    InterleaverType::Symbol => {
                        if let Some(cfg) = symbol_config {
                            let dir = match cfg.direction {
                                InterleaverDirection::Interleave => "Interleave",
                                InterleaverDirection::Deinterleave => "Deinterleave",
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
                input.clone()
            }
            BitOperation::TakeSkipSequence { sequence, .. } => sequence.apply(input),
            BitOperation::InvertBits { .. } => {
                let mut result = input.clone();
                result.iter_mut().for_each(|mut bit| *bit = !*bit);
                result
            }
            BitOperation::MultiWorksheetLoad { .. } => {
                // This operation type requires worksheet data
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
