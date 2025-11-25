// Recent files tracking

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFile {
    pub path: PathBuf,
    pub last_opened: std::time::SystemTime,
}

impl RecentFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            last_opened: std::time::SystemTime::now(),
        }
    }

    /// Get file size in bytes
    pub fn file_size(&self) -> Option<u64> {
        std::fs::metadata(&self.path).ok().map(|m| m.len())
    }

    /// Get file size as human-readable string
    pub fn file_size_string(&self) -> String {
        match self.file_size() {
            Some(size) => {
                if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else if size < 1024 * 1024 * 1024 {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                }
            }
            None => "Unknown".to_string(),
        }
    }

    /// Get file name without path
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Check if file still exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentFiles {
    files: Vec<RecentFile>,
    max_entries: usize,
}

impl RecentFiles {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            max_entries: 10,
        }
    }

    /// Add a file to recent files (or update its timestamp if already present)
    pub fn add(&mut self, path: PathBuf) {
        // Remove existing entry if present
        self.files.retain(|f| f.path != path);

        // Add new entry at the beginning
        self.files.insert(0, RecentFile::new(path));

        // Trim to max entries
        if self.files.len() > self.max_entries {
            self.files.truncate(self.max_entries);
        }
    }

    /// Get all recent files
    pub fn list(&self) -> &[RecentFile] {
        &self.files
    }

    /// Clear all recent files
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// Remove non-existent files
    pub fn cleanup(&mut self) {
        self.files.retain(|f| f.exists());
    }

    /// Remove a specific file from recent files
    pub fn remove(&mut self, index: usize) {
        if index < self.files.len() {
            self.files.remove(index);
        }
    }

    /// Get file path for a recent files entry
    pub fn get_path(&self, index: usize) -> Option<PathBuf> {
        self.files.get(index).map(|f| f.path.clone())
    }
}

impl Default for RecentFile {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            last_opened: std::time::SystemTime::now(),
        }
    }
}
