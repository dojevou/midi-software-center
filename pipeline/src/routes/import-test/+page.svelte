<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  let status = 'Ready to import';
  let result: any = null;
  let isImporting = false;
  let error: string | null = null;

  async function testSmallArchive() {
    try {
      isImporting = true;
      status = 'Importing DnB test archive...';
      error = null;

      result = await invoke('import_archive_collection', {
        collectionPath: '/tmp/midi_archive_test'
      });

      status = 'Import complete!';
      console.log('Import result:', result);
    } catch (e: any) {
      error = e.toString();
      status = 'Import failed';
      console.error('Import error:', e);
    } finally {
      isImporting = false;
    }
  }

  async function testFullCollection() {
    if (!confirm('This will import 1M+ files and take 1-2 hours. Continue?')) {
      return;
    }

    try {
      isImporting = true;
      status = 'Importing full collection (this will take 1-2 hours)...';
      error = null;

      result = await invoke('import_archive_collection', {
        collectionPath: '/home/dojevou/floorp_downloads/_1.002.000-Midi-Collection_'
      });

      status = 'Full collection import complete!';
      console.log('Import result:', result);
    } catch (e: any) {
      error = e.toString();
      status = 'Import failed';
      console.error('Import error:', e);
    } finally {
      isImporting = false;
    }
  }

  onMount(() => {
    console.log('Import test page loaded');
  });
</script>

<div class="container">
  <h1>Archive Import Test</h1>

  <div class="status-box">
    <h2>Status</h2>
    <p class:importing={isImporting}>{status}</p>
    {#if error}
      <div class="error">
        <strong>Error:</strong> {error}
      </div>
    {/if}
  </div>

  <div class="buttons">
    <button
      on:click={testSmallArchive}
      disabled={isImporting}
      class="btn-primary"
    >
      Test Small Archive (DnB pack)
    </button>

    <button
      on:click={testFullCollection}
      disabled={isImporting}
      class="btn-warning"
    >
      Import Full Collection (1M+ files)
    </button>
  </div>

  {#if result}
    <div class="result-box">
      <h2>Import Results</h2>
      <pre>{JSON.stringify(result, null, 2)}</pre>
    </div>
  {/if}
</div>

<style>
  .container {
    max-width: 800px;
    margin: 2rem auto;
    padding: 2rem;
  }

  h1 {
    color: #333;
    margin-bottom: 2rem;
  }

  .status-box {
    background: #f5f5f5;
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .status-box h2 {
    margin: 0 0 1rem 0;
    font-size: 1.2rem;
  }

  .status-box p {
    margin: 0;
    font-size: 1.1rem;
  }

  .importing {
    color: #0066cc;
    font-weight: bold;
  }

  .error {
    margin-top: 1rem;
    padding: 1rem;
    background: #fee;
    border: 1px solid #fcc;
    border-radius: 4px;
    color: #c00;
  }

  .buttons {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  button {
    padding: 1rem 2rem;
    font-size: 1rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #0066cc;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #0052a3;
  }

  .btn-warning {
    background: #ff8800;
    color: white;
  }

  .btn-warning:hover:not(:disabled) {
    background: #cc6600;
  }

  .result-box {
    background: #f0f9ff;
    padding: 1.5rem;
    border-radius: 8px;
    border: 1px solid #0066cc;
  }

  .result-box h2 {
    margin: 0 0 1rem 0;
    color: #0066cc;
  }

  pre {
    background: white;
    padding: 1rem;
    border-radius: 4px;
    overflow: auto;
    max-height: 400px;
  }
</style>
