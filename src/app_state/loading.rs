//! File loading state management module
//!
//! This module provides state management for asynchronous file loading operations.
//! It tracks loading progress and handles completion status for large file operations.
//!
//! # Example
//!
//! ```rust,no_run
//! use bit::app_state::LoadingState;
//! use std::sync::mpsc::channel;
//!
//! let mut state = LoadingState::new();
//! // Start loading (receiver would come from background thread)
//! # let (tx, rx) = channel();
//! # let path = std::path::PathBuf::from("test.bin");
//! state.start(rx, path);
//!
//! // Poll for completion in update loop
//! if let Some(complete) = state.poll() {
//!     match complete.result {
//!         Ok(bits) => println!("Loaded {} bits", bits.len()),
//!         Err(e) => eprintln!("Load failed: {}", e),
//!     }
//! }
//! ```

use crate::storage::LoadProgress;
use bitvec::prelude::*;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// Represents the result of a completed file load operation.
///
/// Contains both the loading result and the path of the file that was loaded.
pub struct LoadComplete {
    /// The result of the file load operation (bits or error message)
    pub result: Result<BitVec<u8, Msb0>, String>,
    /// The path of the file that was loaded
    pub path: PathBuf,
}

/// Manages the state of an asynchronous file loading operation.
///
/// This struct encapsulates all state related to loading a file in the background,
/// including progress tracking and completion handling. It's designed to be polled
/// periodically from the main UI loop.
///
/// # Fields
///
/// * `receiver` - Channel receiver for loading progress messages
/// * `file_path` - Path of the file currently being loaded
/// * `progress` - Loading progress as a value between 0.0 and 1.0
/// * `total` - Total number of bytes in the file being loaded
/// * `start_time` - When the loading operation started
#[derive(Default)]
pub struct LoadingState {
    pub receiver: Option<Receiver<LoadProgress>>,
    pub file_path: Option<PathBuf>,
    pub progress: f32,
    pub total: u64,
    pub start_time: Option<std::time::Instant>,
}

impl LoadingState {
    /// Creates a new loading state with default values (no active load).
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts tracking an asynchronous file loading operation.
    ///
    /// # Arguments
    ///
    /// * `receiver` - Channel receiver that will receive progress updates
    /// * `path` - Path of the file being loaded
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bit::app_state::LoadingState;
    /// # use std::sync::mpsc::channel;
    /// # let (tx, rx) = channel();
    /// # let path = std::path::PathBuf::from("data.bin");
    /// let mut state = LoadingState::new();
    /// state.start(rx, path);
    /// ```
    pub fn start(&mut self, receiver: Receiver<LoadProgress>, path: PathBuf) {
        self.receiver = Some(receiver);
        self.file_path = Some(path);
        self.progress = 0.0;
        self.total = 0;
        self.start_time = Some(std::time::Instant::now());
    }

    /// Polls the loading operation for progress updates and completion.
    ///
    /// This method should be called periodically (e.g., each frame) to update
    /// progress and detect when loading finishes. It processes all available
    /// messages from the background thread.
    ///
    /// # Returns
    ///
    /// * `Some(LoadComplete)` if the loading operation finished
    /// * `None` if still loading or no active load
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use bit::app_state::LoadingState;
    /// # let mut state = LoadingState::new();
    /// // In your update loop:
    /// if let Some(complete) = state.poll() {
    ///     println!("Loading finished for {:?}", complete.path);
    /// }
    /// ```
    pub fn poll(&mut self) -> Option<LoadComplete> {
        let mut load_complete: Option<LoadComplete> = None;

        if let Some(receiver) = &self.receiver {
            // Process all available messages
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    LoadProgress::Progress { loaded, total } => {
                        self.total = total;
                        self.progress = if total > 0 {
                            loaded as f32 / total as f32
                        } else {
                            0.0
                        };
                    }
                    LoadProgress::Complete(result) => {
                        // Loading finished
                        if let Some(path) = self.file_path.take() {
                            load_complete = Some(LoadComplete { result, path });
                        }
                    }
                }
            }
        }

        // Clear state if complete
        if load_complete.is_some() {
            self.receiver = None;
            self.file_path = None;
            self.start_time = None;
        }

        load_complete
    }

    /// Checks if a file loading operation is currently in progress.
    ///
    /// # Returns
    ///
    /// * `true` if actively loading a file
    /// * `false` if no load operation is active
    pub fn is_loading(&self) -> bool {
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
