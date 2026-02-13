// Operations module - bit manipulation operations organized by type
//
// Each operation is entirely self-contained in its own file, including its
// editor UI. To add a new operation:
// 1. Create a new module file with the operation struct, apply() logic, and editor
// 2. Add the operation to `register_operations!`, `BitOperation` enum, and
//    `impl_bit_operation_delegating!` below (~10 lines total)
// 3. Add the editor variant to `OperationEditorState` in editor.rs (~5 lines)

pub mod base;
pub mod editor;
pub mod exclude;
pub mod interleaver;
pub mod isolate;
pub mod load_file;
pub mod take_skip_sequence;
pub mod invert;
pub mod multi_worksheet;
pub mod traits;
pub mod truncate;

// Re-export base types
pub use base::OperationSequence;

// Re-export trait types
pub use traits::{EditorAction, EditorContext};

// Re-export editor state
pub use editor::OperationEditorState;

// Re-export interleaver types
pub use interleaver::{
    BlockInterleaverConfig, InterleaverDirection, InterleaverType,
};

// Import operation implementations
use load_file::LoadFileOperation;
use take_skip_sequence::TakeSkipSequenceOperation;
use invert::InvertBitsOperation;
use multi_worksheet::MultiWorksheetLoadOperation;
use truncate::TruncateBitsOperation;
use interleaver::InterleaveBitsOperation;
use isolate::IsolateBitsOperation;
use exclude::ExcludeBitsOperation;

use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

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
        icon: "",
        description: "Load bits from a file",
        category: Loading,
    },
    TakeSkipSequence {
        display_name: "Take/Skip Sequence",
        icon: "",
        description: "Pattern-based bit extraction (t4r3i8s1)",
        category: Transformation,
    },
    InvertBits {
        display_name: "Invert Bits",
        icon: "",
        description: "Invert all bits (0→1, 1→0)",
        category: Transformation,
    },
    MultiWorksheetLoad {
        display_name: "Multi-Worksheet Load",
        icon: "",
        description: "Load bits from multiple worksheets with operations",
        category: Loading,
    },
    TruncateBits {
        display_name: "Truncate Bits",
        icon: "",
        description: "Keep bits in a range and discard the rest",
        category: Transformation,
    },
    InterleaveBits {
        display_name: "Interleave Bits",
        icon: "",
        description: "Interleave/de-interleave bits for error resilience",
        category: Transformation,
    },
    IsolateBits {
        display_name: "Isolate Bits",
        icon: "",
        description: "Keep selected columns from selected rows",
        category: Transformation,
    },
    ExcludeBits {
        display_name: "Exclude Bits",
        icon: "",
        description: "Remove selected columns from selected rows",
        category: Transformation,
    },
}

// ============================================================================
// BIT OPERATION ENUM - Newtype variants wrapping standalone structs
// ============================================================================

/// Unified operation enum - each variant wraps its standalone operation struct.
/// Uses `#[serde(tag = "type")]` with `#[serde(flatten)]` fields for
/// backward-compatible JSON serialization (same format as before).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BitOperation {
    LoadFile { #[serde(flatten)] inner: LoadFileOperation },
    TakeSkipSequence { #[serde(flatten)] inner: TakeSkipSequenceOperation },
    InvertBits { #[serde(flatten)] inner: InvertBitsOperation },
    MultiWorksheetLoad { #[serde(flatten)] inner: MultiWorksheetLoadOperation },
    TruncateBits { #[serde(flatten)] inner: TruncateBitsOperation },
    InterleaveBits { #[serde(flatten)] inner: InterleaveBitsOperation },
    IsolateBits { #[serde(flatten)] inner: IsolateBitsOperation },
    ExcludeBits { #[serde(flatten)] inner: ExcludeBitsOperation },
}

// Delegating macro - auto-generates all method dispatch as one-liner match arms
macro_rules! impl_bit_operation_delegating {
    ( $( $variant:ident ),* $(,)? ) => {
        impl BitOperation {
            pub fn name(&self) -> &str {
                match self { $(Self::$variant { inner } => &inner.name),* }
            }
            pub fn is_enabled(&self) -> bool {
                match self { $(Self::$variant { inner } => inner.enabled),* }
            }
            pub fn set_enabled(&mut self, v: bool) {
                match self { $(Self::$variant { inner } => inner.enabled = v),* }
            }
            pub fn operation_type(&self) -> OperationType {
                match self { $(Self::$variant { .. } => OperationType::$variant),* }
            }
            pub fn description(&self) -> String {
                match self { $(Self::$variant { inner } => inner.description()),* }
            }
            pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
                match self { $(Self::$variant { inner } => inner.apply(input)),* }
            }
            pub fn is_source_operation(&self) -> bool {
                match self { $(Self::$variant { inner } => inner.is_source_operation()),* }
            }
        }
    };
}

impl_bit_operation_delegating!(
    LoadFile,
    TakeSkipSequence,
    InvertBits,
    MultiWorksheetLoad,
    TruncateBits,
    InterleaveBits,
    IsolateBits,
    ExcludeBits,
);
