/**
 * Trusty Module: Grid Calculations
 * Pure functions for MIDI grid and quantization logic
 * Zero I/O, fully deterministic, comprehensive test coverage
 */

export interface BarBeatTick {
  bar: number;
  beat: number;
  tick: number;
}

/**
 * Snap a pixel position to the nearest grid point
 * @param positionPixels - Current pixel position
 * @param gridDivisor - Grid resolution (4=quarters, 8=eighths, 16=sixteenths)
 * @returns Snapped pixel position
 */
export function snapToGrid(positionPixels: number, gridDivisor: number): number {
  if (gridDivisor <= 0) {
    throw new Error('Grid divisor must be positive');
  }
  if (positionPixels < 0) {
    throw new Error('Position cannot be negative');
  }

  // Calculate grid unit size (1 beat = 1 grid unit at divisor 4)
  // Divisor 4 = quarter note grid, 8 = eighth note grid, etc.
  const gridUnitSize = 100 / gridDivisor; // Assuming 100 pixels per beat base unit
  const gridPoints = positionPixels / gridUnitSize;
  const snappedGridPoints = Math.round(gridPoints);
  return snappedGridPoints * gridUnitSize;
}

/**
 * Convert pixel position to beat count
 * @param pixels - Pixel distance
 * @param pixelsPerBeat - Pixels that represent one beat
 * @returns Beat count (can be fractional)
 */
export function pixelsToBeats(pixels: number, pixelsPerBeat: number): number {
  if (pixelsPerBeat <= 0) {
    throw new Error('Pixels per beat must be positive');
  }
  if (pixels < 0) {
    throw new Error('Pixels cannot be negative');
  }

  return pixels / pixelsPerBeat;
}

/**
 * Convert beat count to pixel position
 * @param beats - Beat position (can be fractional)
 * @param pixelsPerBeat - Pixels that represent one beat
 * @returns Pixel position
 */
export function beatsToPixels(beats: number, pixelsPerBeat: number): number {
  if (pixelsPerBeat <= 0) {
    throw new Error('Pixels per beat must be positive');
  }
  if (beats < 0) {
    throw new Error('Beats cannot be negative');
  }

  return beats * pixelsPerBeat;
}

/**
 * Convert beat position to bar:beat:tick format
 * @param beatPosition - Beat offset from start (0-based, can be fractional)
 * @param beatsPerMeasure - Beats per measure/bar (default: 4 for 4/4 time)
 * @returns Object with bar, beat, tick components
 */
export function positionToBarBeatTick(
  beatPosition: number,
  beatsPerMeasure: number = 4
): BarBeatTick {
  if (beatsPerMeasure <= 0) {
    throw new Error('Beats per measure must be positive');
  }
  if (beatPosition < 0) {
    throw new Error('Beat position cannot be negative');
  }

  // Calculate bar (0-based)
  const totalBeats = Math.floor(beatPosition);
  const bar = Math.floor(totalBeats / beatsPerMeasure);
  
  // Calculate beat within measure (0-based, 0-3 for 4/4)
  const beatInMeasure = totalBeats % beatsPerMeasure;
  
  // Calculate tick (fractional part, representing sub-beat divisions)
  // Ticks represent 1/24th of a quarter note (standard MIDI resolution)
  const fractionalBeat = beatPosition - Math.floor(beatPosition);
  const tick = Math.round(fractionalBeat * 24);

  return {
    bar: bar + 1, // 1-based bar numbering
    beat: beatInMeasure + 1, // 1-based beat numbering
    tick
  };
}

/**
 * Quantize a note time to the nearest grid point
 * @param noteTime - Current note time in beats
 * @param gridDivisor - Grid resolution (4=quarters, 8=eighths, 16=sixteenths)
 * @param beatsPerMeasure - Beats per measure (default: 4)
 * @returns Quantized note time in beats
 */
export function quantizeNote(
  noteTime: number,
  gridDivisor: number,
  beatsPerMeasure: number = 4
): number {
  if (gridDivisor <= 0) {
    throw new Error('Grid divisor must be positive');
  }
  if (beatsPerMeasure <= 0) {
    throw new Error('Beats per measure must be positive');
  }
  if (noteTime < 0) {
    throw new Error('Note time cannot be negative');
  }

  // Grid unit size in beats
  // gridDivisor 4 = 1 beat per grid point (quarter notes)
  // gridDivisor 8 = 0.5 beats per grid point (eighth notes)
  // gridDivisor 16 = 0.25 beats per grid point (sixteenth notes)
  const gridUnitInBeats = 4 / gridDivisor;
  
  // Round to nearest grid point
  const gridPoints = noteTime / gridUnitInBeats;
  const snappedGridPoints = Math.round(gridPoints);
  
  return snappedGridPoints * gridUnitInBeats;
}

// ============================================================================
// UNIT TESTS
// ============================================================================

/**
 * Run all unit tests for grid calculation functions
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

  const assertClose = (actual: number, expected: number, tolerance: number, message: string) => {
    if (Math.abs(actual - expected) <= tolerance) {
      results.push(`✓ ${message}`);
      passed++;
    } else {
      results.push(`✗ ${message} (got ${actual}, expected ${expected})`);
      failed++;
    }
  };

  // ========== snapToGrid tests ==========
  results.push('\n--- snapToGrid Tests ---');
  
  assertClose(
    snapToGrid(0, 4),
    0,
    0.01,
    'snapToGrid: zero position stays at zero'
  );
  
  assertClose(
    snapToGrid(150, 4),
    150,
    0.01,
    'snapToGrid: already snapped position unchanged'
  );
  
  assertClose(
    snapToGrid(112, 4),
    100,
    0.01,
    'snapToGrid: rounds 112px down to 100px grid point (div=4)'
  );
  
  assertClose(
    snapToGrid(137, 4),
    125,
    0.01,
    'snapToGrid: rounds 137px to 125px grid point (div=4)'
  );
  
  assertClose(
    snapToGrid(156.25, 8),
    162.5,
    0.01,
    'snapToGrid: rounds 156.25px up to 162.5px grid point (div=8)'
  );

  try {
    snapToGrid(-10, 4);
    results.push('✗ snapToGrid: should reject negative position');
    failed++;
  } catch {
    results.push('✓ snapToGrid: rejects negative position');
    passed++;
  }

  try {
    snapToGrid(100, 0);
    results.push('✗ snapToGrid: should reject zero divisor');
    failed++;
  } catch {
    results.push('✓ snapToGrid: rejects zero divisor');
    passed++;
  }

  // ========== pixelsToBeats tests ==========
  results.push('\n--- pixelsToBeats Tests ---');
  
  assertClose(
    pixelsToBeats(0, 100),
    0,
    0.01,
    'pixelsToBeats: zero pixels = zero beats'
  );
  
  assertClose(
    pixelsToBeats(100, 100),
    1,
    0.01,
    'pixelsToBeats: 100px at 100px/beat = 1 beat'
  );
  
  assertClose(
    pixelsToBeats(250, 100),
    2.5,
    0.01,
    'pixelsToBeats: 250px at 100px/beat = 2.5 beats'
  );
  
  assertClose(
    pixelsToBeats(50, 100),
    0.5,
    0.01,
    'pixelsToBeats: 50px at 100px/beat = 0.5 beats'
  );

  try {
    pixelsToBeats(100, 0);
    results.push('✗ pixelsToBeats: should reject zero pixels per beat');
    failed++;
  } catch {
    results.push('✓ pixelsToBeats: rejects zero pixels per beat');
    passed++;
  }

  // ========== beatsToPixels tests ==========
  results.push('\n--- beatsToPixels Tests ---');
  
  assertClose(
    beatsToPixels(0, 100),
    0,
    0.01,
    'beatsToPixels: zero beats = zero pixels'
  );
  
  assertClose(
    beatsToPixels(1, 100),
    100,
    0.01,
    'beatsToPixels: 1 beat at 100px/beat = 100px'
  );
  
  assertClose(
    beatsToPixels(2.5, 100),
    250,
    0.01,
    'beatsToPixels: 2.5 beats at 100px/beat = 250px'
  );
  
  assertClose(
    beatsToPixels(0.5, 100),
    50,
    0.01,
    'beatsToPixels: 0.5 beats at 100px/beat = 50px'
  );

  // ========== positionToBarBeatTick tests ==========
  results.push('\n--- positionToBarBeatTick Tests ---');
  
  const result1 = positionToBarBeatTick(0, 4);
  assert(
    result1.bar === 1 && result1.beat === 1 && result1.tick === 0,
    'positionToBarBeatTick: beat 0 = bar:1 beat:1 tick:0'
  );
  
  const result2 = positionToBarBeatTick(3.5, 4);
  assert(
    result2.bar === 1 && result2.beat === 4 && result2.tick === 12,
    'positionToBarBeatTick: beat 3.5 = bar:1 beat:4 tick:12 (half of 24)'
  );
  
  const result3 = positionToBarBeatTick(4, 4);
  assert(
    result3.bar === 2 && result3.beat === 1 && result3.tick === 0,
    'positionToBarBeatTick: beat 4 = bar:2 beat:1 tick:0'
  );
  
  const result4 = positionToBarBeatTick(8.75, 4);
  assert(
    result4.bar === 3 && result4.beat === 1 && result4.tick === 18,
    'positionToBarBeatTick: beat 8.75 = bar:3 beat:1 tick:18'
  );
  
  const result5 = positionToBarBeatTick(6, 3);
  assert(
    result5.bar === 3 && result5.beat === 1 && result5.tick === 0,
    'positionToBarBeatTick: works with different beat per measure (3/4 time)'
  );

  try {
    positionToBarBeatTick(-1, 4);
    results.push('✗ positionToBarBeatTick: should reject negative beat position');
    failed++;
  } catch {
    results.push('✓ positionToBarBeatTick: rejects negative beat position');
    passed++;
  }

  // ========== quantizeNote tests ==========
  results.push('\n--- quantizeNote Tests ---');
  
  assertClose(
    quantizeNote(0, 4, 4),
    0,
    0.01,
    'quantizeNote: zero time stays at zero'
  );
  
  assertClose(
    quantizeNote(1, 4, 4),
    1,
    0.01,
    'quantizeNote: beat on grid point (div=4) unchanged'
  );
  
  assertClose(
    quantizeNote(1.1, 4, 4),
    1,
    0.01,
    'quantizeNote: 1.1 beats snaps down to 1 beat (div=4)'
  );
  
  assertClose(
    quantizeNote(1.4, 4, 4),
    1,
    0.01,
    'quantizeNote: 1.4 beats snaps down to 1 beat (div=4)'
  );
  
  assertClose(
    quantizeNote(1.6, 4, 4),
    2,
    0.01,
    'quantizeNote: 1.6 beats snaps up to 2 beats (div=4)'
  );
  
  assertClose(
    quantizeNote(2.3, 8, 4),
    2.5,
    0.01,
    'quantizeNote: finer grid with divisor 8 (0.5 beat grid), 2.3 rounds to 2.5'
  );
  
  assertClose(
    quantizeNote(2.1, 16, 4),
    2.0,
    0.01,
    'quantizeNote: finest grid with divisor 16 (0.25 beat grid), 2.1 rounds down to 2.0'
  );
  
  assertClose(
    quantizeNote(3.7, 4, 3),
    4,
    0.01,
    'quantizeNote: works with different beats per measure'
  );

  try {
    quantizeNote(1, 0, 4);
    results.push('✗ quantizeNote: should reject zero divisor');
    failed++;
  } catch {
    results.push('✓ quantizeNote: rejects zero divisor');
    passed++;
  }

  try {
    quantizeNote(1, 4, 0);
    results.push('✗ quantizeNote: should reject zero beats per measure');
    failed++;
  } catch {
    results.push('✓ quantizeNote: rejects zero beats per measure');
    passed++;
  }

  // ========== Roundtrip tests ==========
  results.push('\n--- Roundtrip Conversion Tests ---');
  
  const originalBeats = 5.75;
  const pixelsPerBeat = 100;
  const pixels = beatsToPixels(originalBeats, pixelsPerBeat);
  const recoveredBeats = pixelsToBeats(pixels, pixelsPerBeat);
  assertClose(
    recoveredBeats,
    originalBeats,
    0.01,
    'Roundtrip: beats -> pixels -> beats preserves value'
  );

  const originalPixels = 487.5;
  const recoveredPixels = beatsToPixels(
    pixelsToBeats(originalPixels, pixelsPerBeat),
    pixelsPerBeat
  );
  assertClose(
    recoveredPixels,
    originalPixels,
    0.01,
    'Roundtrip: pixels -> beats -> pixels preserves value'
  );

  return { passed, failed, tests: results };
}

// Export test runner for easy access
export const test = runTests;