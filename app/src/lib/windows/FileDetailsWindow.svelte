<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import WaveformView from '$lib/components/WaveformView.svelte';
  import TagCloud from '$lib/components/TagCloud.svelte';
  import type { CompatibleFile, FileDetails, Tag } from '$lib/types';

  export let fileId: number;

  // Analysis track interface for display
  interface AnalysisTrack {
    number: number;
    name?: string;
    channel?: number;
    note_count: number;
    average_velocity: number;
    lowest_pitch?: number;
    highest_pitch?: number;
    duration_seconds: number;
  }

  // File-specific analysis results interface (different from batch AnalysisResults)
  interface FileAnalysisResults {
    // Core analysis
    bpm?: number;
    key_signature?: string;
    time_signature?: string;
    duration_seconds?: number;
    total_notes?: number;
    note_count?: number;
    track_count?: number;
    channels_used?: number[];

    // Track data
    tracks?: AnalysisTrack[];
    note_distribution?: number[];

    // Rhythm analysis
    tempo_stability?: number;
    beat_strength?: number;
    syncopation?: number;

    // Harmonic analysis
    chords?: string[];
    key_changes?: { key: string; time: number }[];
    chord_changes?: number;
    modulations?: number;
    dissonance?: number;

    // Velocity and polyphony
    polyphony?: number;
    max_polyphony?: number;
    average_velocity?: number;
    note_density?: number;

    // Event statistics
    total_events?: number;
    note_events?: number;
    control_events?: number;
    program_events?: number;
    pitch_range?: { low: number; high: number };

    // Technical metadata
    chunks?: { type: string; size: number }[];
    loaded_size?: number;
    parsed_size?: number;
    cache_size?: number;
    load_time?: { disk: number; parse: number; total: number };
    events_per_second?: number;
    notes_per_second?: number;

    // Quality metrics
    errors?: number;
    warnings?: number;
    validity?: number;
    compression_ratio?: number;
    efficiency?: number;
  }

  // State
  let fileDetails: FileDetails | null = null;
  let similarFiles: CompatibleFile[] = [];
  let analysisResults: FileAnalysisResults | null = null;
  let waveformData: number[] | null = null;
  let isPlaying = false;
  let selectedTab = 'overview';
  let isLoading = true;
  let error: string | null = null;

  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  const tabs = [
    { id: 'overview', name: 'Overview', icon: '📊' },
    { id: 'analysis', name: 'Analysis', icon: '🔬' },
    { id: 'tags', name: 'Tags', icon: '🏷️' },
    { id: 'similar', name: 'Similar Files', icon: '🔍' },
    { id: 'metadata', name: 'Metadata', icon: '📝' },
    { id: 'technical', name: 'Technical', icon: '⚙️' },
  ];

  onMount(async () => {
    await loadFileDetails();
  });

  async function loadFileDetails() {
    isLoading = true;
    error = null;

    try {
      fileDetails = await api.files.getFileDetails(fileId);
      similarFiles = await api.files.findCompatibleFiles(fileId);
      // Cast to FileAnalysisResults - backend may return richer data than batch AnalysisResults
      analysisResults = (await api.files.analyzeFile(fileId)) as unknown as FileAnalysisResults;
      waveformData = await api.files.getWaveformData(fileId);
    } catch (err) {
      error = String(err);
      console.error('Failed to load file details:', err);
    } finally {
      isLoading = false;
    }
  }

  async function playFile() {
    if (isPlaying) {
      await api.transport.stop();
      isPlaying = false;
    } else {
      await api.transport.playFile(fileId);
      isPlaying = true;
    }
  }

  async function addToFavorites() {
    if (!fileDetails) {
      return;
    }
    try {
      await api.favorites.add(fileId);
      fileDetails = { ...fileDetails, is_favorite: true };
    } catch (err) {
      console.error('Failed to add to favorites:', err);
    }
  }

  async function removeFromFavorites() {
    if (!fileDetails) {
      return;
    }
    try {
      await api.favorites.remove(fileId);
      fileDetails = { ...fileDetails, is_favorite: false };
    } catch (err) {
      console.error('Failed to remove from favorites:', err);
    }
  }

  async function loadIntoSequencer() {
    try {
      // Add file as a new track to the sequencer (channel 0 = default)
      await api.sequencer.addTrack(fileId, 0);
    } catch (err) {
      console.error('Failed to load into sequencer:', err);
    }
  }

  async function exportFile() {
    try {
      await api.export.exportFile(fileId);
    } catch (err) {
      console.error('Failed to export file:', err);
    }
  }

  async function deleteFile() {
    if (!confirm('Are you sure you want to delete this file?')) {
      return;
    }

    try {
      await api.files.deleteFile(fileId);
      // Close window or navigate away
    } catch (err) {
      console.error('Failed to delete file:', err);
    }
  }

  async function analyzeFile() {
    try {
      analysisResults = (await api.files.analyzeFile(fileId)) as unknown as FileAnalysisResults;
    } catch (err) {
      console.error('Failed to analyze file:', err);
    }
  }

  async function addTags() {
    const tagInput = prompt('Enter tags to add (comma-separated):');
    if (!tagInput) return;

    const newTags = tagInput.split(',').map(t => t.trim()).filter(t => t.length > 0);
    if (newTags.length === 0) return;

    try {
      await api.tags.addToFile(fileId, newTags);
      // Refresh file details to show updated tags
      await loadFileDetails();
    } catch (err) {
      console.error('Failed to add tags:', err);
    }
  }

  async function editTags() {
    if (!fileDetails?.tags) return;

    const currentTags = fileDetails.tags.join(', ');
    const tagInput = prompt('Edit tags (comma-separated):', currentTags);
    if (tagInput === null) return;

    const updatedTags = tagInput.split(',').map(t => t.trim()).filter(t => t.length > 0);

    try {
      await api.tags.updateFileTags(fileId, updatedTags);
      // Refresh file details to show updated tags
      await loadFileDetails();
    } catch (err) {
      console.error('Failed to update tags:', err);
    }
  }

  async function removeAllTags() {
    if (!confirm('Are you sure you want to remove all tags from this file?')) {
      return;
    }

    try {
      await api.tags.updateFileTags(fileId, []);
      // Refresh file details to show updated tags
      await loadFileDetails();
    } catch (err) {
      console.error('Failed to remove tags:', err);
    }
  }

  async function loadSimilarFile(similarFileId: number) {
    try {
      await api.sequencer.addTrack(similarFileId, 0);
    } catch (err) {
      console.error('Failed to load similar file:', err);
    }
  }

  function viewSimilarFileDetails(similarFileId: number) {
    // Update the current file ID to view the similar file's details
    // This effectively navigates to the new file within the same window
    fileId = similarFileId;
    loadFileDetails();
  }

  function formatSize(bytes: number): string {
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

  function formatDuration(seconds: number): string {
    if (!seconds) {
      return '0:00';
    }

    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatDate(dateString: string, full = false): string {
    if (!dateString) {
      return 'Unknown';
    }

    const date = new Date(dateString);
    if (full) {
      return date.toLocaleString();
    } else {
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
  }

  function getNoteName(pitch: number): string {
    const octave = Math.floor(pitch / 12) - 1;
    return `${noteNames[pitch % 12]}${octave}`;
  }

  // Reactive computed values - Tag[] compatible with TagCloud component
  $: tagItems =
    fileDetails?.tags?.map(
      (tag, index): Tag => ({
        id: index,
        name: tag,
        category: 'file',
        count: 1,
      })
    ) || [];

  // Computed values from analysis results
  $: maxNoteDistribution = analysisResults?.note_distribution
    ? Math.max(...analysisResults.note_distribution)
    : 1;
</script>

<div class="file-details-window dark:bg-window dark:text-app-text h-full flex flex-col">
  <!-- Header -->
  <div class="header p-6 border-b dark:border-window-border">
    <div class="flex justify-between items-start">
      <div>
        {#if isLoading}
          <div class="h-8 w-64 dark:bg-window-subtle rounded animate-pulse"></div>
          <div class="h-4 w-48 dark:bg-window-subtle rounded animate-pulse mt-2"></div>
        {:else if error}
          <h2 class="text-2xl dark:text-gray-200 mb-2">Error Loading File</h2>
          <p class="dark:text-error">{error}</p>
        {:else if fileDetails}
          <h2 class="text-2xl dark:text-gray-200 mb-2">{fileDetails.filename}</h2>
          <div class="flex items-center gap-4 text-sm dark:text-gray-400">
            <span>{formatSize(fileDetails.file_size_bytes)}</span>
            <span>•</span>
            <span>{formatDuration(fileDetails.duration_seconds ?? 0)}</span>
            <span>•</span>
            <span>Added {formatDate(fileDetails.created_at)}</span>
          </div>
        {/if}
      </div>

      <div class="flex gap-2">
        <button
          on:click={playFile}
          class="px-4 py-2 rounded flex items-center gap-2"
          class:dark:bg-error={isPlaying}
          class:dark:bg-primary={!isPlaying}
        >
          {isPlaying ? '⏹ Stop' : '▶ Play'}
        </button>

        {#if fileDetails}
          <button
            on:click={fileDetails.is_favorite ? removeFromFavorites : addToFavorites}
            class="px-4 py-2 rounded flex items-center gap-2"
            class:dark:bg-secondary={!fileDetails.is_favorite}
            class:dark:bg-yellow-500={fileDetails.is_favorite}
          >
            {fileDetails.is_favorite ? '★ Favorited' : '☆ Add to Favorites'}
          </button>

          <button
            on:click={loadIntoSequencer}
            class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80 flex items-center gap-2"
          >
            🎹 Load into Sequencer
          </button>
        {/if}
      </div>
    </div>
  </div>

  <!-- Tab Navigation -->
  <div class="tabs border-b dark:border-window-border">
    <div class="flex">
      {#each tabs as tab (tab.id)}
        <button
          on:click={() => (selectedTab = tab.id)}
          class="px-6 py-3 border-b-2 flex items-center gap-2"
          class:dark:border-primary={selectedTab === tab.id}
          class:dark:border-transparent={selectedTab !== tab.id}
          class:dark:text-gray-300={selectedTab !== tab.id}
        >
          <span>{tab.icon}</span>
          <span>{tab.name}</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Content -->
  <div class="content flex-1 overflow-auto p-6">
    {#if isLoading}
      <div class="loading dark:text-gray-500 text-center py-12">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 dark:border-gray-400 mb-4"
        ></div>
        <div>Loading file details...</div>
      </div>
    {:else if error}
      <div class="error text-center py-12">
        <div class="text-6xl mb-4 dark:text-error">⚠️</div>
        <h3 class="text-xl dark:text-gray-300 mb-2">Failed to load file</h3>
        <p class="dark:text-gray-400 mb-6">{error}</p>
        <button
          on:click={loadFileDetails}
          class="px-6 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
        >
          Retry
        </button>
      </div>
    {:else if fileDetails}
      <!-- Overview Tab -->
      {#if selectedTab === 'overview'}
        <div class="overview grid grid-cols-3 gap-6">
          <!-- Left Column: Basic Info -->
          <div class="space-y-6">
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h3 class="text-lg dark:text-gray-300 mb-3">File Information</h3>
              <div class="space-y-2 text-sm">
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Filename:</span>
                  <span class="dark:text-gray-200">{fileDetails.filename}</span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Size:</span>
                  <span class="dark:text-gray-200">{formatSize(fileDetails.file_size_bytes)}</span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Format:</span>
                  <span class="dark:text-gray-200">MIDI</span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Duration:</span>
                  <span class="dark:text-gray-200"
                    >{formatDuration(fileDetails.duration_seconds ?? 0)}</span
                  >
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Time Signature:</span>
                  <span class="dark:text-gray-200">{fileDetails.time_signature || 'N/A'}</span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Key:</span>
                  <span class="dark:text-gray-200">{fileDetails.key_signature || 'N/A'}</span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Tracks:</span>
                  <span class="dark:text-gray-200">{fileDetails.track_count}</span>
                </div>
              </div>
            </div>

            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h3 class="text-lg dark:text-gray-300 mb-3">File Information</h3>
              <div class="space-y-2 text-sm">
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Created:</span>
                  <span class="dark:text-gray-200">{formatDate(fileDetails.created_at, true)}</span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Category:</span>
                  <span class="dark:text-gray-200"
                    >{fileDetails.primary_category || 'Uncategorized'}</span
                  >
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Collection:</span>
                  <span class="dark:text-gray-200">{fileDetails.collection_name || 'N/A'}</span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Favorite:</span>
                  <span class="dark:text-gray-200">{fileDetails.is_favorite ? 'Yes' : 'No'}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Middle Column: Waveform & Analysis -->
          <div class="col-span-2 space-y-6">
            <!-- Waveform -->
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h3 class="text-lg dark:text-gray-300 mb-3">Waveform Preview</h3>
              {#if waveformData}
                <WaveformView
                  audioData={waveformData}
                  currentTime={0}
                  duration={fileDetails.duration_seconds}
                  playing={isPlaying}
                  height={200}
                  showControls={true}
                />
              {:else}
                <div class="aspect-video dark:bg-menu rounded flex items-center justify-center">
                  <div class="text-center">
                    <div class="text-4xl dark:text-gray-600 mb-2">🎵</div>
                    <div class="dark:text-gray-400">No waveform available</div>
                  </div>
                </div>
              {/if}
            </div>

            <!-- Analysis Summary -->
            <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
              <h3 class="text-lg dark:text-gray-300 mb-3">Analysis Summary</h3>
              {#if analysisResults}
                <div class="grid grid-cols-3 gap-4">
                  <div class="text-center">
                    <div class="text-3xl dark:text-gray-200 mb-1">
                      {fileDetails.bpm ? Math.round(fileDetails.bpm) : 'N/A'}
                    </div>
                    <div class="text-xs dark:text-gray-400">BPM</div>
                  </div>

                  <div class="text-center">
                    <div class="text-3xl dark:text-gray-200 mb-1">
                      {fileDetails.key_signature || 'N/A'}
                    </div>
                    <div class="text-xs dark:text-gray-400">Key</div>
                  </div>

                  <div class="text-center">
                    <div class="text-3xl dark:text-gray-200 mb-1">
                      {analysisResults.note_count || 'N/A'}
                    </div>
                    <div class="text-xs dark:text-gray-400">Notes</div>
                  </div>
                </div>

                <div class="mt-4 grid grid-cols-2 gap-4">
                  <div>
                    <h4 class="text-sm dark:text-gray-400 mb-2">Track Information</h4>
                    <div class="space-y-1 text-sm">
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Tracks:</span>
                        <span class="dark:text-gray-200">{analysisResults.track_count || 0}</span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Channels Used:</span>
                        <span class="dark:text-gray-200"
                          >{analysisResults.channels_used?.length || 0}</span
                        >
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Time Signature:</span>
                        <span class="dark:text-gray-200"
                          >{analysisResults.time_signature || '4/4'}</span
                        >
                      </div>
                    </div>
                  </div>

                  <div>
                    <h4 class="text-sm dark:text-gray-400 mb-2">Complexity</h4>
                    <div class="space-y-1 text-sm">
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Polyphony:</span>
                        <span class="dark:text-gray-200">
                          {analysisResults.polyphony ? analysisResults.polyphony.toFixed(1) : 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Velocity Avg:</span>
                        <span class="dark:text-gray-200">
                          {analysisResults.average_velocity
                            ? Math.round(analysisResults.average_velocity)
                            : 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Note Density:</span>
                        <span class="dark:text-gray-200">
                          {analysisResults.note_density
                            ? analysisResults.note_density.toFixed(2)
                            : 'N/A'}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              {:else}
                <div class="text-center py-6 dark:text-gray-500">
                  <div class="text-4xl mb-2">🔍</div>
                  <div>No analysis available</div>
                  <button
                    on:click={analyzeFile}
                    class="mt-3 px-4 py-2 dark:bg-secondary rounded hover:opacity-80 text-sm"
                  >
                    Analyze File
                  </button>
                </div>
              {/if}
            </div>
          </div>
        </div>

        <!-- Analysis Tab -->
      {:else if selectedTab === 'analysis'}
        <div class="analysis space-y-6">
          {#if analysisResults}
            <!-- Track Analysis -->
            <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
              <h3 class="text-xl dark:text-gray-300 mb-4">Track Analysis</h3>
              <div class="overflow-x-auto">
                <table class="w-full text-sm">
                  <thead>
                    <tr class="border-b dark:border-window-border">
                      <th class="p-3 text-left dark:text-gray-400">Track</th>
                      <th class="p-3 text-left dark:text-gray-400">Channel</th>
                      <th class="p-3 text-left dark:text-gray-400">Notes</th>
                      <th class="p-3 text-left dark:text-gray-400">Avg Velocity</th>
                      <th class="p-3 text-left dark:text-gray-400">Pitch Range</th>
                      <th class="p-3 text-left dark:text-gray-400">Duration</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each analysisResults.tracks || [] as track (track.number)}
                      <tr class="border-b dark:border-window-border hover:dark:bg-menu">
                        <td class="p-3 dark:text-gray-200"
                          >{track.name || `Track ${track.number}`}</td
                        >
                        <td class="p-3 dark:text-gray-200"
                          >{track.channel !== undefined ? track.channel + 1 : 'Multi'}</td
                        >
                        <td class="p-3 dark:text-gray-200">{track.note_count}</td>
                        <td class="p-3">
                          <div class="flex items-center gap-2">
                            <div class="flex-1 dark:bg-menu rounded-full h-2 overflow-hidden">
                              <div
                                class="dark:bg-primary h-full"
                                style="width: {(track.average_velocity / 127) * 100}%"
                              ></div>
                            </div>
                            <span class="w-8 text-right">{Math.round(track.average_velocity)}</span>
                          </div>
                        </td>
                        <td class="p-3 dark:text-gray-200">
                          {track.lowest_pitch ? getNoteName(track.lowest_pitch) : ''} -
                          {track.highest_pitch ? getNoteName(track.highest_pitch) : ''}
                        </td>
                        <td class="p-3 dark:text-gray-200">
                          {formatDuration(track.duration_seconds)}
                        </td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Musical Analysis -->
            <div class="grid grid-cols-2 gap-6">
              <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
                <h3 class="text-xl dark:text-gray-300 mb-4">Musical Analysis</h3>
                <div class="space-y-4">
                  <div>
                    <h4 class="text-sm dark:text-gray-400 mb-2">Note Distribution</h4>
                    <div class="h-48 relative flex items-end gap-1">
                      {#each analysisResults.note_distribution || [] as note, index (index)}
                        <div
                          class="flex-1 dark:bg-primary rounded-t"
                          style="height: {(note / maxNoteDistribution) * 100}%"
                          title="{noteNames[index % 12]}: {note} notes"
                        ></div>
                      {/each}
                    </div>
                    <div class="flex justify-between text-xs dark:text-gray-500 px-1 mt-1">
                      <span>C</span><span>D</span><span>E</span><span>F</span><span>G</span><span
                        >A</span
                      ><span>B</span>
                    </div>
                  </div>

                  <div>
                    <h4 class="text-sm dark:text-gray-400 mb-2">Rhythm Analysis</h4>
                    <div class="space-y-2 text-sm">
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Tempo Stability:</span>
                        <span class="dark:text-gray-200">
                          {analysisResults.tempo_stability
                            ? (analysisResults.tempo_stability * 100).toFixed(0) + '%'
                            : 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Beat Strength:</span>
                        <span class="dark:text-gray-200">
                          {analysisResults.beat_strength
                            ? analysisResults.beat_strength.toFixed(2)
                            : 'N/A'}
                        </span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Syncopation:</span>
                        <span class="dark:text-gray-200">
                          {analysisResults.syncopation
                            ? analysisResults.syncopation.toFixed(2)
                            : 'N/A'}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
                <h3 class="text-xl dark:text-gray-300 mb-4">Harmonic Analysis</h3>
                <div class="space-y-4">
                  <div>
                    <h4 class="text-sm dark:text-gray-400 mb-2">Chord Progression</h4>
                    <div class="flex flex-wrap gap-2">
                      {#each analysisResults.chords || [] as chord, index (index)}
                        <span
                          class="px-3 py-1 dark:bg-secondary rounded-full text-sm dark:text-gray-300"
                        >
                          {chord}
                        </span>
                      {/each}
                    </div>
                  </div>

                  <div>
                    <h4 class="text-sm dark:text-gray-400 mb-2">Key Changes</h4>
                    <div class="space-y-2">
                      {#each analysisResults.key_changes || [] as change, index (index)}
                        <div class="flex justify-between text-sm">
                          <span class="dark:text-gray-200">{change.key}</span>
                          <span class="dark:text-gray-400">{formatDuration(change.time)}</span>
                        </div>
                      {/each}
                    </div>
                  </div>

                  <div>
                    <h4 class="text-sm dark:text-gray-400 mb-2">Harmonic Complexity</h4>
                    <div class="space-y-2 text-sm">
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Chord Changes:</span>
                        <span class="dark:text-gray-200">{analysisResults.chord_changes || 0}</span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Modulations:</span>
                        <span class="dark:text-gray-200">{analysisResults.modulations || 0}</span>
                      </div>
                      <div class="flex justify-between">
                        <span class="dark:text-gray-400">Dissonance:</span>
                        <span class="dark:text-gray-200">
                          {analysisResults.dissonance
                            ? analysisResults.dissonance.toFixed(2)
                            : 'N/A'}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          {:else}
            <div class="text-center py-12 dark:text-gray-500">
              <div class="text-6xl mb-4">🔍</div>
              <h3 class="text-xl dark:text-gray-300 mb-2">No Analysis Available</h3>
              <p class="dark:text-gray-400 mb-6">This file hasn't been analyzed yet.</p>
              <button
                on:click={analyzeFile}
                class="px-6 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
              >
                Analyze Now
              </button>
            </div>
          {/if}
        </div>

        <!-- Tags Tab -->
      {:else if selectedTab === 'tags'}
        <div class="tags space-y-6">
          <div class="grid grid-cols-3 gap-6">
            <div class="col-span-2">
              <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
                <h3 class="text-xl dark:text-gray-300 mb-4">File Tags</h3>
                {#if tagItems.length > 0}
                  <TagCloud tags={tagItems} maxFontSize={48} minFontSize={16} />
                {:else}
                  <div class="text-center py-12 dark:text-gray-500">
                    <div class="text-6xl mb-4">🏷️</div>
                    <div>No tags assigned</div>
                  </div>
                {/if}
              </div>
            </div>

            <div class="space-y-6">
              <!-- Tag Statistics -->
              <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
                <h4 class="text-lg dark:text-gray-300 mb-3">Tag Statistics</h4>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Total Tags:</span>
                    <span class="dark:text-gray-200">{fileDetails.tags?.length || 0}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Most Common:</span>
                    <span class="dark:text-gray-200">
                      {fileDetails.tags?.[0] || 'None'}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Category:</span>
                    <span class="dark:text-gray-200">
                      {fileDetails.primary_category || 'Uncategorized'}
                    </span>
                  </div>
                </div>
              </div>

              <!-- Tag Actions -->
              <div class="dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border">
                <h4 class="text-lg dark:text-gray-300 mb-3">Tag Management</h4>
                <div class="space-y-3">
                  <button
                    on:click={addTags}
                    class="w-full px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
                  >
                    Add Tags
                  </button>
                  <button
                    on:click={editTags}
                    class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
                  >
                    Edit Tags
                  </button>
                  <button
                    on:click={removeAllTags}
                    class="w-full px-4 py-2 dark:bg-error rounded hover:opacity-80"
                  >
                    Remove All Tags
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Similar Files Tab -->
      {:else if selectedTab === 'similar'}
        <div class="similar-files">
          <h3 class="text-xl dark:text-gray-300 mb-4">Similar Files</h3>

          {#if similarFiles.length > 0}
            <div class="grid grid-cols-2 gap-4">
              {#each similarFiles as similar (similar.id)}
                <div
                  class="similar-file p-4 dark:bg-window-subtle rounded border dark:border-window-border hover:dark:bg-menu transition-colors"
                >
                  <div class="flex justify-between items-start mb-3">
                    <div>
                      <h4 class="font-medium dark:text-gray-200">{similar.filename}</h4>
                      <div class="text-sm dark:text-gray-400">
                        {formatDuration(similar.duration_seconds ?? 0)} • {formatSize(
                          similar.file_size_bytes ?? 0
                        )}
                      </div>
                    </div>
                    <div class="text-sm dark:text-gray-300">
                      {Math.round((similar.similarity ?? 0) * 100)}% match
                    </div>
                  </div>

                  <div class="grid grid-cols-3 gap-2 text-xs mb-3">
                    <div class="text-center">
                      <div class="dark:text-gray-400">BPM</div>
                      <div class="dark:text-gray-200">
                        {similar.bpm ? Math.round(similar.bpm) : 'N/A'}
                      </div>
                    </div>
                    <div class="text-center">
                      <div class="dark:text-gray-400">Key</div>
                      <div class="dark:text-gray-200">{similar.key_signature || 'N/A'}</div>
                    </div>
                    <div class="text-center">
                      <div class="dark:text-gray-400">Similarity</div>
                      <div class="dark:text-gray-200">
                        <div class="h-2 dark:bg-menu rounded-full overflow-hidden">
                          <div
                            class="dark:bg-primary h-full"
                            style="width: {(similar.similarity ?? 0) * 100}%"
                          ></div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="flex gap-2">
                    <button
                      on:click={() => loadSimilarFile(similar.id)}
                      class="flex-1 px-3 py-1 text-xs dark:bg-primary dark:text-white rounded hover:opacity-80"
                    >
                      Load
                    </button>
                    <button
                      on:click={() => viewSimilarFileDetails(similar.id)}
                      class="flex-1 px-3 py-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                    >
                      Details
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-12 dark:text-gray-500">
              <div class="text-6xl mb-4">🔍</div>
              <h3 class="text-xl dark:text-gray-300 mb-2">No Similar Files Found</h3>
              <p class="dark:text-gray-400">Try analyzing more files to find matches.</p>
            </div>
          {/if}
        </div>

        <!-- Metadata Tab -->
      {:else if selectedTab === 'metadata'}
        <div class="metadata space-y-6">
          <!-- Basic Metadata -->
          <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
            <h3 class="text-xl dark:text-gray-300 mb-4">Basic Metadata</h3>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <h4 class="text-sm dark:text-gray-400 mb-2">File Properties</h4>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Format:</span>
                    <span class="dark:text-gray-200">MIDI</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Time Signature:</span>
                    <span class="dark:text-gray-200">{fileDetails.time_signature || '4/4'}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Key:</span>
                    <span class="dark:text-gray-200">{fileDetails.key_signature || 'Unknown'}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Tracks:</span>
                    <span class="dark:text-gray-200">{fileDetails.track_count}</span>
                  </div>
                </div>
              </div>

              <div>
                <h4 class="text-sm dark:text-gray-400 mb-2">Technical Details</h4>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">File Size:</span>
                    <span class="dark:text-gray-200">{formatSize(fileDetails.file_size_bytes)}</span
                    >
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Content Hash:</span>
                    <span
                      class="dark:text-gray-200 font-mono text-xs truncate"
                      title={fileDetails.content_hash || ''}
                    >
                      {fileDetails.content_hash || 'N/A'}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Has Notes:</span>
                    <span class="dark:text-gray-200">{fileDetails.has_notes ? 'Yes' : 'No'}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">File Path:</span>
                    <span class="dark:text-gray-200 truncate" title={fileDetails.filepath}>
                      {fileDetails.filepath}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- MIDI Specific Metadata -->
          <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
            <h3 class="text-xl dark:text-gray-300 mb-4">MIDI Metadata</h3>
            <div class="grid grid-cols-2 gap-6">
              <div>
                <h4 class="text-sm dark:text-gray-400 mb-2">MIDI Events</h4>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Total Events:</span>
                    <span class="dark:text-gray-200">{analysisResults?.total_events || 0}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Note Events:</span>
                    <span class="dark:text-gray-200">{analysisResults?.note_events || 0}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Control Events:</span>
                    <span class="dark:text-gray-200">{analysisResults?.control_events || 0}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Program Events:</span>
                    <span class="dark:text-gray-200">{analysisResults?.program_events || 0}</span>
                  </div>
                </div>
              </div>

              <div>
                <h4 class="text-sm dark:text-gray-400 mb-2">Tracks & Channels</h4>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Track Count:</span>
                    <span class="dark:text-gray-200">{analysisResults?.track_count || 0}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Channels Used:</span>
                    <span class="dark:text-gray-200">
                      {analysisResults?.channels_used?.join(', ') || 'None'}
                    </span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Polyphony Max:</span>
                    <span class="dark:text-gray-200">{analysisResults?.max_polyphony || 0}</span>
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Pitch Range:</span>
                    <span class="dark:text-gray-200">
                      {analysisResults?.pitch_range?.low || 'N/A'} - {analysisResults?.pitch_range
                        ?.high || 'N/A'}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Technical Tab -->
      {:else if selectedTab === 'technical'}
        <div class="technical space-y-6">
          <!-- File Structure -->
          <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
            <h3 class="text-xl dark:text-gray-300 mb-4">File Structure</h3>
            <div class="space-y-4">
              <div>
                <h4 class="text-sm dark:text-gray-400 mb-2">Chunks</h4>
                <div class="space-y-2 text-sm">
                  {#each analysisResults?.chunks || [] as chunk, index (index)}
                    <div class="flex justify-between p-2 dark:bg-menu rounded">
                      <span class="dark:text-gray-200">{chunk.type}</span>
                      <span class="dark:text-gray-400">{formatSize(chunk.size)}</span>
                    </div>
                  {/each}
                </div>
              </div>

              <div>
                <h4 class="text-sm dark:text-gray-400 mb-2">Memory Usage</h4>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Loaded Size:</span>
                    <span class="dark:text-gray-200"
                      >{formatSize(analysisResults?.loaded_size || 0)}</span
                    >
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Parsed Size:</span>
                    <span class="dark:text-gray-200"
                      >{formatSize(analysisResults?.parsed_size || 0)}</span
                    >
                  </div>
                  <div class="flex justify-between">
                    <span class="dark:text-gray-400">Cache Size:</span>
                    <span class="dark:text-gray-200"
                      >{formatSize(analysisResults?.cache_size || 0)}</span
                    >
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Performance -->
          <div class="grid grid-cols-2 gap-6">
            <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
              <h3 class="text-xl dark:text-gray-300 mb-4">Performance Metrics</h3>
              <div class="space-y-4">
                <div>
                  <h4 class="text-sm dark:text-gray-400 mb-2">Load Times</h4>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Disk Read:</span>
                      <span class="dark:text-gray-200"
                        >{analysisResults?.load_time?.disk || 0}ms</span
                      >
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Parse:</span>
                      <span class="dark:text-gray-200"
                        >{analysisResults?.load_time?.parse || 0}ms</span
                      >
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Total:</span>
                      <span class="dark:text-gray-200"
                        >{analysisResults?.load_time?.total || 0}ms</span
                      >
                    </div>
                  </div>
                </div>

                <div>
                  <h4 class="text-sm dark:text-gray-400 mb-2">Processing Speed</h4>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Events/sec:</span>
                      <span class="dark:text-gray-200">
                        {analysisResults?.events_per_second
                          ? Math.round(analysisResults.events_per_second)
                          : 'N/A'}
                      </span>
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Notes/sec:</span>
                      <span class="dark:text-gray-200">
                        {analysisResults?.notes_per_second
                          ? Math.round(analysisResults.notes_per_second)
                          : 'N/A'}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="dark:bg-window-subtle p-6 rounded-lg border dark:border-window-border">
              <h3 class="text-xl dark:text-gray-300 mb-4">Quality Metrics</h3>
              <div class="space-y-4">
                <div>
                  <h4 class="text-sm dark:text-gray-400 mb-2">Data Integrity</h4>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Errors:</span>
                      <span class="dark:text-gray-200">{analysisResults?.errors || 0}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Warnings:</span>
                      <span class="dark:text-gray-200">{analysisResults?.warnings || 0}</span>
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Validity:</span>
                      <span class="dark:text-gray-200">
                        {analysisResults?.validity
                          ? (analysisResults.validity * 100).toFixed(0) + '%'
                          : 'N/A'}
                      </span>
                    </div>
                  </div>
                </div>

                <div>
                  <h4 class="text-sm dark:text-gray-400 mb-2">Compression</h4>
                  <div class="space-y-2 text-sm">
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Compression Ratio:</span>
                      <span class="dark:text-gray-200">
                        {analysisResults?.compression_ratio
                          ? analysisResults.compression_ratio.toFixed(2)
                          : 'N/A'}
                      </span>
                    </div>
                    <div class="flex justify-between">
                      <span class="dark:text-gray-400">Efficiency:</span>
                      <span class="dark:text-gray-200">
                        {analysisResults?.efficiency
                          ? (analysisResults.efficiency * 100).toFixed(0) + '%'
                          : 'N/A'}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      {/if}
    {:else}
      <div class="no-file text-center py-12 dark:text-gray-500">
        <div class="text-6xl mb-4">📁</div>
        <h3 class="text-xl dark:text-gray-300 mb-2">No File Selected</h3>
        <p class="dark:text-gray-400">Select a file to view its details</p>
      </div>
    {/if}
  </div>

  <!-- Footer Actions -->
  {#if fileDetails}
    <div class="footer p-4 border-t dark:border-window-border">
      <div class="flex justify-between items-center">
        <div class="text-sm dark:text-gray-400">
          File ID: {fileId} • MIDI • {formatDate(fileDetails.created_at)}
        </div>

        <div class="flex gap-2">
          <button
            on:click={exportFile}
            class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
          >
            Export
          </button>
          <button on:click={deleteFile} class="px-4 py-2 dark:bg-error rounded hover:opacity-80">
            Delete
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .note-distribution-bar {
    transition: height 0.3s;
  }

  .note-distribution-bar:hover {
    opacity: 0.8;
  }

  .similarity-bar {
    transition: width 0.3s;
  }
</style>
