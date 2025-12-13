import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { playbackStore, playbackActions } from '../app/src/lib/stores/playbackStore';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}));

const { invoke } = vi.mocked('@tauri-apps/api/tauri');

describe('playbackStore', () => {
  beforeEach(() => {
    // Reset store before each test
    playbackStore.set({
      isPlaying: false,
      isPaused: false,
      tempo: 120,
      timeSignature: [4, 4],
      keySignature: 'C',
      position: { current_tick: 0, current_bar: 0, current_beat: 0 },
      loopEnabled: false,
      loopStart: 0,
      loopEnd: 0,
      metronomeEnabled: false,
      metronomeVolume: 0.7,
    });
  });

  it('should initialize with default state', () => {
    const state = get(playbackStore);
    expect(state.isPlaying).toBe(false);
    expect(state.tempo).toBe(120);
    expect(state.position.current_tick).toBe(0);
  });

  it('should play and update state', async () => {
    await playbackActions.play();
    expect(invoke).toHaveBeenCalledWith('daw_play');
    const state = get(playbackStore);
    expect(state.isPlaying).toBe(true);
    expect(state.isPaused).toBe(false);
  });

  it('should pause and update state', async () => {
    await playbackActions.play();
    await playbackActions.pause();
    expect(invoke).toHaveBeenCalledWith('daw_pause');
    const state = get(playbackStore);
    expect(state.isPlaying).toBe(false);
    expect(state.isPaused).toBe(true);
  });

  it('should stop and reset position', async () => {
    await playbackActions.play();
    await playbackActions.stop();
    expect(invoke).toHaveBeenCalledWith('daw_stop');
    const state = get(playbackStore);
    expect(state.isPlaying).toBe(false);
    expect(state.isPaused).toBe(false);
    expect(state.position.current_tick).toBe(0);
  });

  it('should set tempo', async () => {
    await playbackActions.setTempo(140);
    expect(invoke).toHaveBeenCalledWith('daw_set_bpm', { bpm: 140 });
    const state = get(playbackStore);
    expect(state.tempo).toBe(140);
  });

  it('should toggle loop', async () => {
    await playbackActions.toggleLoop();
    expect(invoke).toHaveBeenCalledWith('daw_set_loop', {
      start: 0,
      end: 0,
      enabled: true
    });
    const state = get(playbackStore);
    expect(state.loopEnabled).toBe(true);
  });
});