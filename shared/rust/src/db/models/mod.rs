//! Database models for the MIDI Software Center.
//!
//! This module provides all database model types organized by domain:
//! - `midi_file` - Core file records and metadata
//! - `analysis` - Analysis results, tags, and drum patterns
//! - `midi` - Musical metadata and instrument information
//! - `search` - Search queries, filters, and results
//! - `sequencer` - DAW/sequencer projects and tracks
//! - `error` - Database error types
//! - `timbre` - Sound character categories (VIP3 filtering)
//! - `style` - Musical genres/styles (VIP3 filtering)
//! - `articulation` - Playing styles/pattern types (VIP3 filtering)
//! - `bpm_range` - Tempo ranges for filtering
//! - `musical_key` - Musical keys for filtering
//! - `collection` - User collections/playlists
//! - `saved_search` - Saved filter configurations

pub mod analysis;
pub mod articulation;
pub mod bpm_range;
pub mod collection;
pub mod error;
pub mod midi;
pub mod midi_file;
pub mod musical_key;
pub mod saved_search;
pub mod search;
pub mod sequencer;
pub mod style;
pub mod timbre;

// ============================================================================
// MIDI File Models
// ============================================================================

pub use midi_file::{
    CreateMidiFile, File, FileRef, ImportBatch, MidiFile, SplitFileInfo, UpdateMidiFile,
};

// ============================================================================
// Analysis Models
// ============================================================================

pub use analysis::{
    AnalysisResult, CreateAnalysisResult, CreateDrumPattern, CreateTag, DrumPattern, FileTag, Tag,
    TagWithCount, UpdateAnalysisResult, UpdateTag,
};

// ============================================================================
// Musical Metadata Models
// ============================================================================

pub use midi::{
    CreateFileInstrument, CreateMidiMetadata, FileInstrument, MidiMetadata, MidiTrack,
    UpdateMidiMetadata,
};

// ============================================================================
// Search Models
// ============================================================================

pub use search::{
    QuickSearchResult, SearchFilters, SearchQuery, SearchResult, SearchResults, SortField,
    SortOrder,
};

// ============================================================================
// Sequencer/DAW Models
// ============================================================================

pub use sequencer::{
    AutomationLane, AutomationPoint, CreateSequencerProject, CreateSequencerTrack, PlaybackState,
    SequencerNote, SequencerProject, SequencerTrack, UpdateSequencerTrack,
};

// ============================================================================
// Error Types
// ============================================================================

pub use error::{DbError, DbResult, ErrorContext, OptionExt};

// ============================================================================
// VIP3 Filtering Models - Timbre
// ============================================================================

pub use timbre::{
    CreateTimbre, FileTimbre, Timbre, TimbreWithCount, UpdateTimbre, SYSTEM_TIMBRES,
};

// ============================================================================
// VIP3 Filtering Models - Style
// ============================================================================

pub use style::{CreateStyle, FileStyle, Style, StyleWithCount, UpdateStyle, SYSTEM_STYLES};

// ============================================================================
// VIP3 Filtering Models - Articulation
// ============================================================================

pub use articulation::{
    Articulation, ArticulationWithCount, CreateArticulation, FileArticulation, UpdateArticulation,
    SYSTEM_ARTICULATIONS,
};

// ============================================================================
// VIP3 Filtering Models - BPM Range
// ============================================================================

pub use bpm_range::{bpm_to_range_index, BpmRange, BpmRangeWithCount, SYSTEM_BPM_RANGES};

// ============================================================================
// VIP3 Filtering Models - Musical Key
// ============================================================================

pub use musical_key::{
    parse_key_string, MusicalKey, MusicalKeyWithCount, SYSTEM_MUSICAL_KEYS,
};

// ============================================================================
// Collection Models
// ============================================================================

pub use collection::{
    AddToCollection, Collection, CollectionFile, CollectionWithCount, CreateCollection,
    UpdateCollection, COLLECTION_COLORS, COLLECTION_ICONS,
};

// ============================================================================
// Saved Search Models
// ============================================================================

pub use saved_search::{
    BrowserFilters, CreateRecentSearch, CreateSavedSearch, RecentSearch, SavedSearch,
    UpdateSavedSearch,
};
