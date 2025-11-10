import { writable, derived, get } from 'svelte/store';
import { api } from '$lib/api';
import type { PlaybackPosition } from '$lib/types';

// ============================================================================
// PLAYBACK STATE
// ============================================================================

export interface PlaybackState {
  isPlaying: boolean;
  isPaused: boolean;
  tempo: number;
  timeSignature: [number, number];
  keySignature: string;
  position: PlaybackPosition;
  loopEnabled: boolean;
  loopStart: number;
  loopEnd: number;
  metronomeEnabled: boolean;
  metronomeVolume: number;
}

const initialState: PlaybackState = {
  isPlaying: false,
  isPaused: false,
  tempo: 120,
  timeSignature: [4, 4],
  keySignature: 'C',
  position: {
    current_tick: 0,
    current_bar: 0,
    current_beat: 0,
  },
  loopEnabled: false,
  loopStart: 0,
  loopEnd: 0,
  metronomeEnabled: false,
  metronomeVolume: 0.7,
};

export const playbackStore = writable<PlaybackState>(initialState);

// ============================================================================
// DERIVED STORES
// ============================================================================

export const isPlayingOrPaused = derived(
  playbackStore,
  ($playback) => $playback.isPlaying || $playback.isPaused
);

export const formattedPosition = derived(
  playbackStore,
  ($playback) => {
    const { current_bar, current_beat } = $playback.position;
    return `${current_bar + 1}:${current_beat + 1}`;
  }
);

// ============================================================================
// ACTIONS
// ============================================================================

export const playbackActions = {
  async play() {
    try {
      await api.sequencer.start();
      playbackStore.update(state => ({ ...state, isPlaying: true, isPaused: false }));
    } catch (error) {
      console.error('Failed to play:', error);
      throw error;
    }
  },

  async stop() {
    try {
      await api.sequencer.stop();
      playbackStore.update(state => ({
        ...state,
        isPlaying: false,
        isPaused: false,
        position: { current_tick: 0, current_bar: 0, current_beat: 0 }
      }));
    } catch (error) {
      console.error('Failed to stop:', error);
      throw error;
    }
  },

  async pause() {
    try {
      await api.sequencer.pause();
      playbackStore.update(state => ({ ...state, isPlaying: false, isPaused: true }));
    } catch (error) {
      console.error('Failed to pause:', error);
      throw error;
    }
  },

  async resume() {
    try {
      await api.sequencer.resume();
      playbackStore.update(state => ({ ...state, isPlaying: true, isPaused: false }));
    } catch (error) {
      console.error('Failed to resume:', error);
      throw error;
    }
  },

  async seek(bar: number, beat: number) {
    try {
      await api.sequencer.seekPosition(bar, beat);
      // Position update will come from backend event
    } catch (error) {
      console.error('Failed to seek:', error);
      throw error;
    }
  },

  async setTempo(bpm: number) {
    try {
      await api.sequencer.setTempo(bpm);
      playbackStore.update(state => ({ ...state, tempo: bpm }));
    } catch (error) {
      console.error('Failed to set tempo:', error);
      throw error;
    }
  },

  async setTimeSignature(numerator: number, denominator: number) {
    try {
      await api.window.setTimeSignature(numerator, denominator);
      playbackStore.update(state => ({ ...state, timeSignature: [numerator, denominator] }));
    } catch (error) {
      console.error('Failed to set time signature:', error);
      throw error;
    }
  },

  async setKeySignature(key: string) {
    try {
      await api.window.setKeySignature(key);
      playbackStore.update(state => ({ ...state, keySignature: key }));
    } catch (error) {
      console.error('Failed to set key signature:', error);
      throw error;
    }
  },

  async toggleLoop() {
    const currentState = get(playbackStore);
    console.log('Attempting to toggle loop:', !currentState.loopEnabled);
    try {
      await api.window.setLoopEnabled(!currentState.loopEnabled);
        console.log('Backend loop toggle succeeded');
    } catch (error: any) {
      console.warn('Backend set_loop_enabled failed - using local fallback:', error.message || error);
    }
    playbackStore.update(state => ({ ...state, loopEnabled: !state.loopEnabled }));
    console.log('Local loop state updated to:', !currentState.loopEnabled);
  },

  async setLoopRange(start: number, end: number) {
    console.log('Attempting to set loop range:', { start, end });
    try {
      await api.window.setLoopRange(start, end);
      console.log('Backend loop range set succeeded');
    } catch (error: any) {
      console.warn('Backend set_loop_range failed - using local fallback:', error.message || error);
    }
    playbackStore.update(state => ({ ...state, loopStart: start, loopEnd: end }));
    console.log('Local loop range updated');
  },

  async toggleMetronome() {
    const currentState = get(playbackStore);
    console.log('Attempting to toggle metronome:', !currentState.metronomeEnabled);
    try {
      await api.window.setMetronomeEnabled(!currentState.metronomeEnabled);
      console.log('Backend metronome toggle succeeded');
    } catch (error: any) {
      console.warn('Backend set_metronome_enabled failed - using local fallback:', error.message || error);
    }
    playbackStore.update(state => ({ ...state, metronomeEnabled: !state.metronomeEnabled }));
    console.log('Local metronome state updated to:', !currentState.metronomeEnabled);
  },

  async setMetronomeVolume(volume: number) {
    console.log('Attempting to set metronome volume:', volume);
    try {
      await api.window.setMetronomeVolume(volume);
      console.log('Backend metronome volume set succeeded');
    } catch (error: any) {
      console.warn('Backend set_metronome_volume failed - using local fallback:', error.message || error);
    }
    playbackStore.update(state => ({ ...state, metronomeVolume: volume }));
    console.log('Local metronome volume updated');
  },

  async getTransportInfo() {
    console.log('Attempting to get transport info');
    try {
      const info = await api.window.getTransportInfo();
      console.log('Backend transport info retrieved:', info);
      playbackStore.update(state => ({ ...state, ...info }));
      return info;
    } catch (error: any) {
      console.warn('Backend get_transport_info failed:', error.message || error);
      return null;
    }
  },

  updatePosition(position: PlaybackPosition) {
    playbackStore.update(state => ({ ...state, position }));
  },
};