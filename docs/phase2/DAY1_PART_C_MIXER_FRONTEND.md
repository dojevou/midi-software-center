# Day 1, Part 1C: Frontend Mixer API

**Duration:** 2 hours
**Prerequisites:** Parts 1A and 1B complete
**Files to create:** 3

---

## Overview

Create TypeScript frontend layer for mixer:
1. TypeScript types matching Rust models
2. Mixer API client with all commands
3. Reactive Svelte stores for mixer state
4. Real-time state synchronization

---

## Step 1: TypeScript Types (20 min)

Create `app/src/lib/types/daw.ts`:

```typescript
/**
 * DAW Mixer TypeScript Types
 * Matches Rust models from app/src-tauri/src/daw/mixer/models/
 */

export interface TrackState {
  trackId: number;
  gainDb: number;        // -∞ to +12 dB
  pan: number;           // -1.0 (left) to +1.0 (right)
  muted: boolean;
  soloed: boolean;
  enabled: boolean;
  name: string;
  color?: string;
}

export interface MixerConfig {
  masterGainDb: number;
  masterMuted: boolean;
  trackStates: Record<number, TrackState>;
  soloActive: boolean;
}

export interface MixerState {
  config: MixerConfig;
  tracks: TrackState[];
  loading: boolean;
  error: string | null;
}

/**
 * Gain utilities (match Rust implementation)
 */
export class GainUtils {
  /**
   * Convert dB to linear gain
   * Formula: 10^(dB/20)
   */
  static dbToLinear(db: number): number {
    if (db <= -100.0) return 0.0;
    return Math.pow(10, db / 20.0);
  }

  /**
   * Convert linear gain to dB
   * Formula: 20 * log10(linear)
   */
  static linearToDb(linear: number): number {
    if (linear <= 0.0) return -100.0;
    return 20.0 * Math.log10(linear);
  }

  /**
   * Format gain for display
   * Examples: "0.0 dB", "-6.0 dB", "-∞ dB"
   */
  static formatDb(db: number): string {
    if (db <= -100.0) return '-∞ dB';
    return `${db.toFixed(1)} dB`;
  }

  /**
   * Format pan for display
   * Examples: "C", "L50", "R75"
   */
  static formatPan(pan: number): string {
    if (Math.abs(pan) < 0.01) return 'C';
    const percent = Math.round(Math.abs(pan) * 100);
    return pan < 0 ? `L${percent}` : `R${percent}`;
  }
}

/**
 * Pan utilities (match Rust equal-power panning)
 */
export class PanUtils {
  /**
   * Calculate left channel gain using equal-power panning
   * Pan: -1.0 (left) to +1.0 (right)
   */
  static leftGain(pan: number, gainLinear: number): number {
    const panAngle = (pan + 1.0) * 0.25 * Math.PI;
    return gainLinear * Math.cos(panAngle);
  }

  /**
   * Calculate right channel gain using equal-power panning
   */
  static rightGain(pan: number, gainLinear: number): number {
    const panAngle = (pan + 1.0) * 0.25 * Math.PI;
    return gainLinear * Math.sin(panAngle);
  }
}
```

---

## Step 2: Mixer API Client (30 min)

Create `app/src/lib/api/mixerApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { TrackState, MixerConfig } from '../types/daw';

/**
 * Mixer API Client
 * Wraps all Tauri mixer commands with TypeScript types
 */
export class MixerApi {
  // ============================================================================
  // TRACK MIXER COMMANDS
  // ============================================================================

  /**
   * Set track gain in decibels
   * @param trackId Track ID
   * @param gainDb Gain in dB (-∞ to +12)
   */
  static async setTrackGain(trackId: number, gainDb: number): Promise<void> {
    await invoke('set_track_gain', { trackId, gainDb });
  }

  /**
   * Set track pan (-1.0 to +1.0)
   * @param trackId Track ID
   * @param pan Pan value (-1.0 = left, 0.0 = center, +1.0 = right)
   */
  static async setTrackPan(trackId: number, pan: number): Promise<void> {
    await invoke('set_track_pan', { trackId, pan });
  }

  /**
   * Set track mute state
   */
  static async setTrackMute(trackId: number, muted: boolean): Promise<void> {
    await invoke('set_track_mute', { trackId, muted });
  }

  /**
   * Toggle track mute
   * @returns New mute state
   */
  static async toggleTrackMute(trackId: number): Promise<boolean> {
    return await invoke<boolean>('toggle_track_mute', { trackId });
  }

  /**
   * Set track solo state
   */
  static async setTrackSolo(trackId: number, soloed: boolean): Promise<void> {
    await invoke('set_track_solo', { trackId, soloed });
  }

  /**
   * Toggle track solo
   * @returns New solo state
   */
  static async toggleTrackSolo(trackId: number): Promise<boolean> {
    return await invoke<boolean>('toggle_track_solo', { trackId });
  }

  /**
   * Set track name
   */
  static async setTrackName(trackId: number, name: string): Promise<void> {
    await invoke('set_track_name', { trackId, name });
  }

  /**
   * Set track color (hex string like "#ff0000")
   */
  static async setTrackColor(trackId: number, color: string): Promise<void> {
    await invoke('set_track_color', { trackId, color });
  }

  /**
   * Get track state
   */
  static async getTrackState(trackId: number): Promise<TrackState> {
    return await invoke<TrackState>('get_track_state', { trackId });
  }

  // ============================================================================
  // MASTER MIXER COMMANDS
  // ============================================================================

  /**
   * Set master gain in decibels
   */
  static async setMasterGain(gainDb: number): Promise<void> {
    await invoke('set_master_gain', { gainDb });
  }

  /**
   * Set master mute state
   */
  static async setMasterMute(muted: boolean): Promise<void> {
    await invoke('set_master_mute', { muted });
  }

  /**
   * Toggle master mute
   * @returns New mute state
   */
  static async toggleMasterMute(): Promise<boolean> {
    return await invoke<boolean>('toggle_master_mute');
  }

  // ============================================================================
  // MIXER STATE COMMANDS
  // ============================================================================

  /**
   * Get complete mixer configuration
   */
  static async getMixerConfig(): Promise<MixerConfig> {
    return await invoke<MixerConfig>('get_mixer_config');
  }

  /**
   * Get all track states (sorted by track ID)
   */
  static async getAllTrackStates(): Promise<TrackState[]> {
    return await invoke<TrackState[]>('get_all_track_states');
  }

  /**
   * Reset all mixer settings to defaults
   */
  static async resetMixer(): Promise<void> {
    await invoke('reset_mixer');
  }

  // ============================================================================
  // BATCH OPERATIONS
  // ============================================================================

  /**
   * Set gain for multiple tracks at once
   */
  static async setTracksGain(trackIds: number[], gainDb: number): Promise<void> {
    await invoke('set_tracks_gain', { trackIds, gainDb });
  }

  /**
   * Mute all tracks
   */
  static async muteAllTracks(): Promise<void> {
    await invoke('mute_all_tracks');
  }

  /**
   * Unmute all tracks
   */
  static async unmuteAllTracks(): Promise<void> {
    await invoke('unmute_all_tracks');
  }

  /**
   * Clear all solos
   */
  static async clearAllSolos(): Promise<void> {
    await invoke('clear_all_solos');
  }
}
```

---

## Step 3: Mixer Store (50 min)

Create `app/src/lib/stores/mixerStore.ts`:

```typescript
import { writable, derived, get } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import { MixerApi } from '../api/mixerApi';
import type { MixerConfig, TrackState, MixerState } from '../types/daw';

/**
 * Mixer Store
 * Central state management for DAW mixer
 */
class MixerStore {
  private state: Writable<MixerState>;

  // Public readable stores
  public readonly config: Readable<MixerConfig>;
  public readonly tracks: Readable<TrackState[]>;
  public readonly loading: Readable<boolean>;
  public readonly error: Readable<string | null>;

  constructor() {
    // Initial state
    this.state = writable<MixerState>({
      config: {
        masterGainDb: 0.0,
        masterMuted: false,
        trackStates: {},
        soloActive: false,
      },
      tracks: [],
      loading: false,
      error: null,
    });

    // Derived stores
    this.config = derived(this.state, ($state) => $state.config);
    this.tracks = derived(this.state, ($state) => $state.tracks);
    this.loading = derived(this.state, ($state) => $state.loading);
    this.error = derived(this.state, ($state) => $state.error);
  }

  // ============================================================================
  // STATE LOADING
  // ============================================================================

  /**
   * Load mixer configuration from backend
   */
  async loadMixerConfig(): Promise<void> {
    this.state.update((s) => ({ ...s, loading: true, error: null }));

    try {
      const config = await MixerApi.getMixerConfig();
      const tracks = await MixerApi.getAllTrackStates();

      this.state.update((s) => ({
        ...s,
        config,
        tracks,
        loading: false,
      }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      console.error('Failed to load mixer config:', errorMessage);

      this.state.update((s) => ({
        ...s,
        loading: false,
        error: errorMessage,
      }));
    }
  }

  /**
   * Reload single track state
   */
  async reloadTrack(trackId: number): Promise<void> {
    try {
      const trackState = await MixerApi.getTrackState(trackId);

      this.state.update((s) => {
        const tracks = s.tracks.map((t) =>
          t.trackId === trackId ? trackState : t
        );
        return { ...s, tracks };
      });
    } catch (error) {
      console.error(`Failed to reload track ${trackId}:`, error);
    }
  }

  // ============================================================================
  // TRACK OPERATIONS
  // ============================================================================

  /**
   * Set track gain and update local state
   */
  async setTrackGain(trackId: number, gainDb: number): Promise<void> {
    try {
      await MixerApi.setTrackGain(trackId, gainDb);
      await this.reloadTrack(trackId);
    } catch (error) {
      console.error('Failed to set track gain:', error);
      throw error;
    }
  }

  /**
   * Set track pan and update local state
   */
  async setTrackPan(trackId: number, pan: number): Promise<void> {
    try {
      await MixerApi.setTrackPan(trackId, pan);
      await this.reloadTrack(trackId);
    } catch (error) {
      console.error('Failed to set track pan:', error);
      throw error;
    }
  }

  /**
   * Toggle track mute
   */
  async toggleTrackMute(trackId: number): Promise<void> {
    try {
      await MixerApi.toggleTrackMute(trackId);
      await this.reloadTrack(trackId);
    } catch (error) {
      console.error('Failed to toggle track mute:', error);
      throw error;
    }
  }

  /**
   * Toggle track solo
   */
  async toggleTrackSolo(trackId: number): Promise<void> {
    try {
      await MixerApi.toggleTrackSolo(trackId);
      await this.loadMixerConfig(); // Reload all to update solo_active
    } catch (error) {
      console.error('Failed to toggle track solo:', error);
      throw error;
    }
  }

  /**
   * Set track name
   */
  async setTrackName(trackId: number, name: string): Promise<void> {
    try {
      await MixerApi.setTrackName(trackId, name);
      await this.reloadTrack(trackId);
    } catch (error) {
      console.error('Failed to set track name:', error);
      throw error;
    }
  }

  /**
   * Set track color
   */
  async setTrackColor(trackId: number, color: string): Promise<void> {
    try {
      await MixerApi.setTrackColor(trackId, color);
      await this.reloadTrack(trackId);
    } catch (error) {
      console.error('Failed to set track color:', error);
      throw error;
    }
  }

  // ============================================================================
  // MASTER OPERATIONS
  // ============================================================================

  /**
   * Set master gain
   */
  async setMasterGain(gainDb: number): Promise<void> {
    try {
      await MixerApi.setMasterGain(gainDb);
      await this.loadMixerConfig();
    } catch (error) {
      console.error('Failed to set master gain:', error);
      throw error;
    }
  }

  /**
   * Toggle master mute
   */
  async toggleMasterMute(): Promise<void> {
    try {
      await MixerApi.toggleMasterMute();
      await this.loadMixerConfig();
    } catch (error) {
      console.error('Failed to toggle master mute:', error);
      throw error;
    }
  }

  // ============================================================================
  // BATCH OPERATIONS
  // ============================================================================

  /**
   * Reset entire mixer to defaults
   */
  async resetMixer(): Promise<void> {
    try {
      await MixerApi.resetMixer();
      await this.loadMixerConfig();
    } catch (error) {
      console.error('Failed to reset mixer:', error);
      throw error;
    }
  }

  /**
   * Mute all tracks
   */
  async muteAllTracks(): Promise<void> {
    try {
      await MixerApi.muteAllTracks();
      await this.loadMixerConfig();
    } catch (error) {
      console.error('Failed to mute all tracks:', error);
      throw error;
    }
  }

  /**
   * Unmute all tracks
   */
  async unmuteAllTracks(): Promise<void> {
    try {
      await MixerApi.unmuteAllTracks();
      await this.loadMixerConfig();
    } catch (error) {
      console.error('Failed to unmute all tracks:', error);
      throw error;
    }
  }

  /**
   * Clear all solos
   */
  async clearAllSolos(): Promise<void> {
    try {
      await MixerApi.clearAllSolos();
      await this.loadMixerConfig();
    } catch (error) {
      console.error('Failed to clear all solos:', error);
      throw error;
    }
  }

  // ============================================================================
  // HELPER METHODS
  // ============================================================================

  /**
   * Get track by ID from current state
   */
  getTrack(trackId: number): TrackState | undefined {
    const state = get(this.state);
    return state.tracks.find((t) => t.trackId === trackId);
  }

  /**
   * Subscribe to state changes
   */
  subscribe(callback: (state: MixerState) => void) {
    return this.state.subscribe(callback);
  }
}

// Export singleton instance
export const mixerStore = new MixerStore();
```

---

## Verification (20 min)

### 1. Type Check

```bash
cd app
npm run check
```

**Expected:** No TypeScript errors

### 2. Test API Client

Create test file `app/src/lib/api/__tests__/mixerApi.test.ts`:

```typescript
import { describe, it, expect, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { MixerApi } from '../mixerApi';

vi.mock('@tauri-apps/api/core');

describe('MixerApi', () => {
  it('should call set_track_gain with correct params', async () => {
    const mockInvoke = vi.mocked(invoke);

    await MixerApi.setTrackGain(1, -6.0);

    expect(mockInvoke).toHaveBeenCalledWith('set_track_gain', {
      trackId: 1,
      gainDb: -6.0,
    });
  });

  it('should return track state', async () => {
    const mockTrack = {
      trackId: 1,
      gainDb: 0.0,
      pan: 0.0,
      muted: false,
      soloed: false,
      enabled: true,
      name: 'Track 1',
    };

    vi.mocked(invoke).mockResolvedValue(mockTrack);

    const track = await MixerApi.getTrackState(1);

    expect(track).toEqual(mockTrack);
  });
});
```

Run tests:

```bash
npm test -- mixerApi
```

### 3. Test Store in Browser

```bash
make dev
```

In browser console:

```javascript
import { mixerStore } from './stores/mixerStore';
import { GainUtils } from './types/daw';

// Load mixer state
await mixerStore.loadMixerConfig();

// Subscribe to changes
mixerStore.tracks.subscribe((tracks) => {
  console.log('Tracks:', tracks);
});

// Set gain
await mixerStore.setTrackGain(1, -6.0);

// Test utilities
console.log(GainUtils.dbToLinear(-6.0));  // ~0.501
console.log(GainUtils.formatDb(-6.0));     // "-6.0 dB"
console.log(GainUtils.formatPan(0.5));     // "R50"
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| TypeScript errors | Run `npm run check` to see details |
| Store not updating | Check browser console for API errors |
| Commands not found | Verify Tauri commands registered in Part 1B |
| Types mismatch | Ensure Rust models match TypeScript interfaces |

---

## What's Next?

✅ **You've completed:**
- TypeScript types for mixer
- Mixer API client (17 commands wrapped)
- Reactive Svelte store with state management
- Gain/pan utility functions

**Next:** [Part 1D: Mixer UI Components](./DAY1_PART_D_MIXER_UI.md)
- MixerPanel component
- ChannelStrip component
- Fader and pan knob controls
- Visual feedback for mute/solo

**Note:** UI components will use the `mixerStore` created in this part for reactive state updates.
