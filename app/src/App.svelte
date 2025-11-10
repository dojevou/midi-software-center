<script lang="ts">
  import { onMount } from 'svelte';
  import { setupEventListeners } from '$lib/events';
  import { playbackStore, playbackActions } from '$lib/stores/playbackStore';
  import { projectStore, projectActions } from '$lib/stores/projectStore';
  import { uiStore, uiActions } from '$lib/stores/uiStore';
  import { pipelineActions } from '$lib/stores/pipelineStore';
  import { analysisActions } from '$lib/stores/analysisStore';
  import { archiveActions } from '$lib/stores/archiveStore';
  import MenuBar from '$lib/components/MenuBar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import DAWWindow from '$lib/windows/DAWWindow.svelte';
  import MixerWindow from '$lib/windows/MixerWindow.svelte';
  import DatabaseWindow from '$lib/windows/DatabaseWindow.svelte';
  import PipelineWindow from '$lib/windows/PipelineWindow.svelte';

  let destroy: (() => void) | undefined;

  onMount(() => {
    (async () => {
      try {
        destroy = await setupEventListeners({
          onPipelineProgress: (progress) => {
            pipelineActions.updateProgress(progress);
          },
          onPipelineComplete: (result) => {
            pipelineActions.setComplete(result);
          },
          onAnalysisProgress: (progress) => {
            // Transform AnalysisProgressPayload to AnalysisProgress
            analysisActions.updateProgress({
              current: progress.current,
              total: progress.total,
              current_file: progress.current_file,
              rate: progress.rate
            });
          },
          onAnalysisComplete: (result) => {
            // Transform AnalysisSummaryPayload to AnalysisSummary
            analysisActions.setComplete({
              total_files: result.total_analyzed,
              analyzed: result.success,
              failed: result.failed,
              errors: [],
              duration_secs: result.duration_secs,
              rate: result.success / result.duration_secs
            });
          },
          onArchiveProgress: (progress) => {
            // Transform ArchiveProgressPayload to ArchiveProgress
            archiveActions.updateProgress({
              current: progress.extracted_count,
              total: progress.total_count,
              current_archive: progress.current_file,
              rate: 0
            });
          },
          onArchiveError: (error) => {
            // Transform { path, error } to ArchiveError
            archiveActions.addError({
              archivePath: error.path,
              error: error.error
            });
          },
          onProgressUpdate: (update) => {
            // General progress - handled by specific stores
          },
          onPlaybackStarted: () => {
            playbackStore.update(state => ({ ...state, isPlaying: true, isPaused: false }));
          },
          onPlaybackStopped: () => {
            playbackStore.update(state => ({
              ...state,
              isPlaying: false,
              isPaused: false,
              position: { current_tick: 0, current_bar: 0, current_beat: 0 }
            }));
          },
          onPlaybackPaused: () => {
            playbackStore.update(state => ({ ...state, isPlaying: false, isPaused: true }));
          },
          onPlaybackPosition: (payload) => {
            playbackActions.updatePosition(payload.position);
          },
          onTrackAdded: (trackId) => {
            projectActions.loadTracks();
            projectStore.update(state => ({ ...state, selectedTrackId: trackId, hasUnsavedChanges: true }));
          },
          onTrackRemoved: (trackId) => {
            projectStore.update(state => ({
              ...state,
              tracks: state.tracks.filter(t => t.id !== trackId),
              selectedTrackId: state.selectedTrackId === trackId ? null : state.selectedTrackId,
              hasUnsavedChanges: true
            }));
          },
          onCommandToggleSidebar: () => {
            uiActions.toggleSidebar();
          },
          onCommandToggleInspector: () => {
            uiActions.toggleInspector();
          }
        });
      } catch (error) {
        console.error('Failed to setup event listeners:', error);
      }
    })();

    return () => {
      if (destroy) {
        destroy();
      }
    };
  });
</script>

<MenuBar />

<div class="workspace">
  <DAWWindow />
  <MixerWindow />
  <DatabaseWindow />
  <PipelineWindow />
</div>

<StatusBar />

<style>
  .workspace {
    position: relative;
    height: calc(100vh - 4rem); /* Adjust for menu and status bar */
    overflow: hidden;
    background-color: var(--bg-primary);
  }
</style>