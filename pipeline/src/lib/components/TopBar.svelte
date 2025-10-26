<script lang="ts">
  import { appStateStore } from '../stores/appState';

  type ViewType = 'search' | 'import' | 'sequencer';

  interface Tab {
    id: ViewType;
    label: string;
    icon: string;
  }

  const tabs: Tab[] = [
    { id: 'search', label: 'Search', icon: '🔍' },
    { id: 'import', label: 'Import', icon: '📥' },
    { id: 'sequencer', label: 'Sequencer', icon: '🎹' }
  ];

  function handleTabClick(viewId: ViewType) {
    appStateStore.setView(viewId);
  }

  // Subscribe to store state
  $: currentView = $appStateStore.currentView;
  $: isConnected = $appStateStore.isConnected;
  $: connectionStatus = isConnected ? 'Connected' : 'Disconnected';
</script>

<header class="topbar">
  <div class="topbar-left">
    <div class="app-logo">
      <span class="logo-icon">🎵</span>
      <h1 class="app-title">MIDI Library Pipeline</h1>
    </div>
  </div>

  <nav class="topbar-center">
    <div class="tabs">
      {#each tabs as tab}
        <button
          class="tab"
          class:active={currentView === tab.id}
          on:click={() => handleTabClick(tab.id)}
          aria-label={`Switch to ${tab.label} view`}
          aria-current={currentView === tab.id ? 'page' : undefined}
        >
          <span class="tab-icon">{tab.icon}</span>
          <span class="tab-label">{tab.label}</span>
        </button>
      {/each}
    </div>
  </nav>

  <div class="topbar-right">
    <div class="connection-status" class:connected={isConnected}>
      <span class="status-dot"></span>
      <span class="status-text">{connectionStatus}</span>
    </div>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--topbar-height, 60px);
    padding: 0 var(--spacing-lg, 24px);
    background: var(--color-surface, #2d2d2d);
    color: var(--color-text, #e5e5e5);
  }

  .topbar-left,
  .topbar-center,
  .topbar-right {
    display: flex;
    align-items: center;
    flex: 1;
  }

  .topbar-center {
    justify-content: center;
  }

  .topbar-right {
    justify-content: flex-end;
  }

  /* Logo section */
  .app-logo {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
  }

  .logo-icon {
    font-size: 28px;
    line-height: 1;
  }

  .app-title {
    font-size: var(--font-size-lg, 18px);
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
    margin: 0;
    white-space: nowrap;
  }

  /* Tabs navigation */
  .tabs {
    display: flex;
    gap: var(--spacing-xs, 4px);
    padding: var(--spacing-xs, 4px);
    background: var(--color-bg, #1a1a1a);
    border-radius: var(--radius-lg, 8px);
  }

  .tab {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-sm, 8px) var(--spacing-md, 16px);
    background: transparent;
    border: none;
    border-radius: var(--radius-md, 6px);
    color: var(--color-text-secondary, #a3a3a3);
    font-size: var(--font-size-sm, 14px);
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    white-space: nowrap;
  }

  .tab:hover {
    background: var(--color-surface, #2d2d2d);
    color: var(--color-text, #e5e5e5);
  }

  .tab.active {
    background: var(--color-primary, #3b82f6);
    color: white;
    font-weight: 600;
  }

  .tab-icon {
    font-size: 18px;
    line-height: 1;
  }

  .tab-label {
    line-height: 1;
  }

  /* Connection status */
  .connection-status {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-sm, 8px) var(--spacing-md, 16px);
    background: var(--color-bg, #1a1a1a);
    border-radius: var(--radius-md, 6px);
    font-size: var(--font-size-sm, 14px);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-error, #ef4444);
    animation: pulse-error 2s infinite;
  }

  .connection-status.connected .status-dot {
    background: var(--color-success, #10b981);
    animation: pulse-success 2s infinite;
  }

  .status-text {
    color: var(--color-text-secondary, #a3a3a3);
    font-weight: 500;
    white-space: nowrap;
  }

  .connection-status.connected .status-text {
    color: var(--color-success, #10b981);
  }

  @keyframes pulse-success {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.6;
    }
  }

  @keyframes pulse-error {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .topbar {
      padding: 0 var(--spacing-md, 16px);
    }

    .app-title {
      display: none;
    }

    .tab-label {
      display: none;
    }

    .status-text {
      display: none;
    }
  }
</style>
