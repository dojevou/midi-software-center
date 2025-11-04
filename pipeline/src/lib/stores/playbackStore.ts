/**
 * Playback Store - Transport controls and playback state management
 *
 * Grown-up Script: Manages DAW playback state with auto-updating position.
 * Handles play/pause/stop, tempo, time signature, and key signature changes.
 */

import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

export interface Position {
  bar: number;
  beat: number;
  tick: number;
  totalTicks: number;
}

export interface TimeSignature {
  numerator: number;
  denominator: number;
}

export interface PlaybackState {
  position: Position;
  isPlaying: boolean;
  isRecording: boolean;
  tempo: number;
  timeSignature: TimeSignature;
  keySignature: string;
  loop: {
    enabled: boolean;
    start: Position;
    end: Position;
  };
  metronome: {
    enabled: boolean;
    volume: number;
  };
  lastUpdateTime: number;
}

// ============================================================================
// Constants
// ============================================================================

const TICKS_PER_BEAT = 480; // Standard MIDI resolution
const UPDATE_INTERVAL_MS = 50; // 20fps position updates

const DEFAULT_POSITION: Position = {
  bar: 1,
  beat: 1,
  tick: 0,
  totalTicks: 0
};

const DEFAULT_TIME_SIGNATURE: TimeSignature = {
  numerator: 4,
  denominator: 4
};

const DEFAULT_STATE: PlaybackState = {
  position: DEFAULT_POSITION,
  isPlaying: false,
  isRecording: false,
  tempo: 120,
  timeSignature: DEFAULT_TIME_SIGNATURE,
  keySignature: 'C',
  loop: {
    enabled: false,
    start: DEFAULT_POSITION,
    end: { bar: 5, beat: 1, tick: 0, totalTicks: 4 * 4 * TICKS_PER_BEAT }
  },
  metronome: {
    enabled: false,
    volume: 0.8
  },
  lastUpdateTime: 0
};

// ============================================================================
// Store
// ============================================================================

const { subscribe, set, update } = writable<PlaybackState>(DEFAULT_STATE);

// Animation frame ID for position updates
let animationFrameId: number | null = null;
let lastFrameTime = 0;

// ============================================================================
// Position Calculations
// ============================================================================

function ticksToPosition(totalTicks: number, timeSignature: TimeSignature): Position {
  const ticksPerBeat = TICKS_PER_BEAT;
  const ticksPerBar = ticksPerBeat * timeSignature.numerator;

  const bar = Math.floor(totalTicks / ticksPerBar) + 1;
  const remainingTicks = totalTicks % ticksPerBar;
  const beat = Math.floor(remainingTicks / ticksPerBeat) + 1;
  const tick = remainingTicks % ticksPerBeat;

  return { bar, beat, tick, totalTicks };
}

function positionToTicks(position: Position): number {
  return position.totalTicks;
}

function calculateTicksElapsed(deltaMs: number, tempo: number): number {
  // Calculate ticks based on tempo and elapsed time
  // tempo is in BPM (beats per minute)
  const beatsPerSecond = tempo / 60;
  const ticksPerSecond = beatsPerSecond * TICKS_PER_BEAT;
  const ticksPerMs = ticksPerSecond / 1000;
  return Math.floor(ticksPerMs * deltaMs);
}

// ============================================================================
// Position Update Loop
// ============================================================================

function startPositionUpdates() {
  if (animationFrameId !== null) {
    return; // Already running
  }

  lastFrameTime = performance.now();

  const updateLoop = (currentTime: number) => {
    const state = get(playbackStore);

    if (!state.isPlaying) {
      animationFrameId = null;
      return;
    }

    const deltaMs = currentTime - lastFrameTime;

    if (deltaMs >= UPDATE_INTERVAL_MS) {
      const ticksElapsed = calculateTicksElapsed(deltaMs, state.tempo);

      update(s => {
        let newTotalTicks = s.position.totalTicks + ticksElapsed;

        // Handle looping
        if (s.loop.enabled) {
          const loopStart = positionToTicks(s.loop.start);
          const loopEnd = positionToTicks(s.loop.end);
          if (newTotalTicks >= loopEnd) {
            newTotalTicks = loopStart;
          }
        }

        const newPosition = ticksToPosition(newTotalTicks, s.timeSignature);

        return {
          ...s,
          position: newPosition,
          lastUpdateTime: currentTime
        };
      });

      lastFrameTime = currentTime;
    }

    animationFrameId = requestAnimationFrame(updateLoop);
  };

  animationFrameId = requestAnimationFrame(updateLoop);
}

function stopPositionUpdates() {
  if (animationFrameId !== null) {
    cancelAnimationFrame(animationFrameId);
    animationFrameId = null;
  }
}

// ============================================================================
// Actions
// ============================================================================

export const playbackActions = {
  /**
   * Start playback
   */
  async play(): Promise<void> {
    try {
      await invoke('start_playback');
      update(s => ({ ...s, isPlaying: true, lastUpdateTime: performance.now() }));
      startPositionUpdates();
    } catch (error) {
      console.error('Failed to start playback:', error);
      throw error;
    }
  },

  /**
   * Pause playback (keeps position)
   */
  async pause(): Promise<void> {
    try {
      await invoke('pause_playback');
      update(s => ({ ...s, isPlaying: false }));
      stopPositionUpdates();
    } catch (error) {
      console.error('Failed to pause playback:', error);
      throw error;
    }
  },

  /**
   * Stop playback (resets to start)
   */
  async stop(): Promise<void> {
    try {
      await invoke('stop_playback');
      update(s => ({
        ...s,
        isPlaying: false,
        isRecording: false,
        position: DEFAULT_POSITION
      }));
      stopPositionUpdates();
    } catch (error) {
      console.error('Failed to stop playback:', error);
      throw error;
    }
  },

  /**
   * Start recording
   */
  async record(): Promise<void> {
    try {
      await invoke('start_recording');
      update(s => ({ ...s, isRecording: true, isPlaying: true, lastUpdateTime: performance.now() }));
      startPositionUpdates();
    } catch (error) {
      console.error('Failed to start recording:', error);
      throw error;
    }
  },

  /**
   * Set playback position
   */
  async setPosition(bar: number, beat: number, tick: number): Promise<void> {
    const state = get(playbackStore);
    const totalTicks = ((bar - 1) * state.timeSignature.numerator * TICKS_PER_BEAT) +
                       ((beat - 1) * TICKS_PER_BEAT) +
                       tick;

    const position = ticksToPosition(totalTicks, state.timeSignature);

    try {
      await invoke('set_playback_position', { position: totalTicks });
      update(s => ({ ...s, position }));
    } catch (error) {
      console.error('Failed to set position:', error);
      throw error;
    }
  },

  /**
   * Set playback position by total ticks
   */
  async setPositionTicks(totalTicks: number): Promise<void> {
    const state = get(playbackStore);
    const position = ticksToPosition(totalTicks, state.timeSignature);

    try {
      await invoke('set_playback_position', { position: totalTicks });
      update(s => ({ ...s, position }));
    } catch (error) {
      console.error('Failed to set position:', error);
      throw error;
    }
  },

  /**
   * Set tempo (BPM)
   */
  async setTempo(tempo: number): Promise<void> {
    if (tempo < 20 || tempo > 300) {
      throw new Error('Tempo must be between 20 and 300 BPM');
    }

    try {
      await invoke('set_tempo', { tempo });
      update(s => ({ ...s, tempo }));
    } catch (error) {
      console.error('Failed to set tempo:', error);
      throw error;
    }
  },

  /**
   * Set time signature
   */
  async setTimeSignature(numerator: number, denominator: number): Promise<void> {
    const validDenominators = [2, 4, 8, 16];
    if (!validDenominators.includes(denominator)) {
      throw new Error('Denominator must be 2, 4, 8, or 16');
    }
    if (numerator < 1 || numerator > 16) {
      throw new Error('Numerator must be between 1 and 16');
    }

    const timeSignature: TimeSignature = { numerator, denominator };

    try {
      await invoke('set_time_signature', { numerator, denominator });
      update(s => {
        // Recalculate position with new time signature
        const position = ticksToPosition(s.position.totalTicks, timeSignature);
        return { ...s, timeSignature, position };
      });
    } catch (error) {
      console.error('Failed to set time signature:', error);
      throw error;
    }
  },

  /**
   * Set key signature
   */
  async setKeySignature(key: string): Promise<void> {
    const validKeys = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
    if (!validKeys.includes(key)) {
      throw new Error(`Invalid key signature: ${key}`);
    }

    try {
      await invoke('set_key_signature', { key });
      update(s => ({ ...s, keySignature: key }));
    } catch (error) {
      console.error('Failed to set key signature:', error);
      throw error;
    }
  },

  /**
   * Enable/disable loop
   */
  enableLoop(enabled: boolean): void {
    update(s => ({
      ...s,
      loop: { ...s.loop, enabled }
    }));
  },

  /**
   * Set loop region
   */
  setLoopRegion(start: Position, end: Position): void {
    update(s => ({
      ...s,
      loop: { ...s.loop, start, end }
    }));
  },

  /**
   * Enable/disable metronome
   */
  enableMetronome(enabled: boolean): void {
    update(s => ({
      ...s,
      metronome: { ...s.metronome, enabled }
    }));
  },

  /**
   * Set metronome volume
   */
  setMetronomeVolume(volume: number): void {
    if (volume < 0 || volume > 1) {
      throw new Error('Volume must be between 0 and 1');
    }
    update(s => ({
      ...s,
      metronome: { ...s.metronome, volume }
    }));
  },

  /**
   * Toggle play/pause
   */
  async togglePlayPause(): Promise<void> {
    const state = get(playbackStore);
    if (state.isPlaying) {
      await this.pause();
    } else {
      await this.play();
    }
  },

  /**
   * Reset to initial state
   */
  reset(): void {
    stopPositionUpdates();
    set(DEFAULT_STATE);
  }
};

// ============================================================================
// Derived Stores
// ============================================================================

export const positionFormatted = derived(
  playbackStore,
  $playback => `${$playback.position.bar}.${$playback.position.beat}.${$playback.position.tick}`
);

export const tempoDisplay = derived(
  playbackStore,
  $playback => `${$playback.tempo} BPM`
);

export const timeSignatureDisplay = derived(
  playbackStore,
  $playback => `${$playback.timeSignature.numerator}/${$playback.timeSignature.denominator}`
);

// ============================================================================
// Export
// ============================================================================

export const playbackStore = {
  subscribe,
  set,
  update
};
