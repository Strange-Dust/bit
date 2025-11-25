//! Application state management modules
//!
//! This module provides organized state management for the B.I.T. application.
//! It contains specialized state managers for different asynchronous operations,
//! keeping the main application struct clean and focused.
//!
//! # Modules
//!
//! * [`loading`] - Manages file loading state and progress
//! * [`operation_processing`] - Manages bit operation processing state and progress
//!
//! # Example
//!
//! ```rust,no_run
//! use bit::app_state::{LoadingState, OperationProcessingState};
//!
//! struct MyApp {
//!     loading_state: LoadingState,
//!     operation_state: OperationProcessingState,
//! }
//!
//! impl MyApp {
//!     fn update(&mut self) {
//!         // Poll for loading completion
//!         if let Some(complete) = self.loading_state.poll() {
//!             // Handle loaded file
//!         }
//!
//!         // Poll for operation completion
//!         if let Some(complete) = self.operation_state.poll() {
//!             // Handle processed data
//!         }
//!     }
//! }
//! ```

pub mod loading;
pub mod operation_processing;

pub use loading::LoadingState;
pub use operation_processing::OperationProcessingState;
