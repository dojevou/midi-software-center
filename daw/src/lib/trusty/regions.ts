/**
 * Trusty Module: region-management
 * Pure logic for MIDI region representation and manipulation
 * Zero I/O, fully deterministic, comprehensive test coverage
 */

export interface Region {
  id: string;
  trackId: string;
  midiFileId?: number;
  startBeat: number;
  durationBeats: number;
  name: string;
  color?: string;
}

/**
 * Generate a simple unique ID for a region
 * Uses timestamp + random number for pseudo-uniqueness
 */
function generateRegionId(): string {
  return `region_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
}

/**
 * Create a new region with default values
 * @param trackId - ID of the track this region belongs to
 * @param startBeat - Start position in beats
 * @param durationBeats - Duration in beats
 * @returns New Region object with sensible defaults
 */
export function createRegion(
  trackId: string,
  startBeat: number,
  durationBeats: number
): Region {
  if (!trackId || trackId.length === 0) {
    throw new Error('Track ID cannot be empty');
  }
  if (startBeat < 0) {
    throw new Error('Start beat cannot be negative');
  }
  if (durationBeats <= 0) {
    throw new Error('Duration must be positive');
  }

  return {
    id: generateRegionId(),
    trackId,
    startBeat,
    durationBeats,
    name: `Region ${startBeat}`, // Default name
    color: undefined // Optional color
  };
}

/**
 * Calculate the end beat position of a region
 * @param region - Region to calculate end beat for
 * @returns End beat position (startBeat + durationBeats)
 */
export function getRegionEndBeat(region: Region): number {
  return region.startBeat + region.durationBeats;
}

/**
 * Check if two regions overlap in time on the same track
 * @param region1 - First region
 * @param region2 - Second region
 * @returns True if regions overlap, false otherwise
 */
export function regionOverlaps(region1: Region, region2: Region): boolean {
  // Only regions on the same track can overlap
  if (region1.trackId !== region2.trackId) {
    return false;
  }

  const region1End = getRegionEndBeat(region1);
  const region2End = getRegionEndBeat(region2);

  // Two intervals overlap if:
  // region1.start < region2.end AND region2.start < region1.end
  return region1.startBeat < region2End && region2.startBeat < region1End;
}

/**
 * Move a region to a new start beat position
 * Returns a new region with immutability
 * @param region - Region to move
 * @param newStartBeat - New start beat position
 * @returns New region with updated start beat
 */
export function moveRegion(region: Region, newStartBeat: number): Region {
  if (newStartBeat < 0) {
    throw new Error('Start beat cannot be negative');
  }

  return {
    ...region,
    startBeat: newStartBeat
  };
}

/**
 * Resize a region to a new duration
 * Returns a new region with immutability
 * @param region - Region to resize
 * @param newDurationBeats - New duration in beats
 * @returns New region with updated duration
 */
export function resizeRegion(region: Region, newDurationBeats: number): Region {
  if (newDurationBeats <= 0) {
    throw new Error('Duration must be positive');
  }

  return {
    ...region,
    durationBeats: newDurationBeats
  };
}

/**
 * Update region properties while maintaining immutability
 * @param region - Region to update
 * @param updates - Partial object with fields to update
 * @returns New region with updates applied
 */
export function updateRegion(region: Region, updates: Partial<Region>): Region {
  const updated = { ...region, ...updates };

  // Validate startBeat if changed
  if (updated.startBeat !== region.startBeat) {
    if (updated.startBeat < 0) {
      throw new Error('Start beat cannot be negative');
    }
  }

  // Validate durationBeats if changed
  if (updated.durationBeats !== region.durationBeats) {
    if (updated.durationBeats <= 0) {
      throw new Error('Duration must be positive');
    }
  }

  return updated;
}

// ============================================================================
// UNIT TESTS
// ============================================================================

/**
 * Run all unit tests for region functions
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

  // ========== createRegion tests ==========
  results.push('\n--- createRegion Tests ---');

  const region1 = createRegion('track_1', 0, 4);
  assert(region1.trackId === 'track_1', 'createRegion: sets track ID');
  assert(region1.startBeat === 0, 'createRegion: sets start beat');
  assert(region1.durationBeats === 4, 'createRegion: sets duration beats');
  assert(region1.name.includes('Region'), 'createRegion: generates default name');
  assert(region1.id.startsWith('region_'), 'createRegion: generates region ID');
  assert(region1.color === undefined, 'createRegion: color defaults to undefined');
  assert(region1.midiFileId === undefined, 'createRegion: midiFileId defaults to undefined');

  const region2 = createRegion('track_2', 4, 2);
  assert(region2.id !== region1.id, 'createRegion: generates different IDs');

  try {
    createRegion('', 0, 4);
    results.push('✗ createRegion: should reject empty track ID');
    failed++;
  } catch {
    results.push('✓ createRegion: rejects empty track ID');
    passed++;
  }

  try {
    createRegion('track_1', -1, 4);
    results.push('✗ createRegion: should reject negative start beat');
    failed++;
  } catch {
    results.push('✓ createRegion: rejects negative start beat');
    passed++;
  }

  try {
    createRegion('track_1', 0, 0);
    results.push('✗ createRegion: should reject zero duration');
    failed++;
  } catch {
    results.push('✓ createRegion: rejects zero duration');
    passed++;
  }

  try {
    createRegion('track_1', 0, -1);
    results.push('✗ createRegion: should reject negative duration');
    failed++;
  } catch {
    results.push('✓ createRegion: rejects negative duration');
    passed++;
  }

  // ========== getRegionEndBeat tests ==========
  results.push('\n--- getRegionEndBeat Tests ---');

  assert(getRegionEndBeat(region1) === 4, 'getRegionEndBeat: 0 + 4 = 4');
  assert(getRegionEndBeat(region2) === 6, 'getRegionEndBeat: 4 + 2 = 6');

  const region3 = createRegion('track_3', 2.5, 1.5);
  assert(getRegionEndBeat(region3) === 4, 'getRegionEndBeat: handles fractional beats');

  // ========== regionOverlaps tests ==========
  results.push('\n--- regionOverlaps Tests ---');

  const regionA = createRegion('track_1', 0, 4); // 0-4
  const regionB = createRegion('track_1', 2, 4); // 2-6
  const regionC = createRegion('track_1', 4, 2); // 4-6
  const regionD = createRegion('track_1', 6, 2); // 6-8
  const regionE = createRegion('track_2', 0, 4); // Different track

  assert(regionOverlaps(regionA, regionB), 'regionOverlaps: detects overlapping regions');
  assert(regionOverlaps(regionB, regionA), 'regionOverlaps: commutative property');
  assert(
    !regionOverlaps(regionA, regionC),
    'regionOverlaps: adjacent regions do not overlap'
  );
  assert(!regionOverlaps(regionA, regionD), 'regionOverlaps: separated regions do not overlap');
  assert(
    !regionOverlaps(regionA, regionE),
    'regionOverlaps: regions on different tracks do not overlap'
  );

  const regionF = createRegion('track_1', 1, 2); // 1-3
  assert(regionOverlaps(regionA, regionF), 'regionOverlaps: detects contained region');

  const regionG = createRegion('track_1', 0, 0.5); // 0-0.5
  const regionH = createRegion('track_1', 0.25, 0.5); // 0.25-0.75
  assert(regionOverlaps(regionG, regionH), 'regionOverlaps: detects fractional overlap');

  // ========== moveRegion tests ==========
  results.push('\n--- moveRegion Tests ---');

  const original = createRegion('track_1', 0, 4);
  const moved = moveRegion(original, 8);

  assert(moved.startBeat === 8, 'moveRegion: updates start beat');
  assert(moved.durationBeats === 4, 'moveRegion: preserves duration');
  assert(moved.trackId === 'track_1', 'moveRegion: preserves track ID');
  assert(original.startBeat === 0, 'moveRegion: does not mutate original');
  assert(moved.id === original.id, 'moveRegion: preserves region ID');

  const movedToStart = moveRegion(moved, 0);
  assert(movedToStart.startBeat === 0, 'moveRegion: can move to start position');

  try {
    moveRegion(original, -1);
    results.push('✗ moveRegion: should reject negative start beat');
    failed++;
  } catch {
    results.push('✓ moveRegion: rejects negative start beat');
    passed++;
  }

  // ========== resizeRegion tests ==========
  results.push('\n--- resizeRegion Tests ---');

  const resized = resizeRegion(original, 8);

  assert(resized.startBeat === 0, 'resizeRegion: preserves start beat');
  assert(resized.durationBeats === 8, 'resizeRegion: updates duration');
  assert(resized.trackId === 'track_1', 'resizeRegion: preserves track ID');
  assert(original.durationBeats === 4, 'resizeRegion: does not mutate original');
  assert(resized.id === original.id, 'resizeRegion: preserves region ID');

  const shrunk = resizeRegion(resized, 1);
  assert(shrunk.durationBeats === 1, 'resizeRegion: can shrink region');

  const fractional = resizeRegion(original, 2.5);
  assert(fractional.durationBeats === 2.5, 'resizeRegion: handles fractional durations');

  try {
    resizeRegion(original, 0);
    results.push('✗ resizeRegion: should reject zero duration');
    failed++;
  } catch {
    results.push('✓ resizeRegion: rejects zero duration');
    passed++;
  }

  try {
    resizeRegion(original, -2);
    results.push('✗ resizeRegion: should reject negative duration');
    failed++;
  } catch {
    results.push('✓ resizeRegion: rejects negative duration');
    passed++;
  }

  // ========== updateRegion tests ==========
  results.push('\n--- updateRegion Tests ---');

  const baseRegion = createRegion('track_1', 0, 4);
  const updated = updateRegion(baseRegion, { name: 'Verse', color: '#FF0000' });

  assert(updated.name === 'Verse', 'updateRegion: updates name');
  assert(updated.color === '#FF0000', 'updateRegion: updates color');
  assert(updated.startBeat === 0, 'updateRegion: preserves unchanged fields');
  assert(baseRegion.name !== 'Verse', 'updateRegion: does not mutate original');

  const moved2 = updateRegion(baseRegion, { startBeat: 4, durationBeats: 2 });
  assert(moved2.startBeat === 4, 'updateRegion: updates start beat');
  assert(moved2.durationBeats === 2, 'updateRegion: updates duration');

  try {
    updateRegion(baseRegion, { startBeat: -5 });
    results.push('✗ updateRegion: should reject negative start beat');
    failed++;
  } catch {
    results.push('✓ updateRegion: rejects negative start beat');
    passed++;
  }

  try {
    updateRegion(baseRegion, { durationBeats: 0 });
    results.push('✗ updateRegion: should reject zero duration');
    failed++;
  } catch {
    results.push('✓ updateRegion: rejects zero duration');
    passed++;
  }

  // ========== Integration tests ==========
  results.push('\n--- Integration Tests ---');

  // Create regions and move/resize them
  const seq1 = createRegion('track_1', 0, 4);
  const seq2 = moveRegion(seq1, 4);
  const seq3 = resizeRegion(seq2, 2);

  assert(
    !regionOverlaps(seq1, seq2),
    'Integration: moved region does not overlap original'
  );
  assert(
    !regionOverlaps(seq2, seq3),
    'Integration: resized region does not create overlap'
  );
  assert(
    getRegionEndBeat(seq1) === 4,
    'Integration: end beat calculation consistent'
  );

  return { passed, failed, tests: results };
}

export const test = runTests;
