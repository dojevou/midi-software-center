<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { api } from './lib/api';
  import {
    currentView,
    midiDevices,
    connectedDevice,
    midiConnected,
    searchResults,
    searchTotal,
    selectedFile,
    initializeStores,
    startPeriodicUpdates,
    connectMidiDevice,
    disconnectMidiDevice,
    toggleFavorite as storToggleFavorite,
    notifications,
  } from './lib/stores';
  import { keyboardManager } from './lib/utils/keyboard';

  import Search from './lib/components/Search.svelte';
  import FileList from './lib/components/FileList.svelte';
  import PianoRoll from './lib/components/PianoRoll.svelte';
  import Sequencer from './lib/components/Sequencer.svelte';
  import KeyboardShortcutsHelp from './lib/components/KeyboardShortcutsHelp.svelte';

  let status = 'Loading...';
  let filesCount = 0;
  let showMidiPanel = false;
  let showShortcutsHelp = false;
  let viewMode: 'grid' | 'list' = 'grid';
  let cleanupPeriodicUpdates: (() => void) | null = null;
  let dbInitialized = false;
  let initError: string | null = null;

  onMount(async () => {
    try {
      // Initialize database first
      console.log('Initializing database...');
      await invoke('initialize_database');
      dbInitialized = true;
      console.log('✅ Database initialized');

      // Initialize all stores (loads favorites, MIDI, tracks, preferences)
      await initializeStores();

      // Test backend connection and get initial file count
      const result = await api.search.files({});
      filesCount = result.total;
      $searchResults = result.files;
      $searchTotal = result.total;

      status = 'Connected ✅';

      // Start periodic updates for MIDI devices and playback position
      cleanupPeriodicUpdates = startPeriodicUpdates();
    } catch (error) {
      console.error('Backend error:', error);
      const errorMsg = error instanceof Error ? error.message : String(error);
      
      if (!dbInitialized && errorMsg.includes('database')) {
        initError = `Database connection failed: ${errorMsg}`;
        console.error('❌', initError);
      } else {
        status = 'Error: ' + errorMsg;
      }
    }

    // Register keyboard shortcuts
    registerKeyboardShortcuts();
    keyboardManager.start();
  });

  onDestroy(() => {
    keyboardManager.stop();
    keyboardManager.clear();

    // Stop periodic updates
    if (cleanupPeriodicUpdates) {
      cleanupPeriodicUpdates();
    }
  });

  async function connectMidi(deviceName: string) {
    await connectMidiDevice(deviceName);
    showMidiPanel = false;
  }

  async function disconnectMidi() {
    await disconnectMidiDevice();
  }

  function setView(view: 'search' | 'piano-roll' | 'sequencer') {
    $currentView = view;
  }

  function registerKeyboardShortcuts() {
    // Help dialog shortcut
    keyboardManager.register({
      key: '?',
      shift: true,
      description: 'Show keyboard shortcuts',
      category: 'Display',
      action: () => {
        showShortcutsHelp = !showShortcutsHelp;
      },
    });

    // View navigation shortcuts
    keyboardManager.register({
      key: 'Tab',
      description: 'Cycle through views',
      category: 'Display',
      action: cycleViews,
    });

    // Search shortcut
    keyboardManager.register({
      key: 'f',
      ctrl: true,
      description: 'Focus search bar',
      category: 'Search/UI',
      action: focusSearch,
    });

    // Escape to close dialogs/deselect
    keyboardManager.register({
      key: 'Escape',
      description: 'Close dialogs / Deselect',
      category: 'Selection',
      action: handleEscape,
    });

    // Favorite toggle
    keyboardManager.register({
      key: 'f',
      description: 'Toggle favorite on selected file',
      category: 'File Operations',
      action: toggleFavorite,
    });

    // View mode shortcuts
    keyboardManager.register({
      key: 'g',
      description: 'Grid view',
      category: 'Display',
      action: () => {
        viewMode = 'grid';
      },
    });

    keyboardManager.register({
      key: 'l',
      description: 'List view',
      category: 'Display',
      action: () => {
        viewMode = 'list';
      },
    });

    // Selection shortcuts
    keyboardManager.register({
      key: 'a',
      ctrl: true,
      description: 'Select all',
      category: 'Selection',
      action: selectAll,
    });

    // File operations
    keyboardManager.register({
      key: 'e',
      ctrl: true,
      description: 'Export project',
      category: 'File Operations',
      action: exportProject,
    });

    keyboardManager.register({
      key: 'w',
      ctrl: true,
      description: 'Close file',
      category: 'File Operations',
      action: closeFile,
    });

    // Fullscreen toggle
    keyboardManager.register({
      key: 'F11',
      description: 'Toggle fullscreen',
      category: 'Display',
      action: toggleFullscreen,
    });
  }

  function cycleViews() {
    const views: ('search' | 'piano-roll' | 'sequencer')[] = ['search', 'piano-roll', 'sequencer'];
    const currentIndex = views.indexOf($currentView);
    const nextIndex = (currentIndex + 1) % views.length;

    // Skip piano-roll if no file is selected
    if (views[nextIndex] === 'piano-roll' && !$selectedFile) {
      const afterNext = (nextIndex + 1) % views.length;
      $currentView = views[afterNext];
    } else {
      $currentView = views[nextIndex];
    }
  }

  function focusSearch() {
    const searchInput = document.querySelector('input[type="search"]') as HTMLInputElement;
    if (searchInput) {
      searchInput.focus();
      searchInput.select();
    }
  }

  function handleEscape() {
    if (showMidiPanel) {
      showMidiPanel = false;
    } else if (showShortcutsHelp) {
      showShortcutsHelp = false;
    } else if ($selectedFile) {
      $selectedFile = null;
    }
  }

  async function toggleFavorite() {
    if (!$selectedFile) return;
    await storToggleFavorite($selectedFile.id);

    // Refresh the selected file details to get updated favorite status
    try {
      const updated = await api.files.getDetails($selectedFile.id);
      $selectedFile = updated;
    } catch (error) {
      console.error('Failed to refresh file details:', error);
    }
  }

  function selectAll() {
    console.log('Select all - not yet implemented');
    // TODO: Implement select all functionality
  }

  async function exportProject() {
    try {
      // For now, just log. In a full implementation, this would open a save dialog
      console.log('Export project triggered');
      alert('Export functionality coming soon!');
    } catch (error) {
      console.error('Failed to export:', error);
    }
  }

  function closeFile() {
    $selectedFile = null;
    if ($currentView === 'piano-roll') {
      $currentView = 'search';
    }
  }

  function toggleFullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch((err) => {
        console.error('Error entering fullscreen:', err);
      });
    } else {
      document.exitFullscreen();
    }
  }
</script>

<main>
  <!-- Header -->
  <header class="app-header">
    <div class="header-left">
      <h1>🎹 MIDI Library DAW</h1>
      <span class="status">{status}</span>
    </div>

    <div class="header-right">
      <span class="file-count">{filesCount} files</span>

      <button
        class="midi-status {$midiConnected ? 'connected' : 'disconnected'}"
        on:click={() => (showMidiPanel = !showMidiPanel)}
      >
        {#if $midiConnected}
          ✅ {$connectedDevice?.name || 'Connected'}
        {:else}
          ⚠️ No MIDI Device
        {/if}
      </button>
    </div>
  </header>

  <!-- MIDI Connection Panel -->
  {#if showMidiPanel}
    <div class="midi-panel">
      <div class="midi-panel-content">
        <h3>MIDI Devices</h3>

        {#if $midiDevices.length === 0}
          <p>No MIDI devices detected</p>
        {:else}
          <div class="device-list">
            {#each $midiDevices as device}
              <button
                class="device-item {$connectedDevice?.name === device.name ? 'active' : ''}"
                on:click={() => connectMidi(device.name)}
              >
                {device.name}
                {#if $connectedDevice?.name === device.name}
                  <span class="connected-badge">Connected</span>
                {/if}
              </button>
            {/each}
          </div>
        {/if}

        {#if $midiConnected}
          <button class="disconnect-btn" on:click={disconnectMidi}> Disconnect </button>
        {/if}

        <button class="close-btn" on:click={() => (showMidiPanel = false)}> Close </button>
      </div>
    </div>
  {/if}

  <!-- Navigation -->
  <nav class="app-nav">
    <button
      class="nav-btn {$currentView === 'search' ? 'active' : ''}"
      on:click={() => setView('search')}
    >
      🔍 Search & Browse
    </button>
    <button
      class="nav-btn {$currentView === 'piano-roll' ? 'active' : ''}"
      on:click={() => setView('piano-roll')}
      disabled={!$selectedFile}
    >
      🎹 Piano Roll
    </button>
    <button
      class="nav-btn {$currentView === 'sequencer' ? 'active' : ''}"
      on:click={() => setView('sequencer')}
    >
      🎵 Sequencer
    </button>
  </nav>

  <!-- Main Content -->
  <div class="app-content">
    {#if $currentView === 'search'}
      <div class="search-view">
        <Search />

        <div class="file-list-panel">
          <FileList {viewMode} />
        </div>
      </div>
    {:else if $currentView === 'piano-roll'}
      <div class="piano-roll-view">
        <PianoRoll />
      </div>
    {:else if $currentView === 'sequencer'}
      <div class="sequencer-view">
        <Sequencer />
      </div>
    {/if}
  </div>

  <!-- Footer -->
  <footer class="app-footer">
    <div class="footer-left">
      {#if $selectedFile}
        <span class="selected-file">
          Selected: <strong>{$selectedFile.file_name}</strong>
        </span>
      {/if}
    </div>

    <div class="footer-right">
      <button
        class="shortcuts-btn"
        on:click={() => (showShortcutsHelp = true)}
        title="Keyboard Shortcuts (?)"
      >
        ⌨️
      </button>
      <span class="hint">💡 Press ? for keyboard shortcuts</span>
    </div>
  </footer>

  <!-- Keyboard Shortcuts Help Dialog -->
  <KeyboardShortcutsHelp bind:visible={showShortcutsHelp} />

  <!-- Toast Notifications -->
  {#if $notifications.length > 0}
    <div class="notifications">
      {#each $notifications as notification (notification.id)}
        <div class="notification notification-{notification.type}">
          <span class="notification-message">{notification.message}</span>
          <button
            class="notification-close"
            on:click={() => {
              import('./lib/stores').then(({ removeNotification }) => {
                removeNotification(notification.id);
              });
            }}
          >
            ×
          </button>
        </div>
      {/each}
    </div>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family:
      -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: linear-gradient(135deg, #1a1a1a 0%, #2d2d2d 100%);
    color: #fff;
    overflow: hidden;
  }

  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  /* Header */
  .app-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    background: rgba(0, 0, 0, 0.5);
    border-bottom: 2px solid rgba(255, 62, 0, 0.3);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .app-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 700;
    color: #ff3e00;
  }

  .status {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .header-right {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .file-count {
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 6px;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.7);
  }

  .midi-status {
    padding: 0.5rem 1rem;
    border: 2px solid;
    border-radius: 6px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .midi-status.connected {
    background: rgba(76, 175, 80, 0.1);
    border-color: #4caf50;
    color: #4caf50;
  }

  .midi-status.disconnected {
    background: rgba(255, 152, 0, 0.1);
    border-color: #ff9800;
    color: #ff9800;
  }

  .midi-status:hover {
    transform: translateY(-2px);
  }

  /* MIDI Panel */
  .midi-panel {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .midi-panel-content {
    background: #2d2d2d;
    border: 2px solid rgba(255, 62, 0, 0.5);
    border-radius: 12px;
    padding: 2rem;
    max-width: 500px;
    width: 90%;
  }

  .midi-panel-content h3 {
    margin: 0 0 1.5rem 0;
    font-size: 1.25rem;
    color: #ff3e00;
  }

  .device-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }

  .device-item {
    padding: 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #fff;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .device-item:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: #ff3e00;
  }

  .device-item.active {
    background: rgba(255, 62, 0, 0.2);
    border-color: #ff3e00;
  }

  .connected-badge {
    padding: 0.25rem 0.5rem;
    background: #4caf50;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .disconnect-btn,
  .close-btn {
    width: 100%;
    padding: 0.75rem;
    border: none;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .disconnect-btn {
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
    margin-bottom: 0.5rem;
  }

  .disconnect-btn:hover {
    background: rgba(244, 67, 54, 0.3);
  }

  .close-btn {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  /* Navigation */
  .app-nav {
    display: flex;
    gap: 0.5rem;
    padding: 1rem 1.5rem;
    background: rgba(0, 0, 0, 0.3);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .nav-btn {
    padding: 0.75rem 1.5rem;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #fff;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .nav-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    border-color: #ff3e00;
  }

  .nav-btn.active {
    background: rgba(255, 62, 0, 0.2);
    border-color: #ff3e00;
    color: #ff3e00;
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  /* Content */
  .app-content {
    flex: 1;
    overflow: hidden;
    padding: 1.5rem;
  }

  .search-view {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .search-panel {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  .search-panel > :first-child {
    flex: 1;
  }

  .view-mode-toggle {
    display: flex;
    gap: 0.5rem;
  }

  .view-btn {
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s;
  }

  .view-btn:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .view-btn.active {
    background: rgba(255, 62, 0, 0.2);
    border-color: #ff3e00;
  }

  .file-list-panel {
    flex: 1;
    overflow-y: auto;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 8px;
  }

  .piano-roll-view,
  .sequencer-view {
    height: 100%;
  }

  /* Footer */
  .app-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1.5rem;
    background: rgba(0, 0, 0, 0.5);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    font-size: 0.875rem;
  }

  .selected-file {
    color: rgba(255, 255, 255, 0.7);
  }

  .selected-file strong {
    color: #ff3e00;
  }

  .shortcuts-btn {
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    transition: all 0.2s;
    font-size: 1.125rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .shortcuts-btn:hover {
    background: rgba(255, 62, 0, 0.2);
    border-color: rgba(255, 62, 0, 0.5);
    color: #ff3e00;
    transform: translateY(-2px);
  }

  .hint {
    color: rgba(255, 255, 255, 0.4);
    font-size: 0.8125rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  /* Scrollbar styling */
  :global(::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(::-webkit-scrollbar-track) {
    background: rgba(0, 0, 0, 0.2);
  }

  :global(::-webkit-scrollbar-thumb) {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(255, 255, 255, 0.3);
  }

  /* Notifications */
  .notifications {
    position: fixed;
    top: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 2000;
    max-width: 400px;
  }

  .notification {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.9);
    border: 2px solid;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .notification-success {
    border-color: #4caf50;
    background: rgba(76, 175, 80, 0.1);
  }

  .notification-error {
    border-color: #f44336;
    background: rgba(244, 67, 54, 0.1);
  }

  .notification-info {
    border-color: #2196f3;
    background: rgba(33, 150, 243, 0.1);
  }

  .notification-warning {
    border-color: #ff9800;
    background: rgba(255, 152, 0, 0.1);
  }

  .notification-message {
    flex: 1;
    color: #fff;
    font-size: 0.875rem;
  }

  .notification-close {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.6);
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0 0.5rem;
    transition: color 0.2s;
  }

  .notification-close:hover {
    color: #fff;
  }
</style>
