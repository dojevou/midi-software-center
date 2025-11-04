<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import SettingsModal from './SettingsModal.svelte';

  const dispatch = createEventDispatcher<{
    save: LibrarySettingsData;
    cancel: void;
  }>();

  export interface LibrarySettingsData {
    libraryPaths: string[];
    autoWatchFolders: boolean;
    duplicateHandling: 'keep-first' | 'keep-last' | 'skip';
  }

  export let settings: LibrarySettingsData = {
    libraryPaths: [],
    autoWatchFolders: true,
    duplicateHandling: 'keep-first'
  };

  let localSettings: LibrarySettingsData = JSON.parse(JSON.stringify(settings));
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let showModal: boolean = false;
  let modalAction: 'add' | 'remove' | 'rescan' | null = null;
  let selectedPathIndex: number = -1;
  let isScanning: boolean = false;

  // Mock library stats - would come from Tauri in real implementation
  let libraryStats = {
    totalFiles: 12483,
    untagged: 342,
    errors: 7
  };

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);

  function handleAddPath() {
    modalAction = 'add';
    showModal = true;
  }

  function handleRemovePath(index: number) {
    selectedPathIndex = index;
    modalAction = 'remove';
    showModal = true;
  }

  function handleModalConfirm(event: CustomEvent<string | null>) {
    if (modalAction === 'add' && event.detail) {
      localSettings.libraryPaths = [...localSettings.libraryPaths, event.detail];
      localSettings = localSettings;
    } else if (modalAction === 'remove') {
      localSettings.libraryPaths = localSettings.libraryPaths.filter((_, i) => i !== selectedPathIndex);
      localSettings = localSettings;
    } else if (modalAction === 'rescan') {
      handleRescanLibrary();
    }
    showModal = false;
    modalAction = null;
    selectedPathIndex = -1;
  }

  function handleModalCancel() {
    showModal = false;
    modalAction = null;
    selectedPathIndex = -1;
  }

  function handleAutoWatchToggle() {
    localSettings.autoWatchFolders = !localSettings.autoWatchFolders;
    localSettings = localSettings;
  }

  function handleDuplicateHandlingChange(mode: 'keep-first' | 'keep-last' | 'skip') {
    localSettings.duplicateHandling = mode;
    localSettings = localSettings;
  }

  function handleRescanPrompt() {
    modalAction = 'rescan';
    showModal = true;
  }

  async function handleRescanLibrary() {
    isScanning = true;
    // In real implementation, would trigger library rescan via Tauri
    await new Promise(resolve => setTimeout(resolve, 2000));
    isScanning = false;
  }

  function handleApply() {
    dispatch('save', localSettings);
    settings = JSON.parse(JSON.stringify(localSettings));
    hasChanges = false;
    showSaveIndicator();
  }

  function handleCancel() {
    localSettings = JSON.parse(JSON.stringify(settings));
    hasChanges = false;
    dispatch('cancel');
  }

  function handleReset() {
    const defaults: LibrarySettingsData = {
      libraryPaths: [],
      autoWatchFolders: true,
      duplicateHandling: 'keep-first'
    };
    localSettings = JSON.parse(JSON.stringify(defaults));
    localSettings = localSettings;
  }

  function showSaveIndicator() {
    saveIndicator = true;
    setTimeout(() => {
      saveIndicator = false;
    }, 2000);
  }
</script>

<div class="library-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Library Paths</h3>

      <div class="setting-group">
        <label class="setting-label">Watched Folders</label>
        <div class="paths-list">
          {#if localSettings.libraryPaths.length === 0}
            <div class="empty-state">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              <p>No library paths configured</p>
              <p class="empty-hint">Add folders to watch for MIDI files</p>
            </div>
          {:else}
            {#each localSettings.libraryPaths as path, index}
              <div class="path-item">
                <svg class="path-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
                </svg>
                <span class="path-text">{path}</span>
                <button
                  class="btn btn-small btn-danger"
                  on:click={() => handleRemovePath(index)}
                >
                  Remove
                </button>
              </div>
            {/each}
          {/if}
        </div>
        <button class="btn btn-primary" on:click={handleAddPath}>
          Add Folder
        </button>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Folder Watching</h3>

      <div class="setting-group">
        <div class="setting-row">
          <div>
            <label class="setting-label">Auto-Watch Folders</label>
            <p class="setting-description">Automatically detect and import new MIDI files added to watched folders</p>
          </div>
          <button
            class="toggle-btn"
            class:active={localSettings.autoWatchFolders}
            on:click={handleAutoWatchToggle}
            aria-label="Toggle auto-watch"
          >
            <span class="toggle-slider"></span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Duplicate Handling</h3>

      <div class="setting-group">
        <label class="setting-label">When duplicate files are found:</label>
        <div class="duplicate-selector">
          <button
            class="duplicate-option"
            class:active={localSettings.duplicateHandling === 'keep-first'}
            on:click={() => handleDuplicateHandlingChange('keep-first')}
          >
            <span class="duplicate-label">Keep First</span>
            <span class="duplicate-description">Keep the first occurrence and skip duplicates</span>
          </button>

          <button
            class="duplicate-option"
            class:active={localSettings.duplicateHandling === 'keep-last'}
            on:click={() => handleDuplicateHandlingChange('keep-last')}
          >
            <span class="duplicate-label">Keep Last</span>
            <span class="duplicate-description">Replace with the latest occurrence</span>
          </button>

          <button
            class="duplicate-option"
            class:active={localSettings.duplicateHandling === 'skip'}
            on:click={() => handleDuplicateHandlingChange('skip')}
          >
            <span class="duplicate-label">Skip</span>
            <span class="duplicate-description">Skip all duplicate files</span>
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Library Maintenance</h3>

      <div class="setting-group">
        <button
          class="btn btn-secondary"
          on:click={handleRescanPrompt}
          disabled={isScanning}
        >
          {isScanning ? 'Scanning...' : 'Rescan Library'}
        </button>
        <p class="setting-description">Scan all watched folders for new or changed files</p>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Library Statistics</h3>

      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
          </div>
          <div class="stat-info">
            <span class="stat-label">Total Files</span>
            <span class="stat-value">{libraryStats.totalFiles.toLocaleString()}</span>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon warning">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
              <line x1="9" y1="15" x2="15" y2="15"/>
            </svg>
          </div>
          <div class="stat-info">
            <span class="stat-label">Untagged</span>
            <span class="stat-value">{libraryStats.untagged.toLocaleString()}</span>
          </div>
        </div>

        <div class="stat-card">
          <div class="stat-icon error">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
          </div>
          <div class="stat-info">
            <span class="stat-label">Errors</span>
            <span class="stat-value">{libraryStats.errors.toLocaleString()}</span>
          </div>
        </div>
      </div>
    </section>
  </div>

  <div class="settings-footer">
    <div class="footer-left">
      {#if saveIndicator}
        <div class="save-indicator">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <path d="M20 6L9 17l-5-5"/>
          </svg>
          <span>Settings saved</span>
        </div>
      {/if}
    </div>
    <div class="footer-right">
      <button class="btn btn-secondary" on:click={handleReset}>
        Reset to Defaults
      </button>
      <button class="btn btn-secondary" on:click={handleCancel}>
        Cancel
      </button>
      <button
        class="btn btn-primary"
        disabled={!hasChanges}
        on:click={handleApply}
      >
        Apply
      </button>
    </div>
  </div>
</div>

{#if showModal}
  {#if modalAction === 'add'}
    <SettingsModal
      title="Add Library Path"
      message="Enter the path to a folder containing MIDI files:"
      confirmText="Add"
      showInput={true}
      inputPlaceholder="/path/to/midi/folder"
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {:else if modalAction === 'remove'}
    <SettingsModal
      title="Remove Library Path"
      message="Are you sure you want to remove this path from your library? Files will not be deleted."
      confirmText="Remove"
      dangerous={true}
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {:else if modalAction === 'rescan'}
    <SettingsModal
      title="Rescan Library"
      message="This will scan all watched folders for new or changed files. This may take several minutes."
      confirmText="Rescan"
      on:confirm={handleModalConfirm}
      on:cancel={handleModalCancel}
    />
  {/if}
{/if}

<style>
  .library-settings {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary, #252525);
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
  }

  .settings-section {
    margin-bottom: 2.5rem;
  }

  .section-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    margin: 0 0 1.5rem 0;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .setting-group {
    margin-bottom: 1.5rem;
  }

  .setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .setting-label {
    display: block;
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
    margin-bottom: 0.5rem;
  }

  .setting-description {
    font-size: 0.875rem;
    color: var(--text-secondary, #a0a0a0);
    margin: 0.25rem 0 0 0;
    line-height: 1.4;
  }

  .paths-list {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
    min-height: 150px;
    max-height: 300px;
    overflow-y: auto;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    text-align: center;
  }

  .empty-state svg {
    color: var(--text-secondary, #a0a0a0);
    margin-bottom: 1rem;
  }

  .empty-state p {
    color: var(--text-secondary, #a0a0a0);
    margin: 0.25rem 0;
  }

  .empty-hint {
    font-size: 0.875rem;
  }

  .path-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: var(--bg-primary, #252525);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    margin-bottom: 0.5rem;
  }

  .path-item:last-child {
    margin-bottom: 0;
  }

  .path-icon {
    color: var(--accent-color, #4a9eff);
    flex-shrink: 0;
  }

  .path-text {
    flex: 1;
    font-size: 0.9rem;
    color: var(--text-primary, #e0e0e0);
    font-family: monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toggle-btn {
    position: relative;
    width: 48px;
    height: 24px;
    background: var(--bg-tertiary, #333);
    border: none;
    border-radius: 12px;
    cursor: pointer;
    transition: background 0.2s;
    flex-shrink: 0;
  }

  .toggle-btn.active {
    background: var(--accent-color, #4a9eff);
  }

  .toggle-slider {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: #fff;
    border-radius: 10px;
    transition: transform 0.2s;
  }

  .toggle-btn.active .toggle-slider {
    transform: translateX(24px);
  }

  .duplicate-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .duplicate-option {
    background: var(--bg-secondary, #1e1e1e);
    border: 2px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.5rem 1rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    text-align: center;
  }

  .duplicate-option:hover {
    border-color: var(--border-hover, #444);
  }

  .duplicate-option.active {
    border-color: var(--accent-color, #4a9eff);
    background: rgba(74, 158, 255, 0.05);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .duplicate-label {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .duplicate-description {
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
  }

  .stat-card {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .stat-icon {
    color: var(--accent-color, #4a9eff);
  }

  .stat-icon.warning {
    color: var(--warning-color, #ffaa44);
  }

  .stat-icon.error {
    color: var(--error-color, #ff4444);
  }

  .stat-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    text-align: center;
  }

  .stat-label {
    font-size: 0.85rem;
    color: var(--text-secondary, #a0a0a0);
  }

  .stat-value {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .settings-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    background: var(--bg-secondary, #1e1e1e);
    border-top: 1px solid var(--border-color, #333);
  }

  .footer-left {
    min-width: 150px;
  }

  .footer-right {
    display: flex;
    gap: 0.75rem;
  }

  .save-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--success-color, #44ff44);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 6px;
    font-size: 0.95rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-small {
    padding: 0.5rem 0.75rem;
    font-size: 0.85rem;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: var(--bg-tertiary, #333);
    color: var(--text-primary, #e0e0e0);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover, #3a3a3a);
  }

  .btn-primary {
    background: var(--accent-color, #4a9eff);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #3a8eef;
  }

  .btn-danger {
    background: var(--error-color, #ff4444);
    color: #fff;
  }

  .btn-danger:hover:not(:disabled) {
    background: #ee3333;
  }
</style>
