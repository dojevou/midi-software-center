<script lang="ts">
  import { onMount } from 'svelte';
  import type { Track } from '../api';

  export let visible = false;
  export let tracks: Track[] = [];

  let outputPath = '';
  let filename = 'export.mid';
  let format: 0 | 1 = 1;
  let selectedTracks: number[] = [];
  let exportAll = true;

  let overrideTempo = false;
  let tempo = 120;
  let overrideTimeSig = false;
  let timeSigNumerator = 4;
  let timeSigDenominator = 4;
  let overrideKey = false;
  let keySignature = 'C';
  let quantize: 'none' | '1/4' | '1/8' | '1/16' = 'none';

  let includeTempoChanges = true;
  let includeProgramChanges = true;
  let includeControllers = true;
  let normalizeVelocity = false;
  let tpqn = 480;

  let exporting = false;
  let progress = 0;
  let statusText = '';
  let exportError = '';

  $: if (exportAll) {
    selectedTracks = tracks.map((t) => t.id);
  }

  async function chooseLocation() {
    try {
      const result = await (window as any).__TAURI_INTERNALS__.invoke('plugin:dialog|save', {
        defaultPath: filename,
        filters: [
          {
            name: 'MIDI File',
            extensions: ['mid', 'midi'],
          },
        ],
      });

      if (result) {
        outputPath = result;
        // Extract filename from path
        const pathParts = result.split(/[\\/]/);
        filename = pathParts[pathParts.length - 1];
      }
    } catch (error) {
      console.error('Failed to open save dialog:', error);
      exportError = 'Failed to open save dialog: ' + error;
    }
  }

  async function startExport() {
    if (!outputPath) {
      await chooseLocation();
      if (!outputPath) return;
    }

    exporting = true;
    progress = 0;
    exportError = '';
    statusText = 'Preparing export...';

    try {
      const options = {
        format,
        tracks: exportAll ? null : selectedTracks,
        tempo: overrideTempo ? tempo : null,
        time_signature: overrideTimeSig ? `${timeSigNumerator}/${timeSigDenominator}` : null,
        key_signature: overrideKey ? keySignature : null,
        quantize: quantize !== 'none' ? quantize : null,
        include_tempo_changes: includeTempoChanges,
        include_program_changes: includeProgramChanges,
        include_controllers: includeControllers,
        normalize_velocity: normalizeVelocity,
        tpqn,
      };

      // Simulate progress updates
      const progressInterval = setInterval(() => {
        if (progress < 90) {
          progress += 10;
          const trackCount = exportAll ? tracks.length : selectedTracks.length;
          const currentTrack = Math.ceil((progress / 100) * trackCount);
          statusText = `Exporting track ${currentTrack} of ${trackCount}...`;
        }
      }, 200);

      await (window as any).__TAURI_INTERNALS__.invoke('export_project_midi', {
        output_path: outputPath,
        options,
      });

      clearInterval(progressInterval);
      statusText = 'Export completed successfully!';
      progress = 100;

      // Close dialog after 2 seconds
      setTimeout(() => {
        visible = false;
        exporting = false;
        resetForm();
      }, 2000);
    } catch (error) {
      console.error('Export failed:', error);
      exportError = 'Export failed: ' + error;
      statusText = '';
      exporting = false;
      progress = 0;
    }
  }

  function applyPreset(preset: string) {
    switch (preset) {
      case 'standard':
        format = 1;
        tpqn = 480;
        quantize = 'none';
        normalizeVelocity = false;
        includeTempoChanges = true;
        includeProgramChanges = true;
        includeControllers = true;
        break;
      case 'high-res':
        format = 1;
        tpqn = 960;
        quantize = 'none';
        normalizeVelocity = false;
        includeTempoChanges = true;
        includeProgramChanges = true;
        includeControllers = true;
        break;
      case 'logic':
        format = 1;
        tpqn = 480;
        quantize = '1/16';
        normalizeVelocity = true;
        includeTempoChanges = true;
        includeProgramChanges = true;
        includeControllers = false;
        break;
    }
  }

  function resetForm() {
    outputPath = '';
    filename = 'export.mid';
    exportError = '';
    statusText = '';
    progress = 0;
  }

  function closeDialog() {
    if (!exporting) {
      visible = false;
      resetForm();
    }
  }

  const keyOptions = [
    'C',
    'C#',
    'D',
    'D#',
    'E',
    'F',
    'F#',
    'G',
    'G#',
    'A',
    'A#',
    'B',
    'Cm',
    'C#m',
    'Dm',
    'D#m',
    'Em',
    'Fm',
    'F#m',
    'Gm',
    'G#m',
    'Am',
    'A#m',
    'Bm',
  ];
</script>

{#if visible}
  <div class="modal-overlay" on:click={closeDialog}>
    <div class="export-dialog" on:click={(e) => e.stopPropagation()}>
      <h2>Export MIDI</h2>

      <!-- File Settings -->
      <section>
        <h3>File Settings</h3>
        <div class="form-group">
          <label>Filename:</label>
          <input type="text" bind:value={filename} disabled={exporting} />
        </div>

        <div class="form-group">
          <label>Save Location:</label>
          <div class="path-selector">
            <input type="text" bind:value={outputPath} readonly />
            <button on:click={chooseLocation} disabled={exporting}>Browse...</button>
          </div>
        </div>

        <div class="form-group">
          <label>Format:</label>
          <select bind:value={format} disabled={exporting}>
            <option value={0}>MIDI Type 0 (Single Track)</option>
            <option value={1}>MIDI Type 1 (Multi-Track)</option>
          </select>
        </div>
      </section>

      <!-- Track Selection -->
      <section>
        <h3>Tracks</h3>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={exportAll} disabled={exporting} />
          Export All Tracks
        </label>

        {#if !exportAll}
          <div class="track-list">
            {#each tracks as track}
              <label class="track-checkbox">
                <input
                  type="checkbox"
                  value={track.id}
                  bind:group={selectedTracks}
                  disabled={exporting}
                />
                <span class="track-color" style="background-color: {track.color || '#666'}"></span>
                <span class="track-name">{track.name}</span>
              </label>
            {/each}
          </div>
        {/if}
      </section>

      <!-- Musical Settings -->
      <section>
        <h3>Musical Settings</h3>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={overrideTempo} disabled={exporting} />
          Override Tempo:
          <input
            type="number"
            bind:value={tempo}
            min="20"
            max="300"
            disabled={!overrideTempo || exporting}
            class="inline-input"
          />
          BPM
        </label>

        <label class="checkbox-label">
          <input type="checkbox" bind:checked={overrideTimeSig} disabled={exporting} />
          Override Time Signature:
          <input
            type="number"
            bind:value={timeSigNumerator}
            min="1"
            max="16"
            disabled={!overrideTimeSig || exporting}
            class="inline-input-small"
          />
          /
          <select
            bind:value={timeSigDenominator}
            disabled={!overrideTimeSig || exporting}
            class="inline-select"
          >
            <option value={2}>2</option>
            <option value={4}>4</option>
            <option value={8}>8</option>
            <option value={16}>16</option>
          </select>
        </label>

        <label class="checkbox-label">
          <input type="checkbox" bind:checked={overrideKey} disabled={exporting} />
          Override Key Signature:
          <select
            bind:value={keySignature}
            disabled={!overrideKey || exporting}
            class="inline-select"
          >
            {#each keyOptions as key}
              <option value={key}>{key}</option>
            {/each}
          </select>
        </label>

        <div class="form-group">
          <label>Quantize:</label>
          <select bind:value={quantize} disabled={exporting}>
            <option value="none">None</option>
            <option value="1/4">1/4 Note</option>
            <option value="1/8">1/8 Note</option>
            <option value="1/16">1/16 Note</option>
          </select>
        </div>
      </section>

      <!-- Advanced Options -->
      <details>
        <summary>Advanced Options</summary>
        <div class="advanced-content">
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={includeTempoChanges} disabled={exporting} />
            Include Tempo Changes
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={includeProgramChanges} disabled={exporting} />
            Include Program Changes
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={includeControllers} disabled={exporting} />
            Include Controller Data
          </label>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={normalizeVelocity} disabled={exporting} />
            Normalize Velocity
          </label>
          <div class="form-group">
            <label>TPQN (Ticks Per Quarter Note):</label>
            <input
              type="number"
              bind:value={tpqn}
              min="96"
              max="1920"
              step="96"
              disabled={exporting}
            />
          </div>
        </div>
      </details>

      <!-- Presets -->
      <section class="presets">
        <h3>Presets</h3>
        <div class="preset-buttons">
          <button on:click={() => applyPreset('standard')} disabled={exporting}>
            Standard MIDI
          </button>
          <button on:click={() => applyPreset('high-res')} disabled={exporting}>
            High Resolution
          </button>
          <button on:click={() => applyPreset('logic')} disabled={exporting}>
            Logic Pro Compatible
          </button>
        </div>
      </section>

      <!-- Progress -->
      {#if exporting}
        <div class="progress-section">
          <div class="progress-bar">
            <div class="progress-fill" style="width: {progress}%"></div>
          </div>
          <p class="status-text">{statusText}</p>
        </div>
      {/if}

      <!-- Error Display -->
      {#if exportError}
        <div class="error-message">
          <strong>Error:</strong>
          {exportError}
        </div>
      {/if}

      <!-- Actions -->
      <div class="dialog-actions">
        <button
          class="primary"
          on:click={startExport}
          disabled={exporting || !filename || selectedTracks.length === 0}
        >
          {exporting ? 'Exporting...' : 'Export'}
        </button>
        <button on:click={closeDialog} disabled={exporting}>
          {exporting ? 'Please wait...' : 'Cancel'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    backdrop-filter: blur(2px);
  }

  .export-dialog {
    background: linear-gradient(180deg, #1f1f1f 0%, #1a1a1a 100%);
    padding: 2em;
    border-radius: 12px;
    border: 2px solid rgba(255, 62, 0, 0.3);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    max-width: 650px;
    width: 90%;
    max-height: 85vh;
    overflow-y: auto;
  }

  h2 {
    margin: 0 0 1.5em 0;
    color: #ff3e00;
    font-size: 1.5em;
    border-bottom: 2px solid rgba(255, 62, 0, 0.3);
    padding-bottom: 0.5em;
  }

  h3 {
    margin: 0 0 0.75em 0;
    color: rgba(255, 255, 255, 0.9);
    font-size: 1em;
    font-weight: 600;
  }

  section {
    margin: 1.5em 0;
    padding: 1em;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
  }

  .form-group {
    margin: 0.75em 0;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5em;
    color: rgba(255, 255, 255, 0.7);
    font-size: 0.875em;
    font-weight: 500;
  }

  input[type='text'],
  input[type='number'],
  select {
    width: 100%;
    padding: 0.6em;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    color: white;
    font-size: 0.875em;
    transition: all 0.2s;
  }

  input[type='text']:focus,
  input[type='number']:focus,
  select:focus {
    outline: none;
    border-color: #ff3e00;
    background: rgba(255, 255, 255, 0.08);
  }

  input[type='text']:disabled,
  input[type='number']:disabled,
  select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .inline-input {
    width: 80px;
    display: inline-block;
    margin: 0 0.5em;
    padding: 0.4em;
  }

  .inline-input-small {
    width: 50px;
    display: inline-block;
    margin: 0 0.25em;
    padding: 0.4em;
  }

  .inline-select {
    width: auto;
    display: inline-block;
    margin: 0 0.5em;
    padding: 0.4em;
  }

  .path-selector {
    display: flex;
    gap: 0.5em;
  }

  .path-selector input {
    flex: 1;
  }

  .path-selector button {
    flex-shrink: 0;
    padding: 0.6em 1em;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5em;
    margin: 0.75em 0;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.875em;
    cursor: pointer;
    padding: 0.5em;
    border-radius: 4px;
    transition: background 0.2s;
  }

  .checkbox-label:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  input[type='checkbox'] {
    width: 18px;
    height: 18px;
    cursor: pointer;
    accent-color: #ff3e00;
  }

  .track-list {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
    margin-top: 0.75em;
    max-height: 200px;
    overflow-y: auto;
    padding: 0.5em;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
  }

  .track-checkbox {
    display: flex;
    align-items: center;
    gap: 0.5em;
    padding: 0.5em;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .track-checkbox:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .track-color {
    width: 4px;
    height: 20px;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .track-name {
    flex: 1;
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.875em;
  }

  .presets {
    background: rgba(59, 130, 246, 0.05);
    border-color: rgba(59, 130, 246, 0.2);
  }

  .preset-buttons {
    display: flex;
    gap: 0.5em;
    flex-wrap: wrap;
  }

  .preset-buttons button {
    flex: 1;
    min-width: 150px;
    padding: 0.75em;
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    font-size: 0.875em;
  }

  .preset-buttons button:hover:not(:disabled) {
    background: rgba(59, 130, 246, 0.25);
    border-color: rgba(59, 130, 246, 0.5);
  }

  details {
    margin: 1.5em 0;
    padding: 1em;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
  }

  summary {
    cursor: pointer;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
    padding: 0.5em;
    border-radius: 4px;
    transition: background 0.2s;
    user-select: none;
  }

  summary:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .advanced-content {
    margin-top: 1em;
    padding-top: 1em;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .progress-section {
    margin: 1.5em 0;
    padding: 1em;
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.3);
    border-radius: 8px;
  }

  .progress-bar {
    width: 100%;
    height: 24px;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 12px;
    overflow: hidden;
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6 0%, #60a5fa 100%);
    transition: width 0.3s ease;
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.5);
  }

  .status-text {
    text-align: center;
    margin-top: 0.75em;
    color: #60a5fa;
    font-size: 0.875em;
    font-weight: 500;
  }

  .error-message {
    margin: 1em 0;
    padding: 1em;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 6px;
    color: #fca5a5;
    font-size: 0.875em;
  }

  .error-message strong {
    color: #ef4444;
  }

  .dialog-actions {
    display: flex;
    gap: 0.75em;
    justify-content: flex-end;
    margin-top: 2em;
    padding-top: 1.5em;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  button {
    padding: 0.75em 1.5em;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 600;
    font-size: 0.875em;
    background: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.2);
  }

  button:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.15);
    transform: translateY(-1px);
  }

  button:active:not(:disabled) {
    transform: translateY(0);
  }

  button.primary {
    background: linear-gradient(135deg, #ff3e00 0%, #ff6e40 100%);
    color: white;
    border: none;
    box-shadow: 0 4px 12px rgba(255, 62, 0, 0.3);
  }

  button.primary:hover:not(:disabled) {
    background: linear-gradient(135deg, #ff5722 0%, #ff7961 100%);
    box-shadow: 0 6px 16px rgba(255, 62, 0, 0.4);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none !important;
  }

  /* Scrollbar styling */
  .export-dialog::-webkit-scrollbar,
  .track-list::-webkit-scrollbar {
    width: 8px;
  }

  .export-dialog::-webkit-scrollbar-track,
  .track-list::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 4px;
  }

  .export-dialog::-webkit-scrollbar-thumb,
  .track-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }

  .export-dialog::-webkit-scrollbar-thumb:hover,
  .track-list::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }
</style>
