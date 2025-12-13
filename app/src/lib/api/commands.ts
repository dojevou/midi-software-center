/**
 * Centralized Command Registry for MIDI Software Center
 *
 * This module provides a single source of truth for all Tauri command names.
 * All command strings used in `invoke()` calls MUST be defined here to prevent
 * name mismatches with the Rust backend.
 *
 * @example
 * ```typescript
 * import { Commands, CommandInvoker } from '$lib/api/commands';
 *
 * // Direct usage
 * await invoke(Commands.MIXER_SET_MASTER_VOLUME, { volume: 0.8 });
 *
 * // Type-safe wrapper
 * const invoker = new CommandInvoker();
 * await invoker.mixer.setMasterVolume(0.8);
 * ```
 */

import { invoke } from '@tauri-apps/api/core';
import type { AutomationLane, AutomationPoint, MidiIOState, MidiPort } from '$lib/types';

// ============================================================================
// COMMAND NAME CONSTANTS
// These MUST match exactly with the Rust backend TauriCommands struct
// ============================================================================

export const Commands = {
  // ==========================================================================
  // APP LIFECYCLE COMMANDS
  // ==========================================================================
  SHUTDOWN_APPLICATION: 'shutdown_application',

  // ==========================================================================
  // PIPELINE COMMANDS - File Management
  // ==========================================================================
  TEST_DB_CONNECTION: 'test_db_connection',
  GET_FILE_COUNT: 'get_file_count',
  GET_FILE_DETAILS: 'get_file_details',
  GET_FILE: 'get_file',
  LIST_FILES: 'list_files',
  GET_FILES_BY_CATEGORY: 'get_files_by_category',
  GET_RECENT_FILES: 'get_recent_files',
  DELETE_FILE: 'delete_file',

  // ==========================================================================
  // PIPELINE COMMANDS - Import
  // ==========================================================================
  IMPORT_SINGLE_FILE: 'import_single_file',
  IMPORT_DIRECTORY: 'import_directory',
  IMPORT_ARCHIVE_COLLECTION: 'import_archive_collection',

  // ==========================================================================
  // PIPELINE COMMANDS - Search
  // ==========================================================================
  GET_ALL_TAGS: 'get_all_tags',
  GET_FILES_BY_TAG: 'get_files_by_tag',
  GET_BPM_RANGE: 'get_bpm_range',
  GET_ALL_KEYS: 'get_all_keys',

  // ==========================================================================
  // PIPELINE COMMANDS - Analysis
  // ==========================================================================
  START_ANALYSIS: 'start_analysis',

  // ==========================================================================
  // PIPELINE COMMANDS - Statistics
  // ==========================================================================
  GET_CATEGORY_STATS: 'get_category_stats',
  GET_MANUFACTURER_STATS: 'get_manufacturer_stats',
  GET_KEY_SIGNATURE_STATS: 'get_key_signature_stats',
  GET_RECENTLY_ADDED_COUNT: 'get_recently_added_count',
  GET_DUPLICATE_COUNT: 'get_duplicate_count',
  GET_DATABASE_SIZE: 'get_database_size',
  CHECK_DATABASE_HEALTH: 'check_database_health',

  // ==========================================================================
  // PIPELINE COMMANDS - Tags (file-tag associations)
  // ==========================================================================
  GET_FILE_TAGS: 'get_file_tags',
  GET_POPULAR_TAGS: 'get_popular_tags',
  SEARCH_TAGS: 'search_tags',
  GET_TAG_CATEGORIES: 'get_tag_categories',
  GET_TAGS_BY_CATEGORY: 'get_tags_by_category',
  UPDATE_FILE_TAGS: 'update_file_tags',
  ADD_TAGS_TO_FILE: 'add_tags_to_file',
  REMOVE_TAG_FROM_FILE: 'remove_tag_from_file',
  GET_FILES_BY_TAGS: 'get_files_by_tags',
  GET_TAG_STATS: 'get_tag_stats',

  // ==========================================================================
  // PIPELINE COMMANDS - Ratings
  // ==========================================================================
  SET_FILE_RATING: 'set_file_rating',
  GET_FILE_RATING: 'get_file_rating',
  GET_FILES_BY_RATING: 'get_files_by_rating',

  // ==========================================================================
  // DAW COMMANDS - Tags (CRUD operations)
  // ==========================================================================
  CREATE_TAG: 'create_tag',
  UPDATE_TAG: 'update_tag',
  DELETE_TAG: 'delete_tag',
  MERGE_TAGS: 'merge_tags',
  EXPORT_TAGS_CSV: 'export_tags_csv',
  IMPORT_TAGS_CSV: 'import_tags_csv',
  GET_TAG: 'get_tag',
  GET_ALL_TAGS_DAW: 'get_all_tags',

  // ==========================================================================
  // PIPELINE COMMANDS - Progress Tracking
  // ==========================================================================
  START_PROGRESS_TRACKING: 'start_progress_tracking',
  UPDATE_PROGRESS: 'update_progress',
  INCREMENT_ERROR_COUNT: 'increment_error_count',
  INCREMENT_DUPLICATE_COUNT: 'increment_duplicate_count',
  COMPLETE_PROGRESS: 'complete_progress',
  GET_CURRENT_PROGRESS: 'get_current_progress',
  RESET_PROGRESS: 'reset_progress',

  // ==========================================================================
  // PIPELINE COMMANDS - System
  // ==========================================================================
  GET_SYSTEM_INFO: 'get_system_info',

  // ==========================================================================
  // DAW COMMANDS - Database
  // ==========================================================================
  INITIALIZE_DATABASE: 'initialize_database',

  // ==========================================================================
  // DAW COMMANDS - MIDI Hardware
  // ==========================================================================
  MIDI_LIST_DEVICES: 'midi_list_devices',
  MIDI_CONNECT: 'midi_connect',
  MIDI_DISCONNECT: 'midi_disconnect',
  MIDI_IS_CONNECTED: 'midi_is_connected',
  MIDI_GET_CURRENT_DEVICE: 'midi_get_current_device',
  MIDI_SEND_TEST_NOTE: 'midi_send_test_note',

  // ==========================================================================
  // DAW COMMANDS - Sequencer
  // ==========================================================================
  START_SEQUENCER: 'start_sequencer',
  STOP_SEQUENCER: 'stop_sequencer',
  PAUSE_SEQUENCER: 'pause_sequencer',
  RESUME_SEQUENCER: 'resume_sequencer',
  GET_PLAYBACK_POSITION: 'get_playback_position',
  SEEK_POSITION: 'seek_position',
  SET_TEMPO: 'set_tempo',
  GET_TEMPO: 'get_tempo',
  ADD_TRACK: 'add_track',
  REMOVE_TRACK: 'remove_track',
  UPDATE_TRACK: 'update_track',
  GET_TRACKS: 'get_tracks',
  LOAD_SEQUENCER_TRACKS: 'load_sequencer_tracks',
  IS_SEQUENCER_PLAYING: 'is_sequencer_playing',

  // ==========================================================================
  // DAW COMMANDS - Search
  // ==========================================================================
  SEARCH_FILES: 'search_files',
  GET_SEARCH_SUGGESTIONS: 'get_search_suggestions',

  // ==========================================================================
  // DAW COMMANDS - Analysis & Favorites
  // ==========================================================================
  GET_WAVEFORM_DATA: 'get_waveform_data',
  FIND_COMPATIBLE_FILES: 'find_compatible_files',
  ADD_FAVORITE: 'add_favorite',
  REMOVE_FAVORITE: 'remove_favorite',
  IS_FAVORITE: 'is_favorite',
  GET_FAVORITES: 'get_favorites',
  GET_USAGE_STATS: 'get_usage_stats',

  // ==========================================================================
  // DAW COMMANDS - Project
  // ==========================================================================
  LOAD_MULTIPLE_TRACKS: 'load_multiple_tracks',
  CLEAR_ALL_TRACKS: 'clear_all_tracks',
  GET_TRACK_DETAILS: 'get_track_details',
  // Project CRUD - matches backend daw/src-tauri/src/commands/project.rs
  PROJECT_CREATE: 'project_create',
  PROJECT_LOAD: 'project_load',
  PROJECT_LOAD_FROM_FILE: 'project_load_from_file',
  PROJECT_GET: 'project_get',
  PROJECT_UPDATE: 'project_update',
  PROJECT_DELETE: 'project_delete',
  PROJECT_LIST: 'project_list',
  PROJECT_GET_RECENT: 'project_get_recent',
  // Extended project commands (UI helpers)
  CREATE_NEW_PROJECT: 'project_create',  // Alias for PROJECT_CREATE
  OPEN_PROJECT_DIALOG: 'project_load',   // Alias for PROJECT_LOAD
  SAVE_PROJECT: 'project_update',        // Alias for PROJECT_UPDATE
  DUPLICATE_PROJECT: 'project_create',   // Creates copy via PROJECT_CREATE
  EXPORT_PROJECT_ARCHIVE: 'export_project_midi',  // Uses EXPORT_PROJECT_MIDI
  IMPORT_PROJECT: 'project_load',        // Uses PROJECT_LOAD
  // Legacy aliases (deprecated - use PROJECT_* versions)
  CREATE_PROJECT: 'project_create',
  DELETE_PROJECT: 'project_delete',
  LIST_PROJECTS: 'project_list',
  GET_PROJECT_INFO: 'project_get',
  // Project Tracks/Clips persistence
  PROJECT_SAVE_TRACKS: 'project_save_tracks',
  PROJECT_LOAD_TRACKS: 'project_load_tracks',
  PROJECT_LOAD_FULL: 'project_load_full',
  PROJECT_DELETE_TRACKS: 'project_delete_tracks',

  // ==========================================================================
  // DAW COMMANDS - Export
  // ==========================================================================
  EXPORT_PROJECT_MIDI: 'export_project_midi',
  EXPORT_PROJECT: 'export_project',
  EXPORT_FILE: 'export_file',
  GET_EXPORT_PROGRESS: 'get_export_progress',

  // ==========================================================================
  // DAW COMMANDS - Window/Transport
  // ==========================================================================
  PLAY_TRANSPORT: 'play_transport',
  STOP_TRANSPORT: 'stop_transport',
  PAUSE_TRANSPORT: 'pause_transport',
  SET_PLAYBACK_POSITION: 'set_playback_position',
  GET_PLAYBACK_STATE: 'get_playback_state',
  SET_BPM: 'set_bpm',
  GET_BPM: 'get_bpm',
  SET_TIME_SIGNATURE: 'set_time_signature',
  GET_TIME_SIGNATURE: 'get_time_signature',
  SET_KEY_SIGNATURE: 'set_key_signature',
  GET_KEY_SIGNATURE: 'get_key_signature',
  TOGGLE_PLAYBACK: 'toggle_playback',
  TOGGLE_RECORD: 'toggle_record',
  PLAY_FILE: 'play_file',

  // ==========================================================================
  // DAW COMMANDS - Window Tracks
  // ==========================================================================
  ADD_WINDOW_TRACK: 'add_window_track',
  REMOVE_WINDOW_TRACK: 'remove_window_track',
  GET_ALL_WINDOW_TRACKS: 'get_all_window_tracks',
  SET_TRACK_VISIBLE: 'set_track_visible',
  SET_TRACK_MUTED: 'set_track_muted',
  SET_TRACK_SOLOED: 'set_track_soloed',
  GET_TRACK_INFO: 'get_track_info',
  UPDATE_TRACK_LABEL: 'update_track_label',

  // ==========================================================================
  // DAW COMMANDS - Mixer (DawState-based, via window module)
  // ==========================================================================
  GET_MIXER_STATE: 'get_mixer_state',
  SET_CHANNEL_VOLUME: 'set_channel_volume',
  SET_CHANNEL_PAN: 'set_channel_pan',
  SET_CHANNEL_MUTE: 'set_channel_mute',
  SET_CHANNEL_SOLO: 'set_channel_solo',
  GET_DAW_STATE: 'get_daw_state',
  RESET_DAW_STATE: 'reset_daw_state',

  // ==========================================================================
  // DAW COMMANDS - Mixer (MixerState-based, via mixer module)
  // ==========================================================================
  MIXER_GET_CHANNELS: 'mixer_get_channels',
  MIXER_GET_MASTER: 'mixer_get_master',
  MIXER_ADD_CHANNEL: 'mixer_add_channel',
  MIXER_REMOVE_CHANNEL: 'mixer_remove_channel',
  MIXER_SET_VOLUME: 'mixer_set_volume',
  MIXER_SET_PAN: 'mixer_set_pan',
  MIXER_SET_MUTE: 'mixer_set_mute',
  MIXER_SET_SOLO: 'mixer_set_solo',
  MIXER_SET_MASTER_VOLUME: 'mixer_set_master_volume',
  MIXER_SET_MASTER_LIMITER: 'mixer_set_master_limiter',
  MIXER_SET_MASTER_COMPRESSOR: 'mixer_set_master_compressor',
  MIXER_ADD_EFFECT: 'mixer_add_effect',
  MIXER_REMOVE_EFFECT: 'mixer_remove_effect',
  MIXER_SET_EFFECT_ENABLED: 'mixer_set_effect_enabled',
  MIXER_SET_EFFECT_PARAMETER: 'mixer_set_effect_parameter',
  MIXER_REORDER_EFFECTS: 'mixer_reorder_effects',
  MIXER_TOGGLE_CHANNEL_EFFECT: 'mixer_toggle_channel_effect',
  MIXER_SET_MASTER_CLOCK_ENABLED: 'mixer_set_master_clock_enabled',
  MIXER_SET_MASTER_TRANSPORT_ENABLED: 'mixer_set_master_transport_enabled',
  MIXER_SET_MASTER_ENABLED: 'mixer_set_master_enabled',
  MIXER_SET_MASTER_OUTPUT: 'mixer_set_master_output',
  MIXER_RESET_PEAKS: 'mixer_reset_peaks',

  // ==========================================================================
  // DAW COMMANDS - MIDI I/O
  // ==========================================================================
  MIDI_IO_GET_STATE: 'midi_io_get_state',
  MIDI_IO_DETECT_PORTS: 'midi_io_detect_ports',
  MIDI_IO_ADD_PORT: 'midi_io_add_port',
  MIDI_IO_UPDATE_PORT: 'midi_io_update_port',
  MIDI_IO_REMOVE_PORT: 'midi_io_remove_port',
  MIDI_IO_SET_PORT_CONNECTED: 'midi_io_set_port_connected',

  // ==========================================================================
  // PIPELINE COMMANDS - Split File
  // ==========================================================================
  SPLIT_FILE: 'split_file',
  SPLIT_FILE_BATCH: 'split_file_batch',

  // ==========================================================================
  // DAW COMMANDS - Automation
  // ==========================================================================
  CREATE_AUTOMATION_LANE: 'create_automation_lane',
  DELETE_AUTOMATION_LANE: 'delete_automation_lane',
  ADD_AUTOMATION_POINT: 'add_automation_point',
  UPDATE_AUTOMATION_POINT: 'update_automation_point',
  DELETE_AUTOMATION_POINT: 'delete_automation_point',
  REMOVE_AUTOMATION_POINT: 'remove_automation_point',
  MOVE_AUTOMATION_POINT: 'move_automation_point',
  SET_AUTOMATION_CURVE_TYPE: 'set_automation_curve_type',
  GET_AUTOMATION_LANE: 'get_automation_lane',
  GET_ALL_AUTOMATION_LANES: 'get_all_automation_lanes',
  GET_AUTOMATION_POINTS_IN_RANGE: 'get_automation_points_in_range',
  SCALE_AUTOMATION_VALUES: 'scale_automation_values',
  OFFSET_AUTOMATION_VALUES: 'offset_automation_values',
  SMOOTH_AUTOMATION_VALUES: 'smooth_automation_values',
  CLEAR_AUTOMATION_RANGE: 'clear_automation_range',
  GET_TRACK_AUTOMATION: 'get_track_automation',
  GET_AUTOMATION_VALUE: 'get_automation_value',
  CLEAR_TRACK_AUTOMATION: 'clear_track_automation',
  CLEAR_ALL_AUTOMATION: 'clear_all_automation',
  ADD_AUTOMATION_LANE: 'add_automation_lane',
  DELETE_AUTOMATION_POINTS: 'delete_automation_points',
  DELETE_AUTOMATION_POINTS_BATCH: 'delete_automation_points_batch',
  // CC Recording
  START_CC_RECORDING: 'start_cc_recording',
  RECORD_CC_VALUE: 'record_cc_value',
  STOP_CC_RECORDING: 'stop_cc_recording',
  RECORD_CC_AUTOMATION_BATCH: 'record_cc_automation_batch',
  // Automation Clipboard
  COPY_AUTOMATION_POINTS: 'copy_automation_points',
  PASTE_AUTOMATION_POINTS: 'paste_automation_points',
  CUT_AUTOMATION_POINTS: 'cut_automation_points',

  // ==========================================================================
  // DAW COMMANDS - MIDI Device Management (Extended)
  // ==========================================================================
  SCAN_MIDI_DEVICES: 'scan_midi_devices',
  CONNECT_MIDI_INPUT: 'connect_midi_input',
  DISCONNECT_MIDI_INPUT: 'disconnect_midi_input',
  CONNECT_MIDI_OUTPUT: 'connect_midi_output',
  DISCONNECT_MIDI_OUTPUT: 'disconnect_midi_output',
  SEND_MIDI_MESSAGE: 'send_midi_message',

  // ==========================================================================
  // DAW COMMANDS - Velocity Editor
  // ==========================================================================
  UPDATE_NOTE_VELOCITIES: 'update_note_velocities',
  GET_NOTE_VELOCITIES: 'get_note_velocities',
  SCALE_VELOCITIES: 'scale_velocities',
  HUMANIZE_VELOCITIES: 'humanize_velocities',

  // ==========================================================================
  // DAW COMMANDS - Loop & Metronome
  // ==========================================================================
  SET_LOOP_ENABLED: 'set_loop_enabled',
  SET_LOOP_RANGE: 'set_loop_range',
  SET_METRONOME_ENABLED: 'set_metronome_enabled',
  SET_METRONOME_VOLUME: 'set_metronome_volume',
  GET_TRANSPORT_INFO: 'get_transport_info',
  DAW_SET_LOOP: 'daw_set_loop',
  DAW_SET_METRONOME: 'daw_set_metronome',
  DAW_SET_METRONOME_VOLUME: 'daw_set_metronome_volume',
  START_RECORDING: 'start_recording',

  // ==========================================================================
  // PIPELINE COMMANDS - Operations
  // ==========================================================================
  PIPELINE_IMPORT_FILES: 'pipeline_import_files',
  PIPELINE_ANALYZE_FILES: 'pipeline_analyze_files',
  PIPELINE_ARCHIVE_FILES: 'pipeline_archive_files',
  PIPELINE_GET_PROGRESS: 'pipeline_get_progress',
  PIPELINE_CANCEL: 'pipeline_cancel',

  // ==========================================================================
  // DATABASE COMMANDS
  // ==========================================================================
  DATABASE_SEARCH: 'database_search',
  DATABASE_GET_FILE_METADATA: 'database_get_file_metadata',
  DATABASE_ADD_FILE: 'database_add_file',
  DATABASE_REMOVE_FILE: 'database_remove_file',
  DATABASE_GET_STATS: 'database_get_stats',

  // ==========================================================================
  // PIANO ROLL COMMANDS
  // ==========================================================================
  GET_TRACK_NOTES: 'get_track_notes',
  ADD_NOTE: 'add_note',
  UPDATE_NOTES_BATCH: 'update_notes_batch',
  DELETE_NOTES: 'delete_notes',
  SLICE_NOTE: 'slice_note',
  STRETCH_NOTES: 'stretch_notes',
  SCALE_QUANTIZE_NOTES: 'scale_quantize_notes',
  SELECT_NOTES: 'select_notes',
  CLEAR_SELECTION: 'clear_selection',
  GET_SELECTION: 'get_selection',
  COPY_NOTES: 'copy_notes',
  PASTE_NOTES: 'paste_notes',
  CUT_NOTES: 'cut_notes',
  QUANTIZE_NOTES: 'quantize_notes',
  HUMANIZE_NOTES: 'humanize_notes',
  TRANSPOSE_NOTES: 'transpose_notes',

  // ==========================================================================
  // FAVORITES COMMANDS (Extended)
  // ==========================================================================
  GET_FOLDER_FAVORITES: 'get_folder_favorites',
  GET_FAVORITE_FOLDERS: 'get_favorite_folders',
  ADD_TO_FAVORITES: 'add_to_favorites',
  REMOVE_FROM_FAVORITES: 'remove_from_favorites',
  UPDATE_FAVORITE: 'update_favorite',
  CREATE_FAVORITE_FOLDER: 'create_favorite_folder',
  DELETE_FAVORITE_FOLDER: 'delete_favorite_folder',
  MOVE_FAVORITES_TO_FOLDER: 'move_favorites_to_folder',

  // ==========================================================================
  // SETTINGS COMMANDS
  // ==========================================================================
  GET_SETTINGS: 'get_settings',
  SAVE_SETTINGS: 'save_settings',
  RESET_SETTINGS: 'reset_settings',

  // ==========================================================================
  // AUDIO DEVICE COMMANDS
  // ==========================================================================
  AUDIO_GET_DEVICES: 'audio_get_devices',
  AUDIO_TEST_OUTPUT: 'audio_test_output',

  // ==========================================================================
  // PREFERENCES - APP SETTINGS COMMANDS
  // ==========================================================================
  SETTINGS_GET_ALL: 'settings_get_all',
  SETTINGS_GET_BY_CATEGORY: 'settings_get_by_category',
  SETTINGS_GET: 'settings_get',
  SETTINGS_SET_STRING: 'settings_set_string',
  SETTINGS_SET_INT: 'settings_set_int',
  SETTINGS_SET_FLOAT: 'settings_set_float',
  SETTINGS_SET_BOOL: 'settings_set_bool',
  SETTINGS_SET_JSON: 'settings_set_json',
  SETTINGS_RESET: 'settings_reset',
  SETTINGS_RESET_CATEGORY: 'settings_reset_category',
  SETTINGS_DELETE: 'settings_delete',

  // ==========================================================================
  // PREFERENCES - WINDOW LAYOUT COMMANDS
  // ==========================================================================
  LAYOUTS_LIST: 'layouts_list',
  LAYOUTS_GET_CURRENT: 'layouts_get_current',
  LAYOUTS_GET: 'layouts_get',
  LAYOUTS_SAVE: 'layouts_save',
  LAYOUTS_UPDATE: 'layouts_update',
  LAYOUTS_SET_DEFAULT: 'layouts_set_default',
  LAYOUTS_APPLY: 'layouts_apply',
  LAYOUTS_DELETE: 'layouts_delete',

  // ==========================================================================
  // PREFERENCES - KEYBOARD SHORTCUTS COMMANDS
  // ==========================================================================
  SHORTCUTS_LIST: 'shortcuts_list',
  SHORTCUTS_LIST_BY_CATEGORY: 'shortcuts_list_by_category',
  SHORTCUTS_GET_BY_ACTION: 'shortcuts_get_by_action',
  SHORTCUTS_GET_BY_COMBO: 'shortcuts_get_by_combo',
  SHORTCUTS_SET: 'shortcuts_set',
  SHORTCUTS_RESET: 'shortcuts_reset',
  SHORTCUTS_RESET_ALL: 'shortcuts_reset_all',
  SHORTCUTS_ADD: 'shortcuts_add',
  SHORTCUTS_DELETE: 'shortcuts_delete',

  // ==========================================================================
  // PREFERENCES - RECENT PROJECTS COMMANDS
  // ==========================================================================
  RECENT_LIST: 'recent_list',
  RECENT_ADD: 'recent_add',
  RECENT_SET_PINNED: 'recent_set_pinned',
  RECENT_REMOVE: 'recent_remove',
  RECENT_CLEAR: 'recent_clear',
  RECENT_CLEAR_ALL: 'recent_clear_all',

  // ==========================================================================
  // GEAR PROFILE COMMANDS
  // ==========================================================================
  GEAR_PROFILES_LIST: 'gear_profiles_list',
  GEAR_PROFILES_LIST_BY_TYPE: 'gear_profiles_list_by_type',
  GEAR_PROFILES_GET: 'gear_profiles_get',
  GEAR_PROFILES_SEARCH: 'gear_profiles_search',
  GEAR_PROFILES_CREATE: 'gear_profiles_create',
  GEAR_PROFILES_UPDATE: 'gear_profiles_update',
  GEAR_PROFILES_DELETE: 'gear_profiles_delete',

  // ==========================================================================
  // GEAR CC MAPPING COMMANDS
  // ==========================================================================
  GEAR_CC_LIST: 'gear_cc_list',
  GEAR_CC_GET: 'gear_cc_get',
  GEAR_CC_GET_BY_NUMBER: 'gear_cc_get_by_number',
  GEAR_CC_CREATE: 'gear_cc_create',
  GEAR_CC_UPDATE: 'gear_cc_update',
  GEAR_CC_DELETE: 'gear_cc_delete',

  // ==========================================================================
  // GEAR PROGRAM COMMANDS
  // ==========================================================================
  GEAR_PROGRAMS_LIST: 'gear_programs_list',
  GEAR_PROGRAMS_LIST_BY_BANK: 'gear_programs_list_by_bank',
  GEAR_PROGRAMS_GET: 'gear_programs_get',
  GEAR_PROGRAMS_SEARCH: 'gear_programs_search',
  GEAR_PROGRAMS_CREATE: 'gear_programs_create',
  GEAR_PROGRAMS_UPDATE: 'gear_programs_update',
  GEAR_PROGRAMS_DELETE: 'gear_programs_delete',

  // ==========================================================================
  // USER GEAR COMMANDS
  // ==========================================================================
  USER_GEAR_LIST: 'user_gear_list',
  USER_GEAR_LIST_FAVORITES: 'user_gear_list_favorites',
  USER_GEAR_GET: 'user_gear_get',
  USER_GEAR_ADD: 'user_gear_add',
  USER_GEAR_UPDATE: 'user_gear_update',
  USER_GEAR_SET_FAVORITE: 'user_gear_set_favorite',
  USER_GEAR_MARK_USED: 'user_gear_mark_used',
  USER_GEAR_REMOVE: 'user_gear_remove',
  USER_GEAR_GET_WITH_PROFILE: 'user_gear_get_with_profile',

  // ==========================================================================
  // MIXER PRESET COMMANDS
  // ==========================================================================
  MIXER_PRESETS_LIST: 'mixer_presets_list',
  MIXER_PRESETS_LIST_BY_CATEGORY: 'mixer_presets_list_by_category',
  MIXER_PRESETS_GET: 'mixer_presets_get',
  MIXER_PRESETS_SEARCH: 'mixer_presets_search',
  MIXER_PRESETS_CREATE: 'mixer_presets_create',
  MIXER_PRESETS_UPDATE: 'mixer_presets_update',
  MIXER_PRESETS_DELETE: 'mixer_presets_delete',

  // ==========================================================================
  // TRACK TEMPLATE COMMANDS
  // ==========================================================================
  TRACK_TEMPLATES_LIST: 'track_templates_list',
  TRACK_TEMPLATES_LIST_BY_CATEGORY: 'track_templates_list_by_category',
  TRACK_TEMPLATES_LIST_BY_TYPE: 'track_templates_list_by_type',
  TRACK_TEMPLATES_GET: 'track_templates_get',
  TRACK_TEMPLATES_SEARCH: 'track_templates_search',
  TRACK_TEMPLATES_CREATE: 'track_templates_create',
  TRACK_TEMPLATES_UPDATE: 'track_templates_update',
  TRACK_TEMPLATES_DELETE: 'track_templates_delete',

  // ==========================================================================
  // PROJECT TEMPLATE COMMANDS
  // ==========================================================================
  PROJECT_TEMPLATES_LIST: 'project_templates_list',
  PROJECT_TEMPLATES_LIST_BY_CATEGORY: 'project_templates_list_by_category',
  PROJECT_TEMPLATES_GET: 'project_templates_get',
  PROJECT_TEMPLATES_SEARCH: 'project_templates_search',
  PROJECT_TEMPLATES_CREATE: 'project_templates_create',
  PROJECT_TEMPLATES_UPDATE: 'project_templates_update',
  PROJECT_TEMPLATES_DELETE: 'project_templates_delete',
  PROJECT_TEMPLATES_DUPLICATE: 'project_templates_duplicate',
} as const;

// Type for command names
export type CommandName = (typeof Commands)[keyof typeof Commands];

// ============================================================================
// TYPE-SAFE COMMAND INVOKER
// ============================================================================

/**
 * Type-safe command invoker with organized namespaces.
 * Provides IDE autocompletion and type checking for all commands.
 */
export class CommandInvoker {
  // ==========================================================================
  // APP LIFECYCLE
  // ==========================================================================
  readonly app = {
    shutdown: () => invoke<void>(Commands.SHUTDOWN_APPLICATION),
  };

  // ==========================================================================
  // PIPELINE - FILES
  // ==========================================================================
  readonly files = {
    testConnection: () => invoke<boolean>(Commands.TEST_DB_CONNECTION),
    getCount: () => invoke<number>(Commands.GET_FILE_COUNT),
    getDetails: (fileId: number) => invoke<unknown>(Commands.GET_FILE_DETAILS, { fileId }),
    get: (fileId: number) => invoke<unknown>(Commands.GET_FILE, { fileId }),
    list: (limit?: number, offset?: number) =>
      invoke<unknown[]>(Commands.LIST_FILES, { limit, offset }),
    getByCategory: (category: string) =>
      invoke<unknown[]>(Commands.GET_FILES_BY_CATEGORY, { category }),
    getRecent: (limit?: number) => invoke<unknown[]>(Commands.GET_RECENT_FILES, { limit }),
    delete: (fileId: number) => invoke<void>(Commands.DELETE_FILE, { fileId }),
  };

  // ==========================================================================
  // PIPELINE - IMPORT
  // ==========================================================================
  readonly import = {
    singleFile: (filePath: string) => invoke<unknown>(Commands.IMPORT_SINGLE_FILE, { filePath }),
    directory: (dirPath: string, recursive?: boolean) =>
      invoke<unknown>(Commands.IMPORT_DIRECTORY, { dirPath, recursive }),
    archiveCollection: (archivePath: string) =>
      invoke<unknown>(Commands.IMPORT_ARCHIVE_COLLECTION, { archivePath }),
  };

  // ==========================================================================
  // PIPELINE - SEARCH
  // ==========================================================================
  readonly pipelineSearch = {
    getAllTags: () => invoke<string[]>(Commands.GET_ALL_TAGS),
    getFilesByTag: (tag: string) => invoke<unknown[]>(Commands.GET_FILES_BY_TAG, { tag }),
    getBpmRange: () => invoke<{ min: number; max: number }>(Commands.GET_BPM_RANGE),
    getAllKeys: () => invoke<string[]>(Commands.GET_ALL_KEYS),
  };

  // ==========================================================================
  // PIPELINE - ANALYSIS
  // ==========================================================================
  readonly analysis = {
    start: (fileIds?: number[]) => invoke<void>(Commands.START_ANALYSIS, { fileIds }),
  };

  // ==========================================================================
  // PIPELINE - STATISTICS
  // ==========================================================================
  readonly stats = {
    getCategory: () => invoke<unknown>(Commands.GET_CATEGORY_STATS),
    getManufacturer: () => invoke<unknown>(Commands.GET_MANUFACTURER_STATS),
    getKeySignature: () => invoke<unknown>(Commands.GET_KEY_SIGNATURE_STATS),
    getRecentlyAddedCount: () => invoke<number>(Commands.GET_RECENTLY_ADDED_COUNT),
    getDuplicateCount: () => invoke<number>(Commands.GET_DUPLICATE_COUNT),
    getDatabaseSize: () => invoke<string>(Commands.GET_DATABASE_SIZE),
    checkDatabaseHealth: () => invoke<unknown>(Commands.CHECK_DATABASE_HEALTH),
  };

  // ==========================================================================
  // PIPELINE - TAGS
  // ==========================================================================
  readonly tags = {
    getFileTags: (fileId: number) => invoke<string[]>(Commands.GET_FILE_TAGS, { fileId }),
    getPopular: (limit?: number) => invoke<unknown[]>(Commands.GET_POPULAR_TAGS, { limit }),
    search: (query: string) => invoke<unknown[]>(Commands.SEARCH_TAGS, { query }),
    getCategories: () => invoke<string[]>(Commands.GET_TAG_CATEGORIES),
    getByCategory: (category: string) =>
      invoke<unknown[]>(Commands.GET_TAGS_BY_CATEGORY, { category }),
    updateFileTags: (fileId: number, tags: string[]) =>
      invoke<void>(Commands.UPDATE_FILE_TAGS, { fileId, tags }),
    addToFile: (fileId: number, tags: string[]) =>
      invoke<void>(Commands.ADD_TAGS_TO_FILE, { fileId, tags }),
    removeFromFile: (fileId: number, tag: string) =>
      invoke<void>(Commands.REMOVE_TAG_FROM_FILE, { fileId, tag }),
    getFilesByTags: (tags: string[]) => invoke<unknown[]>(Commands.GET_FILES_BY_TAGS, { tags }),
    getStats: () => invoke<unknown>(Commands.GET_TAG_STATS),
  };

  // ==========================================================================
  // PIPELINE - RATINGS
  // ==========================================================================
  readonly ratings = {
    setRating: (fileId: number, rating: number | null) =>
      invoke<void>(Commands.SET_FILE_RATING, { file_id: fileId, rating }),
    getRating: (fileId: number) =>
      invoke<number | null>(Commands.GET_FILE_RATING, { file_id: fileId }),
    getFilesByRating: (rating: number, limit?: number, offset?: number) =>
      invoke<number[]>(Commands.GET_FILES_BY_RATING, { rating, limit, offset }),
  };

  // ==========================================================================
  // PIPELINE - PROGRESS
  // ==========================================================================
  readonly progress = {
    start: (operationType: string, totalItems: number) =>
      invoke<string>(Commands.START_PROGRESS_TRACKING, {
        operationType,
        totalItems,
      }),
    update: (progressId: string, processed: number, currentItem?: string) =>
      invoke<void>(Commands.UPDATE_PROGRESS, {
        progressId,
        processed,
        currentItem,
      }),
    incrementError: (progressId: string) =>
      invoke<void>(Commands.INCREMENT_ERROR_COUNT, { progressId }),
    incrementDuplicate: (progressId: string) =>
      invoke<void>(Commands.INCREMENT_DUPLICATE_COUNT, { progressId }),
    complete: (progressId: string) => invoke<void>(Commands.COMPLETE_PROGRESS, { progressId }),
    getCurrent: () => invoke<unknown>(Commands.GET_CURRENT_PROGRESS),
    reset: () => invoke<void>(Commands.RESET_PROGRESS),
  };

  // ==========================================================================
  // PIPELINE - SYSTEM
  // ==========================================================================
  readonly system = {
    getInfo: () => invoke<unknown>(Commands.GET_SYSTEM_INFO),
  };

  // ==========================================================================
  // DAW - DATABASE
  // ==========================================================================
  readonly database = {
    initialize: () => invoke<void>(Commands.INITIALIZE_DATABASE),
  };

  // ==========================================================================
  // DAW - MIDI HARDWARE
  // ==========================================================================
  readonly midi = {
    listDevices: () => invoke<unknown[]>(Commands.MIDI_LIST_DEVICES),
    connect: (deviceId: string) => invoke<void>(Commands.MIDI_CONNECT, { deviceId }),
    disconnect: () => invoke<void>(Commands.MIDI_DISCONNECT),
    isConnected: () => invoke<boolean>(Commands.MIDI_IS_CONNECTED),
    getCurrentDevice: () => invoke<string | null>(Commands.MIDI_GET_CURRENT_DEVICE),
    sendTestNote: (note?: number, velocity?: number, duration?: number) =>
      invoke<void>(Commands.MIDI_SEND_TEST_NOTE, { note, velocity, duration }),
  };

  // ==========================================================================
  // DAW - SEQUENCER
  // ==========================================================================
  readonly sequencer = {
    start: () => invoke<void>(Commands.START_SEQUENCER),
    stop: () => invoke<void>(Commands.STOP_SEQUENCER),
    pause: () => invoke<void>(Commands.PAUSE_SEQUENCER),
    resume: () => invoke<void>(Commands.RESUME_SEQUENCER),
    getPosition: () => invoke<number>(Commands.GET_PLAYBACK_POSITION),
    seek: (position: number) => invoke<void>(Commands.SEEK_POSITION, { position }),
    setTempo: (bpm: number) => invoke<void>(Commands.SET_TEMPO, { bpm }),
    getTempo: () => invoke<number>(Commands.GET_TEMPO),
    addTrack: (trackData: unknown) => invoke<number>(Commands.ADD_TRACK, { trackData }),
    removeTrack: (trackId: number) => invoke<void>(Commands.REMOVE_TRACK, { trackId }),
    updateTrack: (trackId: number, trackData: unknown) =>
      invoke<void>(Commands.UPDATE_TRACK, { trackId, trackData }),
    getTracks: () => invoke<unknown[]>(Commands.GET_TRACKS),
    loadTracks: (fileIds: number[]) => invoke<void>(Commands.LOAD_SEQUENCER_TRACKS, { fileIds }),
    isPlaying: () => invoke<boolean>(Commands.IS_SEQUENCER_PLAYING),
  };

  // ==========================================================================
  // DAW - SEARCH
  // ==========================================================================
  readonly search = {
    files: (filters: unknown) => invoke<unknown[]>(Commands.SEARCH_FILES, { filters }),
    getSuggestions: (query: string) => invoke<string[]>(Commands.GET_SEARCH_SUGGESTIONS, { query }),
  };

  // ==========================================================================
  // DAW - ANALYSIS & FAVORITES
  // ==========================================================================
  readonly favorites = {
    findCompatible: (fileId: number) =>
      invoke<unknown[]>(Commands.FIND_COMPATIBLE_FILES, { fileId }),
    add: (fileId: number) => invoke<void>(Commands.ADD_FAVORITE, { fileId }),
    remove: (fileId: number) => invoke<void>(Commands.REMOVE_FAVORITE, { fileId }),
    isFavorite: (fileId: number) => invoke<boolean>(Commands.IS_FAVORITE, { fileId }),
    getAll: () => invoke<unknown[]>(Commands.GET_FAVORITES),
    getUsageStats: () => invoke<unknown>(Commands.GET_USAGE_STATS),
  };

  // ==========================================================================
  // DAW - PROJECT
  // ==========================================================================
  readonly project = {
    loadMultipleTracks: (fileIds: number[]) =>
      invoke<void>(Commands.LOAD_MULTIPLE_TRACKS, { fileIds }),
    clearAllTracks: () => invoke<void>(Commands.CLEAR_ALL_TRACKS),
    getTrackDetails: (trackId: number) => invoke<unknown>(Commands.GET_TRACK_DETAILS, { trackId }),
  };

  // ==========================================================================
  // DAW - EXPORT
  // ==========================================================================
  readonly export = {
    projectMidi: (outputPath: string) => invoke<void>(Commands.EXPORT_PROJECT_MIDI, { outputPath }),
  };

  // ==========================================================================
  // DAW - TRANSPORT
  // ==========================================================================
  readonly transport = {
    play: () => invoke<void>(Commands.PLAY_TRANSPORT),
    stop: () => invoke<void>(Commands.STOP_TRANSPORT),
    pause: () => invoke<void>(Commands.PAUSE_TRANSPORT),
    setPosition: (position: number) => invoke<void>(Commands.SET_PLAYBACK_POSITION, { position }),
    getState: () => invoke<unknown>(Commands.GET_PLAYBACK_STATE),
    setBpm: (bpm: number) => invoke<void>(Commands.SET_BPM, { bpm }),
    getBpm: () => invoke<number>(Commands.GET_BPM),
    setTimeSignature: (numerator: number, denominator: number) =>
      invoke<void>(Commands.SET_TIME_SIGNATURE, { numerator, denominator }),
    getTimeSignature: () =>
      invoke<{ numerator: number; denominator: number }>(Commands.GET_TIME_SIGNATURE),
    setKeySignature: (key: string) => invoke<void>(Commands.SET_KEY_SIGNATURE, { key }),
    getKeySignature: () => invoke<string>(Commands.GET_KEY_SIGNATURE),
  };

  // ==========================================================================
  // DAW - WINDOW TRACKS
  // ==========================================================================
  readonly windowTracks = {
    add: (trackData: unknown) => invoke<number>(Commands.ADD_WINDOW_TRACK, { trackData }),
    remove: (trackId: number) => invoke<void>(Commands.REMOVE_WINDOW_TRACK, { trackId }),
    getAll: () => invoke<unknown[]>(Commands.GET_ALL_WINDOW_TRACKS),
    setVisible: (trackId: number, visible: boolean) =>
      invoke<void>(Commands.SET_TRACK_VISIBLE, { trackId, visible }),
    setMuted: (trackId: number, muted: boolean) =>
      invoke<void>(Commands.SET_TRACK_MUTED, { trackId, muted }),
    setSoloed: (trackId: number, soloed: boolean) =>
      invoke<void>(Commands.SET_TRACK_SOLOED, { trackId, soloed }),
    getInfo: (trackId: number) => invoke<unknown>(Commands.GET_TRACK_INFO, { trackId }),
    updateLabel: (trackId: number, label: string) =>
      invoke<void>(Commands.UPDATE_TRACK_LABEL, { trackId, label }),
  };

  // ==========================================================================
  // DAW - MIXER (DawState-based)
  // ==========================================================================
  readonly window = {
    getMixerState: () => invoke<unknown>(Commands.GET_MIXER_STATE),
    setChannelVolume: (trackId: number, volume: number) =>
      invoke<void>(Commands.SET_CHANNEL_VOLUME, { trackId, volume }),
    setChannelPan: (trackId: number, pan: number) =>
      invoke<void>(Commands.SET_CHANNEL_PAN, { trackId, pan }),
    setChannelMute: (trackId: number, muted: boolean) =>
      invoke<void>(Commands.SET_CHANNEL_MUTE, { trackId, muted }),
    setChannelSolo: (trackId: number, soloed: boolean) =>
      invoke<void>(Commands.SET_CHANNEL_SOLO, { trackId, soloed }),
    getDawState: () => invoke<unknown>(Commands.GET_DAW_STATE),
    resetDawState: () => invoke<void>(Commands.RESET_DAW_STATE),
  };

  // ==========================================================================
  // DAW - MIXER (MixerState-based)
  // ==========================================================================
  readonly mixer = {
    getChannels: () => invoke<unknown[]>(Commands.MIXER_GET_CHANNELS),
    getMaster: () => invoke<unknown>(Commands.MIXER_GET_MASTER),
    addChannel: (channelData: unknown) =>
      invoke<number>(Commands.MIXER_ADD_CHANNEL, { channelData }),
    removeChannel: (channelId: number) =>
      invoke<void>(Commands.MIXER_REMOVE_CHANNEL, { channelId }),
    setVolume: (channelId: number, volume: number) =>
      invoke<void>(Commands.MIXER_SET_VOLUME, { channelId, volume }),
    setPan: (channelId: number, pan: number) =>
      invoke<void>(Commands.MIXER_SET_PAN, { channelId, pan }),
    setMute: (channelId: number, muted: boolean) =>
      invoke<void>(Commands.MIXER_SET_MUTE, { channelId, muted }),
    setSolo: (channelId: number, soloed: boolean) =>
      invoke<void>(Commands.MIXER_SET_SOLO, { channelId, soloed }),
    setMasterVolume: (volume: number) => invoke<void>(Commands.MIXER_SET_MASTER_VOLUME, { volume }),
    setMasterLimiter: (enabled: boolean, threshold?: number) =>
      invoke<void>(Commands.MIXER_SET_MASTER_LIMITER, { enabled, threshold }),
    setMasterCompressor: (enabled: boolean, threshold?: number, ratio?: number) =>
      invoke<void>(Commands.MIXER_SET_MASTER_COMPRESSOR, {
        enabled,
        threshold,
        ratio,
      }),
    addEffect: (channelId: number, effectType: string, params?: unknown) =>
      invoke<number>(Commands.MIXER_ADD_EFFECT, {
        channelId,
        effectType,
        params,
      }),
    removeEffect: (channelId: number, effectId: number) =>
      invoke<void>(Commands.MIXER_REMOVE_EFFECT, { channelId, effectId }),
    setEffectEnabled: (channelId: number, effectId: number, enabled: boolean) =>
      invoke<void>(Commands.MIXER_SET_EFFECT_ENABLED, {
        channelId,
        effectId,
        enabled,
      }),
    setEffectParameter: (trackId: number, effectId: number, parameterName: string, value: number) =>
      invoke<void>(Commands.MIXER_SET_EFFECT_PARAMETER, {
        track_id: trackId,
        effect_id: effectId,
        parameter_name: parameterName,
        value,
      }),
    reorderEffects: (trackId: number, effectIds: number[]) =>
      invoke<void>(Commands.MIXER_REORDER_EFFECTS, {
        track_id: trackId,
        effect_ids: effectIds,
      }),
    toggleChannelEffect: (trackId: number, effectId: number) =>
      invoke<boolean>(Commands.MIXER_TOGGLE_CHANNEL_EFFECT, {
        track_id: trackId,
        effect_id: effectId,
      }),
    setMasterClockEnabled: (enabled: boolean) =>
      invoke<void>(Commands.MIXER_SET_MASTER_CLOCK_ENABLED, { enabled }),
    setMasterTransportEnabled: (enabled: boolean) =>
      invoke<void>(Commands.MIXER_SET_MASTER_TRANSPORT_ENABLED, { enabled }),
    setMasterEnabled: (enabled: boolean) =>
      invoke<void>(Commands.MIXER_SET_MASTER_ENABLED, { enabled }),
    setMasterOutput: (outputDevice: string, outputPort?: number) =>
      invoke<void>(Commands.MIXER_SET_MASTER_OUTPUT, { output_device: outputDevice, output_port: outputPort }),
    resetPeaks: () => invoke<void>(Commands.MIXER_RESET_PEAKS),
  };

  // ==========================================================================
  // MIDI I/O
  // ==========================================================================
  readonly midiIO = {
    getState: () => invoke<MidiIOState>(Commands.MIDI_IO_GET_STATE),
    detectPorts: () => invoke<MidiPort[]>(Commands.MIDI_IO_DETECT_PORTS),
    addPort: (name: string, direction: 'input' | 'output', deviceName?: string) =>
      invoke<MidiPort>(Commands.MIDI_IO_ADD_PORT, { name, direction, device_name: deviceName }),
    updatePort: (
      portId: number,
      options: {
        displayName?: string;
        alias?: string;
        enabled?: boolean;
        autoConnect?: boolean;
        sendClock?: boolean;
        sendTransport?: boolean;
        routeTo?: 'selected' | 'all' | 'none';
      }
    ) =>
      invoke<MidiPort>(Commands.MIDI_IO_UPDATE_PORT, {
        port_id: portId,
        display_name: options.displayName,
        alias: options.alias,
        enabled: options.enabled,
        auto_connect: options.autoConnect,
        send_clock: options.sendClock,
        send_transport: options.sendTransport,
        route_to: options.routeTo,
      }),
    removePort: (portId: number) =>
      invoke<void>(Commands.MIDI_IO_REMOVE_PORT, { port_id: portId }),
    setPortConnected: (portId: number, isConnected: boolean) =>
      invoke<void>(Commands.MIDI_IO_SET_PORT_CONNECTED, { port_id: portId, is_connected: isConnected }),
  };

  // ==========================================================================
  // PIPELINE - SPLIT FILE
  // ==========================================================================
  readonly split = {
    file: (fileId: number, outputDir: string) =>
      invoke<unknown>(Commands.SPLIT_FILE, { file_id: fileId, output_dir: outputDir }),
    batch: (fileIds: number[], outputDir: string) =>
      invoke<unknown[]>(Commands.SPLIT_FILE_BATCH, { file_ids: fileIds, output_dir: outputDir }),
  };

  // ==========================================================================
  // DAW - AUTOMATION
  // ==========================================================================
  readonly automation = {
    createLane: (trackId: number, parameter: string) =>
      invoke<AutomationLane>(Commands.CREATE_AUTOMATION_LANE, { trackId, parameter }),
    deleteLane: (laneId: number) => invoke<void>(Commands.DELETE_AUTOMATION_LANE, { laneId }),
    addPoint: (laneId: number, time: number, value: number) =>
      invoke<AutomationPoint>(Commands.ADD_AUTOMATION_POINT, { laneId, time, value }),
    removePoint: (laneId: number, pointId: number) =>
      invoke<void>(Commands.REMOVE_AUTOMATION_POINT, { laneId, pointId }),
    updatePoint: (pointId: number, tick: number, value: number) =>
      invoke<void>(Commands.UPDATE_AUTOMATION_POINT, { pointId, tick, value }),
    deletePoint: (pointId: number) => invoke<void>(Commands.DELETE_AUTOMATION_POINT, { pointId }),
    movePoint: (laneId: number, pointId: number, time: number, value: number) =>
      invoke<void>(Commands.MOVE_AUTOMATION_POINT, {
        laneId,
        pointId,
        time,
        value,
      }),
    setCurveType: (laneId: number, curveType: string) =>
      invoke<void>(Commands.SET_AUTOMATION_CURVE_TYPE, {
        laneId,
        curveType,
      }),
    getLane: (laneId: number) => invoke<AutomationLane>(Commands.GET_AUTOMATION_LANE, { laneId }),
    getAllLanes: (trackId: number) =>
      invoke<AutomationLane[]>(Commands.GET_ALL_AUTOMATION_LANES, { trackId }),
    getTrackAutomation: (trackId: number) =>
      invoke<AutomationLane[]>(Commands.GET_TRACK_AUTOMATION, { trackId }),
    getValue: (laneId: number, time: number) =>
      invoke<number>(Commands.GET_AUTOMATION_VALUE, { laneId, time }),
    clearTrack: (trackId: number) => invoke<void>(Commands.CLEAR_TRACK_AUTOMATION, { trackId }),
    clearAll: () => invoke<void>(Commands.CLEAR_ALL_AUTOMATION),
    deletePointsBatch: (trackId: number, parameterType: string, pointIds: number[]) =>
      invoke<number>(Commands.DELETE_AUTOMATION_POINTS_BATCH, {
        track_id: trackId,
        parameter_type: parameterType,
        point_ids: pointIds,
      }),
  };
}

// ============================================================================
// SINGLETON INSTANCE
// ============================================================================

/**
 * Pre-instantiated command invoker for convenience.
 * Use this for most cases instead of creating new instances.
 */
export const cmd = new CommandInvoker();

// ============================================================================
// VALIDATION UTILITIES
// ============================================================================

/**
 * Get all command names as an array (for debugging/validation).
 */
export function getAllCommandNames(): string[] {
  return Object.values(Commands);
}

/**
 * Get total number of registered commands.
 */
export function getCommandCount(): number {
  return Object.keys(Commands).length;
}

/**
 * Check if a command name exists in the registry.
 */
export function isValidCommand(name: string): boolean {
  return Object.values(Commands).includes(name as CommandName);
}

/**
 * Get commands grouped by category (for documentation).
 */
export function getCommandsByCategory(): Record<string, string[]> {
  return {
    'App Lifecycle': [Commands.SHUTDOWN_APPLICATION],
    'Pipeline - Files': [
      Commands.TEST_DB_CONNECTION,
      Commands.GET_FILE_COUNT,
      Commands.GET_FILE_DETAILS,
      Commands.GET_FILE,
      Commands.LIST_FILES,
      Commands.GET_FILES_BY_CATEGORY,
      Commands.GET_RECENT_FILES,
      Commands.DELETE_FILE,
    ],
    'Pipeline - Import': [
      Commands.IMPORT_SINGLE_FILE,
      Commands.IMPORT_DIRECTORY,
      Commands.IMPORT_ARCHIVE_COLLECTION,
    ],
    'Pipeline - Search': [
      Commands.GET_ALL_TAGS,
      Commands.GET_FILES_BY_TAG,
      Commands.GET_BPM_RANGE,
      Commands.GET_ALL_KEYS,
    ],
    'Pipeline - Analysis': [Commands.START_ANALYSIS],
    'Pipeline - Stats': [
      Commands.GET_CATEGORY_STATS,
      Commands.GET_MANUFACTURER_STATS,
      Commands.GET_KEY_SIGNATURE_STATS,
      Commands.GET_RECENTLY_ADDED_COUNT,
      Commands.GET_DUPLICATE_COUNT,
      Commands.GET_DATABASE_SIZE,
      Commands.CHECK_DATABASE_HEALTH,
    ],
    'Pipeline - Tags': [
      Commands.GET_FILE_TAGS,
      Commands.GET_POPULAR_TAGS,
      Commands.SEARCH_TAGS,
      Commands.GET_TAG_CATEGORIES,
      Commands.GET_TAGS_BY_CATEGORY,
      Commands.UPDATE_FILE_TAGS,
      Commands.ADD_TAGS_TO_FILE,
      Commands.REMOVE_TAG_FROM_FILE,
      Commands.GET_FILES_BY_TAGS,
      Commands.GET_TAG_STATS,
    ],
    'Pipeline - Ratings': [
      Commands.SET_FILE_RATING,
      Commands.GET_FILE_RATING,
      Commands.GET_FILES_BY_RATING,
    ],
    'Pipeline - Progress': [
      Commands.START_PROGRESS_TRACKING,
      Commands.UPDATE_PROGRESS,
      Commands.INCREMENT_ERROR_COUNT,
      Commands.INCREMENT_DUPLICATE_COUNT,
      Commands.COMPLETE_PROGRESS,
      Commands.GET_CURRENT_PROGRESS,
      Commands.RESET_PROGRESS,
    ],
    'Pipeline - System': [Commands.GET_SYSTEM_INFO],
    'DAW - Database': [Commands.INITIALIZE_DATABASE],
    'DAW - MIDI': [
      Commands.MIDI_LIST_DEVICES,
      Commands.MIDI_CONNECT,
      Commands.MIDI_DISCONNECT,
      Commands.MIDI_IS_CONNECTED,
      Commands.MIDI_GET_CURRENT_DEVICE,
      Commands.MIDI_SEND_TEST_NOTE,
    ],
    'DAW - Sequencer': [
      Commands.START_SEQUENCER,
      Commands.STOP_SEQUENCER,
      Commands.PAUSE_SEQUENCER,
      Commands.RESUME_SEQUENCER,
      Commands.GET_PLAYBACK_POSITION,
      Commands.SEEK_POSITION,
      Commands.SET_TEMPO,
      Commands.GET_TEMPO,
      Commands.ADD_TRACK,
      Commands.REMOVE_TRACK,
      Commands.UPDATE_TRACK,
      Commands.GET_TRACKS,
      Commands.LOAD_SEQUENCER_TRACKS,
      Commands.IS_SEQUENCER_PLAYING,
    ],
    'DAW - Search': [Commands.SEARCH_FILES, Commands.GET_SEARCH_SUGGESTIONS],
    'DAW - Favorites': [
      Commands.FIND_COMPATIBLE_FILES,
      Commands.ADD_FAVORITE,
      Commands.REMOVE_FAVORITE,
      Commands.IS_FAVORITE,
      Commands.GET_FAVORITES,
      Commands.GET_USAGE_STATS,
    ],
    'DAW - Project': [
      Commands.LOAD_MULTIPLE_TRACKS,
      Commands.CLEAR_ALL_TRACKS,
      Commands.GET_TRACK_DETAILS,
    ],
    'DAW - Export': [Commands.EXPORT_PROJECT_MIDI],
    'DAW - Transport': [
      Commands.PLAY_TRANSPORT,
      Commands.STOP_TRANSPORT,
      Commands.PAUSE_TRANSPORT,
      Commands.SET_PLAYBACK_POSITION,
      Commands.GET_PLAYBACK_STATE,
      Commands.SET_BPM,
      Commands.GET_BPM,
      Commands.SET_TIME_SIGNATURE,
      Commands.GET_TIME_SIGNATURE,
      Commands.SET_KEY_SIGNATURE,
      Commands.GET_KEY_SIGNATURE,
    ],
    'DAW - Window Tracks': [
      Commands.ADD_WINDOW_TRACK,
      Commands.REMOVE_WINDOW_TRACK,
      Commands.GET_ALL_WINDOW_TRACKS,
      Commands.SET_TRACK_VISIBLE,
      Commands.SET_TRACK_MUTED,
      Commands.SET_TRACK_SOLOED,
      Commands.GET_TRACK_INFO,
      Commands.UPDATE_TRACK_LABEL,
    ],
    'DAW - Mixer (DawState)': [
      Commands.GET_MIXER_STATE,
      Commands.SET_CHANNEL_VOLUME,
      Commands.SET_CHANNEL_PAN,
      Commands.SET_CHANNEL_MUTE,
      Commands.SET_CHANNEL_SOLO,
      Commands.GET_DAW_STATE,
      Commands.RESET_DAW_STATE,
    ],
    'DAW - Mixer (MixerState)': [
      Commands.MIXER_GET_CHANNELS,
      Commands.MIXER_GET_MASTER,
      Commands.MIXER_ADD_CHANNEL,
      Commands.MIXER_REMOVE_CHANNEL,
      Commands.MIXER_SET_VOLUME,
      Commands.MIXER_SET_PAN,
      Commands.MIXER_SET_MUTE,
      Commands.MIXER_SET_SOLO,
      Commands.MIXER_SET_MASTER_VOLUME,
      Commands.MIXER_SET_MASTER_LIMITER,
      Commands.MIXER_SET_MASTER_COMPRESSOR,
      Commands.MIXER_ADD_EFFECT,
      Commands.MIXER_REMOVE_EFFECT,
      Commands.MIXER_SET_EFFECT_ENABLED,
    ],
    'DAW - Automation': [
      Commands.CREATE_AUTOMATION_LANE,
      Commands.DELETE_AUTOMATION_LANE,
      Commands.ADD_AUTOMATION_POINT,
      Commands.REMOVE_AUTOMATION_POINT,
      Commands.MOVE_AUTOMATION_POINT,
      Commands.SET_AUTOMATION_CURVE_TYPE,
      Commands.GET_AUTOMATION_LANE,
      Commands.GET_TRACK_AUTOMATION,
      Commands.GET_AUTOMATION_VALUE,
      Commands.CLEAR_TRACK_AUTOMATION,
      Commands.CLEAR_ALL_AUTOMATION,
    ],
  };
}

/**
 * Print command summary to console (for debugging).
 */
export function printCommandSummary(): void {
  console.log('=== MIDI Software Center Command Registry ===\n');
  const byCategory = getCommandsByCategory();
  for (const [category, commands] of Object.entries(byCategory)) {
    console.log(`${category} (${commands.length} commands):`);
    commands.forEach((cmd) => console.log(`  - ${cmd}`));
    console.log();
  }
  console.log(`Total: ${getCommandCount()} commands`);
}
