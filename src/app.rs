// Main application state and logic

use crate::analysis::{FrameWidthAnalysis, PatternLocatorState};
use crate::app_state::{LoadingState, OperationProcessingState};
use crate::core::{BitSelection, ViewMode};
use crate::operations::{BitOperation, OperationEditorState, OperationType};
use crate::storage::{read_file_as_bits, read_file_as_bits_with_progress, write_bits_to_file, AppSession, AppSettings, Worksheet, LoadProgress};
use crate::viewers::{BitViewer, ByteViewer};
use bitvec::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

/// Message from async operation processing
pub enum OperationProgress {
    LoadingFile { path: PathBuf, loaded: u64, total: u64 },
    ProcessingOperation { index: usize, total: usize, description: String },
    Complete(Result<BitVec<u8, Msb0>, String>),
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
    pub last_applied_font_size: f32, // Cache to avoid style cloning every frame
    pub settings: AppSettings,

    // Worksheet management
    pub worksheets: Vec<Worksheet>,
    pub current_worksheet_index: usize,
    pub renaming_worksheet: Option<usize>,
    pub worksheet_name_buffer: String,
    
    // Operation creation/editing state
    pub show_operation_menu: Option<OperationType>,
    pub editing_operation_index: Option<usize>,
    pub editor_state: Option<OperationEditorState>,

    // Drag and drop state
    pub dragging_operation: Option<usize>,
    
    // Pattern Locator state
    pub pattern_locator: PatternLocatorState,

    // Session restore state
    pub show_restore_dialog: bool,
    pub pending_session: Option<AppSession>,
    
    // Byte view column editor state
    pub show_column_editor: bool,
    pub column_editor_label: String,
    pub column_editor_bit_start: String,
    pub column_editor_bit_end: String,
    pub column_editor_color: [u8; 3],

    // File loading state (managed by state module)
    pub loading_state: LoadingState,

    // Operation processing state (managed by state module)
    pub operation_processing_state: OperationProcessingState,

    // Rendering state
    pub render_progress_message: String,
    pub defer_first_render: bool, // Defer first render to show "preparing" message
    
    // Frame Width Finder state
    pub show_frame_width_finder: bool,
    pub frame_width_min: usize,
    pub frame_width_max: usize,
    pub frame_width_delta: usize,
    pub frame_width_analysis: Option<FrameWidthAnalysis>,
    pub frame_width_sort_by_score: bool, // true = sort by score, false = sort by width
    pub frame_width_selected: Option<usize>, // Last clicked width

    // Confirmation dialog state
    pub show_confirmation_dialog: bool,
    pub confirmation_message: String,
    pub pending_confirmation: Option<PendingConfirmation>,

    // Toast notifications for transient errors/messages
    pub toasts: Vec<Toast>,

    // Notification dialog (when using Dialog mode instead of Toast)
    pub show_notification_dialog: bool,
    pub notification_dialog_message: String,
    pub notification_dialog_type: Option<ToastType>,

    // Unsaved changes tracking
    pub has_unsaved_changes: bool,
    pub close_requested: bool,

    // Template management state
    pub show_save_template_dialog: bool,
    pub template_name_buffer: String,

    // Operations search/filter
    pub operations_search_text: String,

    // Bit offset (shared across all views)
    pub view_bit_offset: usize,
    pub last_ascii_bit_offset: Option<usize>,
    pub show_goto_offset_dialog: bool,
    pub goto_offset_input: String,
    pub goto_offset_format: GotoOffsetFormat,

    // 2D selection state
    pub selection: BitSelection,
    pub show_context_menu: bool,
    pub context_menu_pos: Option<egui::Pos2>,
}

/// Represents actions that require user confirmation
#[derive(Debug, Clone)]
pub enum PendingConfirmation {
    DeleteWorksheet(usize),
    DeleteOperation(usize),
    ClearAllOperations,
    OverwriteFile(PathBuf),
    CloseApp,
}

/// Represents a temporary toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: std::time::Instant,
    pub duration_secs: f32,
}

/// Types of toast notifications
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

/// Format options for the Go-to-Offset dialog
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GotoOffsetFormat {
    Bit,
    Byte,
    Hex,
}

impl Toast {
    pub fn new(message: String, toast_type: ToastType, duration_secs: f32) -> Self {
        Self {
            message,
            toast_type,
            created_at: std::time::Instant::now(),
            duration_secs,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs_f32() > self.duration_secs
    }

    pub fn opacity(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let fade_duration = 0.5;

        if elapsed < self.duration_secs - fade_duration {
            1.0
        } else {
            let fade_progress = (self.duration_secs - elapsed) / fade_duration;
            fade_progress.clamp(0.0, 1.0)
        }
    }
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
            last_applied_font_size: 0.0, // Initialize to 0 to force update on first frame
            settings,
            worksheets,
            current_worksheet_index: 0,
            renaming_worksheet: None,
            worksheet_name_buffer: String::new(),
            show_operation_menu: None,
            editing_operation_index: None,
            editor_state: None,
            dragging_operation: None,
            pattern_locator: PatternLocatorState::default(),
            show_restore_dialog,
            pending_session,
            show_column_editor: false,
            column_editor_label: String::new(),
            column_editor_bit_start: String::from("0"),
            column_editor_bit_end: String::from("7"),
            column_editor_color: [100, 150, 200],
            loading_state: LoadingState::new(),
            operation_processing_state: OperationProcessingState::new(),
            render_progress_message: String::new(),
            defer_first_render: false,
            show_frame_width_finder: false,
            frame_width_min: 4,
            frame_width_max: 200,
            frame_width_delta: 0,
            frame_width_analysis: None,
            frame_width_sort_by_score: true, // Default to sorting by score
            frame_width_selected: None,
            show_confirmation_dialog: false,
            confirmation_message: String::new(),
            pending_confirmation: None,
            toasts: Vec::new(),
            show_notification_dialog: false,
            notification_dialog_message: String::new(),
            notification_dialog_type: None,
            has_unsaved_changes: false,
            close_requested: false,
            show_save_template_dialog: false,
            template_name_buffer: String::new(),
            operations_search_text: String::new(),
            view_bit_offset: 0,
            last_ascii_bit_offset: None,
            show_goto_offset_dialog: false,
            goto_offset_input: String::new(),
            goto_offset_format: GotoOffsetFormat::Bit,
            selection: BitSelection::new(),
            show_context_menu: false,
            context_menu_pos: None,
        }
    }
}

impl BitApp {
    /// Get reference to the current worksheet.
    /// Safely handles invalid indices by clamping to valid range.
    pub fn current_worksheet(&self) -> &Worksheet {
        let index = self.current_worksheet_index.min(self.worksheets.len().saturating_sub(1));
        &self.worksheets[index]
    }

    /// Get mutable reference to the current worksheet.
    /// Safely handles invalid indices by clamping to valid range.
    pub fn current_worksheet_mut(&mut self) -> &mut Worksheet {
        let index = self.current_worksheet_index.min(self.worksheets.len().saturating_sub(1));
        &mut self.worksheets[index]
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
        for pattern in &mut self.pattern_locator.patterns {
            pattern.normal_matches.clear();
            pattern.inverted_matches.clear();
        }
        self.pattern_locator.merged_matches.clear();
        self.pattern_locator.has_searched = false;
        self.pattern_locator.selected_match_index = None;
        self.pattern_locator.match_page = 0;
        // Also clear bit viewer highlights since they're based on pattern matches
        self.viewer.clear_highlights();
    }
    
    pub fn apply_operations(&mut self) {
        // Don't clear pattern matches here - they should only be cleared when operations list changes
        // Pattern matches are based on the processed bits, which may not change even if we reapply
        
        // Check if we need async processing (large files in operations)
        let needs_async = self.operations.iter().any(|op| {
            if let BitOperation::LoadFile { inner } = op {
                if inner.enabled {
                    if let Ok(metadata) = std::fs::metadata(&inner.file_path) {
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
        // Check if we have enabled source operations (LoadFile, MultiWorksheetLoad)
        let has_source_op = self.operations.iter().any(|op| {
            op.is_enabled() && op.is_source_operation()
        });
        
        if has_source_op {
            // Source operations (LoadFile/MultiWorksheetLoad) create new bits from scratch
            let mut result = BitVec::new();

            for op in &self.operations {
                // Skip disabled operations
                if !op.is_enabled() {
                    continue;
                }

                match op {
                    BitOperation::LoadFile { inner } => {
                        // Load bits from the file
                        match read_file_as_bits(&inner.file_path) {
                            Ok(bits) => {
                                result.extend(bits);
                            }
                            Err(e) => {
                                self.error_message = Some(format!("Failed to load file {}: {}",
                                    inner.file_path.display(), e));
                                continue; // Skip if file can't be loaded
                            }
                        }
                    }
                    BitOperation::MultiWorksheetLoad { inner } => {
                        // Process each worksheet operation
                        for wo in &inner.worksheet_operations {
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
                        // Regular operations are applied to result so far (move semantics)
                        result = op.apply(result);
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

            // Clone original bits once, then move through operations (no additional clones)
            let mut result = self.original_bits.clone();

            for op in &self.operations {
                // Skip disabled operations
                if !op.is_enabled() {
                    continue;
                }
                // Move result through each operation - no clones!
                result = op.apply(result);
            }

            self.processed_bits = result;
        }
        
        self.update_viewer();
        self.sync_to_worksheet();
    }

    /// Start loading a file asynchronously with progress reporting
    pub fn start_loading_file(&mut self, path: PathBuf) {
        let (tx, rx) = channel();
        let path_clone = path.clone();

        // Spawn background thread to load file
        thread::spawn(move || {
            let _ = read_file_as_bits_with_progress(&path_clone, tx);
        });

        self.loading_state.start(rx, path);
    }
    
    /// Check for loading progress updates and handle completion
    pub fn update_loading_progress(&mut self) {
        // Poll the loading state for completion
        if let Some(complete) = self.loading_state.poll() {
            match complete.result {
                Ok(bits) => {
                    self.original_bits = bits.clone();
                    self.processed_bits = bits;
                    self.current_file_path = Some(complete.path.clone());
                    self.error_message = None;
                    self.clear_pattern_matches(); // New file loaded, clear old patterns

                    // Show success notification
                    let bit_count = self.original_bits.len();
                    let byte_count = (bit_count + 7) / 8;
                    let file_name = complete.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");
                    self.show_success(format!(
                        "Loaded \"{}\" - {} bits ({} bytes)",
                        file_name, bit_count, byte_count
                    ));

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
    
    /// Check if currently loading a file
    pub fn is_loading(&self) -> bool {
        self.loading_state.is_loading()
    }
    
    /// Check if currently processing operations
    pub fn is_processing_operations(&self) -> bool {
        self.operation_processing_state.is_processing()
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

        self.operation_processing_state.start(rx);
    }

    /// Load a file from the given path by creating a LoadFile operation
    pub fn load_file_from_path(&mut self, path: PathBuf) {
        // Add to recent files
        self.settings.recent_files.add(path.clone());
        self.settings.auto_save();

        // Create LoadFile operation
        let name = format!("Load: {}", path.file_name().unwrap_or_default().to_string_lossy());
        let operation = BitOperation::LoadFile {
            inner: crate::operations::load_file::LoadFileOperation {
                name,
                file_path: path,
                enabled: true,
            },
        };

        // Add operation and apply
        self.operations.push(operation);
        self.mark_unsaved();
        self.apply_operations();
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
            let has_source_op = operations.iter().any(|op| op.is_source_operation());

            if has_source_op {
                let mut result = BitVec::new();
                let total_ops = operations.len();
                
                for (idx, op) in operations.iter().enumerate() {
                    // Skip disabled operations
                    if !op.is_enabled() {
                        continue;
                    }
                    
                    match op {
                        BitOperation::LoadFile { inner } => {
                            let file_path = &inner.file_path;
                            let _ = tx.send(OperationProgress::ProcessingOperation {
                                index: idx + 1,
                                total: total_ops,
                                description: format!("Loading file: {}", inner.name),
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
                        BitOperation::MultiWorksheetLoad { inner } => {
                            let _ = tx.send(OperationProgress::ProcessingOperation {
                                index: idx + 1,
                                total: total_ops,
                                description: "Processing multi-worksheet load".to_string(),
                            });

                            for wo in &inner.worksheet_operations {
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
                            result = op.apply(result);
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
                    if !op.is_enabled() {
                        continue;
                    }
                    let _ = tx.send(OperationProgress::ProcessingOperation {
                        index: idx + 1,
                        total: total_ops,
                        description: format!("Applying operation {}/{}", idx + 1, total_ops),
                    });
                    result = op.apply(result);
                }
                
                Ok(result)
            }
        })();
        
        let _ = tx.send(OperationProgress::Complete(result));
        Ok(())
    }
    
    /// Update operation processing progress
    pub fn update_operation_progress(&mut self) {
        // Poll the operation processing state for completion
        if let Some(complete) = self.operation_processing_state.poll() {
            match complete.result {
                Ok(bits) => {
                    self.processed_bits = bits;
                    self.show_original = false;
                    self.error_message = None;

                    // Show success notification
                    let bit_count = self.processed_bits.len();
                    let byte_count = (bit_count + 7) / 8;
                    self.show_success(format!(
                        "Operations completed successfully! Processed {} bits ({} bytes)",
                        bit_count, byte_count
                    ));

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
            match self.current_worksheet().save_to_file(&path) {
                Ok(_) => {
                    // Mark worksheet as saved after successful save
                    if self.current_worksheet_index < self.worksheets.len() {
                        self.worksheets[self.current_worksheet_index].mark_saved();
                    }
                    self.show_success(format!("Worksheet saved to: {}", path.display()));
                }
                Err(e) => {
                    self.error_message = Some(e);
                }
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
            // Check if file already exists
            if path.exists() {
                self.request_confirmation(
                    PendingConfirmation::OverwriteFile(path.clone()),
                    format!("File \"{}\" already exists.\n\nDo you want to overwrite it?",
                        path.file_name().unwrap_or_default().to_string_lossy())
                );
            } else {
                // File doesn't exist, save directly
                let bits_to_save = if self.show_original {
                    &self.original_bits
                } else {
                    &self.processed_bits
                };

                match write_bits_to_file(&path, bits_to_save) {
                    Ok(_) => {
                        self.error_message = None;
                        self.show_success(format!("File saved: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                        self.mark_saved();
                    }
                    Err(e) => {
                        self.show_error(format!("Failed to save file: {}", e), true);
                    }
                }
            }
        }
    }

    pub fn export_as_hex(&mut self) {
        let dialog = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .set_file_name("export.txt");

        if let Some(path) = dialog.save_file() {
            let bits_to_export = if self.show_original {
                &self.original_bits
            } else {
                &self.processed_bits
            };

            let bytes = bits_to_export.clone().into_vec();
            let hex_dump = bytes.chunks(16)
                .enumerate()
                .map(|(i, chunk)| {
                    let hex: String = chunk.iter()
                        .map(|b| format!("{:02x} ", b))
                        .collect();
                    let ascii: String = chunk.iter()
                        .map(|&b| if (32..127).contains(&b) { b as char } else { '.' })
                        .collect();
                    format!("{:08x}  {:<48} {}", i * 16, hex, ascii)
                })
                .collect::<Vec<_>>()
                .join("\n");

            match std::fs::write(&path, hex_dump) {
                Ok(_) => self.show_success(format!("Hex dump exported to: {}", path.file_name().unwrap_or_default().to_string_lossy())),
                Err(e) => self.show_error(format!("Failed to export hex dump: {}", e), true),
            }
        }
    }

    pub fn export_as_base64(&mut self) {
        let dialog = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .set_file_name("export.txt");

        if let Some(path) = dialog.save_file() {
            let bits_to_export = if self.show_original {
                &self.original_bits
            } else {
                &self.processed_bits
            };

            let bytes = bits_to_export.clone().into_vec();
            use base64::Engine;
            let base64_str = base64::engine::general_purpose::STANDARD.encode(&bytes);

            match std::fs::write(&path, base64_str) {
                Ok(_) => self.show_success(format!("Base64 exported to: {}", path.file_name().unwrap_or_default().to_string_lossy())),
                Err(e) => self.show_error(format!("Failed to export Base64: {}", e), true),
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
        self.editor_state = Some(OperationEditorState::new_for_type(op_type));
    }

    pub fn open_operation_editor(&mut self, index: usize) {
        if let Some(op) = self.operations.get(index) {
            self.show_operation_menu = Some(op.operation_type());
            self.editing_operation_index = Some(index);
            self.editor_state = Some(OperationEditorState::from_operation(op));
        }
    }

    pub fn save_current_operation(&mut self) {
        let Some(ref editor_state) = self.editor_state else {
            return;
        };

        // Try to build the operation from editor state
        let new_operation = match editor_state.try_build_operation() {
            Ok(op) => op,
            Err(e) => {
                self.error_message = Some(e);
                return;
            }
        };

        // For LoadFile operations, add to recent files
        if let BitOperation::LoadFile { ref inner } = new_operation {
            let file_path = &inner.file_path;
            self.settings.recent_files.add(file_path.clone());
            self.settings.auto_save();
        }

        if let Some(index) = self.editing_operation_index {
            // Editing existing operation
            self.operations[index] = new_operation;
        } else {
            // Adding new operation
            self.operations.push(new_operation);
        }

        self.clear_pattern_matches();
        self.mark_unsaved();
        self.show_operation_menu = None;
        self.editing_operation_index = None;
        self.editor_state = None;
        self.error_message = None;
        self.apply_operations();
    }

    pub fn cancel_operation_edit(&mut self) {
        self.show_operation_menu = None;
        self.editing_operation_index = None;
        self.editor_state = None;
    }
    
    pub fn render_ascii_view(
        ui: &mut eframe::egui::Ui,
        bits: &BitVec<u8, Msb0>,
        bit_offset: usize,
        frame_length: usize,
        byte_size: f32,
        show_hex_offset: bool,
        patterns: &[crate::analysis::Pattern],
        last_bit_offset: &mut Option<usize>,
    ) -> usize {
        use eframe::egui;

        if bits.is_empty() {
            ui.label("No data to display");
            return bit_offset;
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

        // Calculate total size for ALL data (aligned to sub_row)
        let total_bits = bits.len();
        let chars_per_row = frame_length / 8;
        let bits_per_row = chars_per_row * 8;
        let sub_row = bit_offset % bits_per_row;
        let effective_bits = total_bits.saturating_sub(sub_row);
        let total_bytes = (effective_bits + 7) / 8;
        let total_rows = (total_bytes + chars_per_row - 1) / chars_per_row;
        let offset_row = bit_offset / bits_per_row;

        let char_width = byte_size * 0.6;
        let char_height = byte_size * 1.25;
        let offset_width = if show_hex_offset { 90.0 } else { 0.0 };

        let mut scroll_area = egui::ScrollArea::vertical()
            .id_salt("ascii_viewer_scroll")
            .auto_shrink([false, false]);

        // Only force scroll when bit_offset changed externally
        if *last_bit_offset != Some(bit_offset) {
            let expected_scroll_y = offset_row as f32 * char_height;
            scroll_area = scroll_area.vertical_scroll_offset(expected_scroll_y);
        }
        let output = scroll_area.show_rows(
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
                                    if show_hex_offset {
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
                                        let bit_start = sub_row + byte_idx * 8;
                                        let bit_end = (bit_start + 8).min(total_bits);
                                        if bit_start >= total_bits {
                                            break;
                                        }
                                        let byte_bits = &bits[bit_start..bit_end];

                                        let mut byte = 0u8;
                                        for (i, bit) in byte_bits.iter().enumerate() {
                                            if *bit {
                                                byte |= 1 << (7 - i);
                                            }
                                        }

                                        // Check if this byte is part of any pattern match
                                        let mut pattern_match: Option<(egui::Color32, String)> = None;
                                        for (pattern_idx, pattern) in patterns.iter().enumerate() {
                                            if !pattern.highlight || !pattern.enabled {
                                                continue;
                                            }
                                            // Check normal matches
                                            for match_info in &pattern.normal_matches {
                                                let match_start = match_info.position;
                                                let match_end = match_info.position + pattern.bits.len();

                                                if bit_start < match_end && bit_end > match_start {
                                                    let color = pattern_colors[pattern_idx % pattern_colors.len()];
                                                    pattern_match = Some((color, pattern.name.clone()));
                                                    break;
                                                }
                                            }
                                            if pattern_match.is_some() {
                                                break;
                                            }
                                            // Check inverted matches
                                            for match_info in &pattern.inverted_matches {
                                                let match_start = match_info.position;
                                                let match_end = match_info.position + pattern.bits.len();

                                                if bit_start < match_end && bit_end > match_start {
                                                    let color = pattern_colors[pattern_idx % pattern_colors.len()];
                                                    pattern_match = Some((color, format!("{} (inv)", pattern.name)));
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
                                            '~'
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
                                            egui::FontId::monospace(byte_size),
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
                                                    ui.label(format!("Pattern: {}", pattern_name));
                                                }
                                            });
                                        }
                                    }
                                });
                            }
                        });
                },
            );

        // Convert scroll Y back to row and derive effective bit offset
        let actual_scroll_y = output.state.offset.y;
        let actual_row = (actual_scroll_y / char_height).round() as usize;
        let effective_bit_offset = actual_row * bits_per_row + sub_row;
        *last_bit_offset = Some(effective_bit_offset);
        effective_bit_offset
    }

    /// Run frame width analysis on the current bits
    pub fn run_frame_width_analysis(&mut self) {
        use crate::analysis::find_best_width;

        let bits_to_analyze = if self.show_original {
            &self.original_bits
        } else {
            &self.processed_bits
        };

        if bits_to_analyze.is_empty() {
            self.error_message = Some("No data to analyze".to_string());
            return;
        }

        // Run analysis
        let analysis = find_best_width(
            bits_to_analyze,
            self.frame_width_min,
            self.frame_width_max,
            self.frame_width_delta,
        );

        self.frame_width_analysis = Some(analysis);
    }

    /// Request confirmation for a destructive action
    pub fn request_confirmation(&mut self, action: PendingConfirmation, message: String) {
        self.pending_confirmation = Some(action);
        self.confirmation_message = message;
        self.show_confirmation_dialog = true;
    }

    /// Execute a confirmed action
    pub fn execute_confirmed_action(&mut self) {
        if let Some(action) = self.pending_confirmation.take() {
            match action {
                PendingConfirmation::DeleteWorksheet(idx) => {
                    self.worksheets.remove(idx);
                    if self.current_worksheet_index >= self.worksheets.len() {
                        self.current_worksheet_index = self.worksheets.len() - 1;
                    }
                    if self.current_worksheet_index == idx || idx < self.current_worksheet_index {
                        self.load_from_worksheet();
                    }
                }
                PendingConfirmation::DeleteOperation(idx) => {
                    self.operations.remove(idx);
                    self.clear_pattern_matches();
                    self.apply_operations();
                    self.mark_unsaved();
                }
                PendingConfirmation::ClearAllOperations => {
                    self.operations.clear();
                    self.clear_pattern_matches();
                    self.processed_bits = self.original_bits.clone();
                    self.update_viewer();
                    self.mark_unsaved();
                }
                PendingConfirmation::OverwriteFile(path) => {
                    let bits_to_save = if self.show_original {
                        &self.original_bits
                    } else {
                        &self.processed_bits
                    };

                    match write_bits_to_file(&path, bits_to_save) {
                        Ok(_) => {
                            self.error_message = None;
                            self.show_success(format!("File saved: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                            self.mark_saved();
                        }
                        Err(e) => {
                            self.show_error(format!("Failed to save file: {}", e), true);
                        }
                    }
                }
                PendingConfirmation::CloseApp => {
                    // User confirmed they want to close without saving
                    self.close_requested = true;
                }
            }
        }
        self.show_confirmation_dialog = false;
    }

    /// Cancel a pending confirmation
    pub fn cancel_confirmation(&mut self) {
        self.pending_confirmation = None;
        self.show_confirmation_dialog = false;
        self.confirmation_message.clear();
    }

    /// Show a toast notification (respects notification_display_mode setting)
    pub fn show_toast(&mut self, message: String, toast_type: ToastType) {
        use crate::storage::NotificationDisplayMode;

        match self.settings.notification_display_mode {
            NotificationDisplayMode::Toast => {
                let duration = match toast_type {
                    ToastType::Info => 3.0,
                    ToastType::Success => 2.5,
                    ToastType::Warning => 4.0,
                    ToastType::Error => 5.0,
                };
                self.toasts.push(Toast::new(message, toast_type, duration));
            }
            NotificationDisplayMode::Dialog => {
                self.notification_dialog_message = message;
                self.notification_dialog_type = Some(toast_type);
                self.show_notification_dialog = true;
            }
            NotificationDisplayMode::None => {
                // Don't show notification
            }
        }
    }

    /// Remove expired toasts
    pub fn clean_expired_toasts(&mut self) {
        self.toasts.retain(|toast| !toast.is_expired());
    }

    /// Show an error message (either as toast for transient errors or modal for critical)
    pub fn show_error(&mut self, message: String, is_critical: bool) {
        if is_critical {
            self.error_message = Some(message);
        } else {
            self.show_toast(message, ToastType::Error);
        }
    }

    /// Show a success message as a toast
    pub fn show_success(&mut self, message: String) {
        self.show_toast(message, ToastType::Success);
    }

    /// Show an info message as a toast
    pub fn show_info(&mut self, message: String) {
        self.show_toast(message, ToastType::Info);
    }

    /// Show a warning message as a toast
    pub fn show_warning(&mut self, message: String) {
        self.show_toast(message, ToastType::Warning);
    }

    /// Mark that there are unsaved changes
    pub fn mark_unsaved(&mut self) {
        self.has_unsaved_changes = true;
        // Also mark the current worksheet as modified
        if self.current_worksheet_index < self.worksheets.len() {
            self.worksheets[self.current_worksheet_index].mark_modified();
        }
    }

    /// Mark that all changes are saved
    pub fn mark_saved(&mut self) {
        self.has_unsaved_changes = false;
        // Also mark the current worksheet as saved
        if self.current_worksheet_index < self.worksheets.len() {
            self.worksheets[self.current_worksheet_index].mark_saved();
        }
    }
}
