//! System Commands - System-level operations and information

use crate::AppState;
use serde::Serialize;
use tauri::State;

/// System information response.
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub version: String,
    pub platform: String,
}

/// Get system information (version and platform).
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    Ok(SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
    })
}

/// Initialize database connection (no-op, database is eagerly initialized).
#[tauri::command]
pub async fn initialize_database(_state: State<'_, AppState>) -> Result<(), String> {
    Ok(())
}
