// Main application state and logic

use crate::analysis::{Pattern, PatternFormat};
use crate::core::{ViewMode, OperationType};
use crate::processing::{BitOperation, OperationSequence, WorksheetOperation};
use crate::storage::{read_file_as_bits, read_file_as_bits_with_progress, write_bits_to_file, AppSession, AppSettings, Worksheet, LoadProgress};
use crate::viewers::{BitViewer, ByteViewer};
use bitvec::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

/// Message from async operation processing
pub enum OperationProgress {
    LoadingFile { path: PathBuf, loaded: u64, total: u64 },
    ProcessingOperation { index: usize, total: usize, description: String },
    Complete(Result<BitVec<u8, Msb0>, String>),
}

/// Message for view rendering progress
#[allow(dead_code)]
pub enum RenderProgress {
    Preparing { total_items: usize },
    Rendering { rendered: usize, total: usize, description: String },
    Complete,
}

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
    
    // Truncate Bits editor state
    pub truncate_name: String,
    pub truncate_start: String,
    pub truncate_end: String,
    
    // Interleave Bits editor state
    pub interleave_name: String,
    pub interleave_type: crate::processing::InterleaverType,
    pub interleave_direction: crate::processing::InterleaverDirection,
    // Block interleaver params
    pub interleave_block_size: String,
    pub interleave_depth: String,
    // Convolutional interleaver params
    pub interleave_branches: String,
    pub interleave_delay_increment: String,
    // Symbol interleaver params
    pub interleave_symbol_size: String,
    
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
    
    // File loading state
    pub loading_receiver: Option<Receiver<LoadProgress>>,
    pub loading_file_path: Option<PathBuf>,
    pub loading_progress: f32,
    pub loading_total: u64,
    
    // Operation processing state
    pub operation_receiver: Option<Receiver<OperationProgress>>,
    pub operation_progress_message: String,
    pub operation_progress: f32,
    
    // Rendering state
    #[allow(dead_code)]
    pub is_rendering: bool,
    pub render_progress_message: String,
    #[allow(dead_code)]
    pub render_progress: f32,
    pub defer_first_render: bool, // Defer first render to show "preparing" message
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
            truncate_name: String::new(),
            truncate_start: String::from("0"),
            truncate_end: String::new(),
            interleave_name: String::new(),
            interleave_type: crate::processing::InterleaverType::Block,
            interleave_direction: crate::processing::InterleaverDirection::Interleave,
            interleave_block_size: String::from("8"),
            interleave_depth: String::from("4"),
            interleave_branches: String::from("4"),
            interleave_delay_increment: String::from("1"),
            interleave_symbol_size: String::from("8"),
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
            loading_receiver: None,
            loading_file_path: None,
            loading_progress: 0.0,
            loading_total: 0,
            operation_receiver: None,
            operation_progress_message: String::new(),
            operation_progress: 0.0,
            is_rendering: false,
            render_progress_message: String::new(),
            render_progress: 0.0,
            defer_first_render: false,
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
    
    /// Clear all pattern matches
    pub fn clear_pattern_matches(&mut self) {
        for pattern in &mut self.patterns {
            pattern.matches.clear();
        }
        // Also clear bit viewer highlights since they're based on pattern matches
        self.viewer.clear_highlights();
    }
    
    pub fn apply_operations(&mut self) {
        // Don't clear pattern matches here - they should only be cleared when operations list changes
        // Pattern matches are based on the processed bits, which may not change even if we reapply
        
        // Check if we need async processing (large files in operations)
        let needs_async = self.operations.iter().any(|op| {
            if let BitOperation::LoadFile { file_path, enabled, .. } = op {
                if *enabled {
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        return metadata.len() > 10 * 1024 * 1024; // >10MB
                    }
                }
            }
            false
        });
        
        if needs_async {
            // Use async processing for heavy operations
            self.start_async_operations();
            return;
        }
        
        // Otherwise use synchronous processing (fast)
        // Check if we have enabled MultiWorksheetLoad or LoadFile operations
        let has_multiworksheet = self.operations.iter().any(|op| {
            matches!(op, BitOperation::MultiWorksheetLoad { enabled: true, .. })
        });
        let has_loadfile = self.operations.iter().any(|op| {
            matches!(op, BitOperation::LoadFile { enabled: true, .. })
        });
        
        if has_multiworksheet || has_loadfile {
            // MultiWorksheetLoad or LoadFile creates new bits from scratch
            let mut result = BitVec::new();
            
            for op in &self.operations {
                // Skip disabled operations
                if !op.is_enabled() {
                    continue;
                }
                
                match op {
                    BitOperation::LoadFile { file_path, .. } => {
                        // Load bits from the file
                        match read_file_as_bits(file_path) {
                            Ok(bits) => {
                                result.extend(bits);
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to load file {}: {}", 
                                    file_path.display(), e));
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
    
    #[allow(dead_code)]
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    #[allow(dead_code)]
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }
    
    /// Start loading a file asynchronously with progress reporting
    pub fn start_loading_file(&mut self, path: PathBuf) {
        let (tx, rx) = channel();
        let path_clone = path.clone();
        
        // Spawn background thread to load file
        thread::spawn(move || {
            let _ = read_file_as_bits_with_progress(&path_clone, tx);
        });
        
        self.loading_receiver = Some(rx);
        self.loading_file_path = Some(path);
        self.loading_progress = 0.0;
        self.loading_total = 0;
    }
    
    /// Check for loading progress updates and handle completion
    pub fn update_loading_progress(&mut self) {
        let mut should_clear = false;
        let mut result_bits: Option<Result<BitVec<u8, Msb0>, String>> = None;
        let mut path_to_load: Option<PathBuf> = None;
        
        if let Some(receiver) = &self.loading_receiver {
            // Process all available messages
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    LoadProgress::Progress { loaded, total } => {
                        self.loading_total = total;
                        self.loading_progress = if total > 0 {
                            loaded as f32 / total as f32
                        } else {
                            0.0
                        };
                    }
                    LoadProgress::Complete(result) => {
                        // Loading finished
                        should_clear = true;
                        result_bits = Some(result);
                        path_to_load = self.loading_file_path.clone();
                    }
                }
            }
        }
        
        // Handle completion outside of the borrow
        if should_clear {
            self.loading_receiver = None;
            self.loading_file_path = None;
            
            if let Some(result) = result_bits {
                match result {
                    Ok(bits) => {
                        self.original_bits = bits.clone();
                        self.processed_bits = bits;
                        self.current_file_path = path_to_load;
                        self.error_message = None;
                        self.clear_pattern_matches(); // New file loaded, clear old patterns
                        self.apply_operations();
                        
                        // If we loaded a large file, defer the first render to show "preparing" message
                        if self.original_bits.len() > 10_000_000 {
                            self.defer_first_render = true;
                            self.render_progress_message = "Preparing view...".to_string();
                        } else {
                            self.update_viewer();
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to load file: {}", e));
                    }
                }
            }
        }
    }
    
    /// Check if currently loading a file
    pub fn is_loading(&self) -> bool {
        self.loading_receiver.is_some()
    }
    
    /// Check if currently processing operations
    pub fn is_processing_operations(&self) -> bool {
        self.operation_receiver.is_some()
    }
    
    /// Start processing operations asynchronously
    pub fn start_async_operations(&mut self) {
        let operations = self.operations.clone();
        let original_bits = self.original_bits.clone();
        let worksheets = self.worksheets.clone();
        let current_worksheet_index = self.current_worksheet_index;
        
        let (tx, rx) = channel();
        
        thread::spawn(move || {
            let _ = Self::process_operations_async(
                operations,
                original_bits,
                worksheets,
                current_worksheet_index,
                tx
            );
        });
        
        self.operation_receiver = Some(rx);
        self.operation_progress = 0.0;
        self.operation_progress_message = "Starting...".to_string();
    }
    
    /// Process operations in background thread with progress reporting
    fn process_operations_async(
        operations: Vec<BitOperation>,
        original_bits: BitVec<u8, Msb0>,
        worksheets: Vec<Worksheet>,
        current_worksheet_index: usize,
        tx: std::sync::mpsc::Sender<OperationProgress>,
    ) -> std::io::Result<()> {
        let result = (|| -> Result<BitVec<u8, Msb0>, String> {
            let has_multiworksheet = operations.iter().any(|op| matches!(op, BitOperation::MultiWorksheetLoad { .. }));
            let has_loadfile = operations.iter().any(|op| matches!(op, BitOperation::LoadFile { .. }));
            
            if has_multiworksheet || has_loadfile {
                let mut result = BitVec::new();
                let total_ops = operations.len();
                
                for (idx, op) in operations.iter().enumerate() {
                    // Skip disabled operations
                    if !op.is_enabled() {
                        continue;
                    }
                    
                    match op {
                        BitOperation::LoadFile { file_path, name, .. } => {
                            let _ = tx.send(OperationProgress::ProcessingOperation {
                                index: idx + 1,
                                total: total_ops,
                                description: format!("Loading file: {}", name),
                            });
                            
                            // Check file size for progress reporting
                            if let Ok(metadata) = std::fs::metadata(file_path) {
                                let file_size = metadata.len();
                                
                                if file_size > 10 * 1024 * 1024 {
                                    // Large file - use progress reporting
                                    let (file_tx, file_rx) = channel();
                                    let path_clone = file_path.clone();
                                    
                                    // Load file with progress
                                    thread::spawn(move || {
                                        let _ = read_file_as_bits_with_progress(&path_clone, file_tx);
                                    });
                                    
                                    // Forward progress messages
                                    loop {
                                        match file_rx.recv() {
                                            Ok(LoadProgress::Progress { loaded, total }) => {
                                                let _ = tx.send(OperationProgress::LoadingFile {
                                                    path: file_path.clone(),
                                                    loaded,
                                                    total,
                                                });
                                            }
                                            Ok(LoadProgress::Complete(Ok(bits))) => {
                                                result.extend(bits);
                                                break;
                                            }
                                            Ok(LoadProgress::Complete(Err(e))) => {
                                                return Err(format!("Failed to load file {}: {}", file_path.display(), e));
                                            }
                                            Err(_) => {
                                                return Err(format!("Failed to load file {}: channel closed", file_path.display()));
                                            }
                                        }
                                    }
                                } else {
                                    // Small file - load directly
                                    match read_file_as_bits(file_path) {
                                        Ok(bits) => result.extend(bits),
                                        Err(e) => return Err(format!("Failed to load file {}: {}", file_path.display(), e)),
                                    }
                                }
                            } else {
                                // Can't get metadata, try loading anyway
                                match read_file_as_bits(file_path) {
                                    Ok(bits) => result.extend(bits),
                                    Err(e) => return Err(format!("Failed to load file {}: {}", file_path.display(), e)),
                                }
                            }
                        }
                        BitOperation::MultiWorksheetLoad { worksheet_operations, .. } => {
                            let _ = tx.send(OperationProgress::ProcessingOperation {
                                index: idx + 1,
                                total: total_ops,
                                description: "Processing multi-worksheet load".to_string(),
                            });
                            
                            for wo in worksheet_operations {
                                if wo.worksheet_index < worksheets.len() && wo.worksheet_index != current_worksheet_index {
                                    let source_bits = if let Some(file_path) = &worksheets[wo.worksheet_index].file_path {
                                        match read_file_as_bits(file_path) {
                                            Ok(bits) => bits,
                                            Err(e) => return Err(format!("Failed to load worksheet {}: {}", wo.worksheet_index + 1, e)),
                                        }
                                    } else {
                                        continue;
                                    };
                                    
                                    let processed = wo.sequence.apply(&source_bits);
                                    result.extend(processed);
                                }
                            }
                        }
                        _ => {
                            let _ = tx.send(OperationProgress::ProcessingOperation {
                                index: idx + 1,
                                total: total_ops,
                                description: format!("Applying operation {}/{}", idx + 1, total_ops),
                            });
                            result = op.apply(&result);
                        }
                    }
                }
                
                Ok(result)
            } else {
                if original_bits.is_empty() {
                    return Ok(BitVec::new());
                }
                
                let mut result = original_bits;
                let total_ops = operations.len();
                
                for (idx, op) in operations.iter().enumerate() {
                    let _ = tx.send(OperationProgress::ProcessingOperation {
                        index: idx + 1,
                        total: total_ops,
                        description: format!("Applying operation {}/{}", idx + 1, total_ops),
                    });
                    result = op.apply(&result);
                }
                
                Ok(result)
            }
        })();
        
        let _ = tx.send(OperationProgress::Complete(result));
        Ok(())
    }
    
    /// Update operation processing progress
    pub fn update_operation_progress(&mut self) {
        let mut should_clear = false;
        let mut result_bits: Option<Result<BitVec<u8, Msb0>, String>> = None;
        
        if let Some(receiver) = &self.operation_receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    OperationProgress::LoadingFile { path, loaded, total } => {
                        let progress = if total > 0 { loaded as f32 / total as f32 } else { 0.0 };
                        self.operation_progress = progress;
                        self.operation_progress_message = format!(
                            "Loading {}: {:.1} MB / {:.1} MB",
                            path.file_name().unwrap_or_default().to_string_lossy(),
                            loaded as f64 / (1024.0 * 1024.0),
                            total as f64 / (1024.0 * 1024.0)
                        );
                    }
                    OperationProgress::ProcessingOperation { index, total, description } => {
                        self.operation_progress = index as f32 / total as f32;
                        self.operation_progress_message = description;
                    }
                    OperationProgress::Complete(result) => {
                        should_clear = true;
                        result_bits = Some(result);
                    }
                }
            }
        }
        
        if should_clear {
            self.operation_receiver = None;
            
            if let Some(result) = result_bits {
                match result {
                    Ok(bits) => {
                        self.processed_bits = bits;
                        self.show_original = false;
                        self.error_message = None;
                        
                        // If we processed a large amount of data, defer the first render to show "preparing" message
                        if self.processed_bits.len() > 10_000_000 {
                            self.defer_first_render = true;
                            self.render_progress_message = "Preparing view...".to_string();
                        } else {
                            self.update_viewer();
                        }
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
            }
        }
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
                // Check file size to decide if we should use async loading
                if let Ok(metadata) = std::fs::metadata(path) {
                    // Use async loading for files larger than 10MB
                    if metadata.len() > 10 * 1024 * 1024 {
                        self.start_loading_file(path.clone());
                        // Operations will be applied when loading completes
                        self.operations = worksheet.operations.clone();
                        return;
                    }
                }
                
                // For smaller files, load synchronously
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
        self.clear_pattern_matches(); // New worksheet loaded, clear patterns
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
        self.truncate_name.clear();
        self.truncate_start = String::from("0");
        self.truncate_end.clear();
        self.interleave_name.clear();
        self.interleave_type = crate::processing::InterleaverType::Block;
        self.interleave_direction = crate::processing::InterleaverDirection::Interleave;
        self.interleave_block_size = String::from("8");
        self.interleave_depth = String::from("4");
        self.interleave_branches = String::from("4");
        self.interleave_delay_increment = String::from("1");
        self.interleave_symbol_size = String::from("8");
        self.multiworksheet_name.clear();
        self.multiworksheet_ops.clear();
        self.multiworksheet_input.clear();
    }

    pub fn open_operation_editor(&mut self, index: usize) {
        if let Some(op) = self.operations.get(index) {
            match op {
                BitOperation::LoadFile { name, file_path, .. } => {
                    self.show_operation_menu = Some(OperationType::LoadFile);
                    self.editing_operation_index = Some(index);
                    self.loadfile_name = name.clone();
                    self.loadfile_path = Some(file_path.clone());
                }
                BitOperation::TakeSkipSequence { name, sequence, .. } => {
                    self.show_operation_menu = Some(OperationType::TakeSkipSequence);
                    self.editing_operation_index = Some(index);
                    self.takeskip_name = name.clone();
                    self.takeskip_input = sequence.to_string();
                }
                BitOperation::InvertBits { name, .. } => {
                    self.show_operation_menu = Some(OperationType::InvertBits);
                    self.editing_operation_index = Some(index);
                    self.invert_name = name.clone();
                }
                BitOperation::TruncateBits { name, start, end, .. } => {
                    self.show_operation_menu = Some(OperationType::TruncateBits);
                    self.editing_operation_index = Some(index);
                    self.truncate_name = name.clone();
                    self.truncate_start = start.to_string();
                    self.truncate_end = end.to_string();
                }
                BitOperation::InterleaveBits { name, interleaver_type, block_config, convolutional_config, symbol_config, .. } => {
                    self.show_operation_menu = Some(OperationType::InterleaveBits);
                    self.editing_operation_index = Some(index);
                    self.interleave_name = name.clone();
                    self.interleave_type = *interleaver_type;
                    
                    match interleaver_type {
                        crate::processing::InterleaverType::Block => {
                            if let Some(cfg) = block_config {
                                self.interleave_direction = cfg.direction;
                                self.interleave_block_size = cfg.block_size.to_string();
                                self.interleave_depth = cfg.depth.to_string();
                            }
                        }
                        crate::processing::InterleaverType::Convolutional => {
                            if let Some(cfg) = convolutional_config {
                                self.interleave_direction = cfg.direction;
                                self.interleave_branches = cfg.branches.to_string();
                                self.interleave_delay_increment = cfg.delay_increment.to_string();
                            }
                        }
                        crate::processing::InterleaverType::Symbol => {
                            if let Some(cfg) = symbol_config {
                                self.interleave_direction = cfg.direction;
                                self.interleave_symbol_size = cfg.symbol_size.to_string();
                                self.interleave_block_size = cfg.block_size.to_string();
                                self.interleave_depth = cfg.depth.to_string();
                            }
                        }
                    }
                }
                BitOperation::MultiWorksheetLoad { name, worksheet_operations, .. } => {
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
                        enabled: true,
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
                                enabled: true,
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
                    
                    BitOperation::InvertBits { name, enabled: true }
                }
                OperationType::TruncateBits => {
                    // Parse start and end
                    let start = self.truncate_start.trim().parse::<usize>().unwrap_or(0);
                    let end = if self.truncate_end.trim().is_empty() {
                        // If no end specified, use a very large number (essentially to the end)
                        usize::MAX
                    } else {
                        match self.truncate_end.trim().parse::<usize>() {
                            Ok(val) => val,
                            Err(_) => {
                                self.error_message = Some("Invalid end value".to_string());
                                return;
                            }
                        }
                    };
                    
                    if start >= end {
                        self.error_message = Some("Start must be less than end".to_string());
                        return;
                    }
                    
                    let name = if self.truncate_name.trim().is_empty() {
                        format!("Truncate: {}-{}", start, if end == usize::MAX { "end".to_string() } else { end.to_string() })
                    } else {
                        self.truncate_name.clone()
                    };
                    
                    BitOperation::TruncateBits { name, start, end, enabled: true }
                }
                OperationType::InterleaveBits => {
                    use crate::processing::{BlockInterleaverConfig, ConvolutionalInterleaverConfig, InterleaverType};
                    use crate::processing::interleaver::SymbolInterleaverConfig;
                    
                    let name = if self.interleave_name.trim().is_empty() {
                        match self.interleave_type {
                            InterleaverType::Block => "Block Interleaver".to_string(),
                            InterleaverType::Convolutional => "Convolutional Interleaver".to_string(),
                            InterleaverType::Symbol => "Symbol Interleaver".to_string(),
                        }
                    } else {
                        self.interleave_name.clone()
                    };
                    
                    let (block_config, convolutional_config, symbol_config) = match self.interleave_type {
                        InterleaverType::Block => {
                            let block_size = match self.interleave_block_size.trim().parse::<usize>() {
                                Ok(val) if val > 0 => val,
                                _ => {
                                    self.error_message = Some("Block size must be a positive number".to_string());
                                    return;
                                }
                            };
                            
                            let depth = match self.interleave_depth.trim().parse::<usize>() {
                                Ok(val) if val > 0 => val,
                                _ => {
                                    self.error_message = Some("Depth must be a positive number".to_string());
                                    return;
                                }
                            };
                            
                            (Some(BlockInterleaverConfig::new(block_size, depth, self.interleave_direction)), None, None)
                        }
                        InterleaverType::Convolutional => {
                            let branches = match self.interleave_branches.trim().parse::<usize>() {
                                Ok(val) if val > 0 => val,
                                _ => {
                                    self.error_message = Some("Branches must be a positive number".to_string());
                                    return;
                                }
                            };
                            
                            let delay_increment = match self.interleave_delay_increment.trim().parse::<usize>() {
                                Ok(val) => val,
                                _ => {
                                    self.error_message = Some("Delay increment must be a valid number".to_string());
                                    return;
                                }
                            };
                            
                            (None, Some(ConvolutionalInterleaverConfig::new(branches, delay_increment, self.interleave_direction)), None)
                        }
                        InterleaverType::Symbol => {
                            let symbol_size = match self.interleave_symbol_size.trim().parse::<usize>() {
                                Ok(val) if val > 0 => val,
                                _ => {
                                    self.error_message = Some("Symbol size must be a positive number".to_string());
                                    return;
                                }
                            };
                            
                            let block_size = match self.interleave_block_size.trim().parse::<usize>() {
                                Ok(val) if val > 0 => val,
                                _ => {
                                    self.error_message = Some("Block size must be a positive number".to_string());
                                    return;
                                }
                            };
                            
                            let depth = match self.interleave_depth.trim().parse::<usize>() {
                                Ok(val) if val > 0 => val,
                                _ => {
                                    self.error_message = Some("Depth must be a positive number".to_string());
                                    return;
                                }
                            };
                            
                            (None, None, Some(SymbolInterleaverConfig::new(symbol_size, block_size, depth, self.interleave_direction)))
                        }
                    };
                    
                    BitOperation::InterleaveBits {
                        name,
                        interleaver_type: self.interleave_type,
                        block_config,
                        convolutional_config,
                        symbol_config,
                        enabled: true,
                    }
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
                        enabled: true,
                    }
                }
            };

            if let Some(index) = self.editing_operation_index {
                // Editing existing operation - data will change
                self.operations[index] = new_operation;
                self.clear_pattern_matches();
            } else {
                // Adding new operation - data will change
                self.operations.push(new_operation);
                self.clear_pattern_matches();
            }

            self.show_operation_menu = None;
            self.editing_operation_index = None;
            self.takeskip_name.clear();
            self.takeskip_input.clear();
            self.loadfile_name.clear();
            self.loadfile_path = None;
            self.invert_name.clear();
            self.truncate_name.clear();
            self.truncate_start = String::from("0");
            self.truncate_end.clear();
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
        self.truncate_name.clear();
        self.truncate_start = String::from("0");
        self.truncate_end.clear();
        self.interleave_name.clear();
        self.interleave_type = crate::processing::InterleaverType::Block;
        self.interleave_direction = crate::processing::InterleaverDirection::Interleave;
        self.interleave_block_size = String::from("8");
        self.interleave_depth = String::from("4");
        self.interleave_branches = String::from("4");
        self.interleave_delay_increment = String::from("1");
        self.interleave_symbol_size = String::from("8");
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

        // Predefined colors for different patterns (same as byte viewer)
        let pattern_colors = [
            egui::Color32::from_rgb(255, 100, 100),  // Red
            egui::Color32::from_rgb(100, 255, 100),  // Green
            egui::Color32::from_rgb(100, 100, 255),  // Blue
            egui::Color32::from_rgb(255, 255, 100),  // Yellow
            egui::Color32::from_rgb(255, 100, 255),  // Magenta
            egui::Color32::from_rgb(100, 255, 255),  // Cyan
            egui::Color32::from_rgb(255, 150, 100),  // Orange
            egui::Color32::from_rgb(150, 100, 255),  // Purple
        ];

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
                            // Set a light background for better text visibility
                            ui.painter().rect_filled(
                                ui.max_rect(),
                                0.0,
                                egui::Color32::from_rgb(245, 245, 245)
                            );
                            
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
                                        
                                        // Check if this byte is part of any pattern match
                                        let mut pattern_match: Option<(egui::Color32, String)> = None;
                                        for (pattern_idx, pattern) in self.patterns.iter().enumerate() {
                                            for match_info in &pattern.matches {
                                                let match_start = match_info.position;
                                                let match_end = match_info.position + pattern.bits.len();
                                                
                                                // Check if this byte overlaps with the pattern match
                                                if bit_start < match_end && bit_end > match_start {
                                                    let color = pattern_colors[pattern_idx % pattern_colors.len()];
                                                    pattern_match = Some((color, pattern.name.clone()));
                                                    break;
                                                }
                                            }
                                            if pattern_match.is_some() {
                                                break;
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
                                        
                                        // Draw background highlight for pattern matches
                                        if let Some((pattern_color, _)) = pattern_match {
                                            ui.painter().rect_filled(
                                                rect,
                                                2.0,
                                                egui::Color32::from_rgba_unmultiplied(
                                                    pattern_color.r(),
                                                    pattern_color.g(),
                                                    pattern_color.b(),
                                                    120
                                                )
                                            );
                                        }
                                        
                                        // Choose color based on character type and pattern match
                                        let text_color = if pattern_match.is_some() {
                                            egui::Color32::BLACK
                                        } else if byte >= 32 && byte <= 126 {
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
                                                
                                                if let Some((_, pattern_name)) = pattern_match {
                                                    ui.separator();
                                                    ui.label(format!("🎯 Pattern: {}", pattern_name));
                                                }
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
