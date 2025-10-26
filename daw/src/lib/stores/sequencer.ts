import { writable, derived, type Writable, type Readable } from 'svelte/store';
import type { Track, PlaybackPosition } from '../types';

/**
 * Sequencer tracks
 */
export const tracks: Writable<Track[]> = writable([]);

/**
 * Playback state
 */
export const isPlaying: Writable<boolean> = writable(false);

/**
 * Current tempo (BPM)
 */
export const tempo: Writable<number> = writable(120);

/**
 * Playback position
 */
export const playbackPosition: Writable<PlaybackPosition> = writable({
  current_tick: 0,
  current_bar: 0,
  current_beat: 0,
});

/**
 * Loop enabled
 */
export const isLooping: Writable<boolean> = writable(false);

/**
 * Metronome enabled
 */
export const metronomeEnabled: Writable<boolean> = writable(false);

/**
 * Helper: Check if sequencer has tracks
 */
export const hasTracks: Readable<boolean> = derived(
  tracks,
  $tracks => $tracks.length > 0
);

/**
 * Helper: Count active tracks (non-muted or soloed)
 */
export const activeTrackCount: Readable<number> = derived(
  tracks,
  $tracks => {
    const hasSolo = $tracks.some(t => t.solo);
    if (hasSolo) {
      return $tracks.filter(t => t.solo).length;
    }
    return $tracks.filter(t => !t.muted).length;
  }
);

/**
 * Reset sequencer state
 */
export function resetSequencer() {
  tracks.set([]);
  isPlaying.set(false);
  tempo.set(120);
  playbackPosition.set({
    current_tick: 0,
    current_bar: 0,
    current_beat: 0,
  });
}
