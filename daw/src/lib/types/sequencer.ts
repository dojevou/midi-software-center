/**
 * Sequencer type definitions
 *
 * Used for multi-track sequencing and playback control.
 */

/**
 * Sequencer track
 */
export interface Track {
  id: number;
  name: string;
  file_id: number;
  channel: number;
  muted: boolean;
  solo: boolean;
  volume: number;      // 0-127
  pan: number;         // 0-127 (64 = center)
  color: string;       // Hex color
}

/**
 * Track properties for updates
 */
export interface TrackProperties {
  muted?: boolean;
  solo?: boolean;
  volume?: number;
  pan?: number;
}

/**
 * Playback position
 */
export interface PlaybackPosition {
  current_tick: number;
  current_bar: number;
  current_beat: number;
}

/**
 * Sequencer state
 */
export interface SequencerState {
  isPlaying: boolean;
  tempo: number;
  currentPosition: number;
  totalDuration: number;
  tracks: Track[];
  isLooping: boolean;
  metronomeEnabled: boolean;
}
