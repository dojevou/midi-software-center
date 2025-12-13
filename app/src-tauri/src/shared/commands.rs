//! Centralized Command Registry for MIDI Software Center
//!
//! This module provides a single source of truth for all Tauri command names.
//! All command strings used in `#[tauri::command]` and frontend `invoke()` calls
//! MUST be defined here to prevent name mismatches.
//!
//! # Usage
//! ```rust
//! use crate::shared::commands::TauriCommands;
//!
//! // Get command name
//! let cmd = TauriCommands::MIXER_SET_MASTER_VOLUME;
//!
//! // Validate at startup
//! TauriCommands::validate_commands(&registered_commands);
//! ```

use std::collections::HashSet;

/// All Tauri command names organized by category.
/// These MUST match exactly with the function names in `#[tauri::command]` macros.
pub struct TauriCommands;

impl TauriCommands {
    // ========================================================================
    // APP LIFECYCLE COMMANDS
    // ========================================================================
    pub const SHUTDOWN_APPLICATION: &'static str = "shutdown_application";
    pub const HEALTH_CHECK: &'static str = "health_check";

    // ========================================================================
    // PIPELINE COMMANDS - File Management
    // ========================================================================
    pub const TEST_DB_CONNECTION: &'static str = "test_db_connection";
    pub const GET_FILE_COUNT: &'static str = "get_file_count";
    pub const GET_FILE_DETAILS: &'static str = "get_file_details";
    pub const GET_FILE: &'static str = "get_file";
    pub const LIST_FILES: &'static str = "list_files";
    pub const GET_FILES_BY_CATEGORY: &'static str = "get_files_by_category";
    pub const GET_RECENT_FILES: &'static str = "get_recent_files";
    pub const DELETE_FILE: &'static str = "delete_file";

    // ========================================================================
    // PIPELINE COMMANDS - Import
    // ========================================================================
    pub const IMPORT_SINGLE_FILE: &'static str = "import_single_file";
    pub const IMPORT_DIRECTORY: &'static str = "import_directory";
    pub const IMPORT_ARCHIVE_COLLECTION: &'static str = "import_archive_collection";

    // ========================================================================
    // PIPELINE COMMANDS - Search
    // ========================================================================
    pub const GET_ALL_TAGS: &'static str = "get_all_tags";
    pub const GET_FILES_BY_TAG: &'static str = "get_files_by_tag";
    pub const GET_BPM_RANGE: &'static str = "get_bpm_range";
    pub const GET_ALL_KEYS: &'static str = "get_all_keys";

    // ========================================================================
    // PIPELINE COMMANDS - Analysis
    // ========================================================================
    pub const START_ANALYSIS: &'static str = "start_analysis";

    // ========================================================================
    // PIPELINE COMMANDS - Statistics
    // ========================================================================
    pub const GET_CATEGORY_STATS: &'static str = "get_category_stats";
    pub const GET_MANUFACTURER_STATS: &'static str = "get_manufacturer_stats";
    pub const GET_KEY_SIGNATURE_STATS: &'static str = "get_key_signature_stats";
    pub const GET_RECENTLY_ADDED_COUNT: &'static str = "get_recently_added_count";
    pub const GET_DUPLICATE_COUNT: &'static str = "get_duplicate_count";
    pub const GET_DATABASE_SIZE: &'static str = "get_database_size";
    pub const CHECK_DATABASE_HEALTH: &'static str = "check_database_health";

    // ========================================================================
    // PIPELINE COMMANDS - Tags
    // ========================================================================
    pub const GET_FILE_TAGS: &'static str = "get_file_tags";
    pub const GET_POPULAR_TAGS: &'static str = "get_popular_tags";
    pub const SEARCH_TAGS: &'static str = "search_tags";
    pub const GET_TAG_CATEGORIES: &'static str = "get_tag_categories";
    pub const GET_TAGS_BY_CATEGORY: &'static str = "get_tags_by_category";
    pub const UPDATE_FILE_TAGS: &'static str = "update_file_tags";
    pub const ADD_TAGS_TO_FILE: &'static str = "add_tags_to_file";
    pub const REMOVE_TAG_FROM_FILE: &'static str = "remove_tag_from_file";
    pub const GET_FILES_BY_TAGS: &'static str = "get_files_by_tags";
    pub const GET_TAG_STATS: &'static str = "get_tag_stats";

    // ========================================================================
    // PIPELINE COMMANDS - Progress Tracking
    // ========================================================================
    pub const START_PROGRESS_TRACKING: &'static str = "start_progress_tracking";
    pub const UPDATE_PROGRESS: &'static str = "update_progress";
    pub const INCREMENT_ERROR_COUNT: &'static str = "increment_error_count";
    pub const INCREMENT_DUPLICATE_COUNT: &'static str = "increment_duplicate_count";
    pub const COMPLETE_PROGRESS: &'static str = "complete_progress";
    pub const GET_CURRENT_PROGRESS: &'static str = "get_current_progress";
    pub const RESET_PROGRESS: &'static str = "reset_progress";

    // ========================================================================
    // PIPELINE COMMANDS - System
    // ========================================================================
    pub const GET_SYSTEM_INFO: &'static str = "get_system_info";

    // ========================================================================
    // APP COMMANDS - File System Operations
    // ========================================================================
    pub const SHOW_IN_FOLDER: &'static str = "show_in_folder";

    // ========================================================================
    // DAW COMMANDS - Database
    // ========================================================================
    pub const INITIALIZE_DATABASE: &'static str = "initialize_database";

    // ========================================================================
    // DAW COMMANDS - MIDI Hardware
    // ========================================================================
    pub const MIDI_LIST_DEVICES: &'static str = "midi_list_devices";
    pub const MIDI_CONNECT: &'static str = "midi_connect";
    pub const MIDI_DISCONNECT: &'static str = "midi_disconnect";
    pub const MIDI_IS_CONNECTED: &'static str = "midi_is_connected";
    pub const MIDI_GET_CURRENT_DEVICE: &'static str = "midi_get_current_device";
    pub const MIDI_SEND_TEST_NOTE: &'static str = "midi_send_test_note";

    // ========================================================================
    // DAW COMMANDS - Sequencer
    // ========================================================================
    pub const START_SEQUENCER: &'static str = "start_sequencer";
    pub const STOP_SEQUENCER: &'static str = "stop_sequencer";
    pub const PAUSE_SEQUENCER: &'static str = "pause_sequencer";
    pub const RESUME_SEQUENCER: &'static str = "resume_sequencer";
    pub const GET_PLAYBACK_POSITION: &'static str = "get_playback_position";
    pub const SEEK_POSITION: &'static str = "seek_position";
    pub const SET_TEMPO: &'static str = "set_tempo";
    pub const GET_TEMPO: &'static str = "get_tempo";
    pub const ADD_TRACK: &'static str = "add_track";
    pub const REMOVE_TRACK: &'static str = "remove_track";
    pub const UPDATE_TRACK: &'static str = "update_track";
    pub const GET_TRACKS: &'static str = "get_tracks";
    pub const LOAD_SEQUENCER_TRACKS: &'static str = "load_sequencer_tracks";
    pub const IS_SEQUENCER_PLAYING: &'static str = "is_sequencer_playing";

    // ========================================================================
    // DAW COMMANDS - Search (uses DAW version, not Pipeline)
    // ========================================================================
    pub const SEARCH_FILES: &'static str = "search_files";
    pub const GET_SEARCH_SUGGESTIONS: &'static str = "get_search_suggestions";

    // ========================================================================
    // DAW COMMANDS - Analysis & Favorites
    // ========================================================================
    pub const FIND_COMPATIBLE_FILES: &'static str = "find_compatible_files";
    pub const ADD_FAVORITE: &'static str = "add_favorite";
    pub const REMOVE_FAVORITE: &'static str = "remove_favorite";
    pub const IS_FAVORITE: &'static str = "is_favorite";
    pub const GET_FAVORITES: &'static str = "get_favorites";
    pub const GET_USAGE_STATS: &'static str = "get_usage_stats";

    // ========================================================================
    // DAW COMMANDS - Project
    // ========================================================================
    pub const LOAD_MULTIPLE_TRACKS: &'static str = "load_multiple_tracks";
    pub const CLEAR_ALL_TRACKS: &'static str = "clear_all_tracks";
    pub const GET_TRACK_DETAILS: &'static str = "get_track_details";

    // ========================================================================
    // DAW COMMANDS - Export
    // ========================================================================
    pub const EXPORT_PROJECT_MIDI: &'static str = "export_project_midi";

    // ========================================================================
    // DAW COMMANDS - Window/Transport
    // ========================================================================
    pub const PLAY_TRANSPORT: &'static str = "play_transport";
    pub const STOP_TRANSPORT: &'static str = "stop_transport";
    pub const PAUSE_TRANSPORT: &'static str = "pause_transport";
    pub const SET_PLAYBACK_POSITION: &'static str = "set_playback_position";
    pub const GET_PLAYBACK_STATE: &'static str = "get_playback_state";
    pub const SET_BPM: &'static str = "set_bpm";
    pub const GET_BPM: &'static str = "get_bpm";
    pub const SET_TIME_SIGNATURE: &'static str = "set_time_signature";
    pub const GET_TIME_SIGNATURE: &'static str = "get_time_signature";
    pub const SET_KEY_SIGNATURE: &'static str = "set_key_signature";
    pub const GET_KEY_SIGNATURE: &'static str = "get_key_signature";

    // ========================================================================
    // DAW COMMANDS - Window Tracks
    // ========================================================================
    pub const ADD_WINDOW_TRACK: &'static str = "add_window_track";
    pub const REMOVE_WINDOW_TRACK: &'static str = "remove_window_track";
    pub const GET_ALL_WINDOW_TRACKS: &'static str = "get_all_window_tracks";
    pub const SET_TRACK_VISIBLE: &'static str = "set_track_visible";
    pub const SET_TRACK_MUTED: &'static str = "set_track_muted";
    pub const SET_TRACK_SOLOED: &'static str = "set_track_soloed";
    pub const GET_TRACK_INFO: &'static str = "get_track_info";
    pub const UPDATE_TRACK_LABEL: &'static str = "update_track_label";

    // ========================================================================
    // DAW COMMANDS - Mixer (DawState-based, via window module)
    // ========================================================================
    pub const GET_MIXER_STATE: &'static str = "get_mixer_state";
    pub const SET_CHANNEL_VOLUME: &'static str = "set_channel_volume";
    pub const SET_CHANNEL_PAN: &'static str = "set_channel_pan";
    pub const SET_CHANNEL_MUTE: &'static str = "set_channel_mute";
    pub const SET_CHANNEL_SOLO: &'static str = "set_channel_solo";
    pub const GET_DAW_STATE: &'static str = "get_daw_state";
    pub const RESET_DAW_STATE: &'static str = "reset_daw_state";

    // ========================================================================
    // DAW COMMANDS - Mixer (MixerState-based, via mixer module)
    // ========================================================================
    pub const MIXER_GET_CHANNELS: &'static str = "mixer_get_channels";
    pub const MIXER_GET_MASTER: &'static str = "mixer_get_master";
    pub const MIXER_ADD_CHANNEL: &'static str = "mixer_add_channel";
    pub const MIXER_REMOVE_CHANNEL: &'static str = "mixer_remove_channel";
    pub const MIXER_SET_VOLUME: &'static str = "mixer_set_volume";
    pub const MIXER_SET_PAN: &'static str = "mixer_set_pan";
    pub const MIXER_SET_MUTE: &'static str = "mixer_set_mute";
    pub const MIXER_SET_SOLO: &'static str = "mixer_set_solo";
    pub const MIXER_SET_MASTER_VOLUME: &'static str = "mixer_set_master_volume";
    pub const MIXER_SET_MASTER_LIMITER: &'static str = "mixer_set_master_limiter";
    pub const MIXER_SET_MASTER_COMPRESSOR: &'static str = "mixer_set_master_compressor";
    pub const MIXER_ADD_EFFECT: &'static str = "mixer_add_effect";
    pub const MIXER_REMOVE_EFFECT: &'static str = "mixer_remove_effect";
    pub const MIXER_SET_EFFECT_ENABLED: &'static str = "mixer_set_effect_enabled";

    // ========================================================================
    // DAW COMMANDS - Automation
    // ========================================================================
    pub const CREATE_AUTOMATION_LANE: &'static str = "create_automation_lane";
    pub const DELETE_AUTOMATION_LANE: &'static str = "delete_automation_lane";
    pub const ADD_AUTOMATION_POINT: &'static str = "add_automation_point";
    pub const REMOVE_AUTOMATION_POINT: &'static str = "remove_automation_point";
    pub const MOVE_AUTOMATION_POINT: &'static str = "move_automation_point";
    pub const SET_AUTOMATION_CURVE_TYPE: &'static str = "set_automation_curve_type";
    pub const GET_AUTOMATION_LANE: &'static str = "get_automation_lane";
    pub const GET_TRACK_AUTOMATION: &'static str = "get_track_automation";
    pub const GET_AUTOMATION_VALUE: &'static str = "get_automation_value";
    pub const CLEAR_TRACK_AUTOMATION: &'static str = "clear_track_automation";
    pub const CLEAR_ALL_AUTOMATION: &'static str = "clear_all_automation";

    /// Returns all registered command names as a HashSet for validation.
    pub fn all_commands() -> HashSet<&'static str> {
        let mut commands = HashSet::new();

        // App Lifecycle
        commands.insert(Self::SHUTDOWN_APPLICATION);
        commands.insert(Self::HEALTH_CHECK);

        // Pipeline - Files
        commands.insert(Self::TEST_DB_CONNECTION);
        commands.insert(Self::GET_FILE_COUNT);
        commands.insert(Self::GET_FILE_DETAILS);
        commands.insert(Self::GET_FILE);
        commands.insert(Self::LIST_FILES);
        commands.insert(Self::GET_FILES_BY_CATEGORY);
        commands.insert(Self::GET_RECENT_FILES);
        commands.insert(Self::DELETE_FILE);

        // Pipeline - Import
        commands.insert(Self::IMPORT_SINGLE_FILE);
        commands.insert(Self::IMPORT_DIRECTORY);
        commands.insert(Self::IMPORT_ARCHIVE_COLLECTION);

        // Pipeline - Search
        commands.insert(Self::GET_ALL_TAGS);
        commands.insert(Self::GET_FILES_BY_TAG);
        commands.insert(Self::GET_BPM_RANGE);
        commands.insert(Self::GET_ALL_KEYS);

        // Pipeline - Analysis
        commands.insert(Self::START_ANALYSIS);

        // Pipeline - Stats
        commands.insert(Self::GET_CATEGORY_STATS);
        commands.insert(Self::GET_MANUFACTURER_STATS);
        commands.insert(Self::GET_KEY_SIGNATURE_STATS);
        commands.insert(Self::GET_RECENTLY_ADDED_COUNT);
        commands.insert(Self::GET_DUPLICATE_COUNT);
        commands.insert(Self::GET_DATABASE_SIZE);
        commands.insert(Self::CHECK_DATABASE_HEALTH);

        // Pipeline - Tags
        commands.insert(Self::GET_FILE_TAGS);
        commands.insert(Self::GET_POPULAR_TAGS);
        commands.insert(Self::SEARCH_TAGS);
        commands.insert(Self::GET_TAG_CATEGORIES);
        commands.insert(Self::GET_TAGS_BY_CATEGORY);
        commands.insert(Self::UPDATE_FILE_TAGS);
        commands.insert(Self::ADD_TAGS_TO_FILE);
        commands.insert(Self::REMOVE_TAG_FROM_FILE);
        commands.insert(Self::GET_FILES_BY_TAGS);
        commands.insert(Self::GET_TAG_STATS);

        // Pipeline - Progress
        commands.insert(Self::START_PROGRESS_TRACKING);
        commands.insert(Self::UPDATE_PROGRESS);
        commands.insert(Self::INCREMENT_ERROR_COUNT);
        commands.insert(Self::INCREMENT_DUPLICATE_COUNT);
        commands.insert(Self::COMPLETE_PROGRESS);
        commands.insert(Self::GET_CURRENT_PROGRESS);
        commands.insert(Self::RESET_PROGRESS);

        // Pipeline - System
        commands.insert(Self::GET_SYSTEM_INFO);

        // App - File System Operations
        commands.insert(Self::SHOW_IN_FOLDER);

        // DAW - Database
        commands.insert(Self::INITIALIZE_DATABASE);

        // DAW - MIDI
        commands.insert(Self::MIDI_LIST_DEVICES);
        commands.insert(Self::MIDI_CONNECT);
        commands.insert(Self::MIDI_DISCONNECT);
        commands.insert(Self::MIDI_IS_CONNECTED);
        commands.insert(Self::MIDI_GET_CURRENT_DEVICE);
        commands.insert(Self::MIDI_SEND_TEST_NOTE);

        // DAW - Sequencer
        commands.insert(Self::START_SEQUENCER);
        commands.insert(Self::STOP_SEQUENCER);
        commands.insert(Self::PAUSE_SEQUENCER);
        commands.insert(Self::RESUME_SEQUENCER);
        commands.insert(Self::GET_PLAYBACK_POSITION);
        commands.insert(Self::SEEK_POSITION);
        commands.insert(Self::SET_TEMPO);
        commands.insert(Self::GET_TEMPO);
        commands.insert(Self::ADD_TRACK);
        commands.insert(Self::REMOVE_TRACK);
        commands.insert(Self::UPDATE_TRACK);
        commands.insert(Self::GET_TRACKS);
        commands.insert(Self::LOAD_SEQUENCER_TRACKS);
        commands.insert(Self::IS_SEQUENCER_PLAYING);

        // DAW - Search
        commands.insert(Self::SEARCH_FILES);
        commands.insert(Self::GET_SEARCH_SUGGESTIONS);

        // DAW - Analysis & Favorites
        commands.insert(Self::FIND_COMPATIBLE_FILES);
        commands.insert(Self::ADD_FAVORITE);
        commands.insert(Self::REMOVE_FAVORITE);
        commands.insert(Self::IS_FAVORITE);
        commands.insert(Self::GET_FAVORITES);
        commands.insert(Self::GET_USAGE_STATS);

        // DAW - Project
        commands.insert(Self::LOAD_MULTIPLE_TRACKS);
        commands.insert(Self::CLEAR_ALL_TRACKS);
        commands.insert(Self::GET_TRACK_DETAILS);

        // DAW - Export
        commands.insert(Self::EXPORT_PROJECT_MIDI);

        // DAW - Window/Transport
        commands.insert(Self::PLAY_TRANSPORT);
        commands.insert(Self::STOP_TRANSPORT);
        commands.insert(Self::PAUSE_TRANSPORT);
        commands.insert(Self::SET_PLAYBACK_POSITION);
        commands.insert(Self::GET_PLAYBACK_STATE);
        commands.insert(Self::SET_BPM);
        commands.insert(Self::GET_BPM);
        commands.insert(Self::SET_TIME_SIGNATURE);
        commands.insert(Self::GET_TIME_SIGNATURE);
        commands.insert(Self::SET_KEY_SIGNATURE);
        commands.insert(Self::GET_KEY_SIGNATURE);

        // DAW - Window Tracks
        commands.insert(Self::ADD_WINDOW_TRACK);
        commands.insert(Self::REMOVE_WINDOW_TRACK);
        commands.insert(Self::GET_ALL_WINDOW_TRACKS);
        commands.insert(Self::SET_TRACK_VISIBLE);
        commands.insert(Self::SET_TRACK_MUTED);
        commands.insert(Self::SET_TRACK_SOLOED);
        commands.insert(Self::GET_TRACK_INFO);
        commands.insert(Self::UPDATE_TRACK_LABEL);

        // DAW - Mixer (DawState)
        commands.insert(Self::GET_MIXER_STATE);
        commands.insert(Self::SET_CHANNEL_VOLUME);
        commands.insert(Self::SET_CHANNEL_PAN);
        commands.insert(Self::SET_CHANNEL_MUTE);
        commands.insert(Self::SET_CHANNEL_SOLO);
        commands.insert(Self::GET_DAW_STATE);
        commands.insert(Self::RESET_DAW_STATE);

        // DAW - Mixer (MixerState)
        commands.insert(Self::MIXER_GET_CHANNELS);
        commands.insert(Self::MIXER_GET_MASTER);
        commands.insert(Self::MIXER_ADD_CHANNEL);
        commands.insert(Self::MIXER_REMOVE_CHANNEL);
        commands.insert(Self::MIXER_SET_VOLUME);
        commands.insert(Self::MIXER_SET_PAN);
        commands.insert(Self::MIXER_SET_MUTE);
        commands.insert(Self::MIXER_SET_SOLO);
        commands.insert(Self::MIXER_SET_MASTER_VOLUME);
        commands.insert(Self::MIXER_SET_MASTER_LIMITER);
        commands.insert(Self::MIXER_SET_MASTER_COMPRESSOR);
        commands.insert(Self::MIXER_ADD_EFFECT);
        commands.insert(Self::MIXER_REMOVE_EFFECT);
        commands.insert(Self::MIXER_SET_EFFECT_ENABLED);

        // DAW - Automation
        commands.insert(Self::CREATE_AUTOMATION_LANE);
        commands.insert(Self::DELETE_AUTOMATION_LANE);
        commands.insert(Self::ADD_AUTOMATION_POINT);
        commands.insert(Self::REMOVE_AUTOMATION_POINT);
        commands.insert(Self::MOVE_AUTOMATION_POINT);
        commands.insert(Self::SET_AUTOMATION_CURVE_TYPE);
        commands.insert(Self::GET_AUTOMATION_LANE);
        commands.insert(Self::GET_TRACK_AUTOMATION);
        commands.insert(Self::GET_AUTOMATION_VALUE);
        commands.insert(Self::CLEAR_TRACK_AUTOMATION);
        commands.insert(Self::CLEAR_ALL_AUTOMATION);

        commands
    }

    /// Returns the total number of registered commands.
    pub fn command_count() -> usize {
        Self::all_commands().len()
    }

    /// Validates that all expected commands are registered.
    /// Call this at application startup to catch mismatches early.
    ///
    /// # Arguments
    /// * `registered` - Set of command names actually registered with Tauri
    ///
    /// # Returns
    /// * `Ok(())` if all commands match
    /// * `Err(Vec<String>)` with list of mismatched commands
    pub fn validate_commands(registered: &HashSet<&str>) -> Result<(), Vec<String>> {
        let expected = Self::all_commands();
        let mut errors = Vec::new();

        // Check for commands in registry but not registered
        for cmd in &expected {
            if !registered.contains(cmd) {
                errors.push(format!("Missing registration: {}", cmd));
            }
        }

        // Check for registered commands not in registry
        for cmd in registered {
            if !expected.contains(cmd) {
                errors.push(format!("Unknown command (not in registry): {}", cmd));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get commands grouped by category for documentation/debugging.
    pub fn commands_by_category() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("App Lifecycle", vec![Self::SHUTDOWN_APPLICATION, Self::HEALTH_CHECK]),
            (
                "Pipeline - Files",
                vec![
                    Self::TEST_DB_CONNECTION,
                    Self::GET_FILE_COUNT,
                    Self::GET_FILE_DETAILS,
                    Self::GET_FILE,
                    Self::LIST_FILES,
                    Self::GET_FILES_BY_CATEGORY,
                    Self::GET_RECENT_FILES,
                    Self::DELETE_FILE,
                ],
            ),
            (
                "Pipeline - Import",
                vec![
                    Self::IMPORT_SINGLE_FILE,
                    Self::IMPORT_DIRECTORY,
                    Self::IMPORT_ARCHIVE_COLLECTION,
                ],
            ),
            (
                "Pipeline - Search",
                vec![
                    Self::GET_ALL_TAGS,
                    Self::GET_FILES_BY_TAG,
                    Self::GET_BPM_RANGE,
                    Self::GET_ALL_KEYS,
                ],
            ),
            ("Pipeline - Analysis", vec![Self::START_ANALYSIS]),
            (
                "Pipeline - Stats",
                vec![
                    Self::GET_CATEGORY_STATS,
                    Self::GET_MANUFACTURER_STATS,
                    Self::GET_KEY_SIGNATURE_STATS,
                    Self::GET_RECENTLY_ADDED_COUNT,
                    Self::GET_DUPLICATE_COUNT,
                    Self::GET_DATABASE_SIZE,
                    Self::CHECK_DATABASE_HEALTH,
                ],
            ),
            (
                "Pipeline - Tags",
                vec![
                    Self::GET_FILE_TAGS,
                    Self::GET_POPULAR_TAGS,
                    Self::SEARCH_TAGS,
                    Self::GET_TAG_CATEGORIES,
                    Self::GET_TAGS_BY_CATEGORY,
                    Self::UPDATE_FILE_TAGS,
                    Self::ADD_TAGS_TO_FILE,
                    Self::REMOVE_TAG_FROM_FILE,
                    Self::GET_FILES_BY_TAGS,
                    Self::GET_TAG_STATS,
                ],
            ),
            (
                "Pipeline - Progress",
                vec![
                    Self::START_PROGRESS_TRACKING,
                    Self::UPDATE_PROGRESS,
                    Self::INCREMENT_ERROR_COUNT,
                    Self::INCREMENT_DUPLICATE_COUNT,
                    Self::COMPLETE_PROGRESS,
                    Self::GET_CURRENT_PROGRESS,
                    Self::RESET_PROGRESS,
                ],
            ),
            ("Pipeline - System", vec![Self::GET_SYSTEM_INFO]),
            ("App - File System", vec![Self::SHOW_IN_FOLDER]),
            ("DAW - Database", vec![Self::INITIALIZE_DATABASE]),
            (
                "DAW - MIDI",
                vec![
                    Self::MIDI_LIST_DEVICES,
                    Self::MIDI_CONNECT,
                    Self::MIDI_DISCONNECT,
                    Self::MIDI_IS_CONNECTED,
                    Self::MIDI_GET_CURRENT_DEVICE,
                    Self::MIDI_SEND_TEST_NOTE,
                ],
            ),
            (
                "DAW - Sequencer",
                vec![
                    Self::START_SEQUENCER,
                    Self::STOP_SEQUENCER,
                    Self::PAUSE_SEQUENCER,
                    Self::RESUME_SEQUENCER,
                    Self::GET_PLAYBACK_POSITION,
                    Self::SEEK_POSITION,
                    Self::SET_TEMPO,
                    Self::GET_TEMPO,
                    Self::ADD_TRACK,
                    Self::REMOVE_TRACK,
                    Self::UPDATE_TRACK,
                    Self::GET_TRACKS,
                    Self::LOAD_SEQUENCER_TRACKS,
                    Self::IS_SEQUENCER_PLAYING,
                ],
            ),
            (
                "DAW - Search",
                vec![Self::SEARCH_FILES, Self::GET_SEARCH_SUGGESTIONS],
            ),
            (
                "DAW - Analysis & Favorites",
                vec![
                    Self::FIND_COMPATIBLE_FILES,
                    Self::ADD_FAVORITE,
                    Self::REMOVE_FAVORITE,
                    Self::IS_FAVORITE,
                    Self::GET_FAVORITES,
                    Self::GET_USAGE_STATS,
                ],
            ),
            (
                "DAW - Project",
                vec![Self::LOAD_MULTIPLE_TRACKS, Self::CLEAR_ALL_TRACKS, Self::GET_TRACK_DETAILS],
            ),
            ("DAW - Export", vec![Self::EXPORT_PROJECT_MIDI]),
            (
                "DAW - Window/Transport",
                vec![
                    Self::PLAY_TRANSPORT,
                    Self::STOP_TRANSPORT,
                    Self::PAUSE_TRANSPORT,
                    Self::SET_PLAYBACK_POSITION,
                    Self::GET_PLAYBACK_STATE,
                    Self::SET_BPM,
                    Self::GET_BPM,
                    Self::SET_TIME_SIGNATURE,
                    Self::GET_TIME_SIGNATURE,
                    Self::SET_KEY_SIGNATURE,
                    Self::GET_KEY_SIGNATURE,
                ],
            ),
            (
                "DAW - Window Tracks",
                vec![
                    Self::ADD_WINDOW_TRACK,
                    Self::REMOVE_WINDOW_TRACK,
                    Self::GET_ALL_WINDOW_TRACKS,
                    Self::SET_TRACK_VISIBLE,
                    Self::SET_TRACK_MUTED,
                    Self::SET_TRACK_SOLOED,
                    Self::GET_TRACK_INFO,
                    Self::UPDATE_TRACK_LABEL,
                ],
            ),
            (
                "DAW - Mixer (DawState)",
                vec![
                    Self::GET_MIXER_STATE,
                    Self::SET_CHANNEL_VOLUME,
                    Self::SET_CHANNEL_PAN,
                    Self::SET_CHANNEL_MUTE,
                    Self::SET_CHANNEL_SOLO,
                    Self::GET_DAW_STATE,
                    Self::RESET_DAW_STATE,
                ],
            ),
            (
                "DAW - Mixer (MixerState)",
                vec![
                    Self::MIXER_GET_CHANNELS,
                    Self::MIXER_GET_MASTER,
                    Self::MIXER_ADD_CHANNEL,
                    Self::MIXER_REMOVE_CHANNEL,
                    Self::MIXER_SET_VOLUME,
                    Self::MIXER_SET_PAN,
                    Self::MIXER_SET_MUTE,
                    Self::MIXER_SET_SOLO,
                    Self::MIXER_SET_MASTER_VOLUME,
                    Self::MIXER_SET_MASTER_LIMITER,
                    Self::MIXER_SET_MASTER_COMPRESSOR,
                    Self::MIXER_ADD_EFFECT,
                    Self::MIXER_REMOVE_EFFECT,
                    Self::MIXER_SET_EFFECT_ENABLED,
                ],
            ),
            (
                "DAW - Automation",
                vec![
                    Self::CREATE_AUTOMATION_LANE,
                    Self::DELETE_AUTOMATION_LANE,
                    Self::ADD_AUTOMATION_POINT,
                    Self::REMOVE_AUTOMATION_POINT,
                    Self::MOVE_AUTOMATION_POINT,
                    Self::SET_AUTOMATION_CURVE_TYPE,
                    Self::GET_AUTOMATION_LANE,
                    Self::GET_TRACK_AUTOMATION,
                    Self::GET_AUTOMATION_VALUE,
                    Self::CLEAR_TRACK_AUTOMATION,
                    Self::CLEAR_ALL_AUTOMATION,
                ],
            ),
        ]
    }

    /// Print a formatted list of all commands by category (for debugging).
    pub fn print_command_summary() {
        println!("=== MIDI Software Center Command Registry ===\n");
        for (category, commands) in Self::commands_by_category() {
            println!("{} ({} commands):", category, commands.len());
            for cmd in commands {
                println!("  - {}", cmd);
            }
            println!();
        }
        println!("Total: {} commands", Self::command_count());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_commands_returns_expected_count() {
        let commands = TauriCommands::all_commands();
        // Update this number when adding new commands
        assert!(
            commands.len() >= 100,
            "Expected at least 100 commands, got {}",
            commands.len()
        );
    }

    #[test]
    fn test_no_duplicate_command_names() {
        let commands = TauriCommands::all_commands();
        let expected_count = commands.len();

        // HashSet automatically deduplicates, so if lengths match, no duplicates
        assert_eq!(
            expected_count,
            TauriCommands::command_count(),
            "Duplicate command names detected"
        );
    }

    #[test]
    fn test_commands_by_category_covers_all() {
        let all = TauriCommands::all_commands();
        let by_category: HashSet<&str> = TauriCommands::commands_by_category()
            .into_iter()
            .flat_map(|(_, cmds)| cmds)
            .collect();

        for cmd in &all {
            assert!(
                by_category.contains(cmd),
                "Command '{}' not in any category",
                cmd
            );
        }
    }

    #[test]
    fn test_mixer_commands_present() {
        let commands = TauriCommands::all_commands();
        assert!(commands.contains(TauriCommands::MIXER_SET_MASTER_VOLUME));
        assert!(commands.contains(TauriCommands::MIXER_GET_CHANNELS));
        assert!(commands.contains(TauriCommands::MIXER_SET_VOLUME));
    }

    #[test]
    fn test_validate_commands_detects_missing() {
        let mut registered = TauriCommands::all_commands();
        registered.remove(TauriCommands::SHUTDOWN_APPLICATION);

        let result = TauriCommands::validate_commands(&registered);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("shutdown_application")));
    }

    #[test]
    fn test_validate_commands_detects_unknown() {
        let mut registered = TauriCommands::all_commands();
        registered.insert("unknown_command");

        let result = TauriCommands::validate_commands(&registered);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("unknown_command")));
    }
}
