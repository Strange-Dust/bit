use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::base::OperationSequence;

/// Represents a take/skip operation to apply to a specific worksheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetOperation {
    pub worksheet_index: usize,
    pub sequence: OperationSequence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiWorksheetLoadOperation {
    pub name: String,
    pub worksheet_operations: Vec<WorksheetOperation>,
    pub enabled: bool,
}

impl MultiWorksheetLoadOperation {
    pub fn description(&self) -> String {
        format!("Load from {} worksheet(s)", self.worksheet_operations.len())
    }

    /// Apply multi-worksheet load operation.
    /// Takes ownership for consistency, though this operation type requires worksheet data
    /// and is handled specially in the main application. Returns empty here.
    pub fn apply(&self, _input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        BitVec::new()
    }
}
