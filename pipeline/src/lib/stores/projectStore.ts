/**
 * Project Store - Project and track state management
 *
 * Grown-up Script: Manages current project state, tracks, and selections.
 * Handles track CRUD operations and syncs with backend.
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ============================================================================
// Types
// ============================================================================

export interface TrackInfo {
  id: number;
  label: string;
  color: string;
  muted: boolean;
  solo: boolean;
  armed: boolean;
  visible: boolean;
  volume: number; // 0.0 to 1.0
  pan: number; // -1.0 (left) to 1.0 (right)
  instrument: string | null;
  midiChannel: number; // 1-16
  order: number;
}

export interface ProjectState {
  projectId: string | null;
  projectName: string;
  tracks: Map<number, TrackInfo>;
  selectedTrackId: number | null;
  modified: boolean;
  lastSavedTime: number | null;
  trackIdCounter: number;
}

// ============================================================================
// Constants
// ============================================================================

const TRACK_COLORS = [
  '#FF6B6B', '#4ECDC4', '#45B7D1', '#FFA07A', '#98D8C8',
  '#F7DC6F', '#BB8FCE', '#85C1E2', '#F8B739', '#52B788'
];

const DEFAULT_STATE: ProjectState = {
  projectId: null,
  projectName: 'Untitled Project',
  tracks: new Map(),
  selectedTrackId: null,
  modified: false,
  lastSavedTime: null,
  trackIdCounter: 1
};

// ============================================================================
// Store
// ============================================================================

const { subscribe, set, update } = writable<ProjectState>(DEFAULT_STATE);

// ============================================================================
// Backend Event Listeners
// ============================================================================

let unlistenTrackAdded: (() => void) | null = null;
let unlistenTrackRemoved: (() => void) | null = null;
let unlistenTrackUpdated: (() => void) | null = null;

async function setupEventListeners(): Promise<void> {
  // Listen for track changes from backend
  unlistenTrackAdded = await listen('track-added', (event) => {
    const track = event.payload as TrackInfo;
    update(s => {
      const newTracks = new Map(s.tracks);
      newTracks.set(track.id, track);
      return { ...s, tracks: newTracks, modified: true };
    });
  });

  unlistenTrackRemoved = await listen('track-removed', (event) => {
    const trackId = event.payload as number;
    update(s => {
      const newTracks = new Map(s.tracks);
      newTracks.delete(trackId);
      return {
        ...s,
        tracks: newTracks,
        selectedTrackId: s.selectedTrackId === trackId ? null : s.selectedTrackId,
        modified: true
      };
    });
  });

  unlistenTrackUpdated = await listen('track-updated', (event) => {
    const track = event.payload as TrackInfo;
    update(s => {
      const newTracks = new Map(s.tracks);
      newTracks.set(track.id, track);
      return { ...s, tracks: newTracks, modified: true };
    });
  });
}

function cleanupEventListeners(): void {
  if (unlistenTrackAdded) unlistenTrackAdded();
  if (unlistenTrackRemoved) unlistenTrackRemoved();
  if (unlistenTrackUpdated) unlistenTrackUpdated();
}

// Initialize listeners
setupEventListeners().catch(console.error);

// ============================================================================
// Helper Functions
// ============================================================================

function getNextTrackColor(trackCount: number): string {
  return TRACK_COLORS[trackCount % TRACK_COLORS.length];
}

function createDefaultTrack(id: number, label: string, order: number): TrackInfo {
  return {
    id,
    label,
    color: getNextTrackColor(order),
    muted: false,
    solo: false,
    armed: false,
    visible: true,
    volume: 0.8,
    pan: 0.0,
    instrument: null,
    midiChannel: Math.min(id, 16),
    order
  };
}

// ============================================================================
// Actions
// ============================================================================

export const projectActions = {
  /**
   * Create a new project
   */
  async newProject(name: string = 'Untitled Project'): Promise<void> {
    try {
      const projectId = await invoke<string>('create_project', { name });

      set({
        projectId,
        projectName: name,
        tracks: new Map(),
        selectedTrackId: null,
        modified: false,
        lastSavedTime: Date.now(),
        trackIdCounter: 1
      });
    } catch (error) {
      console.error('Failed to create project:', error);
      throw error;
    }
  },

  /**
   * Load an existing project
   */
  async loadProject(projectId: string): Promise<void> {
    try {
      const project = await invoke<{
        id: string;
        name: string;
        tracks: TrackInfo[];
      }>('load_project', { projectId });

      const tracksMap = new Map<number, TrackInfo>();
      let maxId = 0;

      for (const track of project.tracks) {
        tracksMap.set(track.id, track);
        if (track.id > maxId) maxId = track.id;
      }

      set({
        projectId: project.id,
        projectName: project.name,
        tracks: tracksMap,
        selectedTrackId: tracksMap.size > 0 ? project.tracks[0].id : null,
        modified: false,
        lastSavedTime: Date.now(),
        trackIdCounter: maxId + 1
      });
    } catch (error) {
      console.error('Failed to load project:', error);
      throw error;
    }
  },

  /**
   * Save the current project
   */
  async saveProject(): Promise<void> {
    const state = get(projectStore);
    if (!state.projectId) {
      throw new Error('No project to save');
    }

    try {
      const tracks = Array.from(state.tracks.values());
      await invoke('save_project', {
        projectId: state.projectId,
        name: state.projectName,
        tracks
      });

      update(s => ({
        ...s,
        modified: false,
        lastSavedTime: Date.now()
      }));
    } catch (error) {
      console.error('Failed to save project:', error);
      throw error;
    }
  },

  /**
   * Add a new track
   */
  async addTrack(label?: string): Promise<number> {
    const state = get(projectStore);
    const trackId = state.trackIdCounter;
    const trackLabel = label || `Track ${trackId}`;
    const order = state.tracks.size;

    const track = createDefaultTrack(trackId, trackLabel, order);

    try {
      await invoke('add_track', { track });

      update(s => {
        const newTracks = new Map(s.tracks);
        newTracks.set(trackId, track);
        return {
          ...s,
          tracks: newTracks,
          trackIdCounter: trackId + 1,
          selectedTrackId: trackId,
          modified: true
        };
      });

      return trackId;
    } catch (error) {
      console.error('Failed to add track:', error);
      throw error;
    }
  },

  /**
   * Remove a track
   */
  async removeTrack(trackId: number): Promise<void> {
    try {
      await invoke('remove_track', { trackId });

      update(s => {
        const newTracks = new Map(s.tracks);
        newTracks.delete(trackId);

        return {
          ...s,
          tracks: newTracks,
          selectedTrackId: s.selectedTrackId === trackId ? null : s.selectedTrackId,
          modified: true
        };
      });
    } catch (error) {
      console.error('Failed to remove track:', error);
      throw error;
    }
  },

  /**
   * Select a track
   */
  selectTrack(trackId: number | null): void {
    update(s => ({ ...s, selectedTrackId: trackId }));
  },

  /**
   * Update track properties
   */
  async updateTrack(trackId: number, updates: Partial<TrackInfo>): Promise<void> {
    const state = get(projectStore);
    const track = state.tracks.get(trackId);

    if (!track) {
      throw new Error(`Track ${trackId} not found`);
    }

    const updatedTrack = { ...track, ...updates };

    try {
      await invoke('update_track', { track: updatedTrack });

      update(s => {
        const newTracks = new Map(s.tracks);
        newTracks.set(trackId, updatedTrack);
        return { ...s, tracks: newTracks, modified: true };
      });
    } catch (error) {
      console.error('Failed to update track:', error);
      throw error;
    }
  },

  /**
   * Set track mute state
   */
  async setTrackMute(trackId: number, muted: boolean): Promise<void> {
    await this.updateTrack(trackId, { muted });
  },

  /**
   * Set track solo state
   */
  async setTrackSolo(trackId: number, solo: boolean): Promise<void> {
    await this.updateTrack(trackId, { solo });
  },

  /**
   * Set track arm state
   */
  async setTrackArm(trackId: number, armed: boolean): Promise<void> {
    await this.updateTrack(trackId, { armed });
  },

  /**
   * Set track volume
   */
  async setTrackVolume(trackId: number, volume: number): Promise<void> {
    if (volume < 0 || volume > 1) {
      throw new Error('Volume must be between 0 and 1');
    }
    await this.updateTrack(trackId, { volume });
  },

  /**
   * Set track pan
   */
  async setTrackPan(trackId: number, pan: number): Promise<void> {
    if (pan < -1 || pan > 1) {
      throw new Error('Pan must be between -1 and 1');
    }
    await this.updateTrack(trackId, { pan });
  },

  /**
   * Rename track
   */
  async renameTrack(trackId: number, label: string): Promise<void> {
    await this.updateTrack(trackId, { label });
  },

  /**
   * Set track color
   */
  async setTrackColor(trackId: number, color: string): Promise<void> {
    await this.updateTrack(trackId, { color });
  },

  /**
   * Reorder tracks
   */
  async reorderTracks(trackIds: number[]): Promise<void> {
    const state = get(projectStore);
    const updatedTracks = new Map<number, TrackInfo>();

    trackIds.forEach((trackId, index) => {
      const track = state.tracks.get(trackId);
      if (track) {
        updatedTracks.set(trackId, { ...track, order: index });
      }
    });

    try {
      await invoke('reorder_tracks', { trackIds });
      update(s => ({ ...s, tracks: updatedTracks, modified: true }));
    } catch (error) {
      console.error('Failed to reorder tracks:', error);
      throw error;
    }
  },

  /**
   * Clear all solo states
   */
  async clearAllSolo(): Promise<void> {
    const state = get(projectStore);
    const updates: Promise<void>[] = [];

    for (const [trackId, track] of state.tracks) {
      if (track.solo) {
        updates.push(this.setTrackSolo(trackId, false));
      }
    }

    await Promise.all(updates);
  },

  /**
   * Reset project to initial state
   */
  reset(): void {
    cleanupEventListeners();
    set(DEFAULT_STATE);
    setupEventListeners().catch(console.error);
  }
};

// ============================================================================
// Derived Stores
// ============================================================================

export const selectedTrack = derived(
  projectStore,
  $project => $project.selectedTrackId !== null
    ? $project.tracks.get($project.selectedTrackId) || null
    : null
);

export const tracksList = derived(
  projectStore,
  $project => Array.from($project.tracks.values()).sort((a, b) => a.order - b.order)
);

export const soloedTracks = derived(
  projectStore,
  $project => Array.from($project.tracks.values()).filter(track => track.solo)
);

export const armedTracks = derived(
  projectStore,
  $project => Array.from($project.tracks.values()).filter(track => track.armed)
);

export const projectModified = derived(
  projectStore,
  $project => $project.modified
);

// ============================================================================
// Export
// ============================================================================

export const projectStore = {
  subscribe,
  set,
  update
};
