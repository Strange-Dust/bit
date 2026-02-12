use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::base::OperationSequence;
use super::traits::{EditorAction, EditorContext, OperationEditor, render_save_cancel_buttons};
use eframe::egui;

/// Represents a take/skip operation to apply to a specific worksheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetOperation {
    pub worksheet_index: usize,
    pub sequence: OperationSequence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiWorksheetLoadOperation {
    pub name: String,
    pub worksheet_operations: Vec<WorksheetOperation>,
    pub enabled: bool,
}

impl MultiWorksheetLoadOperation {
    pub fn description(&self) -> String {
        format!("Load from {} worksheet(s)", self.worksheet_operations.len())
    }

    /// Apply multi-worksheet load operation.
    /// Takes ownership for consistency, though this operation type requires worksheet data
    /// and is handled specially in the main application. Returns empty here.
    pub fn apply(&self, _input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        BitVec::new()
    }

    pub fn is_source_operation(&self) -> bool {
        true
    }
}

// --- Editor ---

#[derive(Debug, Clone, Default)]
pub struct MultiWorksheetEditor {
    pub name: String,
    pub worksheet_ops: Vec<(usize, String)>,
    pub worksheet_input: String,
    pub selected_worksheet: usize,
}

impl OperationEditor for MultiWorksheetEditor {
    type Operation = MultiWorksheetLoadOperation;

    fn from_operation(op: &MultiWorksheetLoadOperation) -> Self {
        Self {
            name: op.name.clone(),
            worksheet_ops: op
                .worksheet_operations
                .iter()
                .map(|wo| (wo.worksheet_index, wo.sequence.to_string()))
                .collect(),
            ..Self::default()
        }
    }

    fn try_build(&self) -> Result<MultiWorksheetLoadOperation, String> {
        if self.worksheet_ops.is_empty() {
            return Err("Must add at least one worksheet operation".to_string());
        }

        let mut worksheet_operations = Vec::new();
        for (ws_idx, seq_str) in &self.worksheet_ops {
            let sequence = OperationSequence::from_string(seq_str).map_err(|e| {
                format!("Invalid sequence for worksheet {}: {}", ws_idx + 1, e)
            })?;
            worksheet_operations.push(WorksheetOperation {
                worksheet_index: *ws_idx,
                sequence,
            });
        }

        let name = if self.name.trim().is_empty() {
            format!("Load from {} worksheets", worksheet_operations.len())
        } else {
            self.name.clone()
        };

        Ok(MultiWorksheetLoadOperation {
            name,
            worksheet_operations,
            enabled: true,
        })
    }

    fn render(&mut self, ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        ui.heading("Multi-Worksheet Load");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label("Add Worksheet Operation:");

            ui.horizontal(|ui| {
                ui.label("Worksheet:");
                egui::ComboBox::from_id_salt("worksheet_selector")
                    .selected_text(if self.selected_worksheet < ctx.worksheets.len() {
                        &ctx.worksheets[self.selected_worksheet].name
                    } else {
                        "Select..."
                    })
                    .show_ui(ui, |ui| {
                        for (idx, worksheet) in ctx.worksheets.iter().enumerate() {
                            if idx != ctx.current_worksheet_index {
                                ui.selectable_value(
                                    &mut self.selected_worksheet,
                                    idx,
                                    &worksheet.name,
                                );
                            }
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Sequence:");
                ui.text_edit_singleline(&mut self.worksheet_input);
            });

            if ui.button("+ Add").clicked() && !self.worksheet_input.is_empty() {
                self.worksheet_ops
                    .push((self.selected_worksheet, self.worksheet_input.clone()));
                self.worksheet_input.clear();
            }
        });

        ui.add_space(8.0);

        if self.worksheet_ops.is_empty() {
            ui.label("No worksheets added yet");
        } else {
            let mut to_remove = None;
            for (idx, (ws_idx, seq)) in self.worksheet_ops.iter().enumerate() {
                ui.horizontal(|ui| {
                    let ws_name = if *ws_idx < ctx.worksheets.len() {
                        &ctx.worksheets[*ws_idx].name
                    } else {
                        "Unknown"
                    };
                    ui.label(format!("{}. {} → {}", idx + 1, ws_name, seq));
                    if ui.button("x").clicked() {
                        to_remove = Some(idx);
                    }
                });
            }

            if let Some(idx) = to_remove {
                self.worksheet_ops.remove(idx);
            }
        }

        ui.add_space(8.0);

        render_save_cancel_buttons(ui, !self.worksheet_ops.is_empty())
    }
}
