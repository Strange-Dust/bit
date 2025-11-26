// Operation editor support - allows operations to render their own UI
//
// This module provides the EditorContext, EditorAction, and OperationEditorState types
// that enable operations to be self-describing, rendering their own editors without
// requiring changes to app.rs or ui/windows.rs for each new operation.

use super::{
    BitOperation, BlockInterleaverConfig, ConvolutionalInterleaverConfig,
    InterleaverDirection, InterleaverType, OperationSequence, OperationType,
    SymbolInterleaverConfig, WorksheetOperation,
};
use crate::storage::Worksheet;
use crate::utils::eval_expression;
use eframe::egui;
use std::path::PathBuf;

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

/// Temporary state used while editing an operation
/// This holds string representations of numeric fields for validation
#[derive(Debug, Clone, Default)]
pub struct OperationEditorState {
    // Common
    pub name: String,

    // LoadFile
    pub file_path: Option<PathBuf>,

    // TakeSkipSequence
    pub sequence_input: String,

    // TruncateBits
    pub start_str: String,
    pub end_str: String,

    // InterleaveBits
    pub interleave_type: InterleaverType,
    pub interleave_direction: InterleaverDirection,
    pub block_size_str: String,
    pub depth_str: String,
    pub branches_str: String,
    pub delay_increment_str: String,
    pub symbol_size_str: String,

    // MultiWorksheetLoad
    pub worksheet_ops: Vec<(usize, String)>,
    pub worksheet_input: String,
    pub selected_worksheet: usize,
}

impl Default for InterleaverType {
    fn default() -> Self {
        InterleaverType::Block
    }
}

impl Default for InterleaverDirection {
    fn default() -> Self {
        InterleaverDirection::Interleave
    }
}

impl OperationEditorState {
    /// Create editor state for a new operation of the given type
    pub fn new_for_type(op_type: OperationType) -> Self {
        let mut state = Self::default();
        state.start_str = "0".to_string();
        state.block_size_str = "8".to_string();
        state.depth_str = "4".to_string();
        state.branches_str = "4".to_string();
        state.delay_increment_str = "1".to_string();
        state.symbol_size_str = "8".to_string();

        // Set default name based on operation type
        state.name = match op_type {
            OperationType::LoadFile => String::new(),
            OperationType::TakeSkipSequence => String::new(),
            OperationType::InvertBits => String::new(),
            OperationType::TruncateBits => String::new(),
            OperationType::InterleaveBits => String::new(),
            OperationType::MultiWorksheetLoad => String::new(),
        };

        state
    }

    /// Create editor state from an existing operation (for editing)
    pub fn from_operation(op: &BitOperation) -> Self {
        match op {
            BitOperation::LoadFile { name, file_path, .. } => Self {
                name: name.clone(),
                file_path: Some(file_path.clone()),
                ..Self::new_for_type(OperationType::LoadFile)
            },
            BitOperation::TakeSkipSequence { name, sequence, .. } => Self {
                name: name.clone(),
                sequence_input: sequence.to_string(),
                ..Self::new_for_type(OperationType::TakeSkipSequence)
            },
            BitOperation::InvertBits { name, .. } => Self {
                name: name.clone(),
                ..Self::new_for_type(OperationType::InvertBits)
            },
            BitOperation::TruncateBits { name, start, end, .. } => Self {
                name: name.clone(),
                start_str: start.to_string(),
                end_str: if *end == usize::MAX {
                    String::new()
                } else {
                    end.to_string()
                },
                ..Self::new_for_type(OperationType::TruncateBits)
            },
            BitOperation::InterleaveBits {
                name,
                interleaver_type,
                block_config,
                convolutional_config,
                symbol_config,
                ..
            } => {
                let mut state = Self {
                    name: name.clone(),
                    interleave_type: *interleaver_type,
                    ..Self::new_for_type(OperationType::InterleaveBits)
                };

                match interleaver_type {
                    InterleaverType::Block => {
                        if let Some(cfg) = block_config {
                            state.interleave_direction = cfg.direction;
                            state.block_size_str = cfg.block_size.to_string();
                            state.depth_str = cfg.depth.to_string();
                        }
                    }
                    InterleaverType::Convolutional => {
                        if let Some(cfg) = convolutional_config {
                            state.interleave_direction = cfg.direction;
                            state.branches_str = cfg.branches.to_string();
                            state.delay_increment_str = cfg.delay_increment.to_string();
                        }
                    }
                    InterleaverType::Symbol => {
                        if let Some(cfg) = symbol_config {
                            state.interleave_direction = cfg.direction;
                            state.symbol_size_str = cfg.symbol_size.to_string();
                            state.block_size_str = cfg.block_size.to_string();
                            state.depth_str = cfg.depth.to_string();
                        }
                    }
                }

                state
            }
            BitOperation::MultiWorksheetLoad {
                name,
                worksheet_operations,
                ..
            } => Self {
                name: name.clone(),
                worksheet_ops: worksheet_operations
                    .iter()
                    .map(|wo| (wo.worksheet_index, wo.sequence.to_string()))
                    .collect(),
                ..Self::new_for_type(OperationType::MultiWorksheetLoad)
            },
        }
    }

    /// Try to build a BitOperation from the current editor state
    /// Returns Err with a message if validation fails
    pub fn try_build_operation(&self, op_type: OperationType) -> Result<BitOperation, String> {
        match op_type {
            OperationType::LoadFile => {
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

                Ok(BitOperation::LoadFile {
                    name,
                    file_path,
                    enabled: true,
                })
            }

            OperationType::TakeSkipSequence => {
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

                Ok(BitOperation::TakeSkipSequence {
                    name,
                    sequence,
                    enabled: true,
                })
            }

            OperationType::InvertBits => {
                let name = if self.name.trim().is_empty() {
                    "Invert All Bits".to_string()
                } else {
                    self.name.clone()
                };

                Ok(BitOperation::InvertBits { name, enabled: true })
            }

            OperationType::TruncateBits => {
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

                Ok(BitOperation::TruncateBits {
                    name,
                    start,
                    end,
                    enabled: true,
                })
            }

            OperationType::InterleaveBits => {
                let name = if self.name.trim().is_empty() {
                    match self.interleave_type {
                        InterleaverType::Block => "Block Interleaver".to_string(),
                        InterleaverType::Convolutional => "Convolutional Interleaver".to_string(),
                        InterleaverType::Symbol => "Symbol Interleaver".to_string(),
                    }
                } else {
                    self.name.clone()
                };

                let (block_config, convolutional_config, symbol_config) = match self.interleave_type
                {
                    InterleaverType::Block => {
                        let block_size = self
                            .block_size_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Block size must be a positive number".to_string())?;
                        if block_size == 0 {
                            return Err("Block size must be a positive number".to_string());
                        }

                        let depth = self
                            .depth_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Depth must be a positive number".to_string())?;
                        if depth == 0 {
                            return Err("Depth must be a positive number".to_string());
                        }

                        (
                            Some(BlockInterleaverConfig::new(
                                block_size,
                                depth,
                                self.interleave_direction,
                            )),
                            None,
                            None,
                        )
                    }
                    InterleaverType::Convolutional => {
                        let branches = self
                            .branches_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Branches must be a positive number".to_string())?;
                        if branches == 0 {
                            return Err("Branches must be a positive number".to_string());
                        }

                        let delay_increment = self
                            .delay_increment_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Delay increment must be a valid number".to_string())?;

                        (
                            None,
                            Some(ConvolutionalInterleaverConfig::new(
                                branches,
                                delay_increment,
                                self.interleave_direction,
                            )),
                            None,
                        )
                    }
                    InterleaverType::Symbol => {
                        let symbol_size = self
                            .symbol_size_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Symbol size must be a positive number".to_string())?;
                        if symbol_size == 0 {
                            return Err("Symbol size must be a positive number".to_string());
                        }

                        let block_size = self
                            .block_size_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Block size must be a positive number".to_string())?;
                        if block_size == 0 {
                            return Err("Block size must be a positive number".to_string());
                        }

                        let depth = self
                            .depth_str
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| "Depth must be a positive number".to_string())?;
                        if depth == 0 {
                            return Err("Depth must be a positive number".to_string());
                        }

                        (
                            None,
                            None,
                            Some(SymbolInterleaverConfig::new(
                                symbol_size,
                                block_size,
                                depth,
                                self.interleave_direction,
                            )),
                        )
                    }
                };

                Ok(BitOperation::InterleaveBits {
                    name,
                    interleaver_type: self.interleave_type,
                    block_config,
                    convolutional_config,
                    symbol_config,
                    enabled: true,
                })
            }

            OperationType::MultiWorksheetLoad => {
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

                Ok(BitOperation::MultiWorksheetLoad {
                    name,
                    worksheet_operations,
                    enabled: true,
                })
            }
        }
    }
}

/// Render the editor UI for an operation type
/// Returns an EditorAction indicating what the user did
pub fn render_operation_editor(
    op_type: OperationType,
    state: &mut OperationEditorState,
    ctx: &EditorContext,
    ui: &mut egui::Ui,
) -> EditorAction {
    match op_type {
        OperationType::LoadFile => render_loadfile_editor(state, ui),
        OperationType::TakeSkipSequence => render_takeskip_editor(state, ui),
        OperationType::InvertBits => render_invert_editor(state, ui),
        OperationType::TruncateBits => render_truncate_editor(state, ui),
        OperationType::InterleaveBits => render_interleave_editor(state, ui),
        OperationType::MultiWorksheetLoad => render_multiworksheet_editor(state, ctx, ui),
    }
}

fn render_loadfile_editor(state: &mut OperationEditorState, ui: &mut egui::Ui) -> EditorAction {
    ui.heading("Load File");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    ui.add_space(8.0);

    if let Some(path) = &state.file_path {
        ui.label(format!("📄 Selected: {}", path.display()));
    } else {
        ui.label("No file selected");
    }

    if ui.button("📂 Browse...").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            state.file_path = Some(path);
        }
    }

    ui.add_space(8.0);

    render_save_cancel_buttons(ui, state.file_path.is_some())
}

fn render_takeskip_editor(state: &mut OperationEditorState, ui: &mut egui::Ui) -> EditorAction {
    ui.heading("Take/Skip Sequence");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    ui.add_space(8.0);

    ui.label("Enter a sequence of operations:");
    ui.label("• t = take N bits");
    ui.label("• r = reverse N bits");
    ui.label("• i = invert N bits");
    ui.label("• s = skip N bits");

    ui.add_space(8.0);

    // Validate the sequence in real-time
    let validation_result = if state.sequence_input.is_empty() {
        Err("Sequence cannot be empty".to_string())
    } else {
        OperationSequence::from_string(&state.sequence_input).map(|_| ())
    };
    let is_valid = validation_result.is_ok();

    ui.horizontal(|ui| {
        ui.label("Sequence:");
        let mut text_edit = egui::TextEdit::singleline(&mut state.sequence_input);

        if !is_valid {
            text_edit = text_edit.text_color(egui::Color32::from_rgb(255, 100, 100));
        }

        ui.add(text_edit);
    });

    if let Err(err) = validation_result {
        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("⚠ {}", err));
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

fn render_invert_editor(state: &mut OperationEditorState, ui: &mut egui::Ui) -> EditorAction {
    ui.heading("Invert All Bits");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    ui.add_space(8.0);

    ui.label("This operation will invert all bits:");
    ui.label("• 0 → 1");
    ui.label("• 1 → 0");

    ui.add_space(8.0);

    render_save_cancel_buttons(ui, true)
}

fn render_truncate_editor(state: &mut OperationEditorState, ui: &mut egui::Ui) -> EditorAction {
    ui.heading("Truncate Bits");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    ui.add_space(8.0);

    ui.label("Specify the range of bits to keep:");
    ui.add_space(4.0);

    // Validate start field
    let start_valid = !state.start_str.is_empty() && eval_expression(&state.start_str).is_ok();
    let start_value = eval_expression(&state.start_str).ok();

    // Validate end field (can be empty)
    let end_valid = state.end_str.is_empty() || eval_expression(&state.end_str).is_ok();
    let end_value = if state.end_str.is_empty() {
        None
    } else {
        eval_expression(&state.end_str).ok()
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
        let mut start_edit = egui::TextEdit::singleline(&mut state.start_str);
        if !start_valid {
            start_edit = start_edit.text_color(egui::Color32::from_rgb(255, 100, 100));
        }
        let start_response = ui.add(start_edit);

        if start_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(result) = eval_expression(&state.start_str) {
                state.start_str = result.to_string();
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("End (exclusive):  ");
        let mut end_edit = egui::TextEdit::singleline(&mut state.end_str);
        if !end_valid {
            end_edit = end_edit.text_color(egui::Color32::from_rgb(255, 100, 100));
        }
        let end_response = ui.add(end_edit);

        if end_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !state.end_str.is_empty() {
                if let Ok(result) = eval_expression(&state.end_str) {
                    state.end_str = result.to_string();
                }
            }
        }
    });

    ui.add_space(4.0);

    if let Some(msg) = validation_message {
        ui.colored_label(egui::Color32::from_rgb(255, 100, 100), format!("⚠ {}", msg));
    }

    ui.label("💡 Tips:");
    ui.label("• Leave end empty to keep until the end");
    ui.label("• You can use math: 8*8, 100+50, 200-10, 64/2");

    ui.add_space(8.0);

    render_save_cancel_buttons(ui, is_valid)
}

fn render_interleave_editor(state: &mut OperationEditorState, ui: &mut egui::Ui) -> EditorAction {
    ui.heading("Bit Interleaving / De-interleaving");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    ui.add_space(8.0);

    // Type selection
    ui.horizontal(|ui| {
        ui.label("Interleaver Type:");
        ui.radio_value(&mut state.interleave_type, InterleaverType::Block, "Block");
        ui.radio_value(
            &mut state.interleave_type,
            InterleaverType::Convolutional,
            "Convolutional",
        );
        ui.radio_value(&mut state.interleave_type, InterleaverType::Symbol, "Symbol");
    });

    ui.add_space(4.0);

    // Direction selection
    ui.horizontal(|ui| {
        ui.label("Direction:");
        ui.radio_value(
            &mut state.interleave_direction,
            InterleaverDirection::Interleave,
            "Interleave",
        );
        ui.radio_value(
            &mut state.interleave_direction,
            InterleaverDirection::Deinterleave,
            "De-interleave",
        );
    });

    ui.add_space(8.0);
    ui.separator();

    // Parameters based on type
    let is_valid = match state.interleave_type {
        InterleaverType::Block => {
            ui.label("📦 Block Interleaver Parameters:");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Block Size (columns):");
                let block_response = ui.text_edit_singleline(&mut state.block_size_str);

                if block_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&state.block_size_str) {
                        state.block_size_str = result.to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Depth (rows):      ");
                let depth_response = ui.text_edit_singleline(&mut state.depth_str);

                if depth_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&state.depth_str) {
                        state.depth_str = result.to_string();
                    }
                }
            });

            let block_size = state.block_size_str.trim().parse::<usize>().ok();
            let depth = state.depth_str.trim().parse::<usize>().ok();
            let is_valid = block_size.map_or(false, |v| v > 0) && depth.map_or(false, |v| v > 0);

            ui.add_space(8.0);

            // Visual preview
            ui.group(|ui| {
                ui.label("📊 Visual Preview:");
                ui.add_space(4.0);

                if let (Some(block_size), Some(depth)) = (block_size, depth) {
                    if block_size > 0 && depth > 0 && block_size <= 16 && depth <= 16 {
                        ui.label(format!(
                            "Matrix: {}×{} = {} bits per block",
                            block_size,
                            depth,
                            block_size * depth
                        ));
                        ui.add_space(4.0);

                        match state.interleave_direction {
                            InterleaverDirection::Interleave => {
                                ui.label("Write row-wise → Read column-wise:");
                            }
                            InterleaverDirection::Deinterleave => {
                                ui.label("Write column-wise → Read row-wise:");
                            }
                        }
                        ui.add_space(2.0);
                        render_block_matrix_preview(ui, block_size, depth);
                    } else {
                        ui.label("⚠ Invalid dimensions (max 16×16)");
                    }
                } else {
                    ui.label("⚠ Enter valid numbers for preview");
                }
            });

            ui.add_space(4.0);
            ui.label("💡 Tips:");
            ui.label("• Block interleaver rearranges data in matrix blocks");
            ui.label("• Interleave: Write row-wise, read column-wise");
            ui.label("• De-interleave: Reverses the process");
            ui.label("• Example: 8×4 matrix handles 32 bits at a time");
            ui.label("• Math supported: 2*4, 16/2, 8+4, etc.");

            is_valid
        }
        InterleaverType::Convolutional => {
            ui.label("🔄 Convolutional Interleaver Parameters:");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Branches (B):       ");
                let branches_response = ui.text_edit_singleline(&mut state.branches_str);

                if branches_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&state.branches_str) {
                        state.branches_str = result.to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Delay Increment (M):");
                let delay_response = ui.text_edit_singleline(&mut state.delay_increment_str);

                if delay_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&state.delay_increment_str) {
                        state.delay_increment_str = result.to_string();
                    }
                }
            });

            let branches = state.branches_str.trim().parse::<usize>().ok();
            let delay_inc = state.delay_increment_str.trim().parse::<usize>().ok();
            let is_valid = branches.map_or(false, |v| v > 0) && delay_inc.is_some();

            ui.add_space(8.0);

            // Visual preview
            ui.group(|ui| {
                ui.label("📊 Visual Preview:");
                ui.add_space(4.0);

                if let (Some(branches), Some(delay_inc)) = (branches, delay_inc) {
                    if branches > 0 && branches <= 16 && delay_inc <= 16 {
                        ui.label(format!(
                            "Branches: {}, Delay Increment: {}",
                            branches, delay_inc
                        ));
                        ui.add_space(4.0);

                        match state.interleave_direction {
                            InterleaverDirection::Interleave => {
                                ui.label("Round-robin distribution with delay:");
                            }
                            InterleaverDirection::Deinterleave => {
                                ui.label("Reverse round-robin with delay:");
                            }
                        }
                        ui.add_space(2.0);
                        render_convolutional_preview(ui, branches, delay_inc);
                    } else {
                        ui.label("⚠ Invalid parameters (max B=16, M=16)");
                    }
                } else {
                    ui.label("⚠ Enter valid numbers for preview");
                }
            });

            ui.add_space(4.0);
            ui.label("💡 Tips:");
            ui.label("• Convolutional uses delay lines for each branch");
            ui.label("• Branch i has delay = i × M symbols");
            ui.label("• Distributes bits round-robin across branches");
            ui.label("• Example: B=4, M=1 → delays [0,1,2,3]");
            ui.label("• Provides time-diversity for burst errors");

            is_valid
        }
        InterleaverType::Symbol => {
            ui.label("🔤 Symbol Interleaver Parameters:");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Symbol Size (bits):");
                let symbol_response = ui.text_edit_singleline(&mut state.symbol_size_str);

                if symbol_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&state.symbol_size_str) {
                        state.symbol_size_str = result.to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Block Size (columns):");
                let block_response = ui.text_edit_singleline(&mut state.block_size_str);

                if block_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&state.block_size_str) {
                        state.block_size_str = result.to_string();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Depth (rows):       ");
                let depth_response = ui.text_edit_singleline(&mut state.depth_str);

                if depth_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&state.depth_str) {
                        state.depth_str = result.to_string();
                    }
                }
            });

            ui.add_space(8.0);

            ui.label("💡 Tips:");
            ui.label("• Symbol interleaver treats multi-bit symbols as atomic units");
            ui.label("• Common symbol sizes: 8 (bytes), 4 (nibbles), 16 (words)");
            ui.label("• Use for AABBCCDD → ABCDABCD transformations");
            ui.label("• Matrix: Write symbols row-wise, read column-wise");
            ui.label("• Example: Symbol=8, Block=2, Depth=4 → AABBCCDD → ABCDABCD");

            state
                .symbol_size_str
                .trim()
                .parse::<usize>()
                .map_or(false, |v| v > 0)
                && state
                    .block_size_str
                    .trim()
                    .parse::<usize>()
                    .map_or(false, |v| v > 0)
                && state
                    .depth_str
                    .trim()
                    .parse::<usize>()
                    .map_or(false, |v| v > 0)
        }
    };

    ui.add_space(8.0);

    render_save_cancel_buttons(ui, is_valid)
}

fn render_block_matrix_preview(ui: &mut egui::Ui, cols: usize, rows: usize) {
    use egui::{Color32, Rect, Stroke};

    let cell_size = 24.0;
    let spacing = 2.0;
    let total_width = cols as f32 * (cell_size + spacing);
    let total_height = rows as f32 * (cell_size + spacing);

    let (response, painter) = ui.allocate_painter(
        egui::vec2(total_width, total_height),
        egui::Sense::hover(),
    );

    let base_pos = response.rect.min;

    for row in 0..rows.min(8) {
        for col in 0..cols.min(8) {
            let x = base_pos.x + col as f32 * (cell_size + spacing);
            let y = base_pos.y + row as f32 * (cell_size + spacing);

            let rect = Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell_size, cell_size));

            let bit_index = row * cols + col;
            let color = Color32::from_rgb(
                (100 + bit_index * 15).min(255) as u8,
                (150 - bit_index * 5).max(50) as u8,
                200,
            );

            painter.rect_filled(rect, 2.0, color);
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::from_gray(80)),
                egui::epaint::StrokeKind::Outside,
            );

            let text = format!("{}", bit_index);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::monospace(10.0),
                Color32::WHITE,
            );
        }
    }

    if rows > 8 || cols > 8 {
        ui.label("  (showing first 8×8)");
    }
}

fn render_convolutional_preview(ui: &mut egui::Ui, branches: usize, delay_inc: usize) {
    use egui::{Color32, Rect, Stroke};

    let branch_width = 80.0;
    let branch_height = 30.0;
    let spacing = 10.0;

    let total_width = branch_width + 150.0;
    let total_height = branches.min(8) as f32 * (branch_height + spacing);

    let (response, painter) = ui.allocate_painter(
        egui::vec2(total_width, total_height),
        egui::Sense::hover(),
    );

    let base_pos = response.rect.min;

    for i in 0..branches.min(8) {
        let y = base_pos.y + i as f32 * (branch_height + spacing);
        let delay = i * delay_inc;

        // Draw branch box
        let rect = Rect::from_min_size(
            egui::pos2(base_pos.x, y),
            egui::vec2(branch_width, branch_height),
        );

        let color = Color32::from_rgb(
            (100 + i * 20).min(255) as u8,
            (150 - i * 10).max(50) as u8,
            200,
        );

        painter.rect_filled(rect, 3.0, color);
        painter.rect_stroke(
            rect,
            3.0,
            Stroke::new(1.5, Color32::from_gray(80)),
            egui::epaint::StrokeKind::Outside,
        );

        let text = format!("Branch {}", i);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );

        // Draw delay indicator
        let delay_text = format!("Delay: {} symbols", delay);
        painter.text(
            egui::pos2(base_pos.x + branch_width + 15.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            delay_text,
            egui::FontId::monospace(10.0),
            ui.style().visuals.text_color(),
        );
    }

    if branches > 8 {
        ui.label("  (showing first 8 branches)");
    }
}

fn render_multiworksheet_editor(
    state: &mut OperationEditorState,
    ctx: &EditorContext,
    ui: &mut egui::Ui,
) -> EditorAction {
    ui.heading("Multi-Worksheet Load");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label("Add Worksheet Operation:");

        ui.horizontal(|ui| {
            ui.label("Worksheet:");
            egui::ComboBox::from_id_salt("worksheet_selector")
                .selected_text(if state.selected_worksheet < ctx.worksheets.len() {
                    &ctx.worksheets[state.selected_worksheet].name
                } else {
                    "Select..."
                })
                .show_ui(ui, |ui| {
                    for (idx, worksheet) in ctx.worksheets.iter().enumerate() {
                        if idx != ctx.current_worksheet_index {
                            ui.selectable_value(&mut state.selected_worksheet, idx, &worksheet.name);
                        }
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Sequence:");
            ui.text_edit_singleline(&mut state.worksheet_input);
        });

        if ui.button("➕ Add").clicked() && !state.worksheet_input.is_empty() {
            state
                .worksheet_ops
                .push((state.selected_worksheet, state.worksheet_input.clone()));
            state.worksheet_input.clear();
        }
    });

    ui.add_space(8.0);

    if state.worksheet_ops.is_empty() {
        ui.label("No worksheets added yet");
    } else {
        let mut to_remove = None;
        for (idx, (ws_idx, seq)) in state.worksheet_ops.iter().enumerate() {
            ui.horizontal(|ui| {
                let ws_name = if *ws_idx < ctx.worksheets.len() {
                    &ctx.worksheets[*ws_idx].name
                } else {
                    "Unknown"
                };
                ui.label(format!("{}. {} → {}", idx + 1, ws_name, seq));
                if ui.button("❌").clicked() {
                    to_remove = Some(idx);
                }
            });
        }

        if let Some(idx) = to_remove {
            state.worksheet_ops.remove(idx);
        }
    }

    ui.add_space(8.0);

    render_save_cancel_buttons(ui, !state.worksheet_ops.is_empty())
}

/// Helper function to render standard Save/Cancel buttons
fn render_save_cancel_buttons(ui: &mut egui::Ui, is_valid: bool) -> EditorAction {
    let mut action = EditorAction::None;

    ui.horizontal(|ui| {
        let save_button = egui::Button::new("✓ Save");
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

        if ui.button("✗ Cancel").clicked() {
            action = EditorAction::Cancel;
        }
    });

    action
}
