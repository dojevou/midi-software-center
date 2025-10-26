/**
 * Trusty Module: track-management
 * Pure logic for MIDI track representation and manipulation
 * Zero I/O, fully deterministic, comprehensive test coverage
 */

export interface Track {
  id: string;
  name: string;
  channel: number; // 0-15 MIDI channel
  muted: boolean;
  solo: boolean;
  volume: number; // 0-127
  pan: number; // -64 to 63
  recordArmed: boolean;
}

/**
 * Generate a simple unique ID for a track
 * Uses timestamp + random number for pseudo-uniqueness
 */
function generateTrackId(): string {
  return `track_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
}

/**
 * Create a new track with default values
 * @param name - Track name
 * @param channel - MIDI channel (0-15)
 * @returns New Track object with sensible defaults
 */
export function createTrack(name: string, channel: number): Track {
  if (!name || name.length === 0) {
    throw new Error('Track name cannot be empty');
  }
  if (channel < 0 || channel > 15) {
    throw new Error('Channel must be between 0 and 15');
  }

  return {
    id: generateTrackId(),
    name,
    channel,
    muted: false,
    solo: false,
    volume: 100, // Default to 100/127 (roughly 80%)
    pan: 0, // Center
    recordArmed: false
  };
}

/**
 * Get list of active MIDI channels from tracks
 * If any track is soloed, returns only soloed tracks' channels
 * Otherwise returns channels from all non-muted tracks
 * @param tracks - Array of tracks
 * @returns Array of MIDI channel numbers
 */
export function getActiveChannels(tracks: Track[]): number[] {
  const soloedTracks = tracks.filter((t) => t.solo);

  // If any track is soloed, only use soloed tracks
  if (soloedTracks.length > 0) {
    const channels = soloedTracks.map((t) => t.channel);
    // Remove duplicates and sort
    return Array.from(new Set(channels)).sort((a, b) => a - b);
  }

  // Otherwise, get all non-muted tracks
  const activeTracks = tracks.filter((t) => !t.muted);
  const channels = activeTracks.map((t) => t.channel);
  // Remove duplicates and sort
  return Array.from(new Set(channels)).sort((a, b) => a - b);
}

/**
 * Filter tracks based on muted visibility
 * @param tracks - Array of tracks to filter
 * @param showMuted - If true, include muted tracks; if false, exclude them
 * @returns Filtered array of tracks
 */
export function filterTracksByVisibility(tracks: Track[], showMuted: boolean): Track[] {
  if (showMuted) {
    return [...tracks];
  }
  return tracks.filter((t) => !t.muted);
}

/**
 * Sort tracks by their ID
 * @param tracks - Array of tracks to sort
 * @returns New sorted array
 */
export function sortTracksById(tracks: Track[]): Track[] {
  return [...tracks].sort((a, b) => a.id.localeCompare(b.id));
}

/**
 * Sort tracks by their channel number
 * @param tracks - Array of tracks to sort
 * @returns New sorted array
 */
export function sortTracksByChannel(tracks: Track[]): Track[] {
  return [...tracks].sort((a, b) => a.channel - b.channel);
}

/**
 * Update a track's properties while maintaining immutability
 * @param track - Track to update
 * @param updates - Partial object with fields to update
 * @returns New track with updates applied
 */
export function updateTrack(track: Track, updates: Partial<Track>): Track {
  const updated = { ...track, ...updates };

  // Validate channel if changed
  if (updated.channel !== track.channel) {
    if (updated.channel < 0 || updated.channel > 15) {
      throw new Error('Channel must be between 0 and 15');
    }
  }

  // Validate volume if changed
  if (updated.volume !== track.volume) {
    if (updated.volume < 0 || updated.volume > 127) {
      throw new Error('Volume must be between 0 and 127');
    }
  }

  // Validate pan if changed
  if (updated.pan !== track.pan) {
    if (updated.pan < -64 || updated.pan > 63) {
      throw new Error('Pan must be between -64 and 63');
    }
  }

  return updated;
}

// ============================================================================
// UNIT TESTS
// ============================================================================

/**
 * Run all unit tests for track functions
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

  // ========== createTrack tests ==========
  results.push('\n--- createTrack Tests ---');

  const track1 = createTrack('Drums', 0);
  assert(track1.name === 'Drums', 'createTrack: sets track name');
  assert(track1.channel === 0, 'createTrack: sets channel');
  assert(track1.muted === false, 'createTrack: defaults muted to false');
  assert(track1.solo === false, 'createTrack: defaults solo to false');
  assert(track1.volume === 100, 'createTrack: defaults volume to 100');
  assert(track1.pan === 0, 'createTrack: defaults pan to 0 (center)');
  assert(track1.recordArmed === false, 'createTrack: defaults recordArmed to false');
  assert(track1.id.startsWith('track_'), 'createTrack: generates track ID');

  const track2 = createTrack('Bass', 1);
  assert(track2.id !== track1.id, 'createTrack: generates different IDs for different tracks');

  try {
    createTrack('', 0);
    results.push('✗ createTrack: should reject empty name');
    failed++;
  } catch {
    results.push('✓ createTrack: rejects empty name');
    passed++;
  }

  try {
    createTrack('Track', -1);
    results.push('✗ createTrack: should reject negative channel');
    failed++;
  } catch {
    results.push('✓ createTrack: rejects negative channel');
    passed++;
  }

  try {
    createTrack('Track', 16);
    results.push('✗ createTrack: should reject channel > 15');
    failed++;
  } catch {
    results.push('✓ createTrack: rejects channel > 15');
    passed++;
  }

  // ========== getActiveChannels tests ==========
  results.push('\n--- getActiveChannels Tests ---');

  const drumTrack = createTrack('Drums', 0);
  const bassTrack = createTrack('Bass', 1);
  const guitarTrack = createTrack('Guitar', 2);
  const muteGuitar = { ...guitarTrack, muted: true };

  assert(
    getActiveChannels([drumTrack, bassTrack, guitarTrack]).length === 3,
    'getActiveChannels: returns all non-muted channels'
  );

  assert(
    getActiveChannels([drumTrack, bassTrack, muteGuitar]).length === 2,
    'getActiveChannels: excludes muted track channels'
  );

  const activeChannels = getActiveChannels([drumTrack, bassTrack, muteGuitar]);
  assert(
    activeChannels[0] === 0 && activeChannels[1] === 1,
    'getActiveChannels: returns correct channel numbers'
  );

  // Test solo behavior
  const soloDrums = { ...drumTrack, solo: true };
  const soloChannels = getActiveChannels([soloDrums, bassTrack, guitarTrack]);
  assert(soloChannels.length === 1, 'getActiveChannels: returns only solo track when solo enabled');
  assert(soloChannels[0] === 0, 'getActiveChannels: solo returns correct channel');

  // Multiple solos
  const soloBass = { ...bassTrack, solo: true };
  const multiSoloChannels = getActiveChannels([soloDrums, soloBass, guitarTrack]);
  assert(multiSoloChannels.length === 2, 'getActiveChannels: handles multiple solo tracks');

  // Duplicate channels
  const trackA = createTrack('Track A', 5);
  const trackB = createTrack('Track B', 5);
  const dupChannels = getActiveChannels([trackA, trackB]);
  assert(dupChannels.length === 1, 'getActiveChannels: removes duplicate channels');
  assert(dupChannels[0] === 5, 'getActiveChannels: keeps correct duplicate channel');

  // ========== filterTracksByVisibility tests ==========
  results.push('\n--- filterTracksByVisibility Tests ---');

  const allTracks = [drumTrack, { ...bassTrack, muted: true }, guitarTrack];

  const showingMuted = filterTracksByVisibility(allTracks, true);
  assert(showingMuted.length === 3, 'filterTracksByVisibility: includes all when showMuted=true');

  const hidingMuted = filterTracksByVisibility(allTracks, false);
  assert(hidingMuted.length === 2, 'filterTracksByVisibility: excludes muted when showMuted=false');
  assert(
    hidingMuted.every((t) => !t.muted),
    'filterTracksByVisibility: all returned tracks are unmuted'
  );

  const emptyFilter = filterTracksByVisibility([], false);
  assert(emptyFilter.length === 0, 'filterTracksByVisibility: handles empty array');

  // ========== sortTracksById tests ==========
  results.push('\n--- sortTracksById Tests ---');

  const unsorted = [guitarTrack, drumTrack, bassTrack];
  const sorted = sortTracksById(unsorted);

  assert(sorted.length === 3, 'sortTracksById: maintains array length');
  assert(
    sorted[0].id.localeCompare(sorted[1].id) <= 0,
    'sortTracksById: sorts ids in ascending order'
  );
  assert(
    sorted[1].id.localeCompare(sorted[2].id) <= 0,
    'sortTracksById: maintains sorted order'
  );

  // Verify original not mutated
  assert(unsorted[0].id === guitarTrack.id, 'sortTracksById: does not mutate original array');

  // ========== sortTracksByChannel tests ==========
  results.push('\n--- sortTracksByChannel Tests ---');

  const unorderedChannels = [guitarTrack, drumTrack, bassTrack];
  const channelSorted = sortTracksByChannel(unorderedChannels);

  assert(
    channelSorted[0].channel === 0 && channelSorted[1].channel === 1,
    'sortTracksByChannel: sorts by channel ascending'
  );
  assert(channelSorted[2].channel === 2, 'sortTracksByChannel: correct last channel');

  // ========== updateTrack tests ==========
  results.push('\n--- updateTrack Tests ---');

  const original = createTrack('Original', 5);
  const updated = updateTrack(original, { name: 'Updated', volume: 80 });

  assert(updated.name === 'Updated', 'updateTrack: updates name');
  assert(updated.volume === 80, 'updateTrack: updates volume');
  assert(updated.channel === 5, 'updateTrack: preserves unchanged fields');
  assert(original.name === 'Original', 'updateTrack: does not mutate original');

  try {
    updateTrack(original, { channel: 16 });
    results.push('✗ updateTrack: should reject invalid channel');
    failed++;
  } catch {
    results.push('✓ updateTrack: rejects invalid channel');
    passed++;
  }

  try {
    updateTrack(original, { volume: 128 });
    results.push('✗ updateTrack: should reject invalid volume');
    failed++;
  } catch {
    results.push('✓ updateTrack: rejects invalid volume');
    passed++;
  }

  try {
    updateTrack(original, { pan: 64 });
    results.push('✗ updateTrack: should reject invalid pan');
    failed++;
  } catch {
    results.push('✓ updateTrack: rejects invalid pan');
    passed++;
  }

  const validUpdate = updateTrack(original, { pan: 63, volume: 127 });
  assert(validUpdate.pan === 63, 'updateTrack: accepts max pan value');
  assert(validUpdate.volume === 127, 'updateTrack: accepts max volume value');

  return { passed, failed, tests: results };
}

export const test = runTests;
