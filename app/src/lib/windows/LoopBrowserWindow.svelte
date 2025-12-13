<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api } from '$lib/api';
  import WaveformView from '$lib/components/WaveformView.svelte';
  import { safeListen } from '$lib/utils/tauri';
  import type {
    Loop as ImportedLoop,
    LoopAnalysis as ImportedLoopAnalysis,
    LoopPreview,
  } from '$lib/types';

  // Use imported types for compatibility with api
  type Loop = ImportedLoop;
  type LoopAnalysis = ImportedLoopAnalysis;

  interface BpmRange {
    min: number;
    max: number;
  }

  interface LoopPreviewData {
    waveform: number[];
    currentTime: number;
  }

  // State using Svelte 4 let bindings
  let loops: Loop[] = [];
  let filteredLoops: Loop[] = [];
  let selectedLoop: Loop | null = null;
  let categories: string[] = [];
  let selectedCategory = '';
  let searchQuery = '';
  let sortBy = 'popularity';
  let sortDesc = true;
  const bpmRange: BpmRange = { min: 0, max: 300 };
  let keyFilter = '';
  let isLoading = false;
  let isPlaying = false;
  let currentPlayback: number | null = null;
  let loopPreviewData: LoopPreviewData | null = null;
  let unlistenPlayback: (() => void) | null = null;

  const keySignatures = [
    'C',
    'G',
    'D',
    'A',
    'E',
    'B',
    'F#',
    'C#',
    'F',
    'Bb',
    'Eb',
    'Ab',
    'Db',
    'Gb',
    'Cb',
    'Am',
    'Em',
    'Bm',
    'F#m',
    'C#m',
    'G#m',
    'D#m',
    'A#m',
    'Dm',
    'Gm',
    'Cm',
    'Fm',
    'Bbm',
    'Ebm',
    'Abm',
  ];

  onMount(async () => {
    await loadLoops();

    // Listen for playback events (only in Tauri context)
    unlistenPlayback = await safeListen<{ playing: boolean }>('loop-playback-update', (payload) => {
      isPlaying = payload.playing;
      if (!payload.playing) {
        currentPlayback = null;
      }
    });
  });

  onDestroy(() => {
    if (unlistenPlayback) {
      unlistenPlayback();
    }
  });

  async function loadLoops() {
    isLoading = true;
    try {
      loops = await api.loops.getLoops({
        category: selectedCategory || undefined,
        bpmMin: bpmRange.min || undefined,
        bpmMax: bpmRange.max || undefined,
        key: keyFilter || undefined,
      });

      // Extract unique categories
      const uniqueCategories = new Set<string>();
      loops.forEach((loop) => {
        if (loop.category) {
          uniqueCategories.add(loop.category);
        }
      });
      categories = Array.from(uniqueCategories);

      filterAndSortLoops();
    } catch (error) {
      console.error('Failed to load loops:', error);
    } finally {
      isLoading = false;
    }
  }

  function filterAndSortLoops() {
    const filtered = loops.filter((loop) => {
      const matchesSearch =
        searchQuery === '' ||
        loop.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        loop.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        loop.tags?.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));

      const matchesBpm =
        (!bpmRange.min || (loop.bpm && loop.bpm >= bpmRange.min)) &&
        (!bpmRange.max || (loop.bpm && loop.bpm <= bpmRange.max));

      const matchesKey = !keyFilter || loop.key === keyFilter;

      return matchesSearch && matchesBpm && matchesKey;
    });

    filtered.sort((a, b) => {
      let valA: string | number;
      let valB: string | number;
      switch (sortBy) {
        case 'name':
          valA = a.name.toLowerCase();
          valB = b.name.toLowerCase();
          break;
        case 'bpm':
          valA = a.bpm || 0;
          valB = b.bpm || 0;
          break;
        case 'length':
          valA = a.duration || 0;
          valB = b.duration || 0;
          break;
        case 'popularity':
          valA = a.play_count || 0;
          valB = b.play_count || 0;
          break;
        case 'rating':
          valA = a.rating || 0;
          valB = b.rating || 0;
          break;
        case 'date':
          valA = a.added_date ? new Date(a.added_date).getTime() : 0;
          valB = b.added_date ? new Date(b.added_date).getTime() : 0;
          break;
        default:
          valA = 0;
          valB = 0;
      }

      return sortDesc ? (valA > valB ? -1 : 1) : valA < valB ? -1 : 1;
    });

    filteredLoops = filtered;
  }

  // Reactive statement to re-filter when search/sort changes
  $: {
    searchQuery;
    sortBy;
    sortDesc;
    bpmRange;
    keyFilter;
    filterAndSortLoops();
  }

  async function playLoop(loopId: number) {
    if (currentPlayback === loopId && isPlaying) {
      // Stop playback
      await api.loops.stopPlayback();
      currentPlayback = null;
      isPlaying = false;
    } else {
      // Start playback
      try {
        await api.loops.play(loopId);
        currentPlayback = loopId;
        isPlaying = true;

        // Load preview data
        const preview = await api.loops.getPreview(loopId);
        loopPreviewData = {
          waveform: preview.waveform || preview.waveform_data || [],
          currentTime: preview.currentTime || 0,
        };
      } catch (error) {
        console.error('Failed to play loop:', error);
      }
    }
  }

  async function addToProject(loopId: number) {
    try {
      await api.loops.addToProject(loopId);
      // Show success message
    } catch (error) {
      console.error('Failed to add loop to project:', error);
    }
  }

  async function setLoopTempo(loopId: number, newBpm: number) {
    try {
      await api.loops.setTempo(loopId, newBpm);
      // Update local state
      const loop = loops.find((l) => l.id === loopId);
      if (loop) {
        loop.bpm = newBpm;
        loops = [...loops];
      }
    } catch (error) {
      console.error('Failed to set loop tempo:', error);
    }
  }

  async function analyzeLoop(loopId: number) {
    try {
      const analysis = await api.loops.analyze(loopId);
      if (selectedLoop) {
        selectedLoop = { ...selectedLoop, analysis: analysis };
      }
    } catch (error) {
      console.error('Failed to analyze loop:', error);
    }
  }

  async function rateLoop(loopId: number, rating: number) {
    try {
      await api.loops.rate(loopId, rating);
      // Update local state
      const loop = loops.find((l) => l.id === loopId);
      if (loop) {
        loop.rating = rating;
        loop.rating_count = (loop.rating_count || 0) + 1;
        loops = [...loops];
      }
    } catch (error) {
      console.error('Failed to rate loop:', error);
    }
  }

  async function importLoops() {
    // Implement loop import
  }

  async function scanForLoops() {
    isLoading = true;
    try {
      await api.loops.scanDirectory();
      await loadLoops(); // Reload after scanning
    } catch (error) {
      console.error('Failed to scan for loops:', error);
    } finally {
      isLoading = false;
    }
  }

  function formatDuration(seconds: number): string {
    if (!seconds) {
      return '0:00';
    }
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatFileSize(bytes: number): string {
    if (!bytes) {
      return '0 B';
    }
    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;
    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }
    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  function formatDate(dateString: string): string {
    if (!dateString) {
      return 'Unknown';
    }
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    if (diffDays === 0) {
      return 'Today';
    }
    if (diffDays === 1) {
      return 'Yesterday';
    }
    if (diffDays < 7) {
      return `${diffDays} days ago`;
    }
    if (diffDays < 30) {
      return `${Math.floor(diffDays / 7)} weeks ago`;
    }
    return date.toLocaleDateString();
  }

  function handleSortClick(option: string) {
    if (sortBy === option) {
      sortDesc = !sortDesc;
    } else {
      sortBy = option;
      sortDesc = true;
    }
  }

  function handleLoopSelect(loop: Loop) {
    selectedLoop = loop;
  }

  function handleChangeBpm() {
    if (!selectedLoop) {
      return;
    }
    const newBpm = prompt('Enter new BPM:', String(selectedLoop.bpm || 120));
    if (newBpm) {
      setLoopTempo(selectedLoop.id, parseFloat(newBpm));
    }
  }
</script>

<div class="loop-browser-window dark:bg-window dark:text-app-text h-full flex flex-col">
  <!-- Header -->
  <div class="header p-6 border-b dark:border-window-border">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-2xl dark:text-gray-200">Loop Browser</h2>

      <div class="flex gap-3">
        <button
          on:click={importLoops}
          class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80 flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M9 12h2v-2h2v2h2V8h2l-4-4-4 4h2v4zM5 14h10v2H5v-2z" />
          </svg>
          Import
        </button>

        <button
          on:click={scanForLoops}
          class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80 flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M10 3a7 7 0 017 7h2a9 9 0 00-9-9v2zM3 10a7 7 0 017-7v2a5 5 0 00-5 5H3z" />
          </svg>
          Scan
        </button>
      </div>
    </div>

    <!-- Filters -->
    <div class="filters grid grid-cols-4 gap-4">
      <div class="search">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search loops..."
          class="w-full px-4 py-2 dark:bg-input dark:border-window-border rounded"
        />
      </div>

      <div class="category-filter">
        <select
          bind:value={selectedCategory}
          on:change={loadLoops}
          class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
        >
          <option value="">All Categories</option>
          {#each categories as category (category)}
            <option value={category}>{category}</option>
          {/each}
        </select>
      </div>

      <div class="bpm-filter">
        <div class="flex gap-2">
          <input
            type="number"
            bind:value={bpmRange.min}
            placeholder="Min BPM"
            min="30"
            max="300"
            class="flex-1 px-2 py-2 dark:bg-input dark:border-window-border rounded"
            on:change={loadLoops}
          />
          <span class="flex items-center dark:text-gray-400">-</span>
          <input
            type="number"
            bind:value={bpmRange.max}
            placeholder="Max BPM"
            min="30"
            max="300"
            class="flex-1 px-2 py-2 dark:bg-input dark:border-window-border rounded"
            on:change={loadLoops}
          />
        </div>
      </div>

      <div class="key-filter">
        <select
          bind:value={keyFilter}
          on:change={loadLoops}
          class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
        >
          <option value="">All Keys</option>
          {#each keySignatures as key (key)}
            <option value={key}>{key}</option>
          {/each}
        </select>
      </div>
    </div>

    <!-- Sort Controls -->
    <div class="sort-controls mt-4 flex items-center gap-4">
      <span class="text-sm dark:text-gray-400">Sort by:</span>
      <div class="flex gap-2">
        {#each ['popularity', 'name', 'bpm', 'length', 'rating', 'date'] as option (option)}
          <button
            on:click={() => handleSortClick(option)}
            class="px-3 py-1 text-sm rounded flex items-center gap-1"
            class:dark:bg-primary={sortBy === option}
            class:dark:bg-secondary={sortBy !== option}
          >
            {option.charAt(0).toUpperCase() + option.slice(1)}
            {#if sortBy === option}
              <span>{sortDesc ? '↓' : '↑'}</span>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  </div>

  <!-- Loops Grid -->
  <div class="content flex-1 overflow-auto p-6">
    {#if isLoading}
      <div class="loading dark:text-gray-500 text-center py-12">
        <div
          class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 dark:border-gray-400 mb-2"
        ></div>
        <div>Loading loops...</div>
      </div>
    {:else if loops.length === 0}
      <div class="empty-state text-center py-12">
        <div class="text-6xl mb-4 dark:text-gray-600">🔁</div>
        <h3 class="text-xl dark:text-gray-300 mb-2">No loops found</h3>
        <p class="dark:text-gray-500 mb-6">Import or scan for loops to get started.</p>
        <button
          on:click={scanForLoops}
          class="px-6 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
        >
          Scan for Loops
        </button>
      </div>
    {:else}
      <div class="loops-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {#each filteredLoops as loop (loop.id)}
          <div
            class="loop-card dark:bg-window-subtle rounded-lg border dark:border-window-border overflow-hidden hover:shadow-lg transition-shadow cursor-pointer group"
            class:dark:border-primary={selectedLoop?.id === loop.id}
            on:click={() => handleLoopSelect(loop)}
            on:keydown={(e) => e.key === 'Enter' && handleLoopSelect(loop)}
            role="button"
            tabindex="0"
          >
            <!-- Loop Header -->
            <div class="p-4 border-b dark:border-window-border">
              <div class="flex justify-between items-start mb-2">
                <h3 class="font-medium dark:text-gray-200 truncate flex-1" title={loop.name}>
                  {loop.name}
                </h3>

                <div class="flex items-center gap-1">
                  <!-- Rating Stars -->
                  <div class="flex">
                    {#each Array(5) as _, i (i)}
                      <button
                        on:click|stopPropagation={() => rateLoop(loop.id, i + 1)}
                        class="text-lg"
                        class:text-yellow-400={i < Math.round(loop.rating || 0)}
                        class:text-gray-600={i >= Math.round(loop.rating || 0)}
                      >
                        ★
                      </button>
                    {/each}
                  </div>
                </div>
              </div>

              <div class="flex justify-between items-center text-sm">
                <span class="dark:text-gray-400">{loop.category || 'Uncategorized'}</span>
                <span class="dark:text-gray-300">
                  {loop.bpm ? `${Math.round(loop.bpm)} BPM` : 'Unknown BPM'}
                </span>
              </div>
            </div>

            <!-- Waveform Preview -->
            <div class="aspect-video dark:bg-menu relative">
              <WaveformView
                audioData={loopPreviewData?.waveform || []}
                currentTime={loopPreviewData?.currentTime || 0}
                duration={loop.duration}
                playing={currentPlayback === loop.id && isPlaying}
                height={120}
              />

              <!-- Playback Controls Overlay -->
              <div
                class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-40 transition-all flex items-center justify-center opacity-0 group-hover:opacity-100"
              >
                <div class="flex gap-3">
                  <button
                    on:click|stopPropagation={() => playLoop(loop.id)}
                    class="p-3 rounded-full hover:scale-110 transition-transform"
                    class:dark:bg-primary={!(currentPlayback === loop.id && isPlaying)}
                    class:dark:bg-error={currentPlayback === loop.id && isPlaying}
                    title={currentPlayback === loop.id && isPlaying ? 'Stop' : 'Play'}
                  >
                    {currentPlayback === loop.id && isPlaying ? '■' : '▶'}
                  </button>

                  <button
                    on:click|stopPropagation={() => addToProject(loop.id)}
                    class="p-3 dark:bg-secondary rounded-full hover:scale-110 transition-transform"
                    title="Add to Project"
                  >
                    +
                  </button>

                  <button
                    on:click|stopPropagation={() => analyzeLoop(loop.id)}
                    class="p-3 dark:bg-secondary rounded-full hover:scale-110 transition-transform"
                    title="Analyze"
                  >
                    🔍
                  </button>
                </div>
              </div>

              <!-- Loop Info Badge -->
              <div class="absolute top-2 right-2">
                <div class="flex gap-1">
                  <span class="px-2 py-1 text-xs dark:bg-menu rounded-full dark:text-gray-400">
                    {formatDuration(loop.duration ?? 0)}
                  </span>
                  {#if loop.key}
                    <span class="px-2 py-1 text-xs dark:bg-menu rounded-full dark:text-gray-400">
                      {loop.key}
                    </span>
                  {/if}
                </div>
              </div>
            </div>

            <!-- Loop Details -->
            <div class="p-4">
              <div class="text-xs dark:text-gray-400 mb-3 line-clamp-2">
                {loop.description || 'No description available'}
              </div>

              <div class="flex justify-between items-center text-xs">
                <div class="dark:text-gray-500">
                  {loop.play_count || 0} plays
                </div>

                <div class="flex gap-1">
                  {#each (loop.tags || []).slice(0, 2) as tag (tag)}
                    <span class="px-2 py-0.5 dark:bg-menu rounded-full dark:text-gray-400">
                      {tag}
                    </span>
                  {/each}
                  {#if loop.tags && loop.tags.length > 2}
                    <span class="px-2 py-0.5 dark:text-gray-500">
                      +{loop.tags.length - 2}
                    </span>
                  {/if}
                </div>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Selected Loop Details -->
  {#if selectedLoop}
    <div class="selected-details border-t dark:border-window-border p-6">
      <div class="max-w-6xl mx-auto">
        <div class="flex justify-between items-start mb-6">
          <div class="flex-1">
            <h3 class="text-2xl dark:text-gray-200 mb-2">{selectedLoop.name}</h3>
            <p class="dark:text-gray-400 mb-4">{selectedLoop.description || 'No description'}</p>

            <div class="flex gap-6 text-sm">
              <div class="flex items-center gap-2">
                <span class="dark:text-gray-400">BPM:</span>
                <span class="dark:text-gray-300">{Math.round(selectedLoop.bpm || 0)}</span>
                <button
                  on:click={handleChangeBpm}
                  class="text-xs px-2 py-0.5 dark:bg-secondary rounded hover:opacity-80"
                >
                  Change
                </button>
              </div>

              {#if selectedLoop.key}
                <div class="flex items-center gap-2">
                  <span class="dark:text-gray-400">Key:</span>
                  <span class="dark:text-gray-300">{selectedLoop.key}</span>
                </div>
              {/if}

              <div class="flex items-center gap-2">
                <span class="dark:text-gray-400">Duration:</span>
                <span class="dark:text-gray-300">{formatDuration(selectedLoop.duration ?? 0)}</span>
              </div>

              <div class="flex items-center gap-2">
                <span class="dark:text-gray-400">Plays:</span>
                <span class="dark:text-gray-300">{selectedLoop.play_count || 0}</span>
              </div>
            </div>
          </div>

          <div class="flex gap-3">
            <button
              on:click={() => selectedLoop && playLoop(selectedLoop.id)}
              class="px-4 py-2 rounded flex items-center gap-2"
              class:dark:bg-error={selectedLoop && currentPlayback === selectedLoop.id && isPlaying}
              class:dark:bg-primary={!(
                selectedLoop &&
                currentPlayback === selectedLoop.id &&
                isPlaying
              )}
            >
              {selectedLoop && currentPlayback === selectedLoop.id && isPlaying
                ? '■ Stop'
                : '▶ Play'}
            </button>
            <button
              on:click={() => selectedLoop && addToProject(selectedLoop.id)}
              class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
            >
              Add to Project
            </button>
            <button
              on:click={() => selectedLoop && analyzeLoop(selectedLoop.id)}
              class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
            >
              Analyze
            </button>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-6">
          <!-- Left Column: Waveform & Analysis -->
          <div class="col-span-2">
            <!-- Waveform -->
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border mb-6">
              <WaveformView
                audioData={loopPreviewData?.waveform || []}
                currentTime={loopPreviewData?.currentTime || 0}
                duration={selectedLoop.duration}
                playing={currentPlayback === selectedLoop.id && isPlaying}
                height={200}
                showControls={true}
              />
            </div>

            <!-- Analysis Results -->
            {#if selectedLoop.analysis}
              <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
                <h4 class="text-lg dark:text-gray-300 mb-3">Analysis Results</h4>
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <h5 class="text-sm dark:text-gray-400 mb-2">Spectral Analysis</h5>
                    <div class="space-y-1 text-sm">
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Brightness:</span>
                        <span class="dark:text-gray-300">
                          {selectedLoop.analysis.spectral?.brightness?.toFixed(2) ?? 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Flatness:</span>
                        <span class="dark:text-gray-300">
                          {selectedLoop.analysis.spectral?.flatness?.toFixed(2) ?? 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Centroid:</span>
                        <span class="dark:text-gray-300">
                          {selectedLoop.analysis.spectral?.centroid?.toFixed(0) ?? 'N/A'} Hz
                        </span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h5 class="text-sm dark:text-gray-400 mb-2">Rhythm Analysis</h5>
                    <div class="space-y-1 text-sm">
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Tempo Confidence:</span>
                        <span class="dark:text-gray-300">
                          {selectedLoop.analysis.rhythm?.tempo_confidence
                            ? (selectedLoop.analysis.rhythm.tempo_confidence * 100).toFixed(0) + '%'
                            : 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Onset Strength:</span>
                        <span class="dark:text-gray-300">
                          {selectedLoop.analysis.rhythm?.onset_strength?.toFixed(2) ?? 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Beat Positions:</span>
                        <span class="dark:text-gray-300">
                          {selectedLoop.analysis.rhythm?.beat_count ?? 'N/A'}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            {/if}
          </div>

          <!-- Right Column: Metadata & Actions -->
          <div class="space-y-6">
            <!-- Metadata -->
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h4 class="text-lg dark:text-gray-300 mb-3">Metadata</h4>
              <div class="space-y-3 text-sm">
                <div>
                  <div class="dark:text-gray-400 mb-1">File Information</div>
                  <div class="space-y-1 pl-2">
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Size:</span>
                      <span class="dark:text-gray-300"
                        >{formatFileSize(selectedLoop.file_size ?? 0)}</span
                      >
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Format:</span>
                      <span class="dark:text-gray-300">{selectedLoop.format}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Channels:</span>
                      <span class="dark:text-gray-300">{selectedLoop.channels}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Sample Rate:</span>
                      <span class="dark:text-gray-300">{selectedLoop.sample_rate} Hz</span>
                    </div>
                  </div>
                </div>

                <div>
                  <div class="dark:text-gray-400 mb-1">Timestamps</div>
                  <div class="space-y-1 pl-2">
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Added:</span>
                      <span class="dark:text-gray-300"
                        >{selectedLoop.added_date
                          ? formatDate(selectedLoop.added_date)
                          : 'Unknown'}</span
                      >
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Last Played:</span>
                      <span class="dark:text-gray-300">
                        {selectedLoop.last_played ? formatDate(selectedLoop.last_played) : 'Never'}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Tags -->
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h4 class="text-lg dark:text-gray-300 mb-3">Tags</h4>
              <div class="flex flex-wrap gap-2">
                {#each selectedLoop.tags || [] as tag (tag)}
                  <span class="px-3 py-1 dark:bg-secondary rounded-full text-sm dark:text-gray-300">
                    {tag}
                  </span>
                {/each}
              </div>
            </div>

            <!-- Compatibility -->
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h4 class="text-lg dark:text-gray-300 mb-3">Compatibility</h4>
              <div class="space-y-2 text-sm">
                <div class="flex items-center gap-2">
                  <div
                    class="w-3 h-3 rounded-full"
                    class:dark:bg-green-500={(selectedLoop.bpm || 0) >= 100 &&
                      (selectedLoop.bpm || 0) <= 140}
                    class:dark:bg-yellow-500={((selectedLoop.bpm || 0) >= 80 &&
                      (selectedLoop.bpm || 0) < 100) ||
                      ((selectedLoop.bpm || 0) > 140 && (selectedLoop.bpm || 0) <= 160)}
                    class:dark:bg-red-500={(selectedLoop.bpm || 0) < 80 ||
                      (selectedLoop.bpm || 0) > 160}
                  ></div>
                  <span class="dark:text-gray-300">BPM: {Math.round(selectedLoop.bpm || 0)}</span>
                </div>

                <div class="flex items-center gap-2">
                  <div
                    class="w-3 h-3 rounded-full"
                    class:dark:bg-green-500={selectedLoop.key === 'C' || selectedLoop.key === 'Am'}
                    class:dark:bg-yellow-500={['G', 'Em', 'F', 'Dm'].includes(
                      selectedLoop.key || ''
                    )}
                    class:dark:bg-red-500={!selectedLoop.key ||
                      !['C', 'Am', 'G', 'Em', 'F', 'Dm'].includes(selectedLoop.key || '')}
                  ></div>
                  <span class="dark:text-gray-300">Key: {selectedLoop.key || 'Unknown'}</span>
                </div>
              </div>
            </div>

            <!-- Quick Actions -->
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h4 class="text-lg dark:text-gray-300 mb-3">Quick Actions</h4>
              <div class="space-y-2">
                <button
                  on:click={handleChangeBpm}
                  class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80 text-sm"
                >
                  Change BPM
                </button>

                <button
                  on:click={() => {
                    /* Create variation */
                  }}
                  class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80 text-sm"
                >
                  Create Variation
                </button>

                <button
                  on:click={() => {
                    /* Export loop */
                  }}
                  class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80 text-sm"
                >
                  Export Loop
                </button>

                <button
                  on:click={() => {
                    /* Show similar loops */
                  }}
                  class="w-full px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80 text-sm"
                >
                  Find Similar Loops
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .loop-card {
    transition:
      transform 0.2s,
      box-shadow 0.2s;
  }

  .loop-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .waveform-container {
    position: relative;
    overflow: hidden;
  }

  .waveform-progress {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: rgba(59, 130, 246, 0.3);
    pointer-events: none;
    transition: width 0.1s linear;
  }
</style>
