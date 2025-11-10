import { invoke } from '@tauri-apps/api/core';
import type {
  FileMetadata,
  FileDetails,
  SearchFilters,
  SearchResponse,
  ImportProgress,
  ImportSummary,
  Track,
  TrackProperties,
  PlaybackPosition,
  PlaybackState,
  MidiDevice,
  MidiPattern,
  CompatibleFile,
  DAWWindowState,
  TrackInfo,
  TransportInfo,
  MixerState,
  AutomationLane,
  AutomationPoint,
  ParameterType,
  CurveType,
  TrackDetails,
} from './types';

// ============================================================================
// MIDI HARDWARE COMMANDS (6 total)
// ============================================================================

export const api = {
  midi: {
    /**
     * List all available MIDI output devices
     * Backend: daw/src-tauri/src/commands/midi.rs:15
     */
    listDevices: async (): Promise<MidiDevice[]> => {
      try {
        return await invoke('midi_list_devices');
      } catch (error) {
        console.error('Failed to list MIDI devices:', error);
        throw error;
      }
    },

    /**
     * Connect to a specific MIDI device by name
     * Backend: daw/src-tauri/src/commands/midi.rs:25
     */
    connect: async (deviceName: string): Promise<void> => {
      try {
        await invoke('midi_connect', { device_name: deviceName });
      } catch (error) {
        console.error('Failed to connect to MIDI device:', error);
        throw error;
      }
    },

    /**
     * Disconnect from current MIDI device
     * Backend: daw/src-tauri/src/commands/midi.rs:36
     */
    disconnect: async (): Promise<void> => {
      try {
        await invoke('midi_disconnect');
      } catch (error) {
        console.error('Failed to disconnect MIDI device:', error);
        throw error;
      }
    },

    /**
     * Check if MIDI device is currently connected
     * Backend: daw/src-tauri/src/commands/midi.rs:47
     */
    isConnected: async (): Promise<boolean> => {
      try {
        return await invoke('midi_is_connected');
      } catch (error) {
        console.error('Failed to check MIDI connection:', error);
        throw error;
      }
    },

    /**
     * Get current MIDI device info
     * Backend: daw/src-tauri/src/commands/midi.rs:57
     */
    getCurrentDevice: async (): Promise<MidiDevice | undefined> => {
      try {
        return await invoke('midi_get_current_device');
      } catch (error) {
        console.error('Failed to get current MIDI device:', error);
        throw error;
      }
    },

    /**
     * Send a test note to verify MIDI connection
     * Backend: daw/src-tauri/src/commands/midi.rs:75
     * ✅ CORRECTED: Added missing channel parameter
     */
    sendTestNote: async (channel: number, note: number, velocity: number): Promise<void> => {
      try {
        await invoke('midi_send_test_note', { channel, note, velocity });
      } catch (error) {
        console.error('Failed to send test note:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // SEQUENCER COMMANDS (13 total)
  // ============================================================================

  sequencer: {
    /**
     * Start sequencer playback
     * Backend: daw/src-tauri/src/commands/sequencer.rs:14
     */
    start: async (): Promise<void> => {
      try {
        await invoke('start_sequencer');
      } catch (error) {
        console.error('Failed to start sequencer:', error);
        throw error;
      }
    },

    /**
     * Stop sequencer playback (resets position)
     * Backend: daw/src-tauri/src/commands/sequencer.rs:22
     */
    stop: async (): Promise<void> => {
      try {
        await invoke('stop_sequencer');
      } catch (error) {
        console.error('Failed to stop sequencer:', error);
        throw error;
      }
    },

    /**
     * Pause sequencer playback (maintains position)
     * Backend: daw/src-tauri/src/commands/sequencer.rs:31
     */
    pause: async (): Promise<void> => {
      try {
        await invoke('pause_sequencer');
      } catch (error) {
        console.error('Failed to pause sequencer:', error);
        throw error;
      }
    },

    /**
     * Resume sequencer playback from paused state
     * Backend: daw/src-tauri/src/commands/sequencer.rs:40
     */
    resume: async (): Promise<void> => {
      try {
        await invoke('resume_sequencer');
      } catch (error) {
        console.error('Failed to resume sequencer:', error);
        throw error;
      }
    },

    /**
     * Get current playback position
     * Backend: daw/src-tauri/src/commands/sequencer.rs:48
     */
    getPosition: async (): Promise<PlaybackPosition> => {
      try {
        return await invoke('get_playback_position');
      } catch (error) {
        console.error('Failed to get playback position:', error);
        throw error;
      }
    },

    /**
     * Seek to a specific position
     * Backend: daw/src-tauri/src/commands/sequencer.rs:61
     * ✅ CORRECTED: Takes (bar, beat) not tick
     */
    seekPosition: async (bar: number, beat: number): Promise<void> => {
      try {
        await invoke('seek_position', { bar, beat });
      } catch (error) {
        console.error('Failed to seek position:', error);
        throw error;
      }
    },

    /**
     * Set global tempo (BPM)
     * Backend: daw/src-tauri/src/commands/sequencer.rs:74
     */
    setTempo: async (bpm: number): Promise<void> => {
      try {
        await invoke('set_tempo', { bpm });
      } catch (error) {
        console.error('Failed to set tempo:', error);
        throw error;
      }
    },

    /**
     * Get current tempo
     * Backend: daw/src-tauri/src/commands/sequencer.rs:83
     */
    getTempo: async (): Promise<number> => {
      try {
        return await invoke('get_tempo');
      } catch (error) {
        console.error('Failed to get tempo:', error);
        throw error;
      }
    },

    /**
     * Add a track to the sequencer
     * Backend: daw/src-tauri/src/commands/sequencer.rs:98
     */
    addTrack: async (fileId: number, channel: number): Promise<Track> => {
      try {
        return await invoke('add_track', { file_id: fileId, channel });
      } catch (error) {
        console.error('Failed to add track:', error);
        throw error;
      }
    },

    /**
     * Remove a track from the sequencer
     * Backend: daw/src-tauri/src/commands/sequencer.rs:145
     */
    removeTrack: async (trackId: number): Promise<void> => {
      try {
        await invoke('remove_track', { track_id: trackId });
      } catch (error) {
        console.error('Failed to remove track:', error);
        throw error;
      }
    },

    /**
     * Update track properties (mute, solo, volume, pan)
     * Backend: daw/src-tauri/src/commands/sequencer.rs:161
     */
    updateTrack: async (trackId: number, properties: TrackProperties): Promise<void> => {
      try {
        await invoke('update_track', { track_id: trackId, properties });
      } catch (error) {
        console.error('Failed to update track:', error);
        throw error;
      }
    },

    /**
     * Get all tracks in current project
     * Backend: daw/src-tauri/src/commands/sequencer.rs:177
     */
    getTracks: async (): Promise<Track[]> => {
      try {
        return await invoke('get_tracks');
      } catch (error) {
        console.error('Failed to get tracks:', error);
        throw error;
      }
    },

    /**
     * Load tracks into sequencer and prepare for playback
     * Backend: daw/src-tauri/src/commands/sequencer.rs:186
     */
    loadTracks: async (): Promise<void> => {
      try {
        await invoke('load_sequencer_tracks');
      } catch (error) {
        console.error('Failed to load tracks:', error);
        throw error;
      }
    },

    /**
     * Check if sequencer is currently playing
     * Backend: daw/src-tauri/src/commands/sequencer.rs:195
     */
    isPlaying: async (): Promise<boolean> => {
      try {
        return await invoke('is_sequencer_playing');
      } catch (error) {
        console.error('Failed to check if playing:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // SEARCH COMMANDS (3 total)
  // ============================================================================

  search: {
    /**
     * Search for files with filters
     * Backend: daw/src-tauri/src/commands/search.rs:25
     */
    files: async (filters: SearchFilters): Promise<SearchResponse> => {
      try {
        return await invoke('search_files', { filters });
      } catch (error) {
        console.error('Failed to search files:', error);
        throw error;
      }
    },

    /**
     * Get detailed information about a specific file
     * Backend: daw/src-tauri/src/commands/search.rs:219
     */
    getDetails: async (fileId: number): Promise<FileDetails> => {
      try {
        return await invoke('get_file_details', { file_id: fileId });
      } catch (error) {
        console.error('Failed to get file details:', error);
        throw error;
      }
    },

    /**
     * Get search suggestions for autocomplete
     * Backend: daw/src-tauri/src/commands/search.rs:278
     * ✅ CORRECTED: Added missing field parameter
     */
    getSuggestions: async (query: string, field: string): Promise<string[]> => {
      try {
        return await invoke('get_search_suggestions', { query, field });
      } catch (error) {
        console.error('Failed to get search suggestions:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // ANALYSIS COMMANDS (6 total)
  // ============================================================================

  analysis: {
    /**
     * Find files that are musically compatible
     * Backend: daw/src-tauri/src/commands/analysis.rs:22
     */
    findCompatible: async (fileId: number, maxResults?: number): Promise<CompatibleFile[]> => {
      try {
        return await invoke('find_compatible_files', { file_id: fileId, max_results: maxResults });
      } catch (error) {
        console.error('Failed to find compatible files:', error);
        throw error;
      }
    },

    /**
     * Add file to favorites
     * Backend: daw/src-tauri/src/commands/analysis.rs:181
     */
    addFavorite: async (fileId: number): Promise<void> => {
      try {
        await invoke('add_favorite', { file_id: fileId });
      } catch (error) {
        console.error('Failed to add favorite:', error);
        throw error;
      }
    },

    /**
     * Remove file from favorites
     * Backend: daw/src-tauri/src/commands/analysis.rs:205
     */
    removeFavorite: async (fileId: number): Promise<void> => {
      try {
        await invoke('remove_favorite', { file_id: fileId });
      } catch (error) {
        console.error('Failed to remove favorite:', error);
        throw error;
      }
    },

    /**
     * Check if a file is favorited
     * Backend: daw/src-tauri/src/commands/analysis.rs:228
     */
    isFavorite: async (fileId: number): Promise<boolean> => {
      try {
        return await invoke('is_favorite', { file_id: fileId });
      } catch (error) {
        console.error('Failed to check favorite:', error);
        throw error;
      }
    },

    /**
     * Get all favorite files with full details
     * Backend: daw/src-tauri/src/commands/analysis.rs:245
     */
    getFavorites: async (): Promise<FileDetails[]> => {
      try {
        return await invoke('get_favorites');
      } catch (error) {
        console.error('Failed to get favorites:', error);
        throw error;
      }
    },

    /**
     * Get usage statistics
     * Backend: daw/src-tauri/src/commands/analysis.rs:298
     */
    getUsageStats: async (): Promise<string> => {
      try {
        return await invoke('get_usage_stats');
      } catch (error) {
        console.error('Failed to get usage stats:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // PROJECT COMMANDS (3 total)
  // ============================================================================

  project: {
    /**
     * Load multiple MIDI files as sequencer tracks
     * Backend: daw/src-tauri/src/commands/project.rs:31
     */
    loadMultipleTracks: async (fileIds: number[]): Promise<Track[]> => {
      try {
        return await invoke('load_multiple_tracks', { file_ids: fileIds });
      } catch (error) {
        console.error('Failed to load multiple tracks:', error);
        throw error;
      }
    },

    /**
     * Clear all tracks from the sequencer
     * Backend: daw/src-tauri/src/commands/project.rs:119
     */
    clearAllTracks: async (): Promise<void> => {
      try {
        await invoke('clear_all_tracks');
      } catch (error) {
        console.error('Failed to clear all tracks:', error);
        throw error;
      }
    },

    /**
     * Get detailed information about loaded tracks
     * Backend: daw/src-tauri/src/commands/project.rs:135
     */
    getTrackDetails: async (): Promise<TrackDetails[]> => {
      try {
        return await invoke('get_track_details');
      } catch (error) {
        console.error('Failed to get track details:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // EXPORT COMMANDS (1 total)
  // ============================================================================

  export: {
    /**
     * Export project as MIDI file
     * Backend: daw/src-tauri/src/commands/export.rs
     */
    projectAsMidi: async (outputPath: string): Promise<void> => {
      try {
        await invoke('export_project_midi', { output_path: outputPath });
      } catch (error) {
        console.error('Failed to export project as MIDI:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // PIPELINE COMMANDS (4 total) - Import, Analysis, Archive
  // ============================================================================

  pipeline: {
    /**
     * Import a single MIDI file into the database
     * Backend: pipeline/src-tauri/src/commands/file_import.rs:157
     */
    importSingleFile: async (filePath: string): Promise<ImportSummary> => {
      try {
        return await invoke('import_single_file', { file_path: filePath });
      } catch (error) {
        console.error('Failed to import single file:', error);
        throw error;
      }
    },

    /**
     * Import a directory of MIDI files
     * Backend: pipeline/src-tauri/src/commands/file_import.rs:368
     */
    importDirectory: async (directoryPath: string): Promise<ImportSummary> => {
      try {
        return await invoke('import_directory', { directory_path: directoryPath });
      } catch (error) {
        console.error('Failed to import directory:', error);
        throw error;
      }
    },

    /**
     * Start analysis operation on database files
     * Backend: pipeline/src-tauri/src/commands/analyze.rs:126
     */
    startAnalysis: async (): Promise<void> => {
      try {
        await invoke('start_analysis');
      } catch (error) {
        console.error('Failed to start analysis:', error);
        throw error;
      }
    },

    /**
     * Import MIDI files from archive collection (ZIP files)
     * Backend: pipeline/src-tauri/src/commands/archive_import.rs:53
     */
    importArchiveCollection: async (archivePath: string): Promise<void> => {
      try {
        await invoke('import_archive_collection', { archive_path: archivePath });
      } catch (error) {
        console.error('Failed to import archive collection:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // WINDOW SYSTEM COMMANDS (33 total) ✅ ALL NEW - COMPLETELY MISSING FROM V1.0
  // ============================================================================

  window: {
    /**
     * Get DAW window state
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    getDawState: async (): Promise<DAWWindowState> => {
      try {
        return await invoke('get_daw_state');
      } catch (error) {
        console.error('Failed to get DAW state:', error);
        throw error;
      }
    },

    /**
     * Reset DAW window state to defaults
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    resetDawState: async (): Promise<void> => {
      try {
        await invoke('reset_daw_state');
      } catch (error) {
        console.error('Failed to reset DAW state:', error);
        throw error;
      }
    },

    /**
     * Start transport playback
     * Backend: daw/src-tauri/src/commands/window.rs:44
     */
    playTransport: async (): Promise<void> => {
      try {
        await invoke('play_transport');
      } catch (error) {
        console.error('Failed to play transport:', error);
        throw error;
      }
    },

    /**
     * Stop transport and reset position
     * Backend: daw/src-tauri/src/commands/window.rs:52
     */
    stopTransport: async (): Promise<void> => {
      try {
        await invoke('stop_transport');
      } catch (error) {
        console.error('Failed to stop transport:', error);
        throw error;
      }
    },

    /**
     * Pause transport at current position
     * Backend: daw/src-tauri/src/commands/window.rs:61
     */
    pauseTransport: async (): Promise<void> => {
      try {
        await invoke('pause_transport');
      } catch (error) {
        console.error('Failed to pause transport:', error);
        throw error;
      }
    },

    /**
     * Set playback position
     * Backend: daw/src-tauri/src/commands/window.rs:72
     */
    setPlaybackPosition: async (bar: number, beat: number, tick: number): Promise<void> => {
      try {
        await invoke('set_playback_position', { bar, beat, tick });
      } catch (error) {
        console.error('Failed to set playback position:', error);
        throw error;
      }
    },

    /**
     * Get playback state
     * Backend: daw/src-tauri/src/commands/window.rs:94
     */
    getPlaybackState: async (): Promise<PlaybackState> => {
      try {
        return await invoke('get_playback_state');
      } catch (error) {
        console.error('Failed to get playback state:', error);
        throw error;
      }
    },

    /**
     * Set BPM
     * Backend: daw/src-tauri/src/commands/window.rs:107
     */
    setBpm: async (bpm: number): Promise<void> => {
      try {
        await invoke('set_bpm', { bpm });
      } catch (error) {
        console.error('Failed to set BPM:', error);
        throw error;
      }
    },

    /**
     * Get BPM
     * Backend: daw/src-tauri/src/commands/window.rs:119
     */
    getBpm: async (): Promise<number> => {
      try {
        return await invoke('get_bpm');
      } catch (error) {
        console.error('Failed to get BPM:', error);
        throw error;
      }
    },

    /**
     * Set time signature
     * Backend: daw/src-tauri/src/commands/window.rs:126
     */
    setTimeSignature: async (numerator: number, denominator: number): Promise<void> => {
      try {
        await invoke('set_time_signature', { numerator, denominator });
      } catch (error) {
        console.error('Failed to set time signature:', error);
        throw error;
      }
    },

    /**
     * Get time signature
     * Backend: daw/src-tauri/src/commands/window.rs:147
     */
    getTimeSignature: async (): Promise<[number, number]> => {
      try {
        return await invoke('get_time_signature');
      } catch (error) {
        console.error('Failed to get time signature:', error);
        throw error;
      }
    },

    /**
     * Set key signature
     * Backend: daw/src-tauri/src/commands/window.rs:157
     */
    setKeySignature: async (key: string): Promise<void> => {
      try {
        await invoke('set_key_signature', { key });
      } catch (error) {
        console.error('Failed to set key signature:', error);
        throw error;
      }
    },

    /**
     * Get key signature
     * Backend: daw/src-tauri/src/commands/window.rs:173
     */
    getKeySignature: async (): Promise<string> => {
      try {
        return await invoke('get_key_signature');
      } catch (error) {
        console.error('Failed to get key signature:', error);
        throw error;
      }
    },

    /**
     * Add new track to window state
     * Backend: daw/src-tauri/src/commands/window.rs:184
     */
    addWindowTrack: async (label: string): Promise<number> => {
      try {
        return await invoke('add_window_track', { label });
      } catch (error) {
        console.error('Failed to add window track:', error);
        throw error;
      }
    },

    /**
     * Remove track from window state
     * Backend: daw/src-tauri/src/commands/window.rs:202
     */
    removeWindowTrack: async (trackId: number): Promise<void> => {
      try {
        await invoke('remove_window_track', { track_id: trackId });
      } catch (error) {
        console.error('Failed to remove window track:', error);
        throw error;
      }
    },

    /**
     * Get all window tracks
     * Backend: daw/src-tauri/src/commands/window.rs:216
     */
    getAllWindowTracks: async (): Promise<TrackInfo[]> => {
      try {
        return await invoke('get_all_window_tracks');
      } catch (error) {
        console.error('Failed to get all window tracks:', error);
        throw error;
      }
    },

    /**
     * Set track visibility
     * Backend: daw/src-tauri/src/commands/window.rs:225
     */
    setTrackVisible: async (trackId: number, visible: boolean): Promise<void> => {
      try {
        await invoke('set_track_visible', { track_id: trackId, visible });
      } catch (error) {
        console.error('Failed to set track visible:', error);
        throw error;
      }
    },

    /**
     * Set track muted state
     * Backend: daw/src-tauri/src/commands/window.rs:240
     */
    setTrackMuted: async (trackId: number, muted: boolean): Promise<void> => {
      try {
        await invoke('set_track_muted', { track_id: trackId, muted });
      } catch (error) {
        console.error('Failed to set track muted:', error);
        throw error;
      }
    },

    /**
     * Set track soloed state
     * Backend: daw/src-tauri/src/commands/window.rs:262
     */
    setTrackSoloed: async (trackId: number, soloed: boolean): Promise<void> => {
      try {
        await invoke('set_track_soloed', { track_id: trackId, soloed });
      } catch (error) {
        console.error('Failed to set track soloed:', error);
        throw error;
      }
    },

    /**
     * Get track info
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    getTrackInfo: async (trackId: number): Promise<TrackInfo> => {
      try {
        return await invoke('get_track_info', { track_id: trackId });
      } catch (error) {
        console.error('Failed to get track info:', error);
        throw error;
      }
    },

    /**
     * Update track label
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    updateTrackLabel: async (trackId: number, label: string): Promise<void> => {
      try {
        await invoke('update_track_label', { track_id: trackId, label });
      } catch (error) {
        console.error('Failed to update track label:', error);
        throw error;
      }
    },

    /**
     * Set loop enabled
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setLoopEnabled: async (enabled: boolean): Promise<void> => {
      try {
        await invoke('set_loop_enabled', { enabled });
      } catch (error) {
        console.error('Failed to set loop enabled:', error);
        throw error;
      }
    },

    /**
     * Set loop range (start/end in ticks)
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setLoopRange: async (start: number, end: number): Promise<void> => {
      try {
        await invoke('set_loop_range', { start, end });
      } catch (error) {
        console.error('Failed to set loop range:', error);
        throw error;
      }
    },

    /**
     * Set metronome enabled
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setMetronomeEnabled: async (enabled: boolean): Promise<void> => {
      try {
        await invoke('set_metronome_enabled', { enabled });
      } catch (error) {
        console.error('Failed to set metronome enabled:', error);
        throw error;
      }
    },

    /**
     * Set metronome volume (0.0-1.0)
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setMetronomeVolume: async (volume: number): Promise<void> => {
      try {
        await invoke('set_metronome_volume', { volume });
      } catch (error) {
        console.error('Failed to set metronome volume:', error);
        throw error;
      }
    },

    /**
     * Get transport info
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    getTransportInfo: async (): Promise<TransportInfo> => {
      try {
        return await invoke('get_transport_info');
      } catch (error) {
        console.error('Failed to get transport info:', error);
        throw error;
      }
    },

    /**
     * Get mixer state
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    getMixerState: async (): Promise<MixerState> => {
      try {
        return await invoke('get_mixer_state');
      } catch (error) {
        console.error('Failed to get mixer state:', error);
        throw error;
      }
    },

    /**
     * Set channel volume
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setChannelVolume: async (trackId: number, volume: number): Promise<void> => {
      try {
        await invoke('set_channel_volume', { track_id: trackId, volume });
      } catch (error) {
        console.error('Failed to set channel volume:', error);
        throw error;
      }
    },

    /**
     * Set channel pan
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setChannelPan: async (trackId: number, pan: number): Promise<void> => {
      try {
        await invoke('set_channel_pan', { track_id: trackId, pan });
      } catch (error) {
        console.error('Failed to set channel pan:', error);
        throw error;
      }
    },

    /**
     * Set channel mute
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setChannelMute: async (trackId: number, muted: boolean): Promise<void> => {
      try {
        await invoke('set_channel_mute', { track_id: trackId, muted });
      } catch (error) {
        console.error('Failed to set channel mute:', error);
        throw error;
      }
    },

    /**
     * Set channel solo
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setChannelSolo: async (trackId: number, soloed: boolean): Promise<void> => {
      try {
        await invoke('set_channel_solo', { track_id: trackId, soloed });
      } catch (error) {
        console.error('Failed to set channel solo:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // AUTOMATION COMMANDS (12 total) ✅ ALL NEW - COMPLETELY MISSING FROM V1.0
  // ============================================================================

  automation: {
    /**
     * Create automation lane
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    createLane: async (trackId: number, parameterType: ParameterType): Promise<number> => {
      try {
        return await invoke('create_automation_lane', { track_id: trackId, parameter_type: parameterType });
      } catch (error) {
        console.error('Failed to create automation lane:', error);
        throw error;
      }
    },

    /**
     * Delete automation lane
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    deleteLane: async (laneId: number): Promise<void> => {
      try {
        await invoke('delete_automation_lane', { lane_id: laneId });
      } catch (error) {
        console.error('Failed to delete automation lane:', error);
        throw error;
      }
    },

    /**
     * Get all automation lanes for track
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    getAllLanes: async (trackId: number): Promise<AutomationLane[]> => {
      try {
        return await invoke('get_all_automation_lanes', { track_id: trackId });
      } catch (error) {
        console.error('Failed to get all automation lanes:', error);
        throw error;
      }
    },

    /**
     * Add automation point
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    addPoint: async (laneId: number, tick: number, value: number): Promise<number> => {
      try {
        return await invoke('add_automation_point', { lane_id: laneId, tick, value });
      } catch (error) {
        console.error('Failed to add automation point:', error);
        throw error;
      }
    },

    /**
     * Update automation point
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    updatePoint: async (pointId: number, tick: number, value: number): Promise<void> => {
      try {
        await invoke('update_automation_point', { point_id: pointId, tick, value });
      } catch (error) {
        console.error('Failed to update automation point:', error);
        throw error;
      }
    },

    /**
     * Delete automation point
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    deletePoint: async (pointId: number): Promise<void> => {
      try {
        await invoke('delete_automation_point', { point_id: pointId });
      } catch (error) {
        console.error('Failed to delete automation point:', error);
        throw error;
      }
    },

    /**
     * Get automation points in range
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    getPointsInRange: async (laneId: number, startTick: number, endTick: number): Promise<AutomationPoint[]> => {
      try {
        return await invoke('get_automation_points_in_range', { lane_id: laneId, start_tick: startTick, end_tick: endTick });
      } catch (error) {
        console.error('Failed to get automation points in range:', error);
        throw error;
      }
    },

    /**
     * Set curve type for automation lane
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    setCurveType: async (laneId: number, curveType: CurveType): Promise<void> => {
      try {
        await invoke('set_automation_curve_type', { lane_id: laneId, curve_type: curveType });
      } catch (error) {
        console.error('Failed to set automation curve type:', error);
        throw error;
      }
    },

    /**
     * Scale automation values
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    scaleValues: async (laneId: number, factor: number): Promise<void> => {
      try {
        await invoke('scale_automation_values', { lane_id: laneId, factor });
      } catch (error) {
        console.error('Failed to scale automation values:', error);
        throw error;
      }
    },

    /**
     * Offset automation values
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    offsetValues: async (laneId: number, offset: number): Promise<void> => {
      try {
        await invoke('offset_automation_values', { lane_id: laneId, offset });
      } catch (error) {
        console.error('Failed to offset automation values:', error);
        throw error;
      }
    },

    /**
     * Smooth automation values
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    smoothValues: async (laneId: number, windowSize: number): Promise<void> => {
      try {
        await invoke('smooth_automation_values', { lane_id: laneId, window_size: windowSize });
      } catch (error) {
        console.error('Failed to smooth automation values:', error);
        throw error;
      }
    },

    /**
     * Clear automation range
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    clearRange: async (laneId: number, startTick: number, endTick: number): Promise<void> => {
      try {
        await invoke('clear_automation_range', { lane_id: laneId, start_tick: startTick, end_tick: endTick });
      } catch (error) {
        console.error('Failed to clear automation range:', error);
        throw error;
      }
    },
  },
};