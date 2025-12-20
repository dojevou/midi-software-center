/**
 * VIP3 filter counts matching Rust FilterCounts struct
 */
export interface FilterCounts {
  folder_counts: Record<number, number>;
  instrument_counts: Record<number, number>;
  timbre_counts: Record<number, number>;
  style_counts: Record<number, number>;
  articulation_counts: Record<number, number>;
  bpm_range_counts: Record<number, number>; // bpm_range_id → count
  bpm_counts: Record<string, number>; // BPM label (e.g., "60-80") → count
  key_counts: Record<number, number>; // key_id → count
  channel_counts: Record<number, number>; // channel_count (1-16) → count
  total_matches: number;
}

/**
 * VIP3 filter state matching Rust Vip3Filters struct
 */
export interface Vip3Filters {
  folder_ids?: number[];
  instrument_ids?: number[];
  timbre_ids?: number[];
  style_ids?: number[];
  articulation_ids?: number[];
  bpm_range_ids?: number[]; // BPM range IDs from bpm_ranges table
  bpm_min?: number; // Minimum BPM for custom range filtering
  bpm_max?: number; // Maximum BPM for custom range filtering
  key_ids?: number[]; // Key signature IDs from musical_keys table
  channel?: number; // MIDI channel count (1-16)
  search_query?: string; // Free-text search
  favorites_only?: boolean; // Show only favorites
  tag_ids?: number[]; // General tag IDs (not instruments)
  min_rating?: number; // Minimum rating (1-5)
  limit?: number; // Result limit for pagination
  offset?: number; // Result offset for pagination
  [key: string]: unknown; // Index signature for InvokeArgs compatibility
}

/**
 * VIP3 category types
 */
export interface Timbre {
  id: number;
  name: string;
  description?: string;
}

export interface Style {
  id: number;
  name: string;
  description?: string;
}

export interface Articulation {
  id: number;
  name: string;
  description?: string;
}

/**
 * VIP3 search results - matches Rust Vip3FileResult
 */
export interface Vip3SearchResult {
  id: number;
  filename: string;
  filepath: string;
  bpm?: number;
  key_signature?: string;
  duration_ms?: number;
  note_count?: number;
  channel?: number;
  rating?: number;
  favorite?: boolean;
  created_at?: string;
}

/**
 * VIP3 search response with pagination - matches Rust Vip3SearchResults
 */
export interface Vip3SearchResponse {
  files: Vip3SearchResult[];
  total_count: number;
  page: number;
  page_size: number;
  total_pages: number;
  has_more?: boolean;
}

/**
 * Saved search response from backend
 */
export interface SavedSearchResponse {
  id: number;
  name: string;
  description?: string;
  filters: any; // JSON value - Vip3Filters
  sort_by?: string;
  sort_order?: string;
  icon?: string;
  color?: string;
  is_pinned: boolean;
  created_at?: string;
  last_used?: string;
  use_count?: number;
}

/**
 * Request to create a saved search
 */
export interface CreateSavedSearchRequest {
  name: string;
  description?: string;
  filters: Vip3Filters;
  sort_by?: string;
  sort_order?: string;
  icon?: string;
  color?: string;
  [key: string]: unknown; // Index signature for InvokeArgs compatibility
}

/**
 * Collection response from backend
 */
export interface CollectionResponse {
  id: number;
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  is_smart?: boolean;
  file_count?: number;
  created_at?: string;
  modified_at?: string;
}

/**
 * Request to create a collection
 */
export interface CreateCollectionRequest {
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  is_smart?: boolean;
  smart_filters?: any; // JSON value - Vip3Filters
  [key: string]: unknown; // Index signature for InvokeArgs compatibility
}

/**
 * Request to update a collection
 */
export interface UpdateCollectionRequest {
  name?: string;
  description?: string;
  icon?: string;
  color?: string;
  smart_filters?: any; // JSON value - Vip3Filters
}
