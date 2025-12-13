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

// Import Pipeline types
use midi_pipeline::{AppState as PipelineState, Database};

// Import DAW types
use midi_software_center_daw::commands::gear::GearState;
use midi_software_center_daw::commands::mixer::MixerState;
use midi_software_center_daw::commands::preferences::{
    AppSettingsState, KeyboardShortcutsState, RecentProjectsState, WindowLayoutState,
};
use midi_software_center_daw::commands::presets::PresetsState;
use midi_software_center_daw::commands::system::SystemState;
use midi_software_center_daw::commands::{
    AppState as DawAppState,
    AutomationState,
    DAWState, // Window state with mixer (different from DawState!)
    DawState,
};
use midi_software_center_daw::midi::MidiManager;
use midi_software_center_daw::profiling::commands::ProfilingState;
use midi_software_center_daw::sequencer::SequencerEngine;
use midi_software_center_daw::undo_redo::commands::UndoRedoState;

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
            // Search commands (Pipeline) - NOTE: search_files moved to DAW section to avoid collision
            midi_pipeline::commands::search::get_all_tags,
            midi_pipeline::commands::search::get_files_by_tag,
            midi_pipeline::commands::search::get_bpm_range,
            midi_pipeline::commands::search::get_all_keys,
            // Analysis commands
            midi_pipeline::commands::analyze::start_analysis,
            // Split file commands
            midi_pipeline::commands::split_file::split_file,
            midi_pipeline::commands::split_file::split_file_batch,
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
            midi_software_center_daw::commands::initialize_database,
            // MIDI commands
            midi_software_center_daw::commands::midi::midi_list_devices,
            midi_software_center_daw::commands::midi::midi_connect,
            midi_software_center_daw::commands::midi::midi_disconnect,
            midi_software_center_daw::commands::midi::midi_is_connected,
            midi_software_center_daw::commands::midi::midi_get_current_device,
            midi_software_center_daw::commands::midi::midi_send_test_note,
            // Sequencer commands
            midi_software_center_daw::commands::sequencer::start_sequencer,
            midi_software_center_daw::commands::sequencer::stop_sequencer,
            midi_software_center_daw::commands::sequencer::pause_sequencer,
            midi_software_center_daw::commands::sequencer::resume_sequencer,
            midi_software_center_daw::commands::sequencer::get_playback_position,
            midi_software_center_daw::commands::sequencer::seek_position,
            midi_software_center_daw::commands::sequencer::set_tempo,
            midi_software_center_daw::commands::sequencer::get_tempo,
            midi_software_center_daw::commands::sequencer::add_track,
            midi_software_center_daw::commands::sequencer::remove_track,
            midi_software_center_daw::commands::sequencer::update_track,
            midi_software_center_daw::commands::sequencer::get_tracks,
            midi_software_center_daw::commands::sequencer::load_sequencer_tracks,
            midi_software_center_daw::commands::sequencer::is_sequencer_playing,
            // Search commands (DAW)
            midi_software_center_daw::commands::search::search_files,
            midi_software_center_daw::commands::search::get_file_details,
            midi_software_center_daw::commands::search::get_search_suggestions,
            // Analysis commands (DAW)
            midi_software_center_daw::commands::analysis::find_compatible_files,
            midi_software_center_daw::commands::analysis::add_favorite,
            midi_software_center_daw::commands::analysis::remove_favorite,
            midi_software_center_daw::commands::analysis::is_favorite,
            midi_software_center_daw::commands::analysis::get_favorites,
            midi_software_center_daw::commands::analysis::get_usage_stats,
            // Project commands
            midi_software_center_daw::commands::project::load_multiple_tracks,
            midi_software_center_daw::commands::project::clear_all_tracks,
            midi_software_center_daw::commands::project::get_track_details,
            midi_software_center_daw::commands::project::project_create,
            midi_software_center_daw::commands::project::project_load,
            midi_software_center_daw::commands::project::project_get,
            midi_software_center_daw::commands::project::project_update,
            midi_software_center_daw::commands::project::project_delete,
            midi_software_center_daw::commands::project::project_list,
            midi_software_center_daw::commands::project::project_get_recent,
            midi_software_center_daw::commands::project::project_save_tracks,
            midi_software_center_daw::commands::project::project_load_tracks,
            midi_software_center_daw::commands::project::project_load_full,
            midi_software_center_daw::commands::project::project_delete_tracks,
            // Export commands
            midi_software_center_daw::commands::export::export_project_midi,
            // Window commands (DAW)
            midi_software_center_daw::commands::window::play_transport,
            midi_software_center_daw::commands::window::stop_transport,
            midi_software_center_daw::commands::window::pause_transport,
            midi_software_center_daw::commands::window::set_playback_position,
            midi_software_center_daw::commands::window::get_playback_state,
            midi_software_center_daw::commands::window::set_bpm,
            midi_software_center_daw::commands::window::get_bpm,
            midi_software_center_daw::commands::window::set_time_signature,
            midi_software_center_daw::commands::window::get_time_signature,
            midi_software_center_daw::commands::window::set_key_signature,
            midi_software_center_daw::commands::window::get_key_signature,
            midi_software_center_daw::commands::window::add_window_track,
            midi_software_center_daw::commands::window::remove_window_track,
            midi_software_center_daw::commands::window::get_all_window_tracks,
            midi_software_center_daw::commands::window::set_track_visible,
            midi_software_center_daw::commands::window::set_track_muted,
            midi_software_center_daw::commands::window::set_track_soloed,
            midi_software_center_daw::commands::window::get_track_info,
            midi_software_center_daw::commands::window::update_track_label,
            midi_software_center_daw::commands::window::get_mixer_state,
            midi_software_center_daw::commands::window::set_channel_volume,
            midi_software_center_daw::commands::window::set_channel_pan,
            midi_software_center_daw::commands::window::set_channel_mute,
            midi_software_center_daw::commands::window::set_channel_solo,
            midi_software_center_daw::commands::window::get_daw_state,
            midi_software_center_daw::commands::window::reset_daw_state,
            // Mixer commands (using mixer:: module for MixerState-based commands)
            midi_software_center_daw::commands::mixer::mixer_get_channels,
            midi_software_center_daw::commands::mixer::mixer_get_master,
            midi_software_center_daw::commands::mixer::mixer_get_channel,
            midi_software_center_daw::commands::mixer::mixer_add_channel,
            midi_software_center_daw::commands::mixer::mixer_remove_channel,
            midi_software_center_daw::commands::mixer::mixer_update_channel,
            midi_software_center_daw::commands::mixer::mixer_set_volume,
            midi_software_center_daw::commands::mixer::mixer_set_pan,
            midi_software_center_daw::commands::mixer::mixer_set_mute,
            midi_software_center_daw::commands::mixer::mixer_set_solo,
            midi_software_center_daw::commands::mixer::mixer_set_enabled,
            midi_software_center_daw::commands::mixer::mixer_set_transpose,
            midi_software_center_daw::commands::mixer::mixer_set_octave_shift,
            midi_software_center_daw::commands::mixer::mixer_set_velocity,
            midi_software_center_daw::commands::mixer::mixer_set_velocity_range,
            midi_software_center_daw::commands::mixer::mixer_set_velocity_scale,
            midi_software_center_daw::commands::mixer::mixer_set_velocity_offset,
            midi_software_center_daw::commands::mixer::mixer_set_pitch_bend,
            midi_software_center_daw::commands::mixer::mixer_set_pitch_bend_range,
            midi_software_center_daw::commands::mixer::mixer_set_note_range,
            midi_software_center_daw::commands::mixer::mixer_set_key_scale,
            midi_software_center_daw::commands::mixer::mixer_set_quantize_grid,
            midi_software_center_daw::commands::mixer::mixer_set_quantize_strength,
            midi_software_center_daw::commands::mixer::mixer_set_swing,
            midi_software_center_daw::commands::mixer::mixer_set_delay,
            midi_software_center_daw::commands::mixer::mixer_set_humanize,
            midi_software_center_daw::commands::mixer::mixer_set_expression,
            midi_software_center_daw::commands::mixer::mixer_set_modulation,
            midi_software_center_daw::commands::mixer::mixer_set_sustain_pedal,
            midi_software_center_daw::commands::mixer::mixer_set_cc_values,
            midi_software_center_daw::commands::mixer::mixer_set_program_change,
            midi_software_center_daw::commands::mixer::mixer_set_midi_port,
            // Master channel commands
            midi_software_center_daw::commands::mixer::mixer_set_master_volume,
            midi_software_center_daw::commands::mixer::mixer_set_master_limiter,
            midi_software_center_daw::commands::mixer::mixer_set_master_compressor,
            midi_software_center_daw::commands::mixer::mixer_set_master_enabled,
            midi_software_center_daw::commands::mixer::mixer_set_master_transpose,
            midi_software_center_daw::commands::mixer::mixer_set_master_velocity,
            midi_software_center_daw::commands::mixer::mixer_set_master_velocity_scale,
            midi_software_center_daw::commands::mixer::mixer_set_master_velocity_range,
            midi_software_center_daw::commands::mixer::mixer_set_master_velocity_offset,
            midi_software_center_daw::commands::mixer::mixer_set_master_pitch_bend,
            midi_software_center_daw::commands::mixer::mixer_set_master_note_range,
            midi_software_center_daw::commands::mixer::mixer_set_master_quantize_grid,
            midi_software_center_daw::commands::mixer::mixer_set_master_quantize_strength,
            midi_software_center_daw::commands::mixer::mixer_set_master_tempo,
            midi_software_center_daw::commands::mixer::mixer_set_master_tempo_multiplier,
            midi_software_center_daw::commands::mixer::mixer_set_master_delay,
            midi_software_center_daw::commands::mixer::mixer_set_master_swing,
            midi_software_center_daw::commands::mixer::mixer_set_master_humanize,
            midi_software_center_daw::commands::mixer::mixer_set_master_output,
            midi_software_center_daw::commands::mixer::mixer_set_master_clock_enabled,
            midi_software_center_daw::commands::mixer::mixer_set_master_transport_enabled,
            midi_software_center_daw::commands::mixer::mixer_reset_peaks,
            // Effects commands
            midi_software_center_daw::commands::mixer::mixer_add_effect,
            midi_software_center_daw::commands::mixer::mixer_remove_effect,
            midi_software_center_daw::commands::mixer::mixer_set_effect_enabled,
            midi_software_center_daw::commands::mixer::mixer_set_effect_parameter,
            midi_software_center_daw::commands::mixer::mixer_reorder_effects,
            midi_software_center_daw::commands::mixer::mixer_toggle_channel_effect,
            // Automation commands
            midi_software_center_daw::commands::automation::create_automation_lane,
            midi_software_center_daw::commands::automation::delete_automation_lane,
            midi_software_center_daw::commands::automation::add_automation_point,
            midi_software_center_daw::commands::automation::remove_automation_point,
            midi_software_center_daw::commands::automation::move_automation_point,
            midi_software_center_daw::commands::automation::set_automation_curve_type,
            midi_software_center_daw::commands::automation::get_automation_lane,
            midi_software_center_daw::commands::automation::get_track_automation,
            midi_software_center_daw::commands::automation::get_automation_value,
            midi_software_center_daw::commands::automation::clear_track_automation,
            midi_software_center_daw::commands::automation::clear_all_automation,
            // ========================================================================
            // UNDO/REDO COMMANDS
            // ========================================================================
            midi_software_center_daw::undo_redo::commands::undo,
            midi_software_center_daw::undo_redo::commands::redo,
            midi_software_center_daw::undo_redo::commands::can_undo,
            midi_software_center_daw::undo_redo::commands::can_redo,
            midi_software_center_daw::undo_redo::commands::undo_description,
            midi_software_center_daw::undo_redo::commands::redo_description,
            midi_software_center_daw::undo_redo::commands::clear_history,
            midi_software_center_daw::undo_redo::commands::undo_count,
            midi_software_center_daw::undo_redo::commands::redo_count,
            midi_software_center_daw::undo_redo::commands::undo_descriptions,
            midi_software_center_daw::undo_redo::commands::redo_descriptions,
            midi_software_center_daw::undo_redo::commands::memory_usage,
            midi_software_center_daw::undo_redo::commands::set_max_depth,
            midi_software_center_daw::undo_redo::commands::set_max_memory,
            midi_software_center_daw::undo_redo::commands::set_compression,
            // ========================================================================
            // PROFILING COMMANDS (Memory, Cache, Render Metrics)
            // ========================================================================
            midi_software_center_daw::profiling::commands::get_memory_metrics,
            midi_software_center_daw::profiling::commands::get_cache_stats,
            midi_software_center_daw::profiling::commands::cache_clear,
            midi_software_center_daw::profiling::commands::get_render_metrics,
            midi_software_center_daw::profiling::commands::record_frame,
            midi_software_center_daw::profiling::commands::reset_render_metrics,
            midi_software_center_daw::profiling::commands::get_memory_report,
            midi_software_center_daw::profiling::commands::track_allocation,
            midi_software_center_daw::profiling::commands::track_deallocation,
            // ========================================================================
            // SYSTEM COMMANDS (CPU, Memory, Audio/MIDI Devices)
            // ========================================================================
            midi_software_center_daw::commands::system::get_system_info,
            midi_software_center_daw::commands::system::get_system_settings,
            midi_software_center_daw::commands::system::update_settings,
            midi_software_center_daw::commands::system::start_system_monitoring,
            midi_software_center_daw::commands::system::stop_system_monitoring,
            midi_software_center_daw::commands::system::set_audio_device,
            midi_software_center_daw::commands::system::set_midi_device,
            midi_software_center_daw::commands::system::get_cpu_usage,
            midi_software_center_daw::commands::system::get_memory_usage,
            // ========================================================================
            // DEVICE DISCOVERY COMMANDS (Audio/MIDI Devices - no state required)
            // ========================================================================
            midi_software_center_daw::commands::settings::get_audio_devices,
            midi_software_center_daw::commands::settings::get_midi_devices,
            // ========================================================================
            // GEAR COMMANDS (MIDI Gear Profiles, CC Mappings, Programs, User Gear)
            // ========================================================================
            // Gear profiles
            midi_software_center_daw::commands::gear::gear_profiles_list,
            midi_software_center_daw::commands::gear::gear_profiles_list_by_type,
            midi_software_center_daw::commands::gear::gear_profiles_get,
            midi_software_center_daw::commands::gear::gear_profiles_search,
            midi_software_center_daw::commands::gear::gear_profiles_create,
            midi_software_center_daw::commands::gear::gear_profiles_update,
            midi_software_center_daw::commands::gear::gear_profiles_delete,
            // Gear CC mappings
            midi_software_center_daw::commands::gear::gear_cc_list,
            midi_software_center_daw::commands::gear::gear_cc_get,
            midi_software_center_daw::commands::gear::gear_cc_get_by_number,
            midi_software_center_daw::commands::gear::gear_cc_create,
            midi_software_center_daw::commands::gear::gear_cc_update,
            midi_software_center_daw::commands::gear::gear_cc_delete,
            // Gear programs
            midi_software_center_daw::commands::gear::gear_programs_list,
            midi_software_center_daw::commands::gear::gear_programs_list_by_bank,
            midi_software_center_daw::commands::gear::gear_programs_get,
            midi_software_center_daw::commands::gear::gear_programs_search,
            midi_software_center_daw::commands::gear::gear_programs_create,
            midi_software_center_daw::commands::gear::gear_programs_update,
            midi_software_center_daw::commands::gear::gear_programs_delete,
            // User gear
            midi_software_center_daw::commands::gear::user_gear_list,
            midi_software_center_daw::commands::gear::user_gear_list_favorites,
            midi_software_center_daw::commands::gear::user_gear_get,
            midi_software_center_daw::commands::gear::user_gear_add,
            midi_software_center_daw::commands::gear::user_gear_update,
            midi_software_center_daw::commands::gear::user_gear_set_favorite,
            midi_software_center_daw::commands::gear::user_gear_mark_used,
            midi_software_center_daw::commands::gear::user_gear_remove,
            midi_software_center_daw::commands::gear::user_gear_get_with_profile,
            // ========================================================================
            // PRESETS COMMANDS (Mixer Presets, Track Templates, Project Templates)
            // ========================================================================
            // Mixer presets
            midi_software_center_daw::commands::presets::mixer_presets_list,
            midi_software_center_daw::commands::presets::mixer_presets_list_by_category,
            midi_software_center_daw::commands::presets::mixer_presets_get,
            midi_software_center_daw::commands::presets::mixer_presets_search,
            midi_software_center_daw::commands::presets::mixer_presets_create,
            midi_software_center_daw::commands::presets::mixer_presets_update,
            midi_software_center_daw::commands::presets::mixer_presets_delete,
            // Track templates
            midi_software_center_daw::commands::presets::track_templates_list,
            midi_software_center_daw::commands::presets::track_templates_list_by_category,
            midi_software_center_daw::commands::presets::track_templates_list_by_type,
            midi_software_center_daw::commands::presets::track_templates_get,
            midi_software_center_daw::commands::presets::track_templates_search,
            midi_software_center_daw::commands::presets::track_templates_create,
            midi_software_center_daw::commands::presets::track_templates_update,
            midi_software_center_daw::commands::presets::track_templates_delete,
            // Project templates
            midi_software_center_daw::commands::presets::project_templates_list,
            midi_software_center_daw::commands::presets::project_templates_list_by_category,
            midi_software_center_daw::commands::presets::project_templates_get,
            midi_software_center_daw::commands::presets::project_templates_search,
            midi_software_center_daw::commands::presets::project_templates_create,
            midi_software_center_daw::commands::presets::project_templates_update,
            midi_software_center_daw::commands::presets::project_templates_delete,
            midi_software_center_daw::commands::presets::project_templates_duplicate,
            // ========================================================================
            // PREFERENCES COMMANDS (Settings, Window Layouts, Keyboard Shortcuts, Recent)
            // ========================================================================
            // App settings
            midi_software_center_daw::commands::preferences::settings_get_all,
            midi_software_center_daw::commands::preferences::settings_get_by_category,
            midi_software_center_daw::commands::preferences::settings_get,
            midi_software_center_daw::commands::preferences::settings_set_string,
            midi_software_center_daw::commands::preferences::settings_set_int,
            midi_software_center_daw::commands::preferences::settings_set_float,
            midi_software_center_daw::commands::preferences::settings_set_bool,
            midi_software_center_daw::commands::preferences::settings_set_json,
            midi_software_center_daw::commands::preferences::settings_reset,
            midi_software_center_daw::commands::preferences::settings_reset_category,
            midi_software_center_daw::commands::preferences::settings_delete,
            // Window layouts
            midi_software_center_daw::commands::preferences::layouts_list,
            midi_software_center_daw::commands::preferences::layouts_get_current,
            midi_software_center_daw::commands::preferences::layouts_get,
            midi_software_center_daw::commands::preferences::layouts_save,
            midi_software_center_daw::commands::preferences::layouts_update,
            midi_software_center_daw::commands::preferences::layouts_set_default,
            midi_software_center_daw::commands::preferences::layouts_apply,
            midi_software_center_daw::commands::preferences::layouts_delete,
            // Keyboard shortcuts
            midi_software_center_daw::commands::preferences::shortcuts_list,
            midi_software_center_daw::commands::preferences::shortcuts_list_by_category,
            midi_software_center_daw::commands::preferences::shortcuts_get_by_action,
            midi_software_center_daw::commands::preferences::shortcuts_get_by_combo,
            midi_software_center_daw::commands::preferences::shortcuts_set,
            midi_software_center_daw::commands::preferences::shortcuts_reset,
            midi_software_center_daw::commands::preferences::shortcuts_reset_all,
            midi_software_center_daw::commands::preferences::shortcuts_add,
            midi_software_center_daw::commands::preferences::shortcuts_delete,
            // Recent projects
            midi_software_center_daw::commands::preferences::recent_list,
            midi_software_center_daw::commands::preferences::recent_add,
            midi_software_center_daw::commands::preferences::recent_set_pinned,
            midi_software_center_daw::commands::preferences::recent_remove,
            midi_software_center_daw::commands::preferences::recent_clear,
            midi_software_center_daw::commands::preferences::recent_clear_all,
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
        std::env::var("SENTRY_ENVIRONMENT").ok()
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
