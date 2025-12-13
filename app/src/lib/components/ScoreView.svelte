<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  // Props
  export let midiData: Uint8Array | null = null;
  export let title: string = 'Untitled';
  export let width: number = 800;
  export let height: number = 400;
  export let showControls: boolean = true;
  
  // State
  let svgContent: string = '';
  let loading: boolean = false;
  let error: string | null = null;
  let zoom: number = 1;
  let scrollX: number = 0;
  let scrollY: number = 0;
  let isDragging: boolean = false;
  let lastMouseX: number = 0;
  let lastMouseY: number = 0;
  
  // Quantization options
  let quantizeLevel: 'whole' | 'half' | 'quarter' | 'eighth' | 'sixteenth' | '32nd' = 'sixteenth';
  
  // Render score when MIDI data changes
  $: if (midiData) {
    renderScore();
  }
  
  async function renderScore() {
    if (!midiData) return;
    
    loading = true;
    error = null;
    
    try {
      svgContent = await invoke('render_score_svg', {
        midiBytes: Array.from(midiData),
        title,
        width: Math.round(width * 2), // Higher resolution
        height: Math.round(height * 2)
      });
    } catch (e) {
      error = `Failed to render score: ${e}`;
      console.error(e);
    } finally {
      loading = false;
    }
  }
  
  async function exportMusicXML() {
    if (!midiData) return;
    
    try {
      const xml: string = await invoke('export_musicxml', {
        midiBytes: Array.from(midiData),
        title
      });
      
      // Download as file
      const blob = new Blob([xml], { type: 'application/xml' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${title.replace(/[^a-z0-9]/gi, '_')}.musicxml`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error = `Failed to export MusicXML: ${e}`;
    }
  }
  
  function handleZoomIn() {
    zoom = Math.min(zoom * 1.2, 4);
  }
  
  function handleZoomOut() {
    zoom = Math.max(zoom / 1.2, 0.25);
  }
  
  function handleZoomReset() {
    zoom = 1;
    scrollX = 0;
    scrollY = 0;
  }
  
  function handleMouseDown(e: MouseEvent) {
    isDragging = true;
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
  }
  
  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    
    const deltaX = e.clientX - lastMouseX;
    const deltaY = e.clientY - lastMouseY;
    
    scrollX += deltaX;
    scrollY += deltaY;
    
    lastMouseX = e.clientX;
    lastMouseY = e.clientY;
  }
  
  function handleMouseUp() {
    isDragging = false;
  }
  
  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey) {
      // Zoom with ctrl+wheel
      e.preventDefault();
      if (e.deltaY < 0) {
        handleZoomIn();
      } else {
        handleZoomOut();
      }
    } else {
      // Scroll
      scrollX -= e.deltaX;
      scrollY -= e.deltaY;
    }
  }
  
  onMount(() => {
    window.addEventListener('mouseup', handleMouseUp);
    return () => {
      window.removeEventListener('mouseup', handleMouseUp);
    };
  });
</script>

<div class="score-view" style="--width: {width}px; --height: {height}px;">
  {#if showControls}
    <div class="controls">
      <div class="control-group">
        <label>
          Quantize:
          <select bind:value={quantizeLevel} on:change={renderScore}>
            <option value="whole">Whole</option>
            <option value="half">Half</option>
            <option value="quarter">Quarter</option>
            <option value="eighth">8th</option>
            <option value="sixteenth">16th</option>
            <option value="32nd">32nd</option>
          </select>
        </label>
      </div>
      
      <div class="control-group">
        <button on:click={handleZoomOut} title="Zoom Out">−</button>
        <span class="zoom-level">{Math.round(zoom * 100)}%</span>
        <button on:click={handleZoomIn} title="Zoom In">+</button>
        <button on:click={handleZoomReset} title="Reset Zoom">⟲</button>
      </div>
      
      <div class="control-group">
        <button on:click={exportMusicXML} disabled={!midiData}>
          Export MusicXML
        </button>
      </div>
    </div>
  {/if}
  
  <div 
    class="score-container"
    on:mousedown={handleMouseDown}
    on:mousemove={handleMouseMove}
    on:wheel={handleWheel}
    role="img"
    aria-label="Musical score"
  >
    {#if loading}
      <div class="loading">
        <div class="spinner"></div>
        <span>Rendering notation...</span>
      </div>
    {:else if error}
      <div class="error">
        <span class="error-icon">⚠</span>
        <span>{error}</span>
      </div>
    {:else if svgContent}
      <div 
        class="score-svg"
        style="transform: scale({zoom}) translate({scrollX / zoom}px, {scrollY / zoom}px);"
      >
        {@html svgContent}
      </div>
    {:else}
      <div class="empty">
        <span>No MIDI data loaded</span>
        <span class="hint">Load a MIDI file to view notation</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .score-view {
    display: flex;
    flex-direction: column;
    width: var(--width);
    height: var(--height);
    background: var(--bg-secondary, #f5f5f5);
    border-radius: 8px;
    overflow: hidden;
  }
  
  .controls {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 8px 12px;
    background: var(--bg-primary, #ffffff);
    border-bottom: 1px solid var(--border-color, #e0e0e0);
  }
  
  .control-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  
  .control-group label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--text-secondary, #666);
  }
  
  .control-group select {
    padding: 4px 8px;
    border: 1px solid var(--border-color, #ccc);
    border-radius: 4px;
    background: var(--bg-primary, #fff);
    font-size: 13px;
  }
  
  .control-group button {
    padding: 4px 12px;
    border: 1px solid var(--border-color, #ccc);
    border-radius: 4px;
    background: var(--bg-primary, #fff);
    cursor: pointer;
    font-size: 13px;
    transition: background 0.2s;
  }
  
  .control-group button:hover {
    background: var(--bg-hover, #f0f0f0);
  }
  
  .control-group button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .zoom-level {
    min-width: 50px;
    text-align: center;
    font-size: 13px;
    color: var(--text-secondary, #666);
  }
  
  .score-container {
    flex: 1;
    overflow: hidden;
    cursor: grab;
    background: white;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .score-container:active {
    cursor: grabbing;
  }
  
  .score-svg {
    transform-origin: center center;
    transition: transform 0.1s ease-out;
  }
  
  .loading, .error, .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: var(--text-secondary, #666);
  }
  
  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border-color, #e0e0e0);
    border-top-color: var(--accent-color, #007bff);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .error {
    color: var(--error-color, #dc3545);
  }
  
  .error-icon {
    font-size: 24px;
  }
  
  .hint {
    font-size: 12px;
    opacity: 0.7;
  }
</style>
