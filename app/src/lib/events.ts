import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ImportProgress, ImportSummary, PlaybackPosition } from './types';

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

  try {
    // Pipeline events
    if (callbacks.onPipelineProgress) {
      const unlisten = await listen<ImportProgress>('pipeline-progress', (event) => {
        try {
          callbacks.onPipelineProgress!(event.payload);
        } catch (error) {
          console.error('Error handling pipeline progress event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPipelineComplete) {
      const unlisten = await listen<ImportSummary>('pipeline-complete', (event) => {
        try {
          callbacks.onPipelineComplete!(event.payload);
        } catch (error) {
          console.error('Error handling pipeline complete event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onAnalysisProgress) {
      const unlisten = await listen<AnalysisProgressPayload>('analysis-progress', (event) => {
        try {
          callbacks.onAnalysisProgress!(event.payload);
        } catch (error) {
          console.error('Error handling analysis progress event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onAnalysisComplete) {
      const unlisten = await listen<AnalysisSummaryPayload>('analysis-complete', (event) => {
        try {
          callbacks.onAnalysisComplete!(event.payload);
        } catch (error) {
          console.error('Error handling analysis complete event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onArchiveProgress) {
      const unlisten = await listen<ArchiveProgressPayload>('archive-progress', (event) => {
        try {
          callbacks.onArchiveProgress!(event.payload);
        } catch (error) {
          console.error('Error handling archive progress event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onArchiveError) {
      const unlisten = await listen<{ path: string; error: string }>('archive-error', (event) => {
        try {
          callbacks.onArchiveError!(event.payload);
        } catch (error) {
          console.error('Error handling archive error event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onProgressUpdate) {
      const unlisten = await listen<ProgressStatePayload>('progress-update', (event) => {
        try {
          callbacks.onProgressUpdate!(event.payload);
        } catch (error) {
          console.error('Error handling progress update event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    // Sequencer events
    if (callbacks.onPlaybackStarted) {
      const unlisten = await listen('playback-started', () => {
        try {
          callbacks.onPlaybackStarted!();
        } catch (error) {
          console.error('Error handling playback started event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPlaybackStopped) {
      const unlisten = await listen('playback-stopped', () => {
        try {
          callbacks.onPlaybackStopped!();
        } catch (error) {
          console.error('Error handling playback stopped event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPlaybackPaused) {
      const unlisten = await listen('playback-paused', () => {
        try {
          callbacks.onPlaybackPaused!();
        } catch (error) {
          console.error('Error handling playback paused event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onPlaybackPosition) {
      const unlisten = await listen<PlaybackPositionPayload>('playback-position', (event) => {
        try {
          callbacks.onPlaybackPosition!(event.payload);
        } catch (error) {
          console.error('Error handling playback position event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onTrackAdded) {
      const unlisten = await listen<{ track_id: number }>('track-added', (event) => {
        try {
          callbacks.onTrackAdded!(event.payload.track_id);
        } catch (error) {
          console.error('Error handling track added event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onTrackRemoved) {
      const unlisten = await listen<{ track_id: number }>('track-removed', (event) => {
        try {
          callbacks.onTrackRemoved!(event.payload.track_id);
        } catch (error) {
          console.error('Error handling track removed event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    // Window command events
    if (callbacks.onCommandToggleSidebar) {
      const unlisten = await listen('command:toggle-sidebar', () => {
        try {
          callbacks.onCommandToggleSidebar!();
        } catch (error) {
          console.error('Error handling toggle sidebar event:', error);
        }
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onCommandToggleInspector) {
      const unlisten = await listen('command:toggle-inspector', () => {
        try {
          callbacks.onCommandToggleInspector!();
        } catch (error) {
          console.error('Error handling toggle inspector event:', error);
        }
      });
      unlisteners.push(unlisten);
    }
  } catch (error) {
    console.error('Failed to setup event listeners:', error);
    // Cleanup any partial unlisteners
    unlisteners.forEach(unlisten => unlisten());
    throw error;
  }

  // Return cleanup function
  return () => {
    unlisteners.forEach(unlisten => unlisten());
  };
}