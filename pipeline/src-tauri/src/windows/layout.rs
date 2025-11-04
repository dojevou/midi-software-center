//! Window layout persistence and management
//!
//! Handles saving, loading, and managing window layouts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::windows::state::Position;

/// A saved window layout
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layout {
    /// Unique layout identifier
    pub name: String,
    /// Description of the layout
    pub description: Option<String>,
    /// Window positions and sizes in this layout
    pub windows: HashMap<String, Position>,
    /// When this layout was created
    pub created_at: u64,
    /// Last modified timestamp
    pub updated_at: u64,
    /// Is this layout locked (can't be modified)
    pub locked: bool,
}

impl Layout {
    /// Create a new layout
    pub fn new(name: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Layout {
            name: name.to_string(),
            description: None,
            windows: HashMap::new(),
            created_at: now,
            updated_at: now,
            locked: false,
        }
    }

    /// Set layout description
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    /// Add window position to layout
    pub fn add_window(&mut self, label: String, position: Position) {
        self.windows.insert(label, position);
    }

    /// Remove window from layout
    pub fn remove_window(&mut self, label: &str) {
        self.windows.remove(label);
    }

    /// Get window position
    pub fn get_window(&self, label: &str) -> Option<&Position> {
        self.windows.get(label)
    }

    /// Update modification time
    pub fn touch(&mut self) {
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Check if layout can be modified
    pub fn can_modify(&self) -> bool {
        !self.locked
    }

    /// Lock the layout to prevent modifications
    pub fn lock(&mut self) {
        self.locked = true;
    }

    /// Unlock the layout
    pub fn unlock(&mut self) {
        self.locked = false;
    }

    /// Get window count in layout
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

/// Layout storage and persistence
pub struct LayoutStorage {
    storage_dir: PathBuf,
}

impl LayoutStorage {
    /// Create new layout storage
    pub fn new(storage_dir: PathBuf) -> Result<Self, String> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&storage_dir)
            .map_err(|e| format!("Failed to create storage directory: {}", e))?;

        Ok(LayoutStorage { storage_dir })
    }

    /// Save a layout to disk
    pub fn save_layout(&self, layout: &Layout) -> Result<(), String> {
        let file_path = self.storage_dir.join(format!("{}.json", layout.name));

        let json = serde_json::to_string_pretty(layout)
            .map_err(|e| format!("Failed to serialize layout: {}", e))?;

        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write layout file: {}", e))?;

        Ok(())
    }

    /// Load a layout from disk
    pub fn load_layout(&self, name: &str) -> Result<Layout, String> {
        let file_path = self.storage_dir.join(format!("{}.json", name));

        let json = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read layout file: {}", e))?;

        let layout = serde_json::from_str::<Layout>(&json)
            .map_err(|e| format!("Failed to parse layout file: {}", e))?;

        Ok(layout)
    }

    /// Delete a layout
    pub fn delete_layout(&self, name: &str) -> Result<(), String> {
        let file_path = self.storage_dir.join(format!("{}.json", name));

        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete layout file: {}", e))?;

        Ok(())
    }

    /// List all available layouts
    pub fn list_layouts(&self) -> Result<Vec<String>, String> {
        let mut layouts = Vec::new();

        let entries = fs::read_dir(&self.storage_dir)
            .map_err(|e| format!("Failed to read storage directory: {}", e))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| format!("Failed to read directory entry: {}", e))?;

            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    layouts.push(name.to_string());
                }
            }
        }

        layouts.sort();
        Ok(layouts)
    }

    /// Check if layout exists
    pub fn layout_exists(&self, name: &str) -> bool {
        let file_path = self.storage_dir.join(format!("{}.json", name));
        file_path.exists()
    }

    /// Export layout to JSON string
    pub fn export_layout(&self, layout: &Layout) -> Result<String, String> {
        serde_json::to_string_pretty(layout)
            .map_err(|e| format!("Failed to export layout: {}", e))
    }

    /// Import layout from JSON string
    pub fn import_layout(&self, json: &str) -> Result<Layout, String> {
        serde_json::from_str::<Layout>(json)
            .map_err(|e| format!("Failed to import layout: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_creation() {
        let layout = Layout::new("test");
        assert_eq!(layout.name, "test");
        assert!(layout.windows.is_empty());
    }

    #[test]
    fn test_layout_with_description() {
        let layout = Layout::new("test").with_description("Test layout".to_string());
        assert_eq!(layout.description, Some("Test layout".to_string()));
    }

    #[test]
    fn test_layout_window_operations() {
        let mut layout = Layout::new("test");
        let pos = Position::new(0, 0, 800, 600);

        layout.add_window("window1".to_string(), pos.clone());
        assert_eq!(layout.window_count(), 1);
        assert!(layout.get_window("window1").is_some());

        layout.remove_window("window1");
        assert_eq!(layout.window_count(), 0);
    }

    #[test]
    fn test_layout_locking() {
        let mut layout = Layout::new("test");
        assert!(layout.can_modify());

        layout.lock();
        assert!(!layout.can_modify());

        layout.unlock();
        assert!(layout.can_modify());
    }

    #[test]
    fn test_layout_serialization() {
        let mut layout = Layout::new("test");
        layout.add_window("window1".to_string(), Position::new(0, 0, 800, 600));

        let json = serde_json::to_string(&layout).unwrap();
        let deserialized: Layout = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, layout.name);
        assert_eq!(deserialized.window_count(), layout.window_count());
    }
}
