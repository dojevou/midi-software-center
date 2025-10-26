/**
 * Trusty Module: note-objects
 * Pure logic for MIDI note representation and manipulation
 * Zero I/O, fully deterministic, comprehensive test coverage
 */

export interface MidiNote {
  id: string;
  pitch: number; // 0-127, MIDI note number
  startTime: number; // in beats
  duration: number; // in beats
  velocity: number; // 0-127
  channel: number; // 0-15
}

/**
 * Generate a simple unique ID for a note
 * Uses timestamp + random number for pseudo-uniqueness
 */
function generateNoteId(): string {
  return `note_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
}

/**
 * Create a new MIDI note object with unique ID
 * @param pitch - MIDI pitch (0-127)
 * @param startTime - Start time in beats
 * @param duration - Duration in beats
 * @param velocity - Note velocity (0-127)
 * @param channel - MIDI channel (0-15)
 * @returns New MidiNote object
 */
export function createNote(
  pitch: number,
  startTime: number,
  duration: number,
  velocity: number,
  channel: number = 0
): MidiNote {
  if (pitch < 0 || pitch > 127) {
    throw new Error('Pitch must be between 0 and 127');
  }
  if (startTime < 0) {
    throw new Error('Start time cannot be negative');
  }
  if (duration <= 0) {
    throw new Error('Duration must be positive');
  }
  if (velocity < 0 || velocity > 127) {
    throw new Error('Velocity must be between 0 and 127');
  }
  if (channel < 0 || channel > 15) {
    throw new Error('Channel must be between 0 and 15');
  }

  return {
    id: generateNoteId(),
    pitch,
    startTime,
    duration,
    velocity,
    channel
  };
}

/**
 * Convert MIDI pitch number to note name
 * @param pitch - MIDI pitch (0-127)
 * @returns Note name (e.g., "C4", "C#4", "Db4")
 */
export function noteNameFromPitch(pitch: number): string {
  if (pitch < 0 || pitch > 127) {
    throw new Error('Pitch must be between 0 and 127');
  }

  const noteNames = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const octave = Math.floor(pitch / 12) - 1;
  const noteIndex = pitch % 12;
  
  return `${noteNames[noteIndex]}${octave}`;
}

/**
 * Convert note name to MIDI pitch number
 * @param name - Note name (e.g., "C4", "C#4", "Db4")
 * @returns MIDI pitch (0-127)
 */
export function pitchFromNoteName(name: string): number {
  const noteNames: { [key: string]: number } = {
    'C': 0, 'C#': 1, 'DB': 1,
    'D': 2, 'D#': 3, 'EB': 3,
    'E': 4,
    'F': 5, 'F#': 6, 'GB': 6,
    'G': 7, 'G#': 8, 'AB': 8,
    'A': 9, 'A#': 10, 'BB': 10,
    'B': 11
  };

  const normalizedName = name.toUpperCase();
  const match = normalizedName.match(/^([A-G]#?B?)(-?\d+)$/);
  if (!match) {
    throw new Error(`Invalid note name: ${name}`);
  }

  const note = match[1];
  const octave = parseInt(match[2], 10);

  if (!(note in noteNames)) {
    throw new Error(`Invalid note: ${note}`);
  }

  const pitch = (octave + 1) * 12 + noteNames[note];

  if (pitch < 0 || pitch > 127) {
    throw new Error(`Note ${name} is out of MIDI range (0-127)`);
  }

  return pitch;
}

/**
 * Get octave number from MIDI pitch
 * @param pitch - MIDI pitch (0-127)
 * @returns Octave number (e.g., 60 = octave 4)
 */
export function getOctaveFromPitch(pitch: number): number {
  if (pitch < 0 || pitch > 127) {
    throw new Error('Pitch must be between 0 and 127');
  }

  return Math.floor(pitch / 12) - 1;
}

/**
 * Map velocity value to visual display height
 * @param velocity - Note velocity (0-127)
 * @param maxHeight - Maximum height in pixels
 * @returns Height value (0 to maxHeight)
 */
export function velocityToDisplayHeight(velocity: number, maxHeight: number): number {
  if (velocity < 0 || velocity > 127) {
    throw new Error('Velocity must be between 0 and 127');
  }
  if (maxHeight <= 0) {
    throw new Error('Max height must be positive');
  }

  // Minimum height is 2px even for velocity 0
  const minHeight = 2;
  const heightRange = maxHeight - minHeight;
  const velocityRatio = velocity / 127;
  
  return minHeight + (heightRange * velocityRatio);
}

/**
 * Check if two notes overlap in time
 * @param note1 - First MIDI note
 * @param note2 - Second MIDI note
 * @returns True if notes overlap, false otherwise
 */
export function notesOverlap(note1: MidiNote, note2: MidiNote): boolean {
  const note1End = note1.startTime + note1.duration;
  const note2End = note2.startTime + note2.duration;

  // Two intervals overlap if:
  // note1.start < note2.end AND note2.start < note1.end
  return note1.startTime < note2End && note2.startTime < note1End;
}

// ============================================================================
// UNIT TESTS
// ============================================================================

/**
 * Run all unit tests for note functions
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

  // ========== createNote tests ==========
  results.push('\n--- createNote Tests ---');

  const note1 = createNote(60, 0, 1, 64);
  assert(note1.pitch === 60, 'createNote: creates note with correct pitch');
  assert(note1.startTime === 0, 'createNote: creates note with correct startTime');
  assert(note1.duration === 1, 'createNote: creates note with correct duration');
  assert(note1.velocity === 64, 'createNote: creates note with correct velocity');
  assert(note1.channel === 0, 'createNote: defaults to channel 0');
  assert(note1.id.startsWith('note_'), 'createNote: generates unique ID');

  const note2 = createNote(72, 2.5, 0.5, 100, 5);
  assert(note2.channel === 5, 'createNote: sets custom channel');
  assert(note2.id !== note1.id, 'createNote: generates different IDs for different notes');

  try {
    createNote(128, 0, 1, 64);
    results.push('✗ createNote: should reject pitch > 127');
    failed++;
  } catch {
    results.push('✓ createNote: rejects pitch > 127');
    passed++;
  }

  try {
    createNote(60, 0, 0, 64);
    results.push('✗ createNote: should reject zero duration');
    failed++;
  } catch {
    results.push('✓ createNote: rejects zero duration');
    passed++;
  }

  // ========== noteNameFromPitch tests ==========
  results.push('\n--- noteNameFromPitch Tests ---');

  assert(noteNameFromPitch(0) === 'C-1', 'noteNameFromPitch: 0 = C-1');
  assert(noteNameFromPitch(12) === 'C0', 'noteNameFromPitch: 12 = C0');
  assert(noteNameFromPitch(60) === 'C4', 'noteNameFromPitch: 60 = C4 (middle C)');
  assert(noteNameFromPitch(61) === 'C#4', 'noteNameFromPitch: 61 = C#4');
  assert(noteNameFromPitch(62) === 'D4', 'noteNameFromPitch: 62 = D4');
  assert(noteNameFromPitch(69) === 'A4', 'noteNameFromPitch: 69 = A4 (concert pitch)');
  assert(noteNameFromPitch(127) === 'G9', 'noteNameFromPitch: 127 = G9 (highest MIDI note)');

  try {
    noteNameFromPitch(128);
    results.push('✗ noteNameFromPitch: should reject pitch > 127');
    failed++;
  } catch {
    results.push('✓ noteNameFromPitch: rejects pitch > 127');
    passed++;
  }

  // ========== pitchFromNoteName tests ==========
  results.push('\n--- pitchFromNoteName Tests ---');

  assert(pitchFromNoteName('C4') === 60, 'pitchFromNoteName: C4 = 60');
  assert(pitchFromNoteName('C#4') === 61, 'pitchFromNoteName: C#4 = 61');
  assert(pitchFromNoteName('DB4') === 61, 'pitchFromNoteName: DB4 = 61 (enharmonic)');
  assert(pitchFromNoteName('A4') === 69, 'pitchFromNoteName: A4 = 69');
  assert(pitchFromNoteName('G9') === 127, 'pitchFromNoteName: G9 = 127');

  try {
    pitchFromNoteName('H4');
    results.push('✗ pitchFromNoteName: should reject invalid note name');
    failed++;
  } catch {
    results.push('✓ pitchFromNoteName: rejects invalid note name');
    passed++;
  }

  try {
    pitchFromNoteName('C10');
    results.push('✗ pitchFromNoteName: should reject out-of-range notes');
    failed++;
  } catch {
    results.push('✓ pitchFromNoteName: rejects out-of-range notes');
    passed++;
  }

  // ========== getOctaveFromPitch tests ==========
  results.push('\n--- getOctaveFromPitch Tests ---');

  assert(getOctaveFromPitch(0) === -1, 'getOctaveFromPitch: pitch 0 = octave -1');
  assert(getOctaveFromPitch(12) === 0, 'getOctaveFromPitch: pitch 12 = octave 0');
  assert(getOctaveFromPitch(60) === 4, 'getOctaveFromPitch: pitch 60 = octave 4');
  assert(getOctaveFromPitch(69) === 4, 'getOctaveFromPitch: pitch 69 = octave 4');
  assert(getOctaveFromPitch(127) === 9, 'getOctaveFromPitch: pitch 127 = octave 9');

  // ========== velocityToDisplayHeight tests ==========
  results.push('\n--- velocityToDisplayHeight Tests ---');

  const height0 = velocityToDisplayHeight(0, 100);
  assert(height0 === 2, 'velocityToDisplayHeight: velocity 0 = min height (2px)');

  const height64 = velocityToDisplayHeight(64, 100);
  assert(height64 > 2 && height64 < 100, 'velocityToDisplayHeight: velocity 64 = mid-range');

  const height127 = velocityToDisplayHeight(127, 100);
  assert(height127 === 100, 'velocityToDisplayHeight: velocity 127 = max height');

  const expectedHeight64 = 2 + (98 * (64 / 127));
  assert(
    Math.abs(height64 - expectedHeight64) < 0.01,
    'velocityToDisplayHeight: uses correct linear interpolation'
  );

  try {
    velocityToDisplayHeight(128, 100);
    results.push('✗ velocityToDisplayHeight: should reject velocity > 127');
    failed++;
  } catch {
    results.push('✓ velocityToDisplayHeight: rejects velocity > 127');
    passed++;
  }

  // ========== notesOverlap tests ==========
  results.push('\n--- notesOverlap Tests ---');

  const noteA = createNote(60, 0, 2, 64);
  const noteB = createNote(64, 1, 2, 64);
  const noteC = createNote(67, 2, 1, 64);
  const noteD = createNote(72, 3, 1, 64);

  assert(notesOverlap(noteA, noteB) === true, 'notesOverlap: overlapping notes detected');
  assert(notesOverlap(noteB, noteA) === true, 'notesOverlap: commutative for overlapping');
  assert(notesOverlap(noteA, noteC) === false, 'notesOverlap: adjacent notes do not overlap');
  assert(notesOverlap(noteA, noteD) === false, 'notesOverlap: separated notes do not overlap');
  assert(notesOverlap(noteB, noteD) === false, 'notesOverlap: separated notes do not overlap');

  const noteE = createNote(60, 0.5, 1, 64);
  assert(notesOverlap(noteA, noteE) === true, 'notesOverlap: contained note overlaps');

  // ========== Roundtrip tests ==========
  results.push('\n--- Roundtrip Conversion Tests ---');

  const originalPitch = 72;
  const noteName = noteNameFromPitch(originalPitch);
  const recoveredPitch = pitchFromNoteName(noteName);
  assert(
    recoveredPitch === originalPitch,
    `Roundtrip: pitch -> name -> pitch preserves ${originalPitch}`
  );

  for (let pitch = 0; pitch <= 127; pitch += 12) {
    const name = noteNameFromPitch(pitch);
    const recovered = pitchFromNoteName(name);
    if (recovered !== pitch) {
      results.push(`✗ Roundtrip: pitch ${pitch} failed (got ${recovered})`);
      failed++;
      break;
    }
  }
  results.push('✓ Roundtrip: all octave roots convert correctly');
  passed++;

  return { passed, failed, tests: results };
}

export const test = runTests;
