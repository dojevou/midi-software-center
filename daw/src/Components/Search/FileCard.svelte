<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let file: {
    id: number;
    file_name: string;
    file_path: string;
    category: string | null;
    bpm: number | null;
    key_signature: string | null;
    time_signature: string | null;
    duration_seconds: number;
    instruments: string[];
    created_at: string;
  };

  export let viewMode: 'grid' | 'list' = 'grid';

  const dispatch = createEventDispatcher();

  function handleSelect() {
    dispatch('select', { fileId: file.id });
  }

  function handlePlay(event: MouseEvent) {
    event.stopPropagation();
    dispatch('play', { fileId: file.id });
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function getCategoryIcon(category: string | null): string {
    const icons: Record<string, string> = {
      'kick': '🥁',
      'snare': '🥁',
      'hihat': '🎩',
      'drum': '🥁',
      'bass': '🎸',
      'chord': '🎹',
      'lead': '🎺',
      'pad': '🌊',
      'melody': '🎵',
      'fx': '✨'
    };

    if (!category) return '📄';

    const lower = category.toLowerCase();
    for (const [key, icon] of Object.entries(icons)) {
      if (lower.includes(key)) return icon;
    }

    return '📄';
  }
</script>

<div 
  class="file-card" 
  class:grid-mode={viewMode === 'grid'}
  class:list-mode={viewMode === 'list'}
  on:click={handleSelect}
  on:keypress={(e) => e.key === 'Enter' && handleSelect()}
  role="button"
  tabindex="0"
>
  <div class="card-header">
    <span class="category-icon">{getCategoryIcon(file.category)}</span>
    <span class="file-name" title={file.file_name}>{file.file_name}</span>
  </div>

  <div class="card-body">
    <div class="metadata">
      {#if file.bpm}
        <div class="meta-item">
          <span class="meta-label">BPM</span>
          <span class="meta-value">{file.bpm}</span>
        </div>
      {/if}

      {#if file.key_signature}
        <div class="meta-item">
          <span class="meta-label">Key</span>
          <span class="meta-value">{file.key_signature}</span>
        </div>
      {/if}

      {#if file.time_signature}
        <div class="meta-item">
          <span class="meta-label">Time</span>
          <span class="meta-value">{file.time_signature}</span>
        </div>
      {/if}

      <div class="meta-item">
        <span class="meta-label">Duration</span>
        <span class="meta-value">{formatDuration(file.duration_seconds ?? 0)}</span>
      </div>
    </div>

    {#if file.instruments && file.instruments.length > 0}
      <div class="instruments">
        {#each file.instruments.slice(0, 3) as instrument}
          <span class="instrument-tag">{instrument}</span>
        {/each}
        {#if file.instruments.length > 3}
          <span class="instrument-tag more">+{file.instruments.length - 3}</span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="card-footer">
    {#if file.category}
      <span class="category-badge">{file.category}</span>
    {/if}

    <button 
      class="play-btn"
      on:click={handlePlay}
      title="Play file"
      aria-label="Play MIDI file"
    >
      ▶
    </button>
  </div>
</div>

<style>
  .file-card {
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
    padding: 16px;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .file-card:hover {
    background: #353535;
    border-color: #4a9eff;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(74, 158, 255, 0.2);
  }

  .file-card:focus {
    outline: 2px solid #4a9eff;
    outline-offset: 2px;
  }

  /* List mode specific styles */
  .file-card.list-mode {
    flex-direction: row;
    align-items: center;
    padding: 12px 16px;
  }

  .file-card.list-mode .card-header {
    flex: 1;
    min-width: 0;
  }

  .file-card.list-mode .card-body {
    flex: 2;
    flex-direction: row;
    align-items: center;
  }

  .file-card.list-mode .metadata {
    flex-direction: row;
    gap: 16px;
  }

  .file-card.list-mode .card-footer {
    margin-left: auto;
  }

  .card-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .category-icon {
    font-size: 24px;
    flex-shrink: 0;
  }

  .file-name {
    flex: 1;
    font-size: 14px;
    font-weight: 600;
    color: #e0e0e0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .metadata {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .meta-label {
    font-size: 10px;
    color: #808080;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .meta-value {
    font-size: 13px;
    color: #e0e0e0;
    font-weight: 600;
  }

  .instruments {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .instrument-tag {
    padding: 3px 8px;
    background: #3d3d3d;
    border-radius: 4px;
    font-size: 11px;
    color: #b0b0b0;
    font-weight: 500;
  }

  .instrument-tag.more {
    background: #4a9eff;
    color: white;
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  .category-badge {
    padding: 4px 10px;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 4px;
    font-size: 11px;
    color: #b0b0b0;
    text-transform: capitalize;
  }

  .play-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #4a9eff;
    border: none;
    border-radius: 50%;
    color: white;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .play-btn:hover {
    background: #357abd;
    transform: scale(1.1);
  }
</style>
