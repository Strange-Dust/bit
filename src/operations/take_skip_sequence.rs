use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::base::OperationSequence;
use super::traits::{EditorAction, EditorContext, OperationEditor, render_save_cancel_buttons};
use eframe::egui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeSkipSequenceOperation {
    pub name: String,
    pub sequence: OperationSequence,
    pub enabled: bool,
}

impl TakeSkipSequenceOperation {
    pub fn description(&self) -> String {
        self.sequence.to_string()
    }

    /// Apply take/skip sequence operation to input bits.
    /// Takes ownership for consistency; borrows internally via &input.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        self.sequence.apply(&input)
    }

    pub fn is_source_operation(&self) -> bool {
        false
    }
}

// --- Editor ---

#[derive(Debug, Clone, Default)]
pub struct TakeSkipEditor {
    pub name: String,
    pub sequence_input: String,
}

impl OperationEditor for TakeSkipEditor {
    type Operation = TakeSkipSequenceOperation;

    fn from_operation(op: &TakeSkipSequenceOperation) -> Self {
        Self {
            name: op.name.clone(),
            sequence_input: op.sequence.to_string(),
        }
    }

    fn try_build(&self) -> Result<TakeSkipSequenceOperation, String> {
        if self.sequence_input.is_empty() {
            return Err("Operation sequence cannot be empty".to_string());
        }

        let sequence = OperationSequence::from_string(&self.sequence_input)
            .map_err(|e| format!("Invalid operation: {}", e))?;

        let name = if self.name.trim().is_empty() {
            format!("Sequence: {}", self.sequence_input)
        } else {
            self.name.clone()
        };

        Ok(TakeSkipSequenceOperation {
            name,
            sequence,
            enabled: true,
        })
    }

    fn render(&mut self, _ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        ui.heading("Take/Skip Sequence");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.add_space(8.0);

        ui.label("Enter a sequence of operations:");
        ui.label("• t = take N bits");
        ui.label("• r = reverse N bits");
        ui.label("• i = invert N bits");
        ui.label("• s = skip N bits");

        ui.add_space(8.0);

        // Validate the sequence in real-time
        let validation_result = if self.sequence_input.is_empty() {
            Err("Sequence cannot be empty".to_string())
        } else {
            OperationSequence::from_string(&self.sequence_input).map(|_| ())
        };
        let is_valid = validation_result.is_ok();

        ui.horizontal(|ui| {
            ui.label("Sequence:");
            let mut text_edit = egui::TextEdit::singleline(&mut self.sequence_input);

            if !is_valid {
                text_edit = text_edit.text_color(egui::Color32::from_rgb(255, 100, 100));
            }

            ui.add(text_edit);
        });

        if let Err(err) = validation_result {
            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("! {}", err));
            ui.colored_label(
                egui::Color32::from_rgb(200, 200, 200),
                "Valid operations: t (take), s (skip), r (reverse), i (invert)",
            );
        } else {
            ui.label("Example: t4r3i8s1");
        }

        ui.add_space(8.0);

        render_save_cancel_buttons(ui, is_valid)
    }
}
