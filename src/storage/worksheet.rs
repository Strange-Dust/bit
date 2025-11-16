use crate::processing::BitOperation;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worksheet {
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub operations: Vec<BitOperation>,
}

impl Worksheet {
    pub fn new(name: String) -> Self {
        Self {
            name,
            file_path: None,
            operations: Vec::new(),
        }
    }
    
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize worksheet: {}", e))?;
        
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write worksheet file: {}", e))?;
        
        Ok(())
    }
    
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read worksheet file: {}", e))?;
        
        let worksheet: Worksheet = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse worksheet file: {}", e))?;
        
        Ok(worksheet)
    }
}
