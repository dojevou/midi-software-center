#[allow(dead_code)]
/// Window state structures and types
///
/// Defines the core data structures for tracking window state,
/// including window types, positions, and docking information.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents different types of windows in the application
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WindowType {
    /// Main application window - persistent, can't close
    Main,
    /// Dockable panel - can be attached to other panels
    Dockable,
    /// Floating window - independent, always on top
    Floating,
    /// Modal dialog - blocks interaction with other windows
    Modal,
    /// Tool palette - always visible, minimal
    Palette,
}

#[allow(dead_code)]
impl WindowType {
    pub fn is_main(&self) -> bool {
        matches!(self, WindowType::Main)
    }

    pub fn is_modal(&self) -> bool {
        matches!(self, WindowType::Modal)
    }

    pub fn is_floating(&self) -> bool {
        matches!(self, WindowType::Floating)
    }
}

/// Window position and size information
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[allow(dead_code)]
impl Position {
    /// Create a new position
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Position { x, y, width, height, maximized: false }
    }

    /// Create a position with maximized flag
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Check if position is valid
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    /// Get center coordinates
    pub fn center(&self) -> (i32, i32) {
        let center_x = self.x + (self.width as i32 / 2);
        let center_y = self.y + (self.height as i32 / 2);
        (center_x, center_y)
    }
}

/// Docking information for dockable windows
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Docking {
    pub docked_to: Option<String>, // Label of the parent window
    pub side: DockSide,
    pub size_ratio: f32, // 0.0 to 1.0
}

impl Default for Docking {
    fn default() -> Self {
        Docking { docked_to: None, side: DockSide::Right, size_ratio: 0.3 }
    }
}

/// Which side of a window to dock to
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DockSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Information about a single window
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowInfo {
    pub label: String,
    pub title: String,
    pub window_type: WindowType,
    pub position: Position,
    pub docking: Docking,
    pub visible: bool,
    pub resizable: bool,
    pub closeable: bool,
    pub created_at: u64,
}

#[allow(dead_code)]
impl WindowInfo {
    /// Create a new window info
    pub fn new(label: &str, title: &str, window_type: WindowType) -> Self {
        WindowInfo {
            label: label.to_string(),
            title: title.to_string(),
            window_type,
            position: Position::default(),
            docking: Docking::default(),
            visible: true,
            resizable: true,
            closeable: !matches!(window_type, WindowType::Main),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Mark window as docked
    pub fn with_docking(mut self, docked_to: String, side: DockSide) -> Self {
        self.docking.docked_to = Some(docked_to);
        self.docking.side = side;
        self
    }

    /// Set position
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Set resizable flag
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Check if window can be closed
    pub fn can_close(&self) -> bool {
        self.closeable && !matches!(self.window_type, WindowType::Main)
    }

    /// Get unique key for persistence
    pub fn persist_key(&self) -> String {
        format!("window_{}", self.label)
    }
}

/// Complete application window state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowState {
    /// All windows currently managed
    pub windows: HashMap<String, WindowInfo>,
    /// Currently active layout name
    pub current_layout: String,
    /// All saved layouts
    pub saved_layouts: HashMap<String, crate::windows::Layout>,
    /// Focused window label
    pub focused_window: Option<String>,
    /// Window z-order (for layering)
    pub window_order: Vec<String>,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState {
            windows: HashMap::new(),
            current_layout: "default".to_string(),
            saved_layouts: HashMap::new(),
            focused_window: None,
            window_order: Vec::new(),
        }
    }
}

impl WindowState {
    /// Create new window state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a window to the state
    pub fn add_window(&mut self, window: WindowInfo) {
        self.window_order.push(window.label.clone());
        self.windows.insert(window.label.clone(), window);
    }

    /// Remove a window from the state
    pub fn remove_window(&mut self, label: &str) -> Option<WindowInfo> {
        self.window_order.retain(|w| w != label);
        self.windows.remove(label)
    }

    /// Get a window by label
    pub fn get_window(&self, label: &str) -> Option<&WindowInfo> {
        self.windows.get(label)
    }

    /// Get mutable window reference
    pub fn get_window_mut(&mut self, label: &str) -> Option<&mut WindowInfo> {
        self.windows.get_mut(label)
    }

    /// Get all visible windows
    pub fn get_visible_windows(&self) -> Vec<&WindowInfo> {
        self.windows.values().filter(|w| w.visible).collect()
    }

    /// Get all windows of a specific type
    pub fn get_windows_by_type(&self, wtype: WindowType) -> Vec<&WindowInfo> {
        self.windows.values().filter(|w| w.window_type == wtype).collect()
    }

    /// Check if window exists
    pub fn window_exists(&self, label: &str) -> bool {
        self.windows.contains_key(label)
    }

    /// Get total window count
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Get visible window count
    pub fn visible_count(&self) -> usize {
        self.windows.values().filter(|w| w.visible).count()
    }

    /// Set focused window
    pub fn set_focused(&mut self, label: Option<String>) {
        self.focused_window = label;
    }

    /// Get focused window
    pub fn get_focused(&self) -> Option<&WindowInfo> {
        self.focused_window.as_ref().and_then(|label| self.windows.get(label))
    }

    /// Update window z-order (bring to front)
    pub fn bring_to_front(&mut self, label: &str) {
        self.window_order.retain(|w| w != label);
        self.window_order.push(label.to_string());
    }

    /// Get windows in order (back to front)
    pub fn get_window_order(&self) -> Vec<String> {
        self.window_order.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_validation() {
        let valid = Position::new(0, 0, 800, 600);
        assert!(valid.is_valid());

        let invalid = Position::new(0, 0, 0, 600);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_position_center() {
        let pos = Position::new(0, 0, 800, 600);
        let (cx, cy) = pos.center();
        assert_eq!(cx, 400);
        assert_eq!(cy, 300);
    }

    #[test]
    fn test_window_info_creation() {
        let window = WindowInfo::new("test", "Test Window", WindowType::Floating);
        assert_eq!(window.label, "test");
        assert_eq!(window.title, "Test Window");
        assert!(window.resizable);
    }

    #[test]
    fn test_window_state_operations() {
        let mut state = WindowState::new();
        let window = WindowInfo::new("test", "Test", WindowType::Floating);

        state.add_window(window);
        assert_eq!(state.window_count(), 1);
        assert!(state.window_exists("test"));

        state.remove_window("test");
        assert_eq!(state.window_count(), 0);
    }

    #[test]
    fn test_window_type_checks() {
        assert!(WindowType::Main.is_main());
        assert!(WindowType::Modal.is_modal());
        assert!(WindowType::Floating.is_floating());
    }
}
