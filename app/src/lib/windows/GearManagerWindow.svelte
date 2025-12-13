<script lang="ts">
  /**
   * GearManagerWindow.svelte
   *
   * Manages MIDI hardware gear profiles, CC mappings, programs, and user equipment.
   * Features three tabs: User Gear (personal collection), Profiles (gear library),
   * and Programs (patches/presets for selected gear).
   */

  import { onMount } from 'svelte';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import {
    gearStore,
    gearActions,
    filteredProfiles,
    filteredUserGear,
    profileCount,
    userGearCount,
  } from '$lib/stores';
  import type {
    GearProfile,
    GearType,
    CCMapping,
    UserGear,
    UserGearWithProfile,
    WindowId,
  } from '$lib/types';

  // Window props
  export let windowId: WindowId;

  // Local UI state
  let activeTab: 'user-gear' | 'profiles' | 'programs' = 'user-gear';
  let searchQuery = '';
  let filterType: GearType | null = null;

  // Modal states
  let showAddGearModal = false;
  let showEditGearModal = false;
  let showAddProfileModal = false;
  let showEditProfileModal = false;
  let showAddProgramModal = false;
  let showEditCCModal = false;

  // Form data for modals
  let newGearForm = {
    profile_id: 0,
    nickname: '',
    midi_input_port: '',
    midi_output_port: '',
    midi_channel: 1,
    notes: '',
    is_favorite: false,
  };

  let editGearForm: Partial<UserGear> = {};

  let newProfileForm = {
    name: '',
    manufacturer: '',
    gear_type: 'synthesizer' as GearType,
    description: '',
    midi_in_channels: [1],
    midi_out_channels: [1],
    default_program: 0,
    default_bank_msb: 0,
    default_bank_lsb: 0,
    icon: '',
    image_url: '',
    manual_url: '',
    is_factory: false,
  };

  let editProfileForm: Partial<GearProfile> = {};

  let newProgramForm = {
    gear_profile_id: 0,
    name: '',
    bank_msb: 0,
    bank_lsb: 0,
    program_number: 0,
    category: '',
    tags: [] as string[],
    description: '',
    is_factory: false,
  };

  let editCCForm: Partial<CCMapping> = {};
  let selectedCCMapping: CCMapping | null = null;

  // Gear type options
  const gearTypes: GearType[] = [
    'synthesizer',
    'drum_machine',
    'sampler',
    'effect',
    'controller',
    'sequencer',
    'interface',
    'other',
  ];

  // Reactive subscriptions
  $: profiles = $filteredProfiles;
  $: userGear = $filteredUserGear;
  $: selectedProfile = $gearStore.selectedProfile;
  $: ccMappings = $gearStore.ccMappings;
  $: programs = $gearStore.programs;
  $: selectedUserGear = $gearStore.selectedUserGear;
  $: isLoading = $gearStore.isLoading;
  $: isSaving = $gearStore.isSaving;
  $: error = $gearStore.error;

  // Update store search when local changes
  $: gearActions.setSearchQuery(searchQuery);
  $: gearActions.setFilterType(filterType);

  onMount(() => {
    void gearActions.initialize();
  });

  // Tab handlers
  function setTab(tab: 'user-gear' | 'profiles' | 'programs') {
    activeTab = tab;
    gearActions.setActiveTab(tab);
  }

  // User Gear handlers
  async function handleAddUserGear() {
    if (!newGearForm.profile_id) {
      return;
    }
    try {
      await gearActions.addUserGear(newGearForm);
      showAddGearModal = false;
      resetNewGearForm();
    } catch (err) {
      console.error('Failed to add gear:', err);
    }
  }

  async function handleUpdateUserGear() {
    if (!selectedUserGear?.id) {
      return;
    }
    try {
      await gearActions.updateUserGear(selectedUserGear.id, editGearForm);
      showEditGearModal = false;
    } catch (err) {
      console.error('Failed to update gear:', err);
    }
  }

  async function handleRemoveUserGear(id: number) {
    if (confirm('Are you sure you want to remove this gear from your collection?')) {
      await gearActions.removeUserGear(id);
    }
  }

  async function handleToggleFavorite(gear: UserGear) {
    await gearActions.setFavorite(gear.id, !gear.is_favorite);
  }

  function openEditGearModal(gear: UserGearWithProfile) {
    gearActions.selectUserGear(gear);
    editGearForm = {
      nickname: gear.nickname,
      midi_input_port: gear.midi_input_port,
      midi_output_port: gear.midi_output_port,
      midi_channel: gear.midi_channel,
      notes: gear.notes,
    };
    showEditGearModal = true;
  }

  async function handleSelectUserGear(gear: UserGear) {
    await gearActions.getUserGearWithProfile(gear.id);
  }

  // Profile handlers
  async function handleAddProfile() {
    try {
      await gearActions.createProfile(newProfileForm);
      showAddProfileModal = false;
      resetNewProfileForm();
    } catch (err) {
      console.error('Failed to add profile:', err);
    }
  }

  async function handleUpdateProfile() {
    if (!selectedProfile?.id) {
      return;
    }
    try {
      await gearActions.updateProfile(selectedProfile.id, editProfileForm);
      showEditProfileModal = false;
    } catch (err) {
      console.error('Failed to update profile:', err);
    }
  }

  async function handleDeleteProfile(id: number) {
    if (confirm('Are you sure you want to delete this gear profile? This will also remove all associated CC mappings and programs.')) {
      await gearActions.deleteProfile(id);
    }
  }

  function selectProfile(profile: GearProfile) {
    gearActions.selectProfile(profile);
  }

  function openEditProfileModal(profile: GearProfile) {
    gearActions.selectProfile(profile);
    editProfileForm = { ...profile };
    showEditProfileModal = true;
  }

  // Program handlers
  async function handleAddProgram() {
    if (!selectedProfile?.id) {
      return;
    }
    try {
      await gearActions.createProgram({
        ...newProgramForm,
        gear_profile_id: selectedProfile.id,
      });
      showAddProgramModal = false;
      resetNewProgramForm();
    } catch (err) {
      console.error('Failed to add program:', err);
    }
  }

  async function handleDeleteProgram(id: number) {
    if (confirm('Are you sure you want to delete this program?')) {
      await gearActions.deleteProgram(id);
    }
  }

  // CC Mapping handlers
  async function handleAddCCMapping() {
    if (!selectedProfile?.id) {
      return;
    }
    try {
      await gearActions.createCCMapping({
        gear_profile_id: selectedProfile.id,
        cc_number: 1,
        name: 'New CC',
        description: '',
        min_value: 0,
        max_value: 127,
        default_value: 64,
        is_bipolar: false,
        display_format: '{value}',
      });
    } catch (err) {
      console.error('Failed to add CC mapping:', err);
    }
  }

  async function handleUpdateCCMapping() {
    if (!selectedCCMapping?.id) {
      return;
    }
    try {
      await gearActions.updateCCMapping(selectedCCMapping.id, editCCForm);
      showEditCCModal = false;
      selectedCCMapping = null;
    } catch (err) {
      console.error('Failed to update CC mapping:', err);
    }
  }

  async function handleDeleteCCMapping(id: number) {
    if (confirm('Are you sure you want to delete this CC mapping?')) {
      await gearActions.deleteCCMapping(id);
    }
  }

  function openEditCCModal(cc: CCMapping) {
    selectedCCMapping = cc;
    editCCForm = { ...cc };
    showEditCCModal = true;
  }

  // Reset forms
  function resetNewGearForm() {
    newGearForm = {
      profile_id: 0,
      nickname: '',
      midi_input_port: '',
      midi_output_port: '',
      midi_channel: 1,
      notes: '',
      is_favorite: false,
    };
  }

  function resetNewProfileForm() {
    newProfileForm = {
      name: '',
      manufacturer: '',
      gear_type: 'synthesizer',
      description: '',
      midi_in_channels: [1],
      midi_out_channels: [1],
      default_program: 0,
      default_bank_msb: 0,
      default_bank_lsb: 0,
      icon: '',
      image_url: '',
      manual_url: '',
      is_factory: false,
    };
  }

  function resetNewProgramForm() {
    newProgramForm = {
      gear_profile_id: 0,
      name: '',
      bank_msb: 0,
      bank_lsb: 0,
      program_number: 0,
      category: '',
      tags: [],
      description: '',
      is_factory: false,
    };
  }

  // Format gear type for display
  function formatGearType(type: GearType): string {
    return type.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }
</script>

<WindowBase
  {windowId}
  title="Gear Manager"
  width={900}
  height={650}
>
  <div class="flex flex-col h-full bg-gray-900 text-gray-200">
    <!-- Header with tabs and search -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-700">
      <!-- Tabs -->
      <div class="flex gap-1">
        <button
          class="px-4 py-2 rounded-t-lg transition-colors {activeTab === 'user-gear'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-800 text-gray-400 hover:bg-gray-700'}"
          on:click={() => setTab('user-gear')}
        >
          My Gear ({$userGearCount})
        </button>
        <button
          class="px-4 py-2 rounded-t-lg transition-colors {activeTab === 'profiles'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-800 text-gray-400 hover:bg-gray-700'}"
          on:click={() => setTab('profiles')}
        >
          Profiles ({$profileCount})
        </button>
        <button
          class="px-4 py-2 rounded-t-lg transition-colors {activeTab === 'programs'
            ? 'bg-blue-600 text-white'
            : 'bg-gray-800 text-gray-400 hover:bg-gray-700'}"
          on:click={() => setTab('programs')}
          disabled={!selectedProfile}
          title={selectedProfile ? `Programs for ${selectedProfile.name}` : 'Select a profile first'}
        >
          Programs {selectedProfile ? `(${programs.length})` : ''}
        </button>
      </div>

      <!-- Search and filter -->
      <div class="flex gap-2">
        <input
          type="text"
          placeholder="Search..."
          class="px-3 py-1.5 bg-gray-800 border border-gray-600 rounded text-sm focus:outline-none focus:border-blue-500"
          bind:value={searchQuery}
        />
        {#if activeTab === 'profiles'}
          <select
            class="px-3 py-1.5 bg-gray-800 border border-gray-600 rounded text-sm focus:outline-none focus:border-blue-500"
            bind:value={filterType}
          >
            <option value={null}>All Types</option>
            {#each gearTypes as type (type)}
              <option value={type}>{formatGearType(type)}</option>
            {/each}
          </select>
        {/if}
      </div>
    </div>

    <!-- Error display -->
    {#if error}
      <div class="px-4 py-2 bg-red-900/50 text-red-300 text-sm">
        {error}
        <button class="ml-2 underline" on:click={() => gearActions.clearError()}>Dismiss</button>
      </div>
    {/if}

    <!-- Content area -->
    <div class="flex-1 overflow-hidden">
      {#if isLoading}
        <div class="flex items-center justify-center h-full">
          <div class="text-gray-400">Loading...</div>
        </div>
      {:else if activeTab === 'user-gear'}
        <!-- User Gear Tab -->
        <div class="flex h-full">
          <!-- Gear list -->
          <div class="w-1/3 border-r border-gray-700 overflow-y-auto">
            <div class="p-2">
              <button
                class="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
                on:click={() => (showAddGearModal = true)}
              >
                + Add Gear
              </button>
            </div>
            {#if userGear.length === 0}
              <div class="p-4 text-center text-gray-500">
                No gear in your collection yet.
              </div>
            {:else}
              <ul class="divide-y divide-gray-800">
                {#each userGear as gear (gear.id)}
                  <li
                    class="p-3 cursor-pointer hover:bg-gray-800 transition-colors {selectedUserGear?.id === gear.id ? 'bg-gray-800' : ''}"
                    on:click={() => handleSelectUserGear(gear)}
                    on:keypress={(e) => e.key === 'Enter' && handleSelectUserGear(gear)}
                    role="button"
                    tabindex="0"
                  >
                    <div class="flex items-center justify-between">
                      <div>
                        <div class="font-medium">{gear.nickname || `Gear #${gear.id}`}</div>
                        <div class="text-xs text-gray-500">Channel {gear.midi_channel}</div>
                      </div>
                      <button
                        class="p-1 hover:text-yellow-400 transition-colors"
                        on:click|stopPropagation={() => handleToggleFavorite(gear)}
                        title={gear.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
                      >
                        {gear.is_favorite ? '★' : '☆'}
                      </button>
                    </div>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>

          <!-- Gear details -->
          <div class="flex-1 p-4 overflow-y-auto">
            {#if selectedUserGear}
              <div class="space-y-4">
                <div class="flex items-start justify-between">
                  <div>
                    <h2 class="text-xl font-semibold">{selectedUserGear.nickname || selectedUserGear.profile.name}</h2>
                    <p class="text-gray-400">{selectedUserGear.profile.manufacturer} - {formatGearType(selectedUserGear.profile.gear_type)}</p>
                  </div>
                  <div class="flex gap-2">
                    <button
                      class="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 rounded text-sm transition-colors"
                      on:click={() => openEditGearModal(selectedUserGear)}
                    >
                      Edit
                    </button>
                    <button
                      class="px-3 py-1.5 bg-red-600 hover:bg-red-700 rounded text-sm transition-colors"
                      on:click={() => handleRemoveUserGear(selectedUserGear.id)}
                    >
                      Remove
                    </button>
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-4">
                  <div class="p-3 bg-gray-800 rounded">
                    <div class="text-xs text-gray-500 mb-1">MIDI Channel</div>
                    <div class="font-medium">{selectedUserGear.midi_channel}</div>
                  </div>
                  <div class="p-3 bg-gray-800 rounded">
                    <div class="text-xs text-gray-500 mb-1">Last Used</div>
                    <div class="font-medium">{selectedUserGear.last_used ? new Date(selectedUserGear.last_used).toLocaleDateString() : 'Never'}</div>
                  </div>
                  <div class="p-3 bg-gray-800 rounded">
                    <div class="text-xs text-gray-500 mb-1">Input Port</div>
                    <div class="font-medium">{selectedUserGear.midi_input_port || 'Not set'}</div>
                  </div>
                  <div class="p-3 bg-gray-800 rounded">
                    <div class="text-xs text-gray-500 mb-1">Output Port</div>
                    <div class="font-medium">{selectedUserGear.midi_output_port || 'Not set'}</div>
                  </div>
                </div>

                {#if selectedUserGear.notes}
                  <div class="p-3 bg-gray-800 rounded">
                    <div class="text-xs text-gray-500 mb-1">Notes</div>
                    <div class="text-sm whitespace-pre-wrap">{selectedUserGear.notes}</div>
                  </div>
                {/if}

                {#if selectedUserGear.profile.description}
                  <div class="p-3 bg-gray-800 rounded">
                    <div class="text-xs text-gray-500 mb-1">Description</div>
                    <div class="text-sm">{selectedUserGear.profile.description}</div>
                  </div>
                {/if}

                <!-- Quick actions -->
                <div class="flex gap-2">
                  <button
                    class="px-3 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm transition-colors"
                    on:click={() => {
                      selectProfile(selectedUserGear.profile);
                      setTab('programs');
                    }}
                  >
                    View Programs
                  </button>
                  {#if selectedUserGear.profile.manual_url}
                    <a
                      href={selectedUserGear.profile.manual_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="px-3 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm transition-colors"
                    >
                      Manual
                    </a>
                  {/if}
                </div>
              </div>
            {:else}
              <div class="flex items-center justify-center h-full text-gray-500">
                Select a gear item to view details
              </div>
            {/if}
          </div>
        </div>
      {:else if activeTab === 'profiles'}
        <!-- Profiles Tab -->
        <div class="flex h-full">
          <!-- Profile list -->
          <div class="w-1/3 border-r border-gray-700 overflow-y-auto">
            <div class="p-2">
              <button
                class="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
                on:click={() => (showAddProfileModal = true)}
              >
                + Add Profile
              </button>
            </div>
            {#if profiles.length === 0}
              <div class="p-4 text-center text-gray-500">
                No profiles found.
              </div>
            {:else}
              <ul class="divide-y divide-gray-800">
                {#each profiles as profile (profile.id)}
                  <li
                    class="p-3 cursor-pointer hover:bg-gray-800 transition-colors {selectedProfile?.id === profile.id ? 'bg-gray-800' : ''}"
                    on:click={() => selectProfile(profile)}
                    on:keypress={(e) => e.key === 'Enter' && selectProfile(profile)}
                    role="button"
                    tabindex="0"
                  >
                    <div class="font-medium">{profile.name}</div>
                    <div class="text-xs text-gray-500">{profile.manufacturer}</div>
                    <div class="text-xs text-gray-600 mt-1">{formatGearType(profile.gear_type)}</div>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>

          <!-- Profile details & CC mappings -->
          <div class="flex-1 overflow-hidden flex flex-col">
            {#if selectedProfile}
              <div class="p-4 border-b border-gray-700">
                <div class="flex items-start justify-between">
                  <div>
                    <h2 class="text-xl font-semibold">{selectedProfile.name}</h2>
                    <p class="text-gray-400">{selectedProfile.manufacturer}</p>
                    <p class="text-sm text-gray-500 mt-1">{selectedProfile.description}</p>
                  </div>
                  <div class="flex gap-2">
                    <button
                      class="px-3 py-1.5 bg-gray-700 hover:bg-gray-600 rounded text-sm transition-colors"
                      on:click={() => openEditProfileModal(selectedProfile)}
                    >
                      Edit
                    </button>
                    {#if !selectedProfile.is_factory}
                      <button
                        class="px-3 py-1.5 bg-red-600 hover:bg-red-700 rounded text-sm transition-colors"
                        on:click={() => handleDeleteProfile(selectedProfile.id)}
                      >
                        Delete
                      </button>
                    {/if}
                  </div>
                </div>
              </div>

              <!-- CC Mappings -->
              <div class="flex-1 overflow-y-auto p-4">
                <div class="flex items-center justify-between mb-3">
                  <h3 class="font-medium">CC Mappings</h3>
                  <button
                    class="px-2 py-1 bg-gray-700 hover:bg-gray-600 rounded text-xs transition-colors"
                    on:click={handleAddCCMapping}
                  >
                    + Add CC
                  </button>
                </div>
                {#if ccMappings.length === 0}
                  <div class="text-center text-gray-500 py-4">No CC mappings defined.</div>
                {:else}
                  <div class="grid grid-cols-2 gap-2">
                    {#each ccMappings as cc (cc.id)}
                      <div class="p-2 bg-gray-800 rounded flex items-center justify-between">
                        <div>
                          <div class="text-sm font-medium">CC{cc.cc_number}: {cc.name}</div>
                          <div class="text-xs text-gray-500">{cc.min_value}-{cc.max_value}</div>
                        </div>
                        <div class="flex gap-1">
                          <button
                            class="p-1 text-gray-400 hover:text-white transition-colors"
                            on:click={() => openEditCCModal(cc)}
                            title="Edit"
                          >
                            ✎
                          </button>
                          <button
                            class="p-1 text-gray-400 hover:text-red-400 transition-colors"
                            on:click={() => handleDeleteCCMapping(cc.id)}
                            title="Delete"
                          >
                            ×
                          </button>
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {:else}
              <div class="flex items-center justify-center h-full text-gray-500">
                Select a profile to view details and CC mappings
              </div>
            {/if}
          </div>
        </div>
      {:else if activeTab === 'programs'}
        <!-- Programs Tab -->
        <div class="flex h-full">
          <div class="w-1/3 border-r border-gray-700 overflow-y-auto">
            <div class="p-2">
              <button
                class="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm font-medium transition-colors"
                on:click={() => (showAddProgramModal = true)}
                disabled={!selectedProfile}
              >
                + Add Program
              </button>
            </div>
            {#if !selectedProfile}
              <div class="p-4 text-center text-gray-500">
                Select a profile first to view programs.
              </div>
            {:else if programs.length === 0}
              <div class="p-4 text-center text-gray-500">
                No programs for {selectedProfile.name}.
              </div>
            {:else}
              <ul class="divide-y divide-gray-800">
                {#each programs as program (program.id)}
                  <li class="p-3 hover:bg-gray-800 transition-colors">
                    <div class="flex items-center justify-between">
                      <div>
                        <div class="font-medium">{program.name}</div>
                        <div class="text-xs text-gray-500">
                          Bank {program.bank_msb}/{program.bank_lsb} • PC {program.program_number}
                        </div>
                        {#if program.category}
                          <div class="text-xs text-blue-400 mt-1">{program.category}</div>
                        {/if}
                      </div>
                      <button
                        class="p-1 text-gray-400 hover:text-red-400 transition-colors"
                        on:click={() => handleDeleteProgram(program.id)}
                        title="Delete"
                      >
                        ×
                      </button>
                    </div>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>

          <!-- Program details (placeholder) -->
          <div class="flex-1 p-4 overflow-y-auto">
            {#if selectedProfile}
              <div class="text-center text-gray-500 py-8">
                <p class="text-lg mb-2">{selectedProfile.name}</p>
                <p class="text-sm">{programs.length} programs available</p>
                <p class="text-xs mt-4">Click a program to send program change to the device</p>
              </div>
            {:else}
              <div class="flex items-center justify-center h-full text-gray-500">
                Select a profile from the Profiles tab first
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Add Gear Modal -->
  {#if showAddGearModal}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={() => (showAddGearModal = false)} on:keydown={() => {}}>
      <div class="bg-gray-800 rounded-lg p-6 w-96" on:click|stopPropagation on:keydown|stopPropagation>
        <h3 class="text-lg font-semibold mb-4">Add Gear to Collection</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm text-gray-400 mb-1">Profile</label>
            <select
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={newGearForm.profile_id}
            >
              <option value={0}>Select a profile...</option>
              {#each profiles as profile (profile.id)}
                <option value={profile.id}>{profile.name} ({profile.manufacturer})</option>
              {/each}
            </select>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Nickname</label>
            <input
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={newGearForm.nickname}
              placeholder="My Synth"
            />
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">MIDI Channel</label>
            <input
              type="number"
              min="1"
              max="16"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={newGearForm.midi_channel}
            />
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Notes</label>
            <textarea
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500 resize-none"
              rows="3"
              bind:value={newGearForm.notes}
              placeholder="Any notes about this gear..."
            ></textarea>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            on:click={() => (showAddGearModal = false)}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            on:click={handleAddUserGear}
            disabled={!newGearForm.profile_id || isSaving}
          >
            {isSaving ? 'Adding...' : 'Add Gear'}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Edit Gear Modal -->
  {#if showEditGearModal && selectedUserGear}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={() => (showEditGearModal = false)} on:keydown={() => {}}>
      <div class="bg-gray-800 rounded-lg p-6 w-96" on:click|stopPropagation on:keydown|stopPropagation>
        <h3 class="text-lg font-semibold mb-4">Edit Gear</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm text-gray-400 mb-1">Nickname</label>
            <input
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={editGearForm.nickname}
            />
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">MIDI Channel</label>
            <input
              type="number"
              min="1"
              max="16"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={editGearForm.midi_channel}
            />
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Input Port</label>
            <input
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={editGearForm.midi_input_port}
              placeholder="e.g., MIDI In 1"
            />
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Output Port</label>
            <input
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={editGearForm.midi_output_port}
              placeholder="e.g., MIDI Out 1"
            />
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Notes</label>
            <textarea
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500 resize-none"
              rows="3"
              bind:value={editGearForm.notes}
            ></textarea>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            on:click={() => (showEditGearModal = false)}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            on:click={handleUpdateUserGear}
            disabled={isSaving}
          >
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Add Profile Modal -->
  {#if showAddProfileModal}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={() => (showAddProfileModal = false)} on:keydown={() => {}}>
      <div class="bg-gray-800 rounded-lg p-6 w-[480px] max-h-[80vh] overflow-y-auto" on:click|stopPropagation on:keydown|stopPropagation>
        <h3 class="text-lg font-semibold mb-4">Add Gear Profile</h3>
        <div class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm text-gray-400 mb-1">Name</label>
              <input
                type="text"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProfileForm.name}
                placeholder="e.g., Prophet 6"
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Manufacturer</label>
              <input
                type="text"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProfileForm.manufacturer}
                placeholder="e.g., Sequential"
              />
            </div>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Type</label>
            <select
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={newProfileForm.gear_type}
            >
              {#each gearTypes as type (type)}
                <option value={type}>{formatGearType(type)}</option>
              {/each}
            </select>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Description</label>
            <textarea
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500 resize-none"
              rows="3"
              bind:value={newProfileForm.description}
              placeholder="Brief description of this gear..."
            ></textarea>
          </div>
          <div class="grid grid-cols-3 gap-4">
            <div>
              <label class="block text-sm text-gray-400 mb-1">Default Program</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProfileForm.default_program}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Bank MSB</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProfileForm.default_bank_msb}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Bank LSB</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProfileForm.default_bank_lsb}
              />
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            on:click={() => (showAddProfileModal = false)}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            on:click={handleAddProfile}
            disabled={!newProfileForm.name || !newProfileForm.manufacturer || isSaving}
          >
            {isSaving ? 'Adding...' : 'Add Profile'}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Edit Profile Modal -->
  {#if showEditProfileModal && selectedProfile}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={() => (showEditProfileModal = false)} on:keydown={() => {}}>
      <div class="bg-gray-800 rounded-lg p-6 w-[480px] max-h-[80vh] overflow-y-auto" on:click|stopPropagation on:keydown|stopPropagation>
        <h3 class="text-lg font-semibold mb-4">Edit Profile</h3>
        <div class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm text-gray-400 mb-1">Name</label>
              <input
                type="text"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={editProfileForm.name}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Manufacturer</label>
              <input
                type="text"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={editProfileForm.manufacturer}
              />
            </div>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Type</label>
            <select
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={editProfileForm.gear_type}
            >
              {#each gearTypes as type (type)}
                <option value={type}>{formatGearType(type)}</option>
              {/each}
            </select>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Description</label>
            <textarea
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500 resize-none"
              rows="3"
              bind:value={editProfileForm.description}
            ></textarea>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            on:click={() => (showEditProfileModal = false)}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            on:click={handleUpdateProfile}
            disabled={isSaving}
          >
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Add Program Modal -->
  {#if showAddProgramModal && selectedProfile}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={() => (showAddProgramModal = false)} on:keydown={() => {}}>
      <div class="bg-gray-800 rounded-lg p-6 w-96" on:click|stopPropagation on:keydown|stopPropagation>
        <h3 class="text-lg font-semibold mb-4">Add Program to {selectedProfile.name}</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm text-gray-400 mb-1">Name</label>
            <input
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={newProgramForm.name}
              placeholder="e.g., Classic Pad"
            />
          </div>
          <div class="grid grid-cols-3 gap-4">
            <div>
              <label class="block text-sm text-gray-400 mb-1">Program #</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProgramForm.program_number}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Bank MSB</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProgramForm.bank_msb}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Bank LSB</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={newProgramForm.bank_lsb}
              />
            </div>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Category</label>
            <input
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={newProgramForm.category}
              placeholder="e.g., Pads, Bass, Lead"
            />
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            on:click={() => (showAddProgramModal = false)}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            on:click={handleAddProgram}
            disabled={!newProgramForm.name || isSaving}
          >
            {isSaving ? 'Adding...' : 'Add Program'}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Edit CC Modal -->
  {#if showEditCCModal && selectedCCMapping}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" on:click={() => (showEditCCModal = false)} on:keydown={() => {}}>
      <div class="bg-gray-800 rounded-lg p-6 w-96" on:click|stopPropagation on:keydown|stopPropagation>
        <h3 class="text-lg font-semibold mb-4">Edit CC Mapping</h3>
        <div class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm text-gray-400 mb-1">CC Number</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={editCCForm.cc_number}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Name</label>
              <input
                type="text"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={editCCForm.name}
              />
            </div>
          </div>
          <div>
            <label class="block text-sm text-gray-400 mb-1">Description</label>
            <input
              type="text"
              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              bind:value={editCCForm.description}
            />
          </div>
          <div class="grid grid-cols-3 gap-4">
            <div>
              <label class="block text-sm text-gray-400 mb-1">Min</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={editCCForm.min_value}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Max</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={editCCForm.max_value}
              />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Default</label>
              <input
                type="number"
                min="0"
                max="127"
                class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
                bind:value={editCCForm.default_value}
              />
            </div>
          </div>
          <div class="flex items-center gap-2">
            <input
              type="checkbox"
              id="is_bipolar"
              bind:checked={editCCForm.is_bipolar}
              class="rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
            />
            <label for="is_bipolar" class="text-sm text-gray-400">Bipolar (center = 64)</label>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-6">
          <button
            class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            on:click={() => (showEditCCModal = false)}
          >
            Cancel
          </button>
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
            on:click={handleUpdateCCMapping}
            disabled={isSaving}
          >
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </div>
    </div>
  {/if}
</WindowBase>
