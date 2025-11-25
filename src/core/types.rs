// Core types used throughout the application

use serde::{Deserialize, Serialize};
use egui;

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

/// Operation categories for visual grouping
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationCategory {
    Loading,
    Transformation,
    Analysis,
}

/// Available operation types that can be added
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    LoadFile,
    TakeSkipSequence,
    InvertBits,
    MultiWorksheetLoad,
    TruncateBits,
    InterleaveBits,
}

impl OperationType {
    pub fn name(&self) -> &str {
        match self {
            OperationType::LoadFile => "Load File",
            OperationType::TakeSkipSequence => "Take/Skip Sequence",
            OperationType::InvertBits => "Invert Bits",
            OperationType::MultiWorksheetLoad => "Multi-Worksheet Load",
            OperationType::TruncateBits => "Truncate Bits",
            OperationType::InterleaveBits => "Interleave Bits",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            OperationType::LoadFile => "📂",
            OperationType::TakeSkipSequence => "📝",
            OperationType::InvertBits => "🔄",
            OperationType::MultiWorksheetLoad => "📚",
            OperationType::TruncateBits => "✂️",
            OperationType::InterleaveBits => "🔀",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            OperationType::LoadFile => "Load bits from a file",
            OperationType::TakeSkipSequence => "Pattern-based bit extraction (t4r3i8s1)",
            OperationType::InvertBits => "Invert all bits (0→1, 1→0)",
            OperationType::MultiWorksheetLoad => "Load bits from multiple worksheets with operations",
            OperationType::TruncateBits => "Keep bits in a range and discard the rest",
            OperationType::InterleaveBits => "Interleave/de-interleave bits for error resilience",
        }
    }

    pub fn category(&self) -> OperationCategory {
        match self {
            OperationType::LoadFile => OperationCategory::Loading,
            OperationType::MultiWorksheetLoad => OperationCategory::Loading,
            OperationType::TakeSkipSequence => OperationCategory::Transformation,
            OperationType::InvertBits => OperationCategory::Transformation,
            OperationType::TruncateBits => OperationCategory::Transformation,
            OperationType::InterleaveBits => OperationCategory::Transformation,
        }
    }
}

impl OperationCategory {
    /// Returns the color for this category (RGB)
    pub fn color(&self) -> egui::Color32 {
        match self {
            OperationCategory::Loading => egui::Color32::from_rgb(100, 150, 255),     // Blue
            OperationCategory::Transformation => egui::Color32::from_rgb(150, 100, 255), // Purple
            OperationCategory::Analysis => egui::Color32::from_rgb(100, 255, 150),    // Green
        }
    }

    pub fn name(&self) -> &str {
        match self {
            OperationCategory::Loading => "Loading",
            OperationCategory::Transformation => "Transformation",
            OperationCategory::Analysis => "Analysis",
        }
    }
}
