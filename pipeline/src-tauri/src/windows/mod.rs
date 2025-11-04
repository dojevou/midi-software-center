//! Window Management System for MIDI Software Center
//!
//! Provides a comprehensive window management system similar to Pro Tools,
//! featuring multi-window support, docking, layout persistence, and keyboard shortcuts.
//!
//! # Architecture
//!
//! - `manager`: Core window management logic
//! - `state`: Window state structures
//! - `commands`: Tauri command handlers
//! - `menu`: Menu creation functions
//! - `shortcuts`: Global shortcut registration
//! - `layout`: Layout persistence
//! - `pipeline_state`: Pipeline window-specific state (processing, progress)

pub mod manager;
pub mod state;
pub mod commands;
// TODO: Fix Tauri 2.x API compatibility
// pub mod menu;
// pub mod shortcuts;
pub mod layout;
pub mod pipeline_state;

pub use manager::WindowManager;
pub use state::{WindowState, WindowInfo, WindowType, Position, Docking, DockSide};
pub use layout::Layout;
pub use pipeline_state::{
    PipelineWindowState, ProcessingStatus, ProcessingStats, OperationType,
    LogMessage, LogLevel,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_creation() {
        let manager = WindowManager::new();
        assert!(manager.get_window_count() == 0);
    }

    #[test]
    fn test_window_registration() {
        let mut manager = WindowManager::new();
        let window_info = WindowInfo::new("test", "Test Window", WindowType::Floating);
        manager.register_window(window_info);
        assert!(manager.get_window_count() == 1);
    }

    #[test]
    fn test_layout_creation() {
        let layout = Layout::new("default");
        assert_eq!(layout.name, "default");
        assert!(layout.windows.is_empty());
    }
}
