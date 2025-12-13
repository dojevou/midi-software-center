<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { isTauri, safeListen } from '$lib/utils/tauri';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '$lib/api';
  import type { AnalysisResults, ImportStats, PipelineProgress } from '$lib/types';

  let activeTab = 'import';
  const dragOver = false;
  let files: FileList | null = null;
  let isDragging = false;
  let isProcessing = false;
  let errors: string[] = [];

  let progress: PipelineProgress = {
    current: 0,
    total: 0,
    stage: 'idle',
    rate: 0,
    eta_seconds: 0,
    details: '',
  };

  let unlistenProgress: (() => void) | null = null;
  let unlistenCompleted: (() => void) | null = null;

  // Reactive
  $: currentProgress = progress;
  $: operationStatus = progress.stage;

  onMount(async () => {
    // Listen for pipeline progress events (only in Tauri context)
    unlistenProgress = await safeListen<PipelineProgress>('pipeline::progress', (payload) => {
      progress = payload;
      console.log('Pipeline progress:', progress);
    });

    unlistenCompleted = await safeListen<void>('pipeline::completed', () => {
      isProcessing = false;
      progress = { ...progress, stage: 'completed', details: 'Operation completed' };
      console.log('Pipeline completed');
    });
  });

  onDestroy(() => {
    // Clean up event listeners
    if (unlistenProgress) {
      unlistenProgress();
    }
    if (unlistenCompleted) {
      unlistenCompleted();
    }
  });

  function setTab(tab: string) {
    activeTab = tab;
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
    files = event.dataTransfer?.files || null;
    if (files && files.length > 0) {
      handleImportFiles(Array.from(files));
    }
  }

  async function handleImportFiles(fileList: File[]) {
    if (isProcessing) {
      return;
    }
    isProcessing = true;
    errors = [];

    try {
      // Convert File objects to paths (Note: This requires File.path which is available in Tauri)
      const filePaths = fileList.map((f: any) => f.path);
      console.log('Importing files:', filePaths);

      const result: ImportStats = await api.pipeline.importFiles(filePaths);
      console.log('Import complete:', result);

      if (result.errors.length > 0) {
        errors = result.errors;
      }
    } catch (error) {
      console.error('Import failed:', error);
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  async function startAnalyze() {
    if (isProcessing) {
      return;
    }
    isProcessing = true;
    errors = [];

    try {
      // Get all files from database to analyze
      const searchResult = await api.search.files({});
      const fileIds = searchResult.files.map((f: { id: number }) => f.id);

      console.log('Analyzing files:', fileIds.length);
      const result: AnalysisResults = await api.pipeline.analyzeFiles(fileIds);
      console.log('Analysis complete:', result);

      if (result.errors.length > 0) {
        errors = result.errors;
      }
    } catch (error) {
      console.error('Analysis failed:', error);
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  async function handleImportArchive() {
    if (isProcessing) {
      return;
    }
    isProcessing = true;
    errors = [];

    try {
      // Open file picker for archive files
      const selected = await open({
        title: 'Select MIDI Archive',
        filters: [
          {
            name: 'Archive Files',
            extensions: ['zip', 'tar', 'gz', '7z'],
          },
          {
            name: 'All Files',
            extensions: ['*'],
          },
        ],
        multiple: false,
      });

      if (!selected || Array.isArray(selected)) {
        console.log('Archive import cancelled');
        isProcessing = false;
        return;
      }

      console.log('Importing archive from:', selected);
      const result: ImportStats = await api.pipeline.importFiles([selected]);
      console.log('Archive import complete:', result);

      if (result.errors.length > 0) {
        errors = result.errors;
      }
    } catch (error) {
      console.error('Archive import failed:', error);
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  async function startArchive() {
    if (isProcessing) {
      return;
    }
    isProcessing = true;
    errors = [];

    try {
      // Open file save dialog for archive destination
      const savePath = await open({
        title: 'Save MIDI Archive',
        filters: [
          {
            name: 'ZIP Archive',
            extensions: ['zip'],
          },
        ],
        multiple: false,
        directory: false,
      });

      if (!savePath || Array.isArray(savePath)) {
        console.log('Archive export cancelled');
        isProcessing = false;
        return;
      }

      // Get all files to archive
      const searchResult = await api.search.files({});
      const fileIds = searchResult.files.map((f: { id: number }) => f.id);

      console.log('Archiving files to:', savePath);
      const result: ImportStats = await api.pipeline.archiveFiles(fileIds, savePath);
      console.log('Archive complete:', result);

      if (result.errors.length > 0) {
        errors = result.errors;
      }
    } catch (error) {
      console.error('Archive failed:', error);
      errors = [String(error)];
    } finally {
      isProcessing = false;
    }
  }

  async function cancelOperation() {
    try {
      await api.pipeline.cancel();
      console.log('Pipeline operation cancelled');
    } catch (error) {
      console.error('Cancel failed:', error);
    }
  }

  function formatRate(rate: number): string {
    return `${rate.toFixed(1)} files/sec`;
  }

  function formatETA(eta: number): string {
    if (eta < 60) {
      return `${Math.floor(eta)}s`;
    }
    if (eta < 3600) {
      return `${Math.floor(eta / 60)}m ${Math.floor(eta % 60)}s`;
    }
    return `${Math.floor(eta / 3600)}h ${Math.floor((eta % 3600) / 60)}m`;
  }
</script>

<div class="pipeline-window dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
  <!-- Tabs -->
  <div class="tabs dark:bg-menu p-2 rounded mb-4 flex space-x-1">
    <button
      on:click={() => setTab('import')}
      class="tab-btn px-4 py-2 rounded {activeTab === 'import'
        ? 'dark:bg-primary dark:text-white'
        : 'dark:bg-secondary dark:text-gray-300'} hover:dark:bg-primary/80 transition-colors"
    >
      Import
    </button>
    <button
      on:click={() => setTab('analyze')}
      class="tab-btn px-4 py-2 rounded {activeTab === 'analyze'
        ? 'dark:bg-primary dark:text-white'
        : 'dark:bg-secondary dark:text-gray-300'} hover:dark:bg-primary/80 transition-colors"
    >
      Analyze
    </button>
    <button
      on:click={() => setTab('archive')}
      class="tab-btn px-4 py-2 rounded {activeTab === 'archive'
        ? 'dark:bg-primary dark:text-white'
        : 'dark:bg-secondary dark:text-gray-300'} hover:dark:bg-primary/80 transition-colors"
    >
      Archive
    </button>
  </div>

  <!-- Content -->
  {#if activeTab === 'import'}
    <div class="import-tab flex-1 flex flex-col items-center justify-center">
      <div
        class="drop-zone w-full h-64 border-2 border-dashed dark:border-gray-600 rounded-lg flex flex-col items-center justify-center {isDragging
          ? 'dark:border-primary dark:bg-primary/10'
          : 'dark:border-gray-500'} transition-colors"
        on:dragover={handleDragOver}
        on:dragleave={handleDragLeave}
        on:drop={handleDrop}
      >
        <div class="text-center">
          <div class="text-4xl mb-2">📁</div>
          <h3 class="dark:text-gray-300 mb-2">Drop MIDI files here</h3>
          <p class="dark:text-gray-400 text-sm">or click to select files</p>
        </div>
        {#if files && files.length > 0}
          <div class="mt-4 text-center dark:text-gray-300">
            <p>Selected: {files.length} files</p>
            <button
              on:click={() => files && handleImportFiles(Array.from(files))}
              class="mt-2 px-4 py-2 dark:bg-primary dark:text-white rounded hover:dark:bg-primary-dark disabled:opacity-50"
              disabled={isProcessing}
            >
              {isProcessing ? 'Importing...' : 'Import Files'}
            </button>
          </div>
        {/if}
      </div>
    </div>
  {:else if activeTab === 'analyze'}
    <div class="analyze-tab flex-1 flex flex-col">
      <div class="controls p-4">
        <button
          on:click={startAnalyze}
          class="w-full px-4 py-2 dark:bg-primary dark:text-white rounded hover:dark:bg-primary-dark disabled:opacity-50"
          disabled={isProcessing}
        >
          {isProcessing ? 'Analyzing...' : 'Start Analysis'}
        </button>
      </div>
      <div class="progress-section flex-1 p-4">
        {#if progress.stage === 'analyzing' || isProcessing}
          <div class="progress-info mb-4">
            <div class="dark:text-gray-300 mb-2">
              {progress.details || 'Processing...'}
            </div>
            <div class="progress-bar dark:bg-gray-700 rounded-full h-4 overflow-hidden mb-2">
              <div
                class="dark:bg-primary h-full transition-all duration-300"
                style="width: {progress.total > 0 ? (progress.current / progress.total) * 100 : 0}%"
              ></div>
            </div>
            <div class="stats flex justify-between dark:text-gray-300 text-sm">
              <span>{progress.current} / {progress.total}</span>
              <span>Rate: {formatRate(progress.rate)}</span>
              <span>ETA: {formatETA(progress.eta_seconds)}</span>
            </div>
            <button
              on:click={cancelOperation}
              class="mt-2 px-4 py-2 dark:bg-red-600 dark:text-white rounded hover:dark:bg-red-500"
            >
              Cancel
            </button>
          </div>
        {:else}
          <div class="no-progress dark:text-gray-400 text-center py-8">No analysis in progress</div>
        {/if}
      </div>
    </div>
  {:else if activeTab === 'archive'}
    <div class="archive-tab flex-1 flex flex-col">
      <div class="controls p-4 space-y-2">
        <div class="grid grid-cols-2 gap-2">
          <button
            on:click={handleImportArchive}
            class="px-4 py-2 dark:bg-secondary dark:text-white rounded hover:dark:bg-secondary/80 disabled:opacity-50"
            disabled={isProcessing}
          >
            {isProcessing ? 'Processing...' : 'Import Archive'}
          </button>
          <button
            on:click={startArchive}
            class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:dark:bg-primary-dark disabled:opacity-50"
            disabled={isProcessing}
          >
            {isProcessing ? 'Archiving...' : 'Export to Archive'}
          </button>
        </div>
      </div>
      <div class="progress-section flex-1 p-4">
        {#if progress.stage === 'archiving' || isProcessing}
          <div class="progress-info mb-4">
            <div class="dark:text-gray-300 mb-2">
              {progress.details || 'Processing...'}
            </div>
            <div class="progress-bar dark:bg-gray-700 rounded-full h-4 overflow-hidden mb-2">
              <div
                class="dark:bg-primary h-full transition-all duration-300"
                style="width: {progress.total > 0 ? (progress.current / progress.total) * 100 : 0}%"
              ></div>
            </div>
            <div class="stats flex justify-between dark:text-gray-300 text-sm">
              <span>{progress.current} / {progress.total}</span>
              <span>Rate: {formatRate(progress.rate)}</span>
              <span>ETA: {formatETA(progress.eta_seconds)}</span>
            </div>
            <button
              on:click={cancelOperation}
              class="mt-2 px-4 py-2 dark:bg-red-600 dark:text-white rounded hover:dark:bg-red-500"
            >
              Cancel
            </button>
          </div>
        {:else}
          <div class="no-progress dark:text-gray-400 text-center py-8">
            No archiving in progress
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Errors -->
  {#if errors.length > 0}
    <div class="errors mt-4 p-3 dark:bg-red-900/20 dark:border dark:border-red-500 rounded">
      <h4 class="dark:text-red-300 mb-2">Errors</h4>
      {#each errors.slice(-3) as error, index (index)}
        <div class="dark:text-red-400 text-sm mb-1">
          {error}
        </div>
      {/each}
      {#if errors.length > 3}
        <button
          on:click={() => (errors = [])}
          class="mt-2 px-3 py-1 dark:bg-red-600 dark:text-white rounded hover:dark:bg-red-500 text-xs"
        >
          Clear Errors
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .pipeline-window {
    height: 100%;
  }

  .tab-btn {
    min-width: 80px;
    transition: all 0.2s ease;
  }

  .drop-zone {
    transition: all 0.3s ease;
    border-style: dashed;
  }

  .progress-bar {
    background-color: var(--bg-secondary);
  }

  .no-progress {
    color: var(--text-muted);
  }
</style>
