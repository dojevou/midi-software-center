//! Database repository layer
//!
//! Provides repository types for database operations:
//! - `FileRepository` - CRUD for MIDI files with deduplication
//! - `MetadataRepository` - Musical metadata operations
//! - `SearchRepository` - Full-text and filtered search
//! - `TagRepository` - Tag management and file relationships
//! - `TimbreRepository` - Sound character categories (VIP3 filtering)
//! - `StyleRepository` - Musical genres (VIP3 filtering)
//! - `ArticulationRepository` - Playing styles/pattern types (VIP3 filtering)
//! - `CollectionRepository` - User collections/playlists
//! - `SavedSearchRepository` - Saved filter configurations
//! - `pipeline` - Pipeline-specific repositories (for CLI tools)

pub mod articulation_repository;
pub mod collection_repository;
pub mod file_repository;
pub mod metadata_repository;
pub mod pipeline;
pub mod saved_search_repository;
pub mod search_repository;
pub mod style_repository;
pub mod tag_repository;
pub mod timbre_repository;

// Re-export repository types
pub use file_repository::FileRepository;
pub use metadata_repository::{MetadataRepository, MetadataStatistics};
pub use search_repository::SearchRepository;
pub use tag_repository::{CategoryStats, TagRepository};

// VIP3 Filtering Repositories
pub use articulation_repository::ArticulationRepository;
pub use collection_repository::CollectionRepository;
pub use saved_search_repository::SavedSearchRepository;
pub use style_repository::StyleRepository;
pub use timbre_repository::TimbreRepository;
