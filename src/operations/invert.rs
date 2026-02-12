use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::traits::{EditorAction, EditorContext, OperationEditor, render_save_cancel_buttons};
use eframe::egui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvertBitsOperation {
    pub name: String,
    pub enabled: bool,
}

impl InvertBitsOperation {
    pub fn description(&self) -> String {
        "Inverts all bits".to_string()
    }

    /// Apply invert operation to the input bits.
    /// Takes ownership of input to avoid cloning large data structures.
    pub fn apply(&self, mut input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        input.iter_mut().for_each(|mut bit| *bit = !*bit);
        input
    }

    pub fn is_source_operation(&self) -> bool {
        false
    }
}

// --- Editor ---

#[derive(Debug, Clone, Default)]
pub struct InvertBitsEditor {
    pub name: String,
}

impl OperationEditor for InvertBitsEditor {
    type Operation = InvertBitsOperation;

    fn from_operation(op: &InvertBitsOperation) -> Self {
        Self {
            name: op.name.clone(),
        }
    }

    fn try_build(&self) -> Result<InvertBitsOperation, String> {
        let name = if self.name.trim().is_empty() {
            "Invert All Bits".to_string()
        } else {
            self.name.clone()
        };

        Ok(InvertBitsOperation { name, enabled: true })
    }

    fn render(&mut self, _ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        ui.heading("Invert All Bits");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.add_space(8.0);

        ui.label("This operation will invert all bits:");
        ui.label("• 0 → 1");
        ui.label("• 1 → 0");

        ui.add_space(8.0);

        render_save_cancel_buttons(ui, true)
    }
}
