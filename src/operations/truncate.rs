use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::traits::{EditorAction, EditorContext, OperationEditor, render_save_cancel_buttons};
use crate::utils::eval_expression;
use eframe::egui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncateBitsOperation {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub enabled: bool,
}

impl TruncateBitsOperation {
    pub fn description(&self) -> String {
        format!("Keep bits {}-{}", self.start, self.end)
    }

    /// Apply truncate operation to the input bits.
    /// Takes ownership of input to enable potential future optimizations.
    /// Note: Slicing still requires copying the extracted portion.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let len = input.len();
        let actual_start = self.start.min(len);
        let actual_end = self.end.min(len);

        if actual_start >= actual_end {
            return BitVec::new();
        }

        input[actual_start..actual_end].to_bitvec()
    }

    pub fn is_source_operation(&self) -> bool {
        false
    }
}

// --- Editor ---

#[derive(Debug, Clone)]
pub struct TruncateEditor {
    pub name: String,
    pub start_str: String,
    pub end_str: String,
}

impl Default for TruncateEditor {
    fn default() -> Self {
        Self {
            name: String::new(),
            start_str: "0".to_string(),
            end_str: String::new(),
        }
    }
}

impl OperationEditor for TruncateEditor {
    type Operation = TruncateBitsOperation;

    fn from_operation(op: &TruncateBitsOperation) -> Self {
        Self {
            name: op.name.clone(),
            start_str: op.start.to_string(),
            end_str: if op.end == usize::MAX {
                String::new()
            } else {
                op.end.to_string()
            },
        }
    }

    fn try_build(&self) -> Result<TruncateBitsOperation, String> {
        let start = eval_expression(&self.start_str)
            .map_err(|_| "Invalid start value".to_string())?;

        let end = if self.end_str.trim().is_empty() {
            usize::MAX
        } else {
            eval_expression(&self.end_str).map_err(|_| "Invalid end value".to_string())?
        };

        if start >= end {
            return Err("Start must be less than end".to_string());
        }

        let name = if self.name.trim().is_empty() {
            format!(
                "Truncate: {}-{}",
                start,
                if end == usize::MAX {
                    "end".to_string()
                } else {
                    end.to_string()
                }
            )
        } else {
            self.name.clone()
        };

        Ok(TruncateBitsOperation {
            name,
            start,
            end,
            enabled: true,
        })
    }

    fn render(&mut self, _ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        ui.heading("Truncate Bits");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.add_space(8.0);

        ui.label("Specify the range of bits to keep:");
        ui.add_space(4.0);

        // Validate start field
        let start_valid = !self.start_str.is_empty() && eval_expression(&self.start_str).is_ok();
        let start_value = eval_expression(&self.start_str).ok();

        // Validate end field (can be empty)
        let end_valid = self.end_str.is_empty() || eval_expression(&self.end_str).is_ok();
        let end_value = if self.end_str.is_empty() {
            None
        } else {
            eval_expression(&self.end_str).ok()
        };

        // Check range validity
        let mut validation_message = None;
        let is_valid = if !start_valid {
            validation_message = Some("Start must be a valid number or expression");
            false
        } else if !end_valid {
            validation_message = Some("End must be a valid number or expression (or empty)");
            false
        } else if let (Some(start), Some(end)) = (start_value, end_value) {
            if end <= start {
                validation_message = Some("End must be greater than start");
                false
            } else {
                true
            }
        } else {
            true
        };

        ui.horizontal(|ui| {
            ui.label("Start (inclusive):");
            let mut start_edit = egui::TextEdit::singleline(&mut self.start_str);
            if !start_valid {
                start_edit = start_edit.text_color(egui::Color32::from_rgb(255, 100, 100));
            }
            let start_response = ui.add(start_edit);

            if start_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(result) = eval_expression(&self.start_str) {
                    self.start_str = result.to_string();
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("End (exclusive):  ");
            let mut end_edit = egui::TextEdit::singleline(&mut self.end_str);
            if !end_valid {
                end_edit = end_edit.text_color(egui::Color32::from_rgb(255, 100, 100));
            }
            let end_response = ui.add(end_edit);

            if end_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !self.end_str.is_empty() {
                    if let Ok(result) = eval_expression(&self.end_str) {
                        self.end_str = result.to_string();
                    }
                }
            }
        });

        ui.add_space(4.0);

        if let Some(msg) = validation_message {
            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("! {}", msg));
        }

        ui.label("Tips:");
        ui.label("• Leave end empty to keep until the end");
        ui.label("• You can use math: 8*8, 100+50, 200-10, 64/2");

        ui.add_space(8.0);

        render_save_cancel_buttons(ui, is_valid)
    }
}
