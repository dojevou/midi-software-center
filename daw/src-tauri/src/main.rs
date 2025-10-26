// daw/src-tauri/src/main.rs
// Task-O-Matic: Main entry point for DAW application
// Purpose: Initialize app, register commands, manage state with MIDI hardware

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod models;
mod core;
mod midi;
mod sequencer;

use commands::AppState;
use midi::MidiManager;
use sequencer::SequencerEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize tracing/logging
    init_logging();

    info!("Starting MIDI DAW application");

    // Initialize database connection pool
    let db_pool = match initialize_database_pool().await {
        Ok(pool) => {
            info!("✅ Database connection pool initialized");
            Some(pool)
        }
        Err(e) => {
            warn!("⚠️  Database connection failed: {}", e);
            warn!("⚠️  DAW will run without database features (search, analysis, etc.)");
            None
        }
    };

    // Initialize MIDI manager (no database required for DAW startup)
    let midi_manager = Arc::new(MidiManager::new());
    info!("MIDI manager initialized");

    // Initialize sequencer engine
    let sequencer_engine = Arc::new(SequencerEngine::new(
        midi_manager.clone(),
        120.0, // Default 120 BPM
        480,   // Standard MIDI resolution
    ));
    info!("Sequencer engine initialized");

    // Create application state
    let state = AppState {
        db_pool,
    };

    // Build and run Tauri application
    tauri::Builder::default()
        .manage(state)
        .manage(midi_manager)
        .manage(sequencer_engine)
        .invoke_handler(tauri::generate_handler![
            // Database commands
            commands::initialize_database,
            // MIDI commands
            commands::midi::midi_list_devices,
            commands::midi::midi_connect,
            commands::midi::midi_disconnect,
            commands::midi::midi_is_connected,
            commands::midi::midi_get_current_device,
            commands::midi::midi_send_test_note,
            // Sequencer commands
            commands::sequencer::start_sequencer,
            commands::sequencer::stop_sequencer,
            commands::sequencer::pause_sequencer,
            commands::sequencer::resume_sequencer,
            commands::sequencer::get_playback_position,
            commands::sequencer::seek_position,
            commands::sequencer::set_tempo,
            commands::sequencer::get_tempo,
            commands::sequencer::add_track,
            commands::sequencer::remove_track,
            commands::sequencer::update_track,
            commands::sequencer::get_tracks,
            commands::sequencer::load_sequencer_tracks,
            commands::sequencer::is_sequencer_playing,
            // Search commands
            commands::search::search_files,
            commands::search::get_file_details,
            commands::search::get_search_suggestions,
            // Analysis commands
            commands::analysis::find_compatible_files,
            commands::analysis::add_favorite,
            commands::analysis::remove_favorite,
            commands::analysis::is_favorite,
            commands::analysis::get_favorites,
            commands::analysis::get_usage_stats,
            // Project commands
            commands::project::load_multiple_tracks,
            commands::project::clear_all_tracks,
            commands::project::get_track_details,
            // Export commands
            commands::export::export_project_midi,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize logging/tracing system
fn init_logging() {
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string());
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(log_dir, "daw.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,midi_daw=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();
}

/// Initialize PostgreSQL database connection pool
///
/// Reads DATABASE_URL from environment and creates a connection pool.
/// Returns an error if DATABASE_URL is not set or connection fails.
async fn initialize_database_pool() -> Result<sqlx::PgPool, String> {
    // Get DATABASE_URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| {
            "DATABASE_URL not set. Please set it to: postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
        })?;

    info!("Connecting to database: {}", database_url.replace(":145278963", ":****"));

    // Get max connections from environment or use default
    let max_connections: u32 = std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // Create connection pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Test connection with a simple query
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to execute test query: {}", e))?;

    info!("Database connection pool created with {} max connections", max_connections);

    Ok(pool)
}
