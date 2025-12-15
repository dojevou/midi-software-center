//! Database layer for the MIDI Software Center.
//!
//! This module provides:
//! - `models` - All database model types (files, analysis, metadata, search, sequencer)
//! - `repositories` - Database access layer with CRUD operations

pub mod models;
pub mod repositories;

// ============================================================================
// Commonly Used Re-exports
// ============================================================================

// Core file types
pub use models::{CreateMidiFile, File, FileRef, MidiFile, UpdateMidiFile};

// Analysis types
pub use models::{AnalysisResult, CreateAnalysisResult, DrumPattern, Tag};

// Metadata types
pub use models::{CreateMidiMetadata, FileInstrument, MidiMetadata, MidiTrack};

// Search types
pub use models::{SearchFilters, SearchQuery, SearchResult, SearchResults, SortField, SortOrder};

// Sequencer/DAW types
pub use models::{PlaybackState, SequencerNote, SequencerProject, SequencerTrack};

// Error types
pub use models::{DbError, DbResult, ErrorContext, OptionExt};
