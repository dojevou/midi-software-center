<script lang="ts">
  /**
   * MenuBar - Application menu bar
   *
   * Task-O-Matic: Main menu with keyboard shortcuts.
   */

  import { playbackActions } from '$lib/stores/playbackStore';
  import { projectActions } from '$lib/stores/projectStore';
  import { uiActions } from '$lib/stores/uiStore';
  import { databaseActions } from '$lib/stores/databaseStore';

  // Menu state
  let activeMenu: string | null = null;

  // Menu actions
  const menuActions = {
    // File menu
    newProject: async () => {
      await projectActions.newProject('New Project');
      closeMenus();
    },
    openProject: async () => {
      // TODO: Open file dialog
      console.log('Open project');
      closeMenus();
    },
    saveProject: async () => {
      await projectActions.saveProject();
      closeMenus();
    },
    saveProjectAs: async () => {
      // TODO: Save as dialog
      console.log('Save project as');
      closeMenus();
    },
    importMIDI: () => {
      // TODO: Import dialog
      console.log('Import MIDI');
      closeMenus();
    },
    exportMIDI: () => {
      // TODO: Export dialog
      console.log('Export MIDI');
      closeMenus();
    },

    // Edit menu
    undo: () => {
      console.log('Undo');
      closeMenus();
    },
    redo: () => {
      console.log('Redo');
      closeMenus();
    },
    cut: () => {
      console.log('Cut');
      closeMenus();
    },
    copy: () => {
      console.log('Copy');
      closeMenus();
    },
    paste: () => {
      console.log('Paste');
      closeMenus();
    },
    delete: () => {
      console.log('Delete');
      closeMenus();
    },

    // View menu
    toggleDAW: () => {
      uiActions.toggleWindow('daw');
      closeMenus();
    },
    toggleMixer: () => {
      uiActions.toggleWindow('mixer');
      closeMenus();
    },
    toggleDatabase: () => {
      uiActions.toggleWindow('database');
      closeMenus();
    },
    togglePipeline: () => {
      uiActions.toggleWindow('pipeline');
      closeMenus();
    },
    toggleGridSnapping: () => {
      uiActions.toggleGridSnapping();
      closeMenus();
    },

    // Track menu
    addTrack: async () => {
      await projectActions.addTrack();
      closeMenus();
    },
    deleteTrack: async () => {
      // TODO: Get selected track
      console.log('Delete track');
      closeMenus();
    },
    duplicateTrack: () => {
      console.log('Duplicate track');
      closeMenus();
    },

    // MIDI menu
    playPause: async () => {
      await playbackActions.togglePlayPause();
      closeMenus();
    },
    stop: async () => {
      await playbackActions.stop();
      closeMenus();
    },
    record: async () => {
      await playbackActions.record();
      closeMenus();
    },
    toggleMetronome: () => {
      // TODO: Get current state and toggle
      console.log('Toggle metronome');
      closeMenus();
    }
  };

  function toggleMenu(menuName: string) {
    activeMenu = activeMenu === menuName ? null : menuName;
  }

  function closeMenus() {
    activeMenu = null;
  }

  // Keyboard shortcuts
  function handleKeydown(e: KeyboardEvent) {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    const modKey = isMac ? e.metaKey : e.ctrlKey;

    if (modKey) {
      switch (e.key.toLowerCase()) {
        case 'n':
          e.preventDefault();
          menuActions.newProject();
          break;
        case 'o':
          e.preventDefault();
          menuActions.openProject();
          break;
        case 's':
          e.preventDefault();
          if (e.shiftKey) {
            menuActions.saveProjectAs();
          } else {
            menuActions.saveProject();
          }
          break;
        case 'i':
          e.preventDefault();
          menuActions.importMIDI();
          break;
        case 't':
          e.preventDefault();
          menuActions.addTrack();
          break;
        case 'z':
          e.preventDefault();
          if (e.shiftKey) {
            menuActions.redo();
          } else {
            menuActions.undo();
          }
          break;
        case 'x':
          e.preventDefault();
          menuActions.cut();
          break;
        case 'c':
          e.preventDefault();
          menuActions.copy();
          break;
        case 'v':
          e.preventDefault();
          menuActions.paste();
          break;
      }
    } else if (e.key === ' ') {
      e.preventDefault();
      menuActions.playPause();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} on:click={closeMenus} />

<div class="menu-bar">
  <div class="menu-items">
    <!-- File Menu -->
    <div class="menu-item">
      <button
        class="menu-button"
        class:active={activeMenu === 'file'}
        on:click|stopPropagation={() => toggleMenu('file')}
      >
        File
      </button>
      {#if activeMenu === 'file'}
        <div class="menu-dropdown" on:click|stopPropagation>
          <button class="menu-option" on:click={menuActions.newProject}>
            New Project <span class="shortcut">Ctrl+N</span>
          </button>
          <button class="menu-option" on:click={menuActions.openProject}>
            Open Project <span class="shortcut">Ctrl+O</span>
          </button>
          <div class="menu-separator" />
          <button class="menu-option" on:click={menuActions.saveProject}>
            Save <span class="shortcut">Ctrl+S</span>
          </button>
          <button class="menu-option" on:click={menuActions.saveProjectAs}>
            Save As... <span class="shortcut">Ctrl+Shift+S</span>
          </button>
          <div class="menu-separator" />
          <button class="menu-option" on:click={menuActions.importMIDI}>
            Import MIDI <span class="shortcut">Ctrl+I</span>
          </button>
          <button class="menu-option" on:click={menuActions.exportMIDI}>
            Export MIDI
          </button>
        </div>
      {/if}
    </div>

    <!-- Edit Menu -->
    <div class="menu-item">
      <button
        class="menu-button"
        class:active={activeMenu === 'edit'}
        on:click|stopPropagation={() => toggleMenu('edit')}
      >
        Edit
      </button>
      {#if activeMenu === 'edit'}
        <div class="menu-dropdown" on:click|stopPropagation>
          <button class="menu-option" on:click={menuActions.undo}>
            Undo <span class="shortcut">Ctrl+Z</span>
          </button>
          <button class="menu-option" on:click={menuActions.redo}>
            Redo <span class="shortcut">Ctrl+Shift+Z</span>
          </button>
          <div class="menu-separator" />
          <button class="menu-option" on:click={menuActions.cut}>
            Cut <span class="shortcut">Ctrl+X</span>
          </button>
          <button class="menu-option" on:click={menuActions.copy}>
            Copy <span class="shortcut">Ctrl+C</span>
          </button>
          <button class="menu-option" on:click={menuActions.paste}>
            Paste <span class="shortcut">Ctrl+V</span>
          </button>
          <button class="menu-option" on:click={menuActions.delete}>
            Delete <span class="shortcut">Del</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- View Menu -->
    <div class="menu-item">
      <button
        class="menu-button"
        class:active={activeMenu === 'view'}
        on:click|stopPropagation={() => toggleMenu('view')}
      >
        View
      </button>
      {#if activeMenu === 'view'}
        <div class="menu-dropdown" on:click|stopPropagation>
          <button class="menu-option" on:click={menuActions.toggleDAW}>
            DAW Window
          </button>
          <button class="menu-option" on:click={menuActions.toggleMixer}>
            Mixer Window
          </button>
          <button class="menu-option" on:click={menuActions.toggleDatabase}>
            Database Window
          </button>
          <button class="menu-option" on:click={menuActions.togglePipeline}>
            Pipeline Window
          </button>
          <div class="menu-separator" />
          <button class="menu-option" on:click={menuActions.toggleGridSnapping}>
            Grid Snapping
          </button>
        </div>
      {/if}
    </div>

    <!-- Track Menu -->
    <div class="menu-item">
      <button
        class="menu-button"
        class:active={activeMenu === 'track'}
        on:click|stopPropagation={() => toggleMenu('track')}
      >
        Track
      </button>
      {#if activeMenu === 'track'}
        <div class="menu-dropdown" on:click|stopPropagation>
          <button class="menu-option" on:click={menuActions.addTrack}>
            Add Track <span class="shortcut">Ctrl+T</span>
          </button>
          <button class="menu-option" on:click={menuActions.deleteTrack}>
            Delete Track
          </button>
          <button class="menu-option" on:click={menuActions.duplicateTrack}>
            Duplicate Track
          </button>
        </div>
      {/if}
    </div>

    <!-- MIDI Menu -->
    <div class="menu-item">
      <button
        class="menu-button"
        class:active={activeMenu === 'midi'}
        on:click|stopPropagation={() => toggleMenu('midi')}
      >
        MIDI
      </button>
      {#if activeMenu === 'midi'}
        <div class="menu-dropdown" on:click|stopPropagation>
          <button class="menu-option" on:click={menuActions.playPause}>
            Play/Pause <span class="shortcut">Space</span>
          </button>
          <button class="menu-option" on:click={menuActions.stop}>
            Stop
          </button>
          <button class="menu-option" on:click={menuActions.record}>
            Record
          </button>
          <div class="menu-separator" />
          <button class="menu-option" on:click={menuActions.toggleMetronome}>
            Toggle Metronome
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .menu-bar {
    background: var(--menubar-bg, #2d2d2d);
    color: var(--menubar-text, #e0e0e0);
    height: 32px;
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    user-select: none;
    padding: 0 8px;
  }

  .menu-items {
    display: flex;
    gap: 4px;
  }

  .menu-item {
    position: relative;
  }

  .menu-button {
    background: none;
    border: none;
    color: var(--menubar-text, #e0e0e0);
    padding: 6px 12px;
    font-size: 13px;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s ease;
  }

  .menu-button:hover,
  .menu-button.active {
    background: var(--menu-hover, rgba(255, 255, 255, 0.1));
  }

  .menu-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    background: var(--dropdown-bg, #2d2d2d);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    min-width: 220px;
    padding: 4px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    z-index: 10000;
    margin-top: 2px;
  }

  .menu-option {
    width: 100%;
    background: none;
    border: none;
    color: var(--menubar-text, #e0e0e0);
    padding: 8px 12px;
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    transition: background 0.15s ease;
  }

  .menu-option:hover {
    background: var(--menu-hover, rgba(255, 255, 255, 0.1));
  }

  .shortcut {
    font-size: 11px;
    color: var(--shortcut-text, #888);
    margin-left: 24px;
  }

  .menu-separator {
    height: 1px;
    background: var(--border-color, #3e3e3e);
    margin: 4px 8px;
  }
</style>
