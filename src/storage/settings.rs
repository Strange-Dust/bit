use crate::viewers::BitShape;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub bit_shape: BitShape,
    pub show_grid: bool,
    pub thick_grid_interval_horizontal: usize,
    pub thick_grid_interval_vertical: usize,
    pub thick_grid_spacing_horizontal: f32,
    pub thick_grid_spacing_vertical: f32,
    pub font_size: f32,
    pub frame_length: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            bit_shape: BitShape::Square,
            show_grid: true,
            thick_grid_interval_horizontal: 0,
            thick_grid_interval_vertical: 0,
            thick_grid_spacing_horizontal: 0.0,
            thick_grid_spacing_vertical: 0.0,
            font_size: 14.0,
            frame_length: 64,
        }
    }
}

impl AppSettings {
    pub fn settings_file_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("bit");
        std::fs::create_dir_all(&path).ok();
        path.push("settings.json");
        path
    }
    
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
        Ok(())
    }
    
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
        let settings: AppSettings = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;
        
        Ok(settings)
    }
    
    pub fn auto_save(&self) {
        let path = Self::settings_file_path();
        self.save_to_file(&path).ok(); // Ignore errors for auto-save
    }
    
    pub fn auto_load() -> Self {
        let path = Self::settings_file_path();
        if path.exists() {
            Self::load_from_file(&path).unwrap_or_default()
        } else {
            Self::default()
        }
    }
}
