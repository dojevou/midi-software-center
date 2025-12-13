import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { Commands } from './api/commands';
import type { AudioSettings, MIDISettings, Settings } from './stores/settingsStore';

/**
 * Wait for Tauri APIs to be ready (handles async injection in Tauri 2.x)
 * The __TAURI_INTERNALS__ may not be available immediately on page load
 */
async function waitForTauri(maxWaitMs = 1000): Promise<boolean> {
  const startTime = Date.now();
  const checkInterval = 50;

  while (Date.now() - startTime < maxWaitMs) {
    // Check multiple possible Tauri indicators
    if (
      typeof window !== 'undefined' &&
      (window.__TAURI_INTERNALS__ !== undefined ||
        window.__TAURI__ !== undefined ||
        (window as unknown as { isTauri?: boolean }).isTauri === true)
    ) {
      return true;
    }
    await new Promise((resolve) => setTimeout(resolve, checkInterval));
  }
  return false;
}

/**
 * Safe invoke wrapper that handles browser context and Tauri 2.x timing gracefully
 */
async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  // First, ensure Tauri APIs are available (handles async injection timing)
  const isTauriReady = () =>
    typeof window !== 'undefined' &&
    (window.__TAURI_INTERNALS__ !== undefined ||
      window.__TAURI__ !== undefined ||
      (window as unknown as { isTauri?: boolean }).isTauri === true);

  // If Tauri APIs aren't ready yet, wait for them
  if (!isTauriReady()) {
    console.log(`[API] Tauri APIs not ready yet, waiting before invoking '${command}'...`);
    const ready = await waitForTauri();
    if (!ready) {
      console.error(`[API] Tauri APIs not available after waiting - '${command}'`);
      throw new Error(`Tauri APIs not available - ${command} unavailable`);
    }
    console.log(`[API] Tauri APIs now ready, proceeding with '${command}'`);
  }

  // Now attempt the invoke
  try {
    console.log(`[API] Invoking command: ${command}`, args || '(no args)');
    const result = await tauriInvoke<T>(command, args);
    console.log(`[API] Command ${command} succeeded:`, result);
    return result;
  } catch (error) {
    // If it's specifically a Tauri context error, provide a clear message
    const errorMessage = String(error);
    if (
      errorMessage.includes('__TAURI_INTERNALS__') ||
      errorMessage.includes('invoke is not a function') ||
      errorMessage.includes('Cannot read properties of undefined')
    ) {
      console.error(`[API] Tauri invoke failed for '${command}':`, error);
      throw new Error(`Tauri context not available - ${command} failed`);
    }
    // Re-throw other errors (these are actual backend errors)
    throw error;
  }
}
// Re-export types for public API consumers
/* eslint-disable @typescript-eslint/no-unused-vars -- intentional re-exports for public API */
import type {
  AnalysisResults,
  AutomationLane,
  AutomationPoint,
  CompatibleFile,
  CurveType,
  DatabaseFilters,
  DatabaseStats,
  DAWWindowState,
  EffectSlot,
  FileDetails,
  FileMetadata,
  FileParams,
  ImportProgress,
  ImportResult,
  ImportStats,
  ImportSummary,
  Loop,
  LoopAnalysis,
  LoopPreview,
  MidiDevice,
  MidiFile,
  MidiPattern,
  MixerState,
  Note,
  ParameterType,
  PipelineProgress,
  PlaybackPosition,
  PlaybackState,
  Project,
  ProjectInfo,
  SearchFilters,
  SearchResponse,
  SearchResults,
  SplitResult,
  Tag,
  TagStats,
  Track,
  TrackDetails,
  TrackInfo,
  TrackProperties,
  TransportInfo,
  VelocityTrack,
} from './types';
/* eslint-enable @typescript-eslint/no-unused-vars */

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
        return await invoke(Commands.MIDI_LIST_DEVICES);
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
        await invoke(Commands.MIDI_CONNECT, { device_name: deviceName });
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
        await invoke(Commands.MIDI_DISCONNECT);
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
        return await invoke(Commands.MIDI_IS_CONNECTED);
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
        return await invoke(Commands.MIDI_GET_CURRENT_DEVICE);
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
        await invoke(Commands.MIDI_SEND_TEST_NOTE, { channel, note, velocity });
      } catch (error) {
        console.error('Failed to send test note:', error);
        throw error;
      }
    },

    /**
     * Get MIDI devices organized by inputs and outputs
     * Used by MIDIDeviceWindow for device management
     */
    getDevices: async (): Promise<{
      inputs: Array<{
        id: string;
        name: string;
        manufacturer: string;
        inputs: number;
        outputs: number;
      }>;
      outputs: Array<{
        id: string;
        name: string;
        manufacturer: string;
        inputs: number;
        outputs: number;
      }>;
    }> => {
      try {
        const devices = await invoke<MidiDevice[]>(Commands.SCAN_MIDI_DEVICES);
        // Transform flat device list into inputs/outputs structure
        const inputs = devices.map((d, idx) => ({
          id: `input-${idx}`,
          name: d.name,
          manufacturer: 'Unknown',
          inputs: 1,
          outputs: 0,
        }));
        const outputs = devices.map((d, idx) => ({
          id: `output-${idx}`,
          name: d.name,
          manufacturer: 'Unknown',
          inputs: 0,
          outputs: 1,
        }));
        return { inputs, outputs };
      } catch (error) {
        console.error('Failed to get MIDI devices:', error);
        return { inputs: [], outputs: [] };
      }
    },

    /**
     * Connect to a MIDI device by ID and type
     * Backend: Uses CONNECT_MIDI_INPUT or CONNECT_MIDI_OUTPUT based on type
     */
    connectDevice: async (deviceId: string, type: 'input' | 'output'): Promise<void> => {
      try {
        const command =
          type === 'input' ? Commands.CONNECT_MIDI_INPUT : Commands.CONNECT_MIDI_OUTPUT;
        await invoke(command, { device_id: deviceId });
      } catch (error) {
        console.error(`Failed to connect ${type} device:`, error);
        throw error;
      }
    },

    /**
     * Disconnect from a MIDI device by ID and type
     * Backend: Uses DISCONNECT_MIDI_INPUT or DISCONNECT_MIDI_OUTPUT based on type
     */
    disconnectDevice: async (deviceId: string, type: 'input' | 'output'): Promise<void> => {
      try {
        const command =
          type === 'input' ? Commands.DISCONNECT_MIDI_INPUT : Commands.DISCONNECT_MIDI_OUTPUT;
        await invoke(command, { device_id: deviceId });
      } catch (error) {
        console.error(`Failed to disconnect ${type} device:`, error);
        throw error;
      }
    },

    /**
     * Test a MIDI device by sending a test message
     * Backend: Uses SEND_MIDI_MESSAGE for outputs, or listens briefly for inputs
     */
    testDevice: async (deviceId: string, type: 'input' | 'output'): Promise<void> => {
      try {
        if (type === 'output') {
          // Send a test note (middle C at medium velocity)
          await invoke(Commands.SEND_MIDI_MESSAGE, {
            device_id: deviceId,
            message: [0x90, 60, 64], // Note On, middle C, velocity 64
          });
          // Send note off after a short delay
          await new Promise((resolve) => setTimeout(resolve, 200));
          await invoke(Commands.SEND_MIDI_MESSAGE, {
            device_id: deviceId,
            message: [0x80, 60, 0], // Note Off, middle C
          });
        }
        // For inputs, testing is passive (handled by device monitoring)
      } catch (error) {
        console.error(`Failed to test ${type} device:`, error);
        throw error;
      }
    },
  },

  // ============================================================================
  // AUDIO DEVICE COMMANDS
  // ============================================================================

  audio: {
    /**
     * Get available audio input and output devices
     * Returns: { inputs: AudioDevice[], outputs: AudioDevice[] }
     */
    getDevices: async (): Promise<{
      inputs: { id: string; name: string }[];
      outputs: { id: string; name: string }[];
    }> => {
      try {
        return await invoke(Commands.AUDIO_GET_DEVICES);
      } catch (error) {
        console.error('Failed to get audio devices:', error);
        // Return empty arrays as fallback for development
        return { inputs: [], outputs: [] };
      }
    },

    /**
     * Test audio output by playing a test tone
     */
    testOutput: async (): Promise<void> => {
      try {
        await invoke(Commands.AUDIO_TEST_OUTPUT);
      } catch (error) {
        console.error('Failed to test audio output:', error);
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
        await invoke(Commands.START_SEQUENCER);
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
        await invoke(Commands.STOP_SEQUENCER);
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
        await invoke(Commands.PAUSE_SEQUENCER);
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
        await invoke(Commands.RESUME_SEQUENCER);
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
        return await invoke(Commands.GET_PLAYBACK_POSITION);
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
        await invoke(Commands.SEEK_POSITION, { bar, beat });
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
        await invoke(Commands.SET_TEMPO, { bpm });
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
        return await invoke(Commands.GET_TEMPO);
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
        return await invoke(Commands.ADD_TRACK, { file_id: fileId, channel });
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
        await invoke(Commands.REMOVE_TRACK, { track_id: trackId });
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
        await invoke(Commands.UPDATE_TRACK, { track_id: trackId, properties });
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
        return await invoke(Commands.GET_TRACKS);
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
        await invoke(Commands.LOAD_SEQUENCER_TRACKS);
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
        return await invoke(Commands.IS_SEQUENCER_PLAYING);
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
        return await invoke(Commands.SEARCH_FILES, { filters });
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
        return await invoke(Commands.GET_FILE_DETAILS, { file_id: fileId });
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
        return await invoke(Commands.GET_SEARCH_SUGGESTIONS, { query, field });
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
        return await invoke(Commands.FIND_COMPATIBLE_FILES, {
          file_id: fileId,
          max_results: maxResults,
        });
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
        await invoke(Commands.ADD_FAVORITE, { file_id: fileId });
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
        await invoke(Commands.REMOVE_FAVORITE, { file_id: fileId });
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
        return await invoke(Commands.IS_FAVORITE, { file_id: fileId });
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
        return await invoke(Commands.GET_FAVORITES);
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
        return await invoke(Commands.GET_USAGE_STATS);
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
        return await invoke(Commands.LOAD_MULTIPLE_TRACKS, { file_ids: fileIds });
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
        await invoke(Commands.CLEAR_ALL_TRACKS);
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
        return await invoke(Commands.GET_TRACK_DETAILS);
      } catch (error) {
        console.error('Failed to get track details:', error);
        throw error;
      }
    },

    /**
     * Create a new project
     */
    createNewProject: async (): Promise<void> => {
      try {
        await invoke(Commands.CREATE_NEW_PROJECT);
      } catch (error) {
        console.error('Failed to create new project:', error);
        throw error;
      }
    },

    /**
     * Open project dialog
     */
    openProjectDialog: async (): Promise<void> => {
      try {
        await invoke(Commands.OPEN_PROJECT_DIALOG);
      } catch (error) {
        console.error('Failed to open project dialog:', error);
        throw error;
      }
    },

    /**
     * Save current project
     */
    saveProject: async (): Promise<void> => {
      try {
        await invoke(Commands.SAVE_PROJECT);
      } catch (error) {
        console.error('Failed to save project:', error);
        throw error;
      }
    },

    /**
     * Get all tracks
     */
    getTracks: async (): Promise<Track[]> => {
      try {
        return await invoke(Commands.GET_TRACKS);
      } catch (error) {
        console.error('Failed to get tracks:', error);
        throw error;
      }
    },

    /**
     * Get project by ID
     * Backend: daw/src-tauri/src/commands/project.rs:project_get
     */
    get: async (id: number): Promise<Project | null> => {
      try {
        return await invoke(Commands.PROJECT_GET, { id });
      } catch (error) {
        console.error('Failed to get project:', error);
        throw error;
      }
    },

    /**
     * Get all projects
     * Backend: daw/src-tauri/src/commands/project.rs:project_list
     */
    getProjects: async (): Promise<Project[]> => {
      try {
        return await invoke(Commands.PROJECT_LIST);
      } catch (error) {
        console.error('Failed to get projects:', error);
        throw error;
      }
    },

    /**
     * Get recent projects
     * Backend: daw/src-tauri/src/commands/project.rs:project_get_recent
     */
    getRecent: async (limit?: number): Promise<Project[]> => {
      try {
        return await invoke(Commands.PROJECT_GET_RECENT, { limit: limit ?? 10 });
      } catch (error) {
        console.error('Failed to get recent projects:', error);
        throw error;
      }
    },

    /**
     * Load a project by ID (updates last_opened_at)
     * Backend: daw/src-tauri/src/commands/project.rs:project_load
     */
    load: async (id: number): Promise<Project | null> => {
      try {
        return await invoke(Commands.PROJECT_LOAD, { id });
      } catch (error) {
        console.error('Failed to load project:', error);
        throw error;
      }
    },

    /**
     * Create a new project with data
     * Backend: daw/src-tauri/src/commands/project.rs:project_create
     */
    create: async (params: {
      name: string;
      description?: string;
      bpm?: number;
      time_signature_numerator?: number;
      time_signature_denominator?: number;
      key_signature?: string;
      sample_rate?: number;
      bit_depth?: number;
    }): Promise<Project> => {
      try {
        return await invoke(Commands.PROJECT_CREATE, { params });
      } catch (error) {
        console.error('Failed to create project:', error);
        throw error;
      }
    },

    /**
     * Update an existing project
     * Backend: daw/src-tauri/src/commands/project.rs:project_update
     */
    update: async (
      id: number,
      params: {
        name?: string;
        description?: string;
        bpm?: number;
        time_signature_numerator?: number;
        time_signature_denominator?: number;
        key_signature?: string;
        sample_rate?: number;
        bit_depth?: number;
      }
    ): Promise<Project> => {
      try {
        return await invoke(Commands.PROJECT_UPDATE, { id, params });
      } catch (error) {
        console.error('Failed to update project:', error);
        throw error;
      }
    },

    /**
     * Delete a project by ID
     * Backend: daw/src-tauri/src/commands/project.rs:project_delete
     */
    delete: async (id: number): Promise<boolean> => {
      try {
        return await invoke(Commands.PROJECT_DELETE, { id });
      } catch (error) {
        console.error('Failed to delete project:', error);
        throw error;
      }
    },

    /**
     * Duplicate a project by ID
     */
    duplicate: async (projectId: number): Promise<Project> => {
      try {
        return await invoke(Commands.DUPLICATE_PROJECT, { project_id: projectId });
      } catch (error) {
        console.error('Failed to duplicate project:', error);
        throw error;
      }
    },

    /**
     * Export a project as archive
     */
    exportArchive: async (projectId: number, exportPath: string): Promise<void> => {
      try {
        await invoke(Commands.EXPORT_PROJECT_ARCHIVE, {
          project_id: projectId,
          export_path: exportPath,
        });
      } catch (error) {
        console.error('Failed to export project archive:', error);
        throw error;
      }
    },

    /**
     * Import a project from file
     */
    import: async (filePath: string): Promise<Project> => {
      try {
        return await invoke(Commands.IMPORT_PROJECT, { file_path: filePath });
      } catch (error) {
        console.error('Failed to import project:', error);
        throw error;
      }
    },

    /**
     * Get current project info (name, length, bpm)
     * Used by ExportWindow to populate export options
     */
    getInfo: async (): Promise<ProjectInfo> => {
      try {
        const info = await invoke<ProjectInfo | null>(Commands.GET_PROJECT_INFO, {});
        return info || { name: 'Untitled Project', length: 1920 * 16, bpm: 120 };
      } catch (error) {
        console.error('Failed to get project info:', error);
        // Return default values if no project is loaded
        return { name: 'Untitled Project', length: 1920 * 16, bpm: 120 };
      }
    },

    /**
     * Open a project by ID (loads project into the DAW)
     * Used by ProjectBrowserWindow to open selected projects
     */
    open: async (id: number): Promise<Project | null> => {
      try {
        return await invoke(Commands.PROJECT_LOAD, { id });
      } catch (error) {
        console.error('Failed to open project:', error);
        throw error;
      }
    },

    /**
     * Save tracks and clips to a project
     * Backend: daw/src-tauri/src/commands/project.rs:project_save_tracks
     */
    saveTracks: async (
      projectId: number,
      tracks: Array<{
        id?: number;
        name: string;
        color: string;
        midiChannel: number;
        trackNumber: number;
        volume: number;
        pan: number;
        isMuted: boolean;
        isSoloed: boolean;
        isArmed: boolean;
        clips: Array<{
          id?: number;
          name: string;
          color: string;
          startTick: number;
          durationTicks: number;
          sourceFileId: number | null;
          isMuted: boolean;
          isSelected: boolean;
          isLocked: boolean;
          sourceStartTick: number;
          sourceEndTick: number | null;
          gainDb: number;
          loopEnabled: boolean;
          loopStartTick: number;
          loopEndTick: number | null;
        }>;
      }>
    ): Promise<void> => {
      try {
        await invoke(Commands.PROJECT_SAVE_TRACKS, { project_id: projectId, tracks });
      } catch (error) {
        console.error('Failed to save project tracks:', error);
        throw error;
      }
    },

    /**
     * Load tracks and clips from a project
     * Backend: daw/src-tauri/src/commands/project.rs:project_load_tracks
     */
    loadTracks: async (
      projectId: number
    ): Promise<{
      tracks: Array<{
        id: number;
        projectId: number;
        name: string;
        color: string;
        midiChannel: number;
        trackNumber: number;
        volume: number;
        pan: number;
        isMuted: boolean;
        isSoloed: boolean;
        isArmed: boolean;
        clips: Array<{
          id: number;
          trackId: number;
          name: string;
          color: string;
          startTick: number;
          durationTicks: number;
          sourceFileId: number | null;
          isMuted: boolean;
          isSelected: boolean;
          isLocked: boolean;
          sourceStartTick: number;
          sourceEndTick: number | null;
          gainDb: number;
          loopEnabled: boolean;
          loopStartTick: number;
          loopEndTick: number | null;
        }>;
      }>;
    }> => {
      try {
        return await invoke(Commands.PROJECT_LOAD_TRACKS, { project_id: projectId });
      } catch (error) {
        console.error('Failed to load project tracks:', error);
        throw error;
      }
    },

    /**
     * Load full project data including metadata, tracks, and clips
     * Backend: daw/src-tauri/src/commands/project.rs:project_load_full
     */
    loadFull: async (
      projectId: number
    ): Promise<{
      project: Project;
      tracks: Array<{
        id: number;
        projectId: number;
        name: string;
        color: string;
        midiChannel: number;
        trackNumber: number;
        volume: number;
        pan: number;
        isMuted: boolean;
        isSoloed: boolean;
        isArmed: boolean;
        clips: Array<{
          id: number;
          trackId: number;
          name: string;
          color: string;
          startTick: number;
          durationTicks: number;
          sourceFileId: number | null;
          isMuted: boolean;
          isSelected: boolean;
          isLocked: boolean;
          sourceStartTick: number;
          sourceEndTick: number | null;
          gainDb: number;
          loopEnabled: boolean;
          loopStartTick: number;
          loopEndTick: number | null;
        }>;
      }>;
    }> => {
      try {
        return await invoke(Commands.PROJECT_LOAD_FULL, { project_id: projectId });
      } catch (error) {
        console.error('Failed to load full project:', error);
        throw error;
      }
    },

    /**
     * Delete all tracks and clips from a project
     * Backend: daw/src-tauri/src/commands/project.rs:project_delete_tracks
     */
    deleteTracks: async (projectId: number): Promise<void> => {
      try {
        await invoke(Commands.PROJECT_DELETE_TRACKS, { project_id: projectId });
      } catch (error) {
        console.error('Failed to delete project tracks:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // EXPORT COMMANDS (4 total)
  // ============================================================================

  export: {
    /**
     * Export project as MIDI file
     * Backend: daw/src-tauri/src/commands/export.rs
     */
    projectAsMidi: async (outputPath: string): Promise<void> => {
      try {
        await invoke(Commands.EXPORT_PROJECT_MIDI, { output_path: outputPath });
      } catch (error) {
        console.error('Failed to export project as MIDI:', error);
        throw error;
      }
    },

    /**
     * Export project with parameters
     */
    exportProject: async (params: unknown): Promise<{ jobId: string }> => {
      try {
        return await invoke(Commands.EXPORT_PROJECT, { params });
      } catch (error) {
        console.error('Failed to export project:', error);
        throw error;
      }
    },

    /**
     * Get export progress
     */
    getProgress: async (
      jobId: string
    ): Promise<{
      status: 'idle' | 'preparing' | 'exporting' | 'completed' | 'error';
      current: number;
      total: number;
      message: string;
      outputPath?: string;
    }> => {
      try {
        return await invoke(Commands.GET_EXPORT_PROGRESS, { job_id: jobId });
      } catch (error) {
        console.error('Failed to get export progress:', error);
        throw error;
      }
    },

    /**
     * Export a single file
     */
    exportFile: async (fileId: number): Promise<void> => {
      try {
        await invoke(Commands.EXPORT_FILE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to export file:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // PIPELINE COMMANDS (5 total) - Import, Analysis, Archive, Progress
  // Backend: daw/src-tauri/src/commands/pipeline.rs
  // ============================================================================

  pipeline: {
    /**
     * Import multiple MIDI files
     * Backend: daw/src-tauri/src/commands/pipeline.rs:56
     */
    importFiles: async (filePaths: string[]): Promise<ImportStats> => {
      try {
        return await invoke(Commands.PIPELINE_IMPORT_FILES, { file_paths: filePaths });
      } catch (error) {
        console.error('Failed to import files:', error);
        throw error;
      }
    },

    /**
     * Analyze MIDI files by IDs
     * Backend: daw/src-tauri/src/commands/pipeline.rs:156
     */
    analyzeFiles: async (fileIds: number[]): Promise<AnalysisResults> => {
      try {
        return await invoke(Commands.PIPELINE_ANALYZE_FILES, { file_ids: fileIds });
      } catch (error) {
        console.error('Failed to analyze files:', error);
        throw error;
      }
    },

    /**
     * Archive MIDI files to ZIP
     * Backend: daw/src-tauri/src/commands/pipeline.rs:254
     */
    archiveFiles: async (fileIds: number[], archivePath: string): Promise<ImportStats> => {
      try {
        return await invoke(Commands.PIPELINE_ARCHIVE_FILES, {
          file_ids: fileIds,
          archive_path: archivePath,
        });
      } catch (error) {
        console.error('Failed to archive files:', error);
        throw error;
      }
    },

    /**
     * Get current pipeline progress
     * Backend: daw/src-tauri/src/commands/pipeline.rs:356
     */
    getProgress: async (): Promise<PipelineProgress> => {
      try {
        return await invoke(Commands.PIPELINE_GET_PROGRESS);
      } catch (error) {
        console.error('Failed to get progress:', error);
        throw error;
      }
    },

    /**
     * Cancel current pipeline operation
     * Backend: daw/src-tauri/src/commands/pipeline.rs:362
     */
    cancel: async (): Promise<void> => {
      try {
        await invoke(Commands.PIPELINE_CANCEL);
      } catch (error) {
        console.error('Failed to cancel pipeline:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // DATABASE COMMANDS (5 total) - Search, Metadata, Add/Remove, Stats
  // Backend: daw/src-tauri/src/commands/database.rs
  // ============================================================================

  database: {
    /**
     * Search database with filters
     * Backend: daw/src-tauri/src/commands/database.rs:48
     */
    search: async (filters: DatabaseFilters): Promise<SearchResults> => {
      try {
        return await invoke(Commands.DATABASE_SEARCH, { filters });
      } catch (error) {
        console.error('Database search failed:', error);
        throw error;
      }
    },

    /**
     * Get file metadata by ID
     * Backend: daw/src-tauri/src/commands/database.rs:131
     */
    getFileMetadata: async (id: number): Promise<MidiFile | null> => {
      try {
        return await invoke(Commands.DATABASE_GET_FILE_METADATA, { id });
      } catch (error) {
        console.error('Get file metadata failed:', error);
        throw error;
      }
    },

    /**
     * Add file to database
     * Backend: daw/src-tauri/src/commands/database.rs:166
     */
    addFile: async (params: FileParams): Promise<number> => {
      try {
        return await invoke(Commands.DATABASE_ADD_FILE, { ...params });
      } catch (error) {
        console.error('Add file failed:', error);
        throw error;
      }
    },

    /**
     * Remove file from database
     * Backend: daw/src-tauri/src/commands/database.rs:201
     */
    removeFile: async (id: number): Promise<void> => {
      try {
        await invoke(Commands.DATABASE_REMOVE_FILE, { id });
      } catch (error) {
        console.error('Remove file failed:', error);
        throw error;
      }
    },

    /**
     * Get database statistics
     * Backend: daw/src-tauri/src/commands/database.rs:219
     */
    getStats: async (): Promise<DatabaseStats> => {
      try {
        return await invoke(Commands.DATABASE_GET_STATS);
      } catch (error) {
        console.error('Get stats failed:', error);
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
        return await invoke(Commands.GET_DAW_STATE);
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
        await invoke(Commands.RESET_DAW_STATE);
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
        await invoke(Commands.PLAY_TRANSPORT);
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
        await invoke(Commands.STOP_TRANSPORT);
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
        await invoke(Commands.PAUSE_TRANSPORT);
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
        await invoke(Commands.SET_PLAYBACK_POSITION, { bar, beat, tick });
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
        return await invoke(Commands.GET_PLAYBACK_STATE);
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
        await invoke(Commands.SET_BPM, { bpm });
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
        return await invoke(Commands.GET_BPM);
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
        await invoke(Commands.SET_TIME_SIGNATURE, { numerator, denominator });
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
        return await invoke(Commands.GET_TIME_SIGNATURE);
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
        await invoke(Commands.SET_KEY_SIGNATURE, { key });
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
        return await invoke(Commands.GET_KEY_SIGNATURE);
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
        return await invoke(Commands.ADD_WINDOW_TRACK, { label });
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
        await invoke(Commands.REMOVE_WINDOW_TRACK, { track_id: trackId });
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
        return await invoke(Commands.GET_ALL_WINDOW_TRACKS);
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
        await invoke(Commands.SET_TRACK_VISIBLE, { track_id: trackId, visible });
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
        await invoke(Commands.SET_TRACK_MUTED, { track_id: trackId, muted });
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
        await invoke(Commands.SET_TRACK_SOLOED, { track_id: trackId, soloed });
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
        return await invoke(Commands.GET_TRACK_INFO, { track_id: trackId });
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
        await invoke(Commands.UPDATE_TRACK_LABEL, { track_id: trackId, label });
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
        await invoke(Commands.SET_LOOP_ENABLED, { enabled });
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
        await invoke(Commands.SET_LOOP_RANGE, { start, end });
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
        await invoke(Commands.SET_METRONOME_ENABLED, { enabled });
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
        await invoke(Commands.SET_METRONOME_VOLUME, { volume });
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
        return await invoke(Commands.GET_TRANSPORT_INFO);
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
      console.log('[API] getMixerState called, command:', Commands.GET_MIXER_STATE);
      try {
        console.log('[API] About to invoke get_mixer_state...');
        const result = await invoke<MixerState>(Commands.GET_MIXER_STATE);
        console.log('[API] getMixerState result:', result);
        return result;
      } catch (error) {
        console.error('[API] getMixerState failed:', error);
        throw error;
      }
    },

    /**
     * Set channel volume
     * Backend: daw/src-tauri/src/commands/window.rs
     */
    setChannelVolume: async (trackId: number, volume: number): Promise<void> => {
      try {
        await invoke(Commands.SET_CHANNEL_VOLUME, { track_id: trackId, volume });
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
        await invoke(Commands.SET_CHANNEL_PAN, { track_id: trackId, pan });
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
        await invoke(Commands.SET_CHANNEL_MUTE, { track_id: trackId, muted });
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
        await invoke(Commands.SET_CHANNEL_SOLO, { track_id: trackId, soloed });
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
    createLane: async (trackId: number, parameterType: ParameterType): Promise<AutomationLane> => {
      try {
        return await invoke(Commands.CREATE_AUTOMATION_LANE, {
          track_id: trackId,
          parameter_type: parameterType,
        });
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
        await invoke(Commands.DELETE_AUTOMATION_LANE, { lane_id: laneId });
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
        return await invoke(Commands.GET_ALL_AUTOMATION_LANES, { track_id: trackId });
      } catch (error) {
        console.error('Failed to get all automation lanes:', error);
        throw error;
      }
    },

    /**
     * Add automation point
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    addPoint: async (laneId: number, tick: number, value: number): Promise<AutomationPoint> => {
      try {
        return await invoke(Commands.ADD_AUTOMATION_POINT, { lane_id: laneId, tick, value });
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
        await invoke(Commands.UPDATE_AUTOMATION_POINT, { point_id: pointId, tick, value });
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
        await invoke(Commands.DELETE_AUTOMATION_POINT, { point_id: pointId });
      } catch (error) {
        console.error('Failed to delete automation point:', error);
        throw error;
      }
    },

    /**
     * Get automation points in range
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    getPointsInRange: async (
      laneId: number,
      startTick: number,
      endTick: number
    ): Promise<AutomationPoint[]> => {
      try {
        return await invoke(Commands.GET_AUTOMATION_POINTS_IN_RANGE, {
          lane_id: laneId,
          start_tick: startTick,
          end_tick: endTick,
        });
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
        await invoke(Commands.SET_AUTOMATION_CURVE_TYPE, {
          lane_id: laneId,
          curve_type: curveType,
        });
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
        await invoke(Commands.SCALE_AUTOMATION_VALUES, { lane_id: laneId, factor });
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
        await invoke(Commands.OFFSET_AUTOMATION_VALUES, { lane_id: laneId, offset });
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
        await invoke(Commands.SMOOTH_AUTOMATION_VALUES, {
          lane_id: laneId,
          window_size: windowSize,
        });
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
        await invoke(Commands.CLEAR_AUTOMATION_RANGE, {
          lane_id: laneId,
          start_tick: startTick,
          end_tick: endTick,
        });
      } catch (error) {
        console.error('Failed to clear automation range:', error);
        throw error;
      }
    },

    /**
     * Delete multiple automation points (batch operation)
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    deletePointsBatch: async (
      trackId: number,
      parameterType: ParameterType,
      pointIds: number[]
    ): Promise<number> => {
      try {
        return await invoke(Commands.DELETE_AUTOMATION_POINTS_BATCH, {
          track_id: trackId,
          parameter_type: parameterType,
          point_ids: pointIds,
        });
      } catch (error) {
        console.error('Failed to delete automation points batch:', error);
        throw error;
      }
    },

    // ========== CC Recording Methods ==========

    /**
     * Start CC recording from MIDI input
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    startCCRecording: async (trackId: number, ccNumber: number, startTick: number): Promise<void> => {
      try {
        await invoke(Commands.START_CC_RECORDING, {
          track_id: trackId,
          cc_number: ccNumber,
          start_tick: startTick,
        });
      } catch (error) {
        console.error('Failed to start CC recording:', error);
        throw error;
      }
    },

    /**
     * Record a single CC value
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    recordCCValue: async (trackId: number, ccNumber: number, value: number, tick: number): Promise<number> => {
      try {
        return await invoke(Commands.RECORD_CC_VALUE, {
          track_id: trackId,
          cc_number: ccNumber,
          value,
          tick,
        });
      } catch (error) {
        console.error('Failed to record CC value:', error);
        throw error;
      }
    },

    /**
     * Stop CC recording
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    stopCCRecording: async (trackId: number, ccNumber: number): Promise<void> => {
      try {
        await invoke(Commands.STOP_CC_RECORDING, {
          track_id: trackId,
          cc_number: ccNumber,
        });
      } catch (error) {
        console.error('Failed to stop CC recording:', error);
        throw error;
      }
    },

    /**
     * Record multiple CC values in batch
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    recordCCAutomationBatch: async (
      trackId: number,
      ccNumber: number,
      values: Array<[number, number]>
    ): Promise<number> => {
      try {
        return await invoke(Commands.RECORD_CC_AUTOMATION_BATCH, {
          track_id: trackId,
          cc_number: ccNumber,
          values,
        });
      } catch (error) {
        console.error('Failed to record CC automation batch:', error);
        throw error;
      }
    },

    // ========== Copy/Paste Automation Methods ==========

    /**
     * Copy automation points to clipboard
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    copyAutomationPoints: async (
      trackId: number,
      parameterType: ParameterType,
      pointIds: number[]
    ): Promise<number> => {
      try {
        return await invoke(Commands.COPY_AUTOMATION_POINTS, {
          track_id: trackId,
          parameter_type: parameterType,
          point_ids: pointIds,
        });
      } catch (error) {
        console.error('Failed to copy automation points:', error);
        throw error;
      }
    },

    /**
     * Paste automation points at specified position
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    pasteAutomationPoints: async (
      trackId: number,
      parameterType: ParameterType,
      pasteTick: number,
      points: Array<[number, number]>
    ): Promise<number[]> => {
      try {
        return await invoke(Commands.PASTE_AUTOMATION_POINTS, {
          track_id: trackId,
          parameter_type: parameterType,
          paste_tick: pasteTick,
          points,
        });
      } catch (error) {
        console.error('Failed to paste automation points:', error);
        throw error;
      }
    },

    /**
     * Cut automation points (copy + delete)
     * Backend: daw/src-tauri/src/commands/automation.rs
     */
    cutAutomationPoints: async (
      trackId: number,
      parameterType: ParameterType,
      pointIds: number[]
    ): Promise<Array<[number, number]>> => {
      try {
        return await invoke(Commands.CUT_AUTOMATION_POINTS, {
          track_id: trackId,
          parameter_type: parameterType,
          point_ids: pointIds,
        });
      } catch (error) {
        console.error('Failed to cut automation points:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // TAG COMMANDS (10 total)
  // Backend: pipeline/src-tauri/src/commands/tags.rs
  // ============================================================================

  tags: {
    /**
     * Get all tags from database
     */
    getAll: async (): Promise<Tag[]> => {
      try {
        return await invoke(Commands.GET_ALL_TAGS);
      } catch (error) {
        console.error('Failed to get all tags:', error);
        throw error;
      }
    },

    /**
     * Get popular tags (most used)
     */
    getPopular: async (limit: number = 50): Promise<Tag[]> => {
      try {
        return await invoke(Commands.GET_POPULAR_TAGS, { limit });
      } catch (error) {
        console.error('Failed to get popular tags:', error);
        throw error;
      }
    },

    /**
     * Search tags by query
     */
    search: async (query: string): Promise<Tag[]> => {
      try {
        return await invoke(Commands.SEARCH_TAGS, { query });
      } catch (error) {
        console.error('Failed to search tags:', error);
        throw error;
      }
    },

    /**
     * Get all tag categories
     */
    getCategories: async (): Promise<string[]> => {
      try {
        return await invoke(Commands.GET_TAG_CATEGORIES);
      } catch (error) {
        console.error('Failed to get tag categories:', error);
        throw error;
      }
    },

    /**
     * Get all tag categories (alias for getCategories)
     */
    getTagCategories: async (): Promise<string[]> => {
      try {
        return await invoke(Commands.GET_TAG_CATEGORIES);
      } catch (error) {
        console.error('Failed to get tag categories:', error);
        throw error;
      }
    },

    /**
     * Get tags by category
     */
    getByCategory: async (category: string): Promise<Tag[]> => {
      try {
        return await invoke(Commands.GET_TAGS_BY_CATEGORY, { category });
      } catch (error) {
        console.error('Failed to get tags by category:', error);
        throw error;
      }
    },

    /**
     * Get tags for a specific file
     */
    getFileTags: async (fileId: number): Promise<string[]> => {
      try {
        return await invoke(Commands.GET_FILE_TAGS, { file_id: fileId });
      } catch (error) {
        console.error('Failed to get file tags:', error);
        throw error;
      }
    },

    /**
     * Add tags to a file
     */
    addToFile: async (fileId: number, tags: string[]): Promise<void> => {
      try {
        await invoke(Commands.ADD_TAGS_TO_FILE, { file_id: fileId, tags });
      } catch (error) {
        console.error('Failed to add tags to file:', error);
        throw error;
      }
    },

    /**
     * Remove a tag from a file
     */
    removeFromFile: async (fileId: number, tag: string): Promise<void> => {
      try {
        await invoke(Commands.REMOVE_TAG_FROM_FILE, { file_id: fileId, tag });
      } catch (error) {
        console.error('Failed to remove tag from file:', error);
        throw error;
      }
    },

    /**
     * Update all tags for a file (replace)
     */
    updateFileTags: async (fileId: number, tags: string[]): Promise<void> => {
      try {
        await invoke(Commands.UPDATE_FILE_TAGS, { file_id: fileId, tags });
      } catch (error) {
        console.error('Failed to update file tags:', error);
        throw error;
      }
    },

    /**
     * Get files that have specific tags
     */
    getFilesByTags: async (tags: string[]): Promise<number[]> => {
      try {
        return await invoke(Commands.GET_FILES_BY_TAGS, { tags });
      } catch (error) {
        console.error('Failed to get files by tags:', error);
        throw error;
      }
    },

    /**
     * Get tag statistics
     */
    getStats: async (): Promise<TagStats> => {
      try {
        return await invoke(Commands.GET_TAG_STATS);
      } catch (error) {
        console.error('Failed to get tag stats:', error);
        throw error;
      }
    },

    /**
     * Get tag statistics (alias for getStats)
     */
    getTagStats: async (): Promise<TagStats> => {
      try {
        return await invoke(Commands.GET_TAG_STATS);
      } catch (error) {
        console.error('Failed to get tag stats:', error);
        throw error;
      }
    },

    /**
     * Get all tags (alias for getAll)
     */
    getAllTags: async (): Promise<Tag[]> => {
      try {
        return await invoke(Commands.GET_ALL_TAGS);
      } catch (error) {
        console.error('Failed to get all tags:', error);
        throw error;
      }
    },

    /**
     * Create a new tag
     * @param tag - Tag data to create
     * @returns Created tag
     */
    createTag: async (tag: {
      name: string;
      category: string;
      description?: string;
    }): Promise<Tag> => {
      return invoke<Tag>(Commands.CREATE_TAG, { tag });
    },

    /**
     * Update an existing tag
     * @param tagId - ID of tag to update
     * @param updates - Fields to update
     */
    updateTag: async (
      tagId: number,
      updates: Partial<{ name: string; category: string; description: string }>
    ): Promise<void> => {
      return invoke<void>(Commands.UPDATE_TAG, { tag_id: tagId, updates });
    },

    /**
     * Delete a tag
     * @param tagId - ID of tag to delete
     */
    deleteTag: async (tagId: number): Promise<void> => {
      return invoke<void>(Commands.DELETE_TAG, { tag_id: tagId });
    },

    /**
     * Merge multiple tags into one
     * @param sourceTagIds - Tags to merge from
     * @param targetTagId - Tag to merge into
     */
    mergeTags: async (sourceTagIds: number[], targetTagId: number): Promise<void> => {
      return invoke<void>(Commands.MERGE_TAGS, {
        source_tag_ids: sourceTagIds,
        target_tag_id: targetTagId,
      });
    },

    /**
     * Export all tags to CSV format
     * @returns CSV string of all tags
     */
    exportTagsCsv: async (): Promise<string> => {
      return invoke<string>(Commands.EXPORT_TAGS_CSV);
    },

    /**
     * Import tags from CSV string
     * @param csv - CSV data to import
     * @returns Import result with counts of imported, skipped, and errored rows
     */
    importTagsCsv: async (csv: string): Promise<ImportResult> => {
      return invoke<ImportResult>(Commands.IMPORT_TAGS_CSV, { csv });
    },
  },

  // ============================================================================
  // MIXER EFFECT COMMANDS (Extended)
  // Backend: daw/src-tauri/src/commands/mixer.rs
  // ============================================================================

  mixerEffects: {
    /**
     * Set an effect parameter value
     */
    setParameter: async (
      trackId: number,
      effectId: number,
      parameterName: string,
      value: number
    ): Promise<void> => {
      try {
        await invoke(Commands.MIXER_SET_EFFECT_PARAMETER, {
          track_id: trackId,
          effect_id: effectId,
          parameter_name: parameterName,
          value,
        });
      } catch (error) {
        console.error('Failed to set effect parameter:', error);
        throw error;
      }
    },

    /**
     * Reorder effects on a channel
     */
    reorder: async (trackId: number, effectIds: number[]): Promise<void> => {
      try {
        await invoke(Commands.MIXER_REORDER_EFFECTS, {
          track_id: trackId,
          effect_ids: effectIds,
        });
      } catch (error) {
        console.error('Failed to reorder effects:', error);
        throw error;
      }
    },

    /**
     * Toggle an effect's enabled state on a channel
     * Returns the new enabled state
     */
    toggleChannelEffect: async (trackId: number, effectId: number): Promise<boolean> => {
      try {
        return await invoke(Commands.MIXER_TOGGLE_CHANNEL_EFFECT, {
          track_id: trackId,
          effect_id: effectId,
        });
      } catch (error) {
        console.error('Failed to toggle channel effect:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // SPLIT FILE COMMANDS (2 total)
  // Backend: pipeline/src-tauri/src/commands/split_file.rs
  // ============================================================================

  split: {
    /**
     * Split a single file into individual tracks
     */
    file: async (fileId: number, outputDir: string): Promise<SplitResult> => {
      try {
        return await invoke(Commands.SPLIT_FILE, {
          file_id: fileId,
          output_dir: outputDir,
        });
      } catch (error) {
        console.error('Failed to split file:', error);
        throw error;
      }
    },

    /**
     * Split multiple files in batch
     */
    batch: async (fileIds: number[], outputDir: string): Promise<SplitResult[]> => {
      try {
        return await invoke(Commands.SPLIT_FILE_BATCH, {
          file_ids: fileIds,
          output_dir: outputDir,
        });
      } catch (error) {
        console.error('Failed to batch split files:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // VELOCITY EDITOR COMMANDS (4 total)
  // Backend: daw/src-tauri/src/commands/velocity.rs
  // ============================================================================

  velocityEditor: {
    /**
     * Update note velocities
     */
    updateNoteVelocities: async (velocities: Record<string, number>): Promise<void> => {
      try {
        await invoke(Commands.UPDATE_NOTE_VELOCITIES, { velocities });
      } catch (error) {
        console.error('Failed to update note velocities:', error);
        throw error;
      }
    },

    /**
     * Get note velocities for a track
     */
    getNoteVelocities: async (trackId: number): Promise<Record<string, number>> => {
      try {
        return await invoke(Commands.GET_NOTE_VELOCITIES, { track_id: trackId });
      } catch (error) {
        console.error('Failed to get note velocities:', error);
        throw error;
      }
    },

    /**
     * Scale velocities by a factor
     */
    scaleVelocities: async (trackId: number, factor: number): Promise<void> => {
      try {
        await invoke(Commands.SCALE_VELOCITIES, { track_id: trackId, factor });
      } catch (error) {
        console.error('Failed to scale velocities:', error);
        throw error;
      }
    },

    /**
     * Humanize velocities with randomization
     */
    humanizeVelocities: async (trackId: number, amount: number): Promise<void> => {
      try {
        await invoke(Commands.HUMANIZE_VELOCITIES, { track_id: trackId, amount });
      } catch (error) {
        console.error('Failed to humanize velocities:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // TRACKS COMMANDS (Track velocity statistics)
  // Provides track list with note statistics for velocity editing
  // ============================================================================

  tracks: {
    /**
     * Get all tracks with note statistics for velocity editor
     * Returns tracks with noteCount and avgVelocity for velocity editing
     */
    getTracks: async (): Promise<VelocityTrack[]> => {
      try {
        // Get base tracks from sequencer
        const baseTracks = await invoke<Track[]>(Commands.GET_TRACKS);

        // Transform to VelocityTrack format with note statistics
        const velocityTracks: VelocityTrack[] = await Promise.all(
          baseTracks.map(async (track) => {
            try {
              // Try to get notes for this track to calculate statistics
              const notes = await invoke<Note[]>(Commands.GET_TRACK_NOTES, { track_id: track.id });
              const noteCount = notes.length;
              const avgVelocity =
                noteCount > 0
                  ? Math.round(notes.reduce((sum, n) => sum + n.velocity, 0) / noteCount)
                  : 0;

              return {
                id: track.id,
                name: `Track ${track.channel + 1}`,
                noteCount,
                avgVelocity,
              };
            } catch {
              // If notes can't be fetched, return with zero stats
              return {
                id: track.id,
                name: `Track ${track.channel + 1}`,
                noteCount: 0,
                avgVelocity: 0,
              };
            }
          })
        );

        return velocityTracks;
      } catch (error) {
        console.error('Failed to get tracks:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // FAVORITES COMMANDS (Convenience wrapper for FavoritesWindow)
  // Wraps analysis.* favorites methods for consistent api.favorites.* calls
  // ============================================================================

  favorites: {
    /**
     * Get all favorite files
     */
    getFavorites: async (): Promise<FileDetails[]> => {
      try {
        return await invoke(Commands.GET_FAVORITES);
      } catch (error) {
        console.error('Failed to get favorites:', error);
        throw error;
      }
    },

    /**
     * Add file to favorites
     */
    addFavorite: async (fileId: number): Promise<void> => {
      try {
        await invoke(Commands.ADD_FAVORITE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to add favorite:', error);
        throw error;
      }
    },

    /**
     * Remove file from favorites
     */
    removeFavorite: async (fileId: number): Promise<void> => {
      try {
        await invoke(Commands.REMOVE_FAVORITE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to remove favorite:', error);
        throw error;
      }
    },

    /**
     * Check if file is favorited
     */
    isFavorite: async (fileId: number): Promise<boolean> => {
      try {
        return await invoke(Commands.IS_FAVORITE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to check favorite:', error);
        throw error;
      }
    },

    // Aliases for component compatibility
    add: async (fileId: number): Promise<void> => {
      try {
        await invoke(Commands.ADD_FAVORITE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to add favorite:', error);
        throw error;
      }
    },

    remove: async (fileId: number): Promise<void> => {
      try {
        await invoke(Commands.REMOVE_FAVORITE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to remove favorite:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // TRANSPORT COMMANDS (Convenience wrapper for playback controls)
  // ============================================================================

  transport: {
    /**
     * Start playback
     */
    play: async (): Promise<void> => {
      try {
        await invoke(Commands.PLAY_TRANSPORT);
      } catch (error) {
        console.error('Failed to play:', error);
        throw error;
      }
    },

    /**
     * Stop playback
     */
    stop: async (): Promise<void> => {
      try {
        await invoke(Commands.STOP_TRANSPORT);
      } catch (error) {
        console.error('Failed to stop:', error);
        throw error;
      }
    },

    /**
     * Pause playback
     */
    pause: async (): Promise<void> => {
      try {
        await invoke(Commands.PAUSE_TRANSPORT);
      } catch (error) {
        console.error('Failed to pause:', error);
        throw error;
      }
    },

    /**
     * Get playback state
     */
    getState: async (): Promise<PlaybackState> => {
      try {
        return await invoke(Commands.GET_PLAYBACK_STATE);
      } catch (error) {
        console.error('Failed to get playback state:', error);
        throw error;
      }
    },

    /**
     * Toggle playback (play/pause)
     */
    togglePlayback: async (): Promise<void> => {
      try {
        await invoke(Commands.TOGGLE_PLAYBACK);
      } catch (error) {
        console.error('Failed to toggle playback:', error);
        throw error;
      }
    },

    /**
     * Toggle recording
     */
    toggleRecord: async (): Promise<void> => {
      try {
        await invoke(Commands.TOGGLE_RECORD);
      } catch (error) {
        console.error('Failed to toggle record:', error);
        throw error;
      }
    },

    /**
     * Play a specific file
     */
    playFile: async (fileId: number): Promise<void> => {
      try {
        await invoke(Commands.PLAY_FILE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to play file:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // PIANO ROLL COMMANDS (4 total)
  // Backend: daw/src-tauri/src/editors/piano_roll.rs
  // ============================================================================

  pianoRoll: {
    /**
     * Get all notes for a track
     */
    getTrackNotes: async (trackId: number): Promise<Note[]> => {
      try {
        return await invoke(Commands.GET_TRACK_NOTES, { track_id: trackId });
      } catch (error) {
        console.error('Failed to get track notes:', error);
        throw error;
      }
    },

    /**
     * Add a note to a track
     */
    addNote: async (trackId: number, noteData: Partial<Note>): Promise<number> => {
      try {
        return await invoke(Commands.ADD_NOTE, { track_id: trackId, note_data: noteData });
      } catch (error) {
        console.error('Failed to add note:', error);
        throw error;
      }
    },

    /**
     * Update multiple notes in batch
     */
    updateNotesBatch: async (updates: Array<{ id: number } & Partial<Note>>): Promise<void> => {
      try {
        await invoke(Commands.UPDATE_NOTES_BATCH, { updates });
      } catch (error) {
        console.error('Failed to update notes batch:', error);
        throw error;
      }
    },

    /**
     * Delete notes by IDs
     */
    deleteNotes: async (trackId: number, noteIds: number[]): Promise<void> => {
      try {
        await invoke(Commands.DELETE_NOTES, { track_id: trackId, note_ids: noteIds });
      } catch (error) {
        console.error('Failed to delete notes:', error);
        throw error;
      }
    },

    /**
     * Slice a note at a specific tick position
     * Returns tuple of [firstNote, secondNote]
     */
    sliceNote: async (trackId: number, noteId: number, sliceTick: number): Promise<[Note, Note]> => {
      try {
        return await invoke(Commands.SLICE_NOTE, {
          track_id: trackId,
          note_id: noteId,
          slice_tick: sliceTick,
        });
      } catch (error) {
        console.error('Failed to slice note:', error);
        throw error;
      }
    },

    /**
     * Stretch notes by ratio with anchor point
     */
    stretchNotes: async (
      trackId: number,
      noteIds: number[],
      ratio: number,
      anchor: 'start' | 'end' | 'center'
    ): Promise<Note[]> => {
      try {
        return await invoke(Commands.STRETCH_NOTES, {
          track_id: trackId,
          note_ids: noteIds,
          ratio,
          anchor,
        });
      } catch (error) {
        console.error('Failed to stretch notes:', error);
        throw error;
      }
    },

    /**
     * Quantize notes to a musical scale
     */
    scaleQuantizeNotes: async (
      trackId: number,
      rootNote: number,
      scaleType: string
    ): Promise<Note[]> => {
      try {
        return await invoke(Commands.SCALE_QUANTIZE_NOTES, {
          track_id: trackId,
          root_note: rootNote,
          scale_type: scaleType,
        });
      } catch (error) {
        console.error('Failed to scale quantize notes:', error);
        throw error;
      }
    },

    /**
     * Copy selected notes to clipboard
     */
    copyNotes: async (trackId: number): Promise<number> => {
      try {
        return await invoke(Commands.COPY_NOTES, { track_id: trackId });
      } catch (error) {
        console.error('Failed to copy notes:', error);
        throw error;
      }
    },

    /**
     * Paste notes from clipboard
     */
    pasteNotes: async (trackId: number, pasteTick: number, pastePitch?: number): Promise<Note[]> => {
      try {
        return await invoke(Commands.PASTE_NOTES, {
          track_id: trackId,
          paste_tick: pasteTick,
          paste_pitch: pastePitch,
        });
      } catch (error) {
        console.error('Failed to paste notes:', error);
        throw error;
      }
    },

    /**
     * Cut selected notes
     */
    cutNotes: async (trackId: number): Promise<number> => {
      try {
        return await invoke(Commands.CUT_NOTES, { track_id: trackId });
      } catch (error) {
        console.error('Failed to cut notes:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // LOOPS COMMANDS (Loop browser functionality)
  // Backend: pipeline/src-tauri/src/commands/loops.rs (planned)
  // ============================================================================

  loops: {
    /**
     * Get loops with filters
     */
    getLoops: async (filters: {
      category?: string;
      bpmMin?: number;
      bpmMax?: number;
      key?: string;
      search?: string;
    }): Promise<Loop[]> => {
      try {
        // Use search_files as backend for loop browsing
        return await invoke(Commands.SEARCH_FILES, { filters });
      } catch (error) {
        console.error('Failed to get loops:', error);
        throw error;
      }
    },

    /**
     * Play a loop preview
     */
    play: async (_loopId: number): Promise<void> => {
      try {
        await invoke(Commands.PLAY_TRANSPORT);
      } catch (error) {
        console.error('Failed to play loop:', error);
        throw error;
      }
    },

    /**
     * Stop loop playback
     */
    stopPlayback: async (): Promise<void> => {
      try {
        await invoke(Commands.STOP_TRANSPORT);
      } catch (error) {
        console.error('Failed to stop playback:', error);
        throw error;
      }
    },

    /**
     * Get loop preview waveform data
     */
    getPreview: async (loopId: number): Promise<LoopPreview> => {
      try {
        return await invoke(Commands.GET_FILE_DETAILS, { file_id: loopId });
      } catch (error) {
        console.error('Failed to get loop preview:', error);
        throw error;
      }
    },

    /**
     * Add loop to current project
     */
    addToProject: async (loopId: number): Promise<void> => {
      try {
        await invoke(Commands.LOAD_MULTIPLE_TRACKS, { file_ids: [loopId] });
      } catch (error) {
        console.error('Failed to add loop to project:', error);
        throw error;
      }
    },

    /**
     * Set tempo for loop
     */
    setTempo: async (loopId: number, bpm: number): Promise<void> => {
      try {
        await invoke(Commands.SET_TEMPO, { bpm });
      } catch (error) {
        console.error('Failed to set loop tempo:', error);
        throw error;
      }
    },

    /**
     * Analyze a loop
     */
    analyze: async (loopId: number): Promise<LoopAnalysis> => {
      try {
        return await invoke(Commands.PIPELINE_ANALYZE_FILES, { file_ids: [loopId] });
      } catch (error) {
        console.error('Failed to analyze loop:', error);
        throw error;
      }
    },

    /**
     * Rate a loop (1-5 stars, or 0/null to clear)
     */
    rate: async (loopId: number, rating: number): Promise<void> => {
      try {
        // Use the backend rating system
        const ratingValue = rating > 0 && rating <= 5 ? rating : null;
        await invoke(Commands.SET_FILE_RATING, { file_id: loopId, rating: ratingValue });
      } catch (error) {
        console.error('Failed to rate loop:', error);
        throw error;
      }
    },

    /**
     * Get the rating for a loop
     */
    getRating: async (loopId: number): Promise<number | null> => {
      try {
        return await invoke<number | null>(Commands.GET_FILE_RATING, { file_id: loopId });
      } catch (error) {
        console.error('Failed to get loop rating:', error);
        throw error;
      }
    },

    /**
     * Get all loops with a specific rating
     */
    getByRating: async (rating: number, limit?: number, offset?: number): Promise<number[]> => {
      try {
        return await invoke<number[]>(Commands.GET_FILES_BY_RATING, {
          rating,
          limit,
          offset,
        });
      } catch (error) {
        console.error('Failed to get loops by rating:', error);
        throw error;
      }
    },

    /**
     * Scan directory for new loops
     */
    scanDirectory: async (dirPath?: string): Promise<ImportStats> => {
      try {
        return await invoke(Commands.IMPORT_DIRECTORY, { dir_path: dirPath || '' });
      } catch (error) {
        console.error('Failed to scan directory:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // EDIT COMMANDS (Undo/Redo functionality)
  // Backend: daw/src-tauri/src/undo_redo/
  // ============================================================================

  edit: {
    /**
     * Undo last action
     */
    undo: async (): Promise<void> => {
      try {
        // Placeholder - undo system uses internal state management
        console.warn('Undo command - using internal state management');
      } catch (error) {
        console.error('Failed to undo:', error);
        throw error;
      }
    },

    /**
     * Redo last undone action
     */
    redo: async (): Promise<void> => {
      try {
        // Placeholder - redo system uses internal state management
        console.warn('Redo command - using internal state management');
      } catch (error) {
        console.error('Failed to redo:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // SETTINGS COMMANDS (Application settings)
  // Backend: daw/src-tauri/src/settings/
  // ============================================================================

  settings: {
    /**
     * Get all settings
     */
    get: async (): Promise<Settings> => {
      try {
        return await invoke(Commands.GET_SETTINGS);
      } catch (error) {
        console.error('Failed to get settings:', error);
        throw error;
      }
    },

    /**
     * Save settings
     */
    save: async (settings: Settings): Promise<void> => {
      try {
        await invoke(Commands.SAVE_SETTINGS, { settings });
      } catch (error) {
        console.error('Failed to save settings:', error);
        throw error;
      }
    },

    /**
     * Get audio settings
     */
    getAudioSettings: async (): Promise<AudioSettings> => {
      try {
        const settings = await invoke<Settings>(Commands.GET_SETTINGS);
        return settings.audio;
      } catch (error) {
        console.error('Failed to get audio settings:', error);
        throw error;
      }
    },

    /**
     * Set audio settings
     */
    setAudioSettings: async (audioSettings: AudioSettings): Promise<void> => {
      try {
        const settings = await invoke<Settings>(Commands.GET_SETTINGS);
        settings.audio = audioSettings;
        await invoke(Commands.SAVE_SETTINGS, { settings });
      } catch (error) {
        console.error('Failed to save audio settings:', error);
        throw error;
      }
    },

    /**
     * Get MIDI settings
     */
    getMidiSettings: async (): Promise<MIDISettings> => {
      try {
        const settings = await invoke<Settings>(Commands.GET_SETTINGS);
        return settings.midi;
      } catch (error) {
        console.error('Failed to get MIDI settings:', error);
        throw error;
      }
    },

    /**
     * Set MIDI settings
     */
    setMidiSettings: async (midiSettings: MIDISettings): Promise<void> => {
      try {
        const settings = await invoke<Settings>(Commands.GET_SETTINGS);
        settings.midi = midiSettings;
        await invoke(Commands.SAVE_SETTINGS, { settings });
      } catch (error) {
        console.error('Failed to save MIDI settings:', error);
        throw error;
      }
    },

    /**
     * Reset all settings to defaults
     */
    resetSettings: async (): Promise<void> => {
      try {
        await invoke(Commands.RESET_SETTINGS);
      } catch (error) {
        console.error('Failed to reset settings:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // FILES COMMANDS (File details, waveform, deletion)
  // Backend: pipeline/src-tauri/src/commands/file_import.rs
  // ============================================================================

  files: {
    /**
     * Get file details by ID
     */
    getFileDetails: async (fileId: number): Promise<FileDetails> => {
      try {
        return await invoke(Commands.GET_FILE_DETAILS, { file_id: fileId });
      } catch (error) {
        console.error('Failed to get file details:', error);
        throw error;
      }
    },

    /**
     * Find compatible files for a given file
     */
    findCompatibleFiles: async (fileId: number, maxResults?: number): Promise<CompatibleFile[]> => {
      try {
        return await invoke(Commands.FIND_COMPATIBLE_FILES, {
          file_id: fileId,
          max_results: maxResults ?? 10,
        });
      } catch (error) {
        console.error('Failed to find compatible files:', error);
        throw error;
      }
    },

    /**
     * Get waveform data for visualization
     */
    getWaveformData: async (fileId: number): Promise<number[]> => {
      try {
        return await invoke(Commands.GET_WAVEFORM_DATA, { file_id: fileId });
      } catch (error) {
        console.error('Failed to get waveform data:', error);
        throw error;
      }
    },

    /**
     * Delete a file
     */
    deleteFile: async (fileId: number): Promise<void> => {
      try {
        await invoke(Commands.DELETE_FILE, { file_id: fileId });
      } catch (error) {
        console.error('Failed to delete file:', error);
        throw error;
      }
    },

    /**
     * Analyze a file
     */
    analyzeFile: async (fileId: number): Promise<AnalysisResults> => {
      try {
        const results = await invoke<AnalysisResults[]>(Commands.PIPELINE_ANALYZE_FILES, {
          file_ids: [fileId],
        });
        return results[0];
      } catch (error) {
        console.error('Failed to analyze file:', error);
        throw error;
      }
    },
  },

  // ============================================================================
  // COMPATIBILITY COMMANDS (Find similar/compatible files)
  // Backend: daw/src-tauri/src/commands/compatibility.rs
  // ============================================================================

  compatibility: {
    /**
     * Find compatible files for mixing/matching
     */
    findCompatibleFiles: async (fileId: number, maxResults?: number): Promise<CompatibleFile[]> => {
      try {
        return await invoke(Commands.FIND_COMPATIBLE_FILES, {
          file_id: fileId,
          max_results: maxResults ?? 10,
        });
      } catch (error) {
        console.error('Failed to find compatible files:', error);
        throw error;
      }
    },
  },
};
