// ============================================================================
// CORE DATABASE TYPES
// ============================================================================

/**
 * File metadata from database
 * Backend: pipeline/src-tauri/src/commands/file_import.rs
 */
export interface FileMetadata {
  id: number;                          // i64 (BIGSERIAL)
  filename: string;                    // TEXT
  original_filename: string;           // TEXT
  filepath: string;                    // TEXT (unique)
  content_hash: string;                // BYTEA (hex-encoded string)
  file_size_bytes: number;             // BIGINT
  bpm: number | undefined;             // NUMERIC(6,2) | NULL
  key_signature: string | undefined;   // musical_key ENUM | NULL
}

/**
 * Complete file details with all metadata
 * Backend: daw/src-tauri/src/models/midi_file.rs
 */
export interface FileDetails {
  id: number;                          // i64
  file_name: string;                   // filename (serde rename)
  file_path: string;                   // filepath (serde rename)
  file_size: number;                   // file_size_bytes (serde rename)
  bpm?: number;                        // Option<f64>
  key?: string;                        // key_signature (serde rename)
  time_signature?: string;             // "4/4" format
  duration_seconds?: number;           // NUMERIC(10,3)
  total_notes?: number;                // INTEGER
  category?: string;                   // primary_category (serde rename)
  parent_folder?: string;              // Option<String>
  created_at: string;                  // ISO 8601 timestamp
  is_favorite: boolean;                // bool
  tags: string[];                      // Vec<String>
  manufacturer?: string;               // Option<String>
  collection?: string;                 // collection_name (serde rename)
  track_count: number;                 // num_tracks (i16)
  has_notes: boolean;                  // bool
  has_drums?: boolean;                 // Option<bool>
}

/**
 * Search filters for database queries
 * Backend: daw/src-tauri/src/models/search.rs
 */
export interface SearchFilters {
  min_bpm?: number;                    // Option<f32> (CORRECTED from f64)
  max_bpm?: number;                    // Option<f32> (CORRECTED from f64)
  key_signature?: string;              // Option<String>
  time_signature?: string;             // Option<String>
  category?: string;                   // Option<String>
  min_notes?: number;                  // Option<i32>
  max_notes?: number;                  // Option<i32>
  min_duration?: number;               // Option<f64> (seconds)
  max_duration?: number;               // Option<f64> (seconds)
  instruments?: string[];              // Option<Vec<String>>
  search_text?: string;                // Option<String>
  sort_by?: string;                    // Option<String>
  sort_desc?: boolean;                 // Option<bool>
  limit?: number;                      // Option<i32> (CORRECTED from i64)
  offset?: number;                     // Option<i32> (CORRECTED from i64)
}

/**
 * Search response from backend
 * Backend: daw/src-tauri/src/models/search.rs
 */
export interface SearchResponse {
  files: FileDetails[];                // Vec<FileDetails>
  total: number;                       // i32 (CORRECTED from i64)
}

/**
 * Import progress event payload
 * Backend: pipeline/src-tauri/src/commands/file_import.rs
 */
export interface ImportProgress {
  current: number;                     // usize
  total: number;                       // usize
  current_file: string;                // String (CRITICAL: not "fileName")
  rate: number;                        // f64 (files per second)
}

/**
 * Import summary result
 * Backend: pipeline/src-tauri/src/commands/file_import.rs
 */
export interface ImportSummary {
  total_files: number;                 // usize (CRITICAL: snake_case)
  imported: number;                    // usize
  skipped: number;                     // usize
  errors: string[];                    // Vec<String>
  duration_secs: number;               // f64 (CRITICAL: not "duration")
  rate: number;                        // f64
}

/**
 * Analysis progress for file analysis operations
 */
export interface AnalysisProgress {
  current: number;                     // Current file index
  total: number;                       // Total files to analyze
  current_file: string;                // Current file being analyzed
  rate: number;                        // Files per second
}

/**
 * Analysis summary result
 */
export interface AnalysisSummary {
  total_files: number;                 // Total files analyzed
  analyzed: number;                    // Successfully analyzed
  failed: number;                      // Failed to analyze
  errors: string[];                    // Error messages
  duration_secs: number;               // Duration in seconds
  rate: number;                        // Files per second
}

/**
 * Archive extraction progress
 */
export interface ArchiveProgress {
  current: number;                     // Current file index
  total: number;                       // Total files to extract
  current_archive: string;             // Current archive path
  rate: number;                        // Files per second
}

/**
 * Archive extraction error
 */
export interface ArchiveError {
  archivePath: string;                 // Path to archive that failed
  error: string;                       // Error message
}

// ============================================================================
// SEQUENCER/PLAYBACK TYPES
// ============================================================================

/**
 * Sequencer track
 * Backend: daw/src-tauri/src/models/sequencer.rs
 */
export interface Track {
  id: number;                          // i32
  name: string;                        // String
  file_id: number;                     // i32
  channel: number;                     // u8 (0-15)
  muted: boolean;                      // bool (CRITICAL: "muted" not "mute")
  solo: boolean;                       // bool
  volume: number;                      // u8 (0-127)
  pan: number;                         // u8 (0-127, 64 = center)
  color: string;                       // String (hex color)
}

/**
 * Track properties for partial updates
 * Backend: daw/src-tauri/src/models/sequencer.rs
 * ✅ CORRECTED: Removed name and color fields that don't exist in Rust
 */
export interface TrackProperties {
  muted?: boolean;                     // Option<bool>
  solo?: boolean;                      // Option<bool>
  volume?: number;                     // Option<u8>
  pan?: number;                        // Option<u8>
  // REMOVED: name and color (don't exist in Rust backend)
}

/**
 * Playback position
 * Backend: daw/src-tauri/src/models/sequencer.rs
 */
export interface PlaybackPosition {
  current_tick: number;                // u64 (CRITICAL: snake_case)
  current_bar: number;                 // u32
  current_beat: number;                // u32
}

/**
 * Playback state enum
 * Backend: daw/src-tauri/src/sequencer/engine.rs
 * ✅ NEW: Added missing type
 */
export type PlaybackState =
  | 'Stopped'
  | 'Playing'
  | 'Paused'
  | 'Recording';

// ============================================================================
// WINDOW STATE TYPES (✅ ALL NEW - MISSING FROM V1.0)
// ============================================================================

/**
 * DAW window state
 * Backend: daw/src-tauri/src/windows/state.rs
 * ✅ NEW: Critical missing type
 */
export interface DAWWindowState {
  tempo: number;                          // f32
  time_signature: [number, number];       // (u8, u8)
  key_signature: string;                  // String
  loop_enabled: boolean;                  // bool
  loop_start: number;                     // u64
  loop_end: number;                       // u64
  metronome_enabled: boolean;             // bool
  metronome_volume: number;               // f32
  tracks: TrackInfo[];                    // Vec<TrackInfo>
}

/**
 * Track info from window state
 * Backend: daw/src-tauri/src/windows/state.rs
 * ✅ NEW: Critical missing type
 */
export interface TrackInfo {
  id: number;                             // i32
  label: string;                          // String
  visible: boolean;                       // bool
  muted: boolean;                         // bool
  soloed: boolean;                        // bool
  color: string;                          // String (hex color)
}

/**
 * Transport information
 * Backend: daw/src-tauri/src/commands/window.rs
 * ✅ NEW: Critical missing type
 */
export interface TransportInfo {
  is_playing: boolean;                    // bool
  is_recording: boolean;                  // bool
  tempo: number;                          // f32
  position: PlaybackPosition;             // PlaybackPosition
  loop_enabled: boolean;                  // bool
}

/**
 * Mixer channel state
 * Backend: daw/src-tauri/src/windows/state.rs
 * ✅ NEW: Critical missing type
 */
export interface MixerChannelState {
  track_id: number;                       // i32
  volume: number;                         // f32 (0.0-1.0)
  pan: number;                            // f32 (-1.0 to 1.0)
  muted: boolean;                         // bool
  soloed: boolean;                        // bool
}

/**
 * Mixer window state
 * Backend: daw/src-tauri/src/windows/state.rs
 * ✅ NEW: Critical missing type
 */
export interface MixerState {
  channels: MixerChannelState[];          // Vec<MixerChannelState>
  master_volume: number;                  // f32
}

// ============================================================================
// AUTOMATION TYPES (✅ ALL NEW - COMPLETELY MISSING FROM V1.0)
// ============================================================================

/**
 * Automation lane
 * Backend: daw/src-tauri/src/automation/lane.rs
 * ✅ NEW: Automation support
 */
export interface AutomationLane {
  id: number;                             // i32
  track_id: number;                       // i32
  parameter_type: ParameterType;          // ParameterType enum
  curve_type: CurveType;                  // CurveType enum
  points: AutomationPoint[];              // Vec<AutomationPoint>
}

/**
 * Automation point
 * Backend: daw/src-tauri/src/automation/lane.rs
 * ✅ NEW: Automation support
 */
export interface AutomationPoint {
  id: number;                             // i32
  tick: number;                           // u64
  value: number;                          // f64
}

/**
 * Parameter type for automation
 * Backend: daw/src-tauri/src/automation/lane.rs
 * ✅ NEW: Automation support
 */
export type ParameterType =
  | 'Volume'
  | 'Pan'
  | 'Pitch'
  | 'CC'
  | 'Custom';

/**
 * Curve type for automation interpolation
 * Backend: daw/src-tauri/src/automation/lane.rs
 * ✅ NEW: Automation support
 */
export type CurveType =
  | 'Linear'
  | 'Exponential'
  | 'Logarithmic'
  | 'SCurve'
  | 'Step';

// ============================================================================
// MIDI HARDWARE TYPES
// ============================================================================

/**
 * MIDI device information
 * Backend: daw/src-tauri/src/models/midi.rs
 */
export interface MidiDevice {
  name: string;                        // String
  manufacturer?: string;               // Option<String>
}

/**
 * MIDI event types
 * Backend: daw/src-tauri/src/models/midi.rs
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
 * Backend: daw/src-tauri/src/models/midi.rs
 */
export interface MidiEvent {
  event_type: MidiEventType;           // MidiEventType enum
  tick: number;                        // u64
  channel: number;                     // u8
  note?: number;                       // Option<u8>
  velocity?: number;                   // Option<u8>
  controller?: number;                 // Option<u8>
  value?: number;                      // Option<u8>
  program?: number;                    // Option<u8>
}

/**
 * MIDI pattern
 * Backend: daw/src-tauri/src/models/midi.rs
 */
export interface MidiPattern {
  events: MidiEvent[];                 // Vec<MidiEvent>
  length_ticks: number;                // u64
}

// ============================================================================
// ANALYSIS TYPES
// ============================================================================

/**
 * Compatible file from analysis
 * Backend: daw/src-tauri/src/models/analysis.rs
 */
export interface CompatibleFile {
  id: number;                          // i32
  file_name: string;                   // String
  compatibility_score: number;         // i32
  key_match: boolean;                  // bool
  bpm_difference?: number;             // Option<f32>
  time_signature_match: boolean;       // bool
  suggested_bpm_multiplier?: number;   // Option<f32>
  category?: string;                   // Option<String>
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/**
 * Window position for UI state
 */
export interface WindowPosition {
  x: number;
  y: number;
  width: number;
  height: number;
  z_index: number;
  visible: boolean;
}

/**
 * Window identifier
 */
export type WindowId = 'daw' | 'mixer' | 'database' | 'pipeline';

/**
 * API error response
 */
export interface ApiError {
  message: string;
  code?: string;
  details?: unknown;
}

/**
 * Track details for frontend display
 * Backend: daw/src-tauri/src/commands/project.rs:164
 */
export interface TrackDetails {
  id: number;
  name: string;
  file_id: number;
  channel: number;
  muted: boolean;
  solo: boolean;
  volume: number;
  pan: number;
  event_count: number;
}