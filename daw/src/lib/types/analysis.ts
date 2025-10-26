/**
 * Analysis and compatibility type definitions
 *
 * Used for finding musically compatible MIDI files.
 */

import type { MidiFile } from './core';

/**
 * Compatible file match
 */
export interface CompatibleFile {
  file: MidiFile;
  compatibility_score: number;    // 0-100
  key_distance: number;           // Semitones
  bpm_difference: number;
  reason: string;
}

/**
 * Compatibility analysis request
 */
export interface CompatibilityRequest {
  fileId: number;
  maxResults: number;
}
