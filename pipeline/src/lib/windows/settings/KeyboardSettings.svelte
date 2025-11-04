<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import SettingsModal from './SettingsModal.svelte';

  const dispatch = createEventDispatcher<{
    save: KeyboardSettingsData;
    cancel: void;
  }>();

  interface Keybinding {
    command: string;
    description: string;
    binding: string;
    category: string;
  }

  export interface KeyboardSettingsData {
    preset: string;
    keybindings: Keybinding[];
  }

  export let settings: KeyboardSettingsData = {
    preset: 'standard',
    keybindings: []
  };

  let localSettings: KeyboardSettingsData = JSON.parse(JSON.stringify(settings));
  let hasChanges: boolean = false;
  let saveIndicator: boolean = false;
  let searchQuery: string = '';
  let editingCommand: string | null = null;
  let recordingKey: boolean = false;
  let newBinding: string = '';
  let conflictCommand: string | null = null;
  let showModal: boolean = false;
  let modalAction: 'export' | 'import' | null = null;

  const presets = [
    { value: 'standard', label: 'Standard' },
    { value: 'ableton', label: 'Ableton Live' },
    { value: 'protools', label: 'Pro Tools' },
    { value: 'studioone', label: 'Studio One' },
    { value: 'fl', label: 'FL Studio' },
    { value: 'logic', label: 'Logic Pro' },
    { value: 'reaper', label: 'Reaper' }
  ];

  // Default keybindings
  if (localSettings.keybindings.length === 0) {
    localSettings.keybindings = [
      { command: 'file.import', description: 'Import Files', binding: 'Ctrl+I', category: 'File' },
      { command: 'file.export', description: 'Export', binding: 'Ctrl+E', category: 'File' },
      { command: 'file.save', description: 'Save Project', binding: 'Ctrl+S', category: 'File' },
      { command: 'file.open', description: 'Open Project', binding: 'Ctrl+O', category: 'File' },
      { command: 'edit.undo', description: 'Undo', binding: 'Ctrl+Z', category: 'Edit' },
      { command: 'edit.redo', description: 'Redo', binding: 'Ctrl+Y', category: 'Edit' },
      { command: 'edit.copy', description: 'Copy', binding: 'Ctrl+C', category: 'Edit' },
      { command: 'edit.paste', description: 'Paste', binding: 'Ctrl+V', category: 'Edit' },
      { command: 'edit.cut', description: 'Cut', binding: 'Ctrl+X', category: 'Edit' },
      { command: 'edit.selectall', description: 'Select All', binding: 'Ctrl+A', category: 'Edit' },
      { command: 'view.search', description: 'Search', binding: 'Ctrl+F', category: 'View' },
      { command: 'view.settings', description: 'Open Settings', binding: 'Ctrl+,', category: 'View' },
      { command: 'playback.play', description: 'Play/Pause', binding: 'Space', category: 'Playback' },
      { command: 'playback.stop', description: 'Stop', binding: 'Escape', category: 'Playback' },
      { command: 'playback.rewind', description: 'Rewind', binding: 'Home', category: 'Playback' },
      { command: 'track.new', description: 'New Track', binding: 'Ctrl+T', category: 'Track' },
      { command: 'track.delete', description: 'Delete Track', binding: 'Delete', category: 'Track' },
      { command: 'track.mute', description: 'Mute Track', binding: 'M', category: 'Track' },
      { command: 'track.solo', description: 'Solo Track', binding: 'S', category: 'Track' }
    ];
  }

  $: hasChanges = JSON.stringify(localSettings) !== JSON.stringify(settings);
  $: filteredKeybindings = localSettings.keybindings.filter(kb =>
    kb.command.toLowerCase().includes(searchQuery.toLowerCase()) ||
    kb.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
    kb.binding.toLowerCase().includes(searchQuery.toLowerCase())
  );
  $: groupedKeybindings = groupByCategory(filteredKeybindings);

  function groupByCategory(bindings: Keybinding[]): Record<string, Keybinding[]> {
    return bindings.reduce((acc, binding) => {
      if (!acc[binding.category]) {
        acc[binding.category] = [];
      }
      acc[binding.category].push(binding);
      return acc;
    }, {} as Record<string, Keybinding[]>);
  }

  function handlePresetChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    localSettings.preset = target.value;
    localSettings = localSettings;
  }

  function handleSearchInput(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;
  }

  function startEditBinding(command: string) {
    editingCommand = command;
    recordingKey = true;
    newBinding = '';
    conflictCommand = null;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!recordingKey) return;

    event.preventDefault();

    const modifiers = [];
    if (event.ctrlKey) modifiers.push('Ctrl');
    if (event.shiftKey) modifiers.push('Shift');
    if (event.altKey) modifiers.push('Alt');

    let key = event.key;
    if (key === ' ') key = 'Space';
    else if (key.length === 1) key = key.toUpperCase();

    if (key !== 'Control' && key !== 'Shift' && key !== 'Alt') {
      modifiers.push(key);
      newBinding = modifiers.join('+');

      // Check for conflicts
      const conflict = localSettings.keybindings.find(kb =>
        kb.binding === newBinding && kb.command !== editingCommand
      );
      conflictCommand = conflict ? conflict.description : null;
    }
  }

  function confirmBinding() {
    if (!editingCommand || !newBinding) return;

    const index = localSettings.keybindings.findIndex(kb => kb.command === editingCommand);
    if (index !== -1) {
      localSettings.keybindings[index].binding = newBinding;
      localSettings = localSettings;
    }

    cancelBinding();
  }

  function cancelBinding() {
    editingCommand = null;
    recordingKey = false;
    newBinding = '';
    conflictCommand = null;
  }

  function clearBinding(command: string) {
    const index = localSettings.keybindings.findIndex(kb => kb.command === command);
    if (index !== -1) {
      localSettings.keybindings[index].binding = '';
      localSettings = localSettings;
    }
  }

  function handleResetPreset() {
    // Would load preset keybindings in real implementation
    localSettings.preset = 'standard';
    localSettings = localSettings;
  }

  function handleExport() {
    modalAction = 'export';
    showModal = true;
  }

  function handleImport() {
    modalAction = 'import';
    showModal = true;
  }

  function handleModalConfirm(event: CustomEvent<string | null>) {
    if (modalAction === 'export') {
      // In real implementation, would export to JSON file
      console.log('Exporting keybindings:', localSettings.keybindings);
    } else if (modalAction === 'import') {
      // In real implementation, would import from JSON file
      console.log('Importing keybindings');
    }
    showModal = false;
    modalAction = null;
  }

  function handleModalCancel() {
    showModal = false;
    modalAction = null;
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
    localSettings.preset = 'standard';
    localSettings = localSettings;
  }

  function showSaveIndicator() {
    saveIndicator = true;
    setTimeout(() => {
      saveIndicator = false;
    }, 2000);
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="keyboard-settings">
  <div class="settings-content">
    <section class="settings-section">
      <h3 class="section-title">Keybinding Preset</h3>

      <div class="preset-row">
        <select
          class="setting-select preset-select"
          value={localSettings.preset}
          on:change={handlePresetChange}
        >
          {#each presets as preset}
            <option value={preset.value}>{preset.label}</option>
          {/each}
        </select>

        <div class="preset-actions">
          <button class="btn btn-secondary" on:click={handleResetPreset}>
            Reset to Preset
          </button>
          <button class="btn btn-secondary" on:click={handleExport}>
            Export
          </button>
          <button class="btn btn-secondary" on:click={handleImport}>
            Import
          </button>
        </div>
      </div>
    </section>

    <section class="settings-section">
      <h3 class="section-title">Keybindings</h3>

      <div class="search-box">
        <svg class="search-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="M21 21l-4.35-4.35"/>
        </svg>
        <input
          type="text"
          class="search-input"
          placeholder="Search commands..."
          value={searchQuery}
          on:input={handleSearchInput}
        />
      </div>

      <div class="keybindings-table">
        {#each Object.entries(groupedKeybindings) as [category, bindings]}
          <div class="category-group">
            <h4 class="category-title">{category}</h4>
            {#each bindings as keybinding}
              <div class="keybinding-row">
                <div class="keybinding-info">
                  <span class="command-name">{keybinding.description}</span>
                  <span class="command-id">{keybinding.command}</span>
                </div>
                <div class="keybinding-actions">
                  {#if editingCommand === keybinding.command}
                    <div class="recording-box">
                      <span class="recording-text">
                        {newBinding || 'Press keys...'}
                      </span>
                      {#if conflictCommand}
                        <span class="conflict-warning">
                          Conflicts with: {conflictCommand}
                        </span>
                      {/if}
                      <button class="btn btn-small btn-primary" on:click={confirmBinding}>
                        OK
                      </button>
                      <button class="btn btn-small btn-secondary" on:click={cancelBinding}>
                        Cancel
                      </button>
                    </div>
                  {:else}
                    <span class="binding-display">
                      {keybinding.binding || 'Not assigned'}
                    </span>
                    <button
                      class="btn btn-small btn-secondary"
                      on:click={() => startEditBinding(keybinding.command)}
                    >
                      Edit
                    </button>
                    <button
                      class="btn btn-small btn-secondary"
                      on:click={() => clearBinding(keybinding.command)}
                      disabled={!keybinding.binding}
                    >
                      Clear
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/each}
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
  <SettingsModal
    title={modalAction === 'export' ? 'Export Keybindings' : 'Import Keybindings'}
    message={modalAction === 'export'
      ? 'Keybindings will be exported to a JSON file in your downloads folder.'
      : 'Select a keybindings JSON file to import.'}
    confirmText={modalAction === 'export' ? 'Export' : 'Import'}
    on:confirm={handleModalConfirm}
    on:cancel={handleModalCancel}
  />
{/if}

<style>
  .keyboard-settings {
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

  .preset-row {
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .preset-select {
    flex: 1;
  }

  .preset-actions {
    display: flex;
    gap: 0.5rem;
  }

  .setting-select {
    padding: 0.75rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    color: var(--text-primary, #e0e0e0);
    font-size: 0.95rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .setting-select:focus {
    outline: none;
    border-color: var(--accent-color, #4a9eff);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .search-box {
    position: relative;
    margin-bottom: 1.5rem;
  }

  .search-icon {
    position: absolute;
    left: 1rem;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-secondary, #a0a0a0);
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 0.75rem 1rem 0.75rem 3rem;
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    color: var(--text-primary, #e0e0e0);
    font-size: 0.95rem;
    transition: all 0.2s;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color, #4a9eff);
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .keybindings-table {
    background: var(--bg-secondary, #1e1e1e);
    border: 1px solid var(--border-color, #333);
    border-radius: 8px;
    overflow: hidden;
  }

  .category-group {
    border-bottom: 1px solid var(--border-color, #333);
  }

  .category-group:last-child {
    border-bottom: none;
  }

  .category-title {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--accent-color, #4a9eff);
    margin: 0;
    padding: 0.75rem 1rem;
    background: rgba(74, 158, 255, 0.05);
    border-bottom: 1px solid var(--border-color, #333);
  }

  .keybinding-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border-color, #333);
    transition: background 0.2s;
  }

  .keybinding-row:last-child {
    border-bottom: none;
  }

  .keybinding-row:hover {
    background: var(--bg-hover, #2a2a2a);
  }

  .keybinding-info {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .command-name {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary, #e0e0e0);
  }

  .command-id {
    font-size: 0.8rem;
    color: var(--text-secondary, #a0a0a0);
    font-family: monospace;
  }

  .keybinding-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .binding-display {
    min-width: 120px;
    padding: 0.5rem 0.75rem;
    background: var(--bg-tertiary, #333);
    border: 1px solid var(--border-color, #444);
    border-radius: 4px;
    font-size: 0.9rem;
    font-family: monospace;
    color: var(--text-primary, #e0e0e0);
    text-align: center;
  }

  .recording-box {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    background: rgba(74, 158, 255, 0.1);
    border: 1px solid var(--accent-color, #4a9eff);
    border-radius: 6px;
  }

  .recording-text {
    min-width: 120px;
    padding: 0.5rem 0.75rem;
    background: var(--bg-primary, #252525);
    border-radius: 4px;
    font-size: 0.9rem;
    font-family: monospace;
    color: var(--accent-color, #4a9eff);
    text-align: center;
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  .conflict-warning {
    font-size: 0.85rem;
    color: var(--error-color, #ff4444);
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
</style>
