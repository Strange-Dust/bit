// Analysis module - pattern matching and bit analysis tools

pub mod pattern_locator;
pub mod frame_width;

pub use pattern_locator::{Pattern, PatternFormat, PatternLocatorState, PatternMatch, MergedMatch, MatchFilter, merge_matches};
pub use frame_width::{FrameWidthAnalysis, find_best_width};
