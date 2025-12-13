import { derived, get, writable } from 'svelte/store';
import { safeInvoke } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';

// ============================================================================
// TYPES
// ============================================================================

export interface AutomationPoint {
  id: number;
  time: number;
  value: number;
  curve: 'linear' | 'exponential' | 'logarithmic' | 'smooth';
}

export interface AutomationLane {
  id: number;
  trackId: number;
  parameterName: string;
  parameterType: string;
  minValue: number;
  maxValue: number;
  defaultValue: number;
  points: AutomationPoint[];
  isVisible: boolean;
  isExpanded: boolean;
  color: string;
}

export interface AutomationState {
  lanes: AutomationLane[];
  selectedLane: number | null;
  selectedPoints: Set<number>;
  currentTool: 'select' | 'draw' | 'erase' | 'line';
  defaultCurve: AutomationPoint['curve'];
  zoomLevel: number;
  isLoading: boolean;
  error: string | null;
}

// ============================================================================
// STORE
// ============================================================================

const initialState: AutomationState = {
  lanes: [],
  selectedLane: null,
  selectedPoints: new Set(),
  currentTool: 'draw',
  defaultCurve: 'linear',
  zoomLevel: 1,
  isLoading: false,
  error: null,
};

const { subscribe, set, update } = writable<AutomationState>(initialState);

export const automationStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const visibleLanes = derived(automationStore, ($store) =>
  $store.lanes.filter((lane) => lane.isVisible)
);

export const selectedLaneData = derived(
  automationStore,
  ($store) => $store.lanes.find((lane) => lane.id === $store.selectedLane) || null
);

export const hasPointSelection = derived(
  automationStore,
  ($store) => $store.selectedPoints.size > 0
);

// ============================================================================
// ACTIONS
// ============================================================================

export const automationActions = {
  /**
   * Load automation lanes for a track
   */
  async loadTrackAutomation(trackId: number): Promise<AutomationLane[]> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const lanes =
        (await safeInvoke<AutomationLane[]>(Commands.GET_TRACK_AUTOMATION, { trackId })) || [];
      update((state) => ({
        ...state,
        lanes,
        isLoading: false,
      }));
      return lanes;
    } catch (error) {
      console.error('Failed to load automation:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
      return [];
    }
  },

  /**
   * Add an automation lane
   */
  async addLane(
    trackId: number,
    parameterName: string,
    parameterType: string
  ): Promise<AutomationLane | null> {
    try {
      const lane = await safeInvoke<AutomationLane>(Commands.ADD_AUTOMATION_LANE, {
        trackId,
        parameterName,
        parameterType,
      });
      if (lane) {
        update((state) => ({
          ...state,
          lanes: [...state.lanes, lane],
        }));
      }
      return lane;
    } catch (error) {
      console.error('Failed to add automation lane:', error);
      return null;
    }
  },

  /**
   * Delete an automation lane
   */
  async deleteLane(laneId: number): Promise<void> {
    try {
      await safeInvoke(Commands.DELETE_AUTOMATION_LANE, { laneId });
      update((state) => ({
        ...state,
        lanes: state.lanes.filter((lane) => lane.id !== laneId),
        selectedLane: state.selectedLane === laneId ? null : state.selectedLane,
      }));
    } catch (error) {
      console.error('Failed to delete automation lane:', error);
    }
  },

  /**
   * Add an automation point
   */
  async addPoint(laneId: number, time: number, value: number): Promise<AutomationPoint | null> {
    const state = get(automationStore);
    try {
      const point = await safeInvoke<AutomationPoint>(Commands.ADD_AUTOMATION_POINT, {
        laneId,
        time,
        value,
        curve: state.defaultCurve,
      });
      if (point) {
        update((s) => ({
          ...s,
          lanes: s.lanes.map((lane) =>
            lane.id === laneId
              ? { ...lane, points: [...lane.points, point].sort((a, b) => a.time - b.time) }
              : lane
          ),
        }));
      }
      return point;
    } catch (error) {
      console.error('Failed to add automation point:', error);
      return null;
    }
  },

  /**
   * Update an automation point
   */
  async updatePoint(
    laneId: number,
    pointId: number,
    updates: Partial<AutomationPoint>
  ): Promise<void> {
    try {
      await safeInvoke(Commands.UPDATE_AUTOMATION_POINT, { laneId, pointId, ...updates });
      update((state) => ({
        ...state,
        lanes: state.lanes.map((lane) =>
          lane.id === laneId
            ? {
                ...lane,
                points: lane.points
                  .map((p) => (p.id === pointId ? { ...p, ...updates } : p))
                  .sort((a, b) => a.time - b.time),
              }
            : lane
        ),
      }));
    } catch (error) {
      console.error('Failed to update automation point:', error);
    }
  },

  /**
   * Delete automation points
   */
  async deletePoints(laneId: number, pointIds: number[]): Promise<void> {
    try {
      await safeInvoke(Commands.DELETE_AUTOMATION_POINTS, { laneId, pointIds });
      update((state) => ({
        ...state,
        lanes: state.lanes.map((lane) =>
          lane.id === laneId
            ? { ...lane, points: lane.points.filter((p) => !pointIds.includes(p.id)) }
            : lane
        ),
        selectedPoints: new Set(),
      }));
    } catch (error) {
      console.error('Failed to delete automation points:', error);
    }
  },

  // ============================================================================
  // LOCAL STATE MANAGEMENT
  // ============================================================================

  selectLane(laneId: number | null): void {
    update((state) => ({ ...state, selectedLane: laneId, selectedPoints: new Set() }));
  },

  selectPoint(pointId: number, addToSelection: boolean = false): void {
    update((state) => {
      const newSelection = addToSelection
        ? new Set([...state.selectedPoints, pointId])
        : new Set([pointId]);
      return { ...state, selectedPoints: newSelection };
    });
  },

  togglePointSelection(pointId: number): void {
    update((state) => {
      const newSelection = new Set(state.selectedPoints);
      if (newSelection.has(pointId)) {
        newSelection.delete(pointId);
      } else {
        newSelection.add(pointId);
      }
      return { ...state, selectedPoints: newSelection };
    });
  },

  clearPointSelection(): void {
    update((state) => ({ ...state, selectedPoints: new Set() }));
  },

  setTool(tool: AutomationState['currentTool']): void {
    update((state) => ({ ...state, currentTool: tool }));
  },

  setDefaultCurve(curve: AutomationPoint['curve']): void {
    update((state) => ({ ...state, defaultCurve: curve }));
  },

  setZoomLevel(level: number): void {
    update((state) => ({ ...state, zoomLevel: Math.max(0.1, Math.min(4, level)) }));
  },

  toggleLaneVisibility(laneId: number): void {
    update((state) => ({
      ...state,
      lanes: state.lanes.map((lane) =>
        lane.id === laneId ? { ...lane, isVisible: !lane.isVisible } : lane
      ),
    }));
  },

  toggleLaneExpanded(laneId: number): void {
    update((state) => ({
      ...state,
      lanes: state.lanes.map((lane) =>
        lane.id === laneId ? { ...lane, isExpanded: !lane.isExpanded } : lane
      ),
    }));
  },

  reset(): void {
    set(initialState);
  },
};
