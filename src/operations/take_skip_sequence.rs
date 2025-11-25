use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use super::base::OperationSequence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeSkipSequenceOperation {
    pub name: String,
    pub sequence: OperationSequence,
    pub enabled: bool,
}

impl TakeSkipSequenceOperation {
    pub fn description(&self) -> String {
        self.sequence.to_string()
    }

    /// Apply take/skip sequence operation to input bits.
    /// Delegates to OperationSequence which takes a reference for sequential processing.
    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        self.sequence.apply(input)
    }
}
