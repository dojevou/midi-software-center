/**
 * Complete file details with all metadata
 * Backend: daw/src-tauri/src/models/midi_file.rs
 */
export interface FileDetails {
  id: number; // i64
  filename: string; // filename (from backend)
  filepath: string; // filepath (from backend)
  file_size_bytes: number; // file_size_bytes (from backend)
  bpm?: number; // Option<f64>
  key_signature?: string; // key_signature option
  time_signature?: string; // "4/4" format
  duration_seconds?: number; // NUMERIC(10,3)
  total_notes?: number; // INTEGER
  primary_category?: string; // primary_category optional
  parent_folder?: string; // Option<String>
  created_at: string; // ISO 8601 timestamp
  is_favorite?: boolean; // bool - made optional for frontend flexibility
  tags: string[]; // Vec<String> (folder_tags from backend)
  manufacturer?: string; // Option<String>
  collection_name?: string; // collection_name optional
  track_count: number; // num_tracks (i16)
  has_notes: boolean; // bool
  has_drums?: boolean; // Option<bool>
  content_hash: string; // content_hash from backend
  rating?: number; // 0-5 stars (frontend-only, stored in localStorage)
}

export interface SearchFilters {
  search_text?: string;
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
  sort_by?: string;
  sort_desc?: boolean;
  limit?: number;
  offset?: number;
}

export interface SearchResponse {
  files: FileDetails[];
  total: number;
}

export interface FilterOptions {
  keys: string[];
  categories: string[];
  instruments: string[];
}

// Import/Export types
export interface ImportProgress {
  current: number;
  total: number;
  rate: number;
  eta: number;
}

export interface ImportSummary {
  files_imported: number;
  total_files: number;
  failed_files: number;
  errors?: string[];
}

// Track and Sequencer types
export interface Track {
  id: number;
  file_id: number;
  channel: number;
  muted: boolean;
  soloed: boolean;
  volume: number;
  pan: number;
}

export interface TrackProperties {
  muted?: boolean;
  soloed?: boolean;
  volume?: number;
  pan?: number;
}

/**
 * Playback position in musical time
 * Uses current_ prefix for backend compatibility (Rust serde)
 */
export interface PlaybackPosition {
  current_bar: number;
  current_beat: number;
  current_tick: number;
}

/**
 * Helper to create PlaybackPosition from bar/beat/tick values
 */
export function createPlaybackPosition(bar: number, beat: number, tick: number): PlaybackPosition {
  return {
    current_bar: bar,
    current_beat: beat,
    current_tick: tick,
  };
}

/**
 * Complete playback state including position and transport controls
 * Used by the DAW window and playback store
 */
export interface PlaybackState {
  current_bar: number;
  current_beat: number;
  current_tick: number;
  is_playing: boolean;
  is_recording: boolean;
  bpm: number;
}

// MIDI types
export interface MidiDevice {
  name: string;
  port_number: number;
  is_available: boolean;
}

export interface MidiPort {
  id: number;
  name: string;
  direction: 'input' | 'output';
  device_name: string | null;
  display_name: string | null;
  alias: string | null;
  enabled: boolean;
  is_connected: boolean;
  auto_connect: boolean;
  send_clock: boolean;
  send_transport: boolean;
  route_to: string;
}

export interface MidiIOState {
  ports: Record<number, MidiPort>;
  next_id: number;
}

export interface MidiPattern {
  id: number;
  name: string;
  notes: number[];
}

export interface CompatibleFile {
  id: number;
  filename: string;
  compatibility_score: number;
  reason: string;
  // Extended properties for detailed display
  similarity?: number;
  duration_seconds?: number;
  file_size_bytes?: number;
  bpm?: number;
  key_signature?: string;
}

// Window state types
export interface DAWWindowState {
  tracks: TrackInfo[];
  transport: TransportInfo;
  mixer: MixerState;
}

export interface TrackInfo {
  id: number;
  label: string;
  visible: boolean;
  muted: boolean;
  soloed: boolean;
  color: string;
  height: number;
  midi_channel: number;
  event_count: number;
}

export interface TransportInfo {
  position: PlaybackPosition;
  is_playing: boolean;
  is_recording: boolean;
  loop_enabled: boolean;
  loop_start: number;
  loop_end: number;
  metronome_enabled: boolean;
  metronome_volume: number;
  bpm: number;
  time_signature: [number, number];
  key_signature: string;
}

/**
 * Mixer window state from backend
 * Backend: daw/src-tauri/src/windows/state.rs:MixerWindowState
 */
export interface MixerState {
  channels: Record<number, MixerChannel>; // HashMap<i32, MixerChannel> in Rust
  master: MixerChannel;
  show_meters: boolean;
  show_effects: boolean;
}

/**
 * Mixer channel information
 * Backend: daw/src-tauri/src/windows/state.rs:MixerChannel
 */
export interface MixerChannel {
  id: number; // i32 - channel ID (matches track ID)
  channel_type: string; // ChannelType enum as string
  label: string; // channel label
  volume: number; // f32 0.0-1.0
  pan: number; // f32 -1.0 to 1.0
  muted: boolean;
  soloed: boolean;
  meter_level: number; // f32 0.0-1.0
}

// Automation types
export interface AutomationLane {
  id: number;
  track_id: number;
  parameter_type: ParameterType;
  curve_type: CurveType;
  points: AutomationPoint[];
}

export interface AutomationPoint {
  id: number;
  tick: number;
  value: number;
  curve_type?: CurveType;
}

export type ParameterType = 'volume' | 'pan' | 'pitch' | 'filter' | 'custom';

export type CurveType = 'linear' | 'exponential' | 'logarithmic' | 'bezier';

export interface TrackDetails {
  id: number;
  label: string;
  file_id: number;
  channel: number;
  muted: boolean;
  soloed: boolean;
  volume: number;
  pan: number;
}

/**
 * Velocity editor track with note statistics
 * Used by PianoRollWindow for velocity editing
 */
export interface VelocityTrack {
  id: number;
  name: string;
  noteCount: number;
  avgVelocity: number;
}

// Database-specific types (for database commands)
export interface MidiFile {
  id: number;
  file_path: string;
  file_name: string;
  bpm: number;
  key_signature: string;
  tags: string[];
  duration: number;
  track_count: number;
  file_size: number;
  created_at: string;
  updated_at: string;
}

export interface DatabaseFilters {
  query?: string;
  bpm_min?: number;
  bpm_max?: number;
  key?: string;
  tag?: string;
  limit?: number;
  offset?: number;
}

export interface SearchResults {
  files: MidiFile[];
  total_count: number;
}

export interface DatabaseStats {
  total_files: number;
  avg_bpm: number;
  total_size: number;
}

// Pipeline-specific types
export interface PipelineProgress {
  current: number;
  total: number;
  stage: string;
  current_file?: string;
  rate: number;
  eta_seconds: number;
  details: string;
}

export interface ImportStats {
  files_processed: number;
  files_imported: number;
  files_skipped: number;
  total_size: number;
  duration_seconds: number;
  errors: string[];
}

export interface AnalysisResults {
  files_analyzed: number;
  bpm_detected: number;
  key_detected: number;
  instruments_found: string[];
  errors: string[];
}

export interface FileParams {
  file_path: string;
  file_name: string;
  bpm?: number;
  key_signature?: string;
  tags?: string[];
  duration?: number;
  track_count?: number;
  file_size?: number;
}

// Pipeline state type
export interface PipelineState {
  operation: 'idle' | 'importing' | 'analyzing' | 'archiving' | 'completed';
  progress: PipelineProgress | null;
  errors: string[];
}

// Window management types
export type WindowId = 'arrangement' | 'database' | 'daw' | 'mixer' | 'midi-io-setup' | 'midi-monitor' | 'pipeline' | 'preferences' | 'gear-manager' | 'presets-manager' | 'vip3-browser' | 'score' | 'script-editor' | 'midi-learn' | 'link-sync' | 'piano-roll' | 'export' | 'tag-editor' | 'favorites';

// ============================================================================
// SEQUENCER TRACK TYPES (MPC 3.0 style)
// ============================================================================

/**
 * Track type for MPC 3.0-style arrangement view
 * Used for color coding and icon display
 */
export type SequencerTrackType = 'midi' | 'drum' | 'audio' | 'bus';

/**
 * Track type colors (MPC 3.0 style)
 */
export const TRACK_TYPE_COLORS: Record<SequencerTrackType, string> = {
  midi: '#3B82F6',   // Blue
  drum: '#EF4444',   // Red
  audio: '#22C55E',  // Green
  bus: '#8B5CF6',    // Purple
};

/**
 * Track type icons (emoji-based)
 */
export const TRACK_TYPE_ICONS: Record<SequencerTrackType, string> = {
  midi: '🎹',
  drum: '🥁',
  audio: '🔊',
  bus: '📤',
};

export interface WindowPosition {
  x: number;
  y: number;
  width: number;
  height: number;
  z_index: number;
  visible: boolean;
}

export interface FileMetadata {
  id: number;
  filename: string;
  filepath: string;
  file_size_bytes: number;
}

// Progress event types
export interface AnalysisProgress {
  current: number;
  total: number;
  current_file: string;
  rate: number;
  eta?: number;
}

export interface AnalysisSummary {
  total_files?: number;
  files_analyzed?: number;
  analyzed?: number;
  files_with_bpm?: number;
  files_with_key?: number;
  failed?: number;
  errors?: string[];
  duration_seconds?: number;
  duration_secs?: number;
  rate?: number;
}

/**
 * Archive extraction progress event
 * Matches backend: pipeline/src-tauri/src/commands/archive_import.rs
 */
export interface ArchiveProgress {
  current: number;
  total: number;
  current_file?: string;
  current_archive?: string;
  bytes_processed?: number;
  total_bytes?: number;
  rate?: number;
}

export interface ArchiveError {
  file_path?: string;
  archivePath?: string;
  error_message?: string;
  error?: string;
}

// ============================================================================
// TAG TYPES
// ============================================================================

/**
 * Tag definition from database
 * Backend: pipeline/src-tauri/src/db/models.rs
 */
export interface Tag {
  id: number;
  name: string;
  category: string;
  count: number;
}

/**
 * Tag statistics summary
 */
export interface TagStats {
  total_tags: number;
  total_file_tags: number;
  top_tags: Tag[];
  categories: string[];
}

/**
 * Parameters for adding tags to files
 */
export interface TagUpdateParams {
  file_id: number;
  tags: string[];
}

/**
 * Result of CSV tag import operation
 * Backend: daw/src-tauri/src/commands/tags.rs:ImportResult
 */
export interface ImportResult {
  imported: number;
  skipped: number;
  errors: string[];
}

/**
 * Effect slot for mixer effects
 * Backend: daw/src-tauri/src/commands/mixer.rs:EffectSlot
 */
export interface EffectSlot {
  id: number;
  name: string;
  enabled: boolean;
  wet_dry: number;
  parameters: Record<string, number>;
}

/**
 * Split result from track splitting
 * Backend: pipeline/src-tauri/src/commands/split_file.rs:SplitResult
 */
export interface SplitResult {
  original_file_id: number;
  split_tracks: SplitTrack[];
  total_tracks: number;
  success: boolean;
  error?: string;
}

/**
 * Individual split track info
 */
export interface SplitTrack {
  file_id: number;
  track_number: number;
  channel: number;
  filename: string;
  filepath: string;
}

// ============================================================================
// PROJECT TYPES
// ============================================================================

/**
 * Project definition from project browser
 * Backend: daw/src-tauri/src/models/project.rs
 */
export interface Project {
  id: number;
  name: string;
  description?: string;
  file_path?: string;
  file_size?: number;
  format?: string;
  track_count?: number;
  event_count?: number;
  note_count?: number;
  duration?: number;
  tags?: string[];
  created_at: string;
  modified_at: string;
  last_opened?: string;
}

/**
 * Project info for display (lightweight version)
 */
export interface ProjectInfo {
  id?: number;
  name?: string;
  description?: string;
  bpm?: number;
  key_signature?: string;
  time_signature?: string;
  length?: number; // Length in ticks (for export operations)
}

// ============================================================================
// LOOP BROWSER TYPES
// ============================================================================

/**
 * Loop item from loop browser
 * Backend: daw/src-tauri/src/browsers/loop_browser.rs
 */
export interface Loop {
  id: number;
  name: string;
  file_path?: string;
  bpm?: number | null;
  key?: string | null;
  key_signature?: string;
  duration?: number;
  duration_seconds?: number;
  category?: string | null;
  description?: string | null;
  play_count?: number;
  rating?: number | null;
  rating_count?: number;
  tags?: string[] | null;
  added_date?: string;
  last_played?: string | null;
  file_size?: number;
  format?: string;
  channels?: number;
  sample_rate?: number;
  preview_url?: string;
  analysis?: LoopAnalysis;
}

/**
 * Loop analysis result with spectral and rhythm data
 */
export interface LoopAnalysis {
  bpm?: number;
  key_signature?: string;
  duration?: number;
  transients?: number[];
  waveform?: number[];
  spectral?: {
    brightness: number;
    flatness: number;
    centroid: number;
  };
  rhythm?: {
    tempo_confidence: number;
    onset_strength: number;
    beat_count: number;
  };
}

/**
 * Loop preview data for waveform display
 */
export interface LoopPreview {
  id: number;
  waveform_data?: number[];
  waveform?: number[];
  currentTime?: number;
  duration_seconds?: number;
}

// ============================================================================
// NOTE TYPES (for piano roll)
// ============================================================================

/**
 * MIDI note for piano roll and editors
 * Backend: daw/src-tauri/src/editors/piano_roll.rs
 */
export interface Note {
  id: number;
  pitch: number;
  velocity: number;
  startTick: number;
  duration: number;
  channel: number;
  trackId?: number; // Optional track reference for editors
}

// ============================================================================
// MIDI MIXER TYPES (for external hardware control via 5-pin MIDI out)
// ============================================================================

/**
 * MIDI channel parameters for external hardware control
 * Each channel corresponds to a MIDI channel (1-16)
 */
export interface MidiMixerChannel {
  id: number; // MIDI channel 1-16
  label: string;
  enabled: boolean;

  // MIDI Port configuration
  midiPort: string | null; // Output port (e.g., 'Port A', 'Port B')

  // Core MIDI parameters
  velocity: number; // 0-127, base velocity scale
  velocityScale: number; // 1-200%, multiply velocity
  velocityMin: number; // 0-127, minimum velocity clamp
  velocityMax: number; // 0-127, maximum velocity clamp
  velocityOffset: number; // -127 to +127, add/subtract

  transpose: number; // -48 to +48 semitones
  octaveShift: number; // -4 to +4 octaves

  pitchBend: number; // -8192 to +8191 (14-bit)
  pitchBendRange: number; // Semitones (typically 2, can be 12 or 24)

  // Channel Controls (CC messages)
  volume: number; // CC7: 0-127
  pan: number; // CC10: 0-127 (64 = center)
  expression: number; // CC11: 0-127
  modulation: number; // CC1: 0-127
  sustainPedal: boolean; // CC64: on/off

  // Additional CCs
  cc74: number; // Brightness/Cutoff: 0-127
  cc71: number; // Resonance: 0-127
  cc73: number; // Attack: 0-127
  cc75: number; // Decay: 0-127
  cc72: number; // Release: 0-127

  // Program/Bank selection
  programChange: number | null; // 0-127, select patch
  bankMsb: number | null; // 0-127, Bank Select MSB
  bankLsb: number | null; // 0-127, Bank Select LSB

  // Channel state
  muted: boolean;
  soloed: boolean;

  // Timing adjustments
  delayMs: number; // Note delay in milliseconds (-100 to +100)
  swingPercent: number; // Swing amount 0-100%
  humanizePercent: number; // Randomization 0-100%
  quantizeGrid: string | null; // 'off', '1/4', '1/8', '1/16', '1/32', '1/4T', '1/8T', '1/16T'
  quantizeStrength: number; // 0-100%, how hard to snap

  // Note filtering
  noteRangeLow: number; // 0-127
  noteRangeHigh: number; // 0-127
  keyScale: string | null; // Scale constraint (e.g., 'C major', 'D minor')
}

/**
 * Master channel that affects all channels
 */
export interface MidiMasterChannel {
  enabled: boolean;

  // Master velocity controls (applied to all channels)
  velocity: number; // 0-127, base master velocity
  velocityScale: number; // 1-200%, global velocity multiplier
  velocityMin: number; // 0-127, global minimum clamp
  velocityMax: number; // 0-127, global maximum clamp
  velocityOffset: number; // -127 to +127, global offset

  transpose: number; // -24 to +24, global transpose
  pitchBend: number; // -8192 to +8191, global pitch bend
  volume: number; // CC7: 0-127

  // Global note filtering
  noteRangeLow: number; // 0-127, global low note filter
  noteRangeHigh: number; // 0-127, global high note filter

  // Tempo control
  tempo: number; // BPM for sync (20-300)
  tempoMultiplier: number; // 0.25x to 4x

  // Global timing
  globalDelayMs: number; // -100 to +100 ms
  globalSwing: number; // 0-100%
  globalHumanize: number; // 0-100%
  quantizeGrid: string | null; // Global quantize setting
  quantizeStrength: number; // 0-100%, global quantize strength

  // MIDI output settings
  outputDevice: string | null;
  outputPort: number;
  clockEnabled: boolean; // Send MIDI clock
  transportEnabled: boolean; // Send start/stop/continue
}

/**
 * Complete MIDI Mixer state
 */
export interface MidiMixerState {
  channels: MidiMixerChannel[];
  master: MidiMasterChannel;
  selectedChannel: number | null;
  linkChannels: number[]; // Linked channels for group editing
  showAdvanced: boolean;
}

/**
 * MIDI CC parameter definition
 */
export interface MidiCCDefinition {
  cc: number;
  name: string;
  description: string;
  min: number;
  max: number;
  defaultValue: number;
  bipolar: boolean; // true if center (64) is neutral
}

/**
 * Common MIDI CC numbers and their standard definitions
 */
export const MIDI_CC_DEFINITIONS: Record<number, MidiCCDefinition> = {
  1: { cc: 1, name: 'Modulation', description: 'Modulation wheel', min: 0, max: 127, defaultValue: 0, bipolar: false },
  7: { cc: 7, name: 'Volume', description: 'Channel volume', min: 0, max: 127, defaultValue: 100, bipolar: false },
  10: { cc: 10, name: 'Pan', description: 'Stereo panning', min: 0, max: 127, defaultValue: 64, bipolar: true },
  11: { cc: 11, name: 'Expression', description: 'Expression controller', min: 0, max: 127, defaultValue: 127, bipolar: false },
  64: { cc: 64, name: 'Sustain', description: 'Sustain pedal', min: 0, max: 127, defaultValue: 0, bipolar: false },
  71: { cc: 71, name: 'Resonance', description: 'Filter resonance', min: 0, max: 127, defaultValue: 64, bipolar: false },
  72: { cc: 72, name: 'Release', description: 'Envelope release', min: 0, max: 127, defaultValue: 64, bipolar: false },
  73: { cc: 73, name: 'Attack', description: 'Envelope attack', min: 0, max: 127, defaultValue: 64, bipolar: false },
  74: { cc: 74, name: 'Brightness', description: 'Filter cutoff', min: 0, max: 127, defaultValue: 64, bipolar: false },
  75: { cc: 75, name: 'Decay', description: 'Envelope decay', min: 0, max: 127, defaultValue: 64, bipolar: false },
};

// ============================================================================
// WINDOW MANAGEMENT TYPES (Pro Tools-style floating windows)
// ============================================================================

/**
 * Window docking state
 */
export type WindowDockState = 'floating' | 'docked-left' | 'docked-right' | 'docked-top' | 'docked-bottom' | 'maximized' | 'minimized';

/**
 * Extended window ID for all application windows
 */
export type ExtendedWindowId =
  | 'database'
  | 'daw'
  | 'mixer'
  | 'pipeline'
  | 'piano-roll'
  | 'settings'
  | 'file-details'
  | 'tag-editor'
  | 'loop-browser'
  | 'project-browser'
  | 'midi-device'
  | 'favorites'
  | 'export'
  | 'command-palette';

/**
 * Enhanced window position with docking support
 */
export interface EnhancedWindowPosition extends WindowPosition {
  dockState: WindowDockState;
  savedPosition?: WindowPosition; // Position before docking/maximizing
  minWidth: number;
  minHeight: number;
  maxWidth?: number;
  maxHeight?: number;
  canResize: boolean;
  canClose: boolean;
  canMinimize: boolean;
  canMaximize: boolean;
  alwaysOnTop: boolean;
  snapToEdges: boolean;
  snapThreshold: number; // Pixels for edge snapping
}

/**
 * Window layout preset
 */
export interface WindowLayoutPreset {
  id: string;
  name: string;
  description: string;
  windows: Record<ExtendedWindowId, Partial<EnhancedWindowPosition>>;
}

// ============================================================================
// SYSTEM HEALTH TYPES
// ============================================================================

/**
 * Database health status
 * Backend: app/src-tauri/src/main.rs:DatabaseHealth
 */
export interface DatabaseHealth {
  connected: boolean;
  pool_status: string;
  latency_ms: number | null;
}

/**
 * Sentry error tracking health status
 * Backend: app/src-tauri/src/main.rs:SentryHealth
 */
export interface SentryHealth {
  configured: boolean;
  environment: string | null;
}

/**
 * Complete health check response from backend
 * Backend: app/src-tauri/src/main.rs:HealthCheckResponse
 */
export interface HealthCheckResponse {
  status: string;
  version: string;
  database: DatabaseHealth;
  sentry: SentryHealth;
  uptime_seconds: number;
  timestamp: string;
}

// ============================================================================
// PREFERENCES TYPES
// ============================================================================

/**
 * Application setting
 * Backend: daw/src-tauri/src/commands/preferences.rs:Setting
 */
export interface Setting {
  key: string;
  value: string;
  setting_type: SettingType;
  category: string;
  description: string;
  default_value: string;
  modified_at: string;
}

export type SettingType = 'string' | 'integer' | 'float' | 'boolean' | 'json';

/**
 * Window layout configuration
 * Backend: daw/src-tauri/src/commands/preferences.rs:WindowLayout
 */
export interface WindowLayoutConfig {
  id: number;
  name: string;
  description: string;
  is_default: boolean;
  is_factory: boolean;
  windows: WindowLayoutEntry[];
  created_at: string;
  modified_at: string;
}

export interface WindowLayoutEntry {
  window_id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  visible: boolean;
  z_index: number;
  docked: boolean;
  dock_position?: string;
}

/**
 * Keyboard shortcut definition
 * Backend: daw/src-tauri/src/commands/preferences.rs:KeyboardShortcut
 */
export interface KeyboardShortcut {
  id: number;
  action: string;
  category: string;
  key_combo: string;
  description: string;
  is_default: boolean;
  is_factory: boolean;
  enabled: boolean;
}

/**
 * Recent project entry
 * Backend: daw/src-tauri/src/commands/preferences.rs:RecentProject
 */
export interface RecentProject {
  id: number;
  name: string;
  file_path: string;
  thumbnail?: string;
  last_opened: string;
  open_count: number;
  is_pinned: boolean;
  bpm?: number;
  key_signature?: string;
  track_count?: number;
}

// ============================================================================
// GEAR PROFILE TYPES
// ============================================================================

/**
 * Hardware gear profile
 * Backend: daw/src-tauri/src/commands/gear.rs:GearProfile
 */
export interface GearProfile {
  id: number;
  name: string;
  manufacturer: string;
  gear_type: GearType;
  description: string;
  midi_in_channels: number[];
  midi_out_channels: number[];
  default_program: number;
  default_bank_msb: number;
  default_bank_lsb: number;
  icon?: string;
  image_url?: string;
  manual_url?: string;
  is_factory: boolean;
  created_at: string;
  modified_at: string;
}

export type GearType = 'synthesizer' | 'drum_machine' | 'sampler' | 'effect' | 'controller' | 'sequencer' | 'interface' | 'other';

/**
 * CC mapping for gear
 * Backend: daw/src-tauri/src/commands/gear.rs:CCMapping
 */
export interface CCMapping {
  id: number;
  gear_profile_id: number;
  cc_number: number;
  name: string;
  description: string;
  min_value: number;
  max_value: number;
  default_value: number;
  is_bipolar: boolean;
  display_format: string;
}

/**
 * Program/patch definition
 * Backend: daw/src-tauri/src/commands/gear.rs:GearProgram
 */
export interface GearProgram {
  id: number;
  gear_profile_id: number;
  name: string;
  bank_msb: number;
  bank_lsb: number;
  program_number: number;
  category: string;
  tags: string[];
  description: string;
  is_factory: boolean;
}

/**
 * User's gear instance
 * Backend: daw/src-tauri/src/commands/gear.rs:UserGear
 */
export interface UserGear {
  id: number;
  profile_id: number;
  nickname: string;
  midi_input_port?: string;
  midi_output_port?: string;
  midi_channel: number;
  notes: string;
  is_favorite: boolean;
  last_used?: string;
  created_at: string;
}

/**
 * User gear with profile details
 */
export interface UserGearWithProfile extends UserGear {
  profile: GearProfile;
}

// ============================================================================
// PRESET TYPES
// ============================================================================

/**
 * Mixer preset configuration
 * Backend: daw/src-tauri/src/commands/presets.rs:MixerPreset
 */
export interface MixerPreset {
  id: number;
  name: string;
  description: string;
  category: string;
  is_factory: boolean;
  channels: MixerPresetChannel[];
  master: MixerPresetMaster;
  created_at: string;
  modified_at: string;
}

export interface MixerPresetChannel {
  channel_id: number;
  label: string;
  volume: number;
  pan: number;
  muted: boolean;
  soloed: boolean;
  midi_channel: number;
  program_change?: number;
  bank_msb?: number;
  bank_lsb?: number;
  cc_values: Record<number, number>;
}

export interface MixerPresetMaster {
  volume: number;
  pan: number;
  limiter_enabled: boolean;
  limiter_threshold: number;
}

/**
 * Track template for quick track creation
 * Backend: daw/src-tauri/src/commands/presets.rs:TrackTemplate
 */
export interface TrackTemplate {
  id: number;
  name: string;
  description: string;
  category: string;
  track_type: TrackType;
  is_factory: boolean;
  color: string;
  default_height: number;
  midi_channel: number;
  instrument_name?: string;
  volume: number;
  pan: number;
  routing: TrackMidiRouting;
  created_at: string;
  modified_at: string;
}

export type TrackType = 'midi' | 'audio' | 'instrument' | 'bus' | 'aux' | 'master';

export interface TrackMidiRouting {
  input_device?: string;
  input_channel: number;
  output_device?: string;
  output_channel: number;
  thru_enabled: boolean;
}

/**
 * Project template for new project creation
 * Backend: daw/src-tauri/src/commands/presets.rs:ProjectTemplate
 */
export interface ProjectTemplate {
  id: number;
  name: string;
  description: string;
  category: string;
  is_factory: boolean;
  bpm: number;
  time_signature_numerator: number;
  time_signature_denominator: number;
  key_signature: string;
  tracks: ProjectTemplateTrack[];
  mixer_preset_id?: number;
  created_at: string;
  modified_at: string;
}

export interface ProjectTemplateTrack {
  name: string;
  track_type: TrackType;
  color: string;
  midi_channel: number;
  instrument_name?: string;
  volume: number;
  pan: number;
}

// ============================================================================
// VIP3 BROWSER TYPES (Akai VIP3-style multi-column filtering)
// ============================================================================

/**
 * Timbre category for sound classification
 * Backend: database/migrations/019_vip3_filtering.sql
 */
export interface Timbre {
  id: number;
  name: string;
  description?: string;
  icon?: string;
  sort_order: number;
  created_at: string;
}

/**
 * Musical style/genre category
 * Backend: database/migrations/019_vip3_filtering.sql
 */
export interface Style {
  id: number;
  name: string;
  description?: string;
  parent_id?: number; // For hierarchical styles
  icon?: string;
  sort_order: number;
  created_at: string;
}

/**
 * Articulation type for playing technique
 * Backend: database/migrations/019_vip3_filtering.sql
 */
export interface Articulation {
  id: number;
  name: string;
  description?: string;
  abbreviation?: string;
  icon?: string;
  sort_order: number;
  created_at: string;
}

/**
 * BPM range for tempo filtering
 * Backend: database/migrations/019_vip3_filtering.sql
 */
export interface BpmRange {
  id: number;
  name: string; // e.g., "Slow", "Medium", "Fast"
  min_bpm: number;
  max_bpm: number;
  description?: string;
  sort_order: number;
}

/**
 * Musical key for harmonic filtering
 * Backend: database/migrations/019_vip3_filtering.sql
 */
export interface MusicalKey {
  id: number;
  name: string; // e.g., "C Major", "A Minor"
  root_note: string; // e.g., "C", "A"
  mode: 'major' | 'minor';
  relative_key_id?: number; // Reference to relative major/minor
  sort_order: number;
}

/**
 * User-created collection of MIDI files
 * Backend: database/migrations/019_vip3_filtering.sql
 */
export interface Collection {
  id: number;
  name: string;
  description?: string;
  color?: string;
  icon?: string;
  is_smart: boolean; // Smart collection uses saved search criteria
  saved_search_id?: number; // Link to saved search if smart collection
  file_count: number;
  created_at: string;
  modified_at: string;
}

/**
 * Collection with file IDs (for editing)
 */
export interface CollectionWithFiles extends Collection {
  file_ids: number[];
}

/**
 * Saved search configuration
 * Backend: database/migrations/019_vip3_filtering.sql
 */
export interface SavedSearch {
  id: number;
  name: string;
  description?: string;
  filters: VIP3BrowserFilters;
  is_favorite: boolean;
  use_count: number;
  last_used?: string;
  created_at: string;
  modified_at: string;
}

/**
 * Recent search entry (auto-saved)
 */
export interface RecentSearch {
  id: number;
  filters: VIP3BrowserFilters;
  result_count: number;
  searched_at: string;
}

/**
 * VIP3-style browser filter configuration
 * Supports multi-select on all filter columns
 */
export interface VIP3BrowserFilters {
  // Text search
  search_text?: string;
  search_fields?: ('filename' | 'tags' | 'path' | 'all')[];

  // Category filters (multi-select)
  timbre_ids?: number[];
  style_ids?: number[];
  articulation_ids?: number[];

  // Musical filters
  bpm_range_ids?: number[];
  key_ids?: number[];
  min_bpm?: number;
  max_bpm?: number;
  key_signatures?: string[];

  // Time signature filter
  time_signatures?: string[];

  // Duration filter (seconds)
  min_duration?: number;
  max_duration?: number;

  // Note count filter
  min_notes?: number;
  max_notes?: number;

  // Track count filter
  min_tracks?: number;
  max_tracks?: number;

  // Tag filters (multi-select)
  include_tags?: string[];
  exclude_tags?: string[];

  // Collection filter
  collection_ids?: number[];

  // Specific file IDs filter
  file_ids?: number[];

  // File properties
  has_drums?: boolean;
  is_favorite?: boolean;

  // Manufacturer/source filter
  manufacturers?: string[];
  sources?: string[];

  // Sorting
  sort_by?: VIP3SortField;
  sort_desc?: boolean;

  // Pagination
  limit?: number;
  offset?: number;
}

/**
 * Sort field options for VIP3 browser
 */
export type VIP3SortField =
  | 'filename'
  | 'created_at'
  | 'modified_at'
  | 'bpm'
  | 'key_signature'
  | 'duration'
  | 'total_notes'
  | 'track_count'
  | 'file_size'
  | 'rating'
  | 'play_count';

/**
 * VIP3 browser results with metadata
 */
export interface VIP3BrowserResults {
  files: FileDetails[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;

  // Aggregations for filter counts
  timbre_counts?: Record<number, number>;
  style_counts?: Record<number, number>;
  articulation_counts?: Record<number, number>;
  bpm_range_counts?: Record<number, number>;
  key_counts?: Record<number, number>;
  tag_counts?: Record<string, number>;
}

/**
 * VIP3 browser column configuration
 */
export interface VIP3ColumnConfig {
  id: string;
  label: string;
  type: 'category' | 'range' | 'tags' | 'search' | 'collections';
  width: number;
  visible: boolean;
  sortOrder: number;
}

/**
 * VIP3 browser state (for store)
 */
export interface VIP3BrowserState {
  // Filter state
  filters: VIP3BrowserFilters;
  activeFilters: string[]; // List of active filter IDs for visual indication

  // Results
  results: VIP3BrowserResults | null;
  isLoading: boolean;
  error: string | null;

  // Selection
  selectedFileIds: number[];
  lastSelectedId: number | null;

  // View options
  viewMode: 'list' | 'grid' | 'compact';
  columns: VIP3ColumnConfig[];
  previewEnabled: boolean;
  previewFileId: number | null;

  // Cached category data
  timbres: Timbre[];
  styles: Style[];
  articulations: Articulation[];
  bpmRanges: BpmRange[];
  musicalKeys: MusicalKey[];
}

/**
 * Category counts for filter UI
 */
export interface CategoryCounts {
  timbres: Record<number, number>;
  styles: Record<number, number>;
  articulations: Record<number, number>;
  bpmRanges: Record<number, number>;
  keys: Record<number, number>;
  tags: Record<string, number>;
}

/**
 * File-category assignment (for batch operations)
 */
export interface FileCategoryAssignment {
  file_id: number;
  timbre_ids?: number[];
  style_ids?: number[];
  articulation_ids?: number[];
}

/**
 * Bulk operation result
 */
export interface BulkOperationResult {
  success_count: number;
  failed_count: number;
  errors: Array<{ file_id: number; error: string }>;
}
