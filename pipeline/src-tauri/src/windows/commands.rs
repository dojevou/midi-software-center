   /// Tauri command handlers for window management
   ///
   /// These commands expose window management functionality to the frontend.

use tauri::command;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::windows::manager::WindowManager;

/// Show a window by label
#[command]
pub async fn show_window(
    label: String,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;
    mgr.show_window(&label)
}

/// Hide a window by label
#[command]
pub async fn hide_window(
    label: String,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;
    mgr.hide_window(&label)
}

/// Toggle window visibility
#[command]
pub async fn toggle_window(
    label: String,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;
    mgr.toggle_window(&label)
}

/// Save current window layout
#[command]
pub async fn save_layout(
    name: String,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;
    mgr.save_layout(name)
}

/// Load a saved window layout
#[command]
pub async fn load_layout(
    name: String,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;
    mgr.load_layout(&name)
}

/// Get list of available layouts
#[command]
pub async fn get_layout_list(
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<Vec<String>, String> {
    let mgr = manager.lock().await;
    mgr.list_layouts()
}

/// Delete a layout
#[command]
pub async fn delete_layout(
    name: String,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;
    mgr.delete_layout(&name)
}

/// Arrange windows (tile_h, tile_v, cascade)
#[command]
pub async fn arrange_windows(
    arrangement: String,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;

    match arrangement.as_str() {
        "tile_h" => mgr.tile_windows_horizontal(),
        "tile_v" => mgr.tile_windows_vertical(),
        "cascade" => mgr.cascade_windows(),
        _ => Err(format!("Unknown arrangement: {}", arrangement)),
    }
}

/// Get all windows
#[command]
pub async fn get_all_windows(
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<Vec<crate::windows::state::WindowInfo>, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_all_windows())
}

/// Get visible windows only
#[command]
pub async fn get_visible_windows(
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<Vec<crate::windows::state::WindowInfo>, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_visible_windows())
}

/// Get window count
#[command]
pub async fn get_window_count(
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<usize, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_window_count())
}

/// Get focused window
#[command]
pub async fn get_focused_window(
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<Option<crate::windows::state::WindowInfo>, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_focused())
}

/// Set focused window
#[command]
pub async fn set_focused_window(
    label: Option<String>,
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<(), String> {
    let mut mgr = manager.lock().await;
    mgr.set_focused(label.as_deref());
    Ok(())
}

/// Get current layout name
#[command]
pub async fn get_current_layout(
    manager: tauri::State<'_, Arc<Mutex<WindowManager>>>,
) -> Result<String, String> {
    let mgr = manager.lock().await;
    Ok(mgr.get_current_layout())
}
