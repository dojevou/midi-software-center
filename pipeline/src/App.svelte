<script lang="ts">
  import { onMount } from 'svelte';
  import TopBar from './lib/components/TopBar.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import { appStateStore } from './lib/stores/appState';

  // View components (will be created later)
  // import SearchView from './lib/views/SearchView.svelte';
  // import ImportView from './lib/views/ImportView.svelte';
  // import SequencerView from './lib/views/SequencerView.svelte';

  let appReady = false;
  let initError = '';

  onMount(async () => {
    try {
      // Load settings from localStorage
      appStateStore.loadSettings();
      appReady = true;
    } catch (error) {
      console.error('App initialization failed:', error);
      initError = error instanceof Error ? error.message : 'Unknown initialization error';
      appReady = true; // Show error screen
    }
  });

  // Subscribe to current view from store
  $: currentView = $appStateStore.currentView;
</script>

<div class="app">
  {#if !appReady}
    <div class="loading-screen">
      <div class="spinner"></div>
      <p>Initializing MIDI Library Pipeline...</p>
    </div>
  {:else if initError}
    <div class="error-screen">
      <div class="error-icon">⚠️</div>
      <h2>Initialization Error</h2>
      <p>{initError}</p>
      <button class="btn-retry" on:click={() => window.location.reload()}>
        Retry
      </button>
    </div>
  {:else}
    <div class="app-grid">
      <div class="app-topbar">
        <TopBar />
      </div>

      <div class="app-sidebar">
        <Sidebar />
      </div>

      <main class="app-content">
        {#if currentView === 'search'}
          <div class="view-placeholder">
            <h1>Search View</h1>
            <p>Search and browse MIDI files</p>
          </div>
        {:else if currentView === 'import'}
          <div class="view-placeholder">
            <h1>Import View</h1>
            <p>Import MIDI files from folders</p>
          </div>
        {:else if currentView === 'sequencer'}
          <div class="view-placeholder">
            <h1>Sequencer View</h1>
            <p>Multi-track MIDI sequencer</p>
          </div>
        {:else}
          <div class="view-placeholder">
            <h1>Welcome</h1>
            <p>Select a view to get started</p>
          </div>
        {/if}
      </main>

      <div class="app-statusbar">
        <StatusBar />
      </div>
    </div>
  {/if}
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: var(--color-bg, #1a1a1a);
    color: var(--color-text, #e5e5e5);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
                 'Helvetica Neue', Arial, sans-serif;
  }

  :global(:root) {
    /* Design system colors */
    --color-bg: #1a1a1a;
    --color-surface: #2d2d2d;
    --color-primary: #3b82f6;
    --color-text: #e5e5e5;
    --color-text-secondary: #a3a3a3;
    --color-border: #404040;
    --color-success: #10b981;
    --color-error: #ef4444;
    --color-warning: #f59e0b;

    /* Spacing */
    --spacing-xs: 4px;
    --spacing-sm: 8px;
    --spacing-md: 16px;
    --spacing-lg: 24px;
    --spacing-xl: 32px;

    /* Typography */
    --font-size-xs: 12px;
    --font-size-sm: 14px;
    --font-size-md: 16px;
    --font-size-lg: 18px;
    --font-size-xl: 24px;

    /* Border radius */
    --radius-sm: 4px;
    --radius-md: 6px;
    --radius-lg: 8px;

    /* Shadows */
    --shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.5);
    --shadow-md: 0 4px 6px rgba(0, 0, 0, 0.5);
    --shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.5);

    /* Layout dimensions */
    --topbar-height: 60px;
    --sidebar-width: 250px;
    --sidebar-collapsed-width: 60px;
    --statusbar-height: 40px;
  }

  .app {
    width: 100vw;
    height: 100vh;
    background: var(--color-bg);
    overflow: hidden;
  }

  .loading-screen,
  .error-screen {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    gap: var(--spacing-lg);
    padding: var(--spacing-xl);
  }

  .spinner {
    width: 64px;
    height: 64px;
    border: 6px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-screen p {
    font-size: var(--font-size-md);
    color: var(--color-text-secondary);
  }

  .error-icon {
    font-size: 72px;
  }

  .error-screen h2 {
    margin: 0;
    font-size: var(--font-size-xl);
    color: var(--color-error);
  }

  .error-screen p {
    margin: 0;
    font-size: var(--font-size-md);
    color: var(--color-text-secondary);
    text-align: center;
    max-width: 500px;
  }

  .btn-retry {
    padding: 12px 24px;
    background: var(--color-primary);
    border: none;
    border-radius: var(--radius-md);
    color: white;
    font-size: var(--font-size-md);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-retry:hover {
    background: #2563eb;
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }

  .app-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: var(--topbar-height) 1fr var(--statusbar-height);
    grid-template-areas:
      "topbar topbar"
      "sidebar content"
      "statusbar statusbar";
    width: 100vw;
    height: 100vh;
    overflow: hidden;
  }

  .app-topbar {
    grid-area: topbar;
    border-bottom: 1px solid var(--color-border);
  }

  .app-sidebar {
    grid-area: sidebar;
    border-right: 1px solid var(--color-border);
  }

  .app-content {
    grid-area: content;
    overflow: auto;
    background: var(--color-bg);
  }

  .app-statusbar {
    grid-area: statusbar;
    border-top: 1px solid var(--color-border);
  }

  /* Temporary view placeholder styles */
  .view-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--spacing-md);
    padding: var(--spacing-xl);
  }

  .view-placeholder h1 {
    font-size: var(--font-size-xl);
    color: var(--color-text);
    margin: 0;
  }

  .view-placeholder p {
    font-size: var(--font-size-md);
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* Custom scrollbar for content area */
  .app-content::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .app-content::-webkit-scrollbar-track {
    background: var(--color-bg);
  }

  .app-content::-webkit-scrollbar-thumb {
    background: var(--color-border);
    border-radius: var(--radius-sm);
  }

  .app-content::-webkit-scrollbar-thumb:hover {
    background: #525252;
  }
</style>
