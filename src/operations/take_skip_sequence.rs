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
    pub fn new(name: String, sequence: OperationSequence) -> Self {
        Self {
            name,
            sequence,
            enabled: true,
        }
    }

    pub fn description(&self) -> String {
        self.sequence.to_string()
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        self.sequence.apply(input)
    }
}
