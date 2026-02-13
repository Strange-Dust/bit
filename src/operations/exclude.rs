use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::traits::{EditorAction, EditorContext, OperationEditor, render_save_cancel_buttons};
use crate::utils::eval_expression;
use eframe::egui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludeBitsOperation {
    pub name: String,
    pub start_col: usize,
    pub width: usize,
    pub start_row: usize,
    pub end_row: usize,
    pub source_frame_length: usize,
    pub enabled: bool,
}

impl ExcludeBitsOperation {
    pub fn description(&self) -> String {
        let end_col = self.start_col + self.width - 1;
        let end_row_display = self.end_row.saturating_sub(1);
        format!(
            "Exclude cols {}-{} from rows {}-{} (frame={})",
            self.start_col, end_col, self.start_row, end_row_display, self.source_frame_length
        )
    }

    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let fl = self.source_frame_length;
        if fl == 0 {
            return input;
        }
        let total_rows = (input.len() + fl - 1) / fl;
        let mut result = BitVec::new();
        for row in 0..total_rows {
            let row_start = row * fl;
            let is_affected_row = row >= self.start_row && row < self.end_row;
            for col in 0..fl {
                let bit_idx = row_start + col;
                if bit_idx >= input.len() {
                    break;
                }
                if is_affected_row && col >= self.start_col && col < self.start_col + self.width {
                    continue; // skip excluded columns in affected rows
                }
                result.push(input[bit_idx]);
            }
        }
        result
    }

    pub fn is_source_operation(&self) -> bool {
        false
    }
}

// --- Editor ---

#[derive(Debug, Clone)]
pub struct ExcludeEditor {
    pub name: String,
    pub start_col_str: String,
    pub width_str: String,
    pub start_row_str: String,
    pub end_row_str: String,
    pub source_frame_length_str: String,
}

impl Default for ExcludeEditor {
    fn default() -> Self {
        Self {
            name: String::new(),
            start_col_str: "0".to_string(),
            width_str: "8".to_string(),
            start_row_str: "0".to_string(),
            end_row_str: String::new(),
            source_frame_length_str: "64".to_string(),
        }
    }
}

impl OperationEditor for ExcludeEditor {
    type Operation = ExcludeBitsOperation;

    fn from_operation(op: &ExcludeBitsOperation) -> Self {
        Self {
            name: op.name.clone(),
            start_col_str: op.start_col.to_string(),
            width_str: op.width.to_string(),
            start_row_str: op.start_row.to_string(),
            end_row_str: if op.end_row == usize::MAX {
                String::new()
            } else {
                op.end_row.to_string()
            },
            source_frame_length_str: op.source_frame_length.to_string(),
        }
    }

    fn try_build(&self) -> Result<ExcludeBitsOperation, String> {
        let start_col = eval_expression(&self.start_col_str)
            .map_err(|_| "Invalid start column".to_string())?;
        let width = eval_expression(&self.width_str)
            .map_err(|_| "Invalid width".to_string())?;
        let start_row = eval_expression(&self.start_row_str)
            .map_err(|_| "Invalid start row".to_string())?;
        let end_row = if self.end_row_str.trim().is_empty() {
            usize::MAX
        } else {
            eval_expression(&self.end_row_str)
                .map_err(|_| "Invalid end row".to_string())?
        };
        let source_frame_length = eval_expression(&self.source_frame_length_str)
            .map_err(|_| "Invalid source frame length".to_string())?;

        if width == 0 {
            return Err("Width must be greater than 0".to_string());
        }
        if source_frame_length == 0 {
            return Err("Source frame length must be greater than 0".to_string());
        }
        if start_col + width > source_frame_length {
            return Err("Start column + width exceeds frame length".to_string());
        }
        if start_row >= end_row {
            return Err("Start row must be less than end row".to_string());
        }

        let name = if self.name.trim().is_empty() {
            let end_col = start_col + width - 1;
            format!("Exclude cols {}-{}", start_col, end_col)
        } else {
            self.name.clone()
        };

        Ok(ExcludeBitsOperation {
            name,
            start_col,
            width,
            start_row,
            end_row,
            source_frame_length,
            enabled: true,
        })
    }

    fn render(&mut self, _ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        ui.heading("Exclude Bits");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.add_space(8.0);
        ui.label("Remove the selected columns from the selected rows.");
        ui.label("Unaffected rows pass through unchanged.");
        ui.add_space(4.0);

        let start_col_valid = !self.start_col_str.is_empty() && eval_expression(&self.start_col_str).is_ok();
        let width_valid = !self.width_str.is_empty() && eval_expression(&self.width_str).is_ok();
        let start_row_valid = !self.start_row_str.is_empty() && eval_expression(&self.start_row_str).is_ok();
        let end_row_valid = self.end_row_str.is_empty() || eval_expression(&self.end_row_str).is_ok();
        let fl_valid = !self.source_frame_length_str.is_empty() && eval_expression(&self.source_frame_length_str).is_ok();

        let is_valid = start_col_valid && width_valid && start_row_valid && end_row_valid && fl_valid;

        render_field(ui, "Start Column:", &mut self.start_col_str, start_col_valid);
        render_field(ui, "Width (columns):", &mut self.width_str, width_valid);
        render_field(ui, "Start Row:", &mut self.start_row_str, start_row_valid);
        render_field(ui, "End Row (exclusive):", &mut self.end_row_str, end_row_valid);
        render_field(ui, "Source Frame Length:", &mut self.source_frame_length_str, fl_valid);

        ui.add_space(4.0);
        ui.label("Tips:");
        ui.label("  Leave end row empty to affect all rows");
        ui.label("  You can use math: 8*8, 100+50");

        ui.add_space(8.0);
        render_save_cancel_buttons(ui, is_valid)
    }
}

fn render_field(ui: &mut egui::Ui, label: &str, value: &mut String, valid: bool) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut edit = egui::TextEdit::singleline(value);
        if !valid {
            edit = edit.text_color(egui::Color32::from_rgb(255, 100, 100));
        }
        let response = ui.add(edit);
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !value.is_empty() {
                if let Ok(result) = eval_expression(value) {
                    *value = result.to_string();
                }
            }
        }
    });
}
