// daw/src-tauri/src/main.rs
// Task-O-Matic: Main entry point for DAW application
// Purpose: Initialize app, register commands, manage state with MIDI hardware

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod automation;
mod commands;
mod core;
mod hardware;
mod midi;
mod midi_clock;
mod models;
mod notation;
mod scripting;
mod sequencer;
mod settings;
mod windows;

use commands::daw::{
    daw_add_track, daw_get_tracks, daw_get_transport_state, daw_load_midi_file, daw_pause,
    daw_play, daw_record, daw_remove_track, daw_set_bpm, daw_stop, DawState,
};
use commands::gear::GearState;
use commands::mixer::{
    mixer_add_channel, mixer_get_channels, mixer_get_master, mixer_remove_channel,
    mixer_set_master_volume, mixer_set_mute, mixer_set_pan, mixer_set_solo, mixer_set_volume,
    mixer_toggle_channel_effect, MixerState,
};
use commands::pipeline::PipelineState;
use commands::preferences::{
    AppSettingsState, KeyboardShortcutsState, RecentProjectsState, WindowLayoutState,
};
use commands::presets::PresetsState;
use commands::window::DAWState;
use commands::AppState;
// MIDI Hardware State wrappers
use hardware::HardwareState;
use midi::MidiManager;
use midi::{MidiInputState, MidiOutputSendState, MidiRecordingState, MidiThruState, PlaybackState_};
use sequencer::SequencerEngine;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    info!("Starting MIDI Software Center DAW");

    // Initialize database pool
    let db_pool = match initialize_database_pool().await {
        Ok(pool) => {
            info!("✅ Database connection pool initialized");
            Some(pool)
        },
        Err(e) => {
            warn!("⚠️ Database connection failed: {}", e);
            None
        },
    };

    // Initialize MIDI manager
    let midi_manager = Arc::new(MidiManager::new());
    info!("MIDI manager initialized");

    // Initialize sequencer engine
    let sequencer_engine = Arc::new(SequencerEngine::new(midi_manager.clone(), 120.0, 480));
    info!("Sequencer engine initialized");

    // Create app state
    let app_state = AppState { db_pool };

    // Create DAW state (commands/daw.rs)
    let daw_state = DawState::default();

    // Create Mixer state (commands/mixer.rs)
    let mixer_state = MixerState::default();

    // Create DAW Window state (commands/window.rs) - unified state for window commands
    let daw_window_state = DAWState::default();
    info!("✅ DAW window state (with mixer) initialized");

    // Create Pipeline state
    let pipeline_state = PipelineState::default();

    // Create Preferences states
    let app_settings_state = AppSettingsState::default();
    let window_layout_state = WindowLayoutState::default();
    let keyboard_shortcuts_state = KeyboardShortcutsState::default();
    let recent_projects_state = RecentProjectsState::default();
    info!("✅ Preferences states initialized");

    // Create Gear state
    let gear_state = GearState::default();
    info!("✅ Gear state initialized");

    // Create Presets state
    let presets_state = PresetsState::default();
    info!("✅ Presets state initialized");

    // Create MIDI Clock state
    let (midi_clock, _tick_rx, _message_rx) = midi_clock::MidiClock::new();
    let clock_arc = Arc::new(midi_clock);
    let midi_clock_state = commands::midi_clock::MidiClockState {
        clock: clock_arc.clone(),
        sync_manager: Arc::new(tokio::sync::RwLock::new(midi_clock::SyncManager::new(clock_arc))),
    };
    info!("✅ MIDI Clock state initialized");

    // Ableton Link state
    let link = midi::link::AbletonLink::new(120.0);
    let link_for_engine = Arc::clone(&link);
    let link_state = midi::link::LinkStateWrapper(link);
    info!("✅ Ableton Link state initialized");

    // Start Link engine for continuous sync updates (5ms output latency estimate)
    tokio::spawn(midi::link::run_link_engine(link_for_engine, 5000));

    // MIDI Learn state
    let (learn, _learn_events_rx, _param_requests_rx) = midi::learn::MidiLearn::new();
    let learn_state = midi::learn::MidiLearnState(Arc::new(learn));
    info!("✅ MIDI Learn state initialized");

    // Notation state
    let notation_state = notation::ScoreRendererState(
        std::sync::Mutex::new(notation::ScoreRenderer::new(480))
    );
    info!("✅ Notation renderer state initialized");

    // Scripting state (Lua runtime for macros and automation)
    let scripting_state = scripting::lua_runtime::ScriptingState::new();
    info!("✅ Scripting state initialized");

    // ========== MIDI Hardware Stack States ==========

    // MIDI Input state (dedicated thread for ALSA input handling)
    let midi_input_state = MidiInputState::default();
    info!("✅ MIDI Input state initialized");

    // MIDI Output state (priority queue: Realtime > High > Normal > Low)
    let midi_output_state = MidiOutputSendState::default();
    info!("✅ MIDI Output state initialized");

    // MIDI Playback state (file playback with loop, chase, seek)
    let playback_state = PlaybackState_::default();
    info!("✅ MIDI Playback state initialized");

    // MIDI Recording state (punch in/out, quantize, record modes)
    let recording_state = MidiRecordingState::default();
    info!("✅ MIDI Recording state initialized");

    // MIDI Thru state (real-time pass-through with velocity curves)
    let thru_state = MidiThruState::default();
    info!("✅ MIDI Thru state initialized");

    // Hardware state (device manager, monitor, router)
    let hardware_state = HardwareState::new();
    if let Err(e) = hardware_state.initialize().await {
        warn!("⚠️ Hardware initialization warning: {}", e);
    }
    info!("✅ Hardware state (devices, monitor, router) initialized");

    // Monitor state (wrapped for Tauri state)
    let monitor_state = hardware_state.monitor_state.clone();

    // Build and run Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .manage(midi_manager)
        .manage(sequencer_engine)
        .manage(daw_state)
        .manage(mixer_state)
        .manage(daw_window_state)
        .manage(pipeline_state)
        .manage(app_settings_state)
        .manage(window_layout_state)
        .manage(keyboard_shortcuts_state)
        .manage(recent_projects_state)
        .manage(gear_state)
        .manage(presets_state)
        .manage(midi_clock_state)
        .manage(link_state)
        .manage(learn_state)
        .manage(notation_state)
        .manage(scripting_state)
        // MIDI Hardware Stack States
        .manage(midi_input_state)
        .manage(midi_output_state)
        .manage(playback_state)
        .manage(recording_state)
        .manage(thru_state)
        .manage(hardware_state)
        .manage(monitor_state)
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
            // MIDI Clock commands
            commands::midi_clock::get_transport,
            commands::midi_clock::transport_play,
            commands::midi_clock::transport_pause,
            commands::midi_clock::transport_stop,
            commands::midi_clock::transport_continue,
            commands::midi_clock::clock_set_bpm,
            commands::midi_clock::clock_get_bpm,
            commands::midi_clock::clock_set_time_signature,
            commands::midi_clock::set_sync_mode,
            commands::midi_clock::get_sync_status,
            commands::midi_clock::set_position_bars,
            // MTC/SMPTE commands
            commands::midi_clock::clock_get_timecode,
            commands::midi_clock::clock_set_timecode,
            commands::midi_clock::clock_get_mtc_quarter_frames,
            commands::midi_clock::clock_get_song_position,
            commands::midi_clock::clock_set_song_position,
            commands::midi_clock::clock_get_sync_mode,
            commands::midi_clock::clock_get_frame_rates,
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
            commands::project::project_create,
            commands::project::project_load,
            commands::project::project_get,
            commands::project::project_update,
            commands::project::project_delete,
            commands::project::project_list,
            commands::project::project_get_recent,
            // Export commands
            commands::export::export_project_midi,
            // Window commands
            commands::window::play_transport,
            commands::window::stop_transport,
            commands::window::pause_transport,
            commands::window::set_playback_position,
            commands::window::get_playback_state,
            commands::window::set_bpm,
            commands::window::get_bpm,
            commands::window::set_time_signature,
            commands::window::get_time_signature,
            commands::window::set_key_signature,
            commands::window::get_key_signature,
            commands::window::add_window_track,
            commands::window::remove_window_track,
            commands::window::get_all_window_tracks,
            commands::window::set_track_visible,
            commands::window::set_track_muted,
            commands::window::set_track_soloed,
            commands::window::get_track_info,
            commands::window::update_track_label,
            commands::window::get_mixer_state,
            commands::window::set_channel_volume,
            commands::window::set_channel_pan,
            commands::window::set_channel_mute,
            commands::window::set_channel_solo,
            commands::window::get_daw_state,
            commands::window::reset_daw_state,
            // Automation commands
            commands::automation::create_automation_lane,
            commands::automation::delete_automation_lane,
            commands::automation::add_automation_point,
            commands::automation::remove_automation_point,
            commands::automation::move_automation_point,
            commands::automation::set_automation_curve_type,
            commands::automation::get_automation_lane,
            commands::automation::get_track_automation,
            commands::automation::get_automation_value,
            commands::automation::clear_track_automation,
            commands::automation::clear_all_automation,
            commands::automation::delete_automation_points_batch,
            // DAW Commands
            daw_play,
            daw_pause,
            daw_stop,
            daw_record,
            daw_set_bpm,
            daw_get_transport_state,
            daw_get_tracks,
            daw_add_track,
            daw_remove_track,
            daw_load_midi_file,
            // Mixer Commands
            mixer_get_channels,
            mixer_get_master,
            mixer_add_channel,
            mixer_remove_channel,
            mixer_set_volume,
            mixer_set_pan,
            mixer_set_mute,
            mixer_set_solo,
            mixer_set_master_volume,
            mixer_toggle_channel_effect,
            // Database Commands
            commands::database::database_search,
            commands::database::database_get_file_metadata,
            commands::database::database_add_file,
            commands::database::database_remove_file,
            commands::database::database_get_stats,
            // Pipeline Commands
            commands::pipeline::pipeline_import_files,
            commands::pipeline::pipeline_analyze_files,
            commands::pipeline::pipeline_archive_files,
            commands::pipeline::pipeline_get_progress,
            commands::pipeline::pipeline_cancel,
            // Tag Commands
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::merge_tags,
            commands::tags::export_tags_csv,
            commands::tags::import_tags_csv,
            commands::tags::get_tag,
            commands::tags::get_all_tags,
            // Preferences - Settings Commands
            commands::preferences::settings_get_all,
            commands::preferences::settings_get_by_category,
            commands::preferences::settings_get,
            commands::preferences::settings_set_string,
            commands::preferences::settings_set_int,
            commands::preferences::settings_set_float,
            commands::preferences::settings_set_bool,
            commands::preferences::settings_set_json,
            commands::preferences::settings_reset,
            commands::preferences::settings_reset_category,
            commands::preferences::settings_delete,
            // Preferences - Window Layout Commands
            commands::preferences::layouts_list,
            commands::preferences::layouts_get_current,
            commands::preferences::layouts_get,
            commands::preferences::layouts_save,
            commands::preferences::layouts_update,
            commands::preferences::layouts_set_default,
            commands::preferences::layouts_apply,
            commands::preferences::layouts_delete,
            // Preferences - Keyboard Shortcuts Commands
            commands::preferences::shortcuts_list,
            commands::preferences::shortcuts_list_by_category,
            commands::preferences::shortcuts_get_by_action,
            commands::preferences::shortcuts_get_by_combo,
            commands::preferences::shortcuts_set,
            commands::preferences::shortcuts_reset,
            commands::preferences::shortcuts_reset_all,
            commands::preferences::shortcuts_add,
            commands::preferences::shortcuts_delete,
            // Preferences - Recent Projects Commands
            commands::preferences::recent_list,
            commands::preferences::recent_add,
            commands::preferences::recent_set_pinned,
            commands::preferences::recent_remove,
            commands::preferences::recent_clear,
            commands::preferences::recent_clear_all,
            // Gear Profile Commands
            commands::gear::gear_profiles_list,
            commands::gear::gear_profiles_list_by_type,
            commands::gear::gear_profiles_get,
            commands::gear::gear_profiles_search,
            commands::gear::gear_profiles_create,
            commands::gear::gear_profiles_update,
            commands::gear::gear_profiles_delete,
            // Gear CC Mapping Commands
            commands::gear::gear_cc_list,
            commands::gear::gear_cc_get,
            commands::gear::gear_cc_get_by_number,
            commands::gear::gear_cc_create,
            commands::gear::gear_cc_update,
            commands::gear::gear_cc_delete,
            // Gear Program Commands
            commands::gear::gear_programs_list,
            commands::gear::gear_programs_list_by_bank,
            commands::gear::gear_programs_get,
            commands::gear::gear_programs_search,
            commands::gear::gear_programs_create,
            commands::gear::gear_programs_update,
            commands::gear::gear_programs_delete,
            // User Gear Commands
            commands::gear::user_gear_list,
            commands::gear::user_gear_list_favorites,
            commands::gear::user_gear_get,
            commands::gear::user_gear_add,
            commands::gear::user_gear_update,
            commands::gear::user_gear_set_favorite,
            commands::gear::user_gear_mark_used,
            commands::gear::user_gear_remove,
            commands::gear::user_gear_get_with_profile,
            // Mixer Preset Commands
            commands::presets::mixer_presets_list,
            commands::presets::mixer_presets_list_by_category,
            commands::presets::mixer_presets_get,
            commands::presets::mixer_presets_search,
            commands::presets::mixer_presets_create,
            commands::presets::mixer_presets_update,
            commands::presets::mixer_presets_delete,
            // Track Template Commands
            commands::presets::track_templates_list,
            commands::presets::track_templates_list_by_category,
            commands::presets::track_templates_list_by_type,
            commands::presets::track_templates_get,
            commands::presets::track_templates_search,
            commands::presets::track_templates_create,
            commands::presets::track_templates_update,
            commands::presets::track_templates_delete,
            // Project Template Commands
            commands::presets::project_templates_list,
            commands::presets::project_templates_list_by_category,
            commands::presets::project_templates_get,
            commands::presets::project_templates_search,
            commands::presets::project_templates_create,
            commands::presets::project_templates_update,
            commands::presets::project_templates_delete,
            commands::presets::project_templates_duplicate,
            // System Commands
            commands::system::log_frontend_error,
            // Logging Commands
            commands::logging::get_recent_logs,
            commands::logging::get_performance_metrics,
            commands::logging::clear_logs,
            commands::logging::export_logs,
            commands::logging::subscribe_to_logs,
            // Ableton Link commands
            midi::link::link_enable,
            midi::link::link_is_enabled,
            midi::link::link_set_tempo,
            midi::link::link_get_tempo,
            midi::link::link_set_quantum,
            midi::link::link_start,
            midi::link::link_stop,
            midi::link::link_get_state,
            midi::link::link_num_peers,
            midi::link::link_set_start_stop_sync,
            midi::link::link_get_beat,
            midi::link::link_get_phase,
            midi::link::link_request_beat_at_time,
            midi::link::link_force_beat_at_time,
            midi::link::link_update,
            midi::link::link_subscribe,
            // MIDI Learn commands
            midi::learn::learn_start,
            midi::learn::learn_cancel,
            midi::learn::learn_is_active,
            midi::learn::learn_list_mappings,
            midi::learn::learn_remove_mapping,
            midi::learn::learn_export_mappings,
            midi::learn::learn_import_mappings,
            midi::learn::learn_add_mapping,
            midi::learn::learn_update_mapping,
            midi::learn::learn_clear_mappings,
            midi::learn::learn_process_midi,
            midi::learn::learn_scale_value,
            midi::learn::learn_subscribe,
            // ========== MIDI Hardware Stack Commands ==========
            // MIDI Input commands (6 commands)
            midi::input::list_midi_inputs,
            midi::input::open_midi_input,
            midi::input::close_midi_input,
            midi::input::get_midi_input_status,
            midi::input::get_all_midi_input_statuses,
            midi::input::subscribe_midi_input,
            // MIDI Output commands (18 commands)
            midi::output::midi_output_list_devices,
            midi::output::midi_output_open_device,
            midi::output::midi_output_close_device,
            midi::output::midi_send_note,
            midi::output::midi_send_cc,
            midi::output::midi_send_program,
            midi::output::midi_send_pitch_bend,
            midi::output::midi_send_aftertouch,
            midi::output::midi_send_sysex,
            midi::output::midi_send_bank_select,
            midi::output::midi_send_raw,
            midi::output::midi_send_scheduled,
            midi::output::midi_send_all_notes_off,
            midi::output::midi_send_all_sounds_off,
            midi::output::midi_send_reset_all_controllers,
            midi::output::midi_send_panic,
            midi::output::midi_output_get_status,
            midi::output::midi_output_get_all_statuses,
            // MIDI Playback commands (17 commands)
            midi::playback::play_midi_file,
            midi::playback::stop_playback,
            midi::playback::pause_playback,
            midi::playback::resume_playback,
            midi::playback::set_loop_region,
            midi::playback::clear_loop,
            midi::playback::enable_loop,
            midi::playback::enable_chase,
            midi::playback::set_playback_outputs,
            midi::playback::add_playback_output,
            midi::playback::remove_playback_output,
            midi::playback::seek_to_position,
            midi::playback::get_midi_playback_position,
            midi::playback::get_midi_playback_state,
            midi::playback::set_playback_bpm,
            midi::playback::get_playback_bpm,
            midi::playback::list_playback_outputs,
            // MIDI Recording commands (16 commands)
            midi::recording::start_midi_recording,
            midi::recording::stop_midi_recording,
            midi::recording::set_quantize_value,
            midi::recording::get_quantize_value,
            midi::recording::set_record_mode,
            midi::recording::get_record_mode,
            midi::recording::set_punch_in,
            midi::recording::set_punch_out,
            midi::recording::clear_punch_points,
            midi::recording::get_recorded_events,
            midi::recording::clear_recording,
            midi::recording::get_recording_status,
            midi::recording::set_recording_tempo,
            midi::recording::set_recording_ppq,
            midi::recording::set_recording_time_signature,
            midi::recording::update_recording_position,
            // MIDI Thru commands (16 commands)
            midi::thru::enable_midi_thru,
            midi::thru::disable_midi_thru,
            midi::thru::set_thru_output,
            midi::thru::set_thru_channel,
            midi::thru::set_thru_mode,
            midi::thru::set_thru_velocity_curve,
            midi::thru::set_thru_transpose,
            midi::thru::set_thru_latency_compensation,
            midi::thru::get_thru_status,
            midi::thru::get_thru_config,
            midi::thru::set_thru_config,
            midi::thru::is_thru_enabled,
            midi::thru::reset_thru_stats,
            midi::thru::process_thru_message,
            midi::thru::subscribe_midi_thru,
            // Hardware Monitor commands (4 commands)
            hardware::midi_monitor::start_monitoring,
            hardware::midi_monitor::stop_monitoring,
            hardware::midi_monitor::clear_events,
            hardware::midi_monitor::get_events,
            // Hardware Device commands (5 commands)
            hardware::device_manager::list_devices,
            hardware::device_manager::connect_device,
            hardware::device_manager::disconnect_device,
            hardware::device_manager::get_device_info,
            hardware::device_manager::set_device_mapping,
            // Hardware Router commands (6 commands)
            hardware::midi_router::create_route,
            hardware::midi_router::delete_route,
            hardware::midi_router::enable_route,
            hardware::midi_router::disable_route,
            hardware::midi_router::get_all_routes,
            hardware::midi_router::test_route,
            // Notation commands (enhanced with velocity, voice, ties, rests, key name)
            notation::render_score_svg,
            notation::export_musicxml,
            notation::set_quantize_level,
            notation::get_quantize_levels,
            // Scripting commands (persistent LuaRuntime with full MIDI/transport API)
            scripting::lua_runtime::scripting_load_script,
            scripting::lua_runtime::scripting_unload_script,
            scripting::lua_runtime::scripting_list_scripts,
            scripting::lua_runtime::scripting_run_function,
            scripting::lua_runtime::scripting_get_example_scripts,
            scripting::lua_runtime::scripting_process_midi,
            scripting::lua_runtime::scripting_set_enabled,
            scripting::lua_runtime::scripting_get_script,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
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
        .connect(&database_url)
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
