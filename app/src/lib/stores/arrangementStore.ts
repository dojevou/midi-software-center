/**
 * Arrangement Store - MPC 3.0 Style Arrangement View State
 *
 * Manages arrangement-specific UI state that extends the sequencerStore.
 * This store handles:
 * - Active clip for editing (piano roll context)
 * - Detached window states (mixer, piano roll)
 * - Track type assignments for color coding
 * - View preferences specific to arrangement
 */

import { writable, derived, get } from 'svelte/store';
import type { SequencerTrackType } from '$lib/types';

// ============================================================================
// TYPES
// ============================================================================

/**
 * Active clip context for piano roll editing
 */
export interface ActiveEditClip {
  clipId: number;
  trackId: number;
  fileId: number | null;
  clipName: string;
}

/**
 * Arrangement-specific UI state
 */
export interface ArrangementState {
  // Editing context
  activeEditClip: ActiveEditClip | null;

  // Detached window states
  mixerDetached: boolean;
  pianoRollDetached: boolean;

  // Track type assignments (trackId -> trackType)
  trackTypes: Record<number, SequencerTrackType>;

  // View preferences
  showTrackTypeIcons: boolean;
  showClipNames: boolean;
  showClipPreviews: boolean;
  trackColorMode: 'type' | 'custom'; // Use track type colors or custom colors

  // Split view state (for future piano roll/automation integration)
  splitViewEnabled: boolean;
  splitViewHeight: number; // Height percentage for bottom panel (0-50)
  splitViewContent: 'piano-roll' | 'automation' | 'velocity' | null;
}

// ============================================================================
// INITIAL STATE
// ============================================================================

const initialState: ArrangementState = {
  activeEditClip: null,
  mixerDetached: false,
  pianoRollDetached: false,
  trackTypes: {},
  showTrackTypeIcons: true,
  showClipNames: true,
  showClipPreviews: true,
  trackColorMode: 'type',
  splitViewEnabled: false,
  splitViewHeight: 30,
  splitViewContent: null,
};

// ============================================================================
// STORE
// ============================================================================

function createArrangementStore() {
  const { subscribe, set, update } = writable<ArrangementState>(initialState);

  return {
    subscribe,

    // ========================================================================
    // CLIP EDITING ACTIONS
    // ========================================================================

    /**
     * Set the active clip for editing (opens in piano roll)
     */
    setActiveEditClip: (clip: ActiveEditClip | null) => {
      update((state) => ({
        ...state,
        activeEditClip: clip,
      }));
    },

    /**
     * Clear the active edit clip
     */
    clearActiveEditClip: () => {
      update((state) => ({
        ...state,
        activeEditClip: null,
      }));
    },

    // ========================================================================
    // WINDOW DETACHMENT ACTIONS
    // ========================================================================

    /**
     * Toggle mixer detached state
     */
    toggleMixerDetached: () => {
      update((state) => ({
        ...state,
        mixerDetached: !state.mixerDetached,
      }));
    },

    /**
     * Set mixer detached state
     */
    setMixerDetached: (detached: boolean) => {
      update((state) => ({
        ...state,
        mixerDetached: detached,
      }));
    },

    /**
     * Toggle piano roll detached state
     */
    togglePianoRollDetached: () => {
      update((state) => ({
        ...state,
        pianoRollDetached: !state.pianoRollDetached,
      }));
    },

    /**
     * Set piano roll detached state
     */
    setPianoRollDetached: (detached: boolean) => {
      update((state) => ({
        ...state,
        pianoRollDetached: detached,
      }));
    },

    // ========================================================================
    // TRACK TYPE ACTIONS
    // ========================================================================

    /**
     * Set track type for a specific track
     */
    setTrackType: (trackId: number, trackType: SequencerTrackType) => {
      update((state) => ({
        ...state,
        trackTypes: {
          ...state.trackTypes,
          [trackId]: trackType,
        },
      }));
    },

    /**
     * Set multiple track types at once
     */
    setTrackTypes: (trackTypes: Record<number, SequencerTrackType>) => {
      update((state) => ({
        ...state,
        trackTypes: {
          ...state.trackTypes,
          ...trackTypes,
        },
      }));
    },

    /**
     * Remove track type assignment
     */
    removeTrackType: (trackId: number) => {
      update((state) => {
        const { [trackId]: _, ...rest } = state.trackTypes;
        return {
          ...state,
          trackTypes: rest,
        };
      });
    },

    /**
     * Clear all track type assignments
     */
    clearTrackTypes: () => {
      update((state) => ({
        ...state,
        trackTypes: {},
      }));
    },

    // ========================================================================
    // VIEW PREFERENCE ACTIONS
    // ========================================================================

    /**
     * Toggle track type icons visibility
     */
    toggleTrackTypeIcons: () => {
      update((state) => ({
        ...state,
        showTrackTypeIcons: !state.showTrackTypeIcons,
      }));
    },

    /**
     * Toggle clip names visibility
     */
    toggleClipNames: () => {
      update((state) => ({
        ...state,
        showClipNames: !state.showClipNames,
      }));
    },

    /**
     * Toggle clip previews visibility
     */
    toggleClipPreviews: () => {
      update((state) => ({
        ...state,
        showClipPreviews: !state.showClipPreviews,
      }));
    },

    /**
     * Set track color mode
     */
    setTrackColorMode: (mode: 'type' | 'custom') => {
      update((state) => ({
        ...state,
        trackColorMode: mode,
      }));
    },

    // ========================================================================
    // SPLIT VIEW ACTIONS
    // ========================================================================

    /**
     * Toggle split view
     */
    toggleSplitView: () => {
      update((state) => ({
        ...state,
        splitViewEnabled: !state.splitViewEnabled,
      }));
    },

    /**
     * Set split view content type
     */
    setSplitViewContent: (
      content: 'piano-roll' | 'automation' | 'velocity' | null
    ) => {
      update((state) => ({
        ...state,
        splitViewContent: content,
        splitViewEnabled: content !== null,
      }));
    },

    /**
     * Set split view height percentage
     */
    setSplitViewHeight: (height: number) => {
      update((state) => ({
        ...state,
        splitViewHeight: Math.max(10, Math.min(50, height)),
      }));
    },

    // ========================================================================
    // UTILITY ACTIONS
    // ========================================================================

    /**
     * Reset to initial state
     */
    reset: () => {
      set(initialState);
    },

    /**
     * Get current state (for debugging)
     */
    getState: () => get({ subscribe }),
  };
}

// ============================================================================
// EXPORTS
// ============================================================================

export const arrangementStore = createArrangementStore();

// Convenience exports for common operations
export const arrangementActions = {
  setActiveEditClip: arrangementStore.setActiveEditClip,
  clearActiveEditClip: arrangementStore.clearActiveEditClip,
  toggleMixerDetached: arrangementStore.toggleMixerDetached,
  setMixerDetached: arrangementStore.setMixerDetached,
  togglePianoRollDetached: arrangementStore.togglePianoRollDetached,
  setPianoRollDetached: arrangementStore.setPianoRollDetached,
  setTrackType: arrangementStore.setTrackType,
  setTrackTypes: arrangementStore.setTrackTypes,
  removeTrackType: arrangementStore.removeTrackType,
  clearTrackTypes: arrangementStore.clearTrackTypes,
  toggleTrackTypeIcons: arrangementStore.toggleTrackTypeIcons,
  toggleClipNames: arrangementStore.toggleClipNames,
  toggleClipPreviews: arrangementStore.toggleClipPreviews,
  setTrackColorMode: arrangementStore.setTrackColorMode,
  toggleSplitView: arrangementStore.toggleSplitView,
  setSplitViewContent: arrangementStore.setSplitViewContent,
  setSplitViewHeight: arrangementStore.setSplitViewHeight,
  reset: arrangementStore.reset,
};

// ============================================================================
// DERIVED STORES
// ============================================================================

/**
 * Check if any window is detached
 */
export const hasDetachedWindows = derived(
  arrangementStore,
  ($state) => $state.mixerDetached || $state.pianoRollDetached
);

/**
 * Get active edit clip (convenience)
 */
export const activeEditClip = derived(
  arrangementStore,
  ($state) => $state.activeEditClip
);

/**
 * Check if split view is showing piano roll
 */
export const showingPianoRoll = derived(
  arrangementStore,
  ($state) =>
    $state.splitViewEnabled && $state.splitViewContent === 'piano-roll'
);

/**
 * Check if split view is showing automation
 */
export const showingAutomation = derived(
  arrangementStore,
  ($state) =>
    $state.splitViewEnabled && $state.splitViewContent === 'automation'
);
