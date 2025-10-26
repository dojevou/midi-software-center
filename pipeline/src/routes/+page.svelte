<script lang="ts">
  import { onMount } from 'svelte';
  import { Music } from 'lucide-svelte';
  import FileBrowser from '$lib/components/FileBrowser/FileBrowser.svelte';
  import ImportButton from '$lib/components/Import/ImportButton.svelte';
  import SearchBar from '$lib/components/Search/SearchBar.svelte';
  import FilterPanel from '$lib/components/Search/FilterPanel.svelte';
  import DetailsPanel from '$lib/components/FileDetails/DetailsPanel.svelte';
  import { uiStore } from '$lib/stores/ui';
  import { checkDatabaseConnection } from '$lib/api';

  let appVersion = '0.1.0';
  let isLoading = true;
  let connectionError = '';

  onMount(async () => {
    try {
      // Attempt to connect to backend
      const isConnected = await checkDatabaseConnection();
      if (!isConnected) {
        connectionError = 'Unable to connect to database. Make sure PostgreSQL is running.';
      } else {
        connectionError = '';
      }
    } catch (error) {
      console.error('Failed to connect to backend:', error);
      connectionError = 'Unable to connect to backend. Make sure Tauri is running.';
    } finally {
      isLoading = false;
    }
  });

  function toggleSidebar() {
    uiStore.toggleSidebar();
  }

  function toggleDetailsPanel() {
    uiStore.toggleDetailsPanel();
  }
</script>

<svelte:head>
  <title>MIDI Library System</title>
</svelte:head>

<div class="flex h-screen bg-slate-50 dark:bg-slate-950">
  <!-- Sidebar -->
  {#if $uiStore.sidebarOpen}
    <aside
      class="w-80 border-r border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 flex flex-col transition-all duration-300 ease-in-out"
      role="complementary"
      aria-label="Sidebar"
    >
      <!-- Header -->
      <div class="p-4 border-b border-slate-200 dark:border-slate-800">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 bg-primary-600 rounded-lg flex items-center justify-center">
            <Music size={20} class="text-white" />
          </div>
          <div>
            <h1 class="text-xl font-bold text-slate-900 dark:text-slate-100">MIDI Library</h1>
            <p class="text-xs text-slate-500 dark:text-slate-400">v{appVersion}</p>
          </div>
        </div>
      </div>

      <!-- Import Section -->
      <div class="p-4 border-b border-slate-200 dark:border-slate-800">
        <ImportButton />
      </div>

      <!-- Filters Section -->
      <div class="flex-1 overflow-auto">
        <FilterPanel />
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-800">
        <div class="text-xs text-center">
          {#if isLoading}
            <div class="flex items-center justify-center gap-2 text-slate-600 dark:text-slate-400">
              <div class="spinner"></div>
              <span>Connecting...</span>
            </div>
          {:else if connectionError}
            <span class="text-red-600 dark:text-red-400">{connectionError}</span>
          {:else}
            <span class="text-green-600 dark:text-green-400">● Ready</span>
          {/if}
        </div>
      </div>
    </aside>
  {/if}

  <!-- Main Content Area -->
  <main class="flex-1 flex flex-col min-w-0">
    <!-- Top Bar -->
    <header class="flex-shrink-0 border-b border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 p-4">
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-4 flex-1">
          <!-- Sidebar Toggle -->
          <button
            on:click={toggleSidebar}
            class="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
            title={$uiStore.sidebarOpen ? 'Hide sidebar' : 'Show sidebar'}
            aria-label="Toggle sidebar"
          >
            {#if $uiStore.sidebarOpen}
              <span class="text-lg">←</span>
            {:else}
              <span class="text-lg">→</span>
            {/if}
          </button>

          <!-- Search Bar -->
          <div class="flex-1 max-w-2xl">
            <SearchBar />
          </div>
        </div>

        <!-- Details Panel Toggle -->
        <button
          on:click={toggleDetailsPanel}
          class="p-2 rounded-lg transition-colors {$uiStore.detailsPanelOpen ? 'bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-400' : 'hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-400'}"
          title={$uiStore.detailsPanelOpen ? 'Hide details' : 'Show details'}
          aria-label="Toggle details panel"
        >
          <span class="text-lg">ⓘ</span>
        </button>
      </div>
    </header>

    <!-- File Browser Area -->
    <div class="flex-1 overflow-hidden">
      {#if isLoading}
        <div class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="spinner mx-auto mb-4"></div>
            <p class="text-slate-600 dark:text-slate-400">Loading MIDI Library...</p>
          </div>
        </div>
      {:else if connectionError}
        <div class="flex items-center justify-center h-full">
          <div class="text-center max-w-md p-8">
            <div class="w-16 h-16 bg-red-100 dark:bg-red-900 rounded-full flex items-center justify-center mx-auto mb-4">
              <span class="text-3xl">⚠</span>
            </div>
            <h2 class="text-xl font-semibold text-slate-900 dark:text-slate-100 mb-2">Connection Error</h2>
            <p class="text-slate-600 dark:text-slate-400 mb-4">{connectionError}</p>
            <button
              on:click={() => window.location.reload()}
              class="btn-primary px-4 py-2"
            >
              Retry Connection
            </button>
          </div>
        </div>
      {:else}
        <FileBrowser />
      {/if}
    </div>
  </main>

  <!-- Details Panel -->
  <DetailsPanel />
</div>
