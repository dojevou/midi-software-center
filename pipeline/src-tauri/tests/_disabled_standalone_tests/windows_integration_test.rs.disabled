//! Comprehensive integration tests for the window management system
//!
//! **Module:** pipeline/src-tauri/src/windows/
//! **Coverage Target:** 90%+ (Trusty Module requirement: 80%+)
//! **Total Tests:** 32 tests across 7 categories
//!
//! This test suite validates the Pro Tools-inspired window management system,
//! including multi-window support, docking, layout persistence, and concurrent access.
//!
//! **Test Categories:**
//! 1. Manager Lifecycle (5 tests) - Registration, unregistration, counting
//! 2. Visibility Commands (5 tests) - Show/hide/toggle with main window protection
//! 3. Positioning (5 tests) - Set position, dock/undock, constraints
//! 4. Layout Operations (5 tests) - Save/load/delete/list/get_current
//! 5. Command Integration (5 tests) - Tauri state locking, error propagation
//! 6. Concurrent Access (3 tests) - Multiple tasks modifying state
//! 7. Edge Cases (4 tests) - Invalid labels, empty layouts, full state
//!
//! **Architecture:**
//! - WindowManager: Core management logic (Grown-up Script)
//! - Arc<Mutex<WindowManager>>: Async concurrent access pattern
//! - Layout persistence via LayoutStorage (filesystem I/O)
//! - Window state tracking with z-order management
//!
//! **Performance Characteristics:**
//! - Manager operations: O(1) to O(n) depending on operation
//! - Layout persistence: Filesystem I/O with JSON serialization
//! - Concurrent access: Async mutex locking with minimal contention
//! - Focus on correctness over performance (Phase 4 requirement)

use midi_pipeline::windows::{WindowManager, WindowInfo, WindowType, Position, DockSide, Layout};
use std::sync::Arc;
use tokio::sync::Mutex;
use tempfile::TempDir;

// ============================================================================
// TEST FIXTURES & HELPERS
// ============================================================================

/// Create a test window manager without storage
fn create_test_manager() -> WindowManager {
    WindowManager::new()
}

/// Create a test window manager with temporary layout storage
fn create_manager_with_storage() -> (WindowManager, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let manager = WindowManager::with_storage(temp_dir.path().to_path_buf())
        .expect("Failed to create manager with storage");
    (manager, temp_dir)
}

/// Create a basic floating window for testing
fn create_test_window(label: &str) -> WindowInfo {
    WindowInfo::new(label, &format!("{} Window", label), WindowType::Floating)
}

/// Create a dockable window for testing
fn create_dockable_window(label: &str) -> WindowInfo {
    WindowInfo::new(label, &format!("{} Panel", label), WindowType::Dockable)
}

/// Create a main window (non-closeable, persistent)
fn create_main_window() -> WindowInfo {
    WindowInfo::new("main", "Main Window", WindowType::Main)
}

/// Create a test position
fn create_test_position() -> Position {
    Position::new(100, 100, 800, 600)
}

/// Assert error contains expected text
fn assert_error_contains(result: Result<(), String>, expected: &str) {
    match result {
        Ok(_) => panic!("Expected error containing '{}', got Ok", expected),
        Err(e) => assert!(
            e.contains(expected),
            "Expected error containing '{}', got '{}'",
            expected,
            e
        ),
    }
}

// ============================================================================
// CATEGORY 1: MANAGER LIFECYCLE (5 tests)
// ============================================================================

#[tokio::test]
async fn test_manager_creation_empty() {
    let manager = create_test_manager();
    assert_eq!(manager.get_window_count(), 0);
    assert_eq!(manager.get_visible_count(), 0);
    assert!(manager.get_all_windows().is_empty());
}

#[tokio::test]
async fn test_window_registration_single() {
    let mut manager = create_test_manager();
    let window = create_test_window("test1");

    manager.register_window(window);

    assert_eq!(manager.get_window_count(), 1);
    assert!(manager.window_exists("test1"));
    assert_eq!(manager.get_visible_count(), 1); // Windows are visible by default
}

#[tokio::test]
async fn test_window_registration_multiple() {
    let mut manager = create_test_manager();

    manager.register_window(create_test_window("test1"));
    manager.register_window(create_test_window("test2"));
    manager.register_window(create_test_window("test3"));

    assert_eq!(manager.get_window_count(), 3);
    assert!(manager.window_exists("test1"));
    assert!(manager.window_exists("test2"));
    assert!(manager.window_exists("test3"));
}

#[tokio::test]
async fn test_window_unregistration_success() {
    let mut manager = create_test_manager();
    let window = create_test_window("test1");

    manager.register_window(window);
    assert_eq!(manager.get_window_count(), 1);

    let result = manager.unregister_window("test1");
    assert!(result.is_ok());
    assert_eq!(manager.get_window_count(), 0);
    assert!(!manager.window_exists("test1"));
}

#[tokio::test]
async fn test_window_unregistration_not_found() {
    let mut manager = create_test_manager();

    let result = manager.unregister_window("nonexistent");
    assert!(result.is_err());
    assert_error_contains(result, "not found");
}

// ============================================================================
// CATEGORY 2: VISIBILITY COMMANDS (5 tests)
// ============================================================================

#[tokio::test]
async fn test_show_window_success() {
    let mut manager = create_test_manager();
    let mut window = create_test_window("test1");
    window.visible = false; // Start hidden

    manager.register_window(window);
    assert_eq!(manager.get_visible_count(), 0);

    let result = manager.show_window("test1");
    assert!(result.is_ok());
    assert_eq!(manager.get_visible_count(), 1);

    let window_info = manager.get_window("test1").unwrap();
    assert!(window_info.visible);
}

#[tokio::test]
async fn test_hide_window_success() {
    let mut manager = create_test_manager();
    let window = create_test_window("test1");

    manager.register_window(window);
    assert_eq!(manager.get_visible_count(), 1);

    let result = manager.hide_window("test1");
    assert!(result.is_ok());
    assert_eq!(manager.get_visible_count(), 0);

    let window_info = manager.get_window("test1").unwrap();
    assert!(!window_info.visible);
}

#[tokio::test]
async fn test_hide_main_window_protected() {
    let mut manager = create_test_manager();
    let main_window = create_main_window();

    manager.register_window(main_window);

    let result = manager.hide_window("main");
    assert!(result.is_err());
    assert_error_contains(result, "Cannot hide main window");

    // Main window should still be visible
    let window_info = manager.get_window("main").unwrap();
    assert!(window_info.visible);
}

#[tokio::test]
async fn test_toggle_window_show_to_hide() {
    let mut manager = create_test_manager();
    let window = create_test_window("test1");

    manager.register_window(window);
    assert_eq!(manager.get_visible_count(), 1);

    // Toggle visible -> hidden
    let result = manager.toggle_window("test1");
    assert!(result.is_ok());
    assert_eq!(manager.get_visible_count(), 0);
}

#[tokio::test]
async fn test_toggle_window_hide_to_show() {
    let mut manager = create_test_manager();
    let mut window = create_test_window("test1");
    window.visible = false;

    manager.register_window(window);
    assert_eq!(manager.get_visible_count(), 0);

    // Toggle hidden -> visible
    let result = manager.toggle_window("test1");
    assert!(result.is_ok());
    assert_eq!(manager.get_visible_count(), 1);
}

// ============================================================================
// CATEGORY 3: POSITIONING (5 tests)
// ============================================================================

#[tokio::test]
async fn test_set_position_valid() {
    let mut manager = create_test_manager();
    let window = create_test_window("test1");

    manager.register_window(window);

    let position = Position::new(100, 200, 800, 600);
    let result = manager.set_position("test1", position.clone());
    assert!(result.is_ok());

    let retrieved = manager.get_position("test1").unwrap();
    assert_eq!(retrieved.x, 100);
    assert_eq!(retrieved.y, 200);
    assert_eq!(retrieved.width, 800);
    assert_eq!(retrieved.height, 600);
}

#[tokio::test]
async fn test_set_position_invalid() {
    let mut manager = create_test_manager();
    let window = create_test_window("test1");

    manager.register_window(window);

    // Invalid position (zero width)
    let position = Position::new(100, 100, 0, 600);
    let result = manager.set_position("test1", position);
    assert!(result.is_err());
    assert_error_contains(result, "Invalid position");
}

#[tokio::test]
async fn test_dock_window_success() {
    let mut manager = create_test_manager();
    let parent_window = create_test_window("parent");
    let dockable_window = create_dockable_window("child");

    manager.register_window(parent_window);
    manager.register_window(dockable_window);

    let result = manager.dock_window("child", "parent", DockSide::Right);
    assert!(result.is_ok());

    let docked_windows = manager.get_docked_windows("parent");
    assert_eq!(docked_windows.len(), 1);
    assert_eq!(docked_windows[0].label, "child");
}

#[tokio::test]
async fn test_dock_window_parent_not_found() {
    let mut manager = create_test_manager();
    let dockable_window = create_dockable_window("child");

    manager.register_window(dockable_window);

    let result = manager.dock_window("child", "nonexistent_parent", DockSide::Right);
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Parent window"), "Error should contain 'Parent window', got: {}", e);
        assert!(e.contains("not found"), "Error should contain 'not found', got: {}", e);
    }
}

#[tokio::test]
async fn test_undock_window_success() {
    let mut manager = create_test_manager();
    let parent_window = create_test_window("parent");
    let dockable_window = create_dockable_window("child");

    manager.register_window(parent_window);
    manager.register_window(dockable_window);

    // Dock first
    manager.dock_window("child", "parent", DockSide::Right).unwrap();
    assert_eq!(manager.get_docked_windows("parent").len(), 1);

    // Then undock
    let result = manager.undock_window("child");
    assert!(result.is_ok());

    let docked_windows = manager.get_docked_windows("parent");
    assert_eq!(docked_windows.len(), 0);
}

// ============================================================================
// CATEGORY 4: LAYOUT OPERATIONS (5 tests)
// ============================================================================

#[tokio::test]
async fn test_save_layout_success() {
    let (mut manager, _temp_dir) = create_manager_with_storage();

    // Register some windows with positions
    let mut window1 = create_test_window("test1");
    window1.position = Position::new(0, 0, 800, 600);
    manager.register_window(window1);

    let mut window2 = create_test_window("test2");
    window2.position = Position::new(100, 100, 600, 400);
    manager.register_window(window2);

    let result = manager.save_layout("my_layout".to_string());
    assert!(result.is_ok());

    // Verify layout exists in list
    let layouts = manager.list_layouts().unwrap();
    assert!(layouts.contains(&"my_layout".to_string()));
}

#[tokio::test]
async fn test_load_layout_success() {
    let (mut manager, _temp_dir) = create_manager_with_storage();

    // Create and save a layout
    let mut window1 = create_test_window("test1");
    window1.position = Position::new(100, 200, 800, 600);
    manager.register_window(window1);

    manager.save_layout("test_layout".to_string()).unwrap();

    // Modify window position
    manager.set_position("test1", Position::new(999, 999, 100, 100)).unwrap();

    // Load the layout (should restore original position)
    let result = manager.load_layout("test_layout");
    assert!(result.is_ok());

    let position = manager.get_position("test1").unwrap();
    assert_eq!(position.x, 100);
    assert_eq!(position.y, 200);

    // Current layout should be updated
    assert_eq!(manager.get_current_layout(), "test_layout");
}

#[tokio::test]
async fn test_delete_layout_success() {
    let (mut manager, _temp_dir) = create_manager_with_storage();

    // Create and save a layout
    manager.register_window(create_test_window("test1"));
    manager.save_layout("to_delete".to_string()).unwrap();

    // Verify it exists
    let layouts_before = manager.list_layouts().unwrap();
    assert!(layouts_before.contains(&"to_delete".to_string()));

    // Delete it
    let result = manager.delete_layout("to_delete");
    assert!(result.is_ok());

    // Verify it's gone
    let layouts_after = manager.list_layouts().unwrap();
    assert!(!layouts_after.contains(&"to_delete".to_string()));
}

#[tokio::test]
async fn test_list_layouts_empty() {
    let (_manager, _temp_dir) = create_manager_with_storage();

    let layouts = _manager.list_layouts().unwrap();
    assert!(layouts.is_empty());
}

#[tokio::test]
async fn test_get_current_layout_default() {
    let manager = create_test_manager();

    let current = manager.get_current_layout();
    assert_eq!(current, "default");
}

// ============================================================================
// CATEGORY 5: COMMAND INTEGRATION (5 tests)
// Test Arc<Mutex<WindowManager>> pattern used by Tauri commands
// ============================================================================

#[tokio::test]
async fn test_command_pattern_show_window() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    // Register window
    {
        let mut mgr = manager.lock().await;
        let mut window = create_test_window("test1");
        window.visible = false;
        mgr.register_window(window);
    }

    // Simulate Tauri command pattern
    {
        let mut mgr = manager.lock().await;
        let result = mgr.show_window("test1");
        assert!(result.is_ok());
    }

    // Verify result
    {
        let mgr = manager.lock().await;
        assert_eq!(mgr.get_visible_count(), 1);
    }
}

#[tokio::test]
async fn test_command_pattern_error_propagation() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    // Try to show non-existent window
    let result = {
        let mut mgr = manager.lock().await;
        mgr.show_window("nonexistent")
    };

    assert!(result.is_err());
    assert_error_contains(result, "not found");
}

#[tokio::test]
async fn test_command_pattern_get_window_count() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    // Register windows
    {
        let mut mgr = manager.lock().await;
        mgr.register_window(create_test_window("test1"));
        mgr.register_window(create_test_window("test2"));
        mgr.register_window(create_test_window("test3"));
    }

    // Get count
    let count = {
        let mgr = manager.lock().await;
        mgr.get_window_count()
    };

    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_command_pattern_set_focused() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    // Register and focus window
    {
        let mut mgr = manager.lock().await;
        mgr.register_window(create_test_window("test1"));
        mgr.set_focused(Some("test1"));
    }

    // Get focused window
    let focused = {
        let mgr = manager.lock().await;
        mgr.get_focused()
    };

    assert!(focused.is_some());
    assert_eq!(focused.unwrap().label, "test1");
}

#[tokio::test]
async fn test_command_pattern_layout_save_without_storage() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    // Try to save layout without storage configured
    let result = {
        let mut mgr = manager.lock().await;
        mgr.save_layout("test_layout".to_string())
    };

    assert!(result.is_err());
    assert_error_contains(result, "Layout storage not configured");
}

// ============================================================================
// CATEGORY 6: CONCURRENT ACCESS (3 tests)
// Test thread-safety with Arc<Mutex>
// ============================================================================

#[tokio::test]
async fn test_concurrent_window_registration() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    let mut handles = vec![];

    // Spawn 10 tasks to register windows concurrently
    for i in 0..10 {
        let mgr_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mut mgr = mgr_clone.lock().await;
            let window = create_test_window(&format!("window_{}", i));
            mgr.register_window(window);
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all windows were registered
    let mgr = manager.lock().await;
    assert_eq!(mgr.get_window_count(), 10);
}

#[tokio::test]
async fn test_concurrent_visibility_toggle() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    // Register windows first
    {
        let mut mgr = manager.lock().await;
        for i in 0..5 {
            mgr.register_window(create_test_window(&format!("window_{}", i)));
        }
    }

    let mut handles = vec![];

    // Toggle visibility concurrently
    for i in 0..5 {
        let mgr_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mut mgr = mgr_clone.lock().await;
            let _ = mgr.toggle_window(&format!("window_{}", i));
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // All windows should now be hidden (toggled from visible to hidden)
    let mgr = manager.lock().await;
    assert_eq!(mgr.get_visible_count(), 0);
}

#[tokio::test]
async fn test_concurrent_position_updates() {
    let manager = Arc::new(Mutex::new(create_test_manager()));

    // Register windows
    {
        let mut mgr = manager.lock().await;
        for i in 0..5 {
            mgr.register_window(create_test_window(&format!("window_{}", i)));
        }
    }

    let mut handles = vec![];

    // Update positions concurrently
    for i in 0..5 {
        let mgr_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let mut mgr = mgr_clone.lock().await;
            let position = Position::new(i * 100, i * 100, 800, 600);
            let _ = mgr.set_position(&format!("window_{}", i), position);
        });
        handles.push(handle);
    }

    // Wait for all updates
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify positions were updated
    let mgr = manager.lock().await;
    for i in 0..5 {
        let position = mgr.get_position(&format!("window_{}", i)).unwrap();
        assert_eq!(position.x, i * 100);
        assert_eq!(position.y, i * 100);
    }
}

// ============================================================================
// CATEGORY 7: EDGE CASES (4 tests)
// ============================================================================

#[tokio::test]
async fn test_invalid_window_label_operations() {
    let mut manager = create_test_manager();

    // Try operations on non-existent window
    assert!(manager.show_window("nonexistent").is_err());
    assert!(manager.hide_window("nonexistent").is_err());
    assert!(manager.toggle_window("nonexistent").is_err());
    assert!(manager.set_position("nonexistent", create_test_position()).is_err());
    assert!(manager.undock_window("nonexistent").is_err());

    // get_window should return None
    assert!(manager.get_window("nonexistent").is_none());
    assert!(manager.get_position("nonexistent").is_none());
}

#[tokio::test]
async fn test_empty_layout_operations() {
    let (mut manager, _temp_dir) = create_manager_with_storage();

    // Save empty layout
    let result = manager.save_layout("empty_layout".to_string());
    assert!(result.is_ok());

    // Load empty layout
    let result = manager.load_layout("empty_layout");
    assert!(result.is_ok());

    // Delete empty layout
    let result = manager.delete_layout("empty_layout");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_window_z_order_tracking() {
    let mut manager = create_test_manager();

    // Register windows
    manager.register_window(create_test_window("window1"));
    manager.register_window(create_test_window("window2"));
    manager.register_window(create_test_window("window3"));

    // Initial order should match registration order
    let order = manager.get_window_order();
    assert_eq!(order, vec!["window1", "window2", "window3"]);

    // Bring window1 to front
    manager.bring_to_front("window1");

    let order = manager.get_window_order();
    assert_eq!(order, vec!["window2", "window3", "window1"]);

    // Show operation brings window to front
    manager.show_window("window2").unwrap();

    let order = manager.get_window_order();
    assert_eq!(order, vec!["window3", "window1", "window2"]);
}

#[tokio::test]
async fn test_complete_state_save_and_restore() {
    let mut manager = create_test_manager();

    // Register multiple windows with different states
    let mut window1 = create_test_window("test1");
    window1.position = Position::new(0, 0, 800, 600);
    window1.visible = true;
    manager.register_window(window1);

    let mut window2 = create_test_window("test2");
    window2.position = Position::new(100, 100, 600, 400);
    window2.visible = false;
    manager.register_window(window2);

    manager.set_focused(Some("test1"));

    // Get complete state
    let saved_state = manager.get_state();

    // Create new manager and restore state
    let mut new_manager = create_test_manager();
    new_manager.restore_state(saved_state);

    // Verify all state was restored
    assert_eq!(new_manager.get_window_count(), 2);
    assert_eq!(new_manager.get_visible_count(), 1);
    assert!(new_manager.window_exists("test1"));
    assert!(new_manager.window_exists("test2"));

    let focused = new_manager.get_focused().unwrap();
    assert_eq!(focused.label, "test1");

    let window1 = new_manager.get_window("test1").unwrap();
    assert!(window1.visible);
    assert_eq!(window1.position.width, 800);

    let window2 = new_manager.get_window("test2").unwrap();
    assert!(!window2.visible);
    assert_eq!(window2.position.width, 600);
}
