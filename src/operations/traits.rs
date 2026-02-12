// Operation editor traits and shared UI helpers
//
// This module defines the OperationEditor trait that each operation implements
// to provide its own self-contained editor UI.

use crate::storage::Worksheet;
use eframe::egui;

/// Result of rendering an operation editor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorAction {
    /// No action taken - continue showing editor
    None,
    /// User clicked Save - operation is ready to be saved
    Save,
    /// User clicked Cancel - discard changes
    Cancel,
}

/// Context provided to operation editors
/// Contains data that operations may need to render their UI
pub struct EditorContext<'a> {
    /// All worksheets (for MultiWorksheetLoad)
    pub worksheets: &'a [Worksheet],
    /// Current worksheet index (for MultiWorksheetLoad to exclude self)
    pub current_worksheet_index: usize,
}

impl<'a> EditorContext<'a> {
    pub fn new(worksheets: &'a [Worksheet], current_worksheet_index: usize) -> Self {
        Self {
            worksheets,
            current_worksheet_index,
        }
    }
}

/// Trait for self-contained operation editors.
/// Each operation implements this to provide its own editor UI.
pub trait OperationEditor: Default {
    type Operation;

    /// Create editor state from an existing operation (for editing)
    fn from_operation(op: &Self::Operation) -> Self;

    /// Try to build an operation from the current editor state.
    /// Returns Err with a message if validation fails.
    fn try_build(&self) -> Result<Self::Operation, String>;

    /// Render the editor UI and return the action taken.
    fn render(&mut self, ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction;
}

/// Helper function to render standard Save/Cancel buttons
pub fn render_save_cancel_buttons(ui: &mut egui::Ui, is_valid: bool) -> EditorAction {
    let mut action = EditorAction::None;

    ui.horizontal(|ui| {
        let save_button = egui::Button::new("Save");
        if ui
            .add_enabled(is_valid, save_button)
            .on_hover_text(if is_valid {
                "Save operation"
            } else {
                "Fix validation errors first"
            })
            .clicked()
        {
            action = EditorAction::Save;
        }

        if ui.button("Cancel").clicked() {
            action = EditorAction::Cancel;
        }
    });

    action
}
