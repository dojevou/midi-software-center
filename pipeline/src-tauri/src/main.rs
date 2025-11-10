// pipeline/src-tauri/src/main.rs
// Task-O-Matic: Main entry point for Pipeline application
// Purpose: Initialize app, register commands, manage state

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import from lib
use midi_pipeline::{AppState, Database};

// Window management module
mod windows;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();

    // Initialize tracing/logging
    init_logging();

    info!("Starting MIDI Pipeline application");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    // Initialize database connection
    let database = match Database::new(&database_url).await {
        Ok(db) => {
            info!("Database connection established");
            db
        },
        Err(e) => {
            info!(
                "Database initialization deferred (will retry on first command): {}",
                e
            );
            // Retry once
            Database::new(&database_url).await.map_err(|retry_err| {
                format!(
                    "Failed to create database instance after retry: {}",
                    retry_err
                )
            })?
        },
    };

    // Create application state
    let state = AppState { database };

    // Create window manager
    let window_manager = Arc::new(Mutex::new(windows::WindowManager::new()));

    // Build and run Tauri application
    tauri::Builder::default()
        .manage(state)
        .manage(window_manager)
        .invoke_handler(tauri::generate_handler![
            // File commands
            midi_pipeline::commands::files::test_db_connection,
            midi_pipeline::commands::files::get_file_count,
            midi_pipeline::commands::files::get_file_details,
            midi_pipeline::commands::files::get_file,
            midi_pipeline::commands::files::list_files,
            midi_pipeline::commands::files::get_files_by_category,
            midi_pipeline::commands::files::get_recent_files,
            midi_pipeline::commands::files::delete_file,
            // Import commands
            midi_pipeline::commands::file_import::import_single_file,
            midi_pipeline::commands::file_import::import_directory,
            midi_pipeline::commands::archive_import::import_archive_collection,
            // Search commands
            midi_pipeline::commands::search::search_files,
            midi_pipeline::commands::search::get_all_tags,
            midi_pipeline::commands::search::get_files_by_tag,
            midi_pipeline::commands::search::get_bpm_range,
            midi_pipeline::commands::search::get_all_keys,
            // Analysis commands
            midi_pipeline::commands::analyze::start_analysis,
            // Statistics commands
            midi_pipeline::commands::stats::get_category_stats,
            midi_pipeline::commands::stats::get_manufacturer_stats,
            midi_pipeline::commands::stats::get_key_signature_stats,
            midi_pipeline::commands::stats::get_recently_added_count,
            midi_pipeline::commands::stats::get_duplicate_count,
            midi_pipeline::commands::stats::get_database_size,
            midi_pipeline::commands::stats::check_database_health,
            // Tag commands
            midi_pipeline::commands::tags::get_file_tags,
            midi_pipeline::commands::tags::get_popular_tags,
            midi_pipeline::commands::tags::search_tags,
            midi_pipeline::commands::tags::get_tag_categories,
            midi_pipeline::commands::tags::get_tags_by_category,
            midi_pipeline::commands::tags::update_file_tags,
            midi_pipeline::commands::tags::add_tags_to_file,
            midi_pipeline::commands::tags::remove_tag_from_file,
            midi_pipeline::commands::tags::get_files_by_tags,
            midi_pipeline::commands::tags::get_tag_stats,
            // Progress tracking commands
            midi_pipeline::commands::progress::start_progress_tracking,
            midi_pipeline::commands::progress::update_progress,
            midi_pipeline::commands::progress::increment_error_count,
            midi_pipeline::commands::progress::increment_duplicate_count,
            midi_pipeline::commands::progress::complete_progress,
            midi_pipeline::commands::progress::get_current_progress,
            midi_pipeline::commands::progress::reset_progress,
            // System commands
            midi_pipeline::commands::system::get_system_info,
            // Window management commands
            windows::commands::show_window,
            windows::commands::hide_window,
            windows::commands::toggle_window,
            windows::commands::save_layout,
            windows::commands::load_layout,
            windows::commands::get_layout_list,
            windows::commands::delete_layout,
            windows::commands::arrange_windows,
            windows::commands::get_all_windows,
            windows::commands::get_visible_windows,
            windows::commands::get_window_count,
            windows::commands::get_focused_window,
            windows::commands::set_focused_window,
            windows::commands::get_current_layout,
        ])
        .setup(|_app| {
            info!("Application setup complete");
            // TODO: Setup window shortcuts (disabled until Tauri 2.x API compatibility fixed)
            // windows::shortcuts::setup_window_shortcuts(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}

/// Initialize logging/tracing system
fn init_logging() {
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string());
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(log_dir, "pipeline.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,midi_pipeline=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        // Test that logging initialization doesn't panic
        init_logging();
    }
}
