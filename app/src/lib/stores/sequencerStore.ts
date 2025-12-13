import { derived, get, writable } from 'svelte/store';
import { isTauri, safeInvoke, safeListen } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';

// ============================================================================
// TYPES
// ============================================================================

export type SyncSource = 'internal' | 'external' | 'midi_clock';
export type SnapValue = '1/1' | '1/2' | '1/4' | '1/8' | '1/16' | '1/32' | 'off';
export type QuantizeStrength = 25 | 50 | 75 | 100;

export interface SequencerProject {
  id: number | null;
  name: string;
  description: string;
  bpm: number;
  timeSignature: [number, number];
  lengthBars: number;
  loopEnabled: boolean;
  loopStartBar: number;
  loopEndBar: number;
  syncSource: SyncSource;
  sendMidiClock: boolean;
  filePath: string | null;
  isDirty: boolean;
  createdAt: string | null;
  modifiedAt: string | null;
}

export interface SequencerTrack {
  id: number;
  projectId: number;
  name: string;
  color: string;
  icon: string;
  sortOrder: number;
  muted: boolean;
  solo: boolean;
  armed: boolean;
  collapsed: boolean;
  height: number;
  midiChannel: number;
  midiPortId: number | null;
  transpose: number;
  velocityScale: number;
  velocityOffset: number;
  clips: SequencerClip[];
}

export interface SequencerClip {
  id: number;
  trackId: number;
  midiFileId: number | null;
  name: string;
  color: string;
  startTick: number;
  lengthTicks: number;
  offsetTicks: number;
  loopCount: number;
  transpose: number;
  velocityScale: number;
  selected: boolean;
}

export interface PlayheadPosition {
  ticks: number;
  bar: number;
  beat: number;
  subBeat: number;
}

export interface TimelineMarker {
  id: number;
  name: string;
  tick: number;
  color: string;
}

export interface DragState {
  isDragging: boolean;
  dragType: 'clip' | 'clip-resize-start' | 'clip-resize-end' | 'playhead' | 'loop-start' | 'loop-end' | null;
  clipId: number | null;
  startX: number;
  startTick: number;
  originalStartTick: number;
  originalLengthTicks: number;
}

// Backend API types (matching daw/src-tauri/src/commands/project.rs)
export interface DawProject {
  id: number;
  name: string;
  description: string | null;
  bpm: number;
  time_signature_numerator: number;
  time_signature_denominator: number;
  key_signature: string;
  sample_rate: number;
  bit_depth: number;
  file_path: string | null;
  file_size_bytes: number | null;
  version: number;
  created_at: string;
  updated_at: string;
  last_opened_at: string | null;
}

export interface CreateProjectParams {
  name: string;
  description: string | null;
  bpm: number | null;
  time_signature_numerator: number | null;
  time_signature_denominator: number | null;
  key_signature: string | null;
  sample_rate: number | null;
  bit_depth: number | null;
}

export interface UpdateProjectParams {
  name: string | null;
  description: string | null;
  bpm: number | null;
  time_signature_numerator: number | null;
  time_signature_denominator: number | null;
  key_signature: string | null;
  sample_rate: number | null;
  bit_depth: number | null;
  file_path: string | null;
  file_size_bytes: number | null;
}

// Backend track/clip types (matching Rust struct field names)
export interface BackendClip {
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
}

export interface BackendTrack {
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
  clips: BackendClip[];
}

// ============================================================================
// INITIAL STATE
// ============================================================================

const createEmptyProject = (): SequencerProject => ({
  id: null,
  name: 'Untitled',
  description: '',
  bpm: 120.0,
  timeSignature: [4, 4],
  lengthBars: 16,
  loopEnabled: false,
  loopStartBar: 1,
  loopEndBar: 8,
  syncSource: 'internal',
  sendMidiClock: true,
  filePath: null,
  isDirty: false,
  createdAt: null,
  modifiedAt: null,
});

export interface SequencerState {
  // Project state
  project: SequencerProject;
  tracks: SequencerTrack[];
  markers: TimelineMarker[];

  // Playback state
  isPlaying: boolean;
  isRecording: boolean;
  playhead: PlayheadPosition;

  // UI state
  zoom: number; // pixels per beat
  scrollX: number; // horizontal scroll position in ticks
  scrollY: number; // vertical scroll position in pixels
  snapValue: SnapValue;
  quantizeStrength: QuantizeStrength;
  selectedTrackIds: number[];
  selectedClipIds: number[];
  dragState: DragState;

  // Timeline constants
  ticksPerBeat: number;
}

const initialState: SequencerState = {
  project: createEmptyProject(),
  tracks: [],
  markers: [],
  isPlaying: false,
  isRecording: false,
  playhead: { ticks: 0, bar: 1, beat: 1, subBeat: 0 },
  zoom: 40, // 40 pixels per beat
  scrollX: 0,
  scrollY: 0,
  snapValue: '1/16',
  quantizeStrength: 100,
  selectedTrackIds: [],
  selectedClipIds: [],
  dragState: {
    isDragging: false,
    dragType: null,
    clipId: null,
    startX: 0,
    startTick: 0,
    originalStartTick: 0,
    originalLengthTicks: 0,
  },
  ticksPerBeat: 480,
};

// ============================================================================
// STORE
// ============================================================================

export const sequencerStore = writable<SequencerState>(initialState);

// Event listeners cleanup
let unlisteners: (() => void)[] = [];

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/**
 * Convert ticks to bar/beat/subbeat position
 */
export function ticksToPosition(
  ticks: number,
  ticksPerBeat: number,
  beatsPerBar: number
): PlayheadPosition {
  const totalBeats = ticks / ticksPerBeat;
  const bar = Math.floor(totalBeats / beatsPerBar) + 1;
  const beat = Math.floor(totalBeats % beatsPerBar) + 1;
  const subBeat = Math.round((totalBeats % 1) * 1000);
  return { ticks, bar, beat, subBeat };
}

/**
 * Convert bar/beat position to ticks
 */
export function positionToTicks(
  bar: number,
  beat: number,
  ticksPerBeat: number,
  beatsPerBar: number
): number {
  return ((bar - 1) * beatsPerBar + (beat - 1)) * ticksPerBeat;
}

/**
 * Get snap value in ticks
 */
export function getSnapTicks(snapValue: SnapValue, ticksPerBeat: number): number {
  switch (snapValue) {
    case '1/1':
      return ticksPerBeat * 4;
    case '1/2':
      return ticksPerBeat * 2;
    case '1/4':
      return ticksPerBeat;
    case '1/8':
      return ticksPerBeat / 2;
    case '1/16':
      return ticksPerBeat / 4;
    case '1/32':
      return ticksPerBeat / 8;
    case 'off':
      return 1;
    default:
      return ticksPerBeat / 4;
  }
}

/**
 * Snap a tick value to the grid
 */
export function snapToGrid(ticks: number, snapValue: SnapValue, ticksPerBeat: number): number {
  if (snapValue === 'off') {return ticks;}
  const snapTicks = getSnapTicks(snapValue, ticksPerBeat);
  return Math.round(ticks / snapTicks) * snapTicks;
}

/**
 * Generate a random track color
 */
function generateTrackColor(): string {
  const colors = [
    '#EF4444', '#F97316', '#F59E0B', '#EAB308', '#84CC16',
    '#22C55E', '#10B981', '#14B8A6', '#06B6D4', '#0EA5E9',
    '#3B82F6', '#6366F1', '#8B5CF6', '#A855F7', '#D946EF',
    '#EC4899', '#F43F5E',
  ];
  return colors[Math.floor(Math.random() * colors.length)];
}

/**
 * Get track icon based on name
 */
function getTrackIcon(name: string): string {
  const lowerName = name.toLowerCase();
  if (lowerName.includes('drum') || lowerName.includes('percussion')) {return '🥁';}
  if (lowerName.includes('bass')) {return '🎸';}
  if (lowerName.includes('lead') || lowerName.includes('synth')) {return '🎹';}
  if (lowerName.includes('pad')) {return '🎛️';}
  if (lowerName.includes('vocal') || lowerName.includes('voice')) {return '🎤';}
  if (lowerName.includes('guitar')) {return '🎸';}
  if (lowerName.includes('string')) {return '🎻';}
  if (lowerName.includes('brass') || lowerName.includes('horn')) {return '🎺';}
  return '🎵';
}

// ============================================================================
// ACTIONS
// ============================================================================

export const sequencerActions = {
  // --------------------------------------------------------------------------
  // Initialization
  // --------------------------------------------------------------------------

  async initListeners() {
    if (!isTauri()) {
      console.warn('[SequencerStore] Not in Tauri context - listeners disabled');
      return;
    }

    // Listen for playhead position updates
    const unlistenPosition = await safeListen<number>('sequencer::position-updated', (ticks) => {
      const state = get(sequencerStore);
      const position = ticksToPosition(
        ticks,
        state.ticksPerBeat,
        state.project.timeSignature[0]
      );
      sequencerStore.update((s) => ({ ...s, playhead: position }));
    });
    if (unlistenPosition) {unlisteners.push(unlistenPosition);}

    // Listen for playback started
    const unlistenPlay = await safeListen<void>('sequencer::playback-started', () => {
      sequencerStore.update((s) => ({ ...s, isPlaying: true }));
    });
    if (unlistenPlay) {unlisteners.push(unlistenPlay);}

    // Listen for playback stopped
    const unlistenStop = await safeListen<void>('sequencer::playback-stopped', () => {
      sequencerStore.update((s) => ({ ...s, isPlaying: false, isRecording: false }));
    });
    if (unlistenStop) {unlisteners.push(unlistenStop);}

    // Listen for recording started
    const unlistenRecord = await safeListen<void>('sequencer::recording-started', () => {
      sequencerStore.update((s) => ({ ...s, isRecording: true }));
    });
    if (unlistenRecord) {unlisteners.push(unlistenRecord);}
  },

  cleanupListeners() {
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners = [];
  },

  // --------------------------------------------------------------------------
  // Project Actions
  // --------------------------------------------------------------------------

  newProject() {
    sequencerStore.update((state) => ({
      ...state,
      project: createEmptyProject(),
      tracks: [],
      markers: [],
      playhead: { ticks: 0, bar: 1, beat: 1, subBeat: 0 },
      selectedTrackIds: [],
      selectedClipIds: [],
    }));
  },

  async loadProjectFromFile(filePath: string) {
    try {
      // Load project from file path via backend
      const dawProject = await safeInvoke<DawProject | null>(Commands.PROJECT_LOAD_FROM_FILE, { file_path: filePath });

      if (!dawProject) {
        throw new Error(`Could not load project from file: ${filePath}`);
      }

      // Continue with standard project loading using the returned project ID
      await this.loadProject(dawProject.id);
    } catch (error) {
      console.error('Failed to load project from file:', error);
      throw error;
    }
  },

  async loadProject(projectId: number) {
    try {
      // Load project from backend database
      const dawProject = await safeInvoke<DawProject | null>(Commands.PROJECT_LOAD, { id: projectId });

      if (!dawProject) {
        throw new Error(`Project with ID ${projectId} not found`);
      }

      // Map DawProject to SequencerProject format
      const project: SequencerProject = {
        id: dawProject.id,
        name: dawProject.name,
        description: dawProject.description || '',
        bpm: dawProject.bpm,
        timeSignature: [dawProject.time_signature_numerator, dawProject.time_signature_denominator],
        lengthBars: 32, // Default, not stored in DawProject
        loopEnabled: false,
        loopStartBar: 1,
        loopEndBar: 9,
        syncSource: 'internal',
        sendMidiClock: false,
        filePath: dawProject.file_path || null,
        isDirty: false,
        createdAt: dawProject.created_at,
        modifiedAt: dawProject.updated_at,
      };

      // Load tracks and clips from database
      let loadedTracks: SequencerTrack[] = [];
      try {
        const trackData = await safeInvoke<{ tracks: BackendTrack[] }>(
          Commands.PROJECT_LOAD_TRACKS,
          { project_id: projectId }
        );

        if (trackData?.tracks) {
          loadedTracks = trackData.tracks.map((t, index): SequencerTrack => ({
            id: t.id,
            projectId: t.projectId,
            name: t.name,
            color: t.color,
            icon: '🎹', // Default icon
            sortOrder: t.trackNumber ?? index,
            muted: t.isMuted,
            solo: t.isSoloed,
            armed: t.isArmed,
            collapsed: false,
            height: 80,
            midiChannel: t.midiChannel,
            midiPortId: null,
            transpose: 0,
            velocityScale: 1.0,
            velocityOffset: 0,
            clips: (t.clips || []).map((c): SequencerClip => ({
              id: c.id,
              trackId: c.trackId,
              midiFileId: c.sourceFileId,
              name: c.name,
              color: c.color,
              startTick: c.startTick,
              lengthTicks: c.durationTicks,
              offsetTicks: c.sourceStartTick,
              loopCount: c.loopEnabled ? -1 : 1,
              transpose: 0,
              velocityScale: 1.0,
              selected: c.isSelected,
            })),
          }));
        }
      } catch (trackError) {
        console.warn('Could not load tracks for project (may not exist yet):', trackError);
      }

      sequencerStore.update((s) => ({
        ...s,
        project,
        tracks: loadedTracks,
        selectedTrackIds: [],
        selectedClipIds: [],
      }));

      console.log('Loaded project:', project.name, 'with', loadedTracks.length, 'tracks');
    } catch (error) {
      console.error('Failed to load project:', error);
      throw error;
    }
  },

  async saveProject() {
    try {
      const state = get(sequencerStore);
      const { project, tracks } = state;
      let projectId = project.id;

      if (projectId === null) {
        // Create new project if no ID exists
        const params: CreateProjectParams = {
          name: project.name,
          description: project.description || null,
          bpm: project.bpm,
          time_signature_numerator: project.timeSignature[0],
          time_signature_denominator: project.timeSignature[1],
          key_signature: null,
          sample_rate: null,
          bit_depth: null,
        };

        const dawProject = await safeInvoke<DawProject>(Commands.PROJECT_CREATE, { params });

        if (!dawProject) {
          throw new Error('Failed to create project: no response from backend');
        }

        projectId = dawProject.id;
        sequencerStore.update((s) => ({
          ...s,
          project: {
            ...s.project,
            id: dawProject.id,
            isDirty: false,
            createdAt: dawProject.created_at,
            modifiedAt: dawProject.updated_at,
          },
        }));
        console.log('Created new project:', dawProject.name);
      } else {
        // Update existing project
        const params: UpdateProjectParams = {
          name: project.name,
          description: project.description || null,
          bpm: project.bpm,
          time_signature_numerator: project.timeSignature[0],
          time_signature_denominator: project.timeSignature[1],
          key_signature: null,
          sample_rate: null,
          bit_depth: null,
          file_path: project.filePath,
          file_size_bytes: null,
        };

        const dawProject = await safeInvoke<DawProject>(Commands.PROJECT_UPDATE, {
          id: projectId,
          params,
        });

        if (!dawProject) {
          throw new Error('Failed to update project: no response from backend');
        }

        sequencerStore.update((s) => ({
          ...s,
          project: { ...s.project, isDirty: false, modifiedAt: dawProject.updated_at },
        }));
        console.log('Saved project:', dawProject.name);
      }

      // Save tracks and clips to database
      if (tracks.length > 0 && projectId !== null) {
        const backendTracks = tracks.map((t, index) => ({
          id: t.id > 0 ? t.id : undefined, // Only include ID if it exists in DB
          name: t.name,
          color: t.color,
          midiChannel: t.midiChannel,
          trackNumber: t.sortOrder ?? index,
          volume: 1.0, // Default volume
          pan: 0.0, // Center pan
          isMuted: t.muted,
          isSoloed: t.solo,
          isArmed: t.armed,
          clips: t.clips.map((c) => ({
            id: c.id > 0 ? c.id : undefined, // Only include ID if it exists in DB
            name: c.name,
            color: c.color,
            startTick: c.startTick,
            durationTicks: c.lengthTicks,
            sourceFileId: c.midiFileId,
            isMuted: false,
            isSelected: c.selected,
            isLocked: false,
            sourceStartTick: c.offsetTicks,
            sourceEndTick: null,
            gainDb: 0.0,
            loopEnabled: c.loopCount === -1,
            loopStartTick: 0,
            loopEndTick: null,
          })),
        }));

        await safeInvoke(Commands.PROJECT_SAVE_TRACKS, {
          project_id: projectId,
          tracks: backendTracks,
        });
        console.log('Saved', tracks.length, 'tracks with clips');
      }
    } catch (error) {
      console.error('Failed to save project:', error);
      throw error;
    }
  },

  setProjectName(name: string) {
    sequencerStore.update((state) => ({
      ...state,
      project: { ...state.project, name, isDirty: true },
    }));
  },

  setBpm(bpm: number) {
    const clampedBpm = Math.max(20, Math.min(300, bpm));
    sequencerStore.update((state) => ({
      ...state,
      project: { ...state.project, bpm: clampedBpm, isDirty: true },
    }));
  },

  setTimeSignature(numerator: number, denominator: number) {
    sequencerStore.update((state) => ({
      ...state,
      project: { ...state.project, timeSignature: [numerator, denominator], isDirty: true },
    }));
  },

  setLoopEnabled(enabled: boolean) {
    sequencerStore.update((state) => ({
      ...state,
      project: { ...state.project, loopEnabled: enabled, isDirty: true },
    }));
  },

  setLoopRange(startBar: number, endBar: number) {
    sequencerStore.update((state) => ({
      ...state,
      project: {
        ...state.project,
        loopStartBar: startBar,
        loopEndBar: endBar,
        isDirty: true,
      },
    }));
  },

  setSyncSource(source: SyncSource) {
    sequencerStore.update((state) => ({
      ...state,
      project: { ...state.project, syncSource: source, isDirty: true },
    }));
  },

  // --------------------------------------------------------------------------
  // Transport Actions
  // --------------------------------------------------------------------------

  async play() {
    try {
      await safeInvoke(Commands.START_SEQUENCER);
      sequencerStore.update((s) => ({ ...s, isPlaying: true }));
    } catch (error) {
      console.error('Failed to play:', error);
      throw error;
    }
  },

  async pause() {
    try {
      await safeInvoke(Commands.PAUSE_SEQUENCER);
      sequencerStore.update((s) => ({ ...s, isPlaying: false }));
    } catch (error) {
      console.error('Failed to pause:', error);
      throw error;
    }
  },

  async stop() {
    try {
      await safeInvoke(Commands.STOP_SEQUENCER);
      sequencerStore.update((s) => ({
        ...s,
        isPlaying: false,
        isRecording: false,
        playhead: { ticks: 0, bar: 1, beat: 1, subBeat: 0 },
      }));
    } catch (error) {
      console.error('Failed to stop:', error);
      throw error;
    }
  },

  async record() {
    try {
      await safeInvoke(Commands.START_RECORDING);
      sequencerStore.update((s) => ({ ...s, isPlaying: true, isRecording: true }));
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw error;
    }
  },

  setPlayheadPosition(ticks: number) {
    const state = get(sequencerStore);
    const position = ticksToPosition(
      ticks,
      state.ticksPerBeat,
      state.project.timeSignature[0]
    );
    sequencerStore.update((s) => ({ ...s, playhead: position }));
  },

  gotoStart() {
    this.setPlayheadPosition(0);
  },

  gotoEnd() {
    const state = get(sequencerStore);
    const totalTicks = state.project.lengthBars * state.project.timeSignature[0] * state.ticksPerBeat;
    this.setPlayheadPosition(totalTicks);
  },

  rewind() {
    const state = get(sequencerStore);
    const barTicks = state.project.timeSignature[0] * state.ticksPerBeat;
    const newTicks = Math.max(0, state.playhead.ticks - barTicks);
    this.setPlayheadPosition(newTicks);
  },

  forward() {
    const state = get(sequencerStore);
    const barTicks = state.project.timeSignature[0] * state.ticksPerBeat;
    const maxTicks = state.project.lengthBars * barTicks;
    const newTicks = Math.min(maxTicks, state.playhead.ticks + barTicks);
    this.setPlayheadPosition(newTicks);
  },

  // --------------------------------------------------------------------------
  // Track Actions
  // --------------------------------------------------------------------------

  addTrack(name: string = `Track ${get(sequencerStore).tracks.length + 1}`) {
    const state = get(sequencerStore);
    const newTrack: SequencerTrack = {
      id: Date.now(),
      projectId: state.project.id || 0,
      name,
      color: generateTrackColor(),
      icon: getTrackIcon(name),
      sortOrder: state.tracks.length,
      muted: false,
      solo: false,
      armed: false,
      collapsed: false,
      height: 80,
      midiChannel: Math.min(state.tracks.length + 1, 16),
      midiPortId: null,
      transpose: 0,
      velocityScale: 100,
      velocityOffset: 0,
      clips: [],
    };

    sequencerStore.update((s) => ({
      ...s,
      tracks: [...s.tracks, newTrack],
      project: { ...s.project, isDirty: true },
    }));

    return newTrack.id;
  },

  removeTrack(trackId: number) {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.filter((t) => t.id !== trackId),
      selectedTrackIds: state.selectedTrackIds.filter((id) => id !== trackId),
      project: { ...state.project, isDirty: true },
    }));
  },

  updateTrack(trackId: number, updates: Partial<SequencerTrack>) {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, ...updates } : t
      ),
      project: { ...state.project, isDirty: true },
    }));
  },

  toggleTrackMute(trackId: number) {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, muted: !t.muted } : t
      ),
    }));
  },

  toggleTrackSolo(trackId: number) {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, solo: !t.solo } : t
      ),
    }));
  },

  toggleTrackArmed(trackId: number) {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, armed: !t.armed } : t
      ),
    }));
  },

  toggleTrackCollapsed(trackId: number) {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((t) =>
        t.id === trackId ? { ...t, collapsed: !t.collapsed } : t
      ),
    }));
  },

  setTrackHeight(trackId: number, height: number) {
    const clampedHeight = Math.max(40, Math.min(200, height));
    this.updateTrack(trackId, { height: clampedHeight });
  },

  reorderTracks(fromIndex: number, toIndex: number) {
    sequencerStore.update((state) => {
      const tracks = [...state.tracks];
      const [removed] = tracks.splice(fromIndex, 1);
      tracks.splice(toIndex, 0, removed);
      return {
        ...state,
        tracks: tracks.map((t, i) => ({ ...t, sortOrder: i })),
        project: { ...state.project, isDirty: true },
      };
    });
  },

  selectTrack(trackId: number, addToSelection: boolean = false) {
    sequencerStore.update((state) => ({
      ...state,
      selectedTrackIds: addToSelection
        ? [...state.selectedTrackIds, trackId]
        : [trackId],
    }));
  },

  deselectAllTracks() {
    sequencerStore.update((state) => ({ ...state, selectedTrackIds: [] }));
  },

  // --------------------------------------------------------------------------
  // Clip Actions
  // --------------------------------------------------------------------------

  addClip(
    trackId: number,
    startTick: number,
    lengthTicks: number,
    midiFileId: number | null = null,
    name: string = 'New Clip'
  ) {
    const state = get(sequencerStore);
    const track = state.tracks.find((t) => t.id === trackId);
    if (!track) {return null;}

    const snappedStart = snapToGrid(startTick, state.snapValue, state.ticksPerBeat);
    const snappedLength = snapToGrid(lengthTicks, state.snapValue, state.ticksPerBeat);

    const newClip: SequencerClip = {
      id: Date.now(),
      trackId,
      midiFileId,
      name,
      color: track.color,
      startTick: snappedStart,
      lengthTicks: snappedLength || state.ticksPerBeat * 4, // Default to 1 bar
      offsetTicks: 0,
      loopCount: 1,
      transpose: 0,
      velocityScale: 100,
      selected: false,
    };

    sequencerStore.update((s) => ({
      ...s,
      tracks: s.tracks.map((t) =>
        t.id === trackId ? { ...t, clips: [...t.clips, newClip] } : t
      ),
      project: { ...s.project, isDirty: true },
    }));

    return newClip.id;
  },

  removeClip(trackId: number, clipId: number) {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((t) =>
        t.id === trackId
          ? { ...t, clips: t.clips.filter((c) => c.id !== clipId) }
          : t
      ),
      selectedClipIds: state.selectedClipIds.filter((id) => id !== clipId),
      project: { ...state.project, isDirty: true },
    }));
  },

  updateClip(trackId: number, clipId: number, updates: Partial<SequencerClip>) {
    const state = get(sequencerStore);

    // Snap position and length if provided
    if (updates.startTick !== undefined) {
      updates.startTick = snapToGrid(updates.startTick, state.snapValue, state.ticksPerBeat);
    }
    if (updates.lengthTicks !== undefined) {
      updates.lengthTicks = Math.max(
        getSnapTicks(state.snapValue, state.ticksPerBeat),
        snapToGrid(updates.lengthTicks, state.snapValue, state.ticksPerBeat)
      );
    }

    sequencerStore.update((s) => ({
      ...s,
      tracks: s.tracks.map((t) =>
        t.id === trackId
          ? {
            ...t,
            clips: t.clips.map((c) =>
              c.id === clipId ? { ...c, ...updates } : c
            ),
          }
          : t
      ),
      project: { ...s.project, isDirty: true },
    }));
  },

  moveClip(clipId: number, newTrackId: number, newStartTick: number) {
    const state = get(sequencerStore);
    let clipToMove: SequencerClip | null = null;
    let sourceTrackId: number | null = null;

    // Find the clip
    for (const track of state.tracks) {
      const clip = track.clips.find((c) => c.id === clipId);
      if (clip) {
        clipToMove = { ...clip };
        sourceTrackId = track.id;
        break;
      }
    }

    if (!clipToMove || sourceTrackId === null) {return;}

    const snappedStart = snapToGrid(newStartTick, state.snapValue, state.ticksPerBeat);
    clipToMove.startTick = Math.max(0, snappedStart);
    clipToMove.trackId = newTrackId;

    sequencerStore.update((s) => ({
      ...s,
      tracks: s.tracks.map((t) => {
        if (t.id === sourceTrackId) {
          return { ...t, clips: t.clips.filter((c) => c.id !== clipId) };
        }
        if (t.id === newTrackId) {
          return { ...t, clips: [...t.clips, clipToMove] };
        }
        return t;
      }),
      project: { ...s.project, isDirty: true },
    }));
  },

  selectClip(clipId: number, addToSelection: boolean = false) {
    sequencerStore.update((state) => {
      const newSelectedClipIds = addToSelection
        ? [...state.selectedClipIds, clipId]
        : [clipId];

      return {
        ...state,
        selectedClipIds: newSelectedClipIds,
        tracks: state.tracks.map((t) => ({
          ...t,
          clips: t.clips.map((c) => ({
            ...c,
            selected: newSelectedClipIds.includes(c.id),
          })),
        })),
      };
    });
  },

  deselectAllClips() {
    sequencerStore.update((state) => ({
      ...state,
      selectedClipIds: [],
      tracks: state.tracks.map((t) => ({
        ...t,
        clips: t.clips.map((c) => ({ ...c, selected: false })),
      })),
    }));
  },

  deleteSelectedClips() {
    sequencerStore.update((state) => ({
      ...state,
      tracks: state.tracks.map((t) => ({
        ...t,
        clips: t.clips.filter((c) => !state.selectedClipIds.includes(c.id)),
      })),
      selectedClipIds: [],
      project: { ...state.project, isDirty: true },
    }));
  },

  duplicateSelectedClips() {
    const state = get(sequencerStore);

    sequencerStore.update((s) => ({
      ...s,
      tracks: s.tracks.map((t) => {
        const newClips = t.clips
          .filter((c) => s.selectedClipIds.includes(c.id))
          .map((c) => ({
            ...c,
            id: Date.now() + Math.random(),
            startTick: c.startTick + c.lengthTicks,
            selected: true,
          }));

        return {
          ...t,
          clips: [
            ...t.clips.map((c) => ({ ...c, selected: false })),
            ...newClips,
          ],
        };
      }),
      selectedClipIds: [], // Will be updated by new clips
      project: { ...s.project, isDirty: true },
    }));
  },

  // --------------------------------------------------------------------------
  // Drag State Actions
  // --------------------------------------------------------------------------

  startDrag(
    dragType: DragState['dragType'],
    startX: number,
    startTick: number,
    clipId: number | null = null,
    originalStartTick: number = 0,
    originalLengthTicks: number = 0
  ) {
    sequencerStore.update((state) => ({
      ...state,
      dragState: {
        isDragging: true,
        dragType,
        clipId,
        startX,
        startTick,
        originalStartTick,
        originalLengthTicks,
      },
    }));
  },

  updateDrag(currentX: number, currentTick: number) {
    const state = get(sequencerStore);
    if (!state.dragState.isDragging) {return;}

    // Drag logic handled by components
  },

  endDrag() {
    sequencerStore.update((state) => ({
      ...state,
      dragState: {
        isDragging: false,
        dragType: null,
        clipId: null,
        startX: 0,
        startTick: 0,
        originalStartTick: 0,
        originalLengthTicks: 0,
      },
    }));
  },

  // --------------------------------------------------------------------------
  // View Actions
  // --------------------------------------------------------------------------

  setZoom(zoom: number) {
    const clampedZoom = Math.max(10, Math.min(200, zoom));
    sequencerStore.update((state) => ({ ...state, zoom: clampedZoom }));
  },

  zoomIn() {
    const state = get(sequencerStore);
    this.setZoom(state.zoom * 1.25);
  },

  zoomOut() {
    const state = get(sequencerStore);
    this.setZoom(state.zoom / 1.25);
  },

  setScrollX(scrollX: number) {
    sequencerStore.update((state) => ({ ...state, scrollX: Math.max(0, scrollX) }));
  },

  setScrollY(scrollY: number) {
    sequencerStore.update((state) => ({ ...state, scrollY: Math.max(0, scrollY) }));
  },

  setSnapValue(snapValue: SnapValue) {
    sequencerStore.update((state) => ({ ...state, snapValue }));
  },

  setQuantizeStrength(strength: QuantizeStrength) {
    sequencerStore.update((state) => ({ ...state, quantizeStrength: strength }));
  },

  // --------------------------------------------------------------------------
  // Marker Actions
  // --------------------------------------------------------------------------

  addMarker(name: string, tick: number) {
    const state = get(sequencerStore);
    const snappedTick = snapToGrid(tick, state.snapValue, state.ticksPerBeat);
    const newMarker: TimelineMarker = {
      id: Date.now(),
      name,
      tick: snappedTick,
      color: '#FBBF24',
    };

    sequencerStore.update((s) => ({
      ...s,
      markers: [...s.markers, newMarker].sort((a, b) => a.tick - b.tick),
      project: { ...s.project, isDirty: true },
    }));

    return newMarker.id;
  },

  removeMarker(markerId: number) {
    sequencerStore.update((state) => ({
      ...state,
      markers: state.markers.filter((m) => m.id !== markerId),
      project: { ...state.project, isDirty: true },
    }));
  },

  updateMarker(markerId: number, updates: Partial<TimelineMarker>) {
    sequencerStore.update((state) => ({
      ...state,
      markers: state.markers.map((m) =>
        m.id === markerId ? { ...m, ...updates } : m
      ),
      project: { ...state.project, isDirty: true },
    }));
  },

  gotoMarker(markerId: number) {
    const state = get(sequencerStore);
    const marker = state.markers.find((m) => m.id === markerId);
    if (marker) {
      this.setPlayheadPosition(marker.tick);
    }
  },
};

// ============================================================================
// DERIVED STORES
// ============================================================================

export const projectName = derived(sequencerStore, ($s) => $s.project.name);

export const isProjectDirty = derived(sequencerStore, ($s) => $s.project.isDirty);

export const currentBpm = derived(sequencerStore, ($s) => $s.project.bpm);

export const currentTimeSignature = derived(sequencerStore, ($s) => $s.project.timeSignature);

export const isPlaying = derived(sequencerStore, ($s) => $s.isPlaying);

export const isRecording = derived(sequencerStore, ($s) => $s.isRecording);

export const formattedPlayhead = derived(sequencerStore, ($s) => {
  const { bar, beat, subBeat } = $s.playhead;
  return `${String(bar).padStart(3, '0')}|${beat}|${String(subBeat).padStart(3, '0')}`;
});

export const loopEnabled = derived(sequencerStore, ($s) => $s.project.loopEnabled);

export const loopRange = derived(sequencerStore, ($s) => ({
  start: $s.project.loopStartBar,
  end: $s.project.loopEndBar,
}));

export const selectedTracks = derived(sequencerStore, ($s) =>
  $s.tracks.filter((t) => $s.selectedTrackIds.includes(t.id))
);

export const selectedClips = derived(sequencerStore, ($s) => {
  const clips: SequencerClip[] = [];
  for (const track of $s.tracks) {
    for (const clip of track.clips) {
      if ($s.selectedClipIds.includes(clip.id)) {
        clips.push(clip);
      }
    }
  }
  return clips;
});

export const hasSoloedTracks = derived(
  sequencerStore,
  ($s) => $s.tracks.some((t) => t.solo)
);

export const visibleTracks = derived(sequencerStore, ($s) => {
  const hasSolo = $s.tracks.some((t) => t.solo);
  return $s.tracks.map((t) => ({
    ...t,
    effectivelyMuted: hasSolo ? !t.solo : t.muted,
  }));
});

export const totalProjectLength = derived(sequencerStore, ($s) => {
  const ticksPerBar = $s.project.timeSignature[0] * $s.ticksPerBeat;
  return $s.project.lengthBars * ticksPerBar;
});

export const pixelsPerTick = derived(
  sequencerStore,
  ($s) => $s.zoom / $s.ticksPerBeat
);

export const ticksPerPixel = derived(
  sequencerStore,
  ($s) => $s.ticksPerBeat / $s.zoom
);
