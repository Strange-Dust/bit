// Main application state and logic

use crate::analysis::{Pattern, PatternFormat};
use crate::core::{ViewMode, OperationType};
use crate::processing::{BitOperation, OperationSequence, WorksheetOperation};
use crate::storage::{read_file_as_bits, write_bits_to_file, AppSession, AppSettings, Worksheet};
use crate::viewers::{BitViewer, ByteViewer};
use bitvec::prelude::*;
use std::path::PathBuf;

pub struct BitApp {
    pub original_bits: BitVec<u8, Msb0>,
    pub processed_bits: BitVec<u8, Msb0>,
    pub viewer: BitViewer,
    pub byte_viewer: ByteViewer,
    pub view_mode: ViewMode,
    pub operations: Vec<BitOperation>,
    pub current_file_path: Option<PathBuf>,
    pub error_message: Option<String>,
    pub show_original: bool,
    pub show_settings: bool,
    pub font_size: f32,
    pub settings: AppSettings,
    
    // Worksheet management
    pub worksheets: Vec<Worksheet>,
    pub current_worksheet_index: usize,
    pub renaming_worksheet: Option<usize>,
    pub worksheet_name_buffer: String,
    
    // Operation creation/editing state
    pub show_operation_menu: Option<OperationType>,
    pub editing_operation_index: Option<usize>,
    
    // Drag and drop state
    pub dragging_operation: Option<usize>,
    
    // Take/Skip Sequence editor state
    pub takeskip_name: String,
    pub takeskip_input: String,
    
    // Load File editor state
    pub loadfile_name: String,
    pub loadfile_path: Option<PathBuf>,
    
    // Invert Bits editor state
    pub invert_name: String,
    
    // Multi-Worksheet Load editor state
    pub multiworksheet_name: String,
    pub multiworksheet_ops: Vec<(usize, String)>, // (worksheet_index, sequence_string)
    pub multiworksheet_input: String, // Temporary input for adding new worksheet operations
    pub multiworksheet_selected_worksheet: usize,
    
    // Pattern Locator state
    pub patterns: Vec<Pattern>,
    pub show_pattern_locator: bool,
    pub pattern_name_input: String,
    pub pattern_input: String,
    pub pattern_format: PatternFormat,
    pub pattern_garbles: usize,
    pub selected_pattern: Option<usize>,
    
    // Session restore state
    pub show_restore_dialog: bool,
    pub pending_session: Option<AppSession>,
    
    // Byte view column editor state
    pub show_column_editor: bool,
    pub column_editor_label: String,
    pub column_editor_bit_start: String,
    pub column_editor_bit_end: String,
    pub column_editor_color: [u8; 3],
}

impl Default for BitApp {
    fn default() -> Self {
        let mut worksheets = Vec::new();
        worksheets.push(Worksheet::new("Worksheet 1".to_string()));
        
        // Load settings from file
        let settings = AppSettings::auto_load();
        
        // Check if there's a previous session to restore
        let pending_session = AppSession::load().ok();
        let show_restore_dialog = pending_session.is_some();
        
        let mut viewer = BitViewer::new();
        viewer.shape = settings.bit_shape;
        viewer.show_grid = settings.show_grid;
        viewer.thick_grid_interval_horizontal = settings.thick_grid_interval_horizontal;
        viewer.thick_grid_interval_vertical = settings.thick_grid_interval_vertical;
        viewer.thick_grid_spacing_horizontal = settings.thick_grid_spacing_horizontal;
        viewer.thick_grid_spacing_vertical = settings.thick_grid_spacing_vertical;
        viewer.frame_length = settings.frame_length;
        
        Self {
            original_bits: BitVec::new(),
            processed_bits: BitVec::new(),
            viewer,
            byte_viewer: ByteViewer::new(),
            view_mode: ViewMode::Bit,
            operations: Vec::new(),
            current_file_path: None,
            error_message: None,
            show_original: true,
            show_settings: false,
            font_size: settings.font_size,
            settings,
            worksheets,
            current_worksheet_index: 0,
            renaming_worksheet: None,
            worksheet_name_buffer: String::new(),
            show_operation_menu: None,
            editing_operation_index: None,
            dragging_operation: None,
            takeskip_name: String::new(),
            takeskip_input: String::new(),
            loadfile_name: String::new(),
            loadfile_path: None,
            invert_name: String::new(),
            multiworksheet_name: String::new(),
            multiworksheet_ops: Vec::new(),
            multiworksheet_input: String::new(),
            multiworksheet_selected_worksheet: 0,
            patterns: Vec::new(),
            show_pattern_locator: false,
            pattern_name_input: String::new(),
            pattern_input: String::new(),
            pattern_format: PatternFormat::Bits,
            pattern_garbles: 0,
            selected_pattern: None,
            show_restore_dialog,
            pending_session,
            show_column_editor: false,
            column_editor_label: String::new(),
            column_editor_bit_start: String::from("0"),
            column_editor_bit_end: String::from("7"),
            column_editor_color: [100, 150, 200],
        }
    }
}

impl BitApp {
    pub fn current_worksheet(&self) -> &Worksheet {
        &self.worksheets[self.current_worksheet_index]
    }
    
    pub fn current_worksheet_mut(&mut self) -> &mut Worksheet {
        &mut self.worksheets[self.current_worksheet_index]
    }
    
    pub fn save_session(&self) {
        let session = AppSession::new(
            self.worksheets.clone(),
            self.current_worksheet_index,
        );
        
        if let Err(e) = session.save() {
            eprintln!("Failed to save session: {}", e);
        }
    }
    
    pub fn restore_session(&mut self, session: AppSession) {
        self.worksheets = session.worksheets;
        self.current_worksheet_index = session.current_worksheet_index.min(self.worksheets.len().saturating_sub(1));
        self.load_from_worksheet();
    }
    
    pub fn apply_operations(&mut self) {
        // Check if we have a MultiWorksheetLoad or LoadFile operation
        let has_multiworksheet = self.operations.iter().any(|op| matches!(op, BitOperation::MultiWorksheetLoad { .. }));
        let has_loadfile = self.operations.iter().any(|op| matches!(op, BitOperation::LoadFile { .. }));
        
        if has_multiworksheet || has_loadfile {
            // MultiWorksheetLoad or LoadFile creates new bits from scratch
            let mut result = BitVec::new();
            
            for op in &self.operations {
                match op {
                    BitOperation::LoadFile { file_path, .. } => {
                        // Load bits from the file
                        match read_file_as_bits(file_path) {
                            Ok(bits) => {
                                result.extend(bits);
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to load file: {}", e));
                                continue; // Skip if file can't be loaded
                            }
                        }
                    }
                    BitOperation::MultiWorksheetLoad { worksheet_operations, .. } => {
                        // Process each worksheet operation
                        for wo in worksheet_operations {
                            if wo.worksheet_index < self.worksheets.len() && wo.worksheet_index != self.current_worksheet_index {
                                // Get the source worksheet's processed bits (if it has a file loaded)
                                let source_bits = if let Some(file_path) = &self.worksheets[wo.worksheet_index].file_path {
                                    match read_file_as_bits(file_path) {
                                        Ok(bits) => bits,
                                        Err(e) => {
                                            self.error_message = Some(format!("Failed to load worksheet {}: {}", wo.worksheet_index + 1, e));
                                            continue; // Skip if file can't be loaded
                                        }
                                    }
                                } else {
                                    continue; // Skip if no file
                                };
                                
                                // Apply the sequence to these bits
                                let processed = wo.sequence.apply(&source_bits);
                                result.extend(processed);
                            }
                        }
                    }
                    _ => {
                        // Regular operations are applied to result so far
                        result = op.apply(&result);
                    }
                }
            }
            
            self.processed_bits = result;
            // When using MultiWorksheetLoad or LoadFile, automatically switch to viewing processed bits
            self.show_original = false;
        } else {
            // Normal operation: start with original bits and apply operations
            if self.original_bits.is_empty() {
                return;
            }

            let mut result = self.original_bits.clone();
            
            for op in &self.operations {
                result = op.apply(&result);
            }

            self.processed_bits = result;
        }
        
        self.update_viewer();
        self.sync_to_worksheet();
    }
    
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
    
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }
    
    pub fn sync_to_worksheet(&mut self) {
        let file_path = self.current_file_path.clone();
        let operations = self.operations.clone();
        let worksheet = self.current_worksheet_mut();
        worksheet.file_path = file_path;
        worksheet.operations = operations;
    }
    
    pub fn load_from_worksheet(&mut self) {
        let worksheet = self.current_worksheet().clone();
        
        // Load file if specified
        if let Some(path) = &worksheet.file_path {
            if path.exists() {
                match read_file_as_bits(path) {
                    Ok(bits) => {
                        self.original_bits = bits.clone();
                        self.processed_bits = bits;
                        self.current_file_path = Some(path.clone());
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load file: {}", e));
                    }
                }
            }
        } else {
            self.original_bits = BitVec::new();
            self.processed_bits = BitVec::new();
            self.current_file_path = None;
        }
        
        // Load operations
        self.operations = worksheet.operations.clone();
        self.apply_operations();
        self.update_viewer();
    }
    
    pub fn switch_worksheet(&mut self, index: usize) {
        if index < self.worksheets.len() {
            self.sync_to_worksheet();
            self.current_worksheet_index = index;
            self.load_from_worksheet();
        }
    }
    
    pub fn save_worksheet_to_file(&mut self) {
        self.sync_to_worksheet();
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Worksheet", &["json"])
            .save_file()
        {
            if let Err(e) = self.current_worksheet().save_to_file(&path) {
                self.error_message = Some(e);
            }
        }
    }
    
    pub fn load_worksheet_from_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Worksheet", &["json"])
            .pick_file()
        {
            match Worksheet::load_from_file(&path) {
                Ok(worksheet) => {
                    self.worksheets.push(worksheet);
                    self.current_worksheet_index = self.worksheets.len() - 1;
                    self.load_from_worksheet();
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(e);
                }
            }
        }
    }
    
    pub fn save_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let bits_to_save = if self.show_original {
                &self.original_bits
            } else {
                &self.processed_bits
            };

            match write_bits_to_file(&path, bits_to_save) {
                Ok(_) => {
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save file: {}", e));
                }
            }
        }
    }
    
    pub fn update_viewer(&mut self) {
        // Only update the bit viewer if we're in bit view mode
        // This prevents freezing when in Byte or ASCII view with large files
        if self.view_mode != ViewMode::Bit {
            return;
        }
        
        let bits_to_show = if self.show_original {
            &self.original_bits
        } else {
            &self.processed_bits
        };
        
        self.viewer.set_bits(bits_to_show.clone());
    }
    
    pub fn open_operation_creator(&mut self, op_type: OperationType) {
        self.show_operation_menu = Some(op_type);
        self.editing_operation_index = None;
        
        // Reset input fields
        self.takeskip_name.clear();
        self.takeskip_input.clear();
        self.loadfile_name.clear();
        self.loadfile_path = None;
        self.invert_name.clear();
        self.multiworksheet_name.clear();
        self.multiworksheet_ops.clear();
        self.multiworksheet_input.clear();
    }

    pub fn open_operation_editor(&mut self, index: usize) {
        if let Some(op) = self.operations.get(index) {
            match op {
                BitOperation::LoadFile { name, file_path } => {
                    self.show_operation_menu = Some(OperationType::LoadFile);
                    self.editing_operation_index = Some(index);
                    self.loadfile_name = name.clone();
                    self.loadfile_path = Some(file_path.clone());
                }
                BitOperation::TakeSkipSequence { name, sequence } => {
                    self.show_operation_menu = Some(OperationType::TakeSkipSequence);
                    self.editing_operation_index = Some(index);
                    self.takeskip_name = name.clone();
                    self.takeskip_input = sequence.to_string();
                }
                BitOperation::InvertBits { name } => {
                    self.show_operation_menu = Some(OperationType::InvertBits);
                    self.editing_operation_index = Some(index);
                    self.invert_name = name.clone();
                }
                BitOperation::MultiWorksheetLoad { name, worksheet_operations } => {
                    self.show_operation_menu = Some(OperationType::MultiWorksheetLoad);
                    self.editing_operation_index = Some(index);
                    self.multiworksheet_name = name.clone();
                    self.multiworksheet_ops = worksheet_operations
                        .iter()
                        .map(|wo| (wo.worksheet_index, wo.sequence.to_string()))
                        .collect();
                }
            }
        }
    }

    pub fn save_current_operation(&mut self) {
        if let Some(op_type) = self.show_operation_menu {
            let new_operation = match op_type {
                OperationType::LoadFile => {
                    if self.loadfile_path.is_none() {
                        self.error_message = Some("Please select a file to load".to_string());
                        return;
                    }
                    
                    let file_path = self.loadfile_path.clone().unwrap();
                    let name = if self.loadfile_name.trim().is_empty() {
                        format!("Load: {}", file_path.file_name().unwrap_or_default().to_string_lossy())
                    } else {
                        self.loadfile_name.clone()
                    };
                    
                    BitOperation::LoadFile {
                        name,
                        file_path,
                    }
                }
                OperationType::TakeSkipSequence => {
                    if self.takeskip_input.is_empty() {
                        self.error_message = Some("Operation sequence cannot be empty".to_string());
                        return;
                    }
                    
                    match OperationSequence::from_string(&self.takeskip_input) {
                        Ok(seq) => {
                            let name = if self.takeskip_name.trim().is_empty() {
                                format!("Sequence: {}", self.takeskip_input)
                            } else {
                                self.takeskip_name.clone()
                            };
                            
                            BitOperation::TakeSkipSequence {
                                name,
                                sequence: seq,
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Invalid operation: {}", e));
                            return;
                        }
                    }
                }
                OperationType::InvertBits => {
                    let name = if self.invert_name.trim().is_empty() {
                        "Invert All Bits".to_string()
                    } else {
                        self.invert_name.clone()
                    };
                    
                    BitOperation::InvertBits { name }
                }
                OperationType::MultiWorksheetLoad => {
                    if self.multiworksheet_ops.is_empty() {
                        self.error_message = Some("Must add at least one worksheet operation".to_string());
                        return;
                    }
                    
                    let mut worksheet_operations = Vec::new();
                    for (ws_idx, seq_str) in &self.multiworksheet_ops {
                        match OperationSequence::from_string(seq_str) {
                            Ok(seq) => {
                                worksheet_operations.push(WorksheetOperation {
                                    worksheet_index: *ws_idx,
                                    sequence: seq,
                                });
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Invalid sequence for worksheet {}: {}", ws_idx + 1, e));
                                return;
                            }
                        }
                    }
                    
                    let name = if self.multiworksheet_name.trim().is_empty() {
                        format!("Load from {} worksheets", worksheet_operations.len())
                    } else {
                        self.multiworksheet_name.clone()
                    };
                    
                    BitOperation::MultiWorksheetLoad {
                        name,
                        worksheet_operations,
                    }
                }
            };

            if let Some(index) = self.editing_operation_index {
                // Editing existing operation
                self.operations[index] = new_operation;
            } else {
                // Adding new operation
                self.operations.push(new_operation);
            }

            self.show_operation_menu = None;
            self.editing_operation_index = None;
            self.takeskip_name.clear();
            self.takeskip_input.clear();
            self.loadfile_name.clear();
            self.loadfile_path = None;
            self.invert_name.clear();
            self.multiworksheet_name.clear();
            self.multiworksheet_ops.clear();
            self.multiworksheet_input.clear();
            self.error_message = None;
            self.apply_operations();
        }
    }

    pub fn cancel_operation_edit(&mut self) {
        self.show_operation_menu = None;
        self.editing_operation_index = None;
        self.takeskip_name.clear();
        self.takeskip_input.clear();
        self.loadfile_name.clear();
        self.loadfile_path = None;
        self.invert_name.clear();
        self.multiworksheet_name.clear();
        self.multiworksheet_ops.clear();
        self.multiworksheet_input.clear();
    }
    
    pub fn render_ascii_view(&self, ui: &mut eframe::egui::Ui, bits: &BitVec<u8, Msb0>) {
        use eframe::egui;
        
        if bits.is_empty() {
            ui.label("No data to display");
            return;
        }

        // Calculate total size WITHOUT converting all bits
        let total_bits = bits.len();
        let total_bytes = (total_bits + 7) / 8;
        
        // Use frame_length (in bits) to determine characters per row
        // Each character represents 8 bits (1 byte)
        let chars_per_row = self.viewer.frame_length / 8;
        let total_rows = (total_bytes + chars_per_row - 1) / chars_per_row;
        
        let char_width = 12.0;
        let char_height = 20.0;
        let offset_width = if self.byte_viewer.config.show_hex_offset { 90.0 } else { 0.0 };
        
        egui::ScrollArea::vertical()
            .id_salt("ascii_viewer_scroll")
            .auto_shrink([false, false])
            .show_rows(
                ui,
                char_height,
                total_rows,
                |ui, row_range| {
                    egui::ScrollArea::horizontal()
                        .id_salt("ascii_viewer_horizontal")
                        .show(ui, |ui| {
                            // Only render visible rows
                            for row in row_range {
                                ui.horizontal(|ui| {
                                    // Show offset if enabled
                                    if self.byte_viewer.config.show_hex_offset {
                                        let offset = row * chars_per_row;
                                        ui.add_sized(
                                            [offset_width, char_height],
                                            egui::Label::new(
                                                egui::RichText::new(format!("{:08X}  ", offset))
                                                    .monospace()
                                                    .color(egui::Color32::GRAY)
                                            )
                                        );
                                    }
                                    
                                    // Draw ASCII characters - only convert bytes we need for this row
                                    let row_start = row * chars_per_row;
                                    let row_end = (row_start + chars_per_row).min(total_bytes);
                                    
                                    for byte_idx in row_start..row_end {
                                        // Convert only this single byte from bits
                                        let bit_start = byte_idx * 8;
                                        let bit_end = (bit_start + 8).min(total_bits);
                                        let byte_bits = &bits[bit_start..bit_end];
                                        
                                        let mut byte = 0u8;
                                        for (i, bit) in byte_bits.iter().enumerate() {
                                            if *bit {
                                                byte |= 1 << (7 - i);
                                            }
                                        }
                                        
                                        let ch = if byte >= 32 && byte <= 126 {
                                            byte as char
                                        } else {
                                            '.'
                                        };
                                        
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::Vec2::new(char_width, char_height),
                                            egui::Sense::hover(),
                                        );
                                        
                                        // Choose color based on character type
                                        let text_color = if byte >= 32 && byte <= 126 {
                                            egui::Color32::BLACK
                                        } else {
                                            egui::Color32::DARK_GRAY
                                        };
                                        
                                        // Draw character
                                        ui.painter().text(
                                            rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            ch,
                                            egui::FontId::monospace(16.0),
                                            text_color
                                        );
                                        
                                        // Show tooltip
                                        if response.hovered() {
                                            response.on_hover_ui(|ui| {
                                                ui.label(format!("Byte: {}", byte_idx));
                                                ui.label(format!("Value: 0x{:02X} ({})", byte, byte));
                                                ui.label(format!("ASCII: '{}'", ch));
                                                ui.label(format!("Binary: {:08b}", byte));
                                            });
                                        }
                                    }
                                });
                            }
                        });
                },
            );
    }
}
