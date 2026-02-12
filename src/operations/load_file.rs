use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::traits::{EditorAction, EditorContext, OperationEditor, render_save_cancel_buttons};
use eframe::egui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadFileOperation {
    pub name: String,
    pub file_path: PathBuf,
    pub enabled: bool,
}

impl LoadFileOperation {
    pub fn description(&self) -> String {
        format!(
            "Load: {}",
            self.file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        )
    }

    /// Apply load file operation.
    /// Takes ownership for consistency, though LoadFile operations are handled
    /// specially in the main application since they need file I/O.
    /// Returns the input unchanged here.
    pub fn apply(&self, input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        input
    }

    pub fn is_source_operation(&self) -> bool {
        true
    }
}

// --- Editor ---

#[derive(Debug, Clone, Default)]
pub struct LoadFileEditor {
    pub name: String,
    pub file_path: Option<PathBuf>,
}

impl OperationEditor for LoadFileEditor {
    type Operation = LoadFileOperation;

    fn from_operation(op: &LoadFileOperation) -> Self {
        Self {
            name: op.name.clone(),
            file_path: Some(op.file_path.clone()),
        }
    }

    fn try_build(&self) -> Result<LoadFileOperation, String> {
        let file_path = self
            .file_path
            .clone()
            .ok_or_else(|| "Please select a file to load".to_string())?;

        let name = if self.name.trim().is_empty() {
            format!(
                "Load: {}",
                file_path.file_name().unwrap_or_default().to_string_lossy()
            )
        } else {
            self.name.clone()
        };

        Ok(LoadFileOperation {
            name,
            file_path,
            enabled: true,
        })
    }

    fn render(&mut self, _ctx: &EditorContext, ui: &mut egui::Ui) -> EditorAction {
        ui.heading("Load File");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name);
        });

        ui.add_space(8.0);

        if let Some(path) = &self.file_path {
            ui.label(format!("Selected: {}", path.display()));
        } else {
            ui.label("No file selected");
        }

        if ui.button("Browse...").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                self.file_path = Some(path);
            }
        }

        ui.add_space(8.0);

        render_save_cancel_buttons(ui, self.file_path.is_some())
    }
}
