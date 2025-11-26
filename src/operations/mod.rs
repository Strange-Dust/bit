// Operations module - bit manipulation operations organized by type
//
// This module uses a registration macro to reduce boilerplate when adding new operations.
// To add a new operation:
// 1. Create a new module file (e.g., `my_operation.rs`) with the operation struct and apply() logic
// 2. Add the operation to the `register_operations!` macro below
// 3. Add the editor rendering in `operations/editor.rs`
//
// You do NOT need to modify app.rs or ui/windows.rs anymore!

pub mod base;
pub mod editor;
pub mod interleaver;
pub mod load_file;
pub mod take_skip_sequence;
pub mod invert;
pub mod multi_worksheet;
pub mod truncate;

// Re-export base types
pub use base::OperationSequence;

// Re-export editor types
pub use editor::{EditorAction, EditorContext, OperationEditorState, render_operation_editor};

// Re-export interleaver types
pub use interleaver::{
    BlockInterleaverConfig, ConvolutionalInterleaverConfig,
    InterleaverDirection, InterleaverType, SymbolInterleaverConfig,
};

// Re-export worksheet operation
pub use multi_worksheet::WorksheetOperation;

// Import operation implementations
use load_file::LoadFileOperation;
use take_skip_sequence::TakeSkipSequenceOperation;
use invert::InvertBitsOperation;
use multi_worksheet::MultiWorksheetLoadOperation;
use truncate::TruncateBitsOperation;
use interleaver::InterleaveBitsOperation;

use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Operation categories for visual grouping in the UI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationCategory {
    Loading,
    Transformation,
    Analysis,
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

// ============================================================================
// OPERATION REGISTRATION MACRO
// ============================================================================
//
// This macro generates:
// - OperationType enum with name(), icon(), description(), category() methods
// - BitOperation enum with name(), is_enabled(), set_enabled() methods
// - Conversion between OperationType and BitOperation
//
// To add a new operation, add an entry to the macro invocation below.

macro_rules! register_operations {
    (
        $(
            $variant:ident {
                display_name: $display_name:literal,
                icon: $icon:literal,
                description: $desc:literal,
                category: $category:ident,
            }
        ),* $(,)?
    ) => {
        /// Available operation types that can be added
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum OperationType {
            $( $variant ),*
        }

        impl OperationType {
            pub fn name(&self) -> &'static str {
                match self {
                    $( OperationType::$variant => $display_name ),*
                }
            }

            pub fn icon(&self) -> &'static str {
                match self {
                    $( OperationType::$variant => $icon ),*
                }
            }

            pub fn description(&self) -> &'static str {
                match self {
                    $( OperationType::$variant => $desc ),*
                }
            }

            pub fn category(&self) -> OperationCategory {
                match self {
                    $( OperationType::$variant => OperationCategory::$category ),*
                }
            }

            /// Returns all available operation types
            #[allow(dead_code)]
            pub fn all() -> &'static [OperationType] {
                &[ $( OperationType::$variant ),* ]
            }
        }
    };
}

// Register all operations here - this is the single source of truth for operation metadata
register_operations! {
    LoadFile {
        display_name: "Load File",
        icon: "📂",
        description: "Load bits from a file",
        category: Loading,
    },
    TakeSkipSequence {
        display_name: "Take/Skip Sequence",
        icon: "📝",
        description: "Pattern-based bit extraction (t4r3i8s1)",
        category: Transformation,
    },
    InvertBits {
        display_name: "Invert Bits",
        icon: "🔄",
        description: "Invert all bits (0→1, 1→0)",
        category: Transformation,
    },
    MultiWorksheetLoad {
        display_name: "Multi-Worksheet Load",
        icon: "📚",
        description: "Load bits from multiple worksheets with operations",
        category: Loading,
    },
    TruncateBits {
        display_name: "Truncate Bits",
        icon: "✂️",
        description: "Keep bits in a range and discard the rest",
        category: Transformation,
    },
    InterleaveBits {
        display_name: "Interleave Bits",
        icon: "🔀",
        description: "Interleave/de-interleave bits for error resilience",
        category: Transformation,
    },
}

// ============================================================================
// BIT OPERATION ENUM
// ============================================================================
//
// The BitOperation enum stores the actual operation data. Each variant has different
// fields, so we can't easily generate this with a simple macro. However, we use
// a helper macro to reduce the boilerplate for common methods.

/// Unified operation enum - stores operation data and parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
}

// Helper macro for generating common BitOperation methods
macro_rules! impl_bit_operation_common {
    ( $( $variant:ident ),* $(,)? ) => {
        impl BitOperation {
            pub fn name(&self) -> &str {
                match self {
                    $( BitOperation::$variant { name, .. } => name ),*
                }
            }

            pub fn is_enabled(&self) -> bool {
                match self {
                    $( BitOperation::$variant { enabled, .. } => *enabled ),*
                }
            }

            pub fn set_enabled(&mut self, new_enabled: bool) {
                match self {
                    $( BitOperation::$variant { enabled, .. } => *enabled = new_enabled ),*
                }
            }

            /// Get the operation type for this operation
            pub fn operation_type(&self) -> OperationType {
                match self {
                    $( BitOperation::$variant { .. } => OperationType::$variant ),*
                }
            }
        }
    };
}

// Apply the helper macro for common methods
impl_bit_operation_common!(
    LoadFile,
    TakeSkipSequence,
    InvertBits,
    MultiWorksheetLoad,
    TruncateBits,
    InterleaveBits,
);

// These methods have operation-specific logic and can't be easily generated by macro
impl BitOperation {
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

    /// Apply the operation to input bits.
    /// Takes ownership to avoid unnecessary cloning of large data structures.
    ///
    /// Note: Some operations (LoadFile, MultiWorksheetLoad) are handled specially
    /// in app.rs and don't use the input, but we maintain consistent signatures.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        match self {
            BitOperation::LoadFile { file_path, .. } => {
                LoadFileOperation {
                    name: String::new(),
                    file_path: file_path.clone(),
                    enabled: true,
                }.apply(input)
            }
            BitOperation::TakeSkipSequence { sequence, .. } => {
                // TakeSkipSequence needs to borrow input because it iterates through it
                TakeSkipSequenceOperation {
                    name: String::new(),
                    sequence: sequence.clone(),
                    enabled: true,
                }.apply(&input)
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
