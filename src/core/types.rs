// Core types used throughout the application

use serde::{Deserialize, Serialize};

/// View mode for switching between different bit visualizations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Bit,
    Byte,
    Ascii,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Bit
    }
}

/// Available operation types that can be added
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    LoadFile,
    TakeSkipSequence,
    InvertBits,
    MultiWorksheetLoad,
    TruncateBits,
}

impl OperationType {
    pub fn name(&self) -> &str {
        match self {
            OperationType::LoadFile => "Load File",
            OperationType::TakeSkipSequence => "Take/Skip Sequence",
            OperationType::InvertBits => "Invert Bits",
            OperationType::MultiWorksheetLoad => "Multi-Worksheet Load",
            OperationType::TruncateBits => "Truncate Bits",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            OperationType::LoadFile => "📂",
            OperationType::TakeSkipSequence => "📝",
            OperationType::InvertBits => "🔄",
            OperationType::MultiWorksheetLoad => "📚",
            OperationType::TruncateBits => "✂️",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            OperationType::LoadFile => "Load bits from a file",
            OperationType::TakeSkipSequence => "Pattern-based bit extraction (t4r3i8s1)",
            OperationType::InvertBits => "Invert all bits (0→1, 1→0)",
            OperationType::MultiWorksheetLoad => "Load bits from multiple worksheets with operations",
            OperationType::TruncateBits => "Keep bits in a range and discard the rest",
        }
    }
    
    #[allow(dead_code)]
    pub fn all() -> &'static [OperationType] {
        &[
            OperationType::LoadFile,
            OperationType::TakeSkipSequence,
            OperationType::InvertBits,
            OperationType::MultiWorksheetLoad,
            OperationType::TruncateBits,
        ]
    }
}
