<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { save } from '@tauri-apps/plugin-dialog';
  import ProgressIndicator from '$lib/components/ProgressIndicator.svelte';
  import type { ProjectInfo } from '$lib/types';

  interface Track {
    id: number;
    name?: string;
    channel?: number;
    eventCount?: number;
    noteCount?: number;
  }

  interface ExportOptions {
    filename: string;
    location: string;
    includeAllTracks: boolean;
    selectedTracks: number[];
    range: { start: number; end: number };
    loop: boolean;
    normalize: boolean;
    dither: boolean;
    midiFormat: number;
    includeMarkers: boolean;
    includeTempoChanges: boolean;
    includeTimeSignatures: boolean;
    includeKeySignatures: boolean;
    resolution: number;
    sampleRate: number;
    bitDepth: number;
    channels: number;
    format: string;
    quality: number;
    mp3Bitrate: number;
    exportMetadata: boolean;
    embedAlbumArt: boolean;
    includeProjectInfo: boolean;
    createZip: boolean;
    compressionLevel?: number;
  }

  interface ExportProgress {
    status: 'idle' | 'preparing' | 'exporting' | 'completed' | 'error';
    current: number;
    total: number;
    message: string;
    errors: string[];
  }

  interface FormatInfo {
    id: string;
    name: string;
    extensions: string[];
  }

  // State using Svelte 4 let bindings
  let exportFormat = 'midi';
  let exportOptions: ExportOptions = {
    filename: '',
    location: '',
    includeAllTracks: true,
    selectedTracks: [],
    range: { start: 0, end: 0 },
    loop: false,
    normalize: false,
    dither: false,
    midiFormat: 1,
    includeMarkers: true,
    includeTempoChanges: true,
    includeTimeSignatures: true,
    includeKeySignatures: true,
    resolution: 480,
    sampleRate: 44100,
    bitDepth: 16,
    channels: 2,
    format: 'wav',
    quality: 90,
    mp3Bitrate: 192,
    exportMetadata: true,
    embedAlbumArt: false,
    includeProjectInfo: true,
    createZip: false,
  };

  let exportProgress: ExportProgress = {
    status: 'idle',
    current: 0,
    total: 0,
    message: '',
    errors: [],
  };

  let tracks: Track[] = [];
  let projectInfo: ProjectInfo = {};
  let progressInterval: ReturnType<typeof setInterval> | null = null;

  const availableFormats: FormatInfo[] = [
    { id: 'midi', name: 'MIDI File', extensions: ['.mid', '.midi'] },
    { id: 'wav', name: 'WAV Audio', extensions: ['.wav'] },
    { id: 'mp3', name: 'MP3 Audio', extensions: ['.mp3'] },
    { id: 'flac', name: 'FLAC Audio', extensions: ['.flac'] },
    { id: 'ogg', name: 'OGG Vorbis', extensions: ['.ogg'] },
    { id: 'zip', name: 'Project Archive', extensions: ['.zip'] },
  ];

  const formatPresets = {
    midi: {
      'Standard MIDI': { midiFormat: 1, resolution: 480, includeMarkers: true },
      'Type 0 (Single Track)': { midiFormat: 0, resolution: 480, includeMarkers: false },
      'GM Compatible': { midiFormat: 1, resolution: 480, includeMarkers: false },
    },
    audio: {
      'CD Quality': { sampleRate: 44100, bitDepth: 16, format: 'wav', channels: 2 },
      'Studio Quality': { sampleRate: 96000, bitDepth: 24, format: 'wav', channels: 2 },
      'MP3 High Quality': { sampleRate: 44100, format: 'mp3', mp3Bitrate: 320, quality: 90 },
      'MP3 Standard': { sampleRate: 44100, format: 'mp3', mp3Bitrate: 192, quality: 90 },
    },
  };

  onMount(async () => {
    await loadProjectData();
  });

  async function loadProjectData() {
    try {
      tracks = await api.project.getTracks();
      projectInfo = await api.project.getInfo();

      const defaultName = projectInfo.name || 'untitled';
      exportOptions.filename = `${defaultName}_${new Date().toISOString().split('T')[0]}`;

      exportOptions.range = {
        start: 0,
        end: projectInfo.length || 1920 * 16,
      };

      exportOptions.selectedTracks = tracks.map((t) => t.id);
    } catch (error) {
      console.error('Failed to load project data:', error);
    }
  }

  async function selectExportLocation() {
    const formatInfo = availableFormats.find((f) => f.id === exportFormat);
    const selected = await save({
      filters: [
        {
          name:
            exportFormat === 'midi'
              ? 'MIDI Files'
              : exportFormat === 'zip'
                ? 'ZIP Archives'
                : 'Audio Files',
          extensions: formatInfo?.extensions.map((e) => e.substring(1)) || ['*'],
        },
      ],
      defaultPath: exportOptions.filename,
    });

    if (selected) {
      exportOptions.location = selected;
    }
  }

  async function startExport() {
    if (!exportOptions.location && exportFormat !== 'zip') {
      await selectExportLocation();
      if (!exportOptions.location) {
        return;
      }
    }

    exportProgress = {
      status: 'preparing',
      current: 0,
      total: 100,
      message: 'Preparing export...',
      errors: [],
    };

    try {
      const exportParams = {
        format: exportFormat,
        options: exportOptions,
        tracks: exportOptions.includeAllTracks ? [] : exportOptions.selectedTracks,
      };

      const result = await api.export.exportProject(exportParams);

      if (result.jobId) {
        monitorExportProgress(result.jobId);
      }
    } catch (error) {
      exportProgress = {
        ...exportProgress,
        status: 'error',
        message: `Export failed: ${error}`,
        errors: [String(error)],
      };
    }
  }

  function monitorExportProgress(jobId: string) {
    progressInterval = setInterval(async () => {
      try {
        const progress = await api.export.getProgress(jobId);

        exportProgress = {
          ...exportProgress,
          status: progress.status,
          current: progress.current,
          total: progress.total,
          message: progress.message,
        };

        if (progress.status === 'completed' || progress.status === 'error') {
          if (progressInterval) {
            clearInterval(progressInterval);
            progressInterval = null;
          }

          if (progress.status === 'completed') {
            exportProgress.message = `Export completed: ${progress.outputPath}`;
          }
        }
      } catch (error) {
        if (progressInterval) {
          clearInterval(progressInterval);
          progressInterval = null;
        }
        exportProgress = {
          ...exportProgress,
          status: 'error',
          message: `Failed to monitor progress: ${error}`,
        };
      }
    }, 500);
  }

  function updateFormat(newFormat: string) {
    exportFormat = newFormat;

    const formatInfo = availableFormats.find((f) => f.id === newFormat);
    if (formatInfo && exportOptions.filename) {
      const ext = formatInfo.extensions[0];
      const baseName = exportOptions.filename.replace(/\.[^/.]+$/, '');
      exportOptions.filename = baseName + ext;
    }
  }

  function toggleTrackSelection(trackId: number) {
    if (exportOptions.selectedTracks.includes(trackId)) {
      exportOptions.selectedTracks = exportOptions.selectedTracks.filter((id) => id !== trackId);
    } else {
      exportOptions.selectedTracks = [...exportOptions.selectedTracks, trackId];
    }
  }

  function selectAllTracks() {
    exportOptions.selectedTracks = tracks.map((t) => t.id);
  }

  function deselectAllTracks() {
    exportOptions.selectedTracks = [];
  }

  function applyPreset(preset: Partial<ExportOptions>) {
    exportOptions = { ...exportOptions, ...preset };
  }

  function resetToDefaults() {
    exportOptions = {
      filename: '',
      location: '',
      includeAllTracks: true,
      selectedTracks: [],
      range: { start: 0, end: 0 },
      loop: false,
      normalize: false,
      dither: false,
      midiFormat: 1,
      includeMarkers: true,
      includeTempoChanges: true,
      includeTimeSignatures: true,
      includeKeySignatures: true,
      resolution: 480,
      sampleRate: 44100,
      bitDepth: 16,
      channels: 2,
      format: 'wav',
      quality: 90,
      mp3Bitrate: 192,
      exportMetadata: true,
      embedAlbumArt: false,
      includeProjectInfo: true,
      createZip: false,
    };
  }

  function estimateFileSize(): string {
    let size = 0;

    if (exportFormat === 'midi') {
      const totalEvents = tracks.reduce((sum, track) => sum + (track.eventCount || 0), 0);
      size = (totalEvents * 100) / (1024 * 1024);
    } else if (exportFormat === 'zip') {
      size = tracks.length * 0.5 + 1;
    } else {
      const duration = (exportOptions.range.end - exportOptions.range.start) / 480;
      const seconds = duration * (60 / (projectInfo.bpm || 120));
      const bytesPerSecond =
        (exportOptions.sampleRate * exportOptions.bitDepth * exportOptions.channels) / 8;
      size = (seconds * bytesPerSecond) / (1024 * 1024);

      if (exportFormat === 'mp3') {
        size *= (320 / exportOptions.mp3Bitrate) * (exportOptions.quality / 100);
      } else if (exportFormat === 'ogg') {
        size *= exportOptions.quality / 100;
      }
    }

    return size.toFixed(2);
  }

  function getFormatIcon(formatId: string): string {
    const icons: Record<string, string> = {
      midi: '🎹',
      wav: '🔊',
      mp3: '🎵',
      flac: '🎧',
      ogg: '🎶',
      zip: '📦',
    };
    return icons[formatId] || '📄';
  }

  // Reactive statements
  $: isExporting = exportProgress.status === 'exporting' || exportProgress.status === 'preparing';
  $: currentFormatPresets =
    exportFormat === 'midi'
      ? formatPresets.midi
      : ['wav', 'mp3', 'flac', 'ogg'].includes(exportFormat)
        ? formatPresets.audio
        : null;
</script>

<div class="export-window dark:bg-window dark:text-app-text h-full flex flex-col">
  <!-- Header -->
  <div class="header p-6 border-b dark:border-window-border">
    <h2 class="text-2xl dark:text-gray-200 mb-2">Export Project</h2>
    <p class="dark:text-gray-400">Export your project to various formats</p>
  </div>

  <div class="content flex-1 overflow-auto p-6">
    <div class="max-w-4xl mx-auto">
      <!-- Format Selection -->
      <div class="export-format mb-8">
        <h3 class="text-lg dark:text-gray-300 mb-4">Export Format</h3>
        <div class="grid grid-cols-3 md:grid-cols-6 gap-3">
          {#each availableFormats as format (format.id)}
            <button
              on:click={() => updateFormat(format.id)}
              class="aspect-square rounded-lg border-2 p-4 flex flex-col items-center justify-center transition-all"
              class:dark:border-primary={exportFormat === format.id}
              class:dark:border-window-border={exportFormat !== format.id}
              class:dark:bg-secondary={exportFormat === format.id}
            >
              <span class="text-2xl mb-2">{getFormatIcon(format.id)}</span>
              <span class="text-xs dark:text-gray-300 text-center">{format.name}</span>
            </button>
          {/each}
        </div>
      </div>

      <div class="grid grid-cols-3 gap-8">
        <!-- Left Column: Settings -->
        <div class="col-span-2 space-y-8">
          <!-- Basic Settings -->
          <div class="basic-settings">
            <h3 class="text-lg dark:text-gray-300 mb-4">Basic Settings</h3>
            <div class="space-y-4">
              <div>
                <label class="block text-sm dark:text-gray-400 mb-2">Filename</label>
                <div class="flex gap-2">
                  <input
                    type="text"
                    bind:value={exportOptions.filename}
                    class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  />
                  <button
                    on:click={selectExportLocation}
                    class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80 whitespace-nowrap"
                  >
                    Choose Location
                  </button>
                </div>
                <p class="text-xs dark:text-gray-500 mt-1">
                  {exportOptions.location || 'No location selected'}
                </p>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">Range Start</label>
                  <div class="flex gap-2">
                    <input
                      type="number"
                      bind:value={exportOptions.range.start}
                      min="0"
                      class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    />
                    <select class="px-2 py-2 dark:bg-input dark:border-window-border rounded">
                      <option>ticks</option>
                      <option>bars</option>
                      <option>seconds</option>
                    </select>
                  </div>
                </div>

                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">Range End</label>
                  <div class="flex gap-2">
                    <input
                      type="number"
                      bind:value={exportOptions.range.end}
                      min="0"
                      class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    />
                    <select class="px-2 py-2 dark:bg-input dark:border-window-border rounded">
                      <option>ticks</option>
                      <option>bars</option>
                      <option>seconds</option>
                    </select>
                  </div>
                </div>
              </div>

              <div class="space-y-2">
                <label class="flex items-center gap-2">
                  <input type="checkbox" bind:checked={exportOptions.loop} class="rounded" />
                  <span class="text-sm dark:text-gray-400">Export loop range only</span>
                </label>

                <label class="flex items-center gap-2">
                  <input type="checkbox" bind:checked={exportOptions.normalize} class="rounded" />
                  <span class="text-sm dark:text-gray-400">Normalize audio</span>
                </label>

                <label class="flex items-center gap-2">
                  <input type="checkbox" bind:checked={exportOptions.dither} class="rounded" />
                  <span class="text-sm dark:text-gray-400">Apply dithering</span>
                </label>
              </div>
            </div>
          </div>

          <!-- Track Selection -->
          <div class="track-selection">
            <div class="flex justify-between items-center mb-4">
              <h3 class="text-lg dark:text-gray-300">Track Selection</h3>
              <div class="flex gap-2">
                <button
                  on:click={selectAllTracks}
                  class="px-3 py-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                >
                  Select All
                </button>
                <button
                  on:click={deselectAllTracks}
                  class="px-3 py-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                >
                  Deselect All
                </button>
              </div>
            </div>

            <div class="space-y-2 max-h-64 overflow-auto">
              {#each tracks as track (track.id)}
                <label
                  class="flex items-center gap-3 p-3 dark:bg-window-subtle rounded border dark:border-window-border hover:dark:bg-menu cursor-pointer"
                >
                  <input
                    type="checkbox"
                    checked={exportOptions.includeAllTracks ||
                      exportOptions.selectedTracks.includes(track.id)}
                    disabled={exportOptions.includeAllTracks}
                    on:change={() => toggleTrackSelection(track.id)}
                    class="rounded"
                  />
                  <div class="flex-1">
                    <div class="flex justify-between items-center">
                      <span class="dark:text-gray-200">{track.name || `Track ${track.id}`}</span>
                      <span class="text-xs dark:text-gray-400"
                        >{track.channel !== undefined ? `Ch ${track.channel}` : 'MIDI'}</span
                      >
                    </div>
                    <div class="text-xs dark:text-gray-400 mt-1">
                      {track.eventCount || 0} events | {track.noteCount || 0} notes
                    </div>
                  </div>
                </label>
              {/each}
            </div>

            <label class="flex items-center gap-2 mt-3">
              <input
                type="checkbox"
                bind:checked={exportOptions.includeAllTracks}
                class="rounded"
              />
              <span class="text-sm dark:text-gray-400">Include all tracks</span>
            </label>
          </div>

          <!-- Format Specific Settings -->
          <div class="format-settings">
            <h3 class="text-lg dark:text-gray-300 mb-4">Format Settings</h3>

            {#if exportFormat === 'midi'}
              <div class="space-y-4">
                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">MIDI Format</label>
                  <div class="flex gap-3">
                    {#each [0, 1, 2] as format (format)}
                      <label class="flex items-center gap-2">
                        <input
                          type="radio"
                          bind:group={exportOptions.midiFormat}
                          value={format}
                          name="midiFormat"
                          class="rounded"
                        />
                        <span class="text-sm dark:text-gray-400">Type {format}</span>
                      </label>
                    {/each}
                  </div>
                  <p class="text-xs dark:text-gray-500 mt-1">
                    Type 0: Single track | Type 1: Multiple tracks | Type 2: Multiple sequences
                  </p>
                </div>

                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">Resolution (PPQN)</label>
                  <select
                    bind:value={exportOptions.resolution}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value={96}>96 (Low)</option>
                    <option value={192}>192</option>
                    <option value={240}>240</option>
                    <option value={480}>480 (Standard)</option>
                    <option value={960}>960 (High)</option>
                    <option value={1920}>1920 (Very High)</option>
                  </select>
                </div>

                <div class="space-y-2">
                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={exportOptions.includeMarkers}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Include markers</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={exportOptions.includeTempoChanges}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Include tempo changes</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={exportOptions.includeTimeSignatures}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Include time signatures</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={exportOptions.includeKeySignatures}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Include key signatures</span>
                  </label>
                </div>
              </div>
            {:else if ['wav', 'mp3', 'flac', 'ogg'].includes(exportFormat)}
              <div class="space-y-4">
                <div class="grid grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm dark:text-gray-400 mb-2">Sample Rate</label>
                    <select
                      bind:value={exportOptions.sampleRate}
                      class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    >
                      <option value={44100}>44.1 kHz (CD Quality)</option>
                      <option value={48000}>48 kHz (Professional)</option>
                      <option value={88200}>88.2 kHz (2x CD)</option>
                      <option value={96000}>96 kHz (Studio)</option>
                      <option value={176400}>176.4 kHz (4x CD)</option>
                      <option value={192000}>192 kHz (High-End)</option>
                    </select>
                  </div>

                  <div>
                    <label class="block text-sm dark:text-gray-400 mb-2">Bit Depth</label>
                    <select
                      bind:value={exportOptions.bitDepth}
                      class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    >
                      <option value={16}>16-bit (CD)</option>
                      <option value={24}>24-bit (Studio)</option>
                      <option value={32}>32-bit float (High Quality)</option>
                    </select>
                  </div>
                </div>

                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">Channels</label>
                  <div class="flex gap-3">
                    {#each [1, 2, 4, 6, 8] as channelCount (channelCount)}
                      <label class="flex items-center gap-2">
                        <input
                          type="radio"
                          bind:group={exportOptions.channels}
                          value={channelCount}
                          name="channels"
                          class="rounded"
                        />
                        <span class="text-sm dark:text-gray-400">
                          {channelCount === 1
                            ? 'Mono'
                            : channelCount === 2
                              ? 'Stereo'
                              : channelCount === 4
                                ? 'Quad'
                                : channelCount === 6
                                  ? '5.1'
                                  : '7.1'}
                        </span>
                      </label>
                    {/each}
                  </div>
                </div>

                {#if exportFormat === 'mp3'}
                  <div>
                    <label class="block text-sm dark:text-gray-400 mb-2">Bitrate (kbps)</label>
                    <select
                      bind:value={exportOptions.mp3Bitrate}
                      class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                    >
                      <option value={64}>64 (Low)</option>
                      <option value={96}>96</option>
                      <option value={128}>128 (Standard)</option>
                      <option value={192}>192 (Good)</option>
                      <option value={256}>256 (High)</option>
                      <option value={320}>320 (Maximum)</option>
                    </select>
                  </div>
                {/if}

                {#if ['mp3', 'ogg'].includes(exportFormat)}
                  <div>
                    <label class="block text-sm dark:text-gray-400 mb-2">Quality</label>
                    <input
                      type="range"
                      bind:value={exportOptions.quality}
                      min="0"
                      max="100"
                      step="1"
                      class="w-full"
                    />
                    <div class="text-xs dark:text-gray-500 text-center">
                      {exportOptions.quality}% {exportOptions.quality < 50
                        ? '(Smaller file)'
                        : exportOptions.quality < 80
                          ? '(Balanced)'
                          : '(Better quality)'}
                    </div>
                  </div>
                {/if}
              </div>
            {:else if exportFormat === 'zip'}
              <div class="space-y-4">
                <div class="space-y-2">
                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={exportOptions.exportMetadata}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Include project metadata</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={exportOptions.embedAlbumArt}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Embed album art</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input
                      type="checkbox"
                      bind:checked={exportOptions.includeProjectInfo}
                      class="rounded"
                    />
                    <span class="text-sm dark:text-gray-400">Include project file</span>
                  </label>

                  <label class="flex items-center gap-2">
                    <input type="checkbox" bind:checked={exportOptions.createZip} class="rounded" />
                    <span class="text-sm dark:text-gray-400">Create ZIP archive</span>
                  </label>
                </div>

                <div>
                  <label class="block text-sm dark:text-gray-400 mb-2">Compression Level</label>
                  <select
                    bind:value={exportOptions.compressionLevel}
                    class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
                  >
                    <option value={0}>No compression</option>
                    <option value={1}>Fastest</option>
                    <option value={5}>Fast</option>
                    <option value={9}>Normal</option>
                    <option value={12}>Maximum</option>
                  </select>
                </div>
              </div>
            {/if}
          </div>
        </div>

        <!-- Right Column: Preview & Actions -->
        <div class="space-y-8">
          <!-- Project Info -->
          <div
            class="project-info dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border"
          >
            <h3 class="text-lg dark:text-gray-300 mb-3">Project Information</h3>
            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Project:</span>
                <span class="dark:text-gray-200">{projectInfo.name || 'Untitled'}</span>
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Tracks:</span>
                <span class="dark:text-gray-200">{tracks.length}</span>
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Length:</span>
                <span class="dark:text-gray-200">
                  {Math.floor((exportOptions.range.end - exportOptions.range.start) / 480)} bars
                </span>
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Format:</span>
                <span class="dark:text-gray-200">
                  {availableFormats.find((f) => f.id === exportFormat)?.name}
                </span>
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Estimated Size:</span>
                <span class="dark:text-gray-200">~{estimateFileSize()} MB</span>
              </div>
            </div>
          </div>

          <!-- Presets -->
          {#if currentFormatPresets}
            <div class="presets">
              <h3 class="text-lg dark:text-gray-300 mb-3">Presets</h3>
              <div class="space-y-2">
                {#each Object.entries(currentFormatPresets) as [name, preset] (name)}
                  <button
                    on:click={() => applyPreset(preset)}
                    class="w-full text-left px-3 py-2 dark:bg-secondary rounded hover:opacity-80 text-sm"
                  >
                    {name}
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Export Progress -->
          {#if exportProgress.status !== 'idle'}
            <div
              class="export-progress dark:bg-window-subtle p-4 rounded-lg border dark:border-window-border"
            >
              <h3 class="text-lg dark:text-gray-300 mb-3">
                {exportProgress.status === 'preparing'
                  ? 'Preparing...'
                  : exportProgress.status === 'exporting'
                    ? 'Exporting...'
                    : exportProgress.status === 'completed'
                      ? 'Complete!'
                      : 'Error'}
              </h3>

              <ProgressIndicator
                current={exportProgress.current}
                total={exportProgress.total}
                message={exportProgress.message}
                showPercentage={true}
              />

              {#if exportProgress.errors.length > 0}
                <div class="mt-3">
                  <h4 class="text-sm dark:text-gray-400 mb-2">Errors:</h4>
                  <div class="text-xs dark:text-error space-y-1">
                    {#each exportProgress.errors as error, index (index)}
                      <div>{error}</div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {/if}

          <!-- Actions -->
          <div class="actions space-y-3">
            <button
              on:click={startExport}
              disabled={isExporting}
              class="w-full px-6 py-3 dark:bg-primary dark:text-white rounded-lg hover:opacity-80 disabled:opacity-50 flex items-center justify-center gap-2"
            >
              {#if isExporting}
                <div class="animate-spin rounded-full h-4 w-4 border-b-2"></div>
                {exportProgress.status === 'exporting' ? 'Exporting...' : 'Preparing...'}
              {:else}
                Start Export
              {/if}
            </button>

            <button
              on:click={() => {
                /* Save preset */
              }}
              class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
            >
              Save as Preset
            </button>

            <button
              on:click={resetToDefaults}
              class="w-full px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
            >
              Reset to Defaults
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .export-window input[type='range'] {
    height: 6px;
  }

  .export-window input[type='range']::-webkit-slider-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    cursor: pointer;
  }

  .export-window input[type='range']::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    cursor: pointer;
    border: 0;
  }
</style>
