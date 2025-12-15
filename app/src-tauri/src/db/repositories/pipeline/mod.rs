//! Pipeline-specific database repositories
//!
//! These repositories are used by CLI tools and batch processing.
//! They use the pipeline models which are compatible with BigDecimal types.

pub mod file_repository;
pub mod metadata_repository;
pub mod search_repository;
pub mod tag_repository;

// Re-export with prefixed names to avoid conflicts
pub use file_repository::FileRepository as PipelineFileRepository;
pub use metadata_repository::MetadataRepository as PipelineMetadataRepository;
pub use search_repository::{SearchQuery as PipelineSearchQuery, SearchRepository as PipelineSearchRepository};
pub use tag_repository::{DbTag, TagRepository as PipelineTagRepository, TagWithCount as PipelineTagWithCount};
