use bitvec::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadFileOperation {
    pub name: String,
    pub file_path: PathBuf,
    pub enabled: bool,
}

impl LoadFileOperation {
    pub fn new(name: String, file_path: PathBuf) -> Self {
        Self {
            name,
            file_path,
            enabled: true,
        }
    }

    pub fn description(&self) -> String {
        format!(
            "Load: {}",
            self.file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        )
    }

    pub fn apply(&self, input: &BitVec<u8, Msb0>) -> BitVec<u8, Msb0> {
        // LoadFile operations are handled specially in the main application
        // since they need file I/O. Return the input unchanged here.
        input.clone()
    }
}
