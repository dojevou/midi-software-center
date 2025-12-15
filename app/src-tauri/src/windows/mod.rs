#[allow(dead_code)]
pub mod commands;
/// Window Management System for MIDI Software Center
///
/// Provides a comprehensive window management system similar to Pro Tools,
/// featuring multi-window support, docking, layout persistence, and keyboard shortcuts.
///
/// # Architecture
///
/// - `manager`: Core window management logic
/// - `state`: Window state structures (windowing/docking)
/// - `daw_state`: DAW-specific state (transport, mixer, tracks)
/// - `commands`: Tauri command handlers
/// - `menu`: Menu creation functions
/// - `shortcuts`: Global shortcut registration
/// - `layout`: Layout persistence
/// - `pipeline_state`: Pipeline window-specific state (processing, progress)
pub mod daw_state;
pub mod manager;
pub mod state;
// NOTE: Menu and shortcuts modules disabled pending Tauri 2.x migration
// These modules use Tauri 1.x APIs that changed in Tauri 2.x
// Once migrated to Tauri 2.x menu/shortcuts APIs, uncomment:
// pub mod menu;
// pub mod shortcuts;
pub mod layout;
pub mod pipeline_state;

pub use layout::Layout;
pub use manager::WindowManager;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::windows::state::{WindowInfo, WindowType};

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
