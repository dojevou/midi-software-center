// shared-types.ts
// TypeScript type definitions for API communication between frontend and backend
// Used by both Pipeline and DAW applications

// ============================================================================
// COMMON TYPES
// ============================================================================

export interface MidiFileMetadata {
  file_path: string;
  bpm: number | null;
  key: string | null;
  track_count: number;
  duration_seconds: number;
  has_notes: boolean;
  has_drums: boolean;
}

export interface MidiDevice {
  name: string;
  id: string;
  manufacturer: string | null;
}

export interface DatabaseStatus {
  connected: boolean;
  connection_string: string;
}

// ============================================================================
// PIPELINE API TYPES
// ============================================================================

export interface ProcessingConfig {
  source_directory: string;
  extract_directory: string;
  enable_deduplication: boolean;
  enable_splitting: boolean;
  enable_renaming: boolean;
  parallel_workers: number;
  batch_size: number;
}

export interface ProcessingJob {
  id: string;
  source_dir: string;
  extract_dir: string;
  status: 'Running' | 'Paused' | 'Completed' | 'Cancelled' | 'Error';
  created_at: string;
}

export interface ProcessingStats {
  decompressed: number;
  analyzed: number;
  split: number;
  renamed: number;
  duplicates: number;
  errors: number;
}

export interface ProcessingProgress {
  phase: string;
  current: number;
  total: number;
  speed: number; // files per second
  eta_seconds: number;
  stats: ProcessingStats;
}

export interface JobSummary {
  id: string;
  created_at: string;
  completed_at: string | null;
  status: string;
  files_processed: number;
  errors: number;
}

export interface DirectoryEntry {
  name: string;
  path: string;
}

export interface FileEntry {
  name: string;
  path: string;
  size: number;
  extension: string | null;
}

export interface DirectoryListing {
  current_path: string;
  directories: DirectoryEntry[];
  files: FileEntry[];
}

export interface AppConfig {
  default_source_directory: string | null;
  default_extract_directory: string | null;
  parallel_workers: number;
  enable_deduplication: boolean;
  enable_splitting: boolean;
  enable_renaming: boolean;
  database_url: string;
}

export interface LibraryStats {
  total_files: number;
  total_size_bytes: number;
  unique_files: number;
  duplicate_files: number;
  total_tracks: number;
  avg_bpm: number | null;
  most_common_key: string | null;
}

// ============================================================================
// DAW API TYPES
// ============================================================================

export interface SearchFilters {
  query: string | null;
  bpm_min: number | null;
  bpm_max: number | null;
  key: string | null;
  category: string | null;
  tags: string[] | null;
  has_drums: boolean | null;
  track_count_min: number | null;
  track_count_max: number | null;
  duration_min: number | null;
  duration_max: number | null;
  manufacturer: string | null;
  sort_by: 'relevance' | 'bpm' | 'created_at' | 'file_name';
  sort_order: 'asc' | 'desc';
  limit: number;
  offset: number;
}

export interface SearchResults {
  files: FileDetails[];
  total: number;
  offset: number;
  limit: number;
}

export interface FileDetails {
  id: number;
  file_name: string;
  file_path: string;
  file_size: number;
  bpm: number | null;
  key: string | null;
  category: string | null;
  tags: string[];
  manufacturer: string | null;
  track_count: number;
  duration_seconds: number;
  has_notes: boolean;
  has_drums: boolean;
  content_hash: string;
  created_at: string;
  is_favorite: boolean;
}

export interface CompatibleFile {
  file: FileDetails;
  compatibility_score: number;
  key_distance: number;
  bpm_difference: number;
  reason: string;
}

export interface MidiPattern {
  id: number;
  file_id: number;
  events: MidiEvent[];
  duration_ticks: number;
  ticks_per_beat: number;
}

export interface MidiEvent {
  tick: number;
  event_type: 'NoteOn' | 'NoteOff' | 'ControlChange' | 'ProgramChange';
  channel: number;
  note: number | null;
  velocity: number | null;
  controller: number | null;
  value: number | null;
  program: number | null;
}

export interface Project {
  id: number;
  name: string;
  tempo: number;
  created_at: string;
  updated_at: string;
  tracks: Track[];
}

export interface Track {
  id: number;
  project_id: number;
  file_id: number;
  channel: number;
  name: string;
  muted: boolean;
  solo: boolean;
  volume: number; // 0-127
  pan: number; // 0-127 (64 = center)
  color: string | null;
  order: number;
}

export interface TrackProperties {
  name?: string;
  muted?: boolean;
  solo?: boolean;
  volume?: number;
  pan?: number;
  color?: string;
}

export interface PlaybackPosition {
  current_tick: number;
  current_bar: number;
  current_beat: number;
  percentage: number;
}

export interface UsageStats {
  total_searches: number;
  total_playbacks: number;
  most_played_files: FileDetails[];
  most_searched_terms: Array<{ term: string; count: number }>;
  recent_activity: Array<{
    action: string;
    file_id: number | null;
    timestamp: string;
  }>;
}

// ============================================================================
// API CLIENT FUNCTIONS (for Svelte frontend)
// ============================================================================

declare global {
  interface Window {
    __TAURI__: {
      invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
    };
  }
}

// Pipeline API Client
export class PipelineAPI {
  private static invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return window.__TAURI__.invoke(cmd, args);
  }

  // Processing
  static startProcessing(config: ProcessingConfig): Promise<string> {
    return this.invoke('start_processing', { config });
  }

  static getProcessingProgress(): Promise<ProcessingProgress> {
    return this.invoke('get_processing_progress');
  }

  static pauseProcessing(): Promise<void> {
    return this.invoke('pause_processing');
  }

  static resumeProcessing(): Promise<void> {
    return this.invoke('resume_processing');
  }

  static cancelProcessing(): Promise<void> {
    return this.invoke('cancel_processing');
  }

  static getJobHistory(limit?: number): Promise<JobSummary[]> {
    return this.invoke('get_job_history', { limit });
  }

  // File Browser
  static browseDirectory(path: string): Promise<DirectoryListing> {
    return this.invoke('browse_directory', { path });
  }

  static validateDirectory(path: string, checkWritable: boolean): Promise<boolean> {
    return this.invoke('validate_directory', { path, checkWritable });
  }

  // Analysis
  static analyzeMidiFile(filePath: string): Promise<MidiFileMetadata> {
    return this.invoke('analyze_midi_file', { filePath });
  }

  // Configuration
  static getConfig(): Promise<AppConfig> {
    return this.invoke('get_config');
  }

  static saveConfig(config: AppConfig): Promise<void> {
    return this.invoke('save_config', { config });
  }

  static getDatabaseStatus(): Promise<DatabaseStatus> {
    return this.invoke('get_database_status');
  }

  // Statistics
  static getLibraryStats(): Promise<LibraryStats> {
    return this.invoke('get_library_stats');
  }
}

// DAW API Client
export class DAWAPI {
  private static invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return window.__TAURI__.invoke(cmd, args);
  }

  // MIDI Hardware
  static midiListDevices(): Promise<MidiDevice[]> {
    return this.invoke('midi_list_devices');
  }

  static midiConnect(deviceName: string): Promise<void> {
    return this.invoke('midi_connect', { deviceName });
  }

  static midiDisconnect(): Promise<void> {
    return this.invoke('midi_disconnect');
  }

  static midiIsConnected(): Promise<boolean> {
    return this.invoke('midi_is_connected');
  }

  static midiGetCurrentDevice(): Promise<MidiDevice | null> {
    return this.invoke('midi_get_current_device');
  }

  static midiSendTestNote(channel: number, note: number, velocity: number): Promise<void> {
    return this.invoke('midi_send_test_note', { channel, note, velocity });
  }

  // Search
  static searchFiles(filters: SearchFilters): Promise<SearchResults> {
    return this.invoke('search_files', { filters });
  }

  static getFileDetails(fileId: number): Promise<FileDetails> {
    return this.invoke('get_file_details', { fileId });
  }

  static findCompatibleFiles(fileId: number, maxResults?: number): Promise<CompatibleFile[]> {
    return this.invoke('find_compatible_files', { fileId, maxResults });
  }

  static getSearchSuggestions(query: string, field: string): Promise<string[]> {
    return this.invoke('get_search_suggestions', { query, field });
  }

  // Playback
  static loadMidiPattern(fileId: number): Promise<MidiPattern> {
    return this.invoke('load_midi_pattern', { fileId });
  }

  static playPattern(fileId: number, channel: number): Promise<void> {
    return this.invoke('play_pattern', { fileId, channel });
  }

  static stopPlayback(): Promise<void> {
    return this.invoke('stop_playback');
  }

  static isPlaying(): Promise<boolean> {
    return this.invoke('is_playing');
  }

  static setTempo(bpm: number): Promise<void> {
    return this.invoke('set_tempo', { bpm });
  }

  static getTempo(): Promise<number> {
    return this.invoke('get_tempo');
  }

  // Sequencer
  static createProject(name: string, tempo: number): Promise<Project> {
    return this.invoke('create_project', { name, tempo });
  }

  static loadProject(projectId: number): Promise<Project> {
    return this.invoke('load_project', { projectId });
  }

  static saveProject(): Promise<void> {
    return this.invoke('save_project');
  }

  static addTrack(fileId: number, channel: number): Promise<Track> {
    return this.invoke('add_track', { fileId, channel });
  }

  static removeTrack(trackId: number): Promise<void> {
    return this.invoke('remove_track', { trackId });
  }

  static updateTrack(trackId: number, properties: TrackProperties): Promise<void> {
    return this.invoke('update_track', { trackId, properties });
  }

  static getTracks(): Promise<Track[]> {
    return this.invoke('get_tracks');
  }

  static startSequencer(): Promise<void> {
    return this.invoke('start_sequencer');
  }

  static stopSequencer(): Promise<void> {
    return this.invoke('stop_sequencer');
  }

  static getPlaybackPosition(): Promise<PlaybackPosition> {
    return this.invoke('get_playback_position');
  }

  // Export
  static exportProjectMidi(outputPath: string): Promise<void> {
    return this.invoke('export_project_midi', { outputPath });
  }

  // Favorites
  static addFavorite(fileId: number): Promise<void> {
    return this.invoke('add_favorite', { fileId });
  }

  static removeFavorite(fileId: number): Promise<void> {
    return this.invoke('remove_favorite', { fileId });
  }

  static getFavorites(): Promise<FileDetails[]> {
    return this.invoke('get_favorites');
  }

  // Statistics
  static getUsageStats(): Promise<UsageStats> {
    return this.invoke('get_usage_stats');
  }
}
