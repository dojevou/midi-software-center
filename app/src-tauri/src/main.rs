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

/// Sentry guard - must be kept alive for the duration of the application
static SENTRY_GUARD: std::sync::OnceLock<Option<sentry::ClientInitGuard>> =
    std::sync::OnceLock::new();

// Import Pipeline types (migrated from pipeline/src-tauri)
use midi_app::{AppState as PipelineState, Database};

// Import DAW types
use midi_app::commands::daw::gear::GearState;
use midi_app::commands::daw::mixer::MixerState;
use midi_app::commands::daw::preferences::{
    AppSettingsState, KeyboardShortcutsState, RecentProjectsState, WindowLayoutState,
};
use midi_app::commands::daw::presets::PresetsState;
use midi_app::commands::daw::system::SystemState;
use midi_app::commands::daw::{
    AutomationState,
    DAWState, // Window state with mixer (different from DawState!)
    DawAppState,
    DawState,
};
use midi_app::midi_io::MidiManager;
use midi_app::profiling::commands::ProfilingState;
use midi_app::sequencer::SequencerEngine;
use midi_app::undo_redo::commands::UndoRedoState;

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

    // Initialize VIP3 analytics service (DuckDB for fast aggregations)
    let vip3_analytics = Arc::new(
        midi_app::services::VIP3AnalyticsService::new(database_url.clone())
            .expect("Failed to initialize VIP3 analytics service")
    );
    info!("✅ VIP3 analytics service initialized (DuckDB)");

    // Initialize Meilisearch client (optional - for full-text search)
    let meilisearch = match std::env::var("MEILISEARCH_URL") {
        Ok(url) => {
            let api_key = std::env::var("MEILISEARCH_API_KEY").ok();
            match midi_app::services::MeilisearchClient::new(&url, api_key.as_deref(), None) {
                Ok(client) => {
                    info!("✅ Meilisearch client initialized ({})", url);
                    Some(Arc::new(client))
                },
                Err(e) => {
                    warn!("⚠️  Meilisearch initialization failed: {}", e);
                    warn!("⚠️  Full-text search will not be available");
                    None
                }
            }
        },
        Err(_) => {
            info!("ℹ️  MEILISEARCH_URL not set, skipping Meilisearch initialization");
            info!("ℹ️  Full-text search will not be available");
            None
        }
    };

    // Create Pipeline state
    let pipeline_state = PipelineState { database, vip3_analytics, meilisearch };

    // Clone db_pool before moving it into state (needed for DAW commands)
    let db_pool_for_daw = db_pool.clone();

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

    // Create DAW window state (transport, tracks)
    let daw_state = DawState::default();

    // Create DAW window state with mixer (for window commands like get_mixer_state)
    let daw_window_state = DAWState::default();
    info!("✅ DAW window state (with mixer) initialized");

    // Create automation state
    let automation_state = AutomationState::new();

    // Create mixer state
    let mixer_state = MixerState::default();
    info!("✅ Mixer state initialized");

    // Create undo/redo state
    let undo_redo_state = UndoRedoState::default();
    info!("✅ Undo/Redo state initialized");

    // Create profiling state
    let profiling_state = ProfilingState::default();
    info!("✅ Profiling state initialized");

    // Create system state
    let system_state = SystemState::default();
    info!("✅ System state initialized");

    // Create gear state (for MIDI gear profiles, CC mappings, programs)
    let gear_state = GearState::default();
    info!("✅ Gear state initialized");

    // Create presets state (for mixer presets, track templates, project templates)
    let presets_state = PresetsState::default();
    info!("✅ Presets state initialized");

    // Create preferences states (settings, window layouts, keyboard shortcuts, recent projects)
    let app_settings_state = AppSettingsState::default();
    let window_layout_state = WindowLayoutState::default();
    let keyboard_shortcuts_state = KeyboardShortcutsState::default();
    let recent_projects_state = RecentProjectsState::default();
    info!("✅ Preferences states initialized (settings, layouts, shortcuts, recent)");

    // Create DAW AppState for DAW commands (search, analysis, etc.)
    let daw_app_state = DawAppState { db_pool: db_pool_for_daw };
    info!("✅ DAW AppState initialized with database pool");

    // Build and run Tauri application with ALL commands from both apps
    tauri::Builder::default()
        .manage(state)
        .manage(daw_app_state) // Register DAW's AppState for DAW commands
        .manage(midi_manager)
        .manage(sequencer_engine)
        .manage(daw_state)
        .manage(daw_window_state) // DAWState for window commands (get_mixer_state, etc.)
        .manage(automation_state)
        .manage(mixer_state)
        .manage(undo_redo_state)
        .manage(profiling_state)
        .manage(system_state)
        .manage(gear_state)
        .manage(presets_state)
        .manage(app_settings_state)
        .manage(window_layout_state)
        .manage(keyboard_shortcuts_state)
        .manage(recent_projects_state)
        .invoke_handler(tauri::generate_handler![
            // ========================================================================
            // APP LIFECYCLE COMMANDS
            // ========================================================================
            shutdown_application,
            health_check,
            // ========================================================================
            // APP FILE SYSTEM COMMANDS
            // ========================================================================
            show_in_folder,
            // ========================================================================
            // PIPELINE COMMANDS (File Management, Import, Search, Analysis)
            // ========================================================================

            // File commands
            midi_app::commands::pipeline::files::test_db_connection,
            midi_app::commands::pipeline::files::get_file_count,
            midi_app::commands::pipeline::files::get_file_details,
            midi_app::commands::pipeline::files::get_file,
            midi_app::commands::pipeline::files::list_files,
            midi_app::commands::pipeline::files::get_files_by_category,
            midi_app::commands::pipeline::files::get_recent_files,
            midi_app::commands::pipeline::files::delete_file,
            // Import commands
            midi_app::commands::pipeline::file_import::import_single_file,
            midi_app::commands::pipeline::file_import::import_directory,
            midi_app::commands::pipeline::archive_import::import_archive_collection,
            // Search commands (Pipeline) - NOTE: search_files moved to DAW section to avoid collision
            midi_app::commands::pipeline::search::get_all_tags,
            midi_app::commands::pipeline::search::get_files_by_tag,
            midi_app::commands::pipeline::search::get_bpm_range,
            midi_app::commands::pipeline::search::get_all_keys,
            // Analysis commands
            midi_app::commands::pipeline::analyze::start_analysis,
            // Split file commands
            midi_app::commands::pipeline::split_file::split_file,
            midi_app::commands::pipeline::split_file::split_file_batch,
            // Statistics commands
            midi_app::commands::pipeline::stats::get_category_stats,
            midi_app::commands::pipeline::stats::get_manufacturer_stats,
            midi_app::commands::pipeline::stats::get_key_signature_stats,
            midi_app::commands::pipeline::stats::get_recently_added_count,
            midi_app::commands::pipeline::stats::get_duplicate_count,
            midi_app::commands::pipeline::stats::get_database_size,
            midi_app::commands::pipeline::stats::check_database_health,
            // Tag commands
            midi_app::commands::pipeline::tags::get_file_tags,
            midi_app::commands::pipeline::tags::get_popular_tags,
            midi_app::commands::pipeline::tags::search_tags,
            midi_app::commands::pipeline::tags::get_tag_categories,
            midi_app::commands::pipeline::tags::get_tags_by_category,
            midi_app::commands::pipeline::tags::update_file_tags,
            midi_app::commands::pipeline::tags::add_tags_to_file,
            midi_app::commands::pipeline::tags::remove_tag_from_file,
            midi_app::commands::pipeline::tags::get_files_by_tags,
            midi_app::commands::pipeline::tags::get_tag_stats,
            // VIP3 Browser lookup commands (legacy tags module)
            midi_app::commands::pipeline::tags::get_all_instruments,
            midi_app::commands::pipeline::tags::get_vip3_genre_tags,
            midi_app::commands::pipeline::tags::get_vip3_moods,
            midi_app::commands::pipeline::tags::get_vip3_keys,
            // VIP3 Browser - Search
            midi_app::commands::pipeline::vip3::search::search_files_vip3,
            midi_app::commands::pipeline::vip3::search::get_vip3_filter_counts,
            // VIP3 Browser - Dynamic Filter Counts (repository-based)
            midi_app::commands::pipeline::vip3::filter_counts::get_vip3_dynamic_filter_counts,
            // VIP3 Browser - Lookups
            midi_app::commands::pipeline::vip3::lookups::get_all_timbres,
            midi_app::commands::pipeline::vip3::lookups::get_all_styles,
            midi_app::commands::pipeline::vip3::lookups::get_all_articulations,
            midi_app::commands::pipeline::vip3::lookups::get_all_bpm_ranges,
            midi_app::commands::pipeline::vip3::lookups::get_all_musical_keys,
            // VIP3 Browser - Categories
            midi_app::commands::pipeline::vip3::categories::get_file_categories,
            midi_app::commands::pipeline::vip3::categories::add_timbre_to_file,
            midi_app::commands::pipeline::vip3::categories::add_style_to_file,
            midi_app::commands::pipeline::vip3::categories::add_articulation_to_file,
            midi_app::commands::pipeline::vip3::categories::remove_timbre_from_file,
            midi_app::commands::pipeline::vip3::categories::remove_style_from_file,
            midi_app::commands::pipeline::vip3::categories::remove_articulation_from_file,
            // VIP3 Browser - Category List Fetching (for initialization)
            midi_app::commands::pipeline::vip3::categories::get_vip3_folders,
            midi_app::commands::pipeline::vip3::categories::get_vip3_instruments,
            midi_app::commands::pipeline::vip3::categories::get_vip3_timbres,
            midi_app::commands::pipeline::vip3::categories::get_vip3_styles,
            midi_app::commands::pipeline::vip3::categories::get_vip3_articulations,
            midi_app::commands::pipeline::vip3::categories::get_vip3_manufacturers,
            // VIP3 Browser - Combined Categories (for efficient initialization)
            midi_app::commands::pipeline::vip3::categories::get_all_vip3_categories,
            // VIP3 Browser - Filter Counts (DuckDB analytics for 10-20x faster aggregations)
            midi_app::commands::pipeline::vip3::categories::get_vip3_analytics_counts,
            // VIP3 Browser - Favorites
            midi_app::commands::pipeline::vip3::favorites::toggle_favorite,
            midi_app::commands::pipeline::vip3::favorites::set_favorite,
            midi_app::commands::pipeline::vip3::favorites::get_favorites,
            midi_app::commands::pipeline::vip3::favorites::get_favorite_count,
            // VIP3 Browser - Collections
            midi_app::commands::pipeline::vip3::collections::create_collection,
            midi_app::commands::pipeline::vip3::collections::get_collections,
            midi_app::commands::pipeline::vip3::collections::get_collection,
            midi_app::commands::pipeline::vip3::collections::update_collection,
            midi_app::commands::pipeline::vip3::collections::delete_collection,
            midi_app::commands::pipeline::vip3::collections::add_file_to_collection,
            midi_app::commands::pipeline::vip3::collections::remove_file_from_collection,
            midi_app::commands::pipeline::vip3::collections::get_collection_files,
            midi_app::commands::pipeline::vip3::collections::batch_add_files_to_collection,
            midi_app::commands::pipeline::vip3::collections::batch_remove_files_from_collection,
            midi_app::commands::pipeline::vip3::collections::clear_collection,
            midi_app::commands::pipeline::vip3::collections::reorder_collection_files,
            // VIP3 Browser - Saved Searches
            midi_app::commands::pipeline::vip3::saved_searches::save_search,
            midi_app::commands::pipeline::vip3::saved_searches::get_saved_searches,
            midi_app::commands::pipeline::vip3::saved_searches::load_saved_search,
            midi_app::commands::pipeline::vip3::saved_searches::delete_saved_search,
            midi_app::commands::pipeline::vip3::saved_searches::toggle_saved_search_pin,
            // VIP3 Bulk retag commands (high-performance Rust + Rayon)
            midi_app::commands::pipeline::vip3::bulk_retag::bulk_retag_vip3,
            midi_app::commands::pipeline::vip3::bulk_retag::update_vip3_counts,
            // Meilisearch commands (full-text search)
            midi_app::commands::pipeline::meilisearch::meilisearch_initialize,
            midi_app::commands::pipeline::meilisearch::meilisearch_search,
            midi_app::commands::pipeline::meilisearch::meilisearch_faceted_search,
            midi_app::commands::pipeline::meilisearch::meilisearch_index_file,
            midi_app::commands::pipeline::meilisearch::meilisearch_index_files_batch,
            midi_app::commands::pipeline::meilisearch::meilisearch_delete_file,
            midi_app::commands::pipeline::meilisearch::meilisearch_delete_files_batch,
            midi_app::commands::pipeline::meilisearch::meilisearch_clear_index,
            midi_app::commands::pipeline::meilisearch::meilisearch_get_stats,
            midi_app::commands::pipeline::meilisearch::meilisearch_rebuild_index,
            // Progress tracking commands
            midi_app::commands::pipeline::progress::start_progress_tracking,
            midi_app::commands::pipeline::progress::update_progress,
            midi_app::commands::pipeline::progress::increment_error_count,
            midi_app::commands::pipeline::progress::increment_duplicate_count,
            midi_app::commands::pipeline::progress::complete_progress,
            midi_app::commands::pipeline::progress::get_current_progress,
            midi_app::commands::pipeline::progress::reset_progress,
            // System commands
            midi_app::commands::pipeline::system::get_system_info,
            // ========================================================================
            // DAW COMMANDS (Sequencer, MIDI, Playback, Automation)
            // ========================================================================

            // Database commands (DAW)
            midi_app::commands::daw::daw_initialize_database,
            // MIDI commands
            midi_app::commands::daw::midi::midi_list_devices,
            midi_app::commands::daw::midi::midi_connect,
            midi_app::commands::daw::midi::midi_disconnect,
            midi_app::commands::daw::midi::midi_is_connected,
            midi_app::commands::daw::midi::midi_get_current_device,
            midi_app::commands::daw::midi::midi_send_test_note,
            // MIDI I/O commands (port management, sync, routing)
            midi_app::commands::daw::midi_io::midi_io_get_state,
            midi_app::commands::daw::midi_io::midi_io_detect_ports,
            midi_app::commands::daw::midi_io::midi_io_add_port,
            midi_app::commands::daw::midi_io::midi_io_update_port,
            midi_app::commands::daw::midi_io::midi_io_remove_port,
            midi_app::commands::daw::midi_io::midi_io_set_port_connected,
            midi_app::midi_io::output::midi_send_cc,
            midi_app::midi_io::output::midi_output_list_devices,
            // Sequencer commands
            midi_app::commands::daw::sequencer::start_sequencer,
            midi_app::commands::daw::sequencer::stop_sequencer,
            midi_app::commands::daw::sequencer::pause_sequencer,
            midi_app::commands::daw::sequencer::resume_sequencer,
            midi_app::commands::daw::sequencer::get_playback_position,
            midi_app::commands::daw::sequencer::seek_position,
            midi_app::commands::daw::sequencer::set_tempo,
            midi_app::commands::daw::sequencer::get_tempo,
            midi_app::commands::daw::sequencer::add_track,
            midi_app::commands::daw::sequencer::load_file_to_daw,
            midi_app::commands::daw::sequencer::remove_track,
            midi_app::commands::daw::sequencer::update_track,
            midi_app::commands::daw::sequencer::get_tracks,
            midi_app::commands::daw::sequencer::load_sequencer_tracks,
            midi_app::commands::daw::sequencer::is_sequencer_playing,
            // Search commands (DAW)
            midi_app::commands::daw::search::search_files,
            midi_app::commands::daw::search::daw_get_file_details,
            midi_app::commands::daw::search::get_search_suggestions,
            // Analysis commands (DAW)
            midi_app::commands::daw::analysis::find_compatible_files,
            midi_app::commands::daw::analysis::add_favorite,
            midi_app::commands::daw::analysis::remove_favorite,
            midi_app::commands::daw::analysis::is_favorite,
            midi_app::commands::daw::analysis::daw_get_favorites,
            midi_app::commands::daw::analysis::get_usage_stats,
            // Project commands
            midi_app::commands::daw::project::load_multiple_tracks,
            midi_app::commands::daw::project::clear_all_tracks,
            midi_app::commands::daw::project::get_track_details,
            midi_app::commands::daw::project::project_create,
            midi_app::commands::daw::project::project_load,
            midi_app::commands::daw::project::project_get,
            midi_app::commands::daw::project::project_update,
            midi_app::commands::daw::project::project_delete,
            midi_app::commands::daw::project::project_list,
            midi_app::commands::daw::project::project_get_recent,
            midi_app::commands::daw::project::project_save_tracks,
            midi_app::commands::daw::project::project_load_tracks,
            midi_app::commands::daw::project::project_load_full,
            midi_app::commands::daw::project::project_delete_tracks,
            // Export commands
            midi_app::commands::daw::export::export_project_midi,
            // Window commands (DAW)
            midi_app::commands::daw::window::play_transport,
            midi_app::commands::daw::window::stop_transport,
            midi_app::commands::daw::window::pause_transport,
            midi_app::commands::daw::window::set_playback_position,
            midi_app::commands::daw::window::get_playback_state,
            midi_app::commands::daw::window::set_bpm,
            midi_app::commands::daw::window::get_bpm,
            midi_app::commands::daw::window::set_time_signature,
            midi_app::commands::daw::window::get_time_signature,
            midi_app::commands::daw::window::set_key_signature,
            midi_app::commands::daw::window::get_key_signature,
            midi_app::commands::daw::window::add_window_track,
            midi_app::commands::daw::window::remove_window_track,
            midi_app::commands::daw::window::get_all_window_tracks,
            midi_app::commands::daw::window::set_track_visible,
            midi_app::commands::daw::window::set_track_muted,
            midi_app::commands::daw::window::set_track_soloed,
            midi_app::commands::daw::window::get_track_info,
            midi_app::commands::daw::window::update_track_label,
            midi_app::commands::daw::window::get_mixer_state,
            midi_app::commands::daw::window::set_channel_volume,
            midi_app::commands::daw::window::set_channel_pan,
            midi_app::commands::daw::window::set_channel_mute,
            midi_app::commands::daw::window::set_channel_solo,
            midi_app::commands::daw::window::get_daw_state,
            midi_app::commands::daw::window::reset_daw_state,
            // Mixer commands (using mixer:: module for MixerState-based commands)
            midi_app::commands::daw::mixer::mixer_get_channels,
            midi_app::commands::daw::mixer::mixer_get_master,
            midi_app::commands::daw::mixer::mixer_get_channel,
            midi_app::commands::daw::mixer::mixer_add_channel,
            midi_app::commands::daw::mixer::mixer_remove_channel,
            midi_app::commands::daw::mixer::mixer_update_channel,
            midi_app::commands::daw::mixer::mixer_set_volume,
            midi_app::commands::daw::mixer::mixer_set_pan,
            midi_app::commands::daw::mixer::mixer_set_mute,
            midi_app::commands::daw::mixer::mixer_set_solo,
            midi_app::commands::daw::mixer::mixer_toggle_mute,
            midi_app::commands::daw::mixer::mixer_toggle_solo,
            midi_app::commands::daw::mixer::mixer_set_enabled,
            midi_app::commands::daw::mixer::mixer_set_transpose,
            midi_app::commands::daw::mixer::mixer_set_octave_shift,
            midi_app::commands::daw::mixer::mixer_set_velocity,
            midi_app::commands::daw::mixer::mixer_set_velocity_range,
            midi_app::commands::daw::mixer::mixer_set_velocity_scale,
            midi_app::commands::daw::mixer::mixer_set_velocity_offset,
            midi_app::commands::daw::mixer::mixer_set_pitch_bend,
            midi_app::commands::daw::mixer::mixer_set_pitch_bend_range,
            midi_app::commands::daw::mixer::mixer_set_note_range,
            midi_app::commands::daw::mixer::mixer_set_key_scale,
            midi_app::commands::daw::mixer::mixer_set_quantize_grid,
            midi_app::commands::daw::mixer::mixer_set_quantize_strength,
            midi_app::commands::daw::mixer::mixer_set_swing,
            midi_app::commands::daw::mixer::mixer_set_delay,
            midi_app::commands::daw::mixer::mixer_set_humanize,
            midi_app::commands::daw::mixer::mixer_set_expression,
            midi_app::commands::daw::mixer::mixer_set_modulation,
            midi_app::commands::daw::mixer::mixer_set_sustain_pedal,
            midi_app::commands::daw::mixer::mixer_set_cc_values,
            midi_app::commands::daw::mixer::mixer_set_program_change,
            midi_app::commands::daw::mixer::mixer_set_midi_port,
            // Master channel commands
            midi_app::commands::daw::mixer::mixer_set_master_volume,
            midi_app::commands::daw::mixer::mixer_set_master_pan,
            midi_app::commands::daw::mixer::mixer_get_meters,
            midi_app::commands::daw::mixer::mixer_set_master_limiter,
            midi_app::commands::daw::mixer::mixer_set_master_compressor,
            midi_app::commands::daw::mixer::mixer_set_master_enabled,
            midi_app::commands::daw::mixer::mixer_set_master_transpose,
            midi_app::commands::daw::mixer::mixer_set_master_velocity,
            midi_app::commands::daw::mixer::mixer_set_master_velocity_scale,
            midi_app::commands::daw::mixer::mixer_set_master_velocity_range,
            midi_app::commands::daw::mixer::mixer_set_master_velocity_offset,
            midi_app::commands::daw::mixer::mixer_set_master_pitch_bend,
            midi_app::commands::daw::mixer::mixer_set_master_note_range,
            midi_app::commands::daw::mixer::mixer_set_master_quantize_grid,
            midi_app::commands::daw::mixer::mixer_set_master_quantize_strength,
            midi_app::commands::daw::mixer::mixer_set_master_tempo,
            midi_app::commands::daw::mixer::mixer_set_master_tempo_multiplier,
            midi_app::commands::daw::mixer::mixer_set_master_delay,
            midi_app::commands::daw::mixer::mixer_set_master_swing,
            midi_app::commands::daw::mixer::mixer_set_master_humanize,
            midi_app::commands::daw::mixer::mixer_set_master_output,
            midi_app::commands::daw::mixer::mixer_set_master_clock_enabled,
            midi_app::commands::daw::mixer::mixer_set_master_transport_enabled,
            midi_app::commands::daw::mixer::mixer_reset_peaks,
            // Effects commands
            midi_app::commands::daw::mixer::mixer_add_effect,
            midi_app::commands::daw::mixer::mixer_remove_effect,
            midi_app::commands::daw::mixer::mixer_set_effect_enabled,
            midi_app::commands::daw::mixer::mixer_bypass_effect,
            midi_app::commands::daw::mixer::mixer_set_effect_parameter,
            midi_app::commands::daw::mixer::mixer_reorder_effects,
            midi_app::commands::daw::mixer::mixer_toggle_channel_effect,
            // Send and routing commands
            midi_app::commands::daw::mixer::mixer_set_send,
            midi_app::commands::daw::mixer::mixer_add_send,
            midi_app::commands::daw::mixer::mixer_remove_send,
            midi_app::commands::daw::mixer::mixer_create_bus,
            midi_app::commands::daw::mixer::mixer_remove_bus,
            midi_app::commands::daw::mixer::mixer_route_track,
            midi_app::commands::daw::mixer::mixer_get_routing,
            midi_app::commands::daw::mixer::mixer_get_buses,
            // Reset and copy commands
            midi_app::commands::daw::mixer::mixer_reset_track,
            midi_app::commands::daw::mixer::mixer_reset_all,
            midi_app::commands::daw::mixer::mixer_copy_settings,
            // Monitoring & Recording commands (Day 3)
            midi_app::commands::daw::mixer::mixer_set_monitoring,
            midi_app::commands::daw::mixer::mixer_set_record_arm,
            midi_app::commands::daw::mixer::mixer_set_latency_compensation,
            midi_app::commands::daw::mixer::mixer_get_latency_report,
            // Preset commands (Day 4)
            midi_app::commands::daw::mixer::mixer_save_preset,
            midi_app::commands::daw::mixer::mixer_load_preset,
            midi_app::commands::daw::mixer::mixer_get_presets,
            midi_app::commands::daw::mixer::mixer_delete_preset,
            midi_app::commands::daw::mixer::mixer_toggle_preset_favorite,
            // Plugin Discovery commands (Day 5)
            midi_app::commands::daw::mixer::mixer_get_plugin_list,
            midi_app::commands::daw::mixer::mixer_scan_plugins,
            // Automation commands
            midi_app::commands::daw::automation::create_automation_lane,
            midi_app::commands::daw::automation::delete_automation_lane,
            midi_app::commands::daw::automation::add_automation_point,
            midi_app::commands::daw::automation::remove_automation_point,
            midi_app::commands::daw::automation::move_automation_point,
            midi_app::commands::daw::automation::set_automation_curve_type,
            midi_app::commands::daw::automation::get_automation_lane,
            midi_app::commands::daw::automation::get_track_automation,
            midi_app::commands::daw::automation::get_automation_value,
            midi_app::commands::daw::automation::clear_track_automation,
            midi_app::commands::daw::automation::clear_all_automation,
            midi_app::commands::daw::automation::set_automation_mode,
            midi_app::commands::daw::automation::get_automation_mode,
            // Automation batch & utility commands (Stream D)
            midi_app::commands::daw::automation::delete_automation_points_batch,
            midi_app::commands::daw::automation::get_automation_points_in_range,
            midi_app::commands::daw::automation::get_automation_point_count,
            midi_app::commands::daw::automation::get_automation_lane_count,
            midi_app::commands::daw::automation::get_automation_lane_display_name,
            midi_app::commands::daw::automation::get_automation_lane_color,
            midi_app::commands::daw::automation::clear_automation_curve,
            midi_app::commands::daw::automation::get_parameter_type_string,
            midi_app::commands::daw::automation::get_parameter_type_color,
            // Automation copy/paste commands (Stream D)
            midi_app::commands::daw::automation::copy_automation_points,
            midi_app::commands::daw::automation::paste_automation_points,
            midi_app::commands::daw::automation::cut_automation_points,
            // CC Recording commands (Stream D)
            midi_app::commands::daw::automation::start_cc_recording,
            midi_app::commands::daw::automation::record_cc_value,
            midi_app::commands::daw::automation::stop_cc_recording,
            midi_app::commands::daw::automation::record_cc_automation_batch,
            // ========================================================================
            // UNDO/REDO COMMANDS
            // ========================================================================
            midi_app::undo_redo::commands::undo,
            midi_app::undo_redo::commands::redo,
            midi_app::undo_redo::commands::can_undo,
            midi_app::undo_redo::commands::can_redo,
            midi_app::undo_redo::commands::undo_description,
            midi_app::undo_redo::commands::redo_description,
            midi_app::undo_redo::commands::clear_history,
            midi_app::undo_redo::commands::undo_count,
            midi_app::undo_redo::commands::redo_count,
            midi_app::undo_redo::commands::undo_descriptions,
            midi_app::undo_redo::commands::redo_descriptions,
            midi_app::undo_redo::commands::memory_usage,
            midi_app::undo_redo::commands::set_max_depth,
            midi_app::undo_redo::commands::set_max_memory,
            midi_app::undo_redo::commands::set_compression,
            // ========================================================================
            // PROFILING COMMANDS (Memory, Cache, Render Metrics)
            // ========================================================================
            midi_app::profiling::commands::get_memory_metrics,
            midi_app::profiling::commands::get_cache_stats,
            midi_app::profiling::commands::cache_clear,
            midi_app::profiling::commands::get_render_metrics,
            midi_app::profiling::commands::record_frame,
            midi_app::profiling::commands::reset_render_metrics,
            midi_app::profiling::commands::get_memory_report,
            midi_app::profiling::commands::track_allocation,
            midi_app::profiling::commands::track_deallocation,
            // ========================================================================
            // SYSTEM COMMANDS (CPU, Memory, Audio/MIDI Devices)
            // ========================================================================
            midi_app::commands::daw::system::daw_get_system_info,
            midi_app::commands::daw::system::get_system_settings,
            midi_app::commands::daw::system::update_settings,
            midi_app::commands::daw::system::start_system_monitoring,
            midi_app::commands::daw::system::stop_system_monitoring,
            midi_app::commands::daw::system::set_audio_device,
            midi_app::commands::daw::system::set_midi_device,
            midi_app::commands::daw::system::get_cpu_usage,
            midi_app::commands::daw::system::get_memory_usage,
            // ========================================================================
            // DEVICE DISCOVERY COMMANDS (Audio/MIDI Devices - no state required)
            // ========================================================================
            midi_app::commands::daw::settings::get_audio_devices,
            midi_app::commands::daw::settings::get_midi_devices,
            // ========================================================================
            // GEAR COMMANDS (MIDI Gear Profiles, CC Mappings, Programs, User Gear)
            // ========================================================================
            // Gear profiles
            midi_app::commands::daw::gear::gear_profiles_list,
            midi_app::commands::daw::gear::gear_profiles_list_by_type,
            midi_app::commands::daw::gear::gear_profiles_get,
            midi_app::commands::daw::gear::gear_profiles_search,
            midi_app::commands::daw::gear::gear_profiles_create,
            midi_app::commands::daw::gear::gear_profiles_update,
            midi_app::commands::daw::gear::gear_profiles_delete,
            // Gear CC mappings
            midi_app::commands::daw::gear::gear_cc_list,
            midi_app::commands::daw::gear::gear_cc_get,
            midi_app::commands::daw::gear::gear_cc_get_by_number,
            midi_app::commands::daw::gear::gear_cc_create,
            midi_app::commands::daw::gear::gear_cc_update,
            midi_app::commands::daw::gear::gear_cc_delete,
            // Gear programs
            midi_app::commands::daw::gear::gear_programs_list,
            midi_app::commands::daw::gear::gear_programs_list_by_bank,
            midi_app::commands::daw::gear::gear_programs_get,
            midi_app::commands::daw::gear::gear_programs_search,
            midi_app::commands::daw::gear::gear_programs_create,
            midi_app::commands::daw::gear::gear_programs_update,
            midi_app::commands::daw::gear::gear_programs_delete,
            // User gear
            midi_app::commands::daw::gear::user_gear_list,
            midi_app::commands::daw::gear::user_gear_list_favorites,
            midi_app::commands::daw::gear::user_gear_get,
            midi_app::commands::daw::gear::user_gear_add,
            midi_app::commands::daw::gear::user_gear_update,
            midi_app::commands::daw::gear::user_gear_set_favorite,
            midi_app::commands::daw::gear::user_gear_mark_used,
            midi_app::commands::daw::gear::user_gear_remove,
            midi_app::commands::daw::gear::user_gear_get_with_profile,
            // ========================================================================
            // PRESETS COMMANDS (Mixer Presets, Track Templates, Project Templates)
            // ========================================================================
            // Mixer presets
            midi_app::commands::daw::presets::mixer_presets_list,
            midi_app::commands::daw::presets::mixer_presets_list_by_category,
            midi_app::commands::daw::presets::mixer_presets_get,
            midi_app::commands::daw::presets::mixer_presets_search,
            midi_app::commands::daw::presets::mixer_presets_create,
            midi_app::commands::daw::presets::mixer_presets_update,
            midi_app::commands::daw::presets::mixer_presets_delete,
            // Track templates
            midi_app::commands::daw::presets::track_templates_list,
            midi_app::commands::daw::presets::track_templates_list_by_category,
            midi_app::commands::daw::presets::track_templates_list_by_type,
            midi_app::commands::daw::presets::track_templates_get,
            midi_app::commands::daw::presets::track_templates_search,
            midi_app::commands::daw::presets::track_templates_create,
            midi_app::commands::daw::presets::track_templates_update,
            midi_app::commands::daw::presets::track_templates_delete,
            // Project templates
            midi_app::commands::daw::presets::project_templates_list,
            midi_app::commands::daw::presets::project_templates_list_by_category,
            midi_app::commands::daw::presets::project_templates_get,
            midi_app::commands::daw::presets::project_templates_search,
            midi_app::commands::daw::presets::project_templates_create,
            midi_app::commands::daw::presets::project_templates_update,
            midi_app::commands::daw::presets::project_templates_delete,
            midi_app::commands::daw::presets::project_templates_duplicate,
            // ========================================================================
            // PREFERENCES COMMANDS (Settings, Window Layouts, Keyboard Shortcuts, Recent)
            // ========================================================================
            // App settings
            midi_app::commands::daw::preferences::settings_get_all,
            midi_app::commands::daw::preferences::settings_get_by_category,
            midi_app::commands::daw::preferences::settings_get,
            midi_app::commands::daw::preferences::settings_set_string,
            midi_app::commands::daw::preferences::settings_set_int,
            midi_app::commands::daw::preferences::settings_set_float,
            midi_app::commands::daw::preferences::settings_set_bool,
            midi_app::commands::daw::preferences::settings_set_json,
            midi_app::commands::daw::preferences::settings_reset,
            midi_app::commands::daw::preferences::settings_reset_category,
            midi_app::commands::daw::preferences::settings_delete,
            // Window layouts
            midi_app::commands::daw::preferences::layouts_list,
            midi_app::commands::daw::preferences::layouts_get_current,
            midi_app::commands::daw::preferences::layouts_get,
            midi_app::commands::daw::preferences::layouts_save,
            midi_app::commands::daw::preferences::layouts_update,
            midi_app::commands::daw::preferences::layouts_set_default,
            midi_app::commands::daw::preferences::layouts_apply,
            midi_app::commands::daw::preferences::layouts_delete,
            // Keyboard shortcuts
            midi_app::commands::daw::preferences::shortcuts_list,
            midi_app::commands::daw::preferences::shortcuts_list_by_category,
            midi_app::commands::daw::preferences::shortcuts_get_by_action,
            midi_app::commands::daw::preferences::shortcuts_get_by_combo,
            midi_app::commands::daw::preferences::shortcuts_set,
            midi_app::commands::daw::preferences::shortcuts_reset,
            midi_app::commands::daw::preferences::shortcuts_reset_all,
            midi_app::commands::daw::preferences::shortcuts_add,
            midi_app::commands::daw::preferences::shortcuts_delete,
            // Recent projects
            midi_app::commands::daw::preferences::recent_list,
            midi_app::commands::daw::preferences::recent_add,
            midi_app::commands::daw::preferences::recent_set_pinned,
            midi_app::commands::daw::preferences::recent_remove,
            midi_app::commands::daw::preferences::recent_clear,
            midi_app::commands::daw::preferences::recent_clear_all,
        ])
        .setup(|_app| {
            info!("✅ Application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}

/// Shutdown application cleanly
#[tauri::command]
async fn shutdown_application(app: tauri::AppHandle) -> Result<(), String> {
    info!("🛑 Shutting down application...");

    // Give time for cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Exit the application
    app.exit(0);

    Ok(())
}

/// Health check response structure
#[derive(serde::Serialize)]
pub struct HealthCheckResponse {
    /// Overall health status
    pub status: String,
    /// Application version
    pub version: String,
    /// Database connection status
    pub database: DatabaseHealth,
    /// Sentry error tracking status
    pub sentry: SentryHealth,
    /// Application uptime in seconds (since startup)
    pub uptime_seconds: u64,
    /// Timestamp of the health check
    pub timestamp: String,
}

/// Database health status
#[derive(serde::Serialize)]
pub struct DatabaseHealth {
    /// Whether database is connected
    pub connected: bool,
    /// Connection pool status
    pub pool_status: String,
    /// Last successful query time
    pub latency_ms: Option<u64>,
}

/// Sentry health status
#[derive(serde::Serialize)]
pub struct SentryHealth {
    /// Whether Sentry is configured
    pub configured: bool,
    /// Current environment
    pub environment: Option<String>,
}

/// Application startup time (for uptime calculation)
static APP_START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Get application health status
///
/// Returns comprehensive health information including:
/// - Database connectivity and latency
/// - Sentry configuration status
/// - Application uptime
/// - Version information
#[tauri::command]
async fn health_check(state: tauri::State<'_, AppState>) -> Result<HealthCheckResponse, String> {
    // Initialize start time on first call
    let _ = APP_START_TIME.get_or_init(std::time::Instant::now);

    // Calculate uptime
    let uptime = APP_START_TIME.get().map(|start| start.elapsed().as_secs()).unwrap_or(0);

    // Check database connectivity
    let db_start = std::time::Instant::now();
    let db_connected = state.pipeline.database.test_connection().await.is_ok();
    let db_latency = if db_connected {
        Some(db_start.elapsed().as_millis() as u64)
    } else {
        None
    };

    let database = DatabaseHealth {
        connected: db_connected,
        pool_status: if db_connected {
            "healthy".to_string()
        } else {
            "disconnected".to_string()
        },
        latency_ms: db_latency,
    };

    // Check Sentry status
    let sentry_configured = SENTRY_GUARD.get().map(|guard| guard.is_some()).unwrap_or(false);

    let sentry_env = if sentry_configured {
        std::env::var("SENtry_ENVIRONMENT").ok()
    } else {
        None
    };

    let sentry = SentryHealth { configured: sentry_configured, environment: sentry_env };

    // Determine overall status
    let status = if db_connected { "healthy" } else { "degraded" };

    let response = HealthCheckResponse {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database,
        sentry,
        uptime_seconds: uptime,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    info!(
        "Health check: status={}, db={}, sentry={}",
        response.status, response.database.connected, response.sentry.configured
    );

    Ok(response)
}

/// Show a file in the system file manager
///
/// Opens the system file manager (Nautilus, Dolphin, Finder, Explorer)
/// and highlights the specified file.
#[tauri::command]
async fn show_in_folder(path: String) -> Result<(), String> {
    use std::path::Path;

    let file_path = Path::new(&path);

    // Verify the path exists
    if !file_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Get the parent directory (for files) or the directory itself (for folders)
    let reveal_path = if file_path.is_file() {
        file_path.parent().unwrap_or(file_path)
    } else {
        file_path
    };

    // Use opener to reveal in file manager
    opener::reveal(reveal_path).map_err(|e| format!("Failed to open file manager: {}", e))?;

    info!("Revealed in file manager: {}", path);
    Ok(())
}

/// Initialize logging/tracing system with Sentry integration
fn init_logging() {
    // Use XDG data directory for logs (writable location for AppImage)
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| {
        dirs::data_local_dir()
            .map(|p| p.join("midi-software-center").join("logs"))
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp/midi-software-center/logs"))
            .to_string_lossy()
            .to_string()
    });
    std::fs::create_dir_all(&log_dir).ok();

    // Initialize Sentry if DSN is configured
    let sentry_guard = init_sentry();
    let _ = SENTRY_GUARD.set(sentry_guard);

    let file_appender = tracing_appender::rolling::daily(log_dir, "midi-app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Build tracing subscriber with optional Sentry layer
    let registry = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info,midi_app=debug,midi_pipeline=debug,midi_daw=debug".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking));

    // Add Sentry layer if configured
    if std::env::var("SENTRY_DSN").is_ok() {
        registry.with(sentry_tracing::layer()).init();
    } else {
        registry.init();
    }
}

/// Initialize Sentry error tracking
///
/// Reads SENTRY_DSN from environment. If not set, Sentry is disabled.
/// Configure with:
///   export SENTRY_DSN="https://your-key@sentry.io/project-id"
///   export SENTRY_ENVIRONMENT="production"  # or "development"
fn init_sentry() -> Option<sentry::ClientInitGuard> {
    let dsn = std::env::var("SENTRY_DSN").ok()?;

    if dsn.is_empty() || dsn == "YOUR_SENTRY_DSN" {
        return None;
    }

    let environment =
        std::env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: Some(std::borrow::Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
            environment: Some(std::borrow::Cow::Owned(environment)),
            traces_sample_rate: 0.1, // Sample 10% of transactions
            attach_stacktrace: true,
            send_default_pii: false, // Don't send PII
            ..Default::default()
        },
    ));

    info!("✅ Sentry error tracking initialized");
    Some(guard)
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
