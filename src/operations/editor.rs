// Operation editor state - thin enum wrapping per-operation editors
//
// Each operation's editor UI is self-contained in its own file.
// This module just provides the OperationEditorState enum that dispatches
// to the appropriate editor.

use super::traits::{EditorAction, EditorContext, OperationEditor};
use super::{BitOperation, OperationType};
use eframe::egui;

use super::load_file::LoadFileEditor;
use super::take_skip_sequence::TakeSkipEditor;
use super::invert::InvertBitsEditor;
use super::multi_worksheet::MultiWorksheetEditor;
use super::truncate::TruncateEditor;
use super::interleaver::InterleaveEditor;
use super::isolate::IsolateEditor;
use super::exclude::ExcludeEditor;

/// Editor state enum - each variant wraps a per-operation editor
#[derive(Debug, Clone)]
pub enum OperationEditorState {
    LoadFile(LoadFileEditor),
    TakeSkipSequence(TakeSkipEditor),
    InvertBits(InvertBitsEditor),
    MultiWorksheetLoad(MultiWorksheetEditor),
    TruncateBits(TruncateEditor),
    InterleaveBits(InterleaveEditor),
    IsolateBits(IsolateEditor),
    ExcludeBits(ExcludeEditor),
}

impl OperationEditorState {
    /// Create editor state for a new operation of the given type
    pub fn new_for_type(op_type: OperationType) -> Self {
        match op_type {
            OperationType::LoadFile => Self::LoadFile(LoadFileEditor::default()),
            OperationType::TakeSkipSequence => Self::TakeSkipSequence(TakeSkipEditor::default()),
            OperationType::InvertBits => Self::InvertBits(InvertBitsEditor::default()),
            OperationType::MultiWorksheetLoad => Self::MultiWorksheetLoad(MultiWorksheetEditor::default()),
            OperationType::TruncateBits => Self::TruncateBits(TruncateEditor::default()),
            OperationType::InterleaveBits => Self::InterleaveBits(InterleaveEditor::default()),
            OperationType::IsolateBits => Self::IsolateBits(IsolateEditor::default()),
            OperationType::ExcludeBits => Self::ExcludeBits(ExcludeEditor::default()),
        }
    }

    /// Create editor state from an existing operation (for editing)
    pub fn from_operation(op: &BitOperation) -> Self {
        match op {
            BitOperation::LoadFile { inner } => Self::LoadFile(LoadFileEditor::from_operation(inner)),
            BitOperation::TakeSkipSequence { inner } => Self::TakeSkipSequence(TakeSkipEditor::from_operation(inner)),
            BitOperation::InvertBits { inner } => Self::InvertBits(InvertBitsEditor::from_operation(inner)),
            BitOperation::MultiWorksheetLoad { inner } => Self::MultiWorksheetLoad(MultiWorksheetEditor::from_operation(inner)),
            BitOperation::TruncateBits { inner } => Self::TruncateBits(TruncateEditor::from_operation(inner)),
            BitOperation::InterleaveBits { inner } => Self::InterleaveBits(InterleaveEditor::from_operation(inner)),
            BitOperation::IsolateBits { inner } => Self::IsolateBits(IsolateEditor::from_operation(inner)),
            BitOperation::ExcludeBits { inner } => Self::ExcludeBits(ExcludeEditor::from_operation(inner)),
        }
    }

    /// Try to build a BitOperation from the current editor state
    /// Returns Err with a message if validation fails
    pub fn try_build_operation(&self) -> Result<BitOperation, String> {
        match self {
            Self::LoadFile(ed) => ed.try_build().map(|inner| BitOperation::LoadFile { inner }),
            Self::TakeSkipSequence(ed) => ed.try_build().map(|inner| BitOperation::TakeSkipSequence { inner }),
            Self::InvertBits(ed) => ed.try_build().map(|inner| BitOperation::InvertBits { inner }),
            Self::MultiWorksheetLoad(ed) => ed.try_build().map(|inner| BitOperation::MultiWorksheetLoad { inner }),
            Self::TruncateBits(ed) => ed.try_build().map(|inner| BitOperation::TruncateBits { inner }),
            Self::InterleaveBits(ed) => ed.try_build().map(|inner| BitOperation::InterleaveBits { inner }),
            Self::IsolateBits(ed) => ed.try_build().map(|inner| BitOperation::IsolateBits { inner }),
            Self::ExcludeBits(ed) => ed.try_build().map(|inner| BitOperation::ExcludeBits { inner }),
        }
    }

    /// Render the editor UI and return the action taken
    pub fn render(&mut self, ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        match self {
            Self::LoadFile(ed) => ed.render(ctx, ui),
            Self::TakeSkipSequence(ed) => ed.render(ctx, ui),
            Self::InvertBits(ed) => ed.render(ctx, ui),
            Self::MultiWorksheetLoad(ed) => ed.render(ctx, ui),
            Self::TruncateBits(ed) => ed.render(ctx, ui),
            Self::InterleaveBits(ed) => ed.render(ctx, ui),
            Self::IsolateBits(ed) => ed.render(ctx, ui),
            Self::ExcludeBits(ed) => ed.render(ctx, ui),
        }
    }
}
