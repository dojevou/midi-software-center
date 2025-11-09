   /// Menu creation for window management

use tauri::{Menu, Submenu, MenuItem, CustomMenuItem};

/// Create the Windows menu (for menu bar)
pub fn create_windows_menu() -> Submenu {
    Submenu::new(
        "Windows",
        Menu::new()
            .add_item(CustomMenuItem::new("show_pipeline", "Show Pipeline"))
            .add_item(CustomMenuItem::new("show_daw", "Show DAW"))
            .add_item(CustomMenuItem::new("show_database", "Show Database"))
            .add_item(CustomMenuItem::new("show_settings", "Show Settings"))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("arrange_tile_h", "Tile Horizontally"))
            .add_item(CustomMenuItem::new("arrange_tile_v", "Tile Vertically"))
            .add_item(CustomMenuItem::new("arrange_cascade", "Cascade"))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("save_layout", "Save Layout..."))
            .add_item(CustomMenuItem::new("load_layout", "Load Layout..."))
            .add_item(CustomMenuItem::new("manage_layouts", "Manage Layouts...")),
    )
}

/// Create View menu with window controls
pub fn create_view_menu() -> Submenu {
    Submenu::new(
        "View",
        Menu::new()
            .add_item(CustomMenuItem::new("toggle_sidebar", "Toggle Sidebar"))
            .add_item(CustomMenuItem::new("toggle_inspector", "Toggle Inspector"))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("zoom_in", "Zoom In"))
            .add_item(CustomMenuItem::new("zoom_out", "Zoom Out"))
            .add_item(CustomMenuItem::new("zoom_reset", "Reset Zoom")),
    )
}
