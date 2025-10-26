<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

  interface MidiDevice {
    name: string;
    manufacturer: string | null;
  }

  let devices: MidiDevice[] = [];
  let selectedDevice: string | null = null;
  let isConnecting = false;
  let error = '';

  async function loadDevices() {
    try {
      devices = await invoke<MidiDevice[]>('midi_list_devices');

      // Auto-select UR22 if available
      const ur22 = devices.find(d => d.name.includes('UR22'));
      if (ur22) {
        selectedDevice = ur22.name;
      }
    } catch (err) {
      console.error('Failed to load MIDI devices:', err);
      error = `Failed to load devices: ${err}`;
    }
  }

  async function connectToDevice() {
    if (!selectedDevice) return;

    isConnecting = true;
    error = '';

    try {
      await invoke('midi_connect', { device_name: selectedDevice });
      dispatch('connected', { deviceName: selectedDevice });
    } catch (err) {
      console.error('Failed to connect:', err);
      error = `Connection failed: ${err}`;
    } finally {
      isConnecting = false;
    }
  }

  async function disconnect() {
    try {
      await invoke('midi_disconnect');
      selectedDevice = null;
      dispatch('disconnected');
    } catch (err) {
      console.error('Failed to disconnect:', err);
      error = `Disconnect failed: ${err}`;
    }
  }

  async function testConnection() {
    if (!selectedDevice) return;

    try {
      // Send middle C with velocity 100 on channel 0
      await invoke('midi_send_test_note', {
        channel: 0,
        note: 60,
        velocity: 100
      });
    } catch (err) {
      console.error('Test note failed:', err);
      error = `Test failed: ${err}`;
    }
  }

  onMount(() => {
    loadDevices();
  });
</script>

<div class="device-selector">
  <div class="selector-header">
    <h3>MIDI Device</h3>
    <button class="btn-refresh" on:click={loadDevices} title="Refresh devices">
      🔄
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  <select
    class="device-dropdown"
    bind:value={selectedDevice}
    disabled={isConnecting}
  >
    <option value={null}>Select a MIDI device...</option>
    {#each devices as device}
      <option value={device.name}>
        {device.name}
        {#if device.manufacturer}
          ({device.manufacturer})
        {/if}
      </option>
    {/each}
  </select>

  <div class="button-group">
    <button
      class="btn-primary"
      on:click={connectToDevice}
      disabled={!selectedDevice || isConnecting}
    >
      {isConnecting ? 'Connecting...' : 'Connect'}
    </button>

    <button
      class="btn-secondary"
      on:click={disconnect}
      disabled={!selectedDevice}
    >
      Disconnect
    </button>

    <button
      class="btn-test"
      on:click={testConnection}
      disabled={!selectedDevice}
      title="Send test note (Middle C)"
    >
      Test
    </button>
  </div>
</div>

<style>
  .device-selector {
    padding: 20px;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .selector-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .selector-header h3 {
    margin: 0;
    font-size: 16px;
    color: #e0e0e0;
  }

  .btn-refresh {
    width: 32px;
    height: 32px;
    background: transparent;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    font-size: 16px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-refresh:hover {
    background: #3d3d3d;
    transform: rotate(180deg);
  }

  .error-message {
    padding: 12px;
    background: #3d1f1f;
    border: 1px solid #5d2f2f;
    border-radius: 6px;
    color: #ff6b6b;
    font-size: 13px;
  }

  .device-dropdown {
    padding: 12px;
    background: #1e1e1e;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    color: #e0e0e0;
    font-size: 14px;
    cursor: pointer;
  }

  .device-dropdown:focus {
    outline: none;
    border-color: #4a9eff;
  }

  .device-dropdown:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .button-group {
    display: flex;
    gap: 8px;
  }

  .btn-primary,
  .btn-secondary,
  .btn-test {
    flex: 1;
    padding: 10px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #4a9eff;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #357abd;
  }

  .btn-secondary {
    background: #3d3d3d;
    color: #e0e0e0;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #4d4d4d;
  }

  .btn-test {
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    color: #e0e0e0;
  }

  .btn-test:hover:not(:disabled) {
    background: #3d3d3d;
    border-color: #4a9eff;
  }

  .btn-primary:disabled,
  .btn-secondary:disabled,
  .btn-test:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
