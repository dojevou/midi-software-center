//! Tauri commands for log access and management.

use midi_library_shared::logging::{metrics::get_metrics, LogEvent, PerformanceMetrics};
use std::path::PathBuf;
use tauri::Emitter;
use tokio::fs;
use tokio::io::AsyncBufReadExt;

/// Get recent log entries from the log file.
#[tauri::command]
pub async fn get_recent_logs(
    limit: Option<usize>,
    level: Option<String>,
    category: Option<String>,
) -> Result<Vec<LogEvent>, String> {
    let limit = limit.unwrap_or(100);
    let log_path = get_log_path()?;

    let file = fs::File::open(&log_path)
        .await
        .map_err(|e| format!("Failed to open log file: {}", e))?;

    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();
    let mut events: Vec<LogEvent> = Vec::new();

    while let Some(line) =
        lines.next_line().await.map_err(|e| format!("Failed to read log: {}", e))?
    {
        if let Ok(event) = serde_json::from_str::<LogEvent>(&line) {
            // Filter by level if specified
            if let Some(ref lvl) = level {
                if event.level.to_lowercase() != lvl.to_lowercase() {
                    continue;
                }
            }

            // Filter by category if specified
            if let Some(ref cat) = category {
                if event.category.to_string() != *cat {
                    continue;
                }
            }

            events.push(event);
        }
    }

    // Return last N entries
    let start = events.len().saturating_sub(limit);
    Ok(events[start..].to_vec())
}

/// Get performance metrics.
#[tauri::command]
pub async fn get_performance_metrics() -> Result<PerformanceMetrics, String> {
    Ok(get_metrics())
}

/// Clear log file.
#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    let log_path = get_log_path()?;
    fs::write(&log_path, "")
        .await
        .map_err(|e| format!("Failed to clear logs: {}", e))?;
    tracing::info!("Log file cleared");
    Ok(())
}

/// Export logs to a file.
#[tauri::command]
pub async fn export_logs(output_path: String) -> Result<(), String> {
    let log_path = get_log_path()?;
    fs::copy(&log_path, &output_path)
        .await
        .map_err(|e| format!("Failed to export logs: {}", e))?;
    tracing::info!(path = %output_path, "Logs exported");
    Ok(())
}

/// Get log file path.
fn get_log_path() -> Result<PathBuf, String> {
    let log_dir = dirs::data_local_dir()
        .ok_or_else(|| "Could not determine data directory".to_string())?
        .join("midi-software-center")
        .join("logs");

    Ok(log_dir.join("midi-software-center.log"))
}

/// Stream log events to the frontend via Tauri events.
#[tauri::command]
pub async fn subscribe_to_logs(app: tauri::AppHandle) -> Result<(), String> {
    // This would typically use a channel or subscriber to forward logs
    // For now, we'll just acknowledge the subscription
    app.emit("log-subscribed", ()).map_err(|e| format!("Failed to emit: {}", e))?;

    Ok(())
}
