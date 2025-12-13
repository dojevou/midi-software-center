<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api } from '$lib/api';
  import { Commands } from '$lib/api/commands';
  import { safeInvoke, safeListen } from '$lib/utils/tauri';
  import type { EffectSlot, MixerChannel, MixerState } from '$lib/types';
  import { arrangementStore, arrangementActions } from '$lib/stores/arrangementStore';

  // Props
  export let showToolbar: boolean = true;

  // Detach state from arrangement store
  $: isDetached = $arrangementStore.mixerDetached;

  let mixerState: MixerState | null = null;
  let meterData: any[] = [];
  let loadError: string | null = null;
  let isLoading = true;

  // Effect rack state
  let channelEffects: Record<number, EffectSlot[]> = {};
  let showEffectRack = true;
  let selectedChannelForEffects: number | null = null;

  // Mock data for browser development (when not in Tauri)
  const mockMixerState: MixerState = {
    channels: {
      1: {
        id: 1,
        channel_type: 'audio',
        label: 'Track 1',
        volume: 0.75,
        pan: -0.2,
        muted: false,
        soloed: false,
        meter_level: 0.4,
      },
      2: {
        id: 2,
        channel_type: 'audio',
        label: 'Track 2',
        volume: 0.65,
        pan: 0.3,
        muted: false,
        soloed: false,
        meter_level: 0.6,
      },
      3: {
        id: 3,
        channel_type: 'audio',
        label: 'Drums',
        volume: 0.9,
        pan: 0,
        muted: false,
        soloed: false,
        meter_level: 0.8,
      },
      4: {
        id: 4,
        channel_type: 'audio',
        label: 'Bass',
        volume: 0.7,
        pan: 0.1,
        muted: true,
        soloed: false,
        meter_level: 0.3,
      },
      5: {
        id: 5,
        channel_type: 'midi',
        label: 'Synth',
        volume: 0.8,
        pan: -0.1,
        muted: false,
        soloed: true,
        meter_level: 0.5,
      },
    },
    master: {
      id: 0,
      channel_type: 'master',
      label: 'Master',
      volume: 0.85,
      pan: 0,
      muted: false,
      soloed: false,
      meter_level: 0.7,
    },
    show_meters: true,
    show_effects: true,
  };

  // Reactive: Convert channels object to sorted array for rendering
  $: channelList = mixerState ? Object.values(mixerState.channels).sort((a, b) => a.id - b.id) : [];
  $: masterVolume = mixerState?.master?.volume ?? 1.0;

  onMount(async () => {
    // Listen for meter updates from backend (only in Tauri context)
    const unlistenMeters = await safeListen<any[]>('mixer::meter-update', (payload) => {
      meterData = payload;
    });
    if (unlistenMeters) {
      onDestroy(() => unlistenMeters());
    }

    // Listen for track changes to refresh mixer state
    const unlistenTrackAdded = await safeListen<{ track_id: number }>('track-added', async (payload) => {
      console.log('[Mixer] Track added event received:', payload);
      await loadMixerState();
    });
    if (unlistenTrackAdded) {
      onDestroy(() => unlistenTrackAdded());
    }

    // Listen for track removal to refresh mixer state
    const unlistenTrackRemoved = await safeListen<{ track_id: number }>('track-removed', async (payload) => {
      console.log('[Mixer] Track removed event received:', payload);
      await loadMixerState();
    });
    if (unlistenTrackRemoved) {
      onDestroy(() => unlistenTrackRemoved());
    }

    // Load initial mixer state
    await loadMixerState();
  });

  async function loadMixerState(retryCount = 0) {
    const maxRetries = 3;
    const baseDelay = 300; // ms

    console.log(`[Mixer] loadMixerState called (attempt ${retryCount + 1}/${maxRetries + 1})`);

    try {
      console.log('[Mixer] Calling api.window.getMixerState()...');
      mixerState = await api.window.getMixerState();
      console.log('[Mixer] SUCCESS - Loaded mixer state:', mixerState);
      loadError = null;
      isLoading = false;
    } catch (error) {
      console.error('[Mixer] FAILED to load mixer state:', error);

      // Retry with exponential backoff if Tauri context isn't ready yet
      if (retryCount < maxRetries) {
        const delay = baseDelay * Math.pow(2, retryCount);
        console.log(`[Mixer] Retrying in ${delay}ms...`);
        setTimeout(() => loadMixerState(retryCount + 1), delay);
      } else {
        console.error('[Mixer] Max retries reached. Using mock data as fallback.');
        // Fall back to mock data so the UI is still usable
        mixerState = mockMixerState;
        loadError = 'Could not connect to audio backend. Using demo mode.';
        isLoading = false;
      }
    }
  }

  async function updateVolume(channelId: number, volume: number) {
    try {
      await api.window.setChannelVolume(channelId, volume);
      await loadMixerState(); // Refresh state
    } catch (error) {
      console.error('Failed to update volume:', error);
    }
  }

  async function updatePan(channelId: number, pan: number) {
    try {
      await api.window.setChannelPan(channelId, pan);
      await loadMixerState(); // Refresh state
    } catch (error) {
      console.error('Failed to update pan:', error);
    }
  }

  async function toggleMute(channelId: number) {
    try {
      const channel = mixerState?.channels[channelId];
      if (channel) {
        await api.window.setChannelMute(channelId, !channel.muted);
        await loadMixerState(); // Refresh state
      }
    } catch (error) {
      console.error('Failed to toggle mute:', error);
    }
  }

  async function toggleSolo(channelId: number) {
    try {
      const channel = mixerState?.channels[channelId];
      if (channel) {
        await api.window.setChannelSolo(channelId, !channel.soloed);
        await loadMixerState(); // Refresh state
      }
    } catch (error) {
      console.error('Failed to toggle solo:', error);
    }
  }

  async function updateMasterVolume(volume: number) {
    try {
      // Call backend to set master volume through mixer API
      await safeInvoke(Commands.MIXER_SET_MASTER_VOLUME, { volume });
      await loadMixerState(); // Refresh state to get updated master
      console.log('Master volume updated to:', volume);
    } catch (error) {
      console.error('Failed to set master volume:', error);
      throw error;
    }
  }

  function formatVolume(volume: number): string {
    return `${Math.round(volume * 100)}%`;
  }

  function formatPan(pan: number): string {
    if (pan < -0.5) {
      return 'L';
    }
    if (pan > 0.5) {
      return 'R';
    }
    return 'C';
  }

  function getMeterLevel(meterData: any[], channelId: number, side: 'left' | 'right'): number {
    const meter = meterData.find((m) => m.track_id === channelId);
    if (meter) {
      const level = side === 'left' ? meter.vu_left : meter.vu_right;
      return Math.max(0, Math.min(32, ((level + 60) / 60) * 32)); // -60dB to 0dB -> 0-32px
    }
    return 0;
  }

  function getMasterMeter(meterData: any[], side: 'left' | 'right'): number {
    // Assume master is average of all channels or from last entry
    if (meterData.length > 0) {
      const lastMeter = meterData[meterData.length - 1];
      const level = side === 'left' ? lastMeter.vu_left : lastMeter.vu_right;
      return Math.max(0, Math.min(32, ((level + 60) / 60) * 32));
    }
    return 0;
  }

  function getMeterColor(level: number): string {
    if (level > 28) {
      return 'bg-red-500';
    }
    if (level > 20) {
      return 'bg-yellow-500';
    }
    return 'bg-green-500';
  }

  // Effect rack functions
  function getChannelEffects(channelId: number): EffectSlot[] {
    return channelEffects[channelId] || [];
  }

  function selectChannelEffects(channelId: number) {
    selectedChannelForEffects = selectedChannelForEffects === channelId ? null : channelId;
  }

  async function toggleEffectEnabled(channelId: number | null, effectId: number) {
    if (channelId === null) {
      return;
    }
    try {
      const newState = await api.mixerEffects.toggleChannelEffect(channelId, effectId);
      const effects = getChannelEffects(channelId);
      const effect = effects.find((e) => e.id === effectId);
      if (effect) {
        effect.enabled = newState;
        channelEffects[channelId] = [...effects];
      }
    } catch (error) {
      console.error('Failed to toggle effect:', error);
    }
  }

  async function updateEffectWetDry(channelId: number | null, effectId: number, value: number) {
    if (channelId === null) {
      return;
    }
    try {
      await api.mixerEffects.setParameter(channelId, effectId, 'wet_dry', value);
      const effects = getChannelEffects(channelId);
      const effect = effects.find((e) => e.id === effectId);
      if (effect) {
        effect.wet_dry = value;
        channelEffects[channelId] = [...effects];
      }
    } catch (error) {
      console.error('Failed to update wet/dry:', error);
    }
  }

  async function moveEffectUp(channelId: number | null, effectIndex: number) {
    if (channelId === null || effectIndex <= 0) {
      return;
    }
    const effects = getChannelEffects(channelId);
    const newEffects = [...effects];
    [newEffects[effectIndex - 1], newEffects[effectIndex]] = [
      newEffects[effectIndex],
      newEffects[effectIndex - 1],
    ];

    try {
      await api.mixerEffects.reorder(
        channelId,
        newEffects.map((e) => e.id)
      );
      channelEffects[channelId] = newEffects;
    } catch (error) {
      console.error('Failed to reorder effects:', error);
    }
  }

  async function moveEffectDown(channelId: number | null, effectIndex: number) {
    if (channelId === null) {
      return;
    }
    const effects = getChannelEffects(channelId);
    if (effectIndex >= effects.length - 1) {
      return;
    }
    const newEffects = [...effects];
    [newEffects[effectIndex], newEffects[effectIndex + 1]] = [
      newEffects[effectIndex + 1],
      newEffects[effectIndex],
    ];

    try {
      await api.mixerEffects.reorder(
        channelId,
        newEffects.map((e) => e.id)
      );
      channelEffects[channelId] = newEffects;
    } catch (error) {
      console.error('Failed to reorder effects:', error);
    }
  }

  function toggleEffectRackVisibility() {
    showEffectRack = !showEffectRack;
  }

  // Initialize demo effects for testing UI
  function initDemoEffects() {
    // Add some demo effects for UI testing (would be loaded from backend in production)
    channelEffects = {
      0: [
        { id: 1, name: 'EQ', enabled: true, wet_dry: 1.0, parameters: { gain: 0 } },
        { id: 2, name: 'Compressor', enabled: false, wet_dry: 0.5, parameters: { threshold: -10 } },
      ],
      1: [{ id: 3, name: 'Reverb', enabled: true, wet_dry: 0.3, parameters: { room_size: 0.5 } }],
    };
  }

  // Call after loading mixer state
  $: if (mixerState && channelList.length > 0) {
    // Initialize demo effects when channels are available
    if (Object.keys(channelEffects).length === 0) {
      initDemoEffects();
    }
  }
</script>

<div class="mixer-window dark:bg-window dark:text-app-text p-4 h-full flex flex-col">
  <!-- Mixer Toolbar -->
  {#if showToolbar}
    <div class="mixer-toolbar flex items-center justify-between mb-3 pb-2 border-b dark:border-window-border">
      <div class="toolbar-left flex items-center gap-2">
        <h3 class="dark:text-gray-200 font-medium text-sm">Mixer</h3>
        <button
          on:click={toggleEffectRackVisibility}
          class="toolbar-btn text-xs px-2 py-1 rounded {showEffectRack ? 'dark:bg-accent' : 'dark:bg-secondary'} dark:text-gray-200 hover:opacity-80"
          title="Toggle Effect Rack"
        >
          FX Rack
        </button>
      </div>
      <div class="toolbar-right flex items-center gap-2">
        <button
          on:click={() => arrangementActions.toggleMixerDetached()}
          class="detach-btn text-xs px-2 py-1 rounded dark:bg-secondary dark:text-gray-200 hover:opacity-80 flex items-center gap-1"
          title={isDetached ? 'Dock Mixer' : 'Detach Mixer'}
        >
          <span>{isDetached ? '📌' : '🔲'}</span>
          <span>{isDetached ? 'Dock' : 'Detach'}</span>
        </button>
      </div>
    </div>
  {/if}

  {#if loadError}
    <div class="demo-banner bg-yellow-600 text-white text-xs px-3 py-1 rounded mb-2 text-center">
      ⚠️ {loadError}
    </div>
  {/if}
  {#if mixerState}
    <div class="channels flex space-x-4 overflow-x-auto pb-4 flex-1">
      {#if channelList.length === 0}
        <div
          class="no-tracks dark:text-gray-500 text-center w-full flex flex-col items-center justify-center"
        >
          <div class="text-sm">No tracks loaded</div>
          <div class="text-xs mt-1">Add tracks from the Database window</div>
        </div>
      {/if}
      {#each channelList as channel (channel.id)}
        <div
          class="channel-strip dark:bg-window-subtle p-3 rounded border dark:border-window-border w-20 flex flex-col items-center space-y-2"
        >
          <!-- Track Name -->
          <div
            class="track-name text-center text-xs dark:text-gray-300 truncate w-full"
            title={channel.label}
          >
            {channel.label || `Track ${channel.id}`}
          </div>

          <!-- Volume Fader -->
          <div class="volume-fader flex flex-col items-center space-y-1">
            <label class="volume-label text-xs dark:text-gray-400">Vol</label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={channel.volume}
              on:input={(e) => updateVolume(channel.id, parseFloat(e.currentTarget.value))}
              class="volume-slider dark:bg-input w-4 h-32"
            />
            <span class="volume-display text-xs dark:text-gray-300"
              >{formatVolume(channel.volume)}</span
            >
          </div>

          <!-- Pan Control -->
          <div class="pan-control flex flex-col items-center space-y-1">
            <label class="pan-label text-xs dark:text-gray-400">Pan</label>
            <input
              type="range"
              min="-1"
              max="1"
              step="0.01"
              value={channel.pan}
              on:input={(e) => updatePan(channel.id, parseFloat(e.currentTarget.value))}
              class="pan-slider dark:bg-input w-16 h-2"
            />
            <span class="pan-display text-xs dark:text-gray-300">{formatPan(channel.pan)}</span>
          </div>

          <!-- VU Meters -->
          <div class="vu-meters flex space-x-1">
            <div
              class={getMeterColor(getMeterLevel(meterData, channel.id, 'left'))}
              style="height: {getMeterLevel(
                meterData,
                channel.id,
                'left'
              )}px; width: 2px; background: currentColor; rounded;"
            ></div>
            <div
              class={getMeterColor(getMeterLevel(meterData, channel.id, 'right'))}
              style="height: {getMeterLevel(
                meterData,
                channel.id,
                'right'
              )}px; width: 2px; background: currentColor; rounded;"
            ></div>
          </div>

          <!-- Mute/Solo/FX Buttons -->
          <div class="controls flex flex-col space-y-1">
            <button
              on:click={() => toggleMute(channel.id)}
              class="mute-btn {channel.muted
                ? 'dark:bg-error text-white'
                : 'dark:bg-secondary dark:text-gray-300'} px-2 py-1 rounded text-xs hover:opacity-80"
            >
              M
            </button>
            <button
              on:click={() => toggleSolo(channel.id)}
              class="solo-btn {channel.soloed
                ? 'dark:bg-primary text-white'
                : 'dark:bg-secondary dark:text-gray-300'} px-2 py-1 rounded text-xs hover:opacity-80"
            >
              S
            </button>
            <button
              on:click={() => selectChannelEffects(channel.id)}
              class="fx-btn {selectedChannelForEffects === channel.id
                ? 'dark:bg-accent text-white'
                : 'dark:bg-secondary dark:text-gray-300'} px-2 py-1 rounded text-xs hover:opacity-80"
              title="Effects"
            >
              FX
            </button>
          </div>

          <!-- Effect Slots Mini View -->
          {#if showEffectRack && getChannelEffects(channel.id).length > 0}
            <div class="effect-slots-mini mt-2 w-full">
              {#each getChannelEffects(channel.id).slice(0, 3) as effect (effect.id)}
                <div
                  class="effect-mini text-xs px-1 py-0.5 rounded mb-0.5 truncate {effect.enabled
                    ? 'dark:bg-accent-dark dark:text-white'
                    : 'dark:bg-gray-700 dark:text-gray-500'}"
                  title="{effect.name} - {effect.enabled ? 'On' : 'Off'}"
                >
                  {effect.name}
                </div>
              {/each}
              {#if getChannelEffects(channel.id).length > 3}
                <div class="text-xs dark:text-gray-500 text-center">
                  +{getChannelEffects(channel.id).length - 3}
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Effect Rack Panel -->
    {#if selectedChannelForEffects !== null}
      <div
        class="effect-rack dark:bg-window-subtle p-3 rounded mt-2 border dark:border-window-border"
      >
        <div class="flex items-center justify-between mb-3">
          <h3 class="dark:text-gray-200 font-medium">
            Effects: {channelList.find((c) => c.id === selectedChannelForEffects)?.label ||
              `Channel ${selectedChannelForEffects}`}
          </h3>
          <button
            on:click={() => (selectedChannelForEffects = null)}
            class="dark:text-gray-400 hover:dark:text-white text-lg"
            title="Close"
          >
            ×
          </button>
        </div>

        {#if getChannelEffects(selectedChannelForEffects).length === 0}
          <div class="dark:text-gray-500 text-sm text-center py-4">No effects on this channel</div>
        {:else}
          <div class="effects-list space-y-2">
            {#each getChannelEffects(selectedChannelForEffects) as effect, index (effect.id)}
              <div
                class="effect-slot dark:bg-window p-2 rounded border dark:border-window-border flex items-center space-x-3"
              >
                <!-- Reorder Buttons -->
                <div class="reorder-btns flex flex-col">
                  <button
                    on:click={() => moveEffectUp(selectedChannelForEffects, index)}
                    class="dark:text-gray-400 hover:dark:text-white text-xs"
                    disabled={index === 0}
                  >
                    ▲
                  </button>
                  <button
                    on:click={() => moveEffectDown(selectedChannelForEffects, index)}
                    class="dark:text-gray-400 hover:dark:text-white text-xs"
                    disabled={selectedChannelForEffects !== null &&
                      index === getChannelEffects(selectedChannelForEffects).length - 1}
                  >
                    ▼
                  </button>
                </div>

                <!-- Enable Toggle -->
                <button
                  on:click={() => toggleEffectEnabled(selectedChannelForEffects, effect.id)}
                  class="enable-btn w-8 h-8 rounded {effect.enabled
                    ? 'dark:bg-accent'
                    : 'dark:bg-gray-700'} flex items-center justify-center"
                  title={effect.enabled ? 'Disable' : 'Enable'}
                >
                  <span class="text-xs {effect.enabled ? 'text-white' : 'dark:text-gray-500'}">
                    {effect.enabled ? 'ON' : 'OFF'}
                  </span>
                </button>

                <!-- Effect Name -->
                <div class="effect-name flex-1">
                  <span class="dark:text-gray-200 font-medium">{effect.name}</span>
                </div>

                <!-- Wet/Dry Control -->
                <div class="wet-dry flex items-center space-x-2">
                  <span class="dark:text-gray-400 text-xs">Wet</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={effect.wet_dry}
                    on:input={(e) =>
                      updateEffectWetDry(
                        selectedChannelForEffects,
                        effect.id,
                        parseFloat(e.currentTarget.value)
                      )}
                    class="wet-dry-slider w-20"
                  />
                  <span class="dark:text-gray-300 text-xs w-8"
                    >{Math.round(effect.wet_dry * 100)}%</span
                  >
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Master Section -->
    <div class="master dark:bg-menu p-3 rounded mt-auto">
      <h3 class="dark:text-gray-200 mb-2">Master</h3>
      <div class="flex items-center space-x-4">
        <div class="volume-master flex flex-col items-center space-y-1">
          <label class="volume-label text-xs dark:text-gray-400">Master Vol</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={masterVolume}
            on:input={(e) => updateMasterVolume(parseFloat(e.currentTarget.value))}
            class="volume-slider dark:bg-input w-4 h-32"
            title="Master Volume"
          />
          <span class="volume-display text-xs dark:text-gray-300">{formatVolume(masterVolume)}</span
          >
        </div>
        <div class="vu-master flex space-x-1">
          <div
            class={getMeterColor(getMasterMeter(meterData, 'left'))}
            style="height: {getMasterMeter(
              meterData,
              'left'
            )}px; width: 2px; background: currentColor; rounded;"
          ></div>
          <div
            class={getMeterColor(getMasterMeter(meterData, 'right'))}
            style="height: {getMasterMeter(
              meterData,
              'right'
            )}px; width: 2px; background: currentColor; rounded;"
          ></div>
        </div>
      </div>
    </div>
  {:else if isLoading}
    <div
      class="no-mixer dark:text-gray-400 p-4 text-center flex flex-col items-center justify-center h-full"
    >
      <div class="text-2xl mb-2">🎚️</div>
      <div>Loading mixer...</div>
      <div class="text-xs mt-2">Connecting to audio system</div>
    </div>
  {:else}
    <div
      class="no-mixer dark:text-gray-400 p-4 text-center flex flex-col items-center justify-center h-full"
    >
      <div class="text-2xl mb-2">⚠️</div>
      <div>Mixer not available</div>
      {#if loadError}
        <div class="text-xs mt-2 text-yellow-500">{loadError}</div>
      {/if}
      <button
        class="mt-4 px-4 py-2 rounded text-sm dark:bg-primary dark:text-white hover:opacity-80"
        on:click={() => {
          isLoading = true;
          loadMixerState();
        }}
      >
        Retry
      </button>
    </div>
  {/if}
</div>

<style>
  .mixer-window {
    height: 100%;
  }

  .volume-slider {
    writing-mode: bt-lr;
    -webkit-appearance: slider-vertical;
    width: 8px;
    height: 128px;
  }

  .pan-slider {
    -webkit-appearance: slider-horizontal;
    width: 16px;
    height: 4px;
  }

  .channel-strip {
    min-width: 80px;
  }

  .vu-meters,
  .vu-master {
    height: 32px;
  }

  .vu-meter {
    transition: height 0.1s ease-out;
  }

  /* Effect Rack Styles */
  .effect-rack {
    max-height: 200px;
    overflow-y: auto;
  }

  .effect-slots-mini {
    font-size: 9px;
  }

  .effect-mini {
    max-width: 70px;
  }

  .fx-btn {
    font-size: 9px;
    font-weight: bold;
  }

  .wet-dry-slider {
    -webkit-appearance: none;
    height: 4px;
    border-radius: 2px;
    background: #4a5568;
  }

  .wet-dry-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #68d391;
    cursor: pointer;
  }

  .reorder-btns button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .enable-btn {
    transition: background-color 0.15s ease;
  }
</style>
