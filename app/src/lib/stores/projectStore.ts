import { derived, get, writable } from 'svelte/store';
import { api } from '$lib/api';
import type { MidiPattern, Track } from '$lib/types';

// ============================================================================
// PROJECT STATE ✅ NEW - Completely missing from v1.0
// ============================================================================

export interface ProjectState {
  tracks: Track[];
  selectedTrackId: number | null;
  clipboardContent: MidiPattern | null;
  hasUnsavedChanges: boolean;
  projectName: string;
}

const initialState: ProjectState = {
  tracks: [],
  selectedTrackId: null,
  clipboardContent: null,
  hasUnsavedChanges: false,
  projectName: 'Untitled Project',
};

export const projectStore = writable<ProjectState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const selectedTrack = derived(
  projectStore,
  ($project) => $project.tracks.find((t) => t.id === $project.selectedTrackId) || null
);

export const trackCount = derived(projectStore, ($project) => $project.tracks.length);

export const hasSelection = derived(projectStore, ($project) => $project.selectedTrackId !== null);

// ============================================================================
// ACTIONS
// ============================================================================

export const projectActions = {
  async addTrack(fileId: number, channel: number) {
    try {
      const track = await api.sequencer.addTrack(fileId, channel);
      projectStore.update((state) => ({
        ...state,
        tracks: [...state.tracks, track],
        hasUnsavedChanges: true,
      }));
      return track;
    } catch (error) {
      console.error('Failed to add track:', error);
      throw error;
    }
  },

  async removeTrack(trackId: number) {
    try {
      await api.sequencer.removeTrack(trackId);
      projectStore.update((state) => ({
        ...state,
        tracks: state.tracks.filter((t) => t.id !== trackId),
        selectedTrackId: state.selectedTrackId === trackId ? null : state.selectedTrackId,
        hasUnsavedChanges: true,
      }));
    } catch (error) {
      console.error('Failed to remove track:', error);
      throw error;
    }
  },

  async updateTrack(trackId: number, properties: Partial<Track>) {
    try {
      await api.sequencer.updateTrack(trackId, properties);
      projectStore.update((state) => ({
        ...state,
        tracks: state.tracks.map((t) => (t.id === trackId ? { ...t, ...properties } : t)),
        hasUnsavedChanges: true,
      }));
    } catch (error) {
      console.error('Failed to update track:', error);
      throw error;
    }
  },

  selectTrack(trackId: number | null) {
    projectStore.update((state) => ({ ...state, selectedTrackId: trackId }));
  },

  copyPattern(pattern: MidiPattern) {
    projectStore.update((state) => ({ ...state, clipboardContent: pattern }));
  },

  pastePattern(): MidiPattern | null {
    const state = get(projectStore);
    return state.clipboardContent;
  },

  async loadTracks() {
    try {
      const tracks = await api.sequencer.getTracks();
      projectStore.update((state) => ({ ...state, tracks }));
    } catch (error) {
      console.error('Failed to load tracks:', error);
      throw error;
    }
  },

  async clearAllTracks() {
    try {
      await api.project.clearAllTracks();
      projectStore.update((state) => ({
        ...state,
        tracks: [],
        selectedTrackId: null,
        hasUnsavedChanges: true,
      }));
    } catch (error) {
      console.error('Failed to clear all tracks:', error);
      throw error;
    }
  },

  markSaved() {
    projectStore.update((state) => ({ ...state, hasUnsavedChanges: false }));
  },

  markUnsaved() {
    projectStore.update((state) => ({ ...state, hasUnsavedChanges: true }));
  },

  setProjectName(name: string) {
    projectStore.update((state) => ({ ...state, projectName: name, hasUnsavedChanges: true }));
  },

  // Project file operations (Ctrl+N, Ctrl+O, Ctrl+S)
  async newProject() {
    const state = get(projectStore);
    if (state.hasUnsavedChanges) {
      const proceed = window.confirm('You have unsaved changes. Create new project anyway?');
      if (!proceed) {return;}
    }
    try {
      await api.project.create({ name: 'Untitled Project' });
      projectStore.set({ ...initialState });
    } catch (error) {
      console.error('Failed to create new project:', error);
      // Fallback: just reset state locally
      projectStore.set({ ...initialState });
    }
  },

  async openProject() {
    const state = get(projectStore);
    if (state.hasUnsavedChanges) {
      const proceed = window.confirm('You have unsaved changes. Open another project anyway?');
      if (!proceed) {return;}
    }
    try {
      // Open project dialog and load selected project
      await api.project.openProjectDialog();
    } catch (error) {
      console.error('Failed to open project:', error);
    }
  },

  async saveProject() {
    try {
      await api.project.saveProject();
      projectStore.update((s) => ({ ...s, hasUnsavedChanges: false }));
    } catch (error) {
      console.error('Failed to save project:', error);
    }
  },
};
