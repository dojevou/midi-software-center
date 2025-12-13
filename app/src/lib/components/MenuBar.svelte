<script lang="ts">
  import { onMount } from 'svelte';
  import { safeInvoke } from '$lib/utils/tauri';
  import { Commands } from '$lib/api/commands';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { open as openUrl } from '@tauri-apps/plugin-shell';
  import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
  import { uiActions, uiStore as _uiStore } from '$lib/stores/uiStore';
  import { playbackActions } from '$lib/stores/playbackStore';
  import { projectActions, projectStore } from '$lib/stores/projectStore';
  import { pipelineActions as _pipelineActions } from '$lib/stores/pipelineStore';
  import { databaseActions as _databaseActions } from '$lib/stores/databaseStore';
  import type { WindowId, Track } from '$lib/types';
  import type { ProjectState } from '$lib/stores/projectStore';

  // ============================================================================
  // PROJECT FILE FORMAT
  // ============================================================================

  interface ProjectFile {
    version: string;
    name: string;
    createdAt: string;
    updatedAt: string;
    tracks: Track[];
    settings?: {
      bpm?: number;
      timeSignature?: { numerator: number; denominator: number };
    };
  }

  const PROJECT_FILE_VERSION = '1.0.0';

  /**
   * Serialize project state to file format
   */
  function serializeProject(state: ProjectState): ProjectFile {
    return {
      version: PROJECT_FILE_VERSION,
      name: state.projectName,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      tracks: state.tracks,
      settings: {},
    };
  }

  /**
   * Extract filename without extension from path
   */
  function getProjectNameFromPath(filePath: string): string {
    const parts = filePath.replace(/\\/g, '/').split('/');
    const filename = parts[parts.length - 1];
    return filename.replace(/\.(msc|json)$/i, '');
  }

  // Track the current project file path for Save functionality
  let currentProjectPath: string | null = null;

  // ============================================================================
  // FILE MENU ACTIONS
  // ============================================================================

  async function newProject() {
    try {
      // Check for unsaved changes
      const state = $projectStore;
      if (state.hasUnsavedChanges) {
        const confirmed = confirm('You have unsaved changes. Create new project anyway?');
        if (!confirmed) {
          return;
        }
      }

      // Clear all tracks and reset state
      await projectActions.clearAllTracks();
      projectActions.setProjectName('Untitled Project');
      projectActions.markSaved();
      currentProjectPath = null;

      // Reset playback
      await playbackActions.stop();

      console.log('New project created successfully');
    } catch (error) {
      console.error('Failed to create new project:', error);
      alert(`Failed to create new project: ${error}`);
    }
  }

  async function openProject() {
    try {
      const filePath = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'MIDI Software Center Project',
            extensions: ['msc', 'json'],
          },
        ],
      });

      if (!filePath) {
        return;
      }

      // Check for unsaved changes
      const state = $projectStore;
      if (state.hasUnsavedChanges) {
        const confirmed = confirm('You have unsaved changes. Open another project anyway?');
        if (!confirmed) {
          return;
        }
      }

      // Read and parse the project file
      const fileContent = await readTextFile(filePath);
      const projectData = JSON.parse(fileContent) as ProjectFile;

      // Validate project file format
      if (!projectData.version || !projectData.tracks) {
        throw new Error('Invalid project file format');
      }

      // Clear current project and load new data
      await projectActions.clearAllTracks();

      // Load tracks from the project file
      // Note: This loads the track definitions - actual MIDI data would need
      // to be loaded from referenced files or stored in the project
      projectStore.update((s) => ({
        ...s,
        tracks: projectData.tracks,
        projectName: projectData.name || getProjectNameFromPath(filePath),
        hasUnsavedChanges: false,
      }));

      currentProjectPath = filePath;
      console.log('Project loaded successfully:', projectData.name);
    } catch (error) {
      console.error('Failed to open project:', error);
      alert(`Failed to open project: ${error}`);
    }
  }

  async function saveProject() {
    try {
      // If we have an existing path, save directly to it
      if (currentProjectPath) {
        const state = $projectStore;
        const projectData = serializeProject(state);
        await writeTextFile(currentProjectPath, JSON.stringify(projectData, null, 2));
        projectActions.markSaved();
        console.log('Project saved successfully:', currentProjectPath);
      } else {
        // No existing path, use Save As
        await saveProjectAs();
      }
    } catch (error) {
      console.error('Failed to save project:', error);
      alert(`Failed to save project: ${error}`);
    }
  }

  async function saveProjectAs() {
    try {
      const filePath = await save({
        filters: [
          {
            name: 'MIDI Software Center Project',
            extensions: ['msc'],
          },
        ],
        defaultPath: `${$projectStore.projectName}.msc`,
      });

      if (!filePath) {
        return;
      }

      // Serialize and save project
      const state = $projectStore;
      const projectData = serializeProject(state);
      await writeTextFile(filePath, JSON.stringify(projectData, null, 2));

      // Update project name from filename and mark as saved
      const projectName = getProjectNameFromPath(filePath);
      projectActions.setProjectName(projectName);
      projectActions.markSaved();
      currentProjectPath = filePath;

      console.log('Project saved successfully:', filePath);
    } catch (error) {
      console.error('Failed to save project as:', error);
      alert(`Failed to save project: ${error}`);
    }
  }

  async function exportMidi() {
    try {
      const state = $projectStore;

      if (state.tracks.length === 0) {
        alert('No tracks to export. Load some MIDI files first.');
        return;
      }

      const filePath = await save({
        filters: [
          {
            name: 'MIDI File',
            extensions: ['mid', 'midi'],
          },
        ],
        defaultPath: `${$projectStore.projectName}.mid`,
      });

      if (!filePath) {
        return;
      }

      // Use the export API - uses Commands registry for correct command name
      await safeInvoke(Commands.EXPORT_PROJECT_MIDI, { output_path: filePath });

      console.log('MIDI exported successfully to:', filePath);
      alert(`MIDI file exported successfully to:\n${filePath}`);
    } catch (error) {
      console.error('Failed to export MIDI:', error);
      alert(`Failed to export MIDI: ${error}`);
    }
  }

  // ============================================================================
  // EDIT MENU ACTIONS
  // ============================================================================

  let showPreferencesDialog = false;

  function openPreferences() {
    showPreferencesDialog = true;
  }

  function closePreferences() {
    showPreferencesDialog = false;
  }

  // ============================================================================
  // VIEW MENU ACTIONS
  // ============================================================================

  const ZOOM_STEP = 0.1;
  const ZOOM_MIN = 0.5;
  const ZOOM_MAX = 2.0;
  const ZOOM_DEFAULT = 1.0;

  let currentZoom = ZOOM_DEFAULT;

  function zoomIn() {
    currentZoom = Math.min(currentZoom + ZOOM_STEP, ZOOM_MAX);
    applyZoom();
  }

  function zoomOut() {
    currentZoom = Math.max(currentZoom - ZOOM_STEP, ZOOM_MIN);
    applyZoom();
  }

  function resetZoom() {
    currentZoom = ZOOM_DEFAULT;
    applyZoom();
  }

  function applyZoom() {
    // Apply zoom to document root
    document.documentElement.style.setProperty('--app-zoom', currentZoom.toString());
    console.log(`Zoom set to ${(currentZoom * 100).toFixed(0)}%`);
  }

  // ============================================================================
  // HELP MENU ACTIONS
  // ============================================================================

  async function openDocumentation() {
    try {
      // Open documentation URL in default browser
      const docsUrl = 'https://github.com/yourusername/midi-software-center/wiki';
      await openUrl(docsUrl);
    } catch (error) {
      console.error('Failed to open documentation:', error);
      alert(`Failed to open documentation: ${error}`);
    }
  }

  let showKeyboardShortcuts = false;

  function openKeyboardShortcuts() {
    showKeyboardShortcuts = true;
  }

  function closeKeyboardShortcuts() {
    showKeyboardShortcuts = false;
  }

  let showAboutDialog = false;

  function openAbout() {
    showAboutDialog = true;
  }

  function closeAbout() {
    showAboutDialog = false;
  }

  // ============================================================================
  // QUIT APPLICATION
  // ============================================================================

  async function quitApplication() {
    try {
      // Check for unsaved changes
      if ($projectStore.hasUnsavedChanges) {
        const confirmed = confirm('You have unsaved changes. Quit anyway?');
        if (!confirmed) {
          return;
        }
      }

      await safeInvoke(Commands.SHUTDOWN_APPLICATION);
    } catch (error) {
      console.error('Failed to shutdown:', error);
      // Fallback to window close
      window.close();
    }
  }

  // ============================================================================
  // MENU CONFIGURATION
  // ============================================================================

  type MenuKey = 'file' | 'edit' | 'view' | 'tools' | 'transport' | 'help';

  type Separator = { separator: true };

  type MenuItem = {
    label: string;
    shortcut?: string;
    action: () => void;
    disabled?: boolean;
  };

  type MenuEntry = MenuItem | Separator;

  let openMenu: MenuKey | null = null;

  const menuItems: Record<MenuKey, MenuEntry[]> = {
    file: [
      { label: 'New Project', shortcut: 'Ctrl+N', action: newProject },
      { label: 'Open Project', shortcut: 'Ctrl+O', action: openProject },
      { label: 'Save Project', shortcut: 'Ctrl+S', action: saveProject },
      { label: 'Save As...', shortcut: 'Ctrl+Shift+S', action: saveProjectAs },
      { separator: true },
      {
        label: 'Import Files...',
        shortcut: 'Ctrl+I',
        action: () => uiActions.showWindow('pipeline'),
      },
      { label: 'Export MIDI', shortcut: 'Ctrl+E', action: exportMidi },
      { separator: true },
      { label: 'Quit', shortcut: 'Ctrl+Q', action: quitApplication },
    ],
    edit: [
      { label: 'Undo', shortcut: 'Ctrl+Z', action: () => console.log('Undo'), disabled: true },
      { label: 'Redo', shortcut: 'Ctrl+Y', action: () => console.log('Redo'), disabled: true },
      { separator: true },
      { label: 'Cut', shortcut: 'Ctrl+X', action: () => console.log('Cut'), disabled: true },
      { label: 'Copy', shortcut: 'Ctrl+C', action: () => console.log('Copy'), disabled: true },
      { label: 'Paste', shortcut: 'Ctrl+V', action: () => console.log('Paste'), disabled: true },
      { label: 'Delete', shortcut: 'Del', action: () => console.log('Delete'), disabled: true },
      { separator: true },
      {
        label: 'Select All',
        shortcut: 'Ctrl+A',
        action: () => console.log('Select All'),
        disabled: true,
      },
      { separator: true },
      { label: 'Preferences', shortcut: 'Ctrl+,', action: openPreferences },
    ],
    view: [
      {
        label: 'Toggle DAW Window',
        shortcut: 'F1',
        action: () => uiActions.toggleWindow('daw' as WindowId),
      },
      {
        label: 'Toggle Mixer Window',
        shortcut: 'F2',
        action: () => uiActions.toggleWindow('mixer' as WindowId),
      },
      {
        label: 'Toggle Database Window',
        shortcut: 'F3',
        action: () => uiActions.toggleWindow('database' as WindowId),
      },
      {
        label: 'Toggle Pipeline Window',
        shortcut: 'F4',
        action: () => uiActions.toggleWindow('pipeline' as WindowId),
      },
      { separator: true },
      {
        label: 'Score View',
        shortcut: 'Ctrl+Shift+N',
        action: () => uiActions.openWindow('score' as WindowId),
      },
      { separator: true },
      { label: 'Zoom In', shortcut: 'Ctrl++', action: zoomIn },
      { label: 'Zoom Out', shortcut: 'Ctrl+-', action: zoomOut },
      { label: 'Reset Zoom', shortcut: 'Ctrl+0', action: resetZoom },
    ],
    tools: [
      {
        label: 'Script Editor',
        shortcut: 'Ctrl+Shift+S',
        action: () => uiActions.openWindow('script-editor' as WindowId),
      },
      {
        label: 'MIDI Learn Mode',
        shortcut: 'Ctrl+Shift+M',
        action: () => uiActions.openWindow('midi-learn' as WindowId),
      },
      { separator: true },
      {
        label: 'Ableton Link',
        shortcut: '',
        action: () => uiActions.openWindow('link-sync' as WindowId),
      },
    ],
    transport: [
      { label: 'Play', shortcut: 'Space', action: () => playbackActions.play() },
      { label: 'Pause', shortcut: 'Space', action: () => playbackActions.pause() },
      { label: 'Stop', shortcut: 'Ctrl+Space', action: () => playbackActions.stop() },
      { separator: true },
      { label: 'Record', shortcut: 'Ctrl+R', action: () => playbackActions.record() },
      { separator: true },
      { label: 'Loop Playback', shortcut: 'Ctrl+L', action: () => playbackActions.toggleLoop() },
      { label: 'Metronome', shortcut: 'Ctrl+M', action: () => playbackActions.toggleMetronome() },
    ],
    help: [
      { label: 'Documentation', shortcut: '', action: openDocumentation },
      { label: 'Keyboard Shortcuts', shortcut: 'Ctrl+Shift+H', action: openKeyboardShortcuts },
      { separator: true },
      { label: 'About MIDI Software Center', shortcut: '', action: openAbout },
    ],
  };

  function isSeparator(entry: MenuEntry): entry is Separator {
    return 'separator' in entry;
  }

  function getMenuItem(entry: MenuEntry): MenuItem {
    return entry as MenuItem;
  }

  function openDropdown(menu: string) {
    const key = menu as MenuKey;
    openMenu = openMenu === key ? null : key;
  }

  function closeDropdown() {
    openMenu = null;
  }

  function handleMenuAction(action: () => void) {
    action();
    closeDropdown();
  }

  // Click outside to close
  onMount(() => {
    function handleClickOutside(event: MouseEvent) {
      const target = event.target as HTMLElement;
      if (!target.closest('.menu-bar') && !target.closest('.dropdown-menu')) {
        closeDropdown();
      }
    }
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="menu-bar dark:bg-menu dark:text-app-text flex space-x-0">
  {#each ['file', 'edit', 'view', 'tools', 'transport', 'help'] as menuKey (menuKey)}
    <div class="menu-item relative">
      <button
        class="px-3 py-2 dark:hover:bg-primary/20 focus:outline-none transition-colors"
        class:active={openMenu === menuKey}
        on:click={() => openDropdown(menuKey)}
      >
        {menuKey.charAt(0).toUpperCase() + menuKey.slice(1)}
      </button>
      {#if openMenu === menuKey}
        <div
          class="dropdown-menu dark:bg-menu dark:border-window-border absolute top-full left-0 mt-1 w-56 rounded shadow-lg z-50"
        >
          {#each menuItems[menuKey] as item, idx (isSeparator(item) ? `sep-${idx}` : getMenuItem(item).label)}
            {#if isSeparator(item)}
              <div class="border-t dark:border-window-border my-1"></div>
            {:else}
              {@const menuItem = getMenuItem(item)}
              <button
                class="w-full text-left px-4 py-2 dark:hover:bg-primary/20 disabled:dark:text-gray-500 disabled:cursor-not-allowed transition-colors"
                class:disabled={menuItem.disabled ?? false}
                disabled={menuItem.disabled ?? false}
                on:click={() => handleMenuAction(menuItem.action)}
              >
                <div class="flex justify-between items-center">
                  <span class="text-sm">{menuItem.label}</span>
                  {#if menuItem.shortcut}
                    <span class="text-xs opacity-60 ml-2">{menuItem.shortcut}</span>
                  {/if}
                </div>
              </button>
            {/if}
          {/each}
        </div>
      {/if}
    </div>
  {/each}
</div>

<!-- Preferences Dialog -->
{#if showPreferencesDialog}
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]"
    on:click={closePreferences}
    on:keydown={(e) => e.key === 'Escape' && closePreferences()}
    role="presentation"
  >
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
      class="dark:bg-window p-6 rounded-lg shadow-xl max-w-2xl w-full mx-4"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      aria-labelledby="preferences-title"
      tabindex="-1"
    >
      <h2 id="preferences-title" class="text-xl font-bold mb-4">Preferences</h2>

      <div class="space-y-4">
        <!-- Theme Selection -->
        <div>
          <label for="theme-select" class="block text-sm font-medium mb-2">Theme</label>
          <select
            id="theme-select"
            class="dark:bg-menu dark:border-window-border border rounded px-3 py-2 w-full focus:outline-none focus:ring-2 focus:ring-primary"
          >
            <option value="dark" selected>Dark</option>
            <option value="light">Light</option>
          </select>
        </div>

        <!-- Audio Settings -->
        <div>
          <label for="buffer-size" class="block text-sm font-medium mb-2">Audio Buffer Size</label>
          <select
            id="buffer-size"
            class="dark:bg-menu dark:border-window-border border rounded px-3 py-2 w-full focus:outline-none focus:ring-2 focus:ring-primary"
          >
            <option value="256">256 samples</option>
            <option value="512" selected>512 samples</option>
            <option value="1024">1024 samples</option>
            <option value="2048">2048 samples</option>
          </select>
        </div>

        <!-- Database Settings -->
        <div>
          <label for="db-connection" class="block text-sm font-medium mb-2"
            >Database Connection</label
          >
          <input
            id="db-connection"
            type="text"
            class="dark:bg-menu dark:border-window-border border rounded px-3 py-2 w-full focus:outline-none focus:ring-2 focus:ring-primary"
            value="postgresql://localhost:5432/midi_library"
            readonly
            aria-readonly="true"
          />
        </div>
      </div>

      <div class="flex justify-end gap-2 mt-6">
        <button
          type="button"
          class="px-4 py-2 dark:bg-menu dark:hover:bg-primary/20 rounded transition-colors focus:outline-none focus:ring-2 focus:ring-primary"
          on:click={closePreferences}
        >
          Cancel
        </button>
        <button
          type="button"
          class="px-4 py-2 bg-primary dark:hover:bg-primary/80 rounded transition-colors focus:outline-none focus:ring-2 focus:ring-primary"
          on:click={closePreferences}
        >
          Save
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Keyboard Shortcuts Dialog -->
{#if showKeyboardShortcuts}
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]"
    on:click={closeKeyboardShortcuts}
    on:keydown={(e) => e.key === 'Escape' && closeKeyboardShortcuts()}
    role="presentation"
  >
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
      class="dark:bg-window p-6 rounded-lg shadow-xl max-w-3xl w-full mx-4 max-h-[80vh] overflow-y-auto"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      aria-labelledby="shortcuts-title"
      tabindex="-1"
    >
      <h2 id="shortcuts-title" class="text-xl font-bold mb-4">Keyboard Shortcuts</h2>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- File -->
        <div>
          <h3 class="font-semibold mb-2">File</h3>
          <div class="space-y-1 text-sm">
            <div class="flex justify-between">
              <span>New Project</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+N</kbd>
            </div>
            <div class="flex justify-between">
              <span>Open Project</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+O</kbd>
            </div>
            <div class="flex justify-between">
              <span>Save Project</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+S</kbd>
            </div>
            <div class="flex justify-between">
              <span>Save As</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+Shift+S</kbd>
            </div>
            <div class="flex justify-between">
              <span>Import MIDI</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+I</kbd>
            </div>
            <div class="flex justify-between">
              <span>Export MIDI</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+E</kbd>
            </div>
            <div class="flex justify-between">
              <span>Quit</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+Q</kbd>
            </div>
          </div>
        </div>

        <!-- Edit -->
        <div>
          <h3 class="font-semibold mb-2">Edit</h3>
          <div class="space-y-1 text-sm">
            <div class="flex justify-between">
              <span>Undo</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+Z</kbd>
            </div>
            <div class="flex justify-between">
              <span>Redo</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+Y</kbd>
            </div>
            <div class="flex justify-between">
              <span>Cut</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+X</kbd>
            </div>
            <div class="flex justify-between">
              <span>Copy</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+C</kbd>
            </div>
            <div class="flex justify-between">
              <span>Paste</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+V</kbd>
            </div>
            <div class="flex justify-between">
              <span>Preferences</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+,</kbd>
            </div>
          </div>
        </div>

        <!-- View -->
        <div>
          <h3 class="font-semibold mb-2">View</h3>
          <div class="space-y-1 text-sm">
            <div class="flex justify-between">
              <span>Toggle DAW</span><kbd class="dark:bg-menu px-2 py-1 rounded">F1</kbd>
            </div>
            <div class="flex justify-between">
              <span>Toggle Mixer</span><kbd class="dark:bg-menu px-2 py-1 rounded">F2</kbd>
            </div>
            <div class="flex justify-between">
              <span>Toggle Database</span><kbd class="dark:bg-menu px-2 py-1 rounded">F3</kbd>
            </div>
            <div class="flex justify-between">
              <span>Toggle Pipeline</span><kbd class="dark:bg-menu px-2 py-1 rounded">F4</kbd>
            </div>
            <div class="flex justify-between">
              <span>Zoom In</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl++</kbd>
            </div>
            <div class="flex justify-between">
              <span>Zoom Out</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+-</kbd>
            </div>
            <div class="flex justify-between">
              <span>Reset Zoom</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+0</kbd>
            </div>
          </div>
        </div>

        <!-- Transport -->
        <div>
          <h3 class="font-semibold mb-2">Transport</h3>
          <div class="space-y-1 text-sm">
            <div class="flex justify-between">
              <span>Play/Pause</span><kbd class="dark:bg-menu px-2 py-1 rounded">Space</kbd>
            </div>
            <div class="flex justify-between">
              <span>Stop</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+Space</kbd>
            </div>
            <div class="flex justify-between">
              <span>Record</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+R</kbd>
            </div>
            <div class="flex justify-between">
              <span>Loop</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+L</kbd>
            </div>
            <div class="flex justify-between">
              <span>Metronome</span><kbd class="dark:bg-menu px-2 py-1 rounded">Ctrl+M</kbd>
            </div>
          </div>
        </div>
      </div>

      <div class="flex justify-end mt-6">
        <button
          type="button"
          class="px-4 py-2 bg-primary dark:hover:bg-primary/80 rounded transition-colors focus:outline-none focus:ring-2 focus:ring-primary"
          on:click={closeKeyboardShortcuts}
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- About Dialog -->
{#if showAboutDialog}
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]"
    on:click={closeAbout}
    on:keydown={(e) => e.key === 'Escape' && closeAbout()}
    role="presentation"
  >
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
      class="dark:bg-window p-6 rounded-lg shadow-xl max-w-md w-full mx-4"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="dialog"
      aria-modal="true"
      aria-labelledby="about-title"
      tabindex="-1"
    >
      <h2 id="about-title" class="text-2xl font-bold mb-2">MIDI Software Center</h2>
      <p class="text-sm opacity-80 mb-4">Version 1.0.0</p>

      <div class="space-y-3 text-sm">
        <p>
          A powerful MIDI library management and DAW application built with Tauri, Rust, and Svelte.
        </p>

        <div class="dark:bg-menu p-3 rounded">
          <h3 class="font-semibold mb-2">Features</h3>
          <ul class="list-disc list-inside space-y-1 opacity-80">
            <li>Batch MIDI file import and analysis</li>
            <li>Real-time sequencer and playback</li>
            <li>PostgreSQL database with 3M+ file capacity</li>
            <li>BPM and key detection</li>
            <li>Advanced search and filtering</li>
          </ul>
        </div>

        <div class="dark:bg-menu p-3 rounded">
          <h3 class="font-semibold mb-2">Technology Stack</h3>
          <ul class="list-disc list-inside space-y-1 opacity-80">
            <li>Rust 1.70+ (Backend)</li>
            <li>Svelte 4.2 (Frontend)</li>
            <li>Tauri 2.0 (Desktop Framework)</li>
            <li>PostgreSQL 16 (Database)</li>
          </ul>
        </div>

        <p class="text-xs opacity-60 pt-2">
          Copyright 2025 MIDI Software Center<br />
          Licensed under MIT License
        </p>
      </div>

      <div class="flex justify-end mt-6">
        <button
          type="button"
          class="px-4 py-2 bg-primary dark:hover:bg-primary/80 rounded transition-colors focus:outline-none focus:ring-2 focus:ring-primary"
          on:click={closeAbout}
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .menu-bar {
    border-bottom: 1px solid var(--window-border);
    user-select: none;
  }

  .menu-item {
    position: relative;
  }

  .active {
    background-color: var(--primary-color);
    color: white;
  }

  .dropdown-menu {
    border: 1px solid var(--window-border);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  kbd {
    font-family: monospace;
    font-size: 0.85em;
  }
</style>
