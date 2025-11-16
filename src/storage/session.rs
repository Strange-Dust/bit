use super::worksheet::Worksheet;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSession {
    pub worksheets: Vec<Worksheet>,
    pub current_worksheet_index: usize,
}

impl AppSession {
    pub fn new(worksheets: Vec<Worksheet>, current_worksheet_index: usize) -> Self {
        Self {
            worksheets,
            current_worksheet_index,
        }
    }
    
    pub fn session_file_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("bit");
        std::fs::create_dir_all(&path).ok();
        path.push("last_session.json");
        path
    }
    
    pub fn save(&self) -> Result<(), String> {
        let path = Self::session_file_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;
        
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write session file: {}", e))?;
        
        Ok(())
    }
    
    pub fn load() -> Result<Self, String> {
        let path = Self::session_file_path();
        
        if !path.exists() {
            return Err("No previous session found".to_string());
        }
        
        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read session file: {}", e))?;
        
        let session: Self = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse session file: {}", e))?;
        
        Ok(session)
    }
    
    pub fn delete() -> Result<(), String> {
        let path = Self::session_file_path();
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete session file: {}", e))?;
        }
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn exists() -> bool {
        Self::session_file_path().exists()
    }
}
