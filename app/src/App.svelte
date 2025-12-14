<script lang="ts">
  // =============================================================================
  // MIDI Software Center - Main Application Component
  // =============================================================================
  // Enterprise-grade DAW-style application with Pro Tools window management
  // =============================================================================

  import { onMount } from 'svelte';
  import { setupEventListeners } from '$lib/events';

  // Stores
  import { playbackActions, playbackStore } from '$lib/stores/playbackStore';
  import { projectActions, projectStore } from '$lib/stores/projectStore';
  import { uiActions, uiStore, handleKeyboardShortcut } from '$lib/stores/uiStore';
  import { pipelineActions } from '$lib/stores/pipelineStore';
  import { analysisActions } from '$lib/stores/analysisStore';
  import { archiveActions } from '$lib/stores/archiveStore';
  import { keyboardStore } from '$lib/stores/keyboardStore';
  import { a11yStore, isKeyboardNav } from '$lib/stores/a11yStore';
  import { reducedMotion } from '$lib/stores/themeStore';

  // Accessibility Components
  import SkipLinks from '$lib/components/SkipLinks.svelte';

  // Error and Loading Components
  import ErrorToast from '$lib/components/ErrorToast.svelte';
  import LoadingOverlay from '$lib/components/LoadingOverlay.svelte';

  // Core Components
  import MenuBar from '$lib/components/MenuBar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import WindowDockBar from '$lib/components/WindowDockBar.svelte';
  import KeyboardShortcuts from '$lib/components/KeyboardShortcuts.svelte';

  // Window Components
  import ArrangementWindow from '$lib/windows/ArrangementWindow.svelte';
  import MixerWindow from '$lib/windows/MixerWindow.svelte';
  import DatabaseWindow from '$lib/windows/DatabaseWindow.svelte';
  import PipelineWindow from '$lib/windows/PipelineWindow.svelte';
  import PianoRollWindow from '$lib/windows/PianoRollWindow.svelte';
  import MidiIOSetupWindow from '$lib/windows/MidiIOSetupWindow.svelte';
  import MidiMonitorWindow from '$lib/windows/MidiMonitorWindow.svelte';
  import PreferencesWindow from '$lib/windows/PreferencesWindow.svelte';
  import GearManagerWindow from '$lib/windows/GearManagerWindow.svelte';
  import PresetsManagerWindow from '$lib/windows/PresetsManagerWindow.svelte';
import ScoreWindow from '$lib/windows/ScoreWindow.svelte';
import ScriptEditorWindow from '$lib/windows/ScriptEditorWindow.svelte';
import MidiLearnWindow from '$lib/windows/MidiLearnWindow.svelte';
import LinkSyncWindow from '$lib/windows/LinkSyncWindow.svelte';
import MidiLearnOverlay from '$lib/components/MidiLearnOverlay.svelte';
import { isLearning } from '$lib/stores/learnStore';

  let destroy: (() => void) | undefined;
  let showShortcuts = false;

  // Window IDs for keyboard navigation
  type WindowId = 'arrangement' | 'daw' | 'mixer' | 'database' | 'pipeline' | 'piano-roll' | 'midi-io-setup' | 'midi-monitor' | 'preferences' | 'gear-manager' | 'presets-manager' | 'score' | 'script-editor' | 'midi-learn' | 'link-sync';

  const windowIds: WindowId[] = [
    'arrangement', 'mixer', 'database', 'pipeline', 'piano-roll',
    'midi-io-setup', 'midi-monitor', 'preferences', 'gear-manager', 'presets-manager',
    'score', 'script-editor', 'midi-learn', 'link-sync'
  ];

  const popupWindows: WindowId[] = [
    'presets-manager', 'gear-manager', 'preferences', 'piano-roll',
    'midi-monitor', 'midi-io-setup', 'pipeline',
    'score', 'script-editor', 'midi-learn', 'link-sync'
  ];

  onMount(() => {
    // Initialize main windows visible, popups hidden
    // Arrangement is the default main view (MPC 3.0 style)
    uiActions.showWindow('arrangement');
    uiActions.showWindow('mixer');
    uiActions.showWindow('database');
    uiActions.hideWindow('daw');  // DAW window replaced by Arrangement
    uiActions.hideWindow('pipeline');
    uiActions.hideWindow('piano-roll');  // Piano Roll opens on demand

    // Initialize keyboard shortcuts store with action mappings
    keyboardStore.init({
      'transport.play': () => void playbackActions.play(),
      'transport.stop': () => void playbackActions.stop(),
      'transport.pause': () => void playbackActions.pause(),
      'transport.record': () => console.log('Record not implemented'),
      'transport.rewind': () => console.log('Rewind not implemented'),
      'transport.forward': () => console.log('Forward not implemented'),
      'file.new': () => void projectActions.newProject(),
      'file.open': () => void projectActions.openProject(),
      'file.save': () => void projectActions.saveProject(),
      'file.saveAs': () => console.log('Save As not implemented'),
      'file.close': () => window.close(),
      'editing.undo': () => console.log('Undo not implemented'),
      'editing.redo': () => console.log('Redo not implemented'),
      'editing.cut': () => console.log('Cut not implemented'),
      'editing.copy': () => console.log('Copy not implemented'),
      'editing.paste': () => console.log('Paste not implemented'),
      'editing.delete': () => console.log('Delete not implemented'),
      'editing.selectAll': () => console.log('Select All not implemented'),
      'navigation.zoomIn': () => console.log('Zoom In not implemented'),
      'navigation.zoomOut': () => console.log('Zoom Out not implemented'),
      'navigation.zoomFit': () => console.log('Zoom Fit not implemented'),
      'windows.togglePipeline': () => uiActions.toggleWindow('pipeline'),
      'windows.toggleMixer': () => uiActions.toggleWindow('mixer'),
      'windows.toggleDatabase': () => uiActions.toggleWindow('database'),
      'windows.toggleMidiMonitor': () => uiActions.toggleWindow('midi-monitor'),
      'windows.showShortcuts': () => { showShortcuts = true; },
      'window.score': () => uiActions.openWindow('score'),
      'window.scriptEditor': () => uiActions.openWindow('script-editor'),
      'window.midiLearn': () => uiActions.openWindow('midi-learn'),
      'window.linkSync': () => uiActions.openWindow('link-sync'),
      'view.toggleSidebar': () => uiActions.toggleSidebar(),
      'view.toggleInspector': () => uiActions.toggleInspector(),
      'view.toggleFullscreen': () => console.log('Fullscreen not implemented'),
      'tools.quantize': () => console.log('Quantize not implemented'),
      'tools.transpose': () => console.log('Transpose not implemented'),
      'tools.velocity': () => console.log('Velocity not implemented'),
    });

    // Setup backend event listeners
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
              rate: progress.rate,
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
              rate: result.success / result.duration_secs,
            });
          },
          onArchiveProgress: (progress) => {
            // Transform ArchiveProgressPayload to ArchiveProgress
            archiveActions.updateProgress({
              current: progress.extracted_count,
              total: progress.total_count,
              current_archive: progress.current_file,
              rate: 0,
            });
          },
          onArchiveError: (error) => {
            // Transform { path, error } to ArchiveError
            archiveActions.addError({
              archivePath: error.path,
              error: error.error,
            });
          },
          onProgressUpdate: (_update) => {
            // General progress - handled by specific stores
          },
          onPlaybackStarted: () => {
            playbackStore.update((state) => ({ ...state, isPlaying: true, isPaused: false }));
          },
          onPlaybackStopped: () => {
            playbackStore.update((state) => ({
              ...state,
              isPlaying: false,
              isPaused: false,
              position: { current_tick: 0, current_bar: 0, current_beat: 0 },
            }));
          },
          onPlaybackPaused: () => {
            playbackStore.update((state) => ({ ...state, isPlaying: false, isPaused: true }));
          },
          onPlaybackPosition: (payload) => {
            playbackActions.updatePosition(payload.position);
          },
          onTrackAdded: (trackId) => {
            void projectActions.loadTracks();
            projectStore.update((state) => ({
              ...state,
              selectedTrackId: trackId,
              hasUnsavedChanges: true,
            }));
          },
          onTrackRemoved: (trackId) => {
            projectStore.update((state) => ({
              ...state,
              tracks: state.tracks.filter((t) => t.id !== trackId),
              selectedTrackId: state.selectedTrackId === trackId ? null : state.selectedTrackId,
              hasUnsavedChanges: true,
            }));
          },
          onCommandToggleSidebar: () => {
            uiActions.toggleSidebar();
          },
          onCommandToggleInspector: () => {
            uiActions.toggleInspector();
          },
        });
      } catch (error) {
        console.error('Failed to setup event listeners:', error);
      }
    })();

    // Global keyboard shortcuts - Pro Tools style
    const handleKeydown = (event: KeyboardEvent) => {
      // Ignore when typing in inputs
      const target = event.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return;
      }

      // Handle Pro Tools-style window shortcuts (Cmd+1-7, Cmd+M, Cmd+T)
      if (handleKeyboardShortcut(event)) {
        return;
      }

      // Transport controls
      if (event.key === ' ') {
        event.preventDefault();
        if ($playbackStore.isPlaying) {
          void playbackActions.pause();
        } else {
          void playbackActions.play();
        }
      }

      // Return/Enter - Stop and return to start
      if (event.key === 'Enter' && !event.ctrlKey && !event.altKey) {
        event.preventDefault();
        void playbackActions.stop();
      }

      // Ctrl shortcuts
      if (event.ctrlKey && !event.altKey && !event.shiftKey) {
        switch (event.key.toLowerCase()) {
          case 'n':
            event.preventDefault();
            void projectActions.newProject();
            break;
          case 'o':
            event.preventDefault();
            void projectActions.openProject();
            break;
          case 's':
            event.preventDefault();
            void projectActions.saveProject();
            break;
          case 'q':
            event.preventDefault();
            window.close();
            break;
          case 'i':
            event.preventDefault();
            // Toggle Import/Pipeline window
            uiActions.toggleWindow('pipeline');
            break;
          case 'm':
            event.preventDefault();
            // Toggle MIDI Monitor
            uiActions.toggleWindow('midi-monitor');
            break;
        }
      }

      // Alt shortcuts for quick window access
      if (event.altKey && !event.ctrlKey && !event.shiftKey) {
        switch (event.key) {
          case '1':
            event.preventDefault();
            uiActions.showWindow('arrangement');
            uiActions.bringToFront('arrangement');
            break;
          case '2':
            event.preventDefault();
            uiActions.showWindow('mixer');
            uiActions.bringToFront('mixer');
            break;
          case '3':
            event.preventDefault();
            uiActions.showWindow('database');
            uiActions.bringToFront('database');
            break;
          case '4':
            event.preventDefault();
            uiActions.showWindow('piano-roll');
            uiActions.bringToFront('piano-roll');
            break;
          case '5':
            event.preventDefault();
            uiActions.showWindow('pipeline');
            uiActions.bringToFront('pipeline');
            break;
          case '6':
            event.preventDefault();
            uiActions.showWindow('midi-io-setup');
            uiActions.bringToFront('midi-io-setup');
            break;
          case '7':
            event.preventDefault();
            uiActions.showWindow('midi-monitor');
            uiActions.bringToFront('midi-monitor');
            break;
          case '8':
            event.preventDefault();
            uiActions.showWindow('preferences');
            uiActions.bringToFront('preferences');
            break;
          case '9':
            event.preventDefault();
            uiActions.showWindow('gear-manager');
            uiActions.bringToFront('gear-manager');
            break;
          case '0':
            event.preventDefault();
            uiActions.showWindow('presets-manager');
            uiActions.bringToFront('presets-manager');
            break;
        }
      }

      // Shift+Alt shortcuts for hiding windows
      if (event.altKey && event.shiftKey && !event.ctrlKey) {
        const numKey = parseInt(event.key);
        if (numKey >= 1 && numKey <= 9) {
          event.preventDefault();
          const windowId = windowIds[numKey - 1];
          uiActions.hideWindow(windowId);
        } else if (event.key === '0') {
          event.preventDefault();
          uiActions.hideWindow('presets-manager');
        }
      }

      // Ctrl+Alt shortcuts for window management
      if (event.ctrlKey && event.altKey && !event.shiftKey) {
        switch (event.key.toLowerCase()) {
          case 'a':
            event.preventDefault();
            // Show all windows
            windowIds.forEach((id) => uiActions.showWindow(id));
            break;
          case 'h':
            event.preventDefault();
            // Hide all popup windows (keep main workspace)
            uiActions.hideWindow('pipeline');
            uiActions.hideWindow('midi-io-setup');
            uiActions.hideWindow('midi-monitor');
            break;
          case 'r':
            event.preventDefault();
            // Reset window layout to defaults (arrangement-centric)
            windowIds.forEach((id) => {
              if (['arrangement', 'mixer', 'database'].includes(id)) {
                uiActions.showWindow(id);
              } else {
                uiActions.hideWindow(id);
              }
            });
            break;
        }
      }

      // F1-F10: Toggle windows (F1-F7 existing, F8-F10 new windows)
      if (event.key >= 'F1' && event.key <= 'F9') {
        event.preventDefault();
        const index = parseInt(event.key.substring(1)) - 1;
        if (windowIds[index]) {
          uiActions.toggleWindow(windowIds[index]);
        }
      }

      // F10: Toggle presets-manager (10th window)
      if (event.key === 'F10') {
        event.preventDefault();
        uiActions.toggleWindow('presets-manager');
      }

      // F11: Toggle sidebar
      if (event.key === 'F11') {
        event.preventDefault();
        uiActions.toggleSidebar();
      }

      // F12: Toggle inspector
      if (event.key === 'F12') {
        event.preventDefault();
        uiActions.toggleInspector();
      }

      // Escape: Close topmost popup window
      if (event.key === 'Escape') {
        for (const popup of popupWindows) {
          if ($uiStore.windows[popup]?.visible) {
            event.preventDefault();
            uiActions.hideWindow(popup);
            break;
          }
        }
      }

      // Tab: Cycle through visible windows (bring next to front)
      if (event.key === 'Tab' && event.ctrlKey && !event.altKey) {
        event.preventDefault();
        const visibleIds = windowIds.filter((id) => $uiStore.windows[id]?.visible);
        if (visibleIds.length > 1) {
          // Find highest z-index window
          let maxZ = -1;
          let topWindowIdx = 0;
          visibleIds.forEach((id, idx) => {
            const z = $uiStore.windows[id]?.z_index ?? 0;
            if (z > maxZ) {
              maxZ = z;
              topWindowIdx = idx;
            }
          });
          // Bring next window to front
          const nextIdx = event.shiftKey ? (topWindowIdx - 1 + visibleIds.length) % visibleIds.length : (topWindowIdx + 1) % visibleIds.length;
          uiActions.bringToFront(visibleIds[nextIdx]);
        }
      }
    };

    document.addEventListener('keydown', handleKeydown);

    // Accessibility: Keyboard vs Mouse detection
    const handleKeyboardDetection = (event: KeyboardEvent) => {
      if (event.key === 'Tab') {
        a11yStore.enableKeyboardMode();
      }
    };

    const handleMouseDetection = () => {
      a11yStore.disableKeyboardMode();
    };

    document.addEventListener('keydown', handleKeyboardDetection);
    document.addEventListener('mousedown', handleMouseDetection);

    // Cleanup on unmount
    return () => {
      document.removeEventListener('keydown', handleKeydown);
      document.removeEventListener('keydown', handleKeyboardDetection);
      document.removeEventListener('mousedown', handleMouseDetection);
      if (destroy) {
        destroy();
      }
    };
  });
</script>

<!-- Accessibility: Body class bindings -->
<svelte:body class:keyboard-navigation={$isKeyboardNav} class:reduced-motion={$reducedMotion} />

<!-- Skip Links for keyboard navigation -->
<SkipLinks />

<MenuBar />

<div id="main-content" class="workspace frogskin-radial" tabindex="-1">
  <div class="grid-container">
    {#if $uiStore.windows.database?.visible}
      <div class="window-wrapper" style="grid-area: database;">
        <DatabaseWindow />
      </div>
    {/if}
    {#if $uiStore.windows.arrangement?.visible}
      <div class="window-wrapper" style="grid-area: arrangement;">
        <ArrangementWindow />
      </div>
    {/if}
    {#if $uiStore.windows.mixer?.visible}
      <div class="window-wrapper" style="grid-area: mixer;">
        <MixerWindow />
      </div>
    {/if}
  </div>

  <!-- Pipeline Window as Modal/Popup -->
  {#if $uiStore.windows.pipeline.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('pipeline')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('pipeline')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content" role="document">
        <div class="modal-header">
          <h2>Import Files</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('pipeline')} aria-label="Close">×</button>
        </div>
        <PipelineWindow />
      </div>
    </div>
  {/if}

  <!-- MIDI I/O Setup Window as Modal/Popup (F6) -->
  {#if $uiStore.windows['midi-io-setup']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('midi-io-setup')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('midi-io-setup')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <div class="modal-header">
          <h2>MIDI I/O Setup</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('midi-io-setup')} aria-label="Close">×</button>
        </div>
        <MidiIOSetupWindow />
      </div>
    </div>
  {/if}

  <!-- MIDI Monitor Window as Modal/Popup (F7) -->
  {#if $uiStore.windows['midi-monitor']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('midi-monitor')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('midi-monitor')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <div class="modal-header">
          <h2>MIDI Monitor</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('midi-monitor')} aria-label="Close">×</button>
        </div>
        <MidiMonitorWindow />
      </div>
    </div>
  {/if}

  <!-- Preferences Window as Modal/Popup (F8) -->
  {#if $uiStore.windows['preferences']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('preferences')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('preferences')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <PreferencesWindow windowId="preferences" />
      </div>
    </div>
  {/if}

  <!-- Gear Manager Window as Modal/Popup (F9) -->
  {#if $uiStore.windows['gear-manager']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('gear-manager')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('gear-manager')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <GearManagerWindow windowId="gear-manager" />
      </div>
    </div>
  {/if}

  <!-- Presets Manager Window as Modal/Popup (F10) -->
  {#if $uiStore.windows['presets-manager']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('presets-manager')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('presets-manager')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <PresetsManagerWindow windowId="presets-manager" />
      </div>
    </div>
  {/if}

  <!-- Score Window as Modal/Popup -->
  {#if $uiStore.windows['score']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('score')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('score')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <div class="modal-header">
          <h2>Score View</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('score')} aria-label="Close">×</button>
        </div>
        <ScoreWindow />
      </div>
    </div>
  {/if}

  <!-- Script Editor Window as Modal/Popup -->
  {#if $uiStore.windows['script-editor']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('script-editor')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('script-editor')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <div class="modal-header">
          <h2>Script Editor</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('script-editor')} aria-label="Close">×</button>
        </div>
        <ScriptEditorWindow />
      </div>
    </div>
  {/if}

  <!-- MIDI Learn Window as Modal/Popup -->
  {#if $uiStore.windows['midi-learn']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('midi-learn')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('midi-learn')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content" role="document">
        <div class="modal-header">
          <h2>MIDI Learn</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('midi-learn')} aria-label="Close">×</button>
        </div>
        <MidiLearnWindow />
      </div>
    </div>
  {/if}

  <!-- Link Sync Window as Modal/Popup -->
  {#if $uiStore.windows['link-sync']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('link-sync')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('link-sync')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content" role="document">
        <div class="modal-header">
          <h2>Ableton Link</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('link-sync')} aria-label="Close">×</button>
        </div>
        <LinkSyncWindow />
      </div>
    </div>
  {/if}

  <!-- Piano Roll Window as Modal/Popup (Alt+4) -->
  {#if $uiStore.windows['piano-roll']?.visible}
    <div class="modal-overlay" on:click={(e) => e.target === e.currentTarget && uiActions.hideWindow('piano-roll')} on:keydown={(e) => e.key === 'Escape' && uiActions.hideWindow('piano-roll')} role="dialog" aria-modal="true" tabindex="-1">
      <div class="modal-content modal-content-wide" role="document">
        <div class="modal-header">
          <h2>Piano Roll</h2>
          <button class="modal-close" on:click={() => uiActions.hideWindow('piano-roll')} aria-label="Close">×</button>
        </div>
        <PianoRollWindow />
      </div>
    </div>
  {/if}
</div>

<WindowDockBar />
<StatusBar />
<KeyboardShortcuts bind:visible={showShortcuts} />

<!-- Global Error and Loading Components -->
<ErrorToast position="bottom-right" />
<LoadingOverlay />

<!-- MIDI Learn Overlay - shows when in learn mode -->
{#if $isLearning}
  <MidiLearnOverlay visible={$isLearning} on:close={() => {}} />
{/if}

<style>
  .workspace {
    position: relative;
    height: calc(100vh - 4rem); /* Adjust for menu and status bar */
    overflow: hidden;
    display: flex;
    /* Frogskin camo theme - Ruby, Emerald, Sapphire radial gradient */
    background: var(--frogskin-gradient-radial);
    background-attachment: fixed;
  }

  .grid-container {
    display: grid;
    grid-template-columns: 280px 1fr;
    grid-template-rows: 1fr auto;
    grid-template-areas:
      'database arrangement'
      'database mixer';
    gap: 8px;
    height: 100%;
    width: 100%;
    padding: 8px;
  }

  .window-wrapper {
    min-height: 0;
    min-width: 0;
    overflow: auto;
    border: 2px solid var(--emerald-500);
    border-radius: 8px;
    background-color: rgba(13, 17, 23, 0.65);
    backdrop-filter: blur(12px);
    box-shadow:
      0 0 20px var(--ruby-glow),
      0 0 40px var(--emerald-glow),
      0 0 8px rgba(5, 150, 105, 0.3),
      inset 0 0 30px rgba(0, 0, 0, 0.4);
    transition: all 0.3s ease;
  }

  .window-wrapper:hover {
    border-color: var(--sapphire-500);
    background-color: rgba(13, 17, 23, 0.7);
    box-shadow:
      0 0 25px var(--ruby-glow),
      0 0 50px var(--emerald-glow),
      0 0 70px var(--sapphire-glow),
      0 0 12px rgba(29, 78, 216, 0.4),
      inset 0 0 30px rgba(0, 0, 0, 0.4);
  }

  /* Modal/Popup styles for Pipeline/Import */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background-color: rgba(13, 17, 23, 0.75);
    border: 2px solid var(--sapphire-500);
    border-radius: 10px;
    width: 80%;
    max-width: 900px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.6),
      0 0 40px var(--ruby-glow),
      0 0 60px var(--emerald-glow),
      0 0 80px var(--sapphire-glow),
      0 0 15px rgba(29, 78, 216, 0.5);
    backdrop-filter: blur(16px);
  }

  .modal-content-wide {
    max-width: 1400px;
    width: 95%;
    max-height: 85vh;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--window-border, #444);
    background-color: var(--bg-secondary, #1e1e1e);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 500;
    color: var(--text-primary, #fff);
  }

  .modal-close {
    background: none;
    border: none;
    color: var(--text-secondary, #aaa);
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
    transition: color 0.2s;
  }

  .modal-close:hover {
    color: var(--text-primary, #fff);
  }

  /* Responsive adjustments for smaller screens */
  @media (max-width: 1280px) {
    .grid-container {
      grid-template-columns: 1fr;
      grid-template-rows: auto 1fr auto;
      grid-template-areas:
        'database'
        'arrangement'
        'mixer';
      max-width: 100%;
      max-height: 100%;
    }

    .modal-content {
      width: 95%;
      max-height: 90vh;
    }
  }
</style>
