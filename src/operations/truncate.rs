use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncateBitsOperation {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub enabled: bool,
}

impl TruncateBitsOperation {
    pub fn new(name: String, start: usize, end: usize) -> Self {
        Self {
            name,
            start,
            end,
            enabled: true,
        }
    }

    pub fn description(&self) -> String {
        format!("Keep bits {}-{}", self.start, self.end)
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        let len = input.len();
        let actual_start = self.start.min(len);
        let actual_end = self.end.min(len);

        if actual_start >= actual_end {
            return BitVec::new();
        }

        input[actual_start..actual_end].to_bitvec()
    }
}
