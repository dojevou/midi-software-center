import { derived, get, writable } from 'svelte/store';
import { safeInvoke } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';

// ============================================================================
// TYPES
// ============================================================================

export interface Note {
  id: number;
  pitch: number;
  velocity: number;
  startTick: number;
  duration: number;
  channel: number;
}

export interface LoopRange {
  start: number;
  end: number;
}

export interface PianoRollState {
  notes: Note[];
  selectedNotes: Set<number>;
  currentTool: 'select' | 'draw' | 'erase' | 'slice';
  gridSize: number; // Grid subdivision (32 = 32nd notes)
  zoomLevel: number;
  snapToGrid: boolean;
  loopRange: LoopRange;
  playbackPosition: number;
  trackId: number | null;
  isLoading: boolean;
  error: string | null;
}

// ============================================================================
// STORE
// ============================================================================

const initialState: PianoRollState = {
  notes: [],
  selectedNotes: new Set(),
  currentTool: 'select',
  gridSize: 32,
  zoomLevel: 1,
  snapToGrid: true,
  loopRange: { start: 0, end: 1920 }, // 4 bars at 480 PPQN
  playbackPosition: 0,
  trackId: null,
  isLoading: false,
  error: null,
};

const { subscribe, set, update } = writable<PianoRollState>(initialState);

export const pianoRollStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const hasSelection = derived(pianoRollStore, ($store) => $store.selectedNotes.size > 0);
export const selectionCount = derived(pianoRollStore, ($store) => $store.selectedNotes.size);
export const sortedNotes = derived(pianoRollStore, ($store) =>
  [...$store.notes].sort((a, b) => a.startTick - b.startTick)
);

// ============================================================================
// ACTIONS
// ============================================================================

export const pianoRollActions = {
  /**
   * Load notes for a specific track
   */
  async loadTrackNotes(trackId: number): Promise<Note[]> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null, trackId }));
      const notes = (await safeInvoke<Note[]>(Commands.GET_TRACK_NOTES, { trackId })) || [];
      update((state) => ({
        ...state,
        notes,
        isLoading: false,
      }));
      return notes;
    } catch (error) {
      console.error('Failed to load track notes:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Add a new note
   */
  async addNote(noteData: Omit<Note, 'id'>): Promise<Note | null> {
    const state = get(pianoRollStore);
    if (!state.trackId) {
      return null;
    }

    try {
      const newNote = await safeInvoke<Note>(Commands.ADD_NOTE, {
        trackId: state.trackId,
        note: noteData,
      });
      if (newNote) {
        update((s) => ({
          ...s,
          notes: [...s.notes, newNote],
          selectedNotes: new Set([newNote.id]),
        }));
      }
      return newNote;
    } catch (error) {
      console.error('Failed to add note:', error);
      return null;
    }
  },

  /**
   * Update multiple notes at once
   */
  async updateNotes(updatedNotes: Partial<Note>[]): Promise<void> {
    try {
      await safeInvoke(Commands.UPDATE_NOTES_BATCH, { notes: updatedNotes });
      update((state) => {
        const noteMap = new Map(state.notes.map((n) => [n.id, n]));
        updatedNotes.forEach((updated) => {
          if (updated.id && noteMap.has(updated.id)) {
            noteMap.set(updated.id, { ...noteMap.get(updated.id)!, ...updated });
          }
        });
        return { ...state, notes: Array.from(noteMap.values()) };
      });
    } catch (error) {
      console.error('Failed to update notes:', error);
    }
  },

  /**
   * Delete selected notes
   */
  async deleteSelectedNotes(): Promise<void> {
    const state = get(pianoRollStore);
    if (state.selectedNotes.size === 0) {
      return;
    }

    try {
      const noteIds = Array.from(state.selectedNotes);
      await safeInvoke(Commands.DELETE_NOTES, { noteIds });
      update((s) => ({
        ...s,
        notes: s.notes.filter((n) => !s.selectedNotes.has(n.id)),
        selectedNotes: new Set(),
      }));
    } catch (error) {
      console.error('Failed to delete notes:', error);
    }
  },

  // ============================================================================
  // LOCAL STATE MANAGEMENT
  // ============================================================================

  selectNote(noteId: number, addToSelection: boolean = false): void {
    update((state) => {
      const newSelection = addToSelection
        ? new Set([...state.selectedNotes, noteId])
        : new Set([noteId]);
      return { ...state, selectedNotes: newSelection };
    });
  },

  deselectNote(noteId: number): void {
    update((state) => {
      const newSelection = new Set(state.selectedNotes);
      newSelection.delete(noteId);
      return { ...state, selectedNotes: newSelection };
    });
  },

  toggleNoteSelection(noteId: number): void {
    update((state) => {
      const newSelection = new Set(state.selectedNotes);
      if (newSelection.has(noteId)) {
        newSelection.delete(noteId);
      } else {
        newSelection.add(noteId);
      }
      return { ...state, selectedNotes: newSelection };
    });
  },

  selectAllNotes(): void {
    update((state) => ({
      ...state,
      selectedNotes: new Set(state.notes.map((n) => n.id)),
    }));
  },

  clearSelection(): void {
    update((state) => ({ ...state, selectedNotes: new Set() }));
  },

  setTool(tool: PianoRollState['currentTool']): void {
    update((state) => ({ ...state, currentTool: tool }));
  },

  setGridSize(size: number): void {
    update((state) => ({ ...state, gridSize: size }));
  },

  setZoomLevel(level: number): void {
    update((state) => ({ ...state, zoomLevel: Math.max(0.1, Math.min(4, level)) }));
  },

  setSnapToGrid(snap: boolean): void {
    update((state) => ({ ...state, snapToGrid: snap }));
  },

  setLoopRange(start: number, end: number): void {
    update((state) => ({ ...state, loopRange: { start, end } }));
  },

  updatePlaybackPosition(position: number): void {
    update((state) => ({ ...state, playbackPosition: position }));
  },

  reset(): void {
    set(initialState);
  },
};
