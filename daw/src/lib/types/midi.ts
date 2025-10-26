/**
 * MIDI hardware and event type definitions
 *
 * Used for MIDI device communication and pattern playback.
 */

/**
 * MIDI device information
 */
export interface MidiDevice {
  name: string;
  manufacturer: string | null;
}

/**
 * MIDI connection status
 */
export type ConnectionStatus =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'error';

/**
 * MIDI event types
 */
export type MidiEventType =
  | 'NoteOn'
  | 'NoteOff'
  | 'ControlChange'
  | 'ProgramChange'
  | 'PitchBend'
  | 'Aftertouch';

/**
 * MIDI note event
 */
export interface MidiNote {
  pitch: number;        // 0-127
  velocity: number;     // 0-127
  start_tick: number;
  duration_ticks: number;
}

/**
 * MIDI pattern (for piano roll display)
 */
export interface MidiPattern {
  events: MidiEvent[];
  ticks_per_quarter_note: number;
  total_ticks: number;
}

/**
 * Generic MIDI event
 */
export interface MidiEvent {
  event_type: MidiEventType;
  tick: number;
  channel: number;
  note?: number;
  velocity?: number;
  controller?: number;
  value?: number;
  program?: number;
}
