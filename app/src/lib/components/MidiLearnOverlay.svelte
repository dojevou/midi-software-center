<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  
  const dispatch = createEventDispatcher();
  
  // Props
  export let targetPath: string = '';
  export let targetName: string = 'Parameter';
  export let visible: boolean = false;
  
  // State
  let isLearning: boolean = false;
  let lastMidiMessage: { device: string; cc: number; channel: number; value: number } | null = null;
  let mappings: MidiMapping[] = [];
  let selectedMapping: string | null = null;
  
  // Types
  interface MidiMapping {
    id: string;
    name: string;
    enabled: boolean;
    source: {
      device_id: string;
      message_type: string;
      channel: number | null;
      data1: number | null;
    };
    target: {
      Parameter?: { path: string };
      Transport?: { action: string };
    };
    scaling: {
      Linear?: { min: number; max: number };
      Toggle?: null;
      Relative?: { sensitivity: number };
    };
  }
  
  let unlistenMidi: UnlistenFn | null = null;
  let unlistenLearn: UnlistenFn | null = null;
  
  onMount(async () => {
    // Listen for MIDI input events
    unlistenMidi = await listen('midi_input', (event: any) => {
      const { device, channel, controller, value } = event.payload;
      lastMidiMessage = { device, cc: controller, channel, value };
    });
    
    // Listen for learn completion
    unlistenLearn = await listen('learn_completed', (event: any) => {
      const mapping = event.payload as MidiMapping;
      mappings = [...mappings, mapping];
      isLearning = false;
      dispatch('mapped', mapping);
    });
    
    // Load existing mappings
    await loadMappings();
  });
  
  onDestroy(() => {
    unlistenMidi?.();
    unlistenLearn?.();
  });
  
  async function loadMappings() {
    try {
      mappings = await invoke('learn_list_mappings');
    } catch (e) {
      console.error('Failed to load mappings:', e);
    }
  }
  
  async function startLearning() {
    isLearning = true;
    lastMidiMessage = null;
    
    try {
      await invoke('learn_start', { targetPath });
    } catch (e) {
      console.error('Failed to start learning:', e);
      isLearning = false;
    }
  }
  
  async function cancelLearning() {
    isLearning = false;
    
    try {
      await invoke('learn_cancel');
    } catch (e) {
      console.error('Failed to cancel learning:', e);
    }
  }
  
  async function removeMapping(id: string) {
    try {
      await invoke('learn_remove_mapping', { mappingId: id });
      mappings = mappings.filter(m => m.id !== id);
      
      if (selectedMapping === id) {
        selectedMapping = null;
      }
    } catch (e) {
      console.error('Failed to remove mapping:', e);
    }
  }
  
  async function exportMappings() {
    try {
      const json = await invoke('learn_export_mappings') as string;
      
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'midi_mappings.json';
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error('Failed to export mappings:', e);
    }
  }
  
  async function importMappings() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      
      const json = await file.text();
      
      try {
        const count = await invoke('learn_import_mappings', { json }) as number;
        await loadMappings();
        alert(`Imported ${count} mappings`);
      } catch (e) {
        console.error('Failed to import mappings:', e);
        alert('Failed to import mappings');
      }
    };
    
    input.click();
  }
  
  function formatMidiSource(source: MidiMapping['source']): string {
    const type = source.message_type.replace('_', ' ');
    const ch = source.channel !== null ? `Ch ${source.channel + 1}` : 'Any Ch';
    const num = source.data1 !== null ? `#${source.data1}` : '';
    return `${type} ${ch} ${num}`.trim();
  }
  
  function close() {
    if (isLearning) {
      cancelLearning();
    }
    visible = false;
    dispatch('close');
  }
</script>

{#if visible}
  <div class="overlay" on:click|self={close}>
    <div class="panel">
      <div class="header">
        <h2>MIDI Learn</h2>
        <button class="close-btn" on:click={close}>×</button>
      </div>
      
      <div class="content">
        <!-- Current Target -->
        <div class="section">
          <h3>Target Parameter</h3>
          <div class="target-info">
            <span class="target-name">{targetName}</span>
            <span class="target-path">{targetPath}</span>
          </div>
        </div>
        
        <!-- Learn Mode -->
        <div class="section">
          <h3>Learn</h3>
          
          {#if isLearning}
            <div class="learning-active">
              <div class="pulse-ring"></div>
              <span class="learning-text">Move a MIDI controller...</span>
              
              {#if lastMidiMessage}
                <div class="detected">
                  <span>Detected: CC {lastMidiMessage.cc} on Ch {lastMidiMessage.channel + 1}</span>
                  <span class="value">Value: {lastMidiMessage.value}</span>
                </div>
              {/if}
              
              <button class="cancel-btn" on:click={cancelLearning}>Cancel</button>
            </div>
          {:else}
            <button class="learn-btn" on:click={startLearning}>
              <span class="icon">🎛</span>
              Start Learning
            </button>
          {/if}
        </div>
        
        <!-- Existing Mappings -->
        <div class="section">
          <div class="section-header">
            <h3>Mappings for this parameter</h3>
          </div>
          
          {#if mappings.filter(m => m.target.Parameter?.path === targetPath).length > 0}
            <div class="mappings-list">
              {#each mappings.filter(m => m.target.Parameter?.path === targetPath) as mapping}
                <div 
                  class="mapping-item"
                  class:selected={selectedMapping === mapping.id}
                  on:click={() => selectedMapping = mapping.id}
                >
                  <div class="mapping-info">
                    <span class="mapping-source">{formatMidiSource(mapping.source)}</span>
                    <span class="mapping-device">{mapping.source.device_id}</span>
                  </div>
                  
                  <div class="mapping-actions">
                    <button 
                      class="remove-btn"
                      on:click|stopPropagation={() => removeMapping(mapping.id)}
                      title="Remove mapping"
                    >
                      🗑
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="no-mappings">
              No mappings for this parameter
            </div>
          {/if}
        </div>
        
        <!-- All Mappings -->
        <div class="section">
          <div class="section-header">
            <h3>All Mappings ({mappings.length})</h3>
            <div class="section-actions">
              <button on:click={importMappings}>Import</button>
              <button on:click={exportMappings}>Export</button>
            </div>
          </div>
          
          {#if mappings.length > 0}
            <div class="mappings-list all-mappings">
              {#each mappings as mapping}
                <div class="mapping-item-compact">
                  <span class="source">{formatMidiSource(mapping.source)}</span>
                  <span class="arrow">→</span>
                  <span class="target">
                    {mapping.target.Parameter?.path || mapping.target.Transport?.action || 'Unknown'}
                  </span>
                  <button 
                    class="remove-btn-small"
                    on:click={() => removeMapping(mapping.id)}
                  >
                    ×
                  </button>
                </div>
              {/each}
            </div>
          {:else}
            <div class="no-mappings">
              No MIDI mappings configured
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  
  .panel {
    background: var(--bg-primary, #ffffff);
    border-radius: 12px;
    width: 480px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }
  
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }
  
  .header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: var(--text-secondary, #666);
    padding: 0;
    line-height: 1;
  }
  
  .content {
    padding: 20px;
    overflow-y: auto;
  }
  
  .section {
    margin-bottom: 24px;
  }
  
  .section:last-child {
    margin-bottom: 0;
  }
  
  .section h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-secondary, #666);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  
  .section-header h3 {
    margin: 0;
  }
  
  .section-actions {
    display: flex;
    gap: 8px;
  }
  
  .section-actions button {
    padding: 4px 12px;
    font-size: 12px;
    border: 1px solid var(--border-color, #ccc);
    background: var(--bg-secondary, #f5f5f5);
    border-radius: 4px;
    cursor: pointer;
  }
  
  .target-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    background: var(--bg-secondary, #f5f5f5);
    border-radius: 8px;
  }
  
  .target-name {
    font-weight: 600;
    font-size: 16px;
  }
  
  .target-path {
    font-size: 12px;
    color: var(--text-secondary, #666);
    font-family: monospace;
  }
  
  .learn-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    padding: 16px;
    font-size: 16px;
    font-weight: 600;
    background: var(--accent-color, #007bff);
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .learn-btn:hover {
    background: var(--accent-hover, #0056b3);
  }
  
  .learning-active {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 24px;
    background: var(--bg-secondary, #f5f5f5);
    border-radius: 8px;
    text-align: center;
  }
  
  .pulse-ring {
    width: 48px;
    height: 48px;
    border: 3px solid var(--accent-color, #007bff);
    border-radius: 50%;
    animation: pulse 1.5s ease-out infinite;
  }
  
  @keyframes pulse {
    0% {
      transform: scale(0.8);
      opacity: 1;
    }
    100% {
      transform: scale(1.5);
      opacity: 0;
    }
  }
  
  .learning-text {
    font-size: 16px;
    font-weight: 500;
  }
  
  .detected {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 16px;
    background: var(--success-bg, #d4edda);
    border-radius: 6px;
    color: var(--success-color, #155724);
  }
  
  .detected .value {
    font-size: 12px;
    opacity: 0.8;
  }
  
  .cancel-btn {
    padding: 8px 24px;
    background: var(--bg-primary, #fff);
    border: 1px solid var(--border-color, #ccc);
    border-radius: 6px;
    cursor: pointer;
  }
  
  .mappings-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  
  .mapping-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px;
    background: var(--bg-secondary, #f5f5f5);
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .mapping-item:hover {
    background: var(--bg-hover, #e8e8e8);
  }
  
  .mapping-item.selected {
    background: var(--accent-bg, #e3f2fd);
    border: 1px solid var(--accent-color, #007bff);
  }
  
  .mapping-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  
  .mapping-source {
    font-weight: 500;
  }
  
  .mapping-device {
    font-size: 12px;
    color: var(--text-secondary, #666);
  }
  
  .remove-btn {
    background: none;
    border: none;
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    opacity: 0.6;
    transition: opacity 0.2s;
  }
  
  .remove-btn:hover {
    opacity: 1;
  }
  
  .all-mappings {
    max-height: 200px;
    overflow-y: auto;
  }
  
  .mapping-item-compact {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--bg-secondary, #f5f5f5);
    border-radius: 6px;
    font-size: 13px;
  }
  
  .mapping-item-compact .source {
    font-weight: 500;
    white-space: nowrap;
  }
  
  .mapping-item-compact .arrow {
    color: var(--text-secondary, #666);
  }
  
  .mapping-item-compact .target {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: monospace;
    font-size: 12px;
    color: var(--text-secondary, #666);
  }
  
  .remove-btn-small {
    background: none;
    border: none;
    font-size: 16px;
    cursor: pointer;
    color: var(--text-secondary, #999);
    padding: 0 4px;
  }
  
  .remove-btn-small:hover {
    color: var(--error-color, #dc3545);
  }
  
  .no-mappings {
    padding: 16px;
    text-align: center;
    color: var(--text-secondary, #666);
    font-style: italic;
  }
</style>
