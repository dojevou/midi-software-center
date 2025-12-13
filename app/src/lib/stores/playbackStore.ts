import { derived, get, writable } from 'svelte/store';
import { isTauri, safeInvoke, safeListen } from '$lib/utils/tauri';
import { Commands } from '$lib/api/commands';
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

// Markup function for listeners
let unlisteners: (() => void)[] = [];

export const playbackActions = {
  async initListeners() {
    // Only set up listeners in Tauri context
    if (!isTauri()) {
      console.warn('[PlaybackStore] Not in Tauri context - listeners disabled');
      return;
    }

    // Listen for playback started
    const unlistenPlay = await safeListen<void>('daw::playback-started', () => {
      playbackStore.update((state) => ({ ...state, isPlaying: true, isPaused: false }));
    });
    if (unlistenPlay) {
      unlisteners.push(unlistenPlay);
    }

    // Listen for playback paused
    const unlistenPause = await safeListen<void>('daw::playback-paused', () => {
      playbackStore.update((state) => ({ ...state, isPlaying: false, isPaused: true }));
    });
    if (unlistenPause) {
      unlisteners.push(unlistenPause);
    }

    // Listen for playback stopped
    const unlistenStop = await safeListen<void>('daw::playback-stopped', () => {
      playbackStore.update((state) => ({
        ...state,
        isPlaying: false,
        isPaused: false,
        position: { current_tick: 0, current_bar: 0, current_beat: 0 },
      }));
    });
    if (unlistenStop) {
      unlisteners.push(unlistenStop);
    }

    // Listen for position updates
    const unlistenPosition = await safeListen<number>('daw::position-updated', (position) => {
      const state = get(playbackStore);
      const ticks_per_beat = 480;
      const beats_per_bar = state.timeSignature[0];
      const seconds_per_beat = 60.0 / state.tempo;
      const total_beats = position / seconds_per_beat;
      const current_bar = Math.floor(total_beats / beats_per_bar);
      const current_beat = Math.floor(total_beats % beats_per_bar);
      const beat_fraction = total_beats % 1;
      const current_tick = Math.floor(beat_fraction * ticks_per_beat);

      playbackStore.update((s) => ({
        ...s,
        position: {
          current_tick,
          current_bar,
          current_beat,
        },
      }));
    });
    if (unlistenPosition) {
      unlisteners.push(unlistenPosition);
    }

    // Listen for metronome clicks
    const unlistenMetronome = await safeListen<void>('daw::metronome-click', () => {
      // Frontend can play a sound or visual cue
      console.log('Metronome tick');
    });
    if (unlistenMetronome) {
      unlisteners.push(unlistenMetronome);
    }
  },

  cleanupListeners() {
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners = [];
  },

  async play() {
    try {
      await safeInvoke(Commands.START_SEQUENCER);
    } catch (error) {
      console.error('Failed to play:', error);
      throw error;
    }
  },

  async pause() {
    try {
      await safeInvoke(Commands.PAUSE_SEQUENCER);
    } catch (error) {
      console.error('Failed to pause:', error);
      throw error;
    }
  },

  async stop() {
    try {
      await safeInvoke(Commands.STOP_SEQUENCER);
    } catch (error) {
      console.error('Failed to stop:', error);
      throw error;
    }
  },

  async record() {
    try {
      await safeInvoke(Commands.START_RECORDING);
      playbackStore.update((state) => ({ ...state, isRecording: true }));
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw error;
    }
  },

  async setTempo(bpm: number) {
    try {
      await safeInvoke(Commands.SET_TEMPO, { bpm });
      playbackStore.update((state) => ({ ...state, tempo: bpm }));
    } catch (error) {
      console.error('Failed to set tempo:', error);
      throw error;
    }
  },

  async setTimeSignature(numerator: number, denominator: number) {
    try {
      await safeInvoke(Commands.SET_TIME_SIGNATURE, { numerator, denominator });
      playbackStore.update((state) => ({ ...state, timeSignature: [numerator, denominator] }));
    } catch (error) {
      console.error('Failed to set time signature:', error);
      throw error;
    }
  },

  async setKeySignature(key: string) {
    try {
      await safeInvoke(Commands.SET_KEY_SIGNATURE, { key });
      playbackStore.update((state) => ({ ...state, keySignature: key }));
    } catch (error) {
      console.error('Failed to set key signature:', error);
      throw error;
    }
  },

  async toggleLoop() {
    const currentState = get(playbackStore);
    try {
      await safeInvoke(Commands.DAW_SET_LOOP, {
        start: currentState.loopStart,
        end: currentState.loopEnd,
        enabled: !currentState.loopEnabled,
      });
      playbackStore.update((state) => ({ ...state, loopEnabled: !state.loopEnabled }));
    } catch (error) {
      console.error('Failed to toggle loop:', error);
      throw error;
    }
  },

  async setLoopRange(start: number, end: number) {
    try {
      await safeInvoke(Commands.DAW_SET_LOOP, { start, end, enabled: true });
      playbackStore.update((state) => ({ ...state, loopStart: start, loopEnd: end }));
    } catch (error) {
      console.error('Failed to set loop range:', error);
      throw error;
    }
  },

  async toggleMetronome() {
    const currentState = get(playbackStore);
    try {
      await safeInvoke(Commands.DAW_SET_METRONOME, { enabled: !currentState.metronomeEnabled });
      playbackStore.update((state) => ({ ...state, metronomeEnabled: !state.metronomeEnabled }));
    } catch (error) {
      console.error('Failed to toggle metronome:', error);
      throw error;
    }
  },

  async setMetronomeVolume(volume: number) {
    try {
      await safeInvoke(Commands.DAW_SET_METRONOME_VOLUME, { volume });
      playbackStore.update((state) => ({ ...state, metronomeVolume: volume }));
    } catch (error) {
      console.error('Failed to set metronome volume:', error);
      throw error;
    }
  },

  updatePosition(position: PlaybackPosition) {
    playbackStore.update((state) => ({ ...state, position }));
  },
};

// ============================================================================
// DERIVED STORES
// ============================================================================

export const formattedPosition = derived(playbackStore, ($playback) => {
  const { current_bar, current_beat } = $playback.position;
  return `${current_bar + 1}:${current_beat + 1}`;
});

export const isPlayingOrPaused = derived(
  playbackStore,
  ($playback) => $playback.isPlaying || $playback.isPaused
);
