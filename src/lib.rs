// Library exports for testing and external use
pub mod analysis;
pub mod app;
pub mod core;
pub mod processing;
pub mod storage;
pub mod ui;
pub mod utils;
pub mod viewers;

// Re-export commonly used items for convenience
pub use analysis::{Pattern, PatternFormat};
pub use app::BitApp;
pub use core::{OperationType, ViewMode};
pub use processing::{BitOperation, Operation, OperationSequence, WorksheetOperation};
pub use storage::{read_file_as_bits, write_bits_to_file, AppSession, AppSettings, Worksheet};
pub use viewers::{BitShape, BitViewer, ByteColumn, ByteViewer};
