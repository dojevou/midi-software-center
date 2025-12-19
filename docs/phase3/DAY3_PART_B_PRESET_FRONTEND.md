# Day 3, Part 3B: Preset Frontend

**Duration:** 2 hours
**Prerequisites:** Part 3A complete (preset backend)
**Files to create:** 3

---

## Overview

Build preset browser UI:
1. TypeScript preset types
2. Preset API client
3. PresetBrowser component
4. Save/load dialogs

---

## Step 1: TypeScript Types (15 min)

Create `app/src/lib/types/preset.ts`:

```typescript
export type PresetType = 'Track' | 'Mixer' | 'Effect' | 'ProjectTemplate';

export interface Preset {
  id: string;
  name: string;
  preset_type: PresetType;
  description?: string;
  author?: string;
  tags: string[];
  data: string; // JSON string
  created_at: string;
  updated_at: string;
}

export interface TrackPresetData {
  track_state: any; // TrackState from mixer types
  effect_chain: any; // EffectChain from mixer types
  automation_lanes: string[];
}

export interface MixerPresetData {
  tracks: Record<number, TrackPresetData>;
  master_gain_db: number;
  master_enabled: boolean;
}

export interface EffectPresetData {
  effect_type: string;
  name: string;
  parameters: Record<string, any>;
}

export interface ProjectTemplateData {
  name: string;
  bpm: number;
  time_signature: [number, number];
  mixer_preset: MixerPresetData;
  default_tracks: TrackPresetData[];
}

export class PresetUtils {
  static parsePresetData<T>(preset: Preset): T | null {
    try {
      return JSON.parse(preset.data) as T;
    } catch (error) {
      console.error('Failed to parse preset data:', error);
      return null;
    }
  }

  static createPresetData<T>(data: T): string {
    return JSON.stringify(data);
  }

  static getTypeIcon(type: PresetType): string {
    const icons: Record<PresetType, string> = {
      Track: '🎹',
      Mixer: '🎚️',
      Effect: '🎛️',
      ProjectTemplate: '📁',
    };
    return icons[type] || '📦';
  }

  static getTypeColor(type: PresetType): string {
    const colors: Record<PresetType, string> = {
      Track: '#22c55e',
      Mixer: '#3b82f6',
      Effect: '#f59e0b',
      ProjectTemplate: '#8b5cf6',
    };
    return colors[type] || '#6b7280';
  }
}
```

---

## Step 2: Preset API (15 min)

Create `app/src/lib/api/presetApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Preset, PresetType } from '../types/preset';

export class PresetApi {
  static async save(preset: Preset): Promise<void> {
    await invoke('save_preset', { preset });
  }

  static async load(presetId: string): Promise<Preset | null> {
    return await invoke<Preset | null>('load_preset', { presetId });
  }

  static async list(presetType: PresetType): Promise<Preset[]> {
    return await invoke<Preset[]>('list_presets', { presetType });
  }

  static async delete(presetId: string): Promise<void> {
    await invoke('delete_preset', { presetId });
  }

  static async search(query: string): Promise<Preset[]> {
    return await invoke<Preset[]>('search_presets', { query });
  }
}
```

---

## Step 3: Preset Browser Component (1 hour)

Create `app/src/lib/components/DAW/PresetBrowser.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { PresetApi } from '../../api/presetApi';
  import { PresetUtils } from '../../types/preset';
  import type { Preset, PresetType } from '../../types/preset';

  export let presetType: PresetType = 'Track';
  export let onLoad: ((preset: Preset) => void) | null = null;
  export let onSave: ((name: string, description: string, tags: string[]) => void) | null = null;

  let presets: Preset[] = [];
  let filteredPresets: Preset[] = [];
  let searchQuery = '';
  let selectedPreset: Preset | null = null;
  let showSaveDialog = false;

  // Save dialog state
  let saveName = '';
  let saveDescription = '';
  let saveTags = '';

  onMount(async () => {
    await loadPresets();
  });

  $: {
    if (searchQuery) {
      filteredPresets = presets.filter(
        (p) =>
          p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          p.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
          p.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()))
      );
    } else {
      filteredPresets = presets;
    }
  }

  async function loadPresets() {
    try {
      presets = await PresetApi.list(presetType);
      filteredPresets = presets;
    } catch (error) {
      console.error('Failed to load presets:', error);
    }
  }

  function handleSelectPreset(preset: Preset) {
    selectedPreset = preset;
  }

  async function handleLoadPreset() {
    if (!selectedPreset) return;

    try {
      if (onLoad) {
        onLoad(selectedPreset);
      }
    } catch (error) {
      console.error('Failed to load preset:', error);
    }
  }

  async function handleDeletePreset(preset: Preset) {
    if (!confirm(`Delete preset "${preset.name}"?`)) return;

    try {
      await PresetApi.delete(preset.id);
      await loadPresets();

      if (selectedPreset?.id === preset.id) {
        selectedPreset = null;
      }
    } catch (error) {
      console.error('Failed to delete preset:', error);
    }
  }

  function handleOpenSaveDialog() {
    showSaveDialog = true;
    saveName = '';
    saveDescription = '';
    saveTags = '';
  }

  async function handleSavePreset() {
    if (!saveName.trim()) {
      alert('Please enter a preset name');
      return;
    }

    const tags = saveTags
      .split(',')
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

    if (onSave) {
      try {
        onSave(saveName, saveDescription, tags);
        showSaveDialog = false;
        await loadPresets();
      } catch (error) {
        console.error('Failed to save preset:', error);
      }
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }
</script>

<div class="preset-browser">
  <div class="browser-header">
    <h3>
      {PresetUtils.getTypeIcon(presetType)} {presetType} Presets
    </h3>

    <div class="header-actions">
      {#if onSave}
        <button class="save-btn" on:click={handleOpenSaveDialog}> 💾 Save Preset </button>
      {/if}

      <button class="refresh-btn" on:click={loadPresets} title="Refresh"> ↻ </button>
    </div>
  </div>

  <div class="search-bar">
    <input
      type="text"
      placeholder="Search presets..."
      bind:value={searchQuery}
      class="search-input"
    />
  </div>

  <div class="preset-list">
    {#if filteredPresets.length === 0}
      <div class="empty-state">
        <p>No presets found.</p>
        {#if onSave}
          <p>Click "Save Preset" to create one.</p>
        {/if}
      </div>
    {:else}
      {#each filteredPresets as preset (preset.id)}
        <div
          class="preset-item"
          class:selected={selectedPreset?.id === preset.id}
          on:click={() => handleSelectPreset(preset)}
          on:dblclick={handleLoadPreset}
        >
          <div class="preset-info">
            <div class="preset-name">{preset.name}</div>
            {#if preset.description}
              <div class="preset-description">{preset.description}</div>
            {/if}
            {#if preset.tags.length > 0}
              <div class="preset-tags">
                {#each preset.tags as tag}
                  <span class="tag">{tag}</span>
                {/each}
              </div>
            {/if}
            <div class="preset-meta">
              {#if preset.author}
                <span>by {preset.author}</span>
                <span>•</span>
              {/if}
              <span>{formatDate(preset.created_at)}</span>
            </div>
          </div>

          <div class="preset-actions">
            <button
              class="delete-btn"
              on:click|stopPropagation={() => handleDeletePreset(preset)}
              title="Delete preset"
            >
              ✕
            </button>
          </div>
        </div>
      {/each}
    {/if}
  </div>

  {#if selectedPreset && onLoad}
    <div class="browser-footer">
      <button class="load-btn" on:click={handleLoadPreset}> Load "{selectedPreset.name}" </button>
    </div>
  {/if}
</div>

{#if showSaveDialog}
  <div class="modal-overlay" on:click={() => (showSaveDialog = false)}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Save {presetType} Preset</h3>
        <button class="close-btn" on:click={() => (showSaveDialog = false)}>✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="preset-name">Name *</label>
          <input
            id="preset-name"
            type="text"
            bind:value={saveName}
            placeholder="My Awesome Preset"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label for="preset-description">Description</label>
          <textarea
            id="preset-description"
            bind:value={saveDescription}
            placeholder="Optional description..."
            rows="3"
            class="form-textarea"
          />
        </div>

        <div class="form-group">
          <label for="preset-tags">Tags (comma-separated)</label>
          <input
            id="preset-tags"
            type="text"
            bind:value={saveTags}
            placeholder="piano, bright, reverb"
            class="form-input"
          />
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" on:click={() => (showSaveDialog = false)}> Cancel </button>
        <button class="confirm-btn" on:click={handleSavePreset} disabled={!saveName.trim()}>
          Save Preset
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .preset-browser {
    display: flex;
    flex-direction: column;
    background: #1a1a1a;
    border-radius: 8px;
    overflow: hidden;
    height: 100%;
  }

  .browser-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #161616;
    border-bottom: 1px solid #333;
  }

  .browser-header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .save-btn,
  .refresh-btn {
    padding: 6px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }

  .save-btn:hover,
  .refresh-btn:hover {
    background: #2563eb;
  }

  .search-bar {
    padding: 12px 16px;
    background: #161616;
    border-bottom: 1px solid #333;
  }

  .search-input {
    width: 100%;
    padding: 8px 12px;
    background: #0a0a0a;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 13px;
  }

  .preset-list {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .preset-item {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 12px;
    background: #252525;
    border: 1px solid #333;
    border-radius: 6px;
    margin-bottom: 8px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .preset-item:hover {
    background: #2a2a2a;
    border-color: #444;
  }

  .preset-item.selected {
    background: #1e3a5f;
    border-color: #3b82f6;
  }

  .preset-info {
    flex: 1;
  }

  .preset-name {
    font-size: 14px;
    font-weight: 600;
    color: #fff;
    margin-bottom: 4px;
  }

  .preset-description {
    font-size: 12px;
    color: #999;
    margin-bottom: 6px;
  }

  .preset-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 6px;
  }

  .tag {
    padding: 2px 8px;
    background: #333;
    color: #3b82f6;
    border-radius: 12px;
    font-size: 11px;
  }

  .preset-meta {
    font-size: 11px;
    color: #666;
  }

  .preset-meta span {
    margin-right: 4px;
  }

  .preset-actions {
    display: flex;
    gap: 4px;
  }

  .delete-btn {
    width: 24px;
    height: 24px;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .delete-btn:hover {
    background: #b91c1c;
  }

  .browser-footer {
    padding: 12px 16px;
    background: #161616;
    border-top: 1px solid #333;
  }

  .load-btn {
    width: 100%;
    padding: 10px;
    background: #22c55e;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .load-btn:hover {
    background: #16a34a;
  }

  .empty-state {
    text-align: center;
    padding: 48px 24px;
    color: #666;
  }

  .empty-state p {
    margin: 4px 0;
  }

  /* Modal styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #161616;
    border-bottom: 1px solid #333;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .close-btn {
    background: none;
    color: #999;
    border: none;
    font-size: 20px;
    cursor: pointer;
  }

  .close-btn:hover {
    color: #fff;
  }

  .modal-body {
    padding: 16px;
    overflow-y: auto;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    color: #999;
  }

  .form-input,
  .form-textarea {
    width: 100%;
    padding: 8px 12px;
    background: #0a0a0a;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 13px;
    font-family: inherit;
  }

  .form-textarea {
    resize: vertical;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px;
    background: #161616;
    border-top: 1px solid #333;
  }

  .cancel-btn,
  .confirm-btn {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }

  .cancel-btn {
    background: #333;
    color: #fff;
  }

  .cancel-btn:hover {
    background: #3a3a3a;
  }

  .confirm-btn {
    background: #22c55e;
    color: #fff;
  }

  .confirm-btn:hover {
    background: #16a34a;
  }

  .confirm-btn:disabled {
    background: #333;
    color: #666;
    cursor: not-allowed;
  }
</style>
```

---

## Step 4: Integration Example (20 min)

Example usage in a mixer panel:

```svelte
<!-- In MixerPanel.svelte or separate PresetsPanel.svelte -->
<script lang="ts">
  import PresetBrowser from './PresetBrowser.svelte';
  import { PresetApi } from '../../api/presetApi';
  import { PresetUtils } from '../../types/preset';
  import type { Preset, TrackPresetData } from '../../types/preset';

  async function handleLoadTrackPreset(preset: Preset) {
    const data = PresetUtils.parsePresetData<TrackPresetData>(preset);

    if (!data) {
      console.error('Failed to parse track preset');
      return;
    }

    // Apply track preset to current track
    console.log('Loading track preset:', data);

    // Set track state
    // await MixerApi.setTrackGain(data.track_state.track_id, data.track_state.gain_db);
    // await MixerApi.setTrackPan(data.track_state.track_id, data.track_state.pan);
    // etc.
  }

  async function handleSaveTrackPreset(name: string, description: string, tags: string[]) {
    // Get current track state
    const trackState = {}; // Get from mixer
    const effectChain = {}; // Get from effects

    const data: TrackPresetData = {
      track_state: trackState,
      effect_chain: effectChain,
      automation_lanes: [],
    };

    const preset: Preset = {
      id: `track-preset-${Date.now()}`,
      name,
      preset_type: 'Track',
      description,
      author: 'User',
      tags,
      data: PresetUtils.createPresetData(data),
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    await PresetApi.save(preset);
  }
</script>

<div class="presets-panel">
  <PresetBrowser
    presetType="Track"
    onLoad={handleLoadTrackPreset}
    onSave={handleSaveTrackPreset}
  />
</div>
```

---

## Verification (10 min)

```bash
npm run check
make dev
```

Test preset browser:
1. Open preset browser
2. Click "Save Preset" and create a new preset
3. Search for presets by name
4. Click preset to select
5. Double-click to load preset
6. Delete a preset
7. Verify tags display correctly
8. Test save dialog validation

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Presets not loading | Check API calls, verify database has presets |
| Search not working | Verify filter logic, check searchQuery reactivity |
| Save dialog not appearing | Check showSaveDialog state, verify modal z-index |
| Double-click not loading | Check onLoad callback, verify event handler |

---

## What's Next?

✅ **Day 3 Complete! You've built:**
- Complete preset system (backend + frontend)
- PresetBrowser with search and filtering
- Save/load dialogs with validation
- Tag support for organization
- 5 preset types (Track, Mixer, Effect, ProjectTemplate)

**Next:** [Day 4, Part 4A: Project Models](./DAY4_PART_A_PROJECT_MODELS.md)
- Project save/load system
- Session state serialization
- Project metadata
- Recent projects list
