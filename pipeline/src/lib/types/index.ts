/**
 * Core type definitions for the MIDI library frontend
 */

/**
 * Tag with metadata
 */
export interface Tag {
  id: number;
  name: string;
  category?: string; // e.g., 'genre', 'instrument', 'brand'
  usageCount: number;
}

/**
 * Tag statistics for analytics
 */
export interface TagStats {
  totalTags: number;
  totalFiles: number;
  topTags: Array<{ name: string; count: number; category?: string }>;
}

/**
 * Tagged file metadata
 */
export interface FileMetadata {
  id: number;
  filename: string;
  originalFilename: string;
  category: FileCategory;
  parentFolder?: string;
  subcategory?: string;
  bpm?: number;
  key?: MusicalKey;
  duration?: number;
  fileSize: number;
  createdAt: string;
  updatedAt: string;
  tags?: Tag[]; // All tags (auto + manual)
  userTags?: string[]; // DEPRECATED: Use tags instead
  autoTags?: string[]; // DEPRECATED: Use tags instead
}

export type FileCategory =
  | 'KICK'
  | 'SNARE'
  | 'HIHAT'
  | 'PERCUSSION'
  | 'BASS'
  | 'LEAD'
  | 'PAD'
  | 'CHORD'
  | 'ARP'
  | 'FX'
  | 'VOCAL'
  | 'LOOP'
  | 'UNKNOWN';

export type MusicalKey =
  | 'C' | 'Cm'
  | 'C#' | 'C#m'
  | 'D' | 'Dm'
  | 'D#' | 'D#m'
  | 'E' | 'Em'
  | 'F' | 'Fm'
  | 'F#' | 'F#m'
  | 'G' | 'Gm'
  | 'G#' | 'G#m'
  | 'A' | 'Am'
  | 'A#' | 'A#m'
  | 'B' | 'Bm'
  | 'UNKNOWN';

export interface ImportProgress {
  current: number;
  total: number;
  fileName: string;
  status: 'pending' | 'processing' | 'complete' | 'error';
}

export interface ImportSummary {
  totalFiles: number;
  imported: number;
  skipped: number;
  errors: string[];
}

export interface SearchFilters {
  category?: FileCategory;
  minBpm?: number;
  maxBpm?: number;
  key?: MusicalKey;
  tags?: string[];
  tagMatchAll?: boolean; // If true, file must have ALL tags (AND logic). If false, at least one tag (OR logic)
}

export interface SearchResults {
  files: FileMetadata[];
  totalCount: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

export interface ApiError {
  message: string;
  code?: string;
  details?: unknown;
}
