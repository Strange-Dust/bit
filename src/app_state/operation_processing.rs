//! Operation processing state management module
//!
//! This module provides state management for asynchronous bit operation processing.
//! It tracks processing progress through multiple operations and handles completion status.
//!
//! # Example
//!
//! ```rust,no_run
//! use bit::app_state::OperationProcessingState;
//! use std::sync::mpsc::channel;
//!
//! let mut state = OperationProcessingState::new();
//! // Start processing (receiver would come from background thread)
//! # let (tx, rx) = channel();
//! state.start(rx);
//!
//! // Poll for completion in update loop
//! if let Some(complete) = state.poll() {
//!     match complete.result {
//!         Ok(bits) => println!("Processed {} bits", bits.len()),
//!         Err(e) => eprintln!("Processing failed: {}", e),
//!     }
//! }
//! ```

use crate::app::OperationProgress;
use bitvec::prelude::*;
use std::sync::mpsc::Receiver;

/// Represents the result of a completed operation processing run.
///
/// Contains the final bit vector after all operations have been applied,
/// or an error message if processing failed.
pub struct ProcessingComplete {
    /// The result of operation processing (processed bits or error message)
    pub result: Result<BitVec<u8, Msb0>, String>,
}

/// Manages the state of an asynchronous operation processing run.
///
/// This struct encapsulates all state related to processing a chain of bit operations
/// in the background, including progress tracking and status messages. It's designed
/// to be polled periodically from the main UI loop.
///
/// # Fields
///
/// * `receiver` - Channel receiver for operation progress messages
/// * `progress_message` - Human-readable status message describing current operation
/// * `progress` - Processing progress as a value between 0.0 and 1.0
/// * `start_time` - When the processing operation started
#[derive(Default)]
pub struct OperationProcessingState {
    pub receiver: Option<Receiver<OperationProgress>>,
    pub progress_message: String,
    pub progress: f32,
    pub start_time: Option<std::time::Instant>,
}

impl OperationProcessingState {
    /// Creates a new operation processing state with default values (no active processing).
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts tracking an asynchronous operation processing run.
    ///
    /// # Arguments
    ///
    /// * `receiver` - Channel receiver that will receive progress updates
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bit::app_state::OperationProcessingState;
    /// # use std::sync::mpsc::channel;
    /// # let (tx, rx) = channel();
    /// let mut state = OperationProcessingState::new();
    /// state.start(rx);
    /// ```
    pub fn start(&mut self, receiver: Receiver<OperationProgress>) {
        self.receiver = Some(receiver);
        self.progress = 0.0;
        self.progress_message = "Starting...".to_string();
        self.start_time = Some(std::time::Instant::now());
    }

    /// Polls the operation processing for progress updates and completion.
    ///
    /// This method should be called periodically (e.g., each frame) to update
    /// progress messages and detect when processing finishes. It processes all
    /// available messages from the background thread.
    ///
    /// # Returns
    ///
    /// * `Some(ProcessingComplete)` if the processing operation finished
    /// * `None` if still processing or no active processing
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bit::app_state::OperationProcessingState;
    /// # let mut state = OperationProcessingState::new();
    /// // In your update loop:
    /// if let Some(complete) = state.poll() {
    ///     println!("Processing finished");
    /// }
    /// ```
    pub fn poll(&mut self) -> Option<ProcessingComplete> {
        let mut processing_complete: Option<ProcessingComplete> = None;

        if let Some(receiver) = &self.receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    OperationProgress::LoadingFile { path, loaded, total } => {
                        self.progress = if total > 0 { loaded as f32 / total as f32 } else { 0.0 };
                        self.progress_message = format!(
                            "Loading {}: {:.1} MB / {:.1} MB",
                            path.file_name().unwrap_or_default().to_string_lossy(),
                            loaded as f64 / (1024.0 * 1024.0),
                            total as f64 / (1024.0 * 1024.0)
                        );
                    }
                    OperationProgress::ProcessingOperation { index, total, description } => {
                        self.progress = index as f32 / total as f32;
                        self.progress_message = description;
                    }
                    OperationProgress::Complete(result) => {
                        processing_complete = Some(ProcessingComplete { result });
                    }
                }
            }
        }

        // Clear state if complete
        if processing_complete.is_some() {
            self.receiver = None;
            self.start_time = None;
        }

        processing_complete
    }

    /// Checks if an operation processing run is currently in progress.
    ///
    /// # Returns
    ///
    /// * `true` if actively processing operations
    /// * `false` if no processing operation is active
    pub fn is_processing(&self) -> bool {
        self.receiver.is_some()
    }

    /// Calculate estimated time remaining in seconds.
    ///
    /// Returns None if not enough progress has been made for a reliable estimate.
    pub fn estimated_time_remaining(&self) -> Option<f64> {
        if let Some(start_time) = self.start_time {
            if self.progress > 0.01 {
                // Only show ETA if we have at least 1% progress
                let elapsed = start_time.elapsed().as_secs_f64();
                let total_estimated = elapsed / self.progress as f64;
                let remaining = total_estimated - elapsed;
                Some(remaining.max(0.0))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Format estimated time remaining as a human-readable string.
    pub fn eta_string(&self) -> String {
        if let Some(remaining_secs) = self.estimated_time_remaining() {
            if remaining_secs < 1.0 {
                "< 1s".to_string()
            } else if remaining_secs < 60.0 {
                format!("{:.0}s", remaining_secs)
            } else if remaining_secs < 3600.0 {
                let mins = (remaining_secs / 60.0).floor();
                let secs = (remaining_secs % 60.0).floor();
                format!("{}m {}s", mins, secs)
            } else {
                let hours = (remaining_secs / 3600.0).floor();
                let mins = ((remaining_secs % 3600.0) / 60.0).floor();
                format!("{}h {}m", hours, mins)
            }
        } else {
            "Calculating...".to_string()
        }
    }
}
