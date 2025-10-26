<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getFileCount,
    listFiles,
    searchFiles,
    checkDatabaseConnection,
    getErrorMessage,
  } from '$lib/api';
  import LoadingSpinner from '$lib/components/common/LoadingSpinner.svelte';
  import ErrorMessage from '$lib/components/common/ErrorMessage.svelte';

  let loading = true;
  let error: string | null = null;
  let dbConnected = false;
  let fileCount = 0;
  let recentFiles: any[] = [];

  onMount(async () => {
    try {
      // Test database connection
      dbConnected = await checkDatabaseConnection();

      if (dbConnected) {
        // Test file count
        fileCount = await getFileCount();

        // Test list files
        recentFiles = await listFiles(5, 0);
      }
    } catch (e) {
      error = getErrorMessage(e);
    } finally {
      loading = false;
    }
  });

  async function testSearch() {
    try {
      loading = true;
      error = null;
      const results = await searchFiles('');
      console.log('Search results:', results);
      alert(`Found ${results.totalCount} files`);
    } catch (e) {
      error = getErrorMessage(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="container mx-auto p-8">
  <h1 class="text-3xl font-bold mb-6">API Layer Test</h1>

  {#if loading}
    <LoadingSpinner size="lg" message="Testing API..." />
  {:else if error}
    <ErrorMessage {error} />
  {:else}
    <div class="space-y-6">
      <div class="card">
        <h2 class="text-xl font-semibold mb-4">Connection Status</h2>
        <p class="flex items-center gap-2">
          <span class="h-3 w-3 rounded-full {dbConnected ? 'bg-green-500' : 'bg-red-500'}"></span>
          {dbConnected ? 'Connected' : 'Disconnected'}
        </p>
      </div>

      <div class="card">
        <h2 class="text-xl font-semibold mb-4">Statistics</h2>
        <p>Total Files: {fileCount}</p>
      </div>

      <div class="card">
        <h2 class="text-xl font-semibold mb-4">Recent Files</h2>
        {#if recentFiles.length > 0}
          <ul class="space-y-2">
            {#each recentFiles as file}
              <li class="text-sm">
                {file.filename} - {file.category}
              </li>
            {/each}
          </ul>
        {:else}
          <p class="text-slate-600">No files found</p>
        {/if}
      </div>

      <button on:click={testSearch} class="btn-primary px-4 py-2">
        Test Search
      </button>
    </div>
  {/if}
</div>
