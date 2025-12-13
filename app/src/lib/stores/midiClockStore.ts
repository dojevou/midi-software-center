import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Types
export type TransportState = 'Stopped' | 'Playing' | 'Recording' | 'Paused';
export type SyncMode = 'Internal' | 'External' | 'MidiTimecode';

export interface TimeSignature {
  numerator: number;
  denominator: number;
}

export interface Transport {
  state: TransportState;
  bpm: number;
  time_signature: TimeSignature;
  position_ticks: number;
  position_millis: number;
  bar: number;
  beat: number;
  tick: number;
  loop_enabled: boolean;
  loop_start_ticks: number;
  loop_end_ticks: number;
  metronome_enabled: boolean;
  count_in_enabled: boolean;
  count_in_bars: number;
}

export interface SyncStatus {
  mode: SyncMode;
  is_locked: boolean;
  external_bpm: number | null;
  drift_ms: number;
  clock_source: string | null;
  last_sync: number | null;
}

interface MidiClockState {
  transport: Transport;
  syncStatus: SyncStatus;
  isConnected: boolean;
  lastUpdate: number;
}

const defaultTransport: Transport = {
  state: 'Stopped',
  bpm: 120.0,
  time_signature: { numerator: 4, denominator: 4 },
  position_ticks: 0,
  position_millis: 0,
  bar: 1,
  beat: 1,
  tick: 0,
  loop_enabled: false,
  loop_start_ticks: 0,
  loop_end_ticks: 0,
  metronome_enabled: false,
  count_in_enabled: false,
  count_in_bars: 1,
};

const defaultSyncStatus: SyncStatus = {
  mode: 'Internal',
  is_locked: false,
  external_bpm: null,
  drift_ms: 0,
  clock_source: null,
  last_sync: null,
};

// Store
function createMidiClockStore() {
  const { subscribe, set, update } = writable<MidiClockState>({
    transport: defaultTransport,
    syncStatus: defaultSyncStatus,
    isConnected: false,
    lastUpdate: Date.now(),
  });

  let updateInterval: ReturnType<typeof setInterval> | null = null;
  let eventUnlisten: (() => void) | null = null;

  // Transport controls
  async function play(): Promise<void> {
    try {
      await invoke('transport_play');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'Playing' as TransportState },
      }));
    } catch (error) {
      console.error('Failed to play:', error);
      throw error;
    }
  }

  async function pause(): Promise<void> {
    try {
      await invoke('transport_pause');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'Paused' as TransportState },
      }));
    } catch (error) {
      console.error('Failed to pause:', error);
      throw error;
    }
  }

  async function stop(): Promise<void> {
    try {
      await invoke('transport_stop');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'Stopped' as TransportState, bar: 1, beat: 1, tick: 0 },
      }));
    } catch (error) {
      console.error('Failed to stop:', error);
      throw error;
    }
  }

  async function continuePlayback(): Promise<void> {
    try {
      await invoke('transport_continue');
      update(s => ({
        ...s,
        transport: { ...s.transport, state: 'Playing' as TransportState },
      }));
    } catch (error) {
      console.error('Failed to continue:', error);
      throw error;
    }
  }

  async function setBpm(bpm: number): Promise<void> {
    const clampedBpm = Math.max(20, Math.min(300, bpm));
    try {
      await invoke('clock_set_bpm', { bpm: clampedBpm });
      update(s => ({
        ...s,
        transport: { ...s.transport, bpm: clampedBpm },
      }));
    } catch (error) {
      console.error('Failed to set BPM:', error);
      throw error;
    }
  }

  async function setTimeSignature(numerator: number, denominator: number): Promise<void> {
    try {
      await invoke('clock_set_time_signature', { numerator, denominator });
      update(s => ({
        ...s,
        transport: {
          ...s.transport,
          time_signature: { numerator, denominator },
        },
      }));
    } catch (error) {
      console.error('Failed to set time signature:', error);
      throw error;
    }
  }

  async function setSyncMode(mode: string): Promise<void> {
    try {
      await invoke('set_sync_mode', { mode: mode.toLowerCase() });
      update(s => ({
        ...s,
        syncStatus: { ...s.syncStatus, mode: mode as SyncMode },
      }));
    } catch (error) {
      console.error('Failed to set sync mode:', error);
      throw error;
    }
  }

  async function setPosition(bar: number, beat: number): Promise<void> {
    try {
      await invoke('set_position_bars', { bar, beat });
      update(s => ({
        ...s,
        transport: { ...s.transport, bar, beat, tick: 0 },
      }));
    } catch (error) {
      console.error('Failed to set position:', error);
      throw error;
    }
  }

  async function fetchTransport(): Promise<void> {
    try {
      const transport = await invoke<Transport>('get_transport');
      update(s => ({
        ...s,
        transport,
        isConnected: true,
        lastUpdate: Date.now(),
      }));
    } catch (error) {
      update(s => ({ ...s, isConnected: false }));
    }
  }

  async function fetchSyncStatus(): Promise<void> {
    try {
      const syncStatus = await invoke<SyncStatus>('get_sync_status');
      update(s => ({ ...s, syncStatus }));
    } catch (error) {
      console.error('Failed to fetch sync status:', error);
    }
  }

  function startUpdates(intervalMs: number = 50): void {
    stopUpdates();

    // Initial fetch
    fetchTransport();
    fetchSyncStatus();

    // Regular updates (for position display)
    updateInterval = setInterval(() => {
      const state = get({ subscribe });
      if (state.transport.state === 'Playing' || state.transport.state === 'Recording') {
        fetchTransport();
      }
    }, intervalMs);
  }

  function stopUpdates(): void {
    if (updateInterval) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
  }

  async function setupEventListener(): Promise<void> {
    // Listen for transport events from backend
    eventUnlisten = await listen<Transport>('transport-update', (event) => {
      update(s => ({
        ...s,
        transport: event.payload,
        lastUpdate: Date.now(),
      }));
    });
  }

  function cleanup(): void {
    stopUpdates();
    if (eventUnlisten) {
      eventUnlisten();
      eventUnlisten = null;
    }
  }

  return {
    subscribe,
    play,
    pause,
    stop,
    continue: continuePlayback,
    setBpm,
    setTimeSignature,
    setSyncMode,
    setPosition,
    fetchTransport,
    fetchSyncStatus,
    startUpdates,
    stopUpdates,
    setupEventListener,
    cleanup,
  };
}

export const midiClockStore = createMidiClockStore();

// Derived stores
export const transport = derived(midiClockStore, ($store) => $store.transport);
export const transportState = derived(midiClockStore, ($store) => $store.transport.state);
export const bpm = derived(midiClockStore, ($store) => $store.transport.bpm);
export const timeSignature = derived(midiClockStore, ($store) => $store.transport.time_signature);
export const position = derived(midiClockStore, ($store) => ({
  bar: $store.transport.bar,
  beat: $store.transport.beat,
  tick: $store.transport.tick,
  millis: $store.transport.position_millis,
}));
export const syncStatus = derived(midiClockStore, ($store) => $store.syncStatus);
export const isPlaying = derived(midiClockStore, ($store) =>
  $store.transport.state === 'Playing' || $store.transport.state === 'Recording'
);
export const isSyncLocked = derived(midiClockStore, ($store) => $store.syncStatus.is_locked);
