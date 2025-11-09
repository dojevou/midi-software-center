   /// Intelligent filename generation

pub mod generator;
pub mod sanitizer;
pub mod templates;

// Re-export main types
pub use generator::{
    generate_filename, generate_from_analysis, resolve_naming_conflict, FileMetadata, NamingConfig,
};
pub use templates::NamingTemplate;
