// API wrapper for Tauri backend commands
// Provides type-safe interface to DAW functionality
// All types match Rust backend models for proper serialization

import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TYPE DEFINITIONS (matching Rust backend)
// ============================================================================

/**
 * Track in the sequencer
 * Matches: src-tauri/src/models/sequencer.rs:14
 */
export interface Track {
  id: number;          // i32
  name: string;
  file_id: number;     // i32
  channel: number;     // u8
  muted: boolean;      // NOT "mute"
  solo: boolean;
  volume: number;      // u8 (0-127)
  pan: number;         // u8 (0-127, 64 = center)
  color: string;       // Hex color
  level?: number;      // UI-only: Current audio level (0-100) for VU meter
}

/**
 * Track properties for partial updates
 * Matches: src-tauri/src/models/sequencer.rs:36
 */
export interface TrackProperties {
  name?: string;
  muted?: boolean;
  solo?: boolean;
  volume?: number;
  pan?: number;
  color?: string;
}

/**
 * Playback position in the sequencer
 * Matches: src-tauri/src/models/sequencer.rs:53
 */
export interface PlaybackPosition {
  current_tick: number;   // u64
  current_bar: number;    // u32
  current_beat: number;   // u32
}

/**
 * MIDI device information
 * Matches: src-tauri/src/models/midi.rs:13
 */
export interface MidiDevice {
  name: string;
  manufacturer?: string | null;
}

/**
 * MIDI event type
 * Matches: src-tauri/src/models/midi.rs:24
 */
export type MidiEventType =
  | 'NoteOn'
  | 'NoteOff'
  | 'ControlChange'
  | 'ProgramChange'
  | 'PitchBend'
  | 'Aftertouch';

/**
 * MIDI event
 * Matches: src-tauri/src/models/midi.rs:38
 */
export interface MidiEvent {
  event_type: MidiEventType;
  tick: number;        // u64
  channel: number;     // u8
  note?: number;       // u8
  velocity?: number;   // u8
  controller?: number; // u8
  value?: number;      // u8
  program?: number;    // u8
}

/**
 * MIDI pattern with events and timing
 * Matches: src-tauri/src/models/midi.rs:75
 */
export interface MidiPattern {
  events: MidiEvent[];
  ticks_per_quarter_note: number;  // u16
  total_ticks: number;              // u64
}

/**
 * Search filters for querying MIDI files
 * Matches: src-tauri/src/models/search.rs:14
 */
export interface SearchFilters {
  min_bpm?: number;
  max_bpm?: number;
  key_signature?: string;
  time_signature?: string;
  category?: string;
  min_notes?: number;
  max_notes?: number;
  min_duration?: number;
  max_duration?: number;
  instruments?: string[];
  search_text?: string;
  sort_by?: string;
  sort_desc?: boolean;
  limit?: number;
  offset?: number;
}

/**
 * Search response with files and total count
 * Matches: src-tauri/src/models/search.rs:53
 */
export interface SearchResponse {
  files: FileDetails[];
  total: number;
}

/**
 * Alias for SearchResponse (for backwards compatibility)
 */
export type SearchResults = SearchResponse;

/**
 * File details from database
 * Matches: src-tauri/src/models/midi_file.rs:60
 */
export interface FileDetails {
  id: number;                    // i64
  file_name: string;             // filename (serialized as file_name)
  file_path: string;             // filepath (serialized as file_path)
  file_size: number;             // file_size_bytes (serialized as file_size)
  bpm?: number;                  // Option<f64>
  key?: string;                  // key_signature (serialized as "key")
  time_signature?: string;
  duration_seconds?: number;
  total_notes?: number;
  category?: string;             // primary_category (serialized as "category")
  parent_folder?: string;
  created_at: string;            // DateTime<Utc> (ISO 8601 string)
  is_favorite: boolean;
  tags: string[];
  manufacturer?: string;
  collection?: string;           // collection_name (serialized as "collection")
  track_count: number;           // i16
  has_notes: boolean;
  has_drums?: boolean;
}

/**
 * Compatible file result
 */
export interface CompatibleFile {
  file_id: number;
  file_name: string;
  compatibility_score: number;
  key_compatibility: number;
  bpm_compatibility: number;
}

// ============================================================================
// MIDI COMMANDS (src-tauri/src/commands/midi.rs)
// ============================================================================

/**
 * List all available MIDI output devices
 */
export const midiListDevices = (): Promise<MidiDevice[]> =>
  invoke('midi_list_devices');

/**
 * Connect to a specific MIDI device by name
 */
export const midiConnect = (deviceName: string): Promise<void> =>
  invoke('midi_connect', { device_name: deviceName });

/**
 * Disconnect from current MIDI device
 */
export const midiDisconnect = (): Promise<void> =>
  invoke('midi_disconnect');

/**
 * Check if MIDI device is currently connected
 */
export const midiIsConnected = (): Promise<boolean> =>
  invoke('midi_is_connected');

/**
 * Get current MIDI device info (null if not connected)
 */
export const midiGetCurrentDevice = (): Promise<MidiDevice | null> =>
  invoke('midi_get_current_device');

/**
 * Send a test note to verify MIDI connection
 * @param channel - MIDI channel (0-15)
 * @param note - Note number (0-127)
 * @param velocity - Note velocity (0-127)
 */
export const midiSendTestNote = (
  channel: number,
  note: number,
  velocity: number
): Promise<void> =>
  invoke('midi_send_test_note', { channel, note, velocity });

// ============================================================================
// SEQUENCER COMMANDS (src-tauri/src/commands/sequencer.rs)
// ============================================================================

/**
 * Start sequencer playback
 */
export const startSequencer = (): Promise<void> =>
  invoke('start_sequencer');

/**
 * Stop sequencer playback (resets position)
 */
export const stopSequencer = (): Promise<void> =>
  invoke('stop_sequencer');

/**
 * Pause sequencer playback (maintains position)
 */
export const pauseSequencer = (): Promise<void> =>
  invoke('pause_sequencer');

/**
 * Resume sequencer playback from paused state
 */
export const resumeSequencer = (): Promise<void> =>
  invoke('resume_sequencer');

/**
 * Get current playback position
 */
export const getPlaybackPosition = (): Promise<PlaybackPosition> =>
  invoke('get_playback_position');

/**
 * Seek to a specific position
 * @param bar - Bar number (0-indexed)
 * @param beat - Beat within bar (0-indexed)
 */
export const seekPosition = (bar: number, beat: number): Promise<void> =>
  invoke('seek_position', { bar, beat });

/**
 * Set global tempo (BPM)
 */
export const setTempo = (bpm: number): Promise<void> =>
  invoke('set_tempo', { bpm });

/**
 * Get current tempo
 */
export const getTempo = (): Promise<number> =>
  invoke('get_tempo');

/**
 * Add a track to the sequencer
 * @param fileId - Database ID of the MIDI file
 * @param channel - MIDI channel (0-15)
 */
export const addTrack = (fileId: number, channel: number): Promise<Track> =>
  invoke('add_track', { file_id: fileId, channel });

/**
 * Remove a track from the sequencer
 * @param trackId - Track ID to remove
 */
export const removeTrack = (trackId: number): Promise<void> =>
  invoke('remove_track', { track_id: trackId });

/**
 * Update track properties (mute, solo, volume, pan)
 * @param trackId - Track ID to update
 * @param properties - Properties to update
 */
export const updateTrack = (
  trackId: number,
  properties: TrackProperties
): Promise<void> =>
  invoke('update_track', { track_id: trackId, properties });

/**
 * Get all tracks in current project
 */
export const getTracks = (): Promise<Track[]> =>
  invoke('get_tracks');

/**
 * Load tracks into sequencer and prepare for playback
 */
export const loadSequencerTracks = (): Promise<void> =>
  invoke('load_sequencer_tracks');

/**
 * Check if sequencer is currently playing
 */
export const isSequencerPlaying = (): Promise<boolean> =>
  invoke('is_sequencer_playing');

// ============================================================================
// SEARCH & FILE COMMANDS (src-tauri/src/commands/search.rs)
// ============================================================================

/**
 * Search for MIDI files with filters
 */
export const searchFiles = (filters: SearchFilters): Promise<SearchResponse> =>
  invoke('search_files', { filters });

/**
 * Get file details by ID
 */
export const getFileDetails = (fileId: number): Promise<FileDetails> =>
  invoke('get_file_details', { file_id: fileId });

/**
 * Load MIDI pattern from file
 */
export const loadMidiPattern = (fileId: number): Promise<any> =>
  invoke('load_midi_pattern', { file_id: fileId });

/**
 * Load multiple tracks into sequencer
 */
export const loadMultipleTracks = (fileIds: number[]): Promise<void> =>
  invoke('load_multiple_tracks', { file_ids: fileIds });

// ============================================================================
// FAVORITES COMMANDS (src-tauri/src/commands/favorites.rs)
// ============================================================================

/**
 * Get all favorite files
 */
export const getFavorites = (): Promise<FileDetails[]> =>
  invoke('get_favorites');

/**
 * Add a file to favorites
 */
export const addFavorite = (fileId: number): Promise<void> =>
  invoke('add_favorite', { file_id: fileId });

/**
 * Remove a file from favorites
 */
export const removeFavorite = (fileId: number): Promise<void> =>
  invoke('remove_favorite', { file_id: fileId });

// ============================================================================
// COMPATIBILITY COMMANDS (src-tauri/src/commands/analysis.rs)
// ============================================================================

/**
 * Find compatible files for current selection
 */
export const findCompatibleFiles = (): Promise<CompatibleFile[]> =>
  invoke('find_compatible_files');

// ============================================================================
// API OBJECT (for nested access pattern used by existing components)
// ============================================================================

export const api = {
  midi: {
    listDevices: midiListDevices,
    connect: midiConnect,
    disconnect: midiDisconnect,
    isConnected: midiIsConnected,
    getCurrentDevice: midiGetCurrentDevice,
    sendTestNote: midiSendTestNote,
  },
  search: {
    files: searchFiles,
  },
  files: {
    getDetails: getFileDetails,
    loadPattern: loadMidiPattern,
  },
  sequencer: {
    getTracks: getTracks,
    addTrack: addTrack,
    removeTrack: removeTrack,
    updateTrack: updateTrack,
    start: startSequencer,
    stop: stopSequencer,
    pause: pauseSequencer,
    resume: resumeSequencer,
    getPosition: getPlaybackPosition,
    seek: seekPosition,
    loadTracks: loadSequencerTracks,
    isPlaying: isSequencerPlaying,
  },
  playback: {
    setTempo: setTempo,
    getTempo: getTempo,
  },
  favorites: {
    getAll: getFavorites,
    add: addFavorite,
    remove: removeFavorite,
  },
  compatibility: {
    findCompatible: findCompatibleFiles,
  },
};
