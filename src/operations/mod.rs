// Operations module - bit manipulation operations organized by type

pub mod base;
pub mod interleaver;
pub mod load_file;
pub mod take_skip_sequence;
pub mod invert;
pub mod multi_worksheet;
pub mod truncate;

// Re-export base types
pub use base::OperationSequence;

// Re-export interleaver types
pub use interleaver::{
    BlockInterleaverConfig, ConvolutionalInterleaverConfig,
    InterleaverDirection, InterleaverType, SymbolInterleaverConfig,
};

// Re-export worksheet operation
pub use multi_worksheet::WorksheetOperation;

// Import operation implementations for internal use
use load_file::LoadFileOperation;
use take_skip_sequence::TakeSkipSequenceOperation;
use invert::InvertBitsOperation;
use multi_worksheet::MultiWorksheetLoadOperation;
use truncate::TruncateBitsOperation;
use interleaver::InterleaveBitsOperation;

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
                LoadFileOperation { 
                    name: String::new(), 
                    file_path: file_path.clone(), 
                    enabled: true 
                }.description()
            }
            BitOperation::TakeSkipSequence { sequence, .. } => {
                TakeSkipSequenceOperation {
                    name: String::new(),
                    sequence: sequence.clone(),
                    enabled: true,
                }.description()
            }
            BitOperation::InvertBits { .. } => {
                InvertBitsOperation {
                    name: String::new(),
                    enabled: true,
                }.description()
            }
            BitOperation::MultiWorksheetLoad { worksheet_operations, .. } => {
                MultiWorksheetLoadOperation {
                    name: String::new(),
                    worksheet_operations: worksheet_operations.clone(),
                    enabled: true,
                }.description()
            }
            BitOperation::TruncateBits { start, end, .. } => {
                TruncateBitsOperation {
                    name: String::new(),
                    start: *start,
                    end: *end,
                    enabled: true,
                }.description()
            }
            BitOperation::InterleaveBits { interleaver_type, block_config, convolutional_config, symbol_config, .. } => {
                InterleaveBitsOperation {
                    name: String::new(),
                    interleaver_type: *interleaver_type,
                    block_config: block_config.clone(),
                    convolutional_config: convolutional_config.clone(),
                    symbol_config: symbol_config.clone(),
                    enabled: true,
                }.description()
            }
        }
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self {
            BitOperation::LoadFile { file_path, .. } => {
                LoadFileOperation {
                    name: String::new(),
                    file_path: file_path.clone(),
                    enabled: true,
                }.apply(input)
            }
            BitOperation::TakeSkipSequence { sequence, .. } => {
                TakeSkipSequenceOperation {
                    name: String::new(),
                    sequence: sequence.clone(),
                    enabled: true,
                }.apply(input)
            }
            BitOperation::InvertBits { .. } => {
                InvertBitsOperation {
                    name: String::new(),
                    enabled: true,
                }.apply(input)
            }
            BitOperation::MultiWorksheetLoad { worksheet_operations, .. } => {
                MultiWorksheetLoadOperation {
                    name: String::new(),
                    worksheet_operations: worksheet_operations.clone(),
                    enabled: true,
                }.apply(input)
            }
            BitOperation::TruncateBits { start, end, .. } => {
                TruncateBitsOperation {
                    name: String::new(),
                    start: *start,
                    end: *end,
                    enabled: true,
                }.apply(input)
            }
            BitOperation::InterleaveBits { interleaver_type, block_config, convolutional_config, symbol_config, .. } => {
                InterleaveBitsOperation {
                    name: String::new(),
                    interleaver_type: *interleaver_type,
                    block_config: block_config.clone(),
                    convolutional_config: convolutional_config.clone(),
                    symbol_config: symbol_config.clone(),
                    enabled: true,
                }.apply(input)
            }
        }
    }
}
