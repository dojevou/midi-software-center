// ============================================================================
// MIDI CONSTANTS
// ============================================================================

export const MIDI_CHANNELS = 16;
export const NOTES_PER_OCTAVE = 12;
export const DEFAULT_TPQN = 480; // Ticks per quarter note
export const MIN_BPM = 30;
export const MAX_BPM = 300;
export const DEFAULT_BPM = 120;

export const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

export const MUSICAL_KEYS = [
  'C', 'Cm', 'C#', 'C#m', 'D', 'Dm', 'D#', 'D#m',
  'E', 'Em', 'F', 'Fm', 'F#', 'F#m', 'G', 'Gm',
  'G#', 'G#m', 'A', 'Am', 'A#', 'A#m', 'B', 'Bm'
];

export const TIME_SIGNATURES = [
  [4, 4], [3, 4], [6, 8], [5, 4], [7, 8], [2, 4], [12, 8]
];

// ============================================================================
// FILE CATEGORIES
// ============================================================================

export const FILE_CATEGORIES = [
  'KICK', 'SNARE', 'HIHAT', 'PERCUSSION', 'BASS', 'LEAD',
  'PAD', 'CHORD', 'ARP', 'FX', 'VOCAL', 'LOOP', 'UNKNOWN'
] as const;

// ============================================================================
// UI CONSTANTS
// ============================================================================

export const DEFAULT_WINDOW_WIDTH = 800;
export const DEFAULT_WINDOW_HEIGHT = 600;
export const MIN_WINDOW_WIDTH = 400;
export const MIN_WINDOW_HEIGHT = 300;

// ============================================================================
// SEARCH CONSTANTS
// ============================================================================

export const DEFAULT_PAGE_SIZE = 50;
export const MAX_SEARCH_RESULTS = 500;

// ============================================================================
// PERFORMANCE CONSTANTS
// ============================================================================

export const DEBOUNCE_DELAY = 300; // ms
export const THROTTLE_INTERVAL = 100; // ms