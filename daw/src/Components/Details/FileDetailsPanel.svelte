<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  export let fileId: number;

  interface FileDetails {
    id: number;
    file_name: string;
    file_path: string;
    file_size: number;
    category: string | null;
    bpm: number | null;
    key_signature: string | null;
    time_signature: string | null;
    duration_seconds: number;
    track_count: number;
    note_count: number;
    instruments: string[];
    created_at: string;
    content_hash: string;
  }

  let details: FileDetails | null = null;
  let loading = true;
  let error = '';

  async function loadDetails() {
    loading = true;
    error = '';

    try {
      details = await invoke<FileDetails>('get_file_details', { fileId });
    } catch (err) {
      console.error('Failed to load file details:', err);
      error = `Failed to load details: ${err}`;
    } finally {
      loading = false;
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  onMount(() => {
    loadDetails();
  });

  $: if (fileId) {
    loadDetails();
  }
</script>

<div class="details-panel">
  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading details...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <p>{error}</p>
      <button class="btn-retry" on:click={loadDetails}>Retry</button>
    </div>
  {:else if details}
    <div class="panel-header">
      <h2>{details.file_name}</h2>
    </div>

    <div class="panel-content">
      <section class="info-section">
        <h3>General</h3>
        <div class="info-grid">
          <div class="info-item">
            <span class="label">File Size</span>
            <span class="value">{formatFileSize(details.file_size)}</span>
          </div>
          <div class="info-item">
            <span class="label">Duration</span>
            <span class="value">{formatDuration(details.duration_seconds ?? 0)}</span>
          </div>
          <div class="info-item">
            <span class="label">Tracks</span>
            <span class="value">{details.track_count}</span>
          </div>
          <div class="info-item">
            <span class="label">Notes</span>
            <span class="value">{details.note_count.toLocaleString()}</span>
          </div>
        </div>
      </section>

      <section class="info-section">
        <h3>Musical Properties</h3>
        <div class="info-grid">
          {#if details.bpm}
            <div class="info-item">
              <span class="label">BPM</span>
              <span class="value highlight">{details.bpm}</span>
            </div>
          {/if}
          {#if details.key_signature}
            <div class="info-item">
              <span class="label">Key</span>
              <span class="value highlight">{details.key_signature}</span>
            </div>
          {/if}
          {#if details.time_signature}
            <div class="info-item">
              <span class="label">Time Signature</span>
              <span class="value">{details.time_signature}</span>
            </div>
          {/if}
          {#if details.category}
            <div class="info-item">
              <span class="label">Category</span>
              <span class="value">{details.category}</span>
            </div>
          {/if}
        </div>
      </section>

      {#if details.instruments && details.instruments.length > 0}
        <section class="info-section">
          <h3>Instruments</h3>
          <div class="instrument-list">
            {#each details.instruments as instrument}
              <span class="instrument-badge">{instrument}</span>
            {/each}
          </div>
        </section>
      {/if}

      <section class="info-section">
        <h3>File Information</h3>
        <div class="info-grid">
          <div class="info-item full-width">
            <span class="label">Path</span>
            <span class="value mono">{details.file_path}</span>
          </div>
          <div class="info-item full-width">
            <span class="label">Hash</span>
            <span class="value mono">{details.content_hash}</span>
          </div>
          <div class="info-item">
            <span class="label">Added</span>
            <span class="value">{formatDate(details.created_at)}</span>
          </div>
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .details-panel {
    height: 100%;
    background: #1e1e1e;
    border-left: 1px solid #3d3d3d;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .loading-state,
  .error-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: #b0b0b0;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #3d3d3d;
    border-top-color: #4a9eff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .btn-retry {
    padding: 8px 16px;
    background: #4a9eff;
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 14px;
    cursor: pointer;
  }

  .panel-header {
    padding: 20px;
    border-bottom: 1px solid #3d3d3d;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 18px;
    color: #e0e0e0;
    word-break: break-word;
  }

  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .info-section {
    margin-bottom: 24px;
  }

  .info-section h3 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #808080;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-item.full-width {
    grid-column: 1 / -1;
  }

  .label {
    font-size: 11px;
    color: #808080;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .value {
    font-size: 14px;
    color: #e0e0e0;
    font-weight: 500;
  }

  .value.highlight {
    color: #4a9eff;
    font-weight: 600;
    font-size: 16px;
  }

  .value.mono {
    font-family: 'Courier New', monospace;
    font-size: 11px;
    word-break: break-all;
  }

  .instrument-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .instrument-badge {
    padding: 6px 12px;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    font-size: 12px;
    color: #e0e0e0;
  }

  /* Custom scrollbar */
  .panel-content::-webkit-scrollbar {
    width: 6px;
  }

  .panel-content::-webkit-scrollbar-track {
    background: #1e1e1e;
  }

  .panel-content::-webkit-scrollbar-thumb {
    background: #3d3d3d;
    border-radius: 3px;
  }
</style>
