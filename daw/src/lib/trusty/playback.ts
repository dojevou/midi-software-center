/**
 * Trusty Module: playback-calculations
 * Pure logic for playback position, timing, and note scheduling
 * Zero I/O, fully deterministic, comprehensive test coverage
 */

import { MidiNote } from './notes';

/**
 * Calculate the pixel position of the playhead on screen
 * @param beatPosition - Current playback position in beats
 * @param pixelsPerBeat - Number of pixels per beat on screen
 * @returns Pixel x-coordinate for the playhead
 */
export function calculatePlaybackPixelPosition(
  beatPosition: number,
  pixelsPerBeat: number
): number {
  if (beatPosition < 0) {
    throw new Error('Beat position cannot be negative');
  }
  if (pixelsPerBeat <= 0) {
    throw new Error('Pixels per beat must be positive');
  }

  return beatPosition * pixelsPerBeat;
}

/**
 * Convert BPM to milliseconds per beat
 * @param bpm - Tempo in beats per minute
 * @returns Milliseconds per beat
 */
export function bpmToMillisecondsPerBeat(bpm: number): number {
  if (bpm <= 0) {
    throw new Error('BPM must be positive');
  }

  // 60,000 ms per minute / BPM = ms per beat
  return 60000 / bpm;
}

/**
 * Convert milliseconds to beat count
 * @param milliseconds - Time in milliseconds
 * @param bpm - Tempo in beats per minute
 * @returns Number of beats
 */
export function millisecondsToBeats(milliseconds: number, bpm: number): number {
  if (milliseconds < 0) {
    throw new Error('Milliseconds cannot be negative');
  }
  if (bpm <= 0) {
    throw new Error('BPM must be positive');
  }

  const msPerBeat = bpmToMillisecondsPerBeat(bpm);
  return milliseconds / msPerBeat;
}

/**
 * Convert beat count to milliseconds
 * @param beats - Number of beats
 * @param bpm - Tempo in beats per minute
 * @returns Time in milliseconds
 */
export function beatsToMilliseconds(beats: number, bpm: number): number {
  if (beats < 0) {
    throw new Error('Beats cannot be negative');
  }
  if (bpm <= 0) {
    throw new Error('BPM must be positive');
  }

  const msPerBeat = bpmToMillisecondsPerBeat(bpm);
  return beats * msPerBeat;
}

/**
 * Calculate the absolute millisecond time to play a note from the start
 * @param note - MIDI note with startTime in beats
 * @param bpm - Tempo in beats per minute
 * @returns Absolute millisecond timestamp when note should play
 */
export function calculateNoteScheduleTime(note: MidiNote, bpm: number): number {
  if (bpm <= 0) {
    throw new Error('BPM must be positive');
  }

  return beatsToMilliseconds(note.startTime, bpm);
}

/**
 * Get all notes that should play within a lookahead window from current beat
 * Useful for scheduling notes in real-time playback
 * @param notes - Array of MIDI notes to check
 * @param currentBeat - Current playback position in beats
 * @param lookAheadBeats - Lookahead window in beats
 * @returns Array of notes that should play within [currentBeat, currentBeat + lookAheadBeats)
 */
export function calculateNotesAtTime(
  notes: MidiNote[],
  currentBeat: number,
  lookAheadBeats: number
): MidiNote[] {
  if (currentBeat < 0) {
    throw new Error('Current beat cannot be negative');
  }
  if (lookAheadBeats < 0) {
    throw new Error('Lookahead cannot be negative');
  }

  const windowEnd = currentBeat + lookAheadBeats;

  return notes.filter(
    (note) => note.startTime >= currentBeat && note.startTime < windowEnd
  );
}

/**
 * Calculate notes that are currently playing at a given beat position
 * @param notes - Array of MIDI notes
 * @param currentBeat - Current playback position in beats
 * @returns Array of notes that are sounding at currentBeat
 */
export function calculateActiveBeatNotes(notes: MidiNote[], currentBeat: number): MidiNote[] {
  if (currentBeat < 0) {
    throw new Error('Current beat cannot be negative');
  }

  return notes.filter((note) => {
    const noteEnd = note.startTime + note.duration;
    return note.startTime <= currentBeat && currentBeat < noteEnd;
  });
}

// ============================================================================
// UNIT TESTS
// ============================================================================

/**
 * Run all unit tests for playback functions
 * Returns test results with pass/fail status
 */
export function runTests(): { passed: number; failed: number; tests: string[] } {
  const results: string[] = [];
  let passed = 0;
  let failed = 0;

  const assert = (condition: boolean, message: string) => {
    if (condition) {
      results.push(`✓ ${message}`);
      passed++;
    } else {
      results.push(`✗ ${message}`);
      failed++;
    }
  };

  // ========== calculatePlaybackPixelPosition tests ==========
  results.push('\n--- calculatePlaybackPixelPosition Tests ---');

  assert(
    calculatePlaybackPixelPosition(0, 10) === 0,
    'calculatePlaybackPixelPosition: beat 0 = pixel 0'
  );

  assert(
    calculatePlaybackPixelPosition(5, 10) === 50,
    'calculatePlaybackPixelPosition: 5 beats @ 10px/beat = 50px'
  );

  assert(
    calculatePlaybackPixelPosition(2.5, 20) === 50,
    'calculatePlaybackPixelPosition: 2.5 beats @ 20px/beat = 50px'
  );

  assert(
    calculatePlaybackPixelPosition(10, 5) === 50,
    'calculatePlaybackPixelPosition: 10 beats @ 5px/beat = 50px'
  );

  try {
    calculatePlaybackPixelPosition(-1, 10);
    results.push('✗ calculatePlaybackPixelPosition: should reject negative beat position');
    failed++;
  } catch {
    results.push('✓ calculatePlaybackPixelPosition: rejects negative beat position');
    passed++;
  }

  try {
    calculatePlaybackPixelPosition(5, 0);
    results.push('✗ calculatePlaybackPixelPosition: should reject zero pixels per beat');
    failed++;
  } catch {
    results.push('✓ calculatePlaybackPixelPosition: rejects zero pixels per beat');
    passed++;
  }

  // ========== bpmToMillisecondsPerBeat tests ==========
  results.push('\n--- bpmToMillisecondsPerBeat Tests ---');

  assert(
    bpmToMillisecondsPerBeat(60) === 1000,
    'bpmToMillisecondsPerBeat: 60 BPM = 1000ms per beat'
  );

  assert(
    bpmToMillisecondsPerBeat(120) === 500,
    'bpmToMillisecondsPerBeat: 120 BPM = 500ms per beat'
  );

  assert(
    bpmToMillisecondsPerBeat(240) === 250,
    'bpmToMillisecondsPerBeat: 240 BPM = 250ms per beat'
  );

  assert(
    Math.abs(bpmToMillisecondsPerBeat(90) - 666.67) < 1,
    'bpmToMillisecondsPerBeat: 90 BPM ≈ 666.67ms per beat'
  );

  try {
    bpmToMillisecondsPerBeat(0);
    results.push('✗ bpmToMillisecondsPerBeat: should reject zero BPM');
    failed++;
  } catch {
    results.push('✓ bpmToMillisecondsPerBeat: rejects zero BPM');
    passed++;
  }

  try {
    bpmToMillisecondsPerBeat(-60);
    results.push('✗ bpmToMillisecondsPerBeat: should reject negative BPM');
    failed++;
  } catch {
    results.push('✓ bpmToMillisecondsPerBeat: rejects negative BPM');
    passed++;
  }

  // ========== millisecondsToBeats tests ==========
  results.push('\n--- millisecondsToBeats Tests ---');

  assert(
    millisecondsToBeats(0, 60) === 0,
    'millisecondsToBeats: 0ms @ 60 BPM = 0 beats'
  );

  assert(
    millisecondsToBeats(1000, 60) === 1,
    'millisecondsToBeats: 1000ms @ 60 BPM = 1 beat'
  );

  assert(
    millisecondsToBeats(500, 120) === 1,
    'millisecondsToBeats: 500ms @ 120 BPM = 1 beat'
  );

  assert(
    millisecondsToBeats(2000, 60) === 2,
    'millisecondsToBeats: 2000ms @ 60 BPM = 2 beats'
  );

  assert(
    millisecondsToBeats(500, 60) === 0.5,
    'millisecondsToBeats: 500ms @ 60 BPM = 0.5 beats'
  );

  try {
    millisecondsToBeats(-100, 60);
    results.push('✗ millisecondsToBeats: should reject negative milliseconds');
    failed++;
  } catch {
    results.push('✓ millisecondsToBeats: rejects negative milliseconds');
    passed++;
  }

  // ========== beatsToMilliseconds tests ==========
  results.push('\n--- beatsToMilliseconds Tests ---');

  assert(
    beatsToMilliseconds(0, 60) === 0,
    'beatsToMilliseconds: 0 beats @ 60 BPM = 0ms'
  );

  assert(
    beatsToMilliseconds(1, 60) === 1000,
    'beatsToMilliseconds: 1 beat @ 60 BPM = 1000ms'
  );

  assert(
    beatsToMilliseconds(1, 120) === 500,
    'beatsToMilliseconds: 1 beat @ 120 BPM = 500ms'
  );

  assert(
    beatsToMilliseconds(4, 60) === 4000,
    'beatsToMilliseconds: 4 beats @ 60 BPM = 4000ms'
  );

  assert(
    beatsToMilliseconds(0.5, 60) === 500,
    'beatsToMilliseconds: 0.5 beats @ 60 BPM = 500ms'
  );

  try {
    beatsToMilliseconds(-2, 60);
    results.push('✗ beatsToMilliseconds: should reject negative beats');
    failed++;
  } catch {
    results.push('✓ beatsToMilliseconds: rejects negative beats');
    passed++;
  }

  // ========== Roundtrip conversion tests ==========
  results.push('\n--- Roundtrip Conversion Tests ---');

  const testBeats = 4;
  const testBpm = 100;
  const ms = beatsToMilliseconds(testBeats, testBpm);
  const recoveredBeats = millisecondsToBeats(ms, testBpm);
  assert(
    Math.abs(recoveredBeats - testBeats) < 0.0001,
    `Roundtrip: beats -> ms -> beats preserves ${testBeats} beats @ ${testBpm} BPM`
  );

  // ========== calculateNoteScheduleTime tests ==========
  results.push('\n--- calculateNoteScheduleTime Tests ---');

  const mockNote1: MidiNote = {
    id: 'test1',
    pitch: 60,
    startTime: 0,
    duration: 1,
    velocity: 64,
    channel: 0
  };

  assert(
    calculateNoteScheduleTime(mockNote1, 60) === 0,
    'calculateNoteScheduleTime: note at beat 0 = 0ms'
  );

  const mockNote2: MidiNote = {
    id: 'test2',
    pitch: 64,
    startTime: 1,
    duration: 1,
    velocity: 64,
    channel: 0
  };

  assert(
    calculateNoteScheduleTime(mockNote2, 60) === 1000,
    'calculateNoteScheduleTime: note at beat 1 @ 60 BPM = 1000ms'
  );

  const mockNote3: MidiNote = {
    id: 'test3',
    pitch: 67,
    startTime: 2.5,
    duration: 0.5,
    velocity: 64,
    channel: 0
  };

  assert(
    calculateNoteScheduleTime(mockNote3, 120) === 1250,
    'calculateNoteScheduleTime: note at beat 2.5 @ 120 BPM = 1250ms'
  );

  try {
    calculateNoteScheduleTime(mockNote1, 0);
    results.push('✗ calculateNoteScheduleTime: should reject zero BPM');
    failed++;
  } catch {
    results.push('✓ calculateNoteScheduleTime: rejects zero BPM');
    passed++;
  }

  // ========== calculateNotesAtTime tests ==========
  results.push('\n--- calculateNotesAtTime Tests ---');

  const noteA: MidiNote = {
    id: 'a',
    pitch: 60,
    startTime: 0,
    duration: 2,
    velocity: 64,
    channel: 0
  };

  const noteB: MidiNote = {
    id: 'b',
    pitch: 64,
    startTime: 2,
    duration: 2,
    velocity: 64,
    channel: 0
  };

  const noteC: MidiNote = {
    id: 'c',
    pitch: 67,
    startTime: 4,
    duration: 1,
    velocity: 64,
    channel: 0
  };

  const testNotes = [noteA, noteB, noteC];

  const notesAt0 = calculateNotesAtTime(testNotes, 0, 1);
  assert(notesAt0.length === 1, 'calculateNotesAtTime: finds note at beat 0');
  assert(notesAt0[0].id === 'a', 'calculateNotesAtTime: correctly identifies note A');

  const notesAt2 = calculateNotesAtTime(testNotes, 2, 1);
  assert(notesAt2.length === 1, 'calculateNotesAtTime: finds note starting at beat 2');
  assert(notesAt2[0].id === 'b', 'calculateNotesAtTime: correctly identifies note B');

  const notesAt0to3 = calculateNotesAtTime(testNotes, 0, 3);
  assert(notesAt0to3.length === 2, 'calculateNotesAtTime: finds multiple notes in window');

  const notesAt5 = calculateNotesAtTime(testNotes, 5, 1);
  assert(notesAt5.length === 0, 'calculateNotesAtTime: finds no notes beyond range');

  try {
    calculateNotesAtTime(testNotes, -1, 1);
    results.push('✗ calculateNotesAtTime: should reject negative beat');
    failed++;
  } catch {
    results.push('✓ calculateNotesAtTime: rejects negative beat');
    passed++;
  }

  // ========== calculateActiveBeatNotes tests ==========
  results.push('\n--- calculateActiveBeatNotes Tests ---');

  const activeAt0 = calculateActiveBeatNotes(testNotes, 0);
  assert(activeAt0.length === 1, 'calculateActiveBeatNotes: finds note at beat 0');

  const activeAt1 = calculateActiveBeatNotes(testNotes, 1);
  assert(activeAt1.length === 1, 'calculateActiveBeatNotes: finds note playing at beat 1');
  assert(activeAt1[0].id === 'a', 'calculateActiveBeatNotes: correctly identifies note A');

  const activeAt2 = calculateActiveBeatNotes(testNotes, 2);
  assert(activeAt2.length === 2, 'calculateActiveBeatNotes: finds overlapping notes');

  const activeAt3 = calculateActiveBeatNotes(testNotes, 3);
  assert(activeAt3.length === 1, 'calculateActiveBeatNotes: finds note B at beat 3');

  const activeAt4 = calculateActiveBeatNotes(testNotes, 4);
  assert(activeAt4.length === 1, 'calculateActiveBeatNotes: finds note C at beat 4');

  const activeAt5 = calculateActiveBeatNotes(testNotes, 5);
  assert(activeAt5.length === 0, 'calculateActiveBeatNotes: finds no notes at beat 5');

  try {
    calculateActiveBeatNotes(testNotes, -0.5);
    results.push('✗ calculateActiveBeatNotes: should reject negative beat');
    failed++;
  } catch {
    results.push('✓ calculateActiveBeatNotes: rejects negative beat');
    passed++;
  }

  return { passed, failed, tests: results };
}

export const test = runTests;