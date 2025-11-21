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

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let mut result = input.clone();
        result.iter_mut().for_each(|mut bit| *bit = !*bit);
        result
    }
}
