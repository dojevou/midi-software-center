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
use midi_pipeline::commands::health::HealthState;
use midi_library_shared::health::HealthChecker;

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

    // Create health checker
    let meilisearch_url = std::env::var("MEILISEARCH_URL")
        .unwrap_or_else(|_| "http://localhost:7700".to_string());
    let meilisearch_key = std::env::var("MEILISEARCH_KEY").ok();

    let health_checker = HealthChecker::new()
        .with_postgres(state.database.pool().await)
        .with_meilisearch(meilisearch_url, meilisearch_key);

    let health_state = HealthState::new(health_checker);
    info!("Health checker initialized");

    // Create window manager
    let window_manager = Arc::new(Mutex::new(windows::WindowManager::new()));

    // Build and run Tauri application
    tauri::Builder::default()
        .manage(state)
        .manage(health_state)
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
            midi_pipeline::commands::search::pipeline_search_files,
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
            // Rating commands
            midi_pipeline::commands::tags::set_file_rating,
            midi_pipeline::commands::tags::get_file_rating,
            midi_pipeline::commands::tags::get_files_by_rating,
            // VIP3 Browser commands
            midi_pipeline::commands::vip3::search_files_vip3,
            midi_pipeline::commands::vip3::get_vip3_filter_counts,
            // VIP3 Favorites commands
            midi_pipeline::commands::vip3::toggle_favorite,
            midi_pipeline::commands::vip3::set_favorite,
            midi_pipeline::commands::vip3::get_favorites,
            midi_pipeline::commands::vip3::get_favorite_count,
            // VIP3 Saved Search commands
            midi_pipeline::commands::vip3::save_search,
            midi_pipeline::commands::vip3::get_saved_searches,
            midi_pipeline::commands::vip3::load_saved_search,
            midi_pipeline::commands::vip3::delete_saved_search,
            midi_pipeline::commands::vip3::toggle_saved_search_pin,
            // VIP3 Collection commands
            midi_pipeline::commands::vip3::create_collection,
            midi_pipeline::commands::vip3::get_collections,
            midi_pipeline::commands::vip3::get_collection,
            midi_pipeline::commands::vip3::add_file_to_collection,
            midi_pipeline::commands::vip3::remove_file_from_collection,
            midi_pipeline::commands::vip3::get_collection_files,
            midi_pipeline::commands::vip3::delete_collection,
            midi_pipeline::commands::vip3::update_collection,
            // VIP3 Category Assignment commands
            midi_pipeline::commands::vip3::add_timbre_to_file,
            midi_pipeline::commands::vip3::remove_timbre_from_file,
            midi_pipeline::commands::vip3::add_style_to_file,
            midi_pipeline::commands::vip3::remove_style_from_file,
            midi_pipeline::commands::vip3::add_articulation_to_file,
            midi_pipeline::commands::vip3::remove_articulation_from_file,
            midi_pipeline::commands::vip3::get_file_categories,
            // VIP3 Lookup commands
            midi_pipeline::commands::vip3::get_all_timbres,
            midi_pipeline::commands::vip3::get_all_styles,
            midi_pipeline::commands::vip3::get_all_articulations,
            midi_pipeline::commands::vip3::get_all_bpm_ranges,
            midi_pipeline::commands::vip3::get_all_musical_keys,
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
            // Health check commands
            midi_pipeline::commands::health::check_system_health,
            midi_pipeline::commands::health::get_cached_health,
            midi_pipeline::commands::health::check_postgres_health,
            midi_pipeline::commands::health::check_meilisearch_health,
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
