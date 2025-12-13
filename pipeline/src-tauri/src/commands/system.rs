use crate::AppState;
/// System command handlers - GROWN-UP SCRIPT ARCHETYPE
///
/// PURPOSE: System-level operations and information
/// ARCHETYPE: Grown-up Script (I/O operations)
///
/// ✅ CAN: Perform system I/O
/// ✅ CAN: Have side effects
/// ✅ SHOULD: Handle errors properly
/// ❌ NO: Complex business logic
use serde::Serialize;
use tauri::State;

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// System information response
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub version: String,
    pub platform: String,
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Get system information
///
/// Returns the application version and platform information.
///
/// # Frontend Usage
///
/// ```typescript
/// const info = await invoke<{version: string, platform: string}>('get_system_info');
/// // { version: "0.1.0", platform: "linux" }
/// ```
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    Ok(SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
    })
}

/// Initialize database connection on first use
///
/// This allows Tauri to start up without blocking, then connects
/// to database on first command. If already connected, returns ok.
///
/// # Frontend Usage
///
/// ```typescript
/// await invoke('initialize_database');
/// // Now all database commands will work
/// ```
#[tauri::command]
pub async fn initialize_database(_state: State<'_, AppState>) -> Result<(), String> {
    // Database is eagerly initialized in main.rs, so this is a no-op
    // This command exists for completeness if we switch to lazy initialization
    Ok(())
}
