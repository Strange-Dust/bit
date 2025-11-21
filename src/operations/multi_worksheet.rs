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
    pub fn new(name: String, worksheet_operations: Vec<WorksheetOperation>) -> Self {
        Self {
            name,
            worksheet_operations,
            enabled: true,
        }
    }

    pub fn description(&self) -> String {
        format!("Load from {} worksheet(s)", self.worksheet_operations.len())
    }

    pub fn apply(&self, _input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        // This operation type requires worksheet data, so it should be handled
        // differently in the main application. For now, return empty.
        BitVec::new()
    }
}
