<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  export let sourceFileId: number;

  interface CompatibleFile {
    file: {
      id: number;
      file_name: string;
      bpm: number | null;
      key_signature: string | null;
      category: string | null;
    };
    compatibility_score: number;
    key_distance: number;
    bpm_difference: number;
    reason: string;
  }

  let compatibleFiles: CompatibleFile[] = [];
  let loading = false;
  let error = '';

  async function findCompatible() {
    loading = true;
    error = '';

    try {
      compatibleFiles = await invoke<CompatibleFile[]>('find_compatible_files', {
        fileId: sourceFileId,
        maxResults: 20
      });
    } catch (err) {
      console.error('Failed to find compatible files:', err);
      error = `Failed to find compatible files: ${err}`;
    } finally {
      loading = false;
    }
  }

  function handleFileSelect(fileId: number) {
    dispatch('fileSelect', { fileId });
  }

  function getScoreColor(score: number): string {
    if (score >= 90) return '#4ade80';
    if (score >= 70) return '#4a9eff';
    if (score >= 50) return '#f59e0b';
    return '#ef4444';
  }
</script>

<div class="compatibility-matcher">
  <div class="matcher-header">
    <h3>Compatible Files</h3>
    <button class="btn-find" on:click={findCompatible} disabled={loading}>
      {loading ? 'Searching...' : 'Find Compatible'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Analyzing compatibility...</p>
    </div>
  {:else if compatibleFiles.length > 0}
    <div class="results-list">
      {#each compatibleFiles as match}
        <button
          class="match-item"
          on:click={() => handleFileSelect(match.file.id)}
        >
          <div class="match-header">
            <span class="file-name">{match.file.file_name}</span>
            <span
              class="score"
              style="color: {getScoreColor(match.compatibility_score)}"
            >
              {Math.round(match.compatibility_score)}%
            </span>
          </div>

          <div class="match-details">
            {#if match.file.bpm}
              <span class="detail">BPM: {match.file.bpm}</span>
            {/if}
            {#if match.file.key_signature}
              <span class="detail">Key: {match.file.key_signature}</span>
            {/if}
            {#if match.file.category}
              <span class="detail">{match.file.category}</span>
            {/if}
          </div>

          <p class="match-reason">{match.reason}</p>
        </button>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <p>Click "Find Compatible" to discover matching files</p>
    </div>
  {/if}
</div>

<style>
  .compatibility-matcher {
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
    overflow: hidden;
  }

  .matcher-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    border-bottom: 1px solid #3d3d3d;
  }

  .matcher-header h3 {
    margin: 0;
    font-size: 16px;
    color: #e0e0e0;
  }

  .btn-find {
    padding: 8px 16px;
    background: #4a9eff;
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-find:hover:not(:disabled) {
    background: #357abd;
  }

  .btn-find:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-message {
    padding: 12px 16px;
    background: #3d1f1f;
    color: #ff6b6b;
    font-size: 13px;
  }

  .loading-state,
  .empty-state {
    padding: 40px 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    color: #b0b0b0;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #3d3d3d;
    border-top-color: #4a9eff;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .results-list {
    max-height: 500px;
    overflow-y: auto;
  }

  .match-item {
    width: 100%;
    padding: 16px;
    background: transparent;
    border: none;
    border-bottom: 1px solid #3d3d3d;
    text-align: left;
    cursor: pointer;
    transition: background 0.2s;
  }

  .match-item:hover {
    background: #353535;
  }

  .match-item:last-child {
    border-bottom: none;
  }

  .match-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .file-name {
    font-size: 14px;
    font-weight: 600;
    color: #e0e0e0;
  }

  .score {
    font-size: 16px;
    font-weight: 700;
  }

  .match-details {
    display: flex;
    gap: 12px;
    margin-bottom: 8px;
  }

  .detail {
    font-size: 12px;
    color: #808080;
  }

  .match-reason {
    margin: 0;
    font-size: 12px;
    color: #b0b0b0;
    font-style: italic;
  }

  /* Custom scrollbar */
  .results-list::-webkit-scrollbar {
    width: 6px;
  }

  .results-list::-webkit-scrollbar-track {
    background: #2d2d2d;
  }

  .results-list::-webkit-scrollbar-thumb {
    background: #3d3d3d;
    border-radius: 3px;
  }
</style>
