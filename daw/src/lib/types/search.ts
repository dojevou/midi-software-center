/**
 * Search and filter type definitions
 *
 * Used for searching and filtering MIDI files in the library.
 */

import type { MidiFile } from './core';

/**
 * Search filters applied to file queries
 */
export interface SearchFilters {
  categories: string[];
  bpmRange: [number, number];
  keySignature: string[];
  timeSignature: string[];
  instruments: string[];
  duration: [number, number];
}

/**
 * Autocomplete suggestion
 */
export interface Suggestion {
  text: string;
  type: 'category' | 'instrument' | 'key' | 'bpm';
  count: number;
}

/**
 * Filter option for dropdowns
 */
export interface FilterOption {
  value: string;
  label: string;
  count: number;
}

/**
 * Search response with pagination
 */
export interface SearchResponse {
  files: MidiFile[];
  total: number;
}

/**
 * Search parameters
 */
export interface SearchParams {
  query: string;
  filters: SearchFilters;
  page: number;
  pageSize: number;
}
