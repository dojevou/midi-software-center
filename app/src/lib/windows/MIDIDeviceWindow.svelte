<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api } from '$lib/api';
  import { safeListen } from '$lib/utils/tauri';

  // Interfaces
  interface MIDIDevice {
    id: string;
    name: string;
    manufacturer: string;
    inputs: number;
    outputs: number;
  }

  interface MIDIDevices {
    inputs: MIDIDevice[];
    outputs: MIDIDevice[];
  }

  interface MIDIMessage {
    deviceId: string;
    type: string;
    channel?: number;
    note?: number;
    velocity?: number;
    controller?: number;
    value?: number;
    timestamp: string;
  }

  interface MIDIAction {
    id: string;
    name: string;
    category: string;
  }

  interface DeviceMapping {
    type: string;
    channel: number;
    note?: number;
    controller?: number;
    value?: number;
  }

  interface MessageType {
    id: string;
    name: string;
  }

  interface MIDIPreset {
    name: string;
    mappings: Record<string, DeviceMapping>;
  }

  // State
  let midiDevices: MIDIDevices = { inputs: [], outputs: [] };
  let connectionStatus: Record<string, boolean> = {};
  let midiMessages: MIDIMessage[] = [];
  let monitoringEnabled = true;
  let filterChannels: number[] = [];
  let filterMessages: string[] = [];
  let learnMode = false;
  let learningFor = '';
  let showLearnDialog = false;
  let deviceMappings: Record<string, DeviceMapping> = {};
  let presetName = '';
  let isLoading = false;

  let unlistenMidi: (() => void) | null = null;

  const messageTypes: MessageType[] = [
    { id: 'noteon', name: 'Note On' },
    { id: 'noteoff', name: 'Note Off' },
    { id: 'cc', name: 'CC' },
    { id: 'pitchbend', name: 'Pitch' },
    { id: 'aftertouch', name: 'AT' },
    { id: 'program', name: 'PC' },
  ];

  const midiActions: MIDIAction[] = [
    { id: 'play', name: 'Play/Pause', category: 'Transport' },
    { id: 'stop', name: 'Stop', category: 'Transport' },
    { id: 'record', name: 'Record', category: 'Transport' },
    { id: 'rewind', name: 'Rewind', category: 'Transport' },
    { id: 'forward', name: 'Forward', category: 'Transport' },
    { id: 'volume', name: 'Master Volume', category: 'Mixer' },
    { id: 'pan', name: 'Master Pan', category: 'Mixer' },
    { id: 'mute', name: 'Mute', category: 'Mixer' },
    { id: 'solo', name: 'Solo', category: 'Mixer' },
    { id: 'undo', name: 'Undo', category: 'Edit' },
    { id: 'redo', name: 'Redo', category: 'Edit' },
    { id: 'save', name: 'Save', category: 'File' },
  ];

  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

  onMount(async () => {
    await loadMIDIDevices();
    loadSavedMappings();

    // Listen for MIDI messages (only in Tauri context)
    unlistenMidi = await safeListen<MIDIMessage>('midi-message', (payload) => {
      if (monitoringEnabled) {
        handleMIDIMessage(payload);
      }
    });
  });

  onDestroy(() => {
    if (unlistenMidi) {
      unlistenMidi();
    }
  });

  async function loadMIDIDevices() {
    isLoading = true;
    try {
      midiDevices = await api.midi.getDevices();

      // Initialize connection status
      const status: Record<string, boolean> = {};
      [...midiDevices.inputs, ...midiDevices.outputs].forEach((device) => {
        status[device.id] = false;
      });
      connectionStatus = status;
    } catch (error) {
      console.error('Failed to load MIDI devices:', error);
    } finally {
      isLoading = false;
    }
  }

  async function connectDevice(deviceId: string, type: 'input' | 'output') {
    try {
      await api.midi.connectDevice(deviceId, type);
      connectionStatus = { ...connectionStatus, [deviceId]: true };
    } catch (error) {
      console.error('Failed to connect device:', error);
    }
  }

  async function disconnectDevice(deviceId: string, type: 'input' | 'output') {
    try {
      await api.midi.disconnectDevice(deviceId, type);
      connectionStatus = { ...connectionStatus, [deviceId]: false };
    } catch (error) {
      console.error('Failed to disconnect device:', error);
    }
  }

  async function testDevice(deviceId: string, type: 'input' | 'output') {
    try {
      await api.midi.testDevice(deviceId, type);
    } catch (error) {
      console.error('Failed to test device:', error);
    }
  }

  function handleMIDIMessage(message: MIDIMessage) {
    // Apply filters
    if (filterChannels.length > 0 && message.channel !== undefined) {
      if (!filterChannels.includes(message.channel)) {
        return;
      }
    }

    if (filterMessages.length > 0) {
      if (!filterMessages.includes(message.type)) {
        return;
      }
    }

    // Add to messages (keep last 100)
    midiMessages = [message, ...midiMessages.slice(0, 99)];

    // Check for MIDI learn
    if (learnMode && learningFor) {
      learnMapping(message);
    }
  }

  function formatMIDIMessage(message: MIDIMessage): string {
    switch (message.type) {
      case 'noteon':
        return `Note On: ${getNoteName(message.note || 0)} (vel: ${message.velocity})`;
      case 'noteoff':
        return `Note Off: ${getNoteName(message.note || 0)}`;
      case 'cc':
        return `CC ${message.controller}: ${message.value}`;
      case 'pitchbend':
        return `Pitch Bend: ${message.value}`;
      case 'aftertouch':
        return `Aftertouch: ${message.value}`;
      case 'program':
        return `Program Change: ${message.value}`;
      default:
        return `${message.type}: ${JSON.stringify(message)}`;
    }
  }

  function getNoteName(noteNumber: number): string {
    const octave = Math.floor(noteNumber / 12) - 1;
    const note = noteNames[noteNumber % 12];
    return `${note}${octave}`;
  }

  function clearMessages() {
    midiMessages = [];
  }

  function startLearn(actionId: string) {
    learnMode = true;
    learningFor = actionId;
    showLearnDialog = true;
  }

  function learnMapping(message: MIDIMessage | DeviceMapping) {
    if (!learningFor) {
      return;
    }

    const mapping: DeviceMapping = {
      type: message.type,
      channel: message.channel || 1,
      note: 'note' in message ? message.note : undefined,
      controller: 'controller' in message ? message.controller : undefined,
      value: 'value' in message ? message.value : undefined,
    };

    deviceMappings = { ...deviceMappings, [learningFor]: mapping };
    saveMappings();

    learnMode = false;
    learningFor = '';
    showLearnDialog = false;
  }

  function clearMappings() {
    deviceMappings = {};
    saveMappings();
  }

  function saveMappings() {
    localStorage.setItem('midiMappings', JSON.stringify(deviceMappings));
  }

  function loadSavedMappings() {
    const saved = localStorage.getItem('midiMappings');
    if (saved) {
      try {
        deviceMappings = JSON.parse(saved);
      } catch (e) {
        console.error('Failed to load saved mappings:', e);
      }
    }
  }

  function savePreset() {
    if (!presetName.trim()) {
      return;
    }

    const presets: MIDIPreset[] = JSON.parse(localStorage.getItem('midiPresets') || '[]');
    const newPreset: MIDIPreset = {
      name: presetName.trim(),
      mappings: { ...deviceMappings },
    };

    // Remove existing preset with same name
    const filtered = presets.filter((p) => p.name !== newPreset.name);
    filtered.push(newPreset);

    localStorage.setItem('midiPresets', JSON.stringify(filtered));
    presetName = '';
  }

  function loadPreset(preset: MIDIPreset) {
    deviceMappings = { ...preset.mappings };
    saveMappings();
  }

  function getSavedPresets(): MIDIPreset[] {
    try {
      return JSON.parse(localStorage.getItem('midiPresets') || '[]');
    } catch {
      return [];
    }
  }

  function openSystemDeviceManager() {
    // Show info about accessing system MIDI settings
    alert(
      'System MIDI Device Manager:\n\n' +
        'Linux: Open "ALSA MIDI Settings" or run "aconnect -l" in terminal\n' +
        'Windows: Control Panel → Hardware and Sound → Devices and Printers\n' +
        'macOS: Audio MIDI Setup app in Applications/Utilities\n\n' +
        'Use the Refresh Devices button above to reload device list after making changes.'
    );
  }

  // Reactive computed value for presets
  $: savedPresets = getSavedPresets();
</script>

<div class="midi-device-window dark:bg-window dark:text-app-text h-full flex">
  <!-- Left Panel: Device List -->
  <div class="device-panel w-80 border-r dark:border-window-border flex flex-col">
    <div class="panel-header p-4 border-b dark:border-window-border">
      <h2 class="text-xl dark:text-gray-200">MIDI Devices</h2>
    </div>

    <!-- Input Devices -->
    <div class="input-devices p-4 border-b dark:border-window-border flex-1 overflow-auto">
      <h3 class="text-lg dark:text-gray-300 mb-3">Input Devices</h3>
      <div class="space-y-3">
        {#each midiDevices.inputs as device (device.id)}
          <div
            class="device-item p-3 dark:bg-window-subtle rounded border dark:border-window-border"
          >
            <div class="flex justify-between items-center mb-2">
              <div>
                <div class="font-medium dark:text-gray-200">{device.name}</div>
                <div class="text-xs dark:text-gray-400">{device.manufacturer}</div>
              </div>

              <div class="flex items-center gap-2">
                <div
                  class="status-indicator w-3 h-3 rounded-full"
                  class:dark:bg-green-500={connectionStatus[device.id]}
                  class:dark:bg-red-500={!connectionStatus[device.id]}
                ></div>

                <div class="flex gap-1">
                  {#if connectionStatus[device.id]}
                    <button
                      on:click={() => disconnectDevice(device.id, 'input')}
                      class="px-2 py-1 text-xs dark:bg-error rounded hover:opacity-80"
                    >
                      Disconnect
                    </button>
                  {:else}
                    <button
                      on:click={() => connectDevice(device.id, 'input')}
                      class="px-2 py-1 text-xs dark:bg-primary rounded hover:opacity-80"
                    >
                      Connect
                    </button>
                  {/if}

                  <button
                    on:click={() => testDevice(device.id, 'input')}
                    class="px-2 py-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                  >
                    Test
                  </button>
                </div>
              </div>
            </div>

            <div class="text-xs dark:text-gray-500">
              {device.inputs || 1} input{device.inputs !== 1 ? 's' : ''} •
              {device.outputs || 0} output{device.outputs !== 1 ? 's' : ''}
            </div>
          </div>
        {/each}

        {#if midiDevices.inputs.length === 0}
          <div class="text-center py-6 dark:text-gray-500">
            <div class="text-4xl mb-2">🎹</div>
            <div>No MIDI input devices found</div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Output Devices -->
    <div class="output-devices p-4 border-b dark:border-window-border flex-1 overflow-auto">
      <h3 class="text-lg dark:text-gray-300 mb-3">Output Devices</h3>
      <div class="space-y-3">
        {#each midiDevices.outputs as device (device.id)}
          <div
            class="device-item p-3 dark:bg-window-subtle rounded border dark:border-window-border"
          >
            <div class="flex justify-between items-center mb-2">
              <div>
                <div class="font-medium dark:text-gray-200">{device.name}</div>
                <div class="text-xs dark:text-gray-400">{device.manufacturer}</div>
              </div>

              <div class="flex items-center gap-2">
                <div
                  class="status-indicator w-3 h-3 rounded-full"
                  class:dark:bg-green-500={connectionStatus[device.id]}
                  class:dark:bg-red-500={!connectionStatus[device.id]}
                ></div>

                <div class="flex gap-1">
                  {#if connectionStatus[device.id]}
                    <button
                      on:click={() => disconnectDevice(device.id, 'output')}
                      class="px-2 py-1 text-xs dark:bg-error rounded hover:opacity-80"
                    >
                      Disconnect
                    </button>
                  {:else}
                    <button
                      on:click={() => connectDevice(device.id, 'output')}
                      class="px-2 py-1 text-xs dark:bg-primary rounded hover:opacity-80"
                    >
                      Connect
                    </button>
                  {/if}

                  <button
                    on:click={() => testDevice(device.id, 'output')}
                    class="px-2 py-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                  >
                    Test
                  </button>
                </div>
              </div>
            </div>

            <div class="text-xs dark:text-gray-500">
              {device.inputs || 0} input{device.inputs !== 1 ? 's' : ''} •
              {device.outputs || 1} output{device.outputs !== 1 ? 's' : ''}
            </div>
          </div>
        {/each}

        {#if midiDevices.outputs.length === 0}
          <div class="text-center py-6 dark:text-gray-500">
            <div class="text-4xl mb-2">🔊</div>
            <div>No MIDI output devices found</div>
          </div>
        {/if}
      </div>
    </div>

    <!-- Device Actions -->
    <div class="device-actions p-4">
      <div class="flex gap-2">
        <button
          on:click={loadMIDIDevices}
          class="flex-1 px-3 py-2 dark:bg-secondary rounded hover:opacity-80"
        >
          Refresh Devices
        </button>
        <button
          on:click={openSystemDeviceManager}
          class="flex-1 px-3 py-2 dark:bg-secondary rounded hover:opacity-80"
        >
          Device Manager
        </button>
      </div>
    </div>
  </div>

  <!-- Right Panel: Monitoring & Mapping -->
  <div class="right-panel flex-1 flex flex-col overflow-hidden">
    <!-- Message Monitoring -->
    <div class="message-monitoring flex-1 flex flex-col">
      <div class="p-4 border-b dark:border-window-border">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-lg dark:text-gray-300">MIDI Monitor</h3>
          <div class="flex gap-2">
            <label class="flex items-center gap-2">
              <input type="checkbox" bind:checked={monitoringEnabled} class="rounded" />
              <span class="text-sm dark:text-gray-400">Enable Monitoring</span>
            </label>
            <button
              on:click={clearMessages}
              class="px-3 py-1 text-sm dark:bg-secondary rounded hover:opacity-80"
            >
              Clear
            </button>
          </div>
        </div>

        <!-- Message Filters -->
        <div class="filters mb-3">
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm dark:text-gray-400 mb-2">Filter Channels</label>
              <div class="flex flex-wrap gap-1">
                {#each Array.from({ length: 16 }, (_, i) => i + 1) as channel (channel)}
                  <button
                    on:click={() => {
                      if (filterChannels.includes(channel)) {
                        filterChannels = filterChannels.filter((c) => c !== channel);
                      } else {
                        filterChannels = [...filterChannels, channel];
                      }
                    }}
                    class="w-8 h-8 flex items-center justify-center rounded text-sm"
                    class:dark:bg-primary={filterChannels.includes(channel)}
                    class:dark:bg-secondary={!filterChannels.includes(channel)}
                  >
                    {channel}
                  </button>
                {/each}
                <button
                  on:click={() => (filterChannels = [])}
                  class="w-8 h-8 flex items-center justify-center dark:bg-secondary rounded text-sm"
                >
                  All
                </button>
              </div>
            </div>

            <div>
              <label class="block text-sm dark:text-gray-400 mb-2">Filter Messages</label>
              <div class="flex flex-wrap gap-1">
                {#each messageTypes as type (type.id)}
                  <button
                    on:click={() => {
                      if (filterMessages.includes(type.id)) {
                        filterMessages = filterMessages.filter((m) => m !== type.id);
                      } else {
                        filterMessages = [...filterMessages, type.id];
                      }
                    }}
                    class="px-2 py-1 text-xs rounded"
                    class:dark:bg-primary={filterMessages.includes(type.id)}
                    class:dark:bg-secondary={!filterMessages.includes(type.id)}
                  >
                    {type.name}
                  </button>
                {/each}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Message List -->
      <div class="message-list flex-1 overflow-auto p-4">
        <div class="space-y-2">
          {#each midiMessages as message, index (`${message.timestamp}-${index}`)}
            <div
              class="message-item p-3 dark:bg-window-subtle rounded border dark:border-window-border font-mono text-sm"
              class:dark:border-primary={index === 0}
            >
              <div class="flex justify-between items-center mb-1">
                <span class="dark:text-gray-300">{formatMIDIMessage(message)}</span>
                <span class="text-xs dark:text-gray-500">{message.timestamp}</span>
              </div>

              <div class="flex gap-4 text-xs dark:text-gray-400">
                <span>Device: {message.deviceId}</span>
                <span>Type: {message.type}</span>
                {#if message.channel !== undefined}
                  <span>Channel: {message.channel}</span>
                {/if}
              </div>
            </div>
          {:else}
            <div class="text-center py-12 dark:text-gray-500">
              <div class="text-4xl mb-4">📨</div>
              <div>No MIDI messages received</div>
              <div class="text-sm mt-2">Connect a device and start playing</div>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- MIDI Mapping -->
    <div class="midi-mapping border-t dark:border-window-border flex-1 flex flex-col">
      <div class="p-4 border-b dark:border-window-border">
        <div class="flex justify-between items-center">
          <h3 class="text-lg dark:text-gray-300">MIDI Learn</h3>
          <button
            on:click={clearMappings}
            class="px-3 py-1 text-sm dark:bg-error rounded hover:opacity-80"
          >
            Clear All Mappings
          </button>
        </div>
      </div>

      <div class="mapping-list flex-1 overflow-auto p-4">
        <div class="grid grid-cols-2 gap-4">
          {#each midiActions as action (action.id)}
            <div
              class="mapping-item p-3 dark:bg-window-subtle rounded border dark:border-window-border"
            >
              <div class="flex justify-between items-center mb-2">
                <span class="font-medium dark:text-gray-200">{action.name}</span>
                <button
                  on:click={() => startLearn(action.id)}
                  class="px-2 py-1 text-xs rounded"
                  class:dark:bg-primary={learnMode && learningFor === action.id}
                  class:dark:bg-secondary={!(learnMode && learningFor === action.id)}
                >
                  {deviceMappings[action.id] ? 'Re-learn' : 'Learn'}
                </button>
              </div>

              {#if deviceMappings[action.id]}
                <div class="text-xs dark:text-gray-400 space-y-1">
                  <div>Type: {deviceMappings[action.id].type}</div>
                  <div>Channel: {deviceMappings[action.id].channel}</div>
                  {#if deviceMappings[action.id].note !== undefined}
                    <div>
                      Note: {deviceMappings[action.id].note} ({getNoteName(
                        deviceMappings[action.id].note || 0
                      )})
                    </div>
                  {/if}
                  {#if deviceMappings[action.id].controller !== undefined}
                    <div>Controller: {deviceMappings[action.id].controller}</div>
                  {/if}
                </div>
              {:else}
                <div class="text-xs dark:text-gray-500 italic">No mapping assigned</div>
              {/if}
            </div>
          {/each}
        </div>
      </div>

      <!-- Preset Management -->
      <div class="preset-management p-4 border-t dark:border-window-border">
        <h4 class="text-sm dark:text-gray-400 mb-2">Save/Load Preset</h4>
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={presetName}
            placeholder="Preset name"
            class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
          />
          <button
            on:click={savePreset}
            disabled={!presetName.trim()}
            class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80 disabled:opacity-50"
          >
            Save
          </button>
        </div>

        <!-- Saved Presets -->
        <div class="saved-presets mt-3">
          {#if savedPresets.length > 0}
            <div class="text-xs dark:text-gray-400 mb-1">Saved Presets:</div>
            <div class="flex flex-wrap gap-1">
              {#each savedPresets as preset (preset.name)}
                <button
                  on:click={() => loadPreset(preset)}
                  class="px-2 py-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                >
                  {preset.name}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Learn Mode Dialog -->
{#if showLearnDialog}
  <div
    class="learn-dialog-overlay fixed inset-0 dark:bg-black dark:bg-opacity-50 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    aria-labelledby="learn-dialog-title"
  >
    <div class="learn-dialog dark:bg-window p-6 rounded-lg w-96">
      <h3 id="learn-dialog-title" class="text-xl dark:text-gray-200 mb-4">MIDI Learn</h3>

      <div class="text-center mb-6">
        <div class="text-6xl mb-4 animate-pulse">🎹</div>
        <div class="text-lg dark:text-gray-300 mb-2">Send a MIDI message</div>
        <div class="text-sm dark:text-gray-400">
          Press a key, turn a knob, or move a slider on your MIDI controller
        </div>
      </div>

      <div class="flex gap-3 justify-center">
        <button
          on:click={() => {
            learnMode = false;
            learningFor = '';
            showLearnDialog = false;
          }}
          class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
        >
          Cancel
        </button>
        <button
          on:click={() => {
            const commonMapping = {
              type: 'cc',
              channel: 1,
              controller: 7,
              value: 64,
            };
            learnMapping(commonMapping);
          }}
          class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
        >
          Use Default
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .status-indicator {
    transition: background-color 0.3s;
  }

  .message-item:first-child {
    animation: highlight 1s ease-out;
  }

  @keyframes highlight {
    0% {
      background-color: rgba(59, 130, 246, 0.3);
    }
    100% {
      background-color: transparent;
    }
  }

  .learn-dialog-overlay {
    backdrop-filter: blur(4px);
  }
</style>
