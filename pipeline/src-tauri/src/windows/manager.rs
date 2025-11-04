//! Window manager - core logic for managing application windows
//!
//! Handles creation, destruction, positioning, and layout management of windows.

use std::collections::HashMap;

use crate::windows::state::{WindowInfo, WindowState, WindowType, Position, DockSide};
use crate::windows::layout::{Layout, LayoutStorage};

/// Central window management system
pub struct WindowManager {
    state: WindowState,
    layout_storage: Option<LayoutStorage>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        WindowManager {
            state: WindowState::new(),
            layout_storage: None,
        }
    }

    /// Create with layout persistence
    pub fn with_storage(layout_dir: std::path::PathBuf) -> Result<Self, String> {
        let storage = LayoutStorage::new(layout_dir)?;
        Ok(WindowManager {
            state: WindowState::new(),
            layout_storage: Some(storage),
        })
    }

    // ========== Window Registration & Management ==========

    /// Register a new window
    pub fn register_window(&mut self, window: WindowInfo) {
        self.state.add_window(window);
    }

    /// Unregister a window
    pub fn unregister_window(&mut self, label: &str) -> Result<(), String> {
        self.state
            .remove_window(label)
            .ok_or_else(|| format!("Window {} not found", label))?;
        Ok(())
    }

    /// Get window info
    pub fn get_window(&self, label: &str) -> Option<WindowInfo> {
        self.state.get_window(label).cloned()
    }

    /// Show a window
    pub fn show_window(&mut self, label: &str) -> Result<(), String> {
        let window = self
            .state
            .get_window_mut(label)
            .ok_or_else(|| format!("Window {} not found", label))?;

        window.visible = true;
        self.state.bring_to_front(label);
        Ok(())
    }

    /// Hide a window
    pub fn hide_window(&mut self, label: &str) -> Result<(), String> {
        let window = self
            .state
            .get_window_mut(label)
            .ok_or_else(|| format!("Window {} not found", label))?;

        // Can't hide main window
        if matches!(window.window_type, WindowType::Main) {
            return Err("Cannot hide main window".to_string());
        }

        window.visible = false;
        Ok(())
    }

    /// Toggle window visibility
    pub fn toggle_window(&mut self, label: &str) -> Result<(), String> {
        if let Some(window) = self.state.get_window(label) {
            if window.visible {
                self.hide_window(label)
            } else {
                self.show_window(label)
            }
        } else {
            Err(format!("Window {} not found", label))
        }
    }

    /// Set window position
    pub fn set_position(&mut self, label: &str, position: Position) -> Result<(), String> {
        let window = self
            .state
            .get_window_mut(label)
            .ok_or_else(|| format!("Window {} not found", label))?;

        if position.is_valid() {
            window.position = position;
            Ok(())
        } else {
            Err("Invalid position".to_string())
        }
    }

    /// Get window position
    pub fn get_position(&self, label: &str) -> Option<Position> {
        self.state.get_window(label).map(|w| w.position.clone())
    }

    // ========== Window Arrangement ==========

    /// Tile windows vertically
    pub fn tile_windows_vertical(&mut self) -> Result<(), String> {
        let visible_windows = self.state.get_visible_windows();
        if visible_windows.is_empty() {
            return Ok(());
        }

        let window_count = visible_windows.len();
        let window_width = 1920 / window_count as u32; // Assume 1920 width screen
        let window_height = 1080;

        // Collect labels to avoid borrow checker issues
        let labels: Vec<String> = visible_windows.iter().map(|w| w.label.clone()).collect();

        for (index, label) in labels.iter().enumerate() {
            let x = (index as u32 * window_width) as i32;
            let position = Position::new(x, 0, window_width, window_height);
            self.set_position(label, position)?;
        }

        Ok(())
    }

    /// Tile windows horizontally
    pub fn tile_windows_horizontal(&mut self) -> Result<(), String> {
        let visible_windows = self.state.get_visible_windows();
        if visible_windows.is_empty() {
            return Ok(());
        }

        let window_count = visible_windows.len();
        let window_width = 1920;
        let window_height = 1080 / window_count as u32;

        // Collect labels to avoid borrow checker issues
        let labels: Vec<String> = visible_windows.iter().map(|w| w.label.clone()).collect();

        for (index, label) in labels.iter().enumerate() {
            let y = (index as u32 * window_height) as i32;
            let position = Position::new(0, y, window_width, window_height);
            self.set_position(label, position)?;
        }

        Ok(())
    }

    /// Cascade windows
    pub fn cascade_windows(&mut self) -> Result<(), String> {
        let visible_windows = self.state.get_visible_windows();
        let offset = 30;

        // Collect labels to avoid borrow checker issues
        let labels: Vec<String> = visible_windows.iter().map(|w| w.label.clone()).collect();

        for (index, label) in labels.iter().enumerate() {
            let x = (index as i32 * offset) + 100;
            let y = (index as i32 * offset) + 100;
            let position = Position::new(x, y, 800, 600);
            self.set_position(label, position)?;
        }

        Ok(())
    }

    // ========== Docking ==========

    /// Dock a window to another window
    pub fn dock_window(&mut self, label: &str, parent: &str, side: DockSide) -> Result<(), String> {
        // Verify parent exists
        if !self.state.window_exists(parent) {
            return Err(format!("Parent window {} not found", parent));
        }

        let window = self
            .state
            .get_window_mut(label)
            .ok_or_else(|| format!("Window {} not found", label))?;

        if !matches!(window.window_type, WindowType::Dockable) {
            return Err(format!("Window {} cannot be docked", label));
        }

        window.docking.docked_to = Some(parent.to_string());
        window.docking.side = side;

        Ok(())
    }

    /// Undock a window
    pub fn undock_window(&mut self, label: &str) -> Result<(), String> {
        let window = self
            .state
            .get_window_mut(label)
            .ok_or_else(|| format!("Window {} not found", label))?;

        window.docking.docked_to = None;
        Ok(())
    }

    /// Get docked windows
    pub fn get_docked_windows(&self, parent: &str) -> Vec<WindowInfo> {
        self.state
            .windows
            .values()
            .filter(|w| {
                w.docking
                    .docked_to
                    .as_ref()
                    .map_or(false, |p| p == parent)
            })
            .cloned()
            .collect()
    }

    // ========== Layout Management ==========

    /// Save current layout
    pub fn save_layout(&mut self, name: String) -> Result<(), String> {
        let storage = self
            .layout_storage
            .as_ref()
            .ok_or("Layout storage not configured")?;

        let mut layout = Layout::new(&name);

        for (label, window) in &self.state.windows {
            layout.add_window(label.clone(), window.position.clone());
        }

        storage.save_layout(&layout)?;
        self.state.saved_layouts.insert(name, layout);

        Ok(())
    }

    /// Load a layout
    pub fn load_layout(&mut self, name: &str) -> Result<(), String> {
        let storage = self
            .layout_storage
            .as_ref()
            .ok_or("Layout storage not configured")?;

        let layout = storage.load_layout(name)?;

        for (label, position) in &layout.windows {
            self.set_position(label, position.clone())?;
        }

        self.state.current_layout = name.to_string();
        self.state.saved_layouts.insert(name.to_string(), layout);

        Ok(())
    }

    /// Delete a layout
    pub fn delete_layout(&mut self, name: &str) -> Result<(), String> {
        let storage = self
            .layout_storage
            .as_ref()
            .ok_or("Layout storage not configured")?;

        storage.delete_layout(name)?;
        self.state.saved_layouts.remove(name);

        Ok(())
    }

    /// List all available layouts
    pub fn list_layouts(&self) -> Result<Vec<String>, String> {
        let storage = self
            .layout_storage
            .as_ref()
            .ok_or("Layout storage not configured")?;

        storage.list_layouts()
    }

    /// Get current layout
    pub fn get_current_layout(&self) -> String {
        self.state.current_layout.clone()
    }

    // ========== Queries ==========

    /// Get all windows
    pub fn get_all_windows(&self) -> Vec<WindowInfo> {
        self.state.windows.values().cloned().collect()
    }

    /// Get visible windows
    pub fn get_visible_windows(&self) -> Vec<WindowInfo> {
        self.state
            .get_visible_windows()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get windows by type
    pub fn get_windows_by_type(&self, wtype: WindowType) -> Vec<WindowInfo> {
        self.state
            .get_windows_by_type(wtype)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get window count
    pub fn get_window_count(&self) -> usize {
        self.state.window_count()
    }

    /// Get visible window count
    pub fn get_visible_count(&self) -> usize {
        self.state.visible_count()
    }

    /// Check if window exists
    pub fn window_exists(&self, label: &str) -> bool {
        self.state.window_exists(label)
    }

    // ========== Focus Management ==========

    /// Set focused window
    pub fn set_focused(&mut self, label: Option<&str>) {
        self.state.set_focused(label.map(String::from));
    }

    /// Get focused window
    pub fn get_focused(&self) -> Option<WindowInfo> {
        self.state.get_focused().cloned()
    }

    /// Bring window to front
    pub fn bring_to_front(&mut self, label: &str) {
        self.state.bring_to_front(label);
    }

    /// Get window order
    pub fn get_window_order(&self) -> Vec<String> {
        self.state.get_window_order()
    }

    // ========== State Management ==========

    /// Get complete state (for persistence)
    pub fn get_state(&self) -> WindowState {
        self.state.clone()
    }

    /// Restore from saved state
    pub fn restore_state(&mut self, state: WindowState) {
        self.state = state;
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = WindowManager::new();
        assert_eq!(manager.get_window_count(), 0);
    }

    #[test]
    fn test_window_registration() {
        let mut manager = WindowManager::new();
        let window = WindowInfo::new("test", "Test Window", WindowType::Floating);

        manager.register_window(window);
        assert_eq!(manager.get_window_count(), 1);
        assert!(manager.window_exists("test"));
    }

    #[test]
    fn test_show_hide_window() {
        let mut manager = WindowManager::new();
        let window = WindowInfo::new("test", "Test", WindowType::Floating);

        manager.register_window(window);
        manager.hide_window("test").unwrap();
        assert_eq!(manager.get_visible_count(), 0);

        manager.show_window("test").unwrap();
        assert_eq!(manager.get_visible_count(), 1);
    }

    #[test]
    fn test_position_operations() {
        let mut manager = WindowManager::new();
        let window = WindowInfo::new("test", "Test", WindowType::Floating);

        manager.register_window(window);

        let pos = Position::new(0, 0, 800, 600);
        manager.set_position("test", pos.clone()).unwrap();

        let retrieved = manager.get_position("test").unwrap();
        assert_eq!(retrieved, pos);
    }

    #[test]
    fn test_focus_management() {
        let mut manager = WindowManager::new();
        let window = WindowInfo::new("test", "Test", WindowType::Floating);

        manager.register_window(window);
        manager.set_focused(Some("test"));

        let focused = manager.get_focused().unwrap();
        assert_eq!(focused.label, "test");
    }
}
