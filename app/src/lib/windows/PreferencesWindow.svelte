<script lang="ts">
  import { onMount } from 'svelte';
  import {
    preferencesStore,
    preferencesActions,
    settingsCount,
    layoutCount,
    shortcutCount,
    recentProjectCount,
  } from '$lib/stores';
  import type {
    Setting,
    WindowLayoutConfig,
    KeyboardShortcut,
    RecentProject,
    WindowId,
  } from '$lib/types';

  // Window props
  export let windowId: WindowId;

  // Types
  type SectionId = 'settings' | 'layouts' | 'shortcuts' | 'recent';

  // State
  let activeSection: SectionId = 'settings';
  let selectedSettingCategory = 'general';
  let selectedShortcutCategory = 'global';
  let editingShortcut: KeyboardShortcut | null = null;
  // eslint-disable-next-line prefer-const -- reserved for future use
  let editingLayout: WindowLayoutConfig | null = null;
  let newLayoutName = '';
  let showNewLayoutModal = false;
  let recordingShortcut = false;
  let recordedKeys: string[] = [];

  // Sections config
  const sections: { id: SectionId; name: string; icon: string }[] = [
    { id: 'settings', name: 'App Settings', icon: '⚙️' },
    { id: 'layouts', name: 'Window Layouts', icon: '🪟' },
    { id: 'shortcuts', name: 'Keyboard Shortcuts', icon: '⌨️' },
    { id: 'recent', name: 'Recent Projects', icon: '📂' },
  ];

  const settingCategories = ['general', 'audio', 'midi', 'appearance', 'performance', 'advanced'];
  const shortcutCategories = ['global', 'transport', 'editing', 'navigation', 'window', 'tools'];

  onMount(async () => {
    await preferencesActions.initialize();
  });

  // Settings actions
  async function handleSettingChange(setting: Setting, value: string | number | boolean) {
    if (typeof value === 'string') {
      await preferencesActions.setStringSetting(setting.key, value);
    } else if (typeof value === 'number') {
      await preferencesActions.setFloatSetting(setting.key, value);
    } else if (typeof value === 'boolean') {
      await preferencesActions.setBoolSetting(setting.key, value);
    }
  }

  async function handleResetSetting(key: string) {
    await preferencesActions.resetSetting(key);
  }

  async function handleResetCategory(category: string) {
    if (confirm(`Reset all ${category} settings to defaults?`)) {
      await preferencesActions.resetCategory(category);
    }
  }

  // Layout actions
  async function saveCurrentLayout() {
    if (!newLayoutName.trim()) {
      return;
    }

    // Pass empty array of WindowLayoutEntry
    await preferencesActions.saveLayout(newLayoutName, []);
    showNewLayoutModal = false;
    newLayoutName = '';
  }

  async function applyLayout(layoutId: string) {
    await preferencesActions.applyLayout(layoutId);
  }

  async function deleteLayout(layoutId: string) {
    if (confirm('Delete this layout?')) {
      await preferencesActions.deleteLayout(layoutId);
    }
  }

  async function setDefaultLayout(layoutId: string) {
    await preferencesActions.setDefaultLayout(layoutId);
  }

  // Shortcut actions
  function startRecordingShortcut(shortcut: KeyboardShortcut) {
    editingShortcut = shortcut;
    recordingShortcut = true;
    recordedKeys = [];
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (!recordingShortcut) {
      return;
    }

    event.preventDefault();
    const keys: string[] = [];

    if (event.ctrlKey || event.metaKey) {
      keys.push('Ctrl');
    }
    if (event.altKey) {
      keys.push('Alt');
    }
    if (event.shiftKey) {
      keys.push('Shift');
    }

    const key = event.key;
    if (!['Control', 'Alt', 'Shift', 'Meta'].includes(key)) {
      keys.push(key.length === 1 ? key.toUpperCase() : key);
    }

    recordedKeys = keys;
  }

  async function saveShortcut() {
    if (!editingShortcut || recordedKeys.length === 0) {
      return;
    }

    const keyCombo = recordedKeys.join('+');
    await preferencesActions.setShortcut(editingShortcut.action, keyCombo);

    editingShortcut = null;
    recordingShortcut = false;
    recordedKeys = [];
  }

  function cancelRecording() {
    editingShortcut = null;
    recordingShortcut = false;
    recordedKeys = [];
  }

  async function resetShortcut(actionId: string) {
    await preferencesActions.resetShortcut(actionId);
  }

  async function resetAllShortcuts() {
    if (confirm('Reset all keyboard shortcuts to defaults?')) {
      await preferencesActions.resetAllShortcuts();
    }
  }

  // Recent projects actions
  async function openProject(project: RecentProject) {
    // This would typically trigger project loading
    console.log('Opening project:', project.file_path);
  }

  async function togglePinProject(projectPath: string, pinned: boolean) {
    await preferencesActions.setPinned(projectPath, pinned);
  }

  async function removeProject(projectPath: string) {
    await preferencesActions.removeRecentProject(projectPath);
  }

  async function clearAllRecent() {
    if (confirm('Clear all recent projects? Pinned projects will also be removed.')) {
      await preferencesActions.clearAllRecentProjects();
    }
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return 'Today';
    }
    if (diffDays === 1) {
      return 'Yesterday';
    }
    if (diffDays < 7) {
      return `${diffDays} days ago`;
    }
    return date.toLocaleDateString();
  }

  // Reactive
  $: settingsByCategory = $preferencesStore.settingsByCategory;
  $: currentSettings = settingsByCategory.get(selectedSettingCategory) || [];
  // shortcuts variable reserved for external API access
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  $: shortcuts = $preferencesStore.shortcuts;
  $: shortcutsByCategory = $preferencesStore.shortcutsByCategory;
  $: currentShortcuts = shortcutsByCategory.get(selectedShortcutCategory) || [];
  $: layouts = $preferencesStore.layouts;
  $: currentLayout = $preferencesStore.currentLayout;
  $: recentProjects = $preferencesStore.recentProjects;
  $: pinnedProjects = $preferencesStore.pinnedProjects;
  $: isLoading = $preferencesStore.isLoading;
  $: isSaving = $preferencesStore.isSaving;
</script>

<svelte:window on:keydown={handleKeyDown} />

<div class="preferences-window dark:bg-window dark:text-app-text h-full flex">
  <!-- Sidebar -->
  <div class="sidebar w-56 border-r dark:border-window-border flex flex-col">
    <div class="p-4 border-b dark:border-window-border">
      <h2 class="text-xl font-semibold dark:text-gray-200">Preferences</h2>
    </div>

    <nav class="flex-1 p-2">
      <ul class="space-y-1">
        {#each sections as section (section.id)}
          <li>
            <button
              on:click={() => (activeSection = section.id)}
              class="w-full text-left px-3 py-2 rounded flex items-center gap-2 transition-colors"
              class:dark:bg-primary={activeSection === section.id}
              class:dark:text-white={activeSection === section.id}
              class:dark:text-gray-300={activeSection !== section.id}
              class:hover:dark:bg-window-subtle={activeSection !== section.id}
            >
              <span>{section.icon}</span>
              <span class="flex-1">{section.name}</span>
              <span class="text-xs dark:text-gray-500">
                {#if section.id === 'settings'}{$settingsCount}
                {:else if section.id === 'layouts'}{$layoutCount}
                {:else if section.id === 'shortcuts'}{$shortcutCount}
                {:else if section.id === 'recent'}{$recentProjectCount}
                {/if}
              </span>
            </button>
          </li>
        {/each}
      </ul>
    </nav>
  </div>

  <!-- Main Content -->
  <div class="main-content flex-1 overflow-auto">
    {#if isLoading}
      <div class="loading h-full flex items-center justify-center">
        <div class="text-center">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 dark:border-gray-400 mb-2"></div>
          <div class="dark:text-gray-500">Loading preferences...</div>
        </div>
      </div>
    {:else if activeSection === 'settings'}
      <!-- App Settings -->
      <div class="p-6">
        <div class="flex justify-between items-center mb-6">
          <h3 class="text-lg font-medium dark:text-gray-200">Application Settings</h3>
          <button
            on:click={() => handleResetCategory(selectedSettingCategory)}
            class="px-3 py-1 text-sm dark:bg-secondary rounded hover:opacity-80"
          >
            Reset Category
          </button>
        </div>

        <!-- Category Tabs -->
        <div class="category-tabs flex gap-2 mb-6 border-b dark:border-window-border pb-4">
          {#each settingCategories as category (category)}
            <button
              on:click={() => (selectedSettingCategory = category)}
              class="px-4 py-2 rounded transition-colors capitalize"
              class:dark:bg-primary={selectedSettingCategory === category}
              class:dark:text-white={selectedSettingCategory === category}
              class:dark:bg-window-subtle={selectedSettingCategory !== category}
              class:dark:text-gray-300={selectedSettingCategory !== category}
            >
              {category}
            </button>
          {/each}
        </div>

        <!-- Settings List -->
        <div class="settings-list space-y-4">
          {#each currentSettings as setting (setting.key)}
            <div class="setting-item p-4 dark:bg-window-subtle rounded border dark:border-window-border">
              <div class="flex justify-between items-start mb-2">
                <div>
                  <h4 class="font-medium dark:text-gray-200">{setting.key.replace(/_/g, ' ')}</h4>
                  {#if setting.description}
                    <p class="text-sm dark:text-gray-400">{setting.description}</p>
                  {/if}
                </div>
                <button
                  on:click={() => handleResetSetting(setting.key)}
                  class="text-xs dark:text-gray-500 hover:dark:text-gray-300"
                  title="Reset to default"
                >
                  ↺
                </button>
              </div>

              <div class="setting-control mt-2">
                {#if setting.setting_type === 'boolean'}
                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={setting.value === 'true'}
                      on:change={(e) => handleSettingChange(setting, e.currentTarget.checked)}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-300">
                      {setting.value === 'true' ? 'Enabled' : 'Disabled'}
                    </span>
                  </label>
                {:else if setting.setting_type === 'integer' || setting.setting_type === 'float'}
                  <input
                    type="number"
                    value={parseFloat(setting.value) || 0}
                    on:change={(e) => handleSettingChange(setting, parseFloat(e.currentTarget.value))}
                    class="w-32 px-3 py-1 dark:bg-input dark:border-window-border rounded"
                  />
                {:else}
                  <input
                    type="text"
                    value={setting.value || ''}
                    on:change={(e) => handleSettingChange(setting, e.currentTarget.value)}
                    class="w-full px-3 py-1 dark:bg-input dark:border-window-border rounded"
                  />
                {/if}
              </div>
            </div>
          {/each}

          {#if currentSettings.length === 0}
            <div class="text-center py-8 dark:text-gray-500">
              No settings in this category
            </div>
          {/if}
        </div>
      </div>

    {:else if activeSection === 'layouts'}
      <!-- Window Layouts -->
      <div class="p-6">
        <div class="flex justify-between items-center mb-6">
          <h3 class="text-lg font-medium dark:text-gray-200">Window Layouts</h3>
          <button
            on:click={() => (showNewLayoutModal = true)}
            class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
          >
            + Save Current Layout
          </button>
        </div>

        <div class="layouts-grid grid grid-cols-2 gap-4">
          {#each layouts as layout (layout.id)}
            <div
              class="layout-card p-4 dark:bg-window-subtle rounded border dark:border-window-border"
              class:border-primary={currentLayout?.id === layout.id}
            >
              <div class="flex justify-between items-start mb-2">
                <div>
                  <h4 class="font-medium dark:text-gray-200">{layout.name}</h4>
                  <p class="text-xs dark:text-gray-400">
                    {layout.windows?.length || 0} windows
                    {#if layout.is_default}
                      <span class="ml-2 px-1 dark:bg-primary text-white rounded text-xs">Default</span>
                    {/if}
                  </p>
                </div>
                <div class="flex gap-1">
                  {#if !layout.is_default}
                    <button
                      on:click={() => setDefaultLayout(layout.name)}
                      class="p-1 text-xs dark:text-gray-400 hover:dark:text-gray-200"
                      title="Set as default"
                    >
                      ⭐
                    </button>
                  {/if}
                  <button
                    on:click={() => deleteLayout(layout.name)}
                    class="p-1 text-xs dark:text-gray-400 hover:dark:text-error"
                    title="Delete"
                  >
                    🗑
                  </button>
                </div>
              </div>

              <div class="flex gap-2 mt-3">
                <button
                  on:click={() => applyLayout(layout.name)}
                  class="flex-1 px-3 py-1 text-sm dark:bg-secondary rounded hover:opacity-80"
                  disabled={isSaving}
                >
                  Apply
                </button>
              </div>
            </div>
          {/each}

          {#if layouts.length === 0}
            <div class="col-span-2 text-center py-12 dark:text-gray-500">
              <div class="text-4xl mb-2">🪟</div>
              <p>No saved layouts yet</p>
              <p class="text-sm">Save your current window arrangement to quickly restore it later</p>
            </div>
          {/if}
        </div>
      </div>

    {:else if activeSection === 'shortcuts'}
      <!-- Keyboard Shortcuts -->
      <div class="p-6">
        <div class="flex justify-between items-center mb-6">
          <h3 class="text-lg font-medium dark:text-gray-200">Keyboard Shortcuts</h3>
          <button
            on:click={resetAllShortcuts}
            class="px-3 py-1 text-sm dark:bg-secondary rounded hover:opacity-80"
          >
            Reset All
          </button>
        </div>

        <!-- Category Tabs -->
        <div class="category-tabs flex gap-2 mb-6 border-b dark:border-window-border pb-4">
          {#each shortcutCategories as category (category)}
            <button
              on:click={() => (selectedShortcutCategory = category)}
              class="px-4 py-2 rounded transition-colors capitalize"
              class:dark:bg-primary={selectedShortcutCategory === category}
              class:dark:text-white={selectedShortcutCategory === category}
              class:dark:bg-window-subtle={selectedShortcutCategory !== category}
              class:dark:text-gray-300={selectedShortcutCategory !== category}
            >
              {category}
            </button>
          {/each}
        </div>

        <!-- Shortcuts List -->
        <div class="shortcuts-list">
          <table class="w-full">
            <thead>
              <tr class="border-b dark:border-window-border">
                <th class="text-left p-2 dark:text-gray-400">Action</th>
                <th class="text-left p-2 dark:text-gray-400">Shortcut</th>
                <th class="text-right p-2 dark:text-gray-400">Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each currentShortcuts as shortcut (shortcut.id)}
                <tr class="border-b dark:border-window-border hover:dark:bg-window-subtle">
                  <td class="p-2">
                    <div class="dark:text-gray-200">{shortcut.action.replace(/_/g, ' ')}</div>
                    {#if shortcut.description}
                      <div class="text-xs dark:text-gray-400">{shortcut.description}</div>
                    {/if}
                  </td>
                  <td class="p-2">
                    {#if editingShortcut?.id === shortcut.id && recordingShortcut}
                      <div class="flex items-center gap-2">
                        <span class="px-3 py-1 dark:bg-primary text-white rounded animate-pulse">
                          {recordedKeys.length > 0 ? recordedKeys.join('+') : 'Press keys...'}
                        </span>
                        <button
                          on:click={saveShortcut}
                          class="px-2 py-1 text-xs dark:bg-success rounded"
                          disabled={recordedKeys.length === 0}
                        >
                          ✓
                        </button>
                        <button
                          on:click={cancelRecording}
                          class="px-2 py-1 text-xs dark:bg-error rounded"
                        >
                          ✕
                        </button>
                      </div>
                    {:else}
                      <button
                        on:click={() => startRecordingShortcut(shortcut)}
                        class="px-3 py-1 dark:bg-input dark:border-window-border rounded text-sm"
                      >
                        {shortcut.key_combo || 'Unassigned'}
                      </button>
                    {/if}
                  </td>
                  <td class="p-2 text-right">
                    <button
                      on:click={() => resetShortcut(shortcut.action)}
                      class="text-xs dark:text-gray-500 hover:dark:text-gray-300"
                      title="Reset to default"
                    >
                      ↺
                    </button>
                  </td>
                </tr>
              {/each}

              {#if currentShortcuts.length === 0}
                <tr>
                  <td colspan="3" class="text-center py-8 dark:text-gray-500">
                    No shortcuts in this category
                  </td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>
      </div>

    {:else if activeSection === 'recent'}
      <!-- Recent Projects -->
      <div class="p-6">
        <div class="flex justify-between items-center mb-6">
          <h3 class="text-lg font-medium dark:text-gray-200">Recent Projects</h3>
          <button
            on:click={clearAllRecent}
            class="px-3 py-1 text-sm dark:bg-secondary rounded hover:opacity-80"
          >
            Clear All
          </button>
        </div>

        <!-- Pinned Projects -->
        {#if pinnedProjects.length > 0}
          <div class="pinned-section mb-6">
            <h4 class="text-sm font-medium dark:text-gray-400 mb-3">📌 Pinned</h4>
            <div class="space-y-2">
              {#each pinnedProjects as project (project.file_path)}
                <div class="project-item p-3 dark:bg-window-subtle rounded border dark:border-window-border hover:dark:bg-menu transition-colors">
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3 flex-1 min-w-0">
                      <span class="text-2xl">📁</span>
                      <div class="flex-1 min-w-0">
                        <h5 class="font-medium dark:text-gray-200 truncate">{project.name}</h5>
                        <p class="text-xs dark:text-gray-400 truncate">{project.file_path}</p>
                        <p class="text-xs dark:text-gray-500">
                          Last opened: {formatDate(project.last_opened)}
                        </p>
                      </div>
                    </div>
                    <div class="flex gap-2">
                      <button
                        on:click={() => togglePinProject(project.file_path, false)}
                        class="p-1 dark:text-yellow-400 hover:opacity-80"
                        title="Unpin"
                      >
                        📌
                      </button>
                      <button
                        on:click={() => openProject(project)}
                        class="px-3 py-1 dark:bg-primary dark:text-white rounded text-sm hover:opacity-80"
                      >
                        Open
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Recent Projects -->
        <div class="recent-section">
          <h4 class="text-sm font-medium dark:text-gray-400 mb-3">🕐 Recent</h4>
          <div class="space-y-2">
            {#each recentProjects.filter(p => !p.is_pinned) as project (project.file_path)}
              <div class="project-item p-3 dark:bg-window-subtle rounded border dark:border-window-border hover:dark:bg-menu transition-colors">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3 flex-1 min-w-0">
                    <span class="text-2xl">📁</span>
                    <div class="flex-1 min-w-0">
                      <h5 class="font-medium dark:text-gray-200 truncate">{project.name}</h5>
                      <p class="text-xs dark:text-gray-400 truncate">{project.file_path}</p>
                      <p class="text-xs dark:text-gray-500">
                        Last opened: {formatDate(project.last_opened)}
                      </p>
                    </div>
                  </div>
                  <div class="flex gap-2">
                    <button
                      on:click={() => togglePinProject(project.file_path, true)}
                      class="p-1 dark:text-gray-400 hover:dark:text-yellow-400"
                      title="Pin"
                    >
                      📍
                    </button>
                    <button
                      on:click={() => removeProject(project.file_path)}
                      class="p-1 dark:text-gray-400 hover:dark:text-error"
                      title="Remove"
                    >
                      ✕
                    </button>
                    <button
                      on:click={() => openProject(project)}
                      class="px-3 py-1 dark:bg-primary dark:text-white rounded text-sm hover:opacity-80"
                    >
                      Open
                    </button>
                  </div>
                </div>
              </div>
            {/each}

            {#if recentProjects.filter(p => !p.is_pinned).length === 0}
              <div class="text-center py-8 dark:text-gray-500">
                <div class="text-4xl mb-2">📂</div>
                <p>No recent projects</p>
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- New Layout Modal -->
{#if showNewLayoutModal}
  <div class="modal-overlay fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="modal dark:bg-window p-6 rounded-lg shadow-xl w-96">
      <h3 class="text-lg font-medium dark:text-gray-200 mb-4">Save Current Layout</h3>

      <div class="mb-4">
        <label class="block text-sm dark:text-gray-400 mb-2">Layout Name</label>
        <input
          type="text"
          bind:value={newLayoutName}
          placeholder="My Layout"
          class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
        />
      </div>

      <div class="flex justify-end gap-2">
        <button
          on:click={() => (showNewLayoutModal = false)}
          class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
        >
          Cancel
        </button>
        <button
          on:click={saveCurrentLayout}
          disabled={!newLayoutName.trim() || isSaving}
          class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80 disabled:opacity-50"
        >
          {isSaving ? 'Saving...' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .preferences-window {
    min-height: 500px;
  }

  .sidebar {
    background: linear-gradient(to bottom, var(--window-bg), var(--window-subtle));
  }

  .modal-overlay {
    backdrop-filter: blur(2px);
  }
</style>
