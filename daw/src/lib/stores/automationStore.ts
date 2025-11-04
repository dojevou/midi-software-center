// automationStore.ts
// Grown-up Script: Automation state management with Tauri IPC
// Manages automation lanes, points, and interaction state

import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  AutomationLane,
  AutomationPoint,
  ParameterType,
  CurveType
} from '../types/automation';

/**
 * Selected automation points for editing
 */
export interface SelectedPoints {
  lane_id: number;
  point_ids: Set<number>;
}

/**
 * Zoom and view settings
 */
export interface AutomationView {
  horizontal_zoom: number;  // pixels per tick
  vertical_zoom: number;    // pixels per value unit
  scroll_x: number;         // horizontal scroll position
  scroll_y: number;         // vertical scroll position
}

/**
 * Grid snap settings
 */
export interface AutomationSnapSettings {
  enabled: boolean;
  snap_to_ticks: number;  // e.g., 96 for 1/4 note at 384 ppq
}

/**
 * Main automation store state
 */
interface AutomationState {
  current_track_id: number | null;
  lanes: AutomationLane[];
  selected_points: SelectedPoints | null;
  view: AutomationView;
  snap: AutomationSnapSettings;
  loading: boolean;
  error: string | null;
}

// Create writable store
const initialState: AutomationState = {
  current_track_id: null,
  lanes: [],
  selected_points: null,
  view: {
    horizontal_zoom: 0.1,  // 0.1 pixels per tick
    vertical_zoom: 100,    // 100 pixels per value unit (0-1)
    scroll_x: 0,
    scroll_y: 0,
  },
  snap: {
    enabled: true,
    snap_to_ticks: 96,  // 1/4 note at 384 ppq
  },
  loading: false,
  error: null,
};

export const automationStore: Writable<AutomationState> = writable(initialState);

/**
 * Derived store: Get lanes for current track
 */
export const currentTrackLanes: Readable<AutomationLane[]> = derived(
  automationStore,
  $store => $store.lanes.filter(lane => lane.track_id === $store.current_track_id)
);

/**
 * Derived store: Check if any points are selected
 */
export const hasSelection: Readable<boolean> = derived(
  automationStore,
  $store => $store.selected_points !== null && $store.selected_points.point_ids.size > 0
);

/**
 * Automation actions (Grown-up Script methods with side effects)
 */
export const automationActions = {
  /**
   * Set current track
   */
  setTrack(track_id: number) {
    automationStore.update(s => ({ ...s, current_track_id: track_id }));
    this.loadTrackAutomation(track_id);
  },

  /**
   * Load automation lanes for track
   */
  async loadTrackAutomation(track_id: number) {
    automationStore.update(s => ({ ...s, loading: true, error: null }));

    try {
      const lanes = await invoke<AutomationLane[]>('get_track_automation', { trackId: track_id });

      automationStore.update(s => ({
        ...s,
        lanes: lanes.filter(l => l.track_id === track_id),
        loading: false
      }));
    } catch (error) {
      console.error('Failed to load automation:', error);
      automationStore.update(s => ({
        ...s,
        loading: false,
        error: error instanceof Error ? error.message : String(error)
      }));
    }
  },

  /**
   * Create automation lane
   */
  async createLane(track_id: number, parameter_type: ParameterType) {
    try {
      const lane_id = await invoke<number>('create_automation_lane', {
        trackId: track_id,
        parameterType: parameter_type
      });

      // Reload lanes for this track
      await this.loadTrackAutomation(track_id);

      return lane_id;
    } catch (error) {
      console.error('Failed to create lane:', error);
      automationStore.update(s => ({
        ...s,
        error: error instanceof Error ? error.message : String(error)
      }));
      throw error;
    }
  },

  /**
   * Delete automation lane
   */
  async deleteLane(track_id: number, parameter_type: ParameterType) {
    try {
      await invoke('delete_automation_lane', {
        trackId: track_id,
        parameterType: parameter_type
      });

      // Remove from local state
      automationStore.update(s => ({
        ...s,
        lanes: s.lanes.filter(l =>
          !(l.track_id === track_id && JSON.stringify(l.parameter_type) === JSON.stringify(parameter_type))
        )
      }));
    } catch (error) {
      console.error('Failed to delete lane:', error);
      throw error;
    }
  },

  /**
   * Add automation point
   */
  async addPoint(track_id: number, parameter_type: ParameterType, time: number, value: number) {
    // Apply snap if enabled
    const state = get(automationStore);
    const snapped_time = state.snap.enabled
      ? Math.round(time / state.snap.snap_to_ticks) * state.snap.snap_to_ticks
      : time;

    try {
      const point_id = await invoke<number>('add_automation_point', {
        trackId: track_id,
        parameterType: parameter_type,
        time: snapped_time,
        value: Math.max(0, Math.min(1, value))  // Clamp 0-1
      });

      // Reload lanes
      await this.loadTrackAutomation(track_id);

      return point_id;
    } catch (error) {
      console.error('Failed to add point:', error);
      throw error;
    }
  },

  /**
   * Remove automation point
   */
  async removePoint(track_id: number, parameter_type: ParameterType, point_id: number) {
    try {
      await invoke('remove_automation_point', {
        trackId: track_id,
        parameterType: parameter_type,
        pointId: point_id
      });

      // Reload lanes
      await this.loadTrackAutomation(track_id);
    } catch (error) {
      console.error('Failed to remove point:', error);
      throw error;
    }
  },

  /**
   * Move automation point
   */
  async movePoint(
    track_id: number,
    parameter_type: ParameterType,
    point_id: number,
    new_time: number,
    new_value: number
  ) {
    // Apply snap if enabled
    const state = get(automationStore);
    const snapped_time = state.snap.enabled
      ? Math.round(new_time / state.snap.snap_to_ticks) * state.snap.snap_to_ticks
      : new_time;

    try {
      await invoke('move_automation_point', {
        trackId: track_id,
        parameterType: parameter_type,
        pointId: point_id,
        newTime: snapped_time,
        newValue: Math.max(0, Math.min(1, new_value))  // Clamp 0-1
      });

      // Reload lanes
      await this.loadTrackAutomation(track_id);
    } catch (error) {
      console.error('Failed to move point:', error);
      throw error;
    }
  },

  /**
   * Set curve type
   */
  async setCurveType(track_id: number, parameter_type: ParameterType, curve_type: CurveType) {
    try {
      await invoke('set_automation_curve_type', {
        trackId: track_id,
        parameterType: parameter_type,
        curveType: curve_type
      });

      // Reload lanes
      await this.loadTrackAutomation(track_id);
    } catch (error) {
      console.error('Failed to set curve type:', error);
      throw error;
    }
  },

  /**
   * Selection management
   */
  selectPoint(lane_id: number, point_id: number, extend: boolean = false) {
    automationStore.update(s => {
      if (extend && s.selected_points && s.selected_points.lane_id === lane_id) {
        // Extend selection
        const new_ids = new Set(s.selected_points.point_ids);
        new_ids.add(point_id);
        return {
          ...s,
          selected_points: { lane_id, point_ids: new_ids }
        };
      } else {
        // New selection
        return {
          ...s,
          selected_points: { lane_id, point_ids: new Set([point_id]) }
        };
      }
    });
  },

  deselectPoint(point_id: number) {
    automationStore.update(s => {
      if (!s.selected_points) return s;

      const new_ids = new Set(s.selected_points.point_ids);
      new_ids.delete(point_id);

      return {
        ...s,
        selected_points: new_ids.size > 0
          ? { ...s.selected_points, point_ids: new_ids }
          : null
      };
    });
  },

  clearSelection() {
    automationStore.update(s => ({ ...s, selected_points: null }));
  },

  /**
   * View controls
   */
  setZoom(horizontal?: number, vertical?: number) {
    automationStore.update(s => ({
      ...s,
      view: {
        ...s.view,
        horizontal_zoom: horizontal ?? s.view.horizontal_zoom,
        vertical_zoom: vertical ?? s.view.vertical_zoom
      }
    }));
  },

  setScroll(x?: number, y?: number) {
    automationStore.update(s => ({
      ...s,
      view: {
        ...s.view,
        scroll_x: x ?? s.view.scroll_x,
        scroll_y: y ?? s.view.scroll_y
      }
    }));
  },

  /**
   * Snap controls
   */
  toggleSnap() {
    automationStore.update(s => ({
      ...s,
      snap: { ...s.snap, enabled: !s.snap.enabled }
    }));
  },

  setSnapGrid(ticks: number) {
    automationStore.update(s => ({
      ...s,
      snap: { ...s.snap, snap_to_ticks: ticks }
    }));
  },

  /**
   * Reset store
   */
  reset() {
    automationStore.set(initialState);
  }
};

// Helper to get current store value
function get<T>(store: Readable<T>): T {
  let value: T;
  store.subscribe(v => value = v)();
  return value!;
}
