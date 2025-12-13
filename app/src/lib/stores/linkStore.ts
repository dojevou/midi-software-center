// Ableton Link store - sync with other DAWs and devices
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface LinkState {
  enabled: boolean;
  numPeers: number;
  tempo: number;
  beat: number;
  phase: number;
  isPlaying: boolean;
  quantum: number;
}

const initialState: LinkState = {
  enabled: false,
  numPeers: 0,
  tempo: 120,
  beat: 0,
  phase: 0,
  isPlaying: false,
  quantum: 4,
};

export const linkState = writable<LinkState>(initialState);
export const linkEnabled = derived(linkState, ($s) => $s.enabled);
export const linkPeers = derived(linkState, ($s) => $s.numPeers);

export const linkActions = {
  async enable(enabled: boolean) {
    await invoke('link_enable', { enabled });
    linkState.update((s) => ({ ...s, enabled }));
  },

  async setTempo(bpm: number) {
    await invoke('link_set_tempo', { bpm });
    linkState.update((s) => ({ ...s, tempo: bpm }));
  },

  async setQuantum(quantum: number) {
    await invoke('link_set_quantum', { quantum });
    linkState.update((s) => ({ ...s, quantum }));
  },

  async start() {
    await invoke('link_start');
    linkState.update((s) => ({ ...s, isPlaying: true }));
  },

  async stop() {
    await invoke('link_stop');
    linkState.update((s) => ({ ...s, isPlaying: false }));
  },

  async refresh() {
    const state = await invoke<LinkState>('link_get_state');
    linkState.set(state);
  },
};
