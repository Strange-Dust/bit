use crate::storage::{RecentFiles, TemplateLibrary};
use crate::viewers::BitShape;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// How to display notifications (success, info, warning messages)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationDisplayMode {
    /// Show as toast notifications (bottom-right corner)
    Toast,
    /// Show as modal dialog popups
    Dialog,
    /// Don't show notifications at all
    None,
}

impl Default for NotificationDisplayMode {
    fn default() -> Self {
        Self::Toast
    }
}

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
    #[serde(default)]
    pub notification_display_mode: NotificationDisplayMode,
    #[serde(default = "TemplateLibrary::with_defaults")]
    pub template_library: TemplateLibrary,
    #[serde(default)]
    pub recent_files: RecentFiles,

    // Workspace layout settings
    #[serde(default = "default_available_ops_width")]
    pub available_operations_panel_width: f32,
    #[serde(default = "default_active_ops_width")]
    pub active_operations_panel_width: f32,
}

fn default_available_ops_width() -> f32 {
    200.0
}

fn default_active_ops_width() -> f32 {
    300.0
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
            notification_display_mode: NotificationDisplayMode::default(),
            template_library: TemplateLibrary::with_defaults(),
            recent_files: RecentFiles::new(),
            available_operations_panel_width: default_available_ops_width(),
            active_operations_panel_width: default_active_ops_width(),
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
        let json = serde_json::to_string(self)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.font_size, deserialized.font_size);
        assert_eq!(settings.frame_length, deserialized.frame_length);
        assert_eq!(settings.notification_display_mode, deserialized.notification_display_mode);
    }

    #[test]
    fn test_workspace_layout_defaults() {
        let settings = AppSettings::default();
        assert_eq!(settings.available_operations_panel_width, 200.0);
        assert_eq!(settings.active_operations_panel_width, 300.0);
    }

    #[test]
    fn test_workspace_layout_persistence() {
        let mut settings = AppSettings::default();
        settings.available_operations_panel_width = 250.0;
        settings.active_operations_panel_width = 350.0;

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.available_operations_panel_width, 250.0);
        assert_eq!(deserialized.active_operations_panel_width, 350.0);
    }

    #[test]
    fn test_backward_compatibility() {
        // Old settings JSON without workspace layout fields
        let old_json = r#"{
            "bit_shape": "Square",
            "show_grid": true,
            "thick_grid_interval_horizontal": 0,
            "thick_grid_interval_vertical": 0,
            "thick_grid_spacing_horizontal": 0.0,
            "thick_grid_spacing_vertical": 0.0,
            "font_size": 14.0,
            "frame_length": 64
        }"#;

        let settings: AppSettings = serde_json::from_str(old_json).unwrap();

        // Should use default values for missing fields
        assert_eq!(settings.available_operations_panel_width, 200.0);
        assert_eq!(settings.active_operations_panel_width, 300.0);
        assert_eq!(settings.notification_display_mode, NotificationDisplayMode::Toast);
    }
}
