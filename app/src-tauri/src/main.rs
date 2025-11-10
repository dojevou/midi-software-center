// app/src-tauri/src/main.rs
// Unified MIDI Software Center Application
// Combines Pipeline (batch processing) and DAW (real-time playback)

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import Pipeline types
use midi_pipeline::{AppState as PipelineState, Database};

// Import DAW types
use daw_lib::commands::{AutomationState, DAWState};
use daw_lib::midi::MidiManager;
use daw_lib::sequencer::SequencerEngine;

/// Combined application state
#[allow(dead_code)]
struct AppState {
    pipeline: PipelineState,
    db_pool: Option<sqlx::PgPool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();

    // Initialize tracing/logging
    init_logging();

    info!("Starting MIDI Software Center (Unified App)");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    // Initialize Pipeline database connection
    let database = match Database::new(&database_url).await {
        Ok(db) => {
            info!("✅ Pipeline database connection established");
            db
        },
        Err(e) => {
            info!(
                "⚠️  Database initialization deferred (will retry on first command): {}",
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

    // Initialize DAW database connection pool (for DAW-specific features)
    let db_pool = match initialize_database_pool(&database_url).await {
        Ok(pool) => {
            info!("✅ DAW database connection pool initialized");
            Some(pool)
        },
        Err(e) => {
            warn!("⚠️  DAW database pool failed: {}", e);
            warn!("⚠️  DAW will run without database features (search, analysis, etc.)");
            None
        },
    };

    // Create Pipeline state
    let pipeline_state = PipelineState { database };

    // Create combined app state
    let state = AppState { pipeline: pipeline_state, db_pool };

    // Initialize MIDI manager (DAW)
    let midi_manager = Arc::new(MidiManager::new());
    info!("✅ MIDI manager initialized");

    // Initialize sequencer engine (DAW)
    let sequencer_engine = Arc::new(SequencerEngine::new(
        midi_manager.clone(),
        120.0, // Default 120 BPM
        480,   // Standard MIDI resolution
    ));
    info!("✅ Sequencer engine initialized");

    // Create DAW window state
    let daw_state = DAWState::new();

    // Create automation state
    let automation_state = AutomationState::new();

    // Build and run Tauri application with ALL commands from both apps
    tauri::Builder::default()
        .manage(state)
        .manage(midi_manager)
        .manage(sequencer_engine)
        .manage(daw_state)
        .manage(automation_state)
        .invoke_handler(tauri::generate_handler![
            // ========================================================================
            // PIPELINE COMMANDS (File Management, Import, Search, Analysis)
            // ========================================================================

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
            // Search commands (Pipeline)
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
            // ========================================================================
            // DAW COMMANDS (Sequencer, MIDI, Playback, Automation)
            // ========================================================================

            // Database commands (DAW)
            daw_lib::commands::initialize_database,
            // MIDI commands
            daw_lib::commands::midi::midi_list_devices,
            daw_lib::commands::midi::midi_connect,
            daw_lib::commands::midi::midi_disconnect,
            daw_lib::commands::midi::midi_is_connected,
            daw_lib::commands::midi::midi_get_current_device,
            daw_lib::commands::midi::midi_send_test_note,
            // Sequencer commands
            daw_lib::commands::sequencer::start_sequencer,
            daw_lib::commands::sequencer::stop_sequencer,
            daw_lib::commands::sequencer::pause_sequencer,
            daw_lib::commands::sequencer::resume_sequencer,
            daw_lib::commands::sequencer::get_playback_position,
            daw_lib::commands::sequencer::seek_position,
            daw_lib::commands::sequencer::set_tempo,
            daw_lib::commands::sequencer::get_tempo,
            daw_lib::commands::sequencer::add_track,
            daw_lib::commands::sequencer::remove_track,
            daw_lib::commands::sequencer::update_track,
            daw_lib::commands::sequencer::get_tracks,
            daw_lib::commands::sequencer::load_sequencer_tracks,
            daw_lib::commands::sequencer::is_sequencer_playing,
            // Search commands (DAW)
            daw_lib::commands::search::search_files,
            daw_lib::commands::search::get_file_details,
            daw_lib::commands::search::get_search_suggestions,
            // Analysis commands (DAW)
            daw_lib::commands::analysis::find_compatible_files,
            daw_lib::commands::analysis::add_favorite,
            daw_lib::commands::analysis::remove_favorite,
            daw_lib::commands::analysis::is_favorite,
            daw_lib::commands::analysis::get_favorites,
            daw_lib::commands::analysis::get_usage_stats,
            // Project commands
            daw_lib::commands::project::load_multiple_tracks,
            daw_lib::commands::project::clear_all_tracks,
            daw_lib::commands::project::get_track_details,
            // Export commands
            daw_lib::commands::export::export_project_midi,
            // Window commands (DAW)
            daw_lib::commands::window::play_transport,
            daw_lib::commands::window::stop_transport,
            daw_lib::commands::window::pause_transport,
            daw_lib::commands::window::set_playback_position,
            daw_lib::commands::window::get_playback_state,
            daw_lib::commands::window::set_bpm,
            daw_lib::commands::window::get_bpm,
            daw_lib::commands::window::set_time_signature,
            daw_lib::commands::window::get_time_signature,
            daw_lib::commands::window::set_key_signature,
            daw_lib::commands::window::get_key_signature,
            daw_lib::commands::window::add_window_track,
            daw_lib::commands::window::remove_window_track,
            daw_lib::commands::window::get_all_window_tracks,
            daw_lib::commands::window::set_track_visible,
            daw_lib::commands::window::set_track_muted,
            daw_lib::commands::window::set_track_soloed,
            daw_lib::commands::window::get_track_info,
            daw_lib::commands::window::update_track_label,
            daw_lib::commands::window::get_mixer_state,
            daw_lib::commands::window::set_channel_volume,
            daw_lib::commands::window::set_channel_pan,
            daw_lib::commands::window::set_channel_mute,
            daw_lib::commands::window::set_channel_solo,
            daw_lib::commands::window::get_daw_state,
            daw_lib::commands::window::reset_daw_state,
            // Automation commands
            daw_lib::commands::automation::create_automation_lane,
            daw_lib::commands::automation::delete_automation_lane,
            daw_lib::commands::automation::add_automation_point,
            daw_lib::commands::automation::remove_automation_point,
            daw_lib::commands::automation::move_automation_point,
            daw_lib::commands::automation::set_automation_curve_type,
            daw_lib::commands::automation::get_automation_lane,
            daw_lib::commands::automation::get_track_automation,
            daw_lib::commands::automation::get_automation_value,
            daw_lib::commands::automation::clear_track_automation,
            daw_lib::commands::automation::clear_all_automation,
        ])
        .setup(|_app| {
            info!("✅ Application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}

/// Initialize logging/tracing system
fn init_logging() {
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string());
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(log_dir, "midi-app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,midi_app=debug,midi_pipeline=debug,midi_daw=debug".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();
}

/// Initialize PostgreSQL database connection pool
async fn initialize_database_pool(database_url: &str) -> Result<sqlx::PgPool, String> {
    info!(
        "Connecting to database: {}",
        database_url.replace(":145278963", ":****")
    );

    // Get max connections from environment or use default
    let max_connections: u32 = std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // Create connection pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    // Test connection with a simple query
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to execute test query: {}", e))?;

    info!(
        "Database connection pool created with {} max connections",
        max_connections
    );

    Ok(pool)
}
