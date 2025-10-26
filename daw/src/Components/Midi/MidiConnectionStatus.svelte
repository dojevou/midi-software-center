<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface MidiDevice {
    name: string;
    manufacturer: string | null;
    is_connected: boolean;
  }

  let currentDevice: MidiDevice | null = null;
  let isConnected = false;
  let statusMessage = 'No MIDI device connected';
  let pollInterval: number;

  async function checkStatus() {
    try {
      isConnected = await invoke<boolean>('midi_is_connected');

      if (isConnected) {
        currentDevice = await invoke<MidiDevice>('midi_get_current_device');
        statusMessage = `Connected to ${currentDevice?.name || 'Unknown Device'}`;
      } else {
        currentDevice = null;
        statusMessage = 'No MIDI device connected';
      }
    } catch (error) {
      console.error('Failed to check MIDI status:', error);
      isConnected = false;
      currentDevice = null;
      statusMessage = 'MIDI status unavailable';
    }
  }

  onMount(() => {
    checkStatus();
    // Poll every 2 seconds
    pollInterval = setInterval(checkStatus, 2000) as unknown as number;
  });

  onDestroy(() => {
    if (pollInterval) {
      clearInterval(pollInterval);
    }
  });
</script>

<div class="midi-status" class:connected={isConnected} class:disconnected={!isConnected}>
  <div class="status-indicator"></div>
  <span class="status-text">{statusMessage}</span>
</div>

<style>
  .midi-status {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: #2d2d2d;
    border: 1px solid #3d3d3d;
    border-radius: 6px;
    font-size: 12px;
    transition: all 0.3s;
  }

  .status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #666;
    transition: all 0.3s;
  }

  .midi-status.connected {
    border-color: #4ade80;
  }

  .midi-status.connected .status-indicator {
    background: #4ade80;
    box-shadow: 0 0 8px rgba(74, 222, 128, 0.6);
    animation: pulse 2s infinite;
  }

  .midi-status.disconnected .status-indicator {
    background: #ef4444;
  }

  .status-text {
    color: #e0e0e0;
    font-weight: 500;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
