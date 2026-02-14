// Library exports for testing and external use
pub mod analysis;
pub mod app;
pub mod app_state;
pub mod core;
pub mod operations;
pub mod storage;
pub mod ui;
pub mod utils;
pub mod viewers;

// Re-export commonly used items for convenience
pub use analysis::{Pattern, PatternFormat, PatternLocatorState, PatternMatch, MergedMatch, MatchFilter, merge_matches};
pub use app::BitApp;
pub use core::{BitSelection, ViewMode};
pub use operations::{BitOperation, OperationCategory, OperationSequence, OperationType};
pub use storage::{read_file_as_bits, write_bits_to_file, AppSession, AppSettings, Worksheet};
pub use viewers::{BitShape, BitViewer, ByteColumn, ByteViewer};
