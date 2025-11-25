use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvertBitsOperation {
    pub name: String,
    pub enabled: bool,
}

impl InvertBitsOperation {
    pub fn description(&self) -> String {
        "Inverts all bits".to_string()
    }

    /// Apply invert operation to the input bits.
    /// Takes ownership of input to avoid cloning large data structures.
    pub fn apply(&self, mut input: BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        input.iter_mut().for_each(|mut bit| *bit = !*bit);
        input
    }
}
