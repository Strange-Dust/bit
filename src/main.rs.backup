mod bit_viewer;
mod byte_viewer;
mod file_io;
mod operations;
mod pattern_locator;
mod session;
mod settings;
mod worksheet;

use bit_viewer::{BitShape, BitViewer};
use byte_viewer::ByteViewer;
use bitvec::prelude::*;
use eframe::egui;
use file_io::{read_file_as_bits, write_bits_to_file};
use operations::{BitOperation, OperationSequence, WorksheetOperation};
use pattern_locator::{Pattern, PatternFormat};
use session::AppSession;
use settings::AppSettings;
use std::path::PathBuf;
use worksheet::Worksheet;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("B.I.T. - Bit Information Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "B.I.T.",
        options,
        Box::new(|_cc| Ok(Box::new(BitApp::default()))),
    )
}

// View mode for switching between bit and byte views
#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    Bit,
    Byte,
    Ascii,
}

// Available operation types that can be added
#[derive(Debug, Clone, Copy, PartialEq)]
enum OperationType {
    LoadFile,
    TakeSkipSequence,
    InvertBits,
    MultiWorksheetLoad,
    // Future operations:
    // FindPattern,
    // Replace,
    // etc.
}

impl OperationType {
    fn name(&self) -> &str {
        match self {
            OperationType::LoadFile => "Load File",
            OperationType::TakeSkipSequence => "Take/Skip Sequence",
            OperationType::InvertBits => "Invert Bits",
            OperationType::MultiWorksheetLoad => "Multi-Worksheet Load",
        }
    }

    fn icon(&self) -> &str {
        match self {
            OperationType::LoadFile => "📂",
            OperationType::TakeSkipSequence => "📝",
            OperationType::InvertBits => "🔄",
            OperationType::MultiWorksheetLoad => "📚",
        }
    }

    fn description(&self) -> &str {
        match self {
            OperationType::LoadFile => "Load bits from a file",
            OperationType::TakeSkipSequence => "Pattern-based bit extraction (t4r3i8s1)",
            OperationType::InvertBits => "Invert all bits (0→1, 1→0)",
            OperationType::MultiWorksheetLoad => "Load bits from multiple worksheets with operations",
        }
    }
}

struct BitApp {
    original_bits: BitVec<u8, Msb0>,
    processed_bits: BitVec<u8, Msb0>,
    viewer: BitViewer,
    byte_viewer: ByteViewer,
    view_mode: ViewMode,
    operations: Vec<BitOperation>,
    current_file_path: Option<PathBuf>,
    error_message: Option<String>,
    show_original: bool,
    show_settings: bool,
    font_size: f32,
    settings: AppSettings,
    
    // Worksheet management
    worksheets: Vec<Worksheet>,
    current_worksheet_index: usize,
    renaming_worksheet: Option<usize>,
    worksheet_name_buffer: String,
    
    // Operation creation/editing state
    show_operation_menu: Option<OperationType>,
    editing_operation_index: Option<usize>,
    
    // Drag and drop state
    dragging_operation: Option<usize>,
    
    // Take/Skip Sequence editor state
    takeskip_name: String,
    takeskip_input: String,
    
    // Load File editor state
    loadfile_name: String,
    loadfile_path: Option<PathBuf>,
    
    // Invert Bits editor state
    invert_name: String,
    
    // Multi-Worksheet Load editor state
    multiworksheet_name: String,
    multiworksheet_ops: Vec<(usize, String)>, // (worksheet_index, sequence_string)
    multiworksheet_input: String, // Temporary input for adding new worksheet operations
    multiworksheet_selected_worksheet: usize,
    
    // Pattern Locator state
    patterns: Vec<Pattern>,
    show_pattern_locator: bool,
    pattern_name_input: String,
    pattern_input: String,
    pattern_format: PatternFormat,
    pattern_garbles: usize,
    selected_pattern: Option<usize>,
    
    // Session restore state
    show_restore_dialog: bool,
    pending_session: Option<AppSession>,
    
    // Byte view column editor state
    show_column_editor: bool,
    column_editor_label: String,
    column_editor_bit_start: String,
    column_editor_bit_end: String,
    column_editor_color: [u8; 3],
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
    fn current_worksheet(&self) -> &Worksheet {
        &self.worksheets[self.current_worksheet_index]
    }
    
    fn current_worksheet_mut(&mut self) -> &mut Worksheet {
        &mut self.worksheets[self.current_worksheet_index]
    }
    
    fn save_session(&self) {
        let session = AppSession::new(
            self.worksheets.clone(),
            self.current_worksheet_index,
        );
        
        if let Err(e) = session.save() {
            eprintln!("Failed to save session: {}", e);
        }
    }
    
    fn restore_session(&mut self, session: AppSession) {
        self.worksheets = session.worksheets;
        self.current_worksheet_index = session.current_worksheet_index.min(self.worksheets.len().saturating_sub(1));
        self.load_from_worksheet();
    }
    
    fn sync_to_worksheet(&mut self) {
        let file_path = self.current_file_path.clone();
        let operations = self.operations.clone();
        let worksheet = self.current_worksheet_mut();
        worksheet.file_path = file_path;
        worksheet.operations = operations;
    }
    
    fn load_from_worksheet(&mut self) {
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
    
    fn switch_worksheet(&mut self, index: usize) {
        if index < self.worksheets.len() {
            self.sync_to_worksheet();
            self.current_worksheet_index = index;
            self.load_from_worksheet();
        }
    }
    
    fn save_worksheet_to_file(&mut self) {
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
    
    fn load_worksheet_from_file(&mut self) {
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
    
    // Deprecated: Use LoadFile operation instead
    // fn load_file(&mut self) {
    //     if let Some(path) = rfd::FileDialog::new().pick_file() {
    //         match read_file_as_bits(&path) {
    //             Ok(bits) => {
    //                 self.original_bits = bits.clone();
    //                 self.processed_bits = bits;
    //                 self.current_file_path = Some(path);
    //                 self.error_message = None;
    //                 self.update_viewer();
    //                 self.sync_to_worksheet();
    //             }
    //             Err(e) => {
    //                 self.error_message = Some(format!("Failed to load file: {}", e));
    //             }
    //         }
    //     }
    // }

    fn save_file(&mut self) {
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

    fn apply_operations(&mut self) {
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

    fn update_viewer(&mut self) {
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

    fn render_ascii_view(&self, ui: &mut egui::Ui, bits: &BitVec<u8, Msb0>) {
        if bits.is_empty() {
            ui.label("No data to display");
            return;
        }

        // Limit rendering to prevent freezing with large files
        // Max 1 million bits (125KB) to keep UI responsive
        const MAX_BITS_TO_RENDER: usize = 1_000_000;
        
        let bits_to_render = if bits.len() > MAX_BITS_TO_RENDER {
            ui.colored_label(
                egui::Color32::from_rgb(255, 200, 0),
                format!("⚠ Large file detected ({:.2} MB). Showing first {} KB only.", 
                    bits.len() as f64 / 8.0 / 1024.0 / 1024.0,
                    MAX_BITS_TO_RENDER / 8 / 1024)
            );
            ui.label("Tip: Use Byte view for better performance with large files");
            ui.separator();
            &bits[..MAX_BITS_TO_RENDER]
        } else {
            bits
        };

        // Convert bits to bytes
        let mut bytes = Vec::new();
        for chunk in bits_to_render.chunks(8) {
            let mut byte = 0u8;
            for (i, bit) in chunk.iter().enumerate() {
                if *bit {
                    byte |= 1 << (7 - i);
                }
            }
            bytes.push(byte);
        }

        // Use frame_length (in bits) to determine characters per row
        // Each character represents 8 bits (1 byte)
        let chars_per_row = self.viewer.frame_length / 8;
        let char_width = 12.0;
        let char_height = 20.0;
        
        egui::ScrollArea::vertical()
            .show(ui, |ui| {
                egui::ScrollArea::horizontal()
                    .show(ui, |ui| {
                        let total_rows = (bytes.len() + chars_per_row - 1) / chars_per_row;
                        
                        for row in 0..total_rows {
                            ui.horizontal(|ui| {
                                // Show offset if enabled
                                if self.byte_viewer.config.show_hex_offset {
                                    let offset = row * chars_per_row;
                                    ui.label(
                                        egui::RichText::new(format!("{:08X}  ", offset))
                                            .monospace()
                                            .color(egui::Color32::GRAY)
                                    );
                                }
                                
                                // Draw ASCII characters
                                let row_start = row * chars_per_row;
                                let row_end = (row_start + chars_per_row).min(bytes.len());
                                
                                for byte_idx in row_start..row_end {
                                    let byte = bytes[byte_idx];
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
            });
    }

    fn open_operation_creator(&mut self, op_type: OperationType) {
        self.show_operation_menu = Some(op_type);
        self.editing_operation_index = None;
        
        // Reset input fields
        self.takeskip_name.clear();
        self.takeskip_input.clear();
        self.loadfile_name.clear();
        self.loadfile_path = None;
    }

    fn open_operation_editor(&mut self, index: usize) {
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

    fn save_current_operation(&mut self) {
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

    fn cancel_operation_edit(&mut self) {
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
}

impl eframe::App for BitApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Auto-save session when closing
        self.save_session();
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Disable text selection while dragging
        if self.dragging_operation.is_some() {
            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }
        
        // Handle Ctrl+Shift+Mouse Wheel for font size adjustment
        // This needs to be checked before any UI elements consume the scroll
        let mut font_size_changed = false;
        ctx.input(|i| {
            // Check if BOTH Ctrl and Shift are being held down
            let ctrl_shift_held = i.modifiers.ctrl && i.modifiers.shift;
            
            if ctrl_shift_held {
                // Combine both scroll delta types
                let delta = i.smooth_scroll_delta.y + i.raw_scroll_delta.y;
                
                if delta.abs() > 0.01 {
                    // Positive scroll = zoom in, negative = zoom out
                    // Increased sensitivity for more responsive feel
                    let sensitivity = 0.1;
                    self.font_size = (self.font_size + delta * sensitivity).clamp(8.0, 24.0);
                    font_size_changed = true;
                }
            }
        });
        
        // Request repaint if font size changed for immediate visual feedback
        if font_size_changed {
            ctx.request_repaint();
        }

        // Apply font size to the context
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(self.font_size * 0.85, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(self.font_size * 1.3, egui::FontFamily::Proportional),
        );
        ctx.set_style(style);

        // Show restore session dialog if there's a pending session
        if self.show_restore_dialog {
            egui::Window::new("Restore Previous Session?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("A previous session was found.");
                        ui.label("Would you like to restore it or start fresh?");
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("🔄 Restore Session").clicked() {
                                if let Some(session) = self.pending_session.take() {
                                    self.restore_session(session);
                                }
                                self.show_restore_dialog = false;
                            }
                            
                            if ui.button("🆕 Start Fresh").clicked() {
                                let _ = AppSession::delete();
                                self.pending_session = None;
                                self.show_restore_dialog = false;
                            }
                        });
                    });
                });
        }

        // Top panel with title and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔧 B.I.T. - Bit Information Tool");
                
                ui.separator();

                if ui.button("💾 Save File").clicked() {
                    self.save_file();
                }

                ui.separator();

                if ui.button("⚙ Settings").clicked() {
                    self.show_settings = !self.show_settings;
                }

                if ui.button("🔍 Pattern Locator").clicked() {
                    self.show_pattern_locator = !self.show_pattern_locator;
                }

                ui.separator();

                // View mode toggle
                ui.label("View:");
                if ui.selectable_label(self.view_mode == ViewMode::Bit, "⬛ Bit").clicked() {
                    self.view_mode = ViewMode::Bit;
                }
                if ui.selectable_label(self.view_mode == ViewMode::Byte, "📊 Byte").clicked() {
                    self.view_mode = ViewMode::Byte;
                }
                if ui.selectable_label(self.view_mode == ViewMode::Ascii, "🔤 ASCII").clicked() {
                    self.view_mode = ViewMode::Ascii;
                }

                ui.separator();

                ui.label("Zoom:");
                if ui.button("➕").clicked() {
                    self.viewer.zoom_in();
                }
                if ui.button("➖").clicked() {
                    self.viewer.zoom_out();
                }
                if ui.button("🔄").clicked() {
                    self.viewer.reset_zoom();
                }

                ui.separator();

                if ui.selectable_label(self.show_original, "Original").clicked() {
                    self.show_original = true;
                    if self.view_mode == ViewMode::Bit {
                        self.update_viewer();
                    }
                }
                if ui.selectable_label(!self.show_original, "Processed").clicked() {
                    self.show_original = false;
                    if self.view_mode == ViewMode::Bit {
                        self.update_viewer();
                    }
                }
            });
        });

        // Leftmost panel: Available Operations
        egui::SidePanel::left("available_operations_panel")
            .default_width(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Available Operations");
                ui.separator();
                
                egui::ScrollArea::vertical()
                    .id_salt("available_ops")
                    .show(ui, |ui| {
                        let operations = [
                            OperationType::LoadFile,
                            OperationType::TakeSkipSequence,
                            OperationType::InvertBits,
                            OperationType::MultiWorksheetLoad,
                        ];
                        
                        for &op_type in &operations {
                            if ui.button(format!("{} {}", op_type.icon(), op_type.name()))
                                .on_hover_text(op_type.description())
                                .clicked() 
                            {
                                self.open_operation_creator(op_type);
                            }
                            ui.add_space(4.0);
                        }
                        
                        ui.separator();
                        ui.label("💡 Click an operation type");
                        ui.label("to add it to the list");
                    });
            });

        // Middle panel: Worksheets and Active Operations
        egui::SidePanel::left("active_operations_panel")
            .default_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                // Worksheets section
                ui.heading("Worksheets");
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("➕").clicked() {
                        let new_name = format!("Worksheet {}", self.worksheets.len() + 1);
                        self.sync_to_worksheet();
                        self.worksheets.push(Worksheet::new(new_name));
                        self.current_worksheet_index = self.worksheets.len() - 1;
                        self.load_from_worksheet();
                    }
                    
                    if ui.button("💾 Save").clicked() {
                        self.save_worksheet_to_file();
                    }
                    
                    if ui.button("📂 Load").clicked() {
                        self.load_worksheet_from_file();
                    }
                });
                
                ui.add_space(4.0);
                
                // Compact worksheet list
                egui::ScrollArea::vertical()
                    .id_salt("worksheets")
                    .max_height(120.0)
                    .show(ui, |ui| {
                        let mut to_switch = None;
                        let mut to_delete = None;
                        let num_worksheets = self.worksheets.len();
                        
                        for i in 0..num_worksheets {
                            let is_current = i == self.current_worksheet_index;
                            let worksheet_name = self.worksheets[i].name.clone();
                            
                            ui.horizontal(|ui| {
                                // Selection button
                                let mut response = ui.selectable_label(is_current, "");
                                if response.clicked() && !is_current {
                                    to_switch = Some(i);
                                }
                                
                                // Editable name
                                if self.renaming_worksheet == Some(i) {
                                    let text_response = ui.text_edit_singleline(&mut self.worksheet_name_buffer);
                                    if text_response.lost_focus() {
                                        self.worksheets[i].name = self.worksheet_name_buffer.clone();
                                        self.renaming_worksheet = None;
                                    }
                                } else {
                                    response = ui.label(&worksheet_name);
                                    if response.clicked() && !is_current {
                                        to_switch = Some(i);
                                    }
                                    if response.double_clicked() {
                                        self.renaming_worksheet = Some(i);
                                        self.worksheet_name_buffer = worksheet_name.clone();
                                    }
                                }
                                
                                // Delete button (can't delete if only one worksheet)
                                if num_worksheets > 1 && ui.small_button("🗑").clicked() {
                                    to_delete = Some(i);
                                }
                            });
                        }
                        
                        if let Some(idx) = to_switch {
                            self.switch_worksheet(idx);
                        }
                        
                        if let Some(idx) = to_delete {
                            self.worksheets.remove(idx);
                            if self.current_worksheet_index >= self.worksheets.len() {
                                self.current_worksheet_index = self.worksheets.len() - 1;
                            }
                            if self.current_worksheet_index == idx || idx < self.current_worksheet_index {
                                self.load_from_worksheet();
                            }
                        }
                    });
                
                ui.separator();
                
                ui.heading("Active Operations");
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("active_ops")
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        let mut to_edit = None;

                        if self.operations.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label("No operations added");
                            });
                        } else {
                            // Track potential drop position
                            let mut drop_target_idx = None;
                            
                            for (i, op) in self.operations.iter().enumerate() {
                                let is_being_dragged = self.dragging_operation == Some(i);
                                
                                // Make dragged item semi-transparent
                                let alpha = if is_being_dragged { 0.3 } else { 1.0 };
                                
                                let mut show_drop_indicator_above = false;
                                let mut show_drop_indicator_below = false;
                                
                                ui.scope(|ui| {
                                    if is_being_dragged {
                                        ui.style_mut().visuals.widgets.inactive.bg_fill = 
                                            ui.style().visuals.widgets.inactive.bg_fill.linear_multiply(alpha);
                                        ui.style_mut().visuals.widgets.noninteractive.bg_fill = 
                                            ui.style().visuals.widgets.noninteractive.bg_fill.linear_multiply(alpha);
                                    }
                                    
                                    let response = ui.group(|ui| {
                                        ui.set_min_width(ui.available_width());
                                        
                                        // Disable text selection for this group
                                        ui.style_mut().interaction.selectable_labels = false;
                                        
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                // Drag handle
                                                let drag_handle = ui.label("☰").interact(egui::Sense::click_and_drag());
                                                
                                                ui.label(format!("{}.", i + 1));
                                                ui.vertical(|ui| {
                                                    ui.label(op.name());
                                                    ui.small(op.description());
                                                });
                                                
                                                // Start dragging on the drag handle
                                                if drag_handle.dragged() {
                                                    self.dragging_operation = Some(i);
                                                }
                                            });
                                            
                                            ui.horizontal(|ui| {
                                                if ui.button("✏ Edit").clicked() {
                                                    to_edit = Some(i);
                                                }
                                                
                                                if ui.button("🗑").clicked() {
                                                    to_remove = Some(i);
                                                }
                                            });
                                        });
                                    }).response;
                                    
                                    // Check if we're hovering over this item while dragging
                                    if let Some(dragged_idx) = self.dragging_operation {
                                        if dragged_idx != i {
                                            // Check if pointer is over this item
                                            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                                                let rect = response.rect;
                                                
                                                // Check if pointer is within this item's rect
                                                if rect.contains(pointer_pos) {
                                                    let mid_y = rect.center().y;
                                                    
                                                    if pointer_pos.y < mid_y {
                                                        // Drop before this item
                                                        drop_target_idx = Some(i);
                                                        show_drop_indicator_above = true;
                                                    } else {
                                                        // Drop after this item
                                                        drop_target_idx = Some(i + 1);
                                                        show_drop_indicator_below = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Draw drop indicators AFTER the group so they're on top
                                    if show_drop_indicator_above {
                                        let rect = response.rect;
                                        let line_rect = egui::Rect::from_min_size(
                                            egui::pos2(rect.min.x - 5.0, rect.min.y - 3.0),
                                            egui::vec2(rect.width() + 10.0, 6.0)
                                        );
                                        ui.painter().rect_filled(
                                            line_rect,
                                            3.0,
                                            egui::Color32::from_rgb(70, 130, 255)
                                        );
                                        ui.painter().rect_stroke(
                                            line_rect,
                                            3.0,
                                            egui::Stroke::new(2.0, egui::Color32::from_rgb(150, 200, 255)),
                                            egui::epaint::StrokeKind::Middle
                                        );
                                    }
                                    
                                    if show_drop_indicator_below {
                                        let rect = response.rect;
                                        let line_rect = egui::Rect::from_min_size(
                                            egui::pos2(rect.min.x - 5.0, rect.max.y - 1.0),
                                            egui::vec2(rect.width() + 10.0, 6.0)
                                        );
                                        ui.painter().rect_filled(
                                            line_rect,
                                            3.0,
                                            egui::Color32::from_rgb(70, 130, 255)
                                        );
                                        ui.painter().rect_stroke(
                                            line_rect,
                                            3.0,
                                            egui::Stroke::new(2.0, egui::Color32::from_rgb(150, 200, 255)),
                                            egui::epaint::StrokeKind::Middle
                                        );
                                    }
                                    
                                    // Handle drag visual feedback
                                    if is_being_dragged {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                        
                                        // Draw floating preview at cursor
                                        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                                            let preview_rect = egui::Rect::from_min_size(
                                                pointer_pos + egui::vec2(10.0, 10.0),
                                                egui::vec2(200.0, 40.0)
                                            );
                                            
                                            ui.painter().rect_filled(
                                                preview_rect,
                                                4.0,
                                                egui::Color32::from_rgba_unmultiplied(60, 60, 80, 230)
                                            );
                                            ui.painter().rect_stroke(
                                                preview_rect,
                                                4.0,
                                                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 150, 255)),
                                                egui::epaint::StrokeKind::Middle
                                            );
                                            ui.painter().text(
                                                preview_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                format!("{}. {}", i + 1, op.name()),
                                                egui::FontId::default(),
                                                egui::Color32::WHITE
                                            );
                                        }
                                    }
                                });
                                
                                ui.add_space(4.0);
                            }
                            
                            // Stop dragging when mouse is released
                            if ui.input(|i| i.pointer.primary_released()) {
                                if let Some(from) = self.dragging_operation {
                                    if let Some(to) = drop_target_idx {
                                        // Move the operation
                                        println!("Moving from {} to {}", from, to);
                                        if from != to {
                                            let op = self.operations.remove(from);
                                            // Adjust insert position based on whether we're moving forward or backward
                                            let insert_pos = if to > from { to - 1 } else { to };
                                            println!("Actually inserting at {}", insert_pos);
                                            self.operations.insert(insert_pos, op);
                                            self.apply_operations();
                                        }
                                    }
                                }
                                self.dragging_operation = None;
                            }
                        }

                        if let Some(idx) = to_remove {
                            self.operations.remove(idx);
                            self.apply_operations();
                        }
                        
                        if let Some(idx) = to_edit {
                            self.open_operation_editor(idx);
                        }
                    });

                if !self.operations.is_empty() {
                    ui.separator();
                    if ui.button("🔄 Reapply All").clicked() {
                        self.apply_operations();
                    }
                    if ui.button("🗑 Clear All").clicked() {
                        self.operations.clear();
                        self.processed_bits = self.original_bits.clone();
                        self.update_viewer();
                    }
                }

                // Byte View Configuration (only shown in byte view mode)
                if self.view_mode == ViewMode::Byte {
                    ui.separator();
                    ui.heading("📊 Byte View Config");
                    
                    ui.horizontal(|ui| {
                        ui.label("Bytes per row:");
                        let mut bytes_per_row = self.byte_viewer.config.bytes_per_row;
                        if ui.add(egui::Slider::new(&mut bytes_per_row, 1..=64)).changed() {
                            self.byte_viewer.set_bytes_per_row(bytes_per_row);
                        }
                    });
                    
                    ui.checkbox(&mut self.byte_viewer.config.show_hex_offset, "Show hex offset");
                    
                    ui.add_space(8.0);
                    ui.strong("Protocol Columns");
                    ui.label("Define labeled regions for protocol analysis");
                    
                    egui::ScrollArea::vertical()
                        .id_salt("byte_columns")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let mut to_remove = None;
                            
                            for (idx, column) in self.byte_viewer.config.columns.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        // Color indicator
                                        let color_rect = ui.allocate_rect(
                                            egui::Rect::from_min_size(
                                                ui.cursor().min,
                                                egui::vec2(20.0, 20.0)
                                            ),
                                            egui::Sense::hover()
                                        );
                                        ui.painter().rect_filled(
                                            color_rect.rect,
                                            3.0,
                                            column.color32()
                                        );
                                        
                                        ui.vertical(|ui| {
                                            ui.label(&column.label);
                                            ui.small(format!("Bits {}..{}", column.bit_start, column.bit_end));
                                        });
                                        
                                        if ui.button("🗑").clicked() {
                                            to_remove = Some(idx);
                                        }
                                    });
                                });
                            }
                            
                            if let Some(idx) = to_remove {
                                self.byte_viewer.remove_column(idx);
                            }
                        });
                    
                    ui.horizontal(|ui| {
                        if ui.button("➕ Add Column").clicked() {
                            self.show_column_editor = true;
                        }
                        
                        if ui.button("💾 Save Config").clicked() {
                            if let Some(file_path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("protocol_config.json")
                                .save_file() {
                                match serde_json::to_string_pretty(&self.byte_viewer.config) {
                                    Ok(json) => {
                                        if let Err(e) = std::fs::write(&file_path, json) {
                                            self.error_message = Some(format!("Failed to save config: {}", e));
                                        }
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("Failed to serialize config: {}", e));
                                    }
                                }
                            }
                        }
                        
                        if ui.button("📂 Load Config").clicked() {
                            if let Some(file_path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .pick_file() {
                                match std::fs::read_to_string(&file_path) {
                                    Ok(json) => {
                                        match serde_json::from_str(&json) {
                                            Ok(config) => {
                                                self.byte_viewer.config = config;
                                            }
                                            Err(e) => {
                                                self.error_message = Some(format!("Failed to parse config: {}", e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("Failed to read config file: {}", e));
                                    }
                                }
                            }
                        }
                    });
                    
                    if ui.button("📄 Export Documentation").clicked() {
                        if let Some(file_path) = rfd::FileDialog::new()
                            .add_filter("Text", &["txt"])
                            .set_file_name("protocol_documentation.txt")
                            .save_file() {
                            let mut doc = String::new();
                            doc.push_str("Protocol Documentation\n");
                            doc.push_str("=====================\n\n");
                            doc.push_str(&format!("Bytes per row: {}\n\n", self.byte_viewer.config.bytes_per_row));
                            doc.push_str("Field Definitions:\n");
                            doc.push_str("------------------\n\n");
                            
                            for (idx, column) in self.byte_viewer.config.columns.iter().enumerate() {
                                doc.push_str(&format!("{}. {}\n", idx + 1, column.label));
                                doc.push_str(&format!("   Bit Range: {} - {}\n", column.bit_start, column.bit_end));
                                let (start_byte, end_byte) = column.byte_range(self.byte_viewer.config.bytes_per_row);
                                doc.push_str(&format!("   Byte Range: {} - {}\n", start_byte, end_byte));
                                doc.push_str(&format!("   Color: RGB({}, {}, {})\n\n", 
                                    column.color[0], column.color[1], column.color[2]));
                            }
                            
                            if let Err(e) = std::fs::write(&file_path, doc) {
                                self.error_message = Some(format!("Failed to export documentation: {}", e));
                            }
                        }
                    }
                }

                ui.separator();

                if let Some(path) = &self.current_file_path {
                    ui.label(format!("📄 {}", path.file_name().unwrap_or_default().to_string_lossy()));
                }
                ui.label(format!("Original: {} bits", self.original_bits.len()));
                ui.label(format!("Processed: {} bits", self.processed_bits.len()));
                ui.label(format!("Bit size: {:.1}px", self.viewer.bit_size));
            });
        
        // Bottom panel with frame length control
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Frame Length (bits per row):");
                if ui.add(egui::Slider::new(&mut self.viewer.frame_length, 8..=512).logarithmic(true)).changed() {
                    self.settings.frame_length = self.viewer.frame_length;
                    self.settings.auto_save();
                }
                ui.label(format!("({} chars in ASCII view)", self.viewer.frame_length / 8));
            });
        });

        // Settings Window
        if self.show_settings {
            egui::Window::new("⚙ Settings")
                .open(&mut self.show_settings)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Display Settings");
                    ui.separator();

                    // Shape selector
                    ui.label("Bit Shape:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.viewer.shape == BitShape::Square, "⬛ Square").clicked() {
                            self.viewer.shape = BitShape::Square;
                            self.settings.bit_shape = BitShape::Square;
                            self.settings.auto_save();
                        }
                        if ui.selectable_label(self.viewer.shape == BitShape::Circle, "⚫ Circle").clicked() {
                            self.viewer.shape = BitShape::Circle;
                            self.settings.bit_shape = BitShape::Circle;
                            self.settings.auto_save();
                        }
                        if ui.selectable_label(self.viewer.shape == BitShape::Octagon, "⬢ Octagon").clicked() {
                            self.viewer.shape = BitShape::Octagon;
                            self.settings.bit_shape = BitShape::Octagon;
                            self.settings.auto_save();
                        }
                    });

                    ui.separator();

                    // Grid settings
                    if ui.checkbox(&mut self.viewer.show_grid, "Show Grid Lines").changed() {
                        self.settings.show_grid = self.viewer.show_grid;
                        self.settings.auto_save();
                    }
                    ui.label("Toggle the grid lines around each bit");

                    ui.add_space(8.0);

                    ui.label("Thick Grid Interval (Horizontal):");
                    if ui.add(egui::Slider::new(&mut self.viewer.thick_grid_interval_horizontal, 0..=64)
                        .text("bits")).changed() {
                        self.settings.thick_grid_interval_horizontal = self.viewer.thick_grid_interval_horizontal;
                        self.settings.auto_save();
                    }
                    ui.label("Thicker vertical lines every N bits horizontally (0 = off)");

                    ui.add_space(4.0);

                    ui.label("Thick Grid Interval (Vertical):");
                    if ui.add(egui::Slider::new(&mut self.viewer.thick_grid_interval_vertical, 0..=64)
                        .text("bits")).changed() {
                        self.settings.thick_grid_interval_vertical = self.viewer.thick_grid_interval_vertical;
                        self.settings.auto_save();
                    }
                    ui.label("Thicker horizontal lines every N bits vertically (0 = off)");

                    ui.add_space(4.0);

                    ui.label("Thick Grid Spacing (Horizontal):");
                    if ui.add(egui::Slider::new(&mut self.viewer.thick_grid_spacing_horizontal, 0.0..=10.0)
                        .text("pixels")).changed() {
                        self.settings.thick_grid_spacing_horizontal = self.viewer.thick_grid_spacing_horizontal;
                        self.settings.auto_save();
                    }
                    ui.label("Horizontal gap size (vertical line spacing)");

                    ui.add_space(4.0);

                    ui.label("Thick Grid Spacing (Vertical):");
                    if ui.add(egui::Slider::new(&mut self.viewer.thick_grid_spacing_vertical, 0.0..=10.0)
                        .text("pixels")).changed() {
                        self.settings.thick_grid_spacing_vertical = self.viewer.thick_grid_spacing_vertical;
                        self.settings.auto_save();
                    }
                    ui.label("Vertical gap size (horizontal line spacing)");

                    ui.separator();

                    // Font size setting
                    ui.label("GUI Font Size:");
                    if ui.add(egui::Slider::new(&mut self.font_size, 8.0..=24.0)
                        .text("pixels")).changed() {
                        self.settings.font_size = self.font_size;
                        self.settings.auto_save();
                    }
                    ui.label("Adjust the size of all interface text");

                    ui.separator();
                    
                    // Settings management buttons
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Settings").clicked() {
                            if let Some(file_path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .set_file_name("settings.json")
                                .save_file() {
                                if let Err(e) = self.settings.save_to_file(&file_path) {
                                    self.error_message = Some(format!("Failed to save settings: {}", e));
                                }
                            }
                        }
                        if ui.button("📂 Load Settings").clicked() {
                            if let Some(file_path) = rfd::FileDialog::new()
                                .add_filter("JSON", &["json"])
                                .pick_file() {
                                match AppSettings::load_from_file(&file_path) {
                                    Ok(loaded_settings) => {
                                        self.settings = loaded_settings;
                                        // Apply loaded settings to viewer
                                        self.viewer.shape = self.settings.bit_shape;
                                        self.viewer.show_grid = self.settings.show_grid;
                                        self.viewer.thick_grid_interval_horizontal = self.settings.thick_grid_interval_horizontal;
                                        self.viewer.thick_grid_interval_vertical = self.settings.thick_grid_interval_vertical;
                                        self.viewer.thick_grid_spacing_horizontal = self.settings.thick_grid_spacing_horizontal;
                                        self.viewer.thick_grid_spacing_vertical = self.settings.thick_grid_spacing_vertical;
                                        self.viewer.frame_length = self.settings.frame_length;
                                        self.font_size = self.settings.font_size;
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("Failed to load settings: {}", e));
                                    }
                                }
                            }
                        }
                        if ui.button("🔄 Reset to Defaults").clicked() {
                            self.settings = AppSettings::default();
                            // Apply default settings to viewer
                            self.viewer.shape = self.settings.bit_shape;
                            self.viewer.show_grid = self.settings.show_grid;
                            self.viewer.thick_grid_interval_horizontal = self.settings.thick_grid_interval_horizontal;
                            self.viewer.thick_grid_interval_vertical = self.settings.thick_grid_interval_vertical;
                            self.viewer.thick_grid_spacing_horizontal = self.settings.thick_grid_spacing_horizontal;
                            self.viewer.thick_grid_spacing_vertical = self.settings.thick_grid_spacing_vertical;
                            self.viewer.frame_length = self.settings.frame_length;
                            self.font_size = self.settings.font_size;
                            self.settings.auto_save();
                        }
                    });
                    
                    ui.separator();
                    
                    ui.label("💡 Tips:");
                    ui.label("• Grid lines help distinguish individual bits");
                    ui.label("• Thick intervals are useful for byte boundaries");
                    ui.label("• Try interval of 8 for byte alignment");
                    ui.label("• Increase spacing for more visible separation");
                    ui.label("• Settings auto-save when changed");
                });
        }

        // Pattern Locator Window
        if self.show_pattern_locator {
            egui::Window::new("🔍 Pattern Locator")
                .open(&mut self.show_pattern_locator)
                .default_width(450.0)
                .default_height(700.0)
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.heading("Search for Bit Patterns");
                            ui.separator();
                            
                            // Pattern input section
                            ui.group(|ui| {
                        ui.heading("Add Pattern");
                        
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.pattern_name_input);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Format:");
                            ui.selectable_value(&mut self.pattern_format, PatternFormat::Bits, "Bits (0/1)");
                            ui.selectable_value(&mut self.pattern_format, PatternFormat::Hex, "Hex (0x...)");
                            ui.selectable_value(&mut self.pattern_format, PatternFormat::Ascii, "ASCII");
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Pattern:");
                            ui.text_edit_singleline(&mut self.pattern_input);
                        });
                        ui.label(match self.pattern_format {
                            PatternFormat::Hex => "Example: 0xFF or 0x1A2B",
                            PatternFormat::Ascii => "Example: Hello",
                            PatternFormat::Bits => "Example: 11100101",
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Garbles allowed:");
                            ui.add(egui::Slider::new(&mut self.pattern_garbles, 0..=16).text("bits"));
                        });
                        ui.label("Number of bit differences tolerated in matches");
                        
                        ui.horizontal(|ui| {
                            if ui.button("➕ Add Pattern").clicked() {
                                let name = if self.pattern_name_input.is_empty() {
                                    format!("Pattern {}", self.patterns.len() + 1)
                                } else {
                                    self.pattern_name_input.clone()
                                };
                                
                                match Pattern::new(name, self.pattern_format, self.pattern_input.clone(), self.pattern_garbles) {
                                    Ok(pattern) => {
                                        self.patterns.push(pattern);
                                        self.pattern_name_input.clear();
                                        self.pattern_input.clear();
                                        self.error_message = None;
                                    }
                                    Err(e) => {
                                        self.error_message = Some(format!("Invalid pattern: {}", e));
                                    }
                                }
                            }
                            
                            if ui.button("🔄 Clear").clicked() {
                                self.pattern_name_input.clear();
                                self.pattern_input.clear();
                                self.pattern_garbles = 0;
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    // Pattern list
                    ui.heading("Patterns");
                    
                    if self.patterns.is_empty() {
                        ui.label("No patterns added yet");
                    } else {
                        let mut to_remove = None;
                        let mut to_search = None;
                        
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for (idx, pattern) in self.patterns.iter().enumerate() {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            let selected = self.selected_pattern == Some(idx);
                                            if ui.selectable_label(selected, &pattern.name).clicked() {
                                                self.selected_pattern = Some(idx);
                                            }
                                            
                                            ui.label(format!("({})", pattern.format.name()));
                                            
                                            if ui.button("🔍 Search").clicked() {
                                                to_search = Some(idx);
                                            }
                                            
                                            if ui.button("❌").clicked() {
                                                to_remove = Some(idx);
                                            }
                                        });
                                        
                                        ui.label(format!("Pattern: {}", pattern.input));
                                        ui.label(format!("Garbles: {} | Matches: {}", pattern.garbles, pattern.matches.len()));
                                    });
                                }
                            });
                        
                        if let Some(idx) = to_remove {
                            self.patterns.remove(idx);
                            if self.selected_pattern == Some(idx) {
                                self.selected_pattern = None;
                            }
                        }
                        
                        if let Some(idx) = to_search {
                            let bits_to_search = if self.show_original {
                                &self.original_bits
                            } else {
                                &self.processed_bits
                            };
                            self.patterns[idx].search(bits_to_search);
                            self.selected_pattern = Some(idx);
                        }
                    }
                    
                    ui.separator();
                    
                    // Search results
                    if let Some(pattern_idx) = self.selected_pattern {
                        if pattern_idx < self.patterns.len() {
                            let pattern = &self.patterns[pattern_idx];
                            
                            ui.heading(format!("Results for '{}'", pattern.name));
                            ui.label(format!("Found {} matches", pattern.matches.len()));
                            
                            if pattern.matches.is_empty() {
                                ui.label("No matches found. Try searching with the 🔍 Search button.");
                            } else {
                                ui.horizontal(|ui| {
                                    if ui.button("🎯 Highlight All").clicked() {
                                        self.viewer.clear_highlights();
                                        for m in &pattern.matches {
                                            self.viewer.add_highlight_range(m.position, pattern.bits.len());
                                        }
                                    }
                                    
                                    if ui.button("🔲 Clear Highlights").clicked() {
                                        self.viewer.clear_highlights();
                                    }
                                });
                                
                                ui.separator();
                                
                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .show(ui, |ui| {
                                        ui.style_mut().spacing.item_spacing.y = 2.0;
                                        
                                        for (idx, m) in pattern.matches.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                if ui.button(format!("#{}", idx + 1)).clicked() {
                                                    self.viewer.clear_highlights();
                                                    self.viewer.add_highlight_range(m.position, pattern.bits.len());
                                                    self.viewer.jump_to_position(m.position);
                                                }
                                                
                                                ui.label(format!("@{}", m.position));
                                                
                                                if let Some(delta) = m.delta {
                                                    ui.label(format!("Δ{}", delta));
                                                }
                                                
                                                if m.mismatches > 0 {
                                                    ui.label(format!("~{}", m.mismatches));
                                                }
                                                
                                                ui.label(format!("{}", m.bits_string()));
                                            });
                                        }
                                    });
                            }
                        }
                    }
                        });
                });
        }

        // Operation creation/editing window
        if let Some(op_type) = self.show_operation_menu {
            let title = if self.editing_operation_index.is_some() {
                format!("Edit {}", op_type.name())
            } else {
                format!("Create {}", op_type.name())
            };
            
            let mut open = true;
            egui::Window::new(title)
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    match op_type {
                        OperationType::LoadFile => {
                            ui.heading("Load File");
                            ui.separator();
                            
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.loadfile_name);
                            });
                            ui.label("Give this operation a custom name (optional)");
                            
                            ui.add_space(8.0);
                            
                            ui.label("Select a file to load bits from:");
                            
                            if let Some(path) = &self.loadfile_path {
                                ui.label(format!("📄 Selected: {}", path.display()));
                            } else {
                                ui.label("No file selected");
                            }
                            
                            if ui.button("📂 Browse...").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    self.loadfile_path = Some(path);
                                }
                            }
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("✓ Save").clicked() {
                                    self.save_current_operation();
                                }
                                
                                if ui.button("✗ Cancel").clicked() {
                                    self.cancel_operation_edit();
                                }
                            });
                        }
                        OperationType::TakeSkipSequence => {
                            ui.heading("Take/Skip Sequence");
                            ui.separator();
                            
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.takeskip_name);
                            });
                            ui.label("Give this operation a custom name (optional)");
                            
                            ui.add_space(8.0);
                            
                            ui.label("Enter a sequence of operations:");
                            ui.label("• t = take N bits");
                            ui.label("• r = reverse N bits");
                            ui.label("• i = invert N bits");
                            ui.label("• s = skip N bits");
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("Sequence:");
                                let response = ui.text_edit_singleline(&mut self.takeskip_input);
                                
                                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    self.save_current_operation();
                                }
                            });
                            
                            ui.label("Example: t4r3i8s1");
                            ui.label("(take 4, reverse 3, invert 8, skip 1)");
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("✓ Save").clicked() {
                                    self.save_current_operation();
                                }
                                
                                if ui.button("✗ Cancel").clicked() {
                                    self.cancel_operation_edit();
                                }
                            });
                        }
                        OperationType::InvertBits => {
                            ui.heading("Invert All Bits");
                            ui.separator();
                            
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.invert_name);
                            });
                            ui.label("Give this operation a custom name (optional)");
                            
                            ui.add_space(8.0);
                            
                            ui.label("This operation will invert all bits:");
                            ui.label("• 0 → 1");
                            ui.label("• 1 → 0");
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("✓ Save").clicked() {
                                    self.save_current_operation();
                                }
                                
                                if ui.button("✗ Cancel").clicked() {
                                    self.cancel_operation_edit();
                                }
                            });
                        }
                        OperationType::MultiWorksheetLoad => {
                            ui.heading("Multi-Worksheet Load");
                            ui.separator();
                            
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut self.multiworksheet_name);
                            });
                            ui.label("Give this operation a custom name (optional)");
                            
                            ui.add_space(8.0);
                            
                            ui.label("Add worksheets to load from:");
                            ui.separator();
                            
                            // Add worksheet operation
                            ui.group(|ui| {
                                ui.label("Add Worksheet Operation:");
                                
                                ui.horizontal(|ui| {
                                    ui.label("Worksheet:");
                                    egui::ComboBox::from_id_salt("worksheet_selector")
                                        .selected_text(if self.multiworksheet_selected_worksheet < self.worksheets.len() {
                                            &self.worksheets[self.multiworksheet_selected_worksheet].name
                                        } else {
                                            "Select..."
                                        })
                                        .show_ui(ui, |ui| {
                                            for (idx, worksheet) in self.worksheets.iter().enumerate() {
                                                // Skip current worksheet
                                                if idx != self.current_worksheet_index {
                                                    ui.selectable_value(&mut self.multiworksheet_selected_worksheet, idx, &worksheet.name);
                                                }
                                            }
                                        });
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label("Sequence:");
                                    ui.text_edit_singleline(&mut self.multiworksheet_input);
                                });
                                ui.label("Example: t4r3i8s1");
                                
                                if ui.button("➕ Add").clicked() {
                                    if !self.multiworksheet_input.is_empty() {
                                        self.multiworksheet_ops.push((
                                            self.multiworksheet_selected_worksheet,
                                            self.multiworksheet_input.clone()
                                        ));
                                        self.multiworksheet_input.clear();
                                    }
                                }
                            });
                            
                            ui.add_space(8.0);
                            
                            // List of worksheet operations
                            ui.label("Worksheet Operations:");
                            if self.multiworksheet_ops.is_empty() {
                                ui.label("No worksheets added yet");
                            } else {
                                let mut to_remove = None;
                                egui::ScrollArea::vertical()
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        for (idx, (ws_idx, seq)) in self.multiworksheet_ops.iter().enumerate() {
                                            ui.horizontal(|ui| {
                                                let ws_name = if *ws_idx < self.worksheets.len() {
                                                    &self.worksheets[*ws_idx].name
                                                } else {
                                                    "Unknown"
                                                };
                                                ui.label(format!("{}. {} → {}", idx + 1, ws_name, seq));
                                                if ui.button("❌").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            });
                                        }
                                    });
                                
                                if let Some(idx) = to_remove {
                                    self.multiworksheet_ops.remove(idx);
                                }
                            }
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("✓ Save").clicked() {
                                    self.save_current_operation();
                                }
                                
                                if ui.button("✗ Cancel").clicked() {
                                    self.cancel_operation_edit();
                                }
                            });
                        }
                    }
                });
            
            if !open {
                self.cancel_operation_edit();
            }
        }

        // Byte View Column Editor Window
        if self.show_column_editor {
            let mut open = true;
            egui::Window::new("➕ Add Protocol Column")
                .open(&mut open)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Define Protocol Column");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Label:");
                        ui.text_edit_singleline(&mut self.column_editor_label);
                    });
                    ui.label("Name for this field (e.g., 'Header', 'Checksum')");
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Start bit:");
                        ui.text_edit_singleline(&mut self.column_editor_bit_start);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("End bit:");
                        ui.text_edit_singleline(&mut self.column_editor_bit_end);
                    });
                    ui.label("Bit range within each row (0-based, inclusive)");
                    
                    ui.add_space(8.0);
                    
                    ui.label("Color:");
                    ui.horizontal(|ui| {
                        ui.label("R:");
                        ui.add(egui::Slider::new(&mut self.column_editor_color[0], 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label("G:");
                        ui.add(egui::Slider::new(&mut self.column_editor_color[1], 0..=255));
                    });
                    ui.horizontal(|ui| {
                        ui.label("B:");
                        ui.add(egui::Slider::new(&mut self.column_editor_color[2], 0..=255));
                    });
                    
                    // Color preview
                    let color = egui::Color32::from_rgb(
                        self.column_editor_color[0],
                        self.column_editor_color[1],
                        self.column_editor_color[2]
                    );
                    ui.horizontal(|ui| {
                        ui.label("Preview:");
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(100.0, 30.0),
                            egui::Sense::hover()
                        );
                        ui.painter().rect_filled(rect, 3.0, color);
                    });
                    
                    ui.add_space(8.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("✓ Add Column").clicked() {
                            if let (Ok(start), Ok(end)) = (
                                self.column_editor_bit_start.parse::<usize>(),
                                self.column_editor_bit_end.parse::<usize>()
                            ) {
                                if start <= end {
                                    let label = if self.column_editor_label.is_empty() {
                                        format!("Column {}", self.byte_viewer.config.columns.len() + 1)
                                    } else {
                                        self.column_editor_label.clone()
                                    };
                                    
                                    self.byte_viewer.add_column(
                                        byte_viewer::ByteColumn::new(
                                            label,
                                            start,
                                            end,
                                            self.column_editor_color
                                        )
                                    );
                                    
                                    // Reset for next column
                                    self.column_editor_label.clear();
                                    self.column_editor_bit_start = format!("{}", end + 1);
                                    self.column_editor_bit_end = format!("{}", end + 8);
                                    self.show_column_editor = false;
                                } else {
                                    self.error_message = Some("Start bit must be <= end bit".to_string());
                                }
                            } else {
                                self.error_message = Some("Invalid bit range values".to_string());
                            }
                        }
                        
                        if ui.button("✗ Cancel").clicked() {
                            self.show_column_editor = false;
                        }
                    });
                });
            
            if !open {
                self.show_column_editor = false;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }

            // Show viewer if we have bits (either original or processed)
            if self.original_bits.is_empty() && self.processed_bits.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.heading("Open a file to view its bits");
                });
            } else {
                match self.view_mode {
                    ViewMode::Bit => {
                        self.viewer.show(ui);
                    }
                    ViewMode::Byte => {
                        let bits_to_display = if self.show_original {
                            &self.original_bits
                        } else {
                            &self.processed_bits
                        };
                        self.byte_viewer.render(ui, bits_to_display);
                    }
                    ViewMode::Ascii => {
                        let bits_to_display = if self.show_original {
                            &self.original_bits
                        } else {
                            &self.processed_bits
                        };
                        self.render_ascii_view(ui, bits_to_display);
                    }
                }
            }
        });
    }
}
