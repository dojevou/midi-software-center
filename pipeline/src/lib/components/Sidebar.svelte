<script lang="ts">
  import { appStateStore } from '../stores/appState';

  type ViewType = 'search' | 'import' | 'sequencer';

  interface NavItem {
    id: ViewType;
    label: string;
    icon: string;
    description: string;
  }

  const navItems: NavItem[] = [
    {
      id: 'search',
      label: 'Search',
      icon: '🔍',
      description: 'Browse and search MIDI files'
    },
    {
      id: 'import',
      label: 'Import',
      icon: '📥',
      description: 'Import MIDI files from folders'
    },
    {
      id: 'sequencer',
      label: 'Sequencer',
      icon: '🎹',
      description: 'Multi-track sequencer'
    }
  ];

  let isCollapsed = false;

  function handleNavClick(viewId: ViewType) {
    appStateStore.setView(viewId);
  }

  function toggleSidebar() {
    isCollapsed = !isCollapsed;
  }

  // Subscribe to current view
  $: currentView = $appStateStore.currentView;
</script>

<aside class="sidebar" class:collapsed={isCollapsed}>
  <div class="sidebar-header">
    <button
      class="collapse-btn"
      on:click={toggleSidebar}
      aria-label={isCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      title={isCollapsed ? 'Expand' : 'Collapse'}
    >
      <span class="collapse-icon">
        {isCollapsed ? '▶' : '◀'}
      </span>
    </button>

    {#if !isCollapsed}
      <h2 class="sidebar-title">Navigation</h2>
    {/if}
  </div>

  <nav class="sidebar-nav">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={currentView === item.id}
        on:click={() => handleNavClick(item.id)}
        aria-label={`Navigate to ${item.label}`}
        aria-current={currentView === item.id ? 'page' : undefined}
        title={isCollapsed ? item.label : item.description}
      >
        <span class="nav-icon">{item.icon}</span>
        {#if !isCollapsed}
          <div class="nav-content">
            <span class="nav-label">{item.label}</span>
            <span class="nav-description">{item.description}</span>
          </div>
        {/if}
        {#if !isCollapsed && currentView === item.id}
          <span class="nav-indicator"></span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="sidebar-footer">
    {#if !isCollapsed}
      <div class="footer-info">
        <span class="footer-icon">ℹ️</span>
        <p class="footer-text">Select a view to get started</p>
      </div>
    {:else}
      <div class="footer-icon-only">ℹ️</div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: var(--sidebar-width, 250px);
    height: 100%;
    background: var(--color-surface, #2d2d2d);
    transition: width 0.3s ease;
    overflow: hidden;
  }

  .sidebar.collapsed {
    width: var(--sidebar-collapsed-width, 60px);
  }

  /* Header */
  .sidebar-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-md, 16px);
    border-bottom: 1px solid var(--color-border, #404040);
    min-height: 60px;
  }

  .collapse-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background: var(--color-bg, #1a1a1a);
    border: 1px solid var(--color-border, #404040);
    border-radius: var(--radius-md, 6px);
    color: var(--color-text, #e5e5e5);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
    flex-shrink: 0;
  }

  .collapse-btn:hover {
    background: var(--color-primary, #3b82f6);
    border-color: var(--color-primary, #3b82f6);
    transform: scale(1.05);
  }

  .collapse-icon {
    display: block;
    line-height: 1;
  }

  .sidebar-title {
    font-size: var(--font-size-md, 16px);
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
  }

  /* Navigation */
  .sidebar-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs, 4px);
    padding: var(--spacing-md, 16px);
    overflow-y: auto;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-md, 16px);
    padding: var(--spacing-md, 16px);
    background: transparent;
    border: none;
    border-radius: var(--radius-md, 6px);
    color: var(--color-text-secondary, #a3a3a3);
    font-size: var(--font-size-sm, 14px);
    cursor: pointer;
    transition: all 0.2s;
    position: relative;
    text-align: left;
    width: 100%;
  }

  .sidebar.collapsed .nav-item {
    justify-content: center;
    padding: var(--spacing-md, 16px) var(--spacing-sm, 8px);
  }

  .nav-item:hover {
    background: var(--color-bg, #1a1a1a);
    color: var(--color-text, #e5e5e5);
  }

  .nav-item.active {
    background: var(--color-primary, #3b82f6);
    color: white;
  }

  .nav-icon {
    font-size: 24px;
    line-height: 1;
    flex-shrink: 0;
  }

  .nav-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }

  .nav-label {
    font-weight: 600;
    line-height: 1.2;
  }

  .nav-description {
    font-size: var(--font-size-xs, 12px);
    opacity: 0.8;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-indicator {
    width: 6px;
    height: 6px;
    background: white;
    border-radius: 50%;
    flex-shrink: 0;
  }

  /* Footer */
  .sidebar-footer {
    padding: var(--spacing-md, 16px);
    border-top: 1px solid var(--color-border, #404040);
  }

  .footer-info {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-sm, 8px);
    padding: var(--spacing-sm, 8px);
    background: var(--color-bg, #1a1a1a);
    border-radius: var(--radius-md, 6px);
  }

  .footer-icon {
    font-size: 20px;
    line-height: 1;
    flex-shrink: 0;
  }

  .footer-text {
    font-size: var(--font-size-xs, 12px);
    color: var(--color-text-secondary, #a3a3a3);
    line-height: 1.4;
    margin: 0;
  }

  .footer-icon-only {
    display: flex;
    justify-content: center;
    font-size: 20px;
  }

  /* Custom scrollbar */
  .sidebar-nav::-webkit-scrollbar {
    width: 6px;
  }

  .sidebar-nav::-webkit-scrollbar-track {
    background: transparent;
  }

  .sidebar-nav::-webkit-scrollbar-thumb {
    background: var(--color-border, #404040);
    border-radius: 3px;
  }

  .sidebar-nav::-webkit-scrollbar-thumb:hover {
    background: #525252;
  }
</style>
