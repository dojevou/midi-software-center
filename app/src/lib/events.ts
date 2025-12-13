import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ImportProgress, ImportSummary, PlaybackPosition } from './types';
import { isTauri } from './utils/tauri';

// ============================================================================
// EVENT PAYLOAD TYPES
// ============================================================================

export interface AnalysisProgressPayload {
  current: number;
  total: number;
  current_file: string;
  rate: number;
}

export interface AnalysisSummaryPayload {
  total_analyzed: number;
  success: number;
  failed: number;
  duration_secs: number;
}

export interface ArchiveProgressPayload {
  current_file: string;
  extracted_count: number;
  total_count: number;
}

export interface ProgressStatePayload {
  state: 'idle' | 'running' | 'paused' | 'complete' | 'error';
  progress: number; // 0.0-1.0
  message: string;
}

export interface PlaybackPositionPayload {
  position: PlaybackPosition;
}

// ============================================================================
// EVENT LISTENER SETUP
// ============================================================================

export interface EventCallbacks {
  // Pipeline Events
  onPipelineProgress?: (progress: ImportProgress) => void;
  onPipelineComplete?: (summary: ImportSummary) => void;
  onAnalysisProgress?: (progress: AnalysisProgressPayload) => void;
  onAnalysisComplete?: (summary: AnalysisSummaryPayload) => void;
  onArchiveProgress?: (progress: ArchiveProgressPayload) => void;
  onArchiveError?: (error: { path: string; error: string }) => void;
  onProgressUpdate?: (state: ProgressStatePayload) => void;

  // Sequencer Events
  onPlaybackStarted?: () => void;
  onPlaybackStopped?: () => void;
  onPlaybackPaused?: () => void;
  onPlaybackPosition?: (payload: PlaybackPositionPayload) => void;
  onTrackAdded?: (trackId: number) => void;
  onTrackRemoved?: (trackId: number) => void;

  // Window Events
  onCommandToggleSidebar?: () => void;
  onCommandToggleInspector?: () => void;
}

/**
 * Setup all event listeners
 * Returns cleanup function to unlisten all events
 */
export async function setupEventListeners(callbacks: EventCallbacks): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];

  // Check if we're in Tauri context
  if (!isTauri()) {
    console.warn('[Events] Not running in Tauri context - event listeners disabled');
    return () => {
      /* no-op cleanup */
    };
  }

  try {
    // Pipeline events
    if (callbacks.onPipelineProgress) {
      const cb = callbacks.onPipelineProgress;
      const unlisten = await listen<ImportProgress>('pipeline-progress', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling pipeline progress event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPipelineComplete) {
      const cb = callbacks.onPipelineComplete;
      const unlisten = await listen<ImportSummary>('pipeline-complete', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling pipeline complete event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onAnalysisProgress) {
      const cb = callbacks.onAnalysisProgress;
      const unlisten = await listen<AnalysisProgressPayload>('analysis-progress', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling analysis progress event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onAnalysisComplete) {
      const cb = callbacks.onAnalysisComplete;
      const unlisten = await listen<AnalysisSummaryPayload>('analysis-complete', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling analysis complete event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onArchiveProgress) {
      const cb = callbacks.onArchiveProgress;
      const unlisten = await listen<ArchiveProgressPayload>('archive-progress', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling archive progress event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onArchiveError) {
      const cb = callbacks.onArchiveError;
      const unlisten = await listen<{ path: string; error: string }>('archive-error', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling archive error event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onProgressUpdate) {
      const cb = callbacks.onProgressUpdate;
      const unlisten = await listen<ProgressStatePayload>('progress-update', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling progress update event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    // Sequencer events
    if (callbacks.onPlaybackStarted) {
      const cb = callbacks.onPlaybackStarted;
      const unlisten = await listen('playback-started', () => {
        try {
          cb();
        } catch (error) {
          console.error('Error handling playback started event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPlaybackStopped) {
      const cb = callbacks.onPlaybackStopped;
      const unlisten = await listen('playback-stopped', () => {
        try {
          cb();
        } catch (error) {
          console.error('Error handling playback stopped event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPlaybackPaused) {
      const cb = callbacks.onPlaybackPaused;
      const unlisten = await listen('playback-paused', () => {
        try {
          cb();
        } catch (error) {
          console.error('Error handling playback paused event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPlaybackPosition) {
      const cb = callbacks.onPlaybackPosition;
      const unlisten = await listen<PlaybackPositionPayload>('playback-position', (event) => {
        try {
          cb(event.payload);
        } catch (error) {
          console.error('Error handling playback position event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onTrackAdded) {
      const cb = callbacks.onTrackAdded;
      const unlisten = await listen<{ track_id: number }>('track-added', (event) => {
        try {
          cb(event.payload.track_id);
        } catch (error) {
          console.error('Error handling track added event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onTrackRemoved) {
      const cb = callbacks.onTrackRemoved;
      const unlisten = await listen<{ track_id: number }>('track-removed', (event) => {
        try {
          cb(event.payload.track_id);
        } catch (error) {
          console.error('Error handling track removed event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    // Window command events
    if (callbacks.onCommandToggleSidebar) {
      const cb = callbacks.onCommandToggleSidebar;
      const unlisten = await listen('command:toggle-sidebar', () => {
        try {
          cb();
        } catch (error) {
          console.error('Error handling toggle sidebar event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onCommandToggleInspector) {
      const cb = callbacks.onCommandToggleInspector;
      const unlisten = await listen('command:toggle-inspector', () => {
        try {
          cb();
        } catch (error) {
          console.error('Error handling toggle inspector event:', error);
        }
      });
      unlisteners.push(unlisten);
    }
  } catch (error) {
    console.error('Failed to setup event listeners:', error);
    // Cleanup any partial unlisteners
    unlisteners.forEach((unlisten) => unlisten());
    throw error;
  }

  // Return cleanup function
  return () => {
    unlisteners.forEach((unlisten) => unlisten());
  };
}
