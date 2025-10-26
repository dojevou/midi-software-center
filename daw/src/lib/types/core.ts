/**
 * Core type definitions for the DAW application
 *
 * These types match the Rust backend types exactly for proper serialization.
 * Do not modify without updating corresponding Rust types.
 */

/**
 * Base MIDI file information from database
 */
export interface MidiFile {
  id: number;
  file_name: string;
  file_path: string;
  file_size: number;
  category: string | null;
  bpm: number | null;
  key_signature: string | null;
  time_signature: string | null;
  duration_seconds: number;
  track_count: number;
  note_count: number;
  instruments: string[];
  created_at: string;
  content_hash: string;
}

/**
 * Extended file details with additional metadata
 */
export interface FileDetails extends MidiFile {
  // All fields from MidiFile
  // Additional computed fields can be added here in the future
}

/**
 * Application state
 */
export interface AppState {
  dbConnected: boolean;
  totalFiles: number;
  selectedFiles: Set<number>;
  viewMode: 'grid' | 'list';
}
