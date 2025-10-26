<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';
  import { save } from '@tauri-apps/plugin-dialog';

  const dispatch = createEventDispatcher();

  export let isOpen = false;

  let outputPath = '';
  let isExporting = false;
  let error = '';

  async function selectOutputPath() {
    try {
      const selected = await save({
        defaultPath: 'project.mid',
        filters: [
          { name: 'MIDI Files', extensions: ['mid', 'midi'] }
        ]
      });

      if (selected) {
        outputPath = selected;
      }
    } catch (err) {
      console.error('Failed to select path:', err);
      error = `Path selection failed: ${err}`;
    }
  }

  async function handleExport() {
    if (!outputPath) {
      error = 'Please select an output path';
      return;
    }

    isExporting = true;
    error = '';

    try {
      await invoke('export_project_midi', { outputPath });
      dispatch('exported', { outputPath });
      isOpen = false;
    } catch (err) {
      console.error('Export failed:', err);
      error = `Export failed: ${err}`;
    } finally {
      isExporting = false;
    }
  }

  function handleCancel() {
    isOpen = false;
    outputPath = '';
    error = '';
    dispatch('cancel');
  }
</script>

{#if isOpen}
  <div class="dialog-overlay" on:click={handleCancel}>
    <div class="dialog" on:click|stopPropagation>
      <div class="dialog-header">
        <h2>Export Project</h2>
        <button class="close-btn" on:click={handleCancel}>✕</button>
      </div>

      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <div class="dialog-body">
        <div class="form-group">
          <label for="output-path">Output File</label>
          <div class="path-input-group">
            <input
              id="output-path"
              type="text"
              class="path-input"
              bind:value={outputPath}
              placeholder="Select output file..."
              readonly
            />
            <button class="btn-browse" on:click={selectOutputPath}>
              Browse
            </button>
          </div>
        </div>

        <div class="info-box">
          <p><strong>Export Format:</strong> Standard MIDI File (Type 1)</p>
          <p><strong>Tracks:</strong> All non-muted tracks will be exported</p>
          <p><strong>Tempo:</strong> Current project tempo will be included</p>
        </div>
      </div>

      <div class="dialog-footer">
        <button class="btn-secondary" on:click={handleCancel}>
          Cancel
        </button>
        <button
          class="btn-primary"
          on:click={handleExport}
          disabled={!outputPath || isExporting}
        >
          {isExporting ? 'Exporting...' : 'Export'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
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

  .dialog {
    width: 500px;
    max-width: 90%;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid #3d3d3d;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 20px;
    color: #e0e0e0;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #b0b0b0;
    font-size: 20px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: #3d3d3d;
    color: #e0e0e0;
  }

  .error-message {
    padding: 12px 20px;
    background: #3d1f1f;
    border-bottom: 1px solid #5d2f2f;
    color: #ff6b6b;
    font-size: 13px;
  }

  .dialog-body {
    padding: 20px;
  }

  .form-group {
    margin-bottom: 20px;
  }

  .form-group label {
    display: block;
    margin-bottom: 8px;
    font-size: 13px;
    color: #b0b0b0;
    font-weight: 600;
  }

  .path-input-group {
    display: flex;
    gap: 8px;
  }

  .path-input {
    flex: 1;
    padding: 10px 12px;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 14px;
  }

  .btn-browse {
    padding: 10px 20px;
    background: #3d3d3d;
    border: 1px solid #4d4d4d;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-browse:hover {
    background: #4d4d4d;
    border-color: #4a9eff;
  }

  .info-box {
    padding: 16px;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
  }

  .info-box p {
    margin: 0 0 8px 0;
    font-size: 13px;
    color: #b0b0b0;
  }

  .info-box p:last-child {
    margin-bottom: 0;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding: 20px;
    border-top: 1px solid #3d3d3d;
  }

  .btn-primary,
  .btn-secondary {
    padding: 10px 24px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #4a9eff;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #357abd;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: transparent;
    border: 1px solid #3d3d3d;
    color: #e0e0e0;
  }

  .btn-secondary:hover {
    background: #3d3d3d;
  }
</style>
