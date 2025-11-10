<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { api } from '$lib/api';
  import type { ImportProgress } from '$lib/types';

  let selectedOperation = 'import'; // 'import' or 'analysis'
  let progress = 0;
  let currentFile = '';
  let processed = 0;
  let total = 0;
  let isRunning = false;
  let isPaused = false;
  let unlisten: (() => void) | null = null;
  let errorMessage = '';

  // Event listener for pipeline progress
  onMount(() => {
    (async () => {
      unlisten = await listen<ImportProgress>('pipeline-progress', (event) => {
        progress = (event.payload.current / event.payload.total) * 100;
        currentFile = event.payload.current_file;
        processed = event.payload.current;
        total = event.payload.total;
        isRunning = true;
        isPaused = false;
      });

      // Listen for completion
      const unlistenComplete = await listen('pipeline-complete', () => {
        isRunning = false;
        progress = 100;
      });

      // Listen for error
      const unlistenError = await listen('pipeline-error', (event) => {
        console.error('Pipeline error:', event.payload);
        isRunning = false;
      });
    })();

    return () => {
      unlisten?.();
    };
  });

  onDestroy(() => {
    unlisten?.();
  });

  async function startPipeline() {
    try {
      errorMessage = '';

      if (selectedOperation === 'import') {
        // Open directory picker for import
        const selected = await open({
          directory: true,
          multiple: false,
          title: 'Select Directory to Import'
        });

        if (selected) {
          isRunning = true;
          isPaused = false;
          await api.pipeline.importDirectory(selected as string);
        }
      } else if (selectedOperation === 'analysis') {
        // Start analysis on all database files
        isRunning = true;
        isPaused = false;
        await api.pipeline.startAnalysis();
      }
    } catch (error) {
      console.error(`Failed to start ${selectedOperation}:`, error);
      errorMessage = `Error: ${error}`;
      isRunning = false;
    }
  }

  async function pausePipeline() {
    // TODO: Implement pause functionality when backend supports it
    isPaused = true;
  }

  async function stopPipeline() {
    // TODO: Implement stop functionality when backend supports it
    isRunning = false;
    progress = 0;
    currentFile = '';
    processed = 0;
    total = 0;
  }

  function formatProgress(processed: number, total: number): string {
    return `${processed} / ${total}`;
  }
</script>

<WindowBase windowId="pipeline" title="Pipeline" width={600} height={400} resizable={true}>
  <div class="pipeline-window dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
    <!-- Operation Selector -->
    <div class="operation-selector dark:bg-menu p-3 rounded mb-4">
      <label class="dark:text-gray-300 block mb-2">Operation:</label>
      <select
        bind:value={selectedOperation}
        class="dark:bg-input dark:text-app-text px-4 py-2 rounded w-full"
        disabled={isRunning}
      >
        <option value="import">Import Files</option>
        <option value="analysis">Analyze Files</option>
      </select>
    </div>

    <!-- Progress Bar -->
    <div class="progress-section dark:bg-window-subtle p-4 rounded mb-4 flex-1 flex flex-col justify-center items-center">
      <div class="progress-bar mb-4 w-full bg-gray-700 rounded-full h-4">
        <div
          class="progress-fill dark:bg-primary h-4 rounded-full transition-all duration-300"
          style="width: {progress}%"
        ></div>
      </div>
      <div class="progress-info dark:text-gray-300 text-center mb-2">
        {progress.toFixed(1)}%
      </div>
      {#if currentFile}
        <div class="current-file dark:text-gray-400 text-sm mb-2">
          Current: {currentFile}
        </div>
      {/if}
      <div class="processed dark:text-gray-500 text-sm">
        {formatProgress(processed, total)}
      </div>
      {#if errorMessage}
        <div class="error-message dark:text-red-400 dark:bg-red-900 dark:bg-opacity-20 px-3 py-2 rounded mt-2 text-sm">
          {errorMessage}
        </div>
      {/if}
    </div>

    <!-- Controls -->
    <div class="controls dark:bg-menu p-3 rounded flex justify-center space-x-4">
      <button
        class="start-btn dark:bg-success dark:text-white px-6 py-2 rounded hover:dark:bg-success-dark"
        on:click={startPipeline}
        disabled={isRunning || isPaused}
      >
        Start
      </button>
      <button
        class="pause-btn dark:bg-secondary dark:text-white px-6 py-2 rounded hover:dark:bg-secondary-dark"
        on:click={pausePipeline}
        disabled={!isRunning || isPaused}
      >
        Pause
      </button>
      <button
        class="stop-btn dark:bg-error dark:text-white px-6 py-2 rounded hover:dark:bg-error-dark"
        on:click={stopPipeline}
        disabled={!isRunning && !isPaused}
      >
        Stop
      </button>
    </div>
  </div>

  <style>
    .pipeline-window {
      height: 100%;
    }
    .start-btn:disabled, .pause-btn:disabled, .stop-btn:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
  </style>
</WindowBase>