<!--
  PresetsManagerWindow.svelte

  Manages preset templates:
  - Mixer presets (channel configurations)
  - Track templates (MIDI routing, defaults)
  - Project templates (complete project setups)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import {
    presetsStore,
    presetsActions,
    filteredMixerPresets,
    filteredTrackTemplates,
    filteredProjectTemplates,
    mixerPresetCount,
    trackTemplateCount,
    projectTemplateCount,
  } from '$lib/stores';
  import type {
    MixerPreset,
    MixerPresetChannel,
    MixerPresetMaster,
    TrackTemplate,
    TrackType,
    TrackMidiRouting,
    ProjectTemplate,
    ProjectTemplateTrack,
    WindowId,
  } from '$lib/types';

  export let windowId: WindowId;

  // Local state
  let activeTab: 'mixer' | 'track' | 'project' = 'mixer';
  let searchQuery = '';
  let filterCategory: string | null = null;

  // Modal states
  let showMixerPresetModal = false;
  let showTrackTemplateModal = false;
  let showProjectTemplateModal = false;
  let showDuplicateProjectModal = false;
  let editingMixerPreset: MixerPreset | null = null;
  let editingTrackTemplate: TrackTemplate | null = null;
  let editingProjectTemplate: ProjectTemplate | null = null;
  let duplicatingProjectTemplate: ProjectTemplate | null = null;

  // Form data for mixer preset
  let mixerPresetForm = {
    name: '',
    description: '',
    category: 'general',
    is_factory: false,
    channels: [] as MixerPresetChannel[],
    master: { volume: 1.0, pan: 0.5, limiter_enabled: false, limiter_threshold: -0.1 } as MixerPresetMaster,
  };

  // Form data for track template
  let trackTemplateForm = {
    name: '',
    description: '',
    category: 'general',
    track_type: 'midi' as TrackType,
    is_factory: false,
    color: '#3b82f6',
    default_height: 100,
    midi_channel: 1,
    instrument_name: '',
    volume: 0.8,
    pan: 0.5,
    routing: {
      input_channel: 0,
      output_channel: 1,
      thru_enabled: false,
    } as TrackMidiRouting,
  };

  // Form data for project template
  let projectTemplateForm = {
    name: '',
    description: '',
    category: 'general',
    is_factory: false,
    bpm: 120,
    time_signature_numerator: 4,
    time_signature_denominator: 4,
    key_signature: 'C',
    tracks: [] as ProjectTemplateTrack[],
    mixer_preset_id: undefined as number | undefined,
  };

  // Duplicate project form
  let duplicateProjectName = '';

  // Reactive store data
  $: mixerPresets = $filteredMixerPresets;
  $: trackTemplates = $filteredTrackTemplates;
  $: projectTemplates = $filteredProjectTemplates;
  $: selectedMixerPreset = $presetsStore.selectedMixerPreset;
  $: selectedTrackTemplate = $presetsStore.selectedTrackTemplate;
  $: selectedProjectTemplate = $presetsStore.selectedProjectTemplate;
  $: isLoading = $presetsStore.isLoading;
  $: isSaving = $presetsStore.isSaving;
  $: error = $presetsStore.error;
  $: totalMixerPresets = $mixerPresetCount;
  $: totalTrackTemplates = $trackTemplateCount;
  $: totalProjectTemplates = $projectTemplateCount;

  // Categories
  const mixerCategories = ['general', 'drums', 'bass', 'keys', 'strings', 'vocals', 'full-mix'];
  const trackCategories = ['general', 'drums', 'bass', 'keys', 'synth', 'strings', 'brass', 'vocals', 'fx'];
  const projectCategories = ['general', 'electronic', 'acoustic', 'orchestral', 'live', 'recording', 'mastering'];
  const trackTypes: TrackType[] = ['midi', 'audio', 'instrument', 'bus', 'aux', 'master'];
  const keySignatures = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B', 'Cm', 'C#m', 'Dm', 'D#m', 'Em', 'Fm', 'F#m', 'Gm', 'G#m', 'Am', 'A#m', 'Bm'];

  // Update store search
  $: presetsActions.setSearchQuery(searchQuery);
  $: presetsActions.setFilterCategory(filterCategory);

  onMount(() => {
    void presetsActions.initialize();
  });

  // Tab change
  function switchTab(tab: 'mixer' | 'track' | 'project') {
    activeTab = tab;
    presetsActions.setActiveTab(tab);
  }

  // ============================================================================
  // MIXER PRESET HANDLERS
  // ============================================================================

  function openAddMixerPresetModal() {
    editingMixerPreset = null;
    mixerPresetForm = {
      name: '',
      description: '',
      category: 'general',
      is_factory: false,
      channels: [],
      master: { volume: 1.0, pan: 0.5, limiter_enabled: false, limiter_threshold: -0.1 },
    };
    showMixerPresetModal = true;
  }

  function openEditMixerPresetModal(preset: MixerPreset) {
    editingMixerPreset = preset;
    mixerPresetForm = {
      name: preset.name,
      description: preset.description,
      category: preset.category,
      is_factory: preset.is_factory,
      channels: [...preset.channels],
      master: { ...preset.master },
    };
    showMixerPresetModal = true;
  }

  async function saveMixerPreset() {
    if (editingMixerPreset) {
      await presetsActions.updateMixerPreset(editingMixerPreset.id, mixerPresetForm);
    } else {
      await presetsActions.createMixerPreset(mixerPresetForm);
    }
    showMixerPresetModal = false;
  }

  async function deleteMixerPreset(preset: MixerPreset) {
    if (confirm(`Delete mixer preset "${preset.name}"?`)) {
      await presetsActions.deleteMixerPreset(preset.id);
    }
  }

  function addMixerChannel() {
    const newChannel: MixerPresetChannel = {
      channel_id: mixerPresetForm.channels.length + 1,
      label: `Channel ${mixerPresetForm.channels.length + 1}`,
      volume: 0.8,
      pan: 0.5,
      muted: false,
      soloed: false,
      midi_channel: mixerPresetForm.channels.length + 1,
      cc_values: {},
    };
    mixerPresetForm.channels = [...mixerPresetForm.channels, newChannel];
  }

  function removeMixerChannel(index: number) {
    mixerPresetForm.channels = mixerPresetForm.channels.filter((_, i) => i !== index);
  }

  // ============================================================================
  // TRACK TEMPLATE HANDLERS
  // ============================================================================

  function openAddTrackTemplateModal() {
    editingTrackTemplate = null;
    trackTemplateForm = {
      name: '',
      description: '',
      category: 'general',
      track_type: 'midi',
      is_factory: false,
      color: '#3b82f6',
      default_height: 100,
      midi_channel: 1,
      instrument_name: '',
      volume: 0.8,
      pan: 0.5,
      routing: {
        input_channel: 0,
        output_channel: 1,
        thru_enabled: false,
      },
    };
    showTrackTemplateModal = true;
  }

  function openEditTrackTemplateModal(template: TrackTemplate) {
    editingTrackTemplate = template;
    trackTemplateForm = {
      name: template.name,
      description: template.description,
      category: template.category,
      track_type: template.track_type,
      is_factory: template.is_factory,
      color: template.color,
      default_height: template.default_height,
      midi_channel: template.midi_channel,
      instrument_name: template.instrument_name || '',
      volume: template.volume,
      pan: template.pan,
      routing: { ...template.routing },
    };
    showTrackTemplateModal = true;
  }

  async function saveTrackTemplate() {
    if (editingTrackTemplate) {
      await presetsActions.updateTrackTemplate(editingTrackTemplate.id, trackTemplateForm);
    } else {
      await presetsActions.createTrackTemplate(trackTemplateForm);
    }
    showTrackTemplateModal = false;
  }

  async function deleteTrackTemplate(template: TrackTemplate) {
    if (confirm(`Delete track template "${template.name}"?`)) {
      await presetsActions.deleteTrackTemplate(template.id);
    }
  }

  // ============================================================================
  // PROJECT TEMPLATE HANDLERS
  // ============================================================================

  function openAddProjectTemplateModal() {
    editingProjectTemplate = null;
    projectTemplateForm = {
      name: '',
      description: '',
      category: 'general',
      is_factory: false,
      bpm: 120,
      time_signature_numerator: 4,
      time_signature_denominator: 4,
      key_signature: 'C',
      tracks: [],
      mixer_preset_id: undefined,
    };
    showProjectTemplateModal = true;
  }

  function openEditProjectTemplateModal(template: ProjectTemplate) {
    editingProjectTemplate = template;
    projectTemplateForm = {
      name: template.name,
      description: template.description,
      category: template.category,
      is_factory: template.is_factory,
      bpm: template.bpm,
      time_signature_numerator: template.time_signature_numerator,
      time_signature_denominator: template.time_signature_denominator,
      key_signature: template.key_signature,
      tracks: [...template.tracks],
      mixer_preset_id: template.mixer_preset_id,
    };
    showProjectTemplateModal = true;
  }

  async function saveProjectTemplate() {
    if (editingProjectTemplate) {
      await presetsActions.updateProjectTemplate(editingProjectTemplate.id, projectTemplateForm);
    } else {
      await presetsActions.createProjectTemplate(projectTemplateForm);
    }
    showProjectTemplateModal = false;
  }

  async function deleteProjectTemplate(template: ProjectTemplate) {
    if (confirm(`Delete project template "${template.name}"?`)) {
      await presetsActions.deleteProjectTemplate(template.id);
    }
  }

  function openDuplicateProjectModal(template: ProjectTemplate) {
    duplicatingProjectTemplate = template;
    duplicateProjectName = `${template.name} (Copy)`;
    showDuplicateProjectModal = true;
  }

  async function duplicateProjectTemplate() {
    if (duplicatingProjectTemplate && duplicateProjectName) {
      await presetsActions.duplicateProjectTemplate(duplicatingProjectTemplate.id, duplicateProjectName);
      showDuplicateProjectModal = false;
    }
  }

  function addProjectTrack() {
    const newTrack: ProjectTemplateTrack = {
      name: `Track ${projectTemplateForm.tracks.length + 1}`,
      track_type: 'midi',
      color: '#3b82f6',
      midi_channel: projectTemplateForm.tracks.length + 1,
      volume: 0.8,
      pan: 0.5,
    };
    projectTemplateForm.tracks = [...projectTemplateForm.tracks, newTrack];
  }

  function removeProjectTrack(index: number) {
    projectTemplateForm.tracks = projectTemplateForm.tracks.filter((_, i) => i !== index);
  }

  // Helpers
  function formatTrackType(type: TrackType): string {
    return type.charAt(0).toUpperCase() + type.slice(1);
  }

  function getTrackTypeColor(type: TrackType): string {
    const colors: Record<TrackType, string> = {
      midi: 'text-blue-400',
      audio: 'text-green-400',
      instrument: 'text-purple-400',
      bus: 'text-yellow-400',
      aux: 'text-orange-400',
      master: 'text-red-400',
    };
    return colors[type] || 'text-gray-400';
  }
</script>

<WindowBase {windowId} title="Presets Manager" width={950} height={700}>
  <div class="flex flex-col h-full bg-gray-900 text-gray-200">
    <!-- Header with tabs and search -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-700">
      <div class="flex space-x-2">
        <button
          class="px-4 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'mixer'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}"
          on:click={() => switchTab('mixer')}
        >
          Mixer Presets ({totalMixerPresets})
        </button>
        <button
          class="px-4 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'track'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}"
          on:click={() => switchTab('track')}
        >
          Track Templates ({totalTrackTemplates})
        </button>
        <button
          class="px-4 py-2 rounded-md text-sm font-medium transition-colors {activeTab === 'project'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}"
          on:click={() => switchTab('project')}
        >
          Project Templates ({totalProjectTemplates})
        </button>
      </div>

      <div class="flex items-center space-x-3">
        <input
          type="text"
          placeholder="Search presets..."
          bind:value={searchQuery}
          class="px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 w-48"
        />
        <select
          bind:value={filterCategory}
          class="px-3 py-2 bg-gray-800 border border-gray-600 rounded-md text-sm text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value={null}>All Categories</option>
          {#if activeTab === 'mixer'}
            {#each mixerCategories as cat (cat)}
              <option value={cat}>{cat}</option>
            {/each}
          {:else if activeTab === 'track'}
            {#each trackCategories as cat (cat)}
              <option value={cat}>{cat}</option>
            {/each}
          {:else}
            {#each projectCategories as cat (cat)}
              <option value={cat}>{cat}</option>
            {/each}
          {/if}
        </select>
      </div>
    </div>

    <!-- Error display -->
    {#if error}
      <div class="px-4 py-2 bg-red-900/50 border-b border-red-700 text-red-200 text-sm">
        {error}
        <button class="ml-2 text-red-400 hover:text-red-300" on:click={() => presetsActions.clearError()}>Dismiss</button>
      </div>
    {/if}

    <!-- Content area -->
    <div class="flex-1 flex overflow-hidden">
      <!-- ================================================================== -->
      <!-- MIXER PRESETS TAB -->
      <!-- ================================================================== -->
      {#if activeTab === 'mixer'}
        <div class="w-72 border-r border-gray-700 flex flex-col">
          <div class="p-3 border-b border-gray-700">
            <button
              class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md text-sm font-medium"
              on:click={openAddMixerPresetModal}
            >
              + New Mixer Preset
            </button>
          </div>
          <div class="flex-1 overflow-y-auto">
            {#if isLoading}
              <div class="p-4 text-gray-400 text-center">Loading...</div>
            {:else if mixerPresets.length === 0}
              <div class="p-4 text-gray-500 text-center text-sm">No mixer presets found</div>
            {:else}
              {#each mixerPresets as preset (preset.id)}
                <button
                  class="w-full px-4 py-3 text-left border-b border-gray-700 hover:bg-gray-800 transition-colors {selectedMixerPreset?.id === preset.id ? 'bg-gray-800 border-l-2 border-l-blue-500' : ''}"
                  on:click={() => presetsActions.selectMixerPreset(preset)}
                >
                  <div class="font-medium text-gray-200">{preset.name}</div>
                  <div class="text-xs text-gray-400 mt-1">
                    {preset.category} · {preset.channels.length} channels
                    {#if preset.is_factory}
                      <span class="ml-1 text-yellow-500">(Factory)</span>
                    {/if}
                  </div>
                </button>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Mixer Preset Detail -->
        <div class="flex-1 p-4 overflow-y-auto">
          {#if selectedMixerPreset}
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <h2 class="text-xl font-semibold">{selectedMixerPreset.name}</h2>
                <div class="flex space-x-2">
                  <button
                    class="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded text-sm"
                    on:click={() => openEditMixerPresetModal(selectedMixerPreset)}
                  >
                    Edit
                  </button>
                  {#if !selectedMixerPreset.is_factory}
                    <button
                      class="px-3 py-1.5 bg-red-700 hover:bg-red-600 text-white rounded text-sm"
                      on:click={() => deleteMixerPreset(selectedMixerPreset)}
                    >
                      Delete
                    </button>
                  {/if}
                </div>
              </div>

              {#if selectedMixerPreset.description}
                <p class="text-gray-400">{selectedMixerPreset.description}</p>
              {/if}

              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span class="text-gray-500">Category:</span>
                  <span class="ml-2 text-gray-200">{selectedMixerPreset.category}</span>
                </div>
                <div>
                  <span class="text-gray-500">Channels:</span>
                  <span class="ml-2 text-gray-200">{selectedMixerPreset.channels.length}</span>
                </div>
              </div>

              <div class="border-t border-gray-700 pt-4">
                <h3 class="text-lg font-medium mb-3">Master Settings</h3>
                <div class="grid grid-cols-4 gap-4 text-sm bg-gray-800 p-3 rounded">
                  <div>
                    <span class="text-gray-500">Volume:</span>
                    <span class="ml-2">{(selectedMixerPreset.master.volume * 100).toFixed(0)}%</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Pan:</span>
                    <span class="ml-2">{((selectedMixerPreset.master.pan - 0.5) * 200).toFixed(0)}</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Limiter:</span>
                    <span class="ml-2">{selectedMixerPreset.master.limiter_enabled ? 'On' : 'Off'}</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Threshold:</span>
                    <span class="ml-2">{selectedMixerPreset.master.limiter_threshold} dB</span>
                  </div>
                </div>
              </div>

              {#if selectedMixerPreset.channels.length > 0}
                <div class="border-t border-gray-700 pt-4">
                  <h3 class="text-lg font-medium mb-3">Channels</h3>
                  <div class="space-y-2">
                    {#each selectedMixerPreset.channels as channel (channel.channel_id)}
                      <div class="bg-gray-800 p-3 rounded text-sm">
                        <div class="font-medium">{channel.label}</div>
                        <div class="grid grid-cols-4 gap-2 mt-2 text-gray-400">
                          <span>Vol: {(channel.volume * 100).toFixed(0)}%</span>
                          <span>Pan: {((channel.pan - 0.5) * 200).toFixed(0)}</span>
                          <span>MIDI Ch: {channel.midi_channel}</span>
                          <span>{channel.muted ? 'Muted' : ''} {channel.soloed ? 'Solo' : ''}</span>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          {:else}
            <div class="flex items-center justify-center h-full text-gray-500">
              Select a mixer preset to view details
            </div>
          {/if}
        </div>

      <!-- ================================================================== -->
      <!-- TRACK TEMPLATES TAB -->
      <!-- ================================================================== -->
      {:else if activeTab === 'track'}
        <div class="w-72 border-r border-gray-700 flex flex-col">
          <div class="p-3 border-b border-gray-700">
            <button
              class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md text-sm font-medium"
              on:click={openAddTrackTemplateModal}
            >
              + New Track Template
            </button>
          </div>
          <div class="flex-1 overflow-y-auto">
            {#if isLoading}
              <div class="p-4 text-gray-400 text-center">Loading...</div>
            {:else if trackTemplates.length === 0}
              <div class="p-4 text-gray-500 text-center text-sm">No track templates found</div>
            {:else}
              {#each trackTemplates as template (template.id)}
                <button
                  class="w-full px-4 py-3 text-left border-b border-gray-700 hover:bg-gray-800 transition-colors {selectedTrackTemplate?.id === template.id ? 'bg-gray-800 border-l-2 border-l-blue-500' : ''}"
                  on:click={() => presetsActions.selectTrackTemplate(template)}
                >
                  <div class="flex items-center">
                    <div class="w-3 h-3 rounded mr-2" style="background-color: {template.color}"></div>
                    <span class="font-medium text-gray-200">{template.name}</span>
                  </div>
                  <div class="text-xs text-gray-400 mt-1">
                    <span class={getTrackTypeColor(template.track_type)}>{formatTrackType(template.track_type)}</span>
                    · {template.category}
                    {#if template.is_factory}
                      <span class="ml-1 text-yellow-500">(Factory)</span>
                    {/if}
                  </div>
                </button>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Track Template Detail -->
        <div class="flex-1 p-4 overflow-y-auto">
          {#if selectedTrackTemplate}
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div class="w-4 h-4 rounded mr-3" style="background-color: {selectedTrackTemplate.color}"></div>
                  <h2 class="text-xl font-semibold">{selectedTrackTemplate.name}</h2>
                </div>
                <div class="flex space-x-2">
                  <button
                    class="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded text-sm"
                    on:click={() => openEditTrackTemplateModal(selectedTrackTemplate)}
                  >
                    Edit
                  </button>
                  {#if !selectedTrackTemplate.is_factory}
                    <button
                      class="px-3 py-1.5 bg-red-700 hover:bg-red-600 text-white rounded text-sm"
                      on:click={() => deleteTrackTemplate(selectedTrackTemplate)}
                    >
                      Delete
                    </button>
                  {/if}
                </div>
              </div>

              {#if selectedTrackTemplate.description}
                <p class="text-gray-400">{selectedTrackTemplate.description}</p>
              {/if}

              <div class="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <span class="text-gray-500">Type:</span>
                  <span class="ml-2 {getTrackTypeColor(selectedTrackTemplate.track_type)}">{formatTrackType(selectedTrackTemplate.track_type)}</span>
                </div>
                <div>
                  <span class="text-gray-500">Category:</span>
                  <span class="ml-2 text-gray-200">{selectedTrackTemplate.category}</span>
                </div>
                <div>
                  <span class="text-gray-500">MIDI Channel:</span>
                  <span class="ml-2 text-gray-200">{selectedTrackTemplate.midi_channel}</span>
                </div>
              </div>

              <div class="grid grid-cols-3 gap-4 text-sm">
                <div>
                  <span class="text-gray-500">Volume:</span>
                  <span class="ml-2 text-gray-200">{(selectedTrackTemplate.volume * 100).toFixed(0)}%</span>
                </div>
                <div>
                  <span class="text-gray-500">Pan:</span>
                  <span class="ml-2 text-gray-200">{((selectedTrackTemplate.pan - 0.5) * 200).toFixed(0)}</span>
                </div>
                <div>
                  <span class="text-gray-500">Height:</span>
                  <span class="ml-2 text-gray-200">{selectedTrackTemplate.default_height}px</span>
                </div>
              </div>

              {#if selectedTrackTemplate.instrument_name}
                <div class="text-sm">
                  <span class="text-gray-500">Instrument:</span>
                  <span class="ml-2 text-gray-200">{selectedTrackTemplate.instrument_name}</span>
                </div>
              {/if}

              <div class="border-t border-gray-700 pt-4">
                <h3 class="text-lg font-medium mb-3">MIDI Routing</h3>
                <div class="bg-gray-800 p-3 rounded text-sm space-y-2">
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <span class="text-gray-500">Input Device:</span>
                      <span class="ml-2">{selectedTrackTemplate.routing.input_device || 'Default'}</span>
                    </div>
                    <div>
                      <span class="text-gray-500">Input Channel:</span>
                      <span class="ml-2">{selectedTrackTemplate.routing.input_channel}</span>
                    </div>
                  </div>
                  <div class="grid grid-cols-2 gap-4">
                    <div>
                      <span class="text-gray-500">Output Device:</span>
                      <span class="ml-2">{selectedTrackTemplate.routing.output_device || 'Default'}</span>
                    </div>
                    <div>
                      <span class="text-gray-500">Output Channel:</span>
                      <span class="ml-2">{selectedTrackTemplate.routing.output_channel}</span>
                    </div>
                  </div>
                  <div>
                    <span class="text-gray-500">MIDI Thru:</span>
                    <span class="ml-2">{selectedTrackTemplate.routing.thru_enabled ? 'Enabled' : 'Disabled'}</span>
                  </div>
                </div>
              </div>
            </div>
          {:else}
            <div class="flex items-center justify-center h-full text-gray-500">
              Select a track template to view details
            </div>
          {/if}
        </div>

      <!-- ================================================================== -->
      <!-- PROJECT TEMPLATES TAB -->
      <!-- ================================================================== -->
      {:else}
        <div class="w-72 border-r border-gray-700 flex flex-col">
          <div class="p-3 border-b border-gray-700">
            <button
              class="w-full px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-md text-sm font-medium"
              on:click={openAddProjectTemplateModal}
            >
              + New Project Template
            </button>
          </div>
          <div class="flex-1 overflow-y-auto">
            {#if isLoading}
              <div class="p-4 text-gray-400 text-center">Loading...</div>
            {:else if projectTemplates.length === 0}
              <div class="p-4 text-gray-500 text-center text-sm">No project templates found</div>
            {:else}
              {#each projectTemplates as template (template.id)}
                <button
                  class="w-full px-4 py-3 text-left border-b border-gray-700 hover:bg-gray-800 transition-colors {selectedProjectTemplate?.id === template.id ? 'bg-gray-800 border-l-2 border-l-blue-500' : ''}"
                  on:click={() => presetsActions.selectProjectTemplate(template)}
                >
                  <div class="font-medium text-gray-200">{template.name}</div>
                  <div class="text-xs text-gray-400 mt-1">
                    {template.bpm} BPM · {template.time_signature_numerator}/{template.time_signature_denominator} · {template.key_signature}
                    {#if template.is_factory}
                      <span class="ml-1 text-yellow-500">(Factory)</span>
                    {/if}
                  </div>
                  <div class="text-xs text-gray-500 mt-0.5">
                    {template.tracks.length} tracks · {template.category}
                  </div>
                </button>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Project Template Detail -->
        <div class="flex-1 p-4 overflow-y-auto">
          {#if selectedProjectTemplate}
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <h2 class="text-xl font-semibold">{selectedProjectTemplate.name}</h2>
                <div class="flex space-x-2">
                  <button
                    class="px-3 py-1.5 bg-green-700 hover:bg-green-600 text-white rounded text-sm"
                    on:click={() => openDuplicateProjectModal(selectedProjectTemplate)}
                  >
                    Duplicate
                  </button>
                  <button
                    class="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded text-sm"
                    on:click={() => openEditProjectTemplateModal(selectedProjectTemplate)}
                  >
                    Edit
                  </button>
                  {#if !selectedProjectTemplate.is_factory}
                    <button
                      class="px-3 py-1.5 bg-red-700 hover:bg-red-600 text-white rounded text-sm"
                      on:click={() => deleteProjectTemplate(selectedProjectTemplate)}
                    >
                      Delete
                    </button>
                  {/if}
                </div>
              </div>

              {#if selectedProjectTemplate.description}
                <p class="text-gray-400">{selectedProjectTemplate.description}</p>
              {/if}

              <div class="grid grid-cols-4 gap-4 text-sm">
                <div>
                  <span class="text-gray-500">BPM:</span>
                  <span class="ml-2 text-gray-200">{selectedProjectTemplate.bpm}</span>
                </div>
                <div>
                  <span class="text-gray-500">Time Sig:</span>
                  <span class="ml-2 text-gray-200">{selectedProjectTemplate.time_signature_numerator}/{selectedProjectTemplate.time_signature_denominator}</span>
                </div>
                <div>
                  <span class="text-gray-500">Key:</span>
                  <span class="ml-2 text-gray-200">{selectedProjectTemplate.key_signature}</span>
                </div>
                <div>
                  <span class="text-gray-500">Category:</span>
                  <span class="ml-2 text-gray-200">{selectedProjectTemplate.category}</span>
                </div>
              </div>

              {#if selectedProjectTemplate.tracks.length > 0}
                <div class="border-t border-gray-700 pt-4">
                  <h3 class="text-lg font-medium mb-3">Tracks ({selectedProjectTemplate.tracks.length})</h3>
                  <div class="space-y-2">
                    {#each selectedProjectTemplate.tracks as track, i (i)}
                      <div class="bg-gray-800 p-3 rounded text-sm flex items-center">
                        <div class="w-3 h-3 rounded mr-3" style="background-color: {track.color}"></div>
                        <div class="flex-1">
                          <div class="font-medium">{track.name}</div>
                          <div class="text-gray-400 text-xs">
                            <span class={getTrackTypeColor(track.track_type)}>{formatTrackType(track.track_type)}</span>
                            · MIDI Ch {track.midi_channel}
                            · Vol {(track.volume * 100).toFixed(0)}%
                            {#if track.instrument_name}
                              · {track.instrument_name}
                            {/if}
                          </div>
                        </div>
                      </div>
                    {/each}
                  </div>
                </div>
              {:else}
                <div class="text-gray-500 text-sm">No tracks in this template</div>
              {/if}
            </div>
          {:else}
            <div class="flex items-center justify-center h-full text-gray-500">
              Select a project template to view details
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</WindowBase>

<!-- ============================================================================ -->
<!-- MODALS -->
<!-- ============================================================================ -->

<!-- Mixer Preset Modal -->
{#if showMixerPresetModal}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" role="button" tabindex="0" on:click|self={() => showMixerPresetModal = false} on:keydown={(e) => { if (e.key === 'Escape') { showMixerPresetModal = false; } }}>
    <div class="bg-gray-800 rounded-lg shadow-xl w-[600px] max-h-[80vh] overflow-y-auto">
      <div class="px-6 py-4 border-b border-gray-700">
        <h3 class="text-lg font-semibold text-gray-200">
          {editingMixerPreset ? 'Edit Mixer Preset' : 'New Mixer Preset'}
        </h3>
      </div>
      <div class="p-6 space-y-4">
        <div>
          <label for="mixer-name" class="block text-sm font-medium text-gray-300 mb-1">Name</label>
          <input id="mixer-name" type="text" bind:value={mixerPresetForm.name} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:ring-2 focus:ring-blue-500" />
        </div>
        <div>
          <label for="mixer-description" class="block text-sm font-medium text-gray-300 mb-1">Description</label>
          <textarea id="mixer-description" bind:value={mixerPresetForm.description} rows="2" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:ring-2 focus:ring-blue-500"></textarea>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="mixer-category" class="block text-sm font-medium text-gray-300 mb-1">Category</label>
            <select id="mixer-category" bind:value={mixerPresetForm.category} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200">
              {#each mixerCategories as cat (cat)}
                <option value={cat}>{cat}</option>
              {/each}
            </select>
          </div>
          <div class="flex items-center pt-6">
            <input type="checkbox" bind:checked={mixerPresetForm.is_factory} id="mixer-factory" class="mr-2" />
            <label for="mixer-factory" class="text-sm text-gray-300">Factory Preset</label>
          </div>
        </div>

        <div class="border-t border-gray-700 pt-4">
          <div class="flex justify-between items-center mb-3">
            <h4 class="font-medium text-gray-200">Channels</h4>
            <button class="px-3 py-1 bg-blue-600 hover:bg-blue-500 text-white rounded text-sm" on:click={addMixerChannel}>
              + Add Channel
            </button>
          </div>
          {#each mixerPresetForm.channels as channel, i (i)}
            <div class="bg-gray-700 p-3 rounded mb-2">
              <div class="flex justify-between items-center mb-2">
                <input type="text" bind:value={channel.label} class="px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200 text-sm w-40" />
                <button class="text-red-400 hover:text-red-300 text-sm" on:click={() => removeMixerChannel(i)}>Remove</button>
              </div>
              <div class="grid grid-cols-4 gap-2 text-sm">
                <label class="block">
                  <span class="text-xs text-gray-400">Volume</span>
                  <input type="number" bind:value={channel.volume} step="0.1" min="0" max="1" class="w-full px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200" />
                </label>
                <label class="block">
                  <span class="text-xs text-gray-400">Pan</span>
                  <input type="number" bind:value={channel.pan} step="0.1" min="0" max="1" class="w-full px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200" />
                </label>
                <label class="block">
                  <span class="text-xs text-gray-400">MIDI Ch</span>
                  <input type="number" bind:value={channel.midi_channel} min="1" max="16" class="w-full px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200" />
                </label>
                <div class="flex items-end space-x-2">
                  <label class="flex items-center text-xs text-gray-400">
                    <input type="checkbox" bind:checked={channel.muted} class="mr-1" /> M
                  </label>
                  <label class="flex items-center text-xs text-gray-400">
                    <input type="checkbox" bind:checked={channel.soloed} class="mr-1" /> S
                  </label>
                </div>
              </div>
            </div>
          {/each}
        </div>

        <div class="border-t border-gray-700 pt-4">
          <h4 class="font-medium text-gray-200 mb-3">Master Settings</h4>
          <div class="grid grid-cols-4 gap-4">
            <label class="block">
              <span class="text-xs text-gray-400">Volume</span>
              <input type="number" bind:value={mixerPresetForm.master.volume} step="0.1" min="0" max="1" class="w-full px-2 py-1 bg-gray-700 border border-gray-600 rounded text-gray-200" />
            </label>
            <label class="block">
              <span class="text-xs text-gray-400">Pan</span>
              <input type="number" bind:value={mixerPresetForm.master.pan} step="0.1" min="0" max="1" class="w-full px-2 py-1 bg-gray-700 border border-gray-600 rounded text-gray-200" />
            </label>
            <div class="flex items-end">
              <label class="flex items-center text-sm text-gray-300">
                <input type="checkbox" bind:checked={mixerPresetForm.master.limiter_enabled} class="mr-2" /> Limiter
              </label>
            </div>
            <label class="block">
              <span class="text-xs text-gray-400">Threshold</span>
              <input type="number" bind:value={mixerPresetForm.master.limiter_threshold} step="0.1" class="w-full px-2 py-1 bg-gray-700 border border-gray-600 rounded text-gray-200" />
            </label>
          </div>
        </div>
      </div>
      <div class="px-6 py-4 border-t border-gray-700 flex justify-end space-x-3">
        <button class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded" on:click={() => showMixerPresetModal = false}>
          Cancel
        </button>
        <button class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded" on:click={saveMixerPreset} disabled={isSaving}>
          {isSaving ? 'Saving...' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Track Template Modal -->
{#if showTrackTemplateModal}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" role="button" tabindex="0" on:click|self={() => showTrackTemplateModal = false} on:keydown={(e) => { if (e.key === 'Escape') { showTrackTemplateModal = false; } }}>
    <div class="bg-gray-800 rounded-lg shadow-xl w-[550px] max-h-[80vh] overflow-y-auto">
      <div class="px-6 py-4 border-b border-gray-700">
        <h3 class="text-lg font-semibold text-gray-200">
          {editingTrackTemplate ? 'Edit Track Template' : 'New Track Template'}
        </h3>
      </div>
      <div class="p-6 space-y-4">
        <div>
          <label for="track-name" class="block text-sm font-medium text-gray-300 mb-1">Name</label>
          <input id="track-name" type="text" bind:value={trackTemplateForm.name} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:ring-2 focus:ring-blue-500" />
        </div>
        <div>
          <label for="track-description" class="block text-sm font-medium text-gray-300 mb-1">Description</label>
          <textarea id="track-description" bind:value={trackTemplateForm.description} rows="2" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:ring-2 focus:ring-blue-500"></textarea>
        </div>
        <div class="grid grid-cols-3 gap-4">
          <div>
            <label for="track-type" class="block text-sm font-medium text-gray-300 mb-1">Type</label>
            <select id="track-type" bind:value={trackTemplateForm.track_type} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200">
              {#each trackTypes as type (type)}
                <option value={type}>{formatTrackType(type)}</option>
              {/each}
            </select>
          </div>
          <div>
            <label for="track-category" class="block text-sm font-medium text-gray-300 mb-1">Category</label>
            <select id="track-category" bind:value={trackTemplateForm.category} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200">
              {#each trackCategories as cat (cat)}
                <option value={cat}>{cat}</option>
              {/each}
            </select>
          </div>
          <div>
            <label for="track-color" class="block text-sm font-medium text-gray-300 mb-1">Color</label>
            <input id="track-color" type="color" bind:value={trackTemplateForm.color} class="w-full h-10 bg-gray-700 border border-gray-600 rounded cursor-pointer" />
          </div>
        </div>
        <div class="grid grid-cols-3 gap-4">
          <div>
            <label for="track-midi-channel" class="block text-sm font-medium text-gray-300 mb-1">MIDI Channel</label>
            <input id="track-midi-channel" type="number" bind:value={trackTemplateForm.midi_channel} min="1" max="16" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
          </div>
          <div>
            <label for="track-volume" class="block text-sm font-medium text-gray-300 mb-1">Volume</label>
            <input id="track-volume" type="number" bind:value={trackTemplateForm.volume} step="0.1" min="0" max="1" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
          </div>
          <div>
            <label for="track-pan" class="block text-sm font-medium text-gray-300 mb-1">Pan</label>
            <input id="track-pan" type="number" bind:value={trackTemplateForm.pan} step="0.1" min="0" max="1" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
          </div>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="track-instrument" class="block text-sm font-medium text-gray-300 mb-1">Instrument Name</label>
            <input id="track-instrument" type="text" bind:value={trackTemplateForm.instrument_name} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
          </div>
          <div>
            <label for="track-height" class="block text-sm font-medium text-gray-300 mb-1">Default Height (px)</label>
            <input id="track-height" type="number" bind:value={trackTemplateForm.default_height} min="50" max="500" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
          </div>
        </div>

        <div class="border-t border-gray-700 pt-4">
          <h4 class="font-medium text-gray-200 mb-3">MIDI Routing</h4>
          <div class="grid grid-cols-2 gap-4">
            <label class="block">
              <span class="text-sm text-gray-400">Input Channel</span>
              <input type="number" bind:value={trackTemplateForm.routing.input_channel} min="0" max="16" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
            </label>
            <label class="block">
              <span class="text-sm text-gray-400">Output Channel</span>
              <input type="number" bind:value={trackTemplateForm.routing.output_channel} min="1" max="16" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
            </label>
          </div>
          <div class="mt-3">
            <label class="flex items-center text-sm text-gray-300">
              <input type="checkbox" bind:checked={trackTemplateForm.routing.thru_enabled} class="mr-2" />
              Enable MIDI Thru
            </label>
          </div>
        </div>

        <div class="flex items-center">
          <input type="checkbox" bind:checked={trackTemplateForm.is_factory} id="track-factory" class="mr-2" />
          <label for="track-factory" class="text-sm text-gray-300">Factory Template</label>
        </div>
      </div>
      <div class="px-6 py-4 border-t border-gray-700 flex justify-end space-x-3">
        <button class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded" on:click={() => showTrackTemplateModal = false}>
          Cancel
        </button>
        <button class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded" on:click={saveTrackTemplate} disabled={isSaving}>
          {isSaving ? 'Saving...' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Project Template Modal -->
{#if showProjectTemplateModal}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" role="button" tabindex="0" on:click|self={() => showProjectTemplateModal = false} on:keydown={(e) => { if (e.key === 'Escape') { showProjectTemplateModal = false; } }}>
    <div class="bg-gray-800 rounded-lg shadow-xl w-[650px] max-h-[85vh] overflow-y-auto">
      <div class="px-6 py-4 border-b border-gray-700">
        <h3 class="text-lg font-semibold text-gray-200">
          {editingProjectTemplate ? 'Edit Project Template' : 'New Project Template'}
        </h3>
      </div>
      <div class="p-6 space-y-4">
        <div>
          <label for="project-name" class="block text-sm font-medium text-gray-300 mb-1">Name</label>
          <input id="project-name" type="text" bind:value={projectTemplateForm.name} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:ring-2 focus:ring-blue-500" />
        </div>
        <div>
          <label for="project-description" class="block text-sm font-medium text-gray-300 mb-1">Description</label>
          <textarea id="project-description" bind:value={projectTemplateForm.description} rows="2" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:ring-2 focus:ring-blue-500"></textarea>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="project-category" class="block text-sm font-medium text-gray-300 mb-1">Category</label>
            <select id="project-category" bind:value={projectTemplateForm.category} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200">
              {#each projectCategories as cat (cat)}
                <option value={cat}>{cat}</option>
              {/each}
            </select>
          </div>
          <div>
            <label for="project-bpm" class="block text-sm font-medium text-gray-300 mb-1">BPM</label>
            <input id="project-bpm" type="number" bind:value={projectTemplateForm.bpm} min="30" max="300" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
          </div>
        </div>
        <div class="grid grid-cols-3 gap-4">
          <div>
            <label for="project-time-num" class="block text-sm font-medium text-gray-300 mb-1">Time Sig (Num)</label>
            <input id="project-time-num" type="number" bind:value={projectTemplateForm.time_signature_numerator} min="1" max="32" class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200" />
          </div>
          <div>
            <label for="project-time-den" class="block text-sm font-medium text-gray-300 mb-1">Time Sig (Den)</label>
            <select id="project-time-den" bind:value={projectTemplateForm.time_signature_denominator} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200">
              <option value={2}>2</option>
              <option value={4}>4</option>
              <option value={8}>8</option>
              <option value={16}>16</option>
            </select>
          </div>
          <div>
            <label for="project-key" class="block text-sm font-medium text-gray-300 mb-1">Key Signature</label>
            <select id="project-key" bind:value={projectTemplateForm.key_signature} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200">
              {#each keySignatures as key (key)}
                <option value={key}>{key}</option>
              {/each}
            </select>
          </div>
        </div>

        <div class="border-t border-gray-700 pt-4">
          <div class="flex justify-between items-center mb-3">
            <h4 class="font-medium text-gray-200">Tracks</h4>
            <button class="px-3 py-1 bg-blue-600 hover:bg-blue-500 text-white rounded text-sm" on:click={addProjectTrack}>
              + Add Track
            </button>
          </div>
          {#each projectTemplateForm.tracks as track, i (i)}
            <div class="bg-gray-700 p-3 rounded mb-2">
              <div class="flex justify-between items-center mb-2">
                <div class="flex items-center space-x-2">
                  <input type="color" bind:value={track.color} class="w-8 h-8 border-0 rounded cursor-pointer" />
                  <input type="text" bind:value={track.name} class="px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200 text-sm w-32" />
                </div>
                <button class="text-red-400 hover:text-red-300 text-sm" on:click={() => removeProjectTrack(i)}>Remove</button>
              </div>
              <div class="grid grid-cols-4 gap-2 text-sm">
                <div>
                  <label class="text-xs text-gray-400">Type</label>
                  <select bind:value={track.track_type} class="w-full px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200">
                    {#each trackTypes as type (type)}
                      <option value={type}>{formatTrackType(type)}</option>
                    {/each}
                  </select>
                </div>
                <div>
                  <label class="text-xs text-gray-400">MIDI Ch</label>
                  <input type="number" bind:value={track.midi_channel} min="1" max="16" class="w-full px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200" />
                </div>
                <div>
                  <label class="text-xs text-gray-400">Volume</label>
                  <input type="number" bind:value={track.volume} step="0.1" min="0" max="1" class="w-full px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200" />
                </div>
                <div>
                  <label class="text-xs text-gray-400">Instrument</label>
                  <input type="text" bind:value={track.instrument_name} class="w-full px-2 py-1 bg-gray-600 border border-gray-500 rounded text-gray-200" placeholder="Optional" />
                </div>
              </div>
            </div>
          {/each}
        </div>

        <div class="flex items-center">
          <input type="checkbox" bind:checked={projectTemplateForm.is_factory} id="project-factory" class="mr-2" />
          <label for="project-factory" class="text-sm text-gray-300">Factory Template</label>
        </div>
      </div>
      <div class="px-6 py-4 border-t border-gray-700 flex justify-end space-x-3">
        <button class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded" on:click={() => showProjectTemplateModal = false}>
          Cancel
        </button>
        <button class="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded" on:click={saveProjectTemplate} disabled={isSaving}>
          {isSaving ? 'Saving...' : 'Save'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Duplicate Project Modal -->
{#if showDuplicateProjectModal}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" role="button" tabindex="0" on:click|self={() => showDuplicateProjectModal = false} on:keydown={(e) => { if (e.key === 'Escape') { showDuplicateProjectModal = false; } }}>
    <div class="bg-gray-800 rounded-lg shadow-xl w-[400px]">
      <div class="px-6 py-4 border-b border-gray-700">
        <h3 class="text-lg font-semibold text-gray-200">Duplicate Project Template</h3>
      </div>
      <div class="p-6">
        <label class="block text-sm font-medium text-gray-300 mb-1">New Name</label>
        <input type="text" bind:value={duplicateProjectName} class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded text-gray-200 focus:ring-2 focus:ring-blue-500" />
      </div>
      <div class="px-6 py-4 border-t border-gray-700 flex justify-end space-x-3">
        <button class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-200 rounded" on:click={() => showDuplicateProjectModal = false}>
          Cancel
        </button>
        <button class="px-4 py-2 bg-green-600 hover:bg-green-500 text-white rounded" on:click={duplicateProjectTemplate} disabled={isSaving}>
          {isSaving ? 'Duplicating...' : 'Duplicate'}
        </button>
      </div>
    </div>
  </div>
{/if}
