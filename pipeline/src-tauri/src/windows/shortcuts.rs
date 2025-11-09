   /// Global keyboard shortcuts for window management

use tauri::{AppHandle, GlobalShortcutManager};

/// Setup window management global shortcuts
///
/// Shortcuts:
/// - Cmd/Ctrl+1: Show Pipeline
/// - Cmd/Ctrl+2: Show DAW
/// - Cmd/Ctrl+3: Show Database
/// - Cmd/Ctrl+`: Cycle windows (next)
/// - Cmd/Ctrl+Shift+`: Cycle windows (prev)
pub fn setup_window_shortcuts(app: &AppHandle) -> Result<(), String> {
    let mut manager = app.global_shortcut_manager();

    // Show Pipeline (Cmd/Ctrl+1)
    {
        let app_handle = app.clone();
        manager
            .register("CmdOrCtrl+1", move || {
                let _ = app_handle.emit_all("command:show-pipeline", ());
            })
            .map_err(|e| format!("Failed to register Cmd+1: {}", e))?;
    }

    // Show DAW (Cmd/Ctrl+2)
    {
        let app_handle = app.clone();
        manager
            .register("CmdOrCtrl+2", move || {
                let _ = app_handle.emit_all("command:show-daw", ());
            })
            .map_err(|e| format!("Failed to register Cmd+2: {}", e))?;
    }

    // Show Database (Cmd/Ctrl+3)
    {
        let app_handle = app.clone();
        manager
            .register("CmdOrCtrl+3", move || {
                let _ = app_handle.emit_all("command:show-database", ());
            })
            .map_err(|e| format!("Failed to register Cmd+3: {}", e))?;
    }

    // Cycle Windows Forward (Cmd/Ctrl+`)
    {
        let app_handle = app.clone();
        manager
            .register("CmdOrCtrl+grave", move || {
                let _ = app_handle.emit_all("command:cycle-windows", ());
            })
            .map_err(|e| format!("Failed to register Cmd+`: {}", e))?;
    }

    // Cycle Windows Backward (Cmd/Ctrl+Shift+`)
    {
        let app_handle = app.clone();
        manager
            .register("CmdOrCtrl+Shift+grave", move || {
                let _ = app_handle.emit_all("command:cycle-windows-back", ());
            })
            .map_err(|e| format!("Failed to register Cmd+Shift+`: {}", e))?;
    }

    // Toggle Sidebar (Cmd/Ctrl+B)
    {
        let app_handle = app.clone();
        manager
            .register("CmdOrCtrl+b", move || {
                let _ = app_handle.emit("command:toggle-sidebar", ());
            })
            .map_err(|e| format!("Failed to register Cmd+B: {}", e))?;
    }

    // Toggle Inspector (Cmd/Ctrl+Opt/Alt+I)
    {
        let app_handle = app.clone();
        manager
            .register("CmdOrCtrl+Alt+i", move || {
                let _ = app_handle.emit("command:toggle-inspector", ());
            })
            .map_err(|e| format!("Failed to register Cmd+Alt+I: {}", e))?;
    }

    info!("✅ Window shortcuts registered");
    Ok(())
}

// Note: Using 'info!' requires tracing to be in scope
// Add to main.rs: use tracing::info;
