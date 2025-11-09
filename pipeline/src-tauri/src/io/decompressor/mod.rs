   /// Archive decompression module
   ///
   /// # Archetype: Grown-up Script
   /// - Performs I/O operations
   /// - Can be run standalone OR imported
   /// - Separates I/O from business logic

pub mod extractor;
pub mod formats;
pub mod temp_manager;

// Re-export main types
pub use extractor::{extract_archive, extract_to_temp, ExtractionConfig, ExtractionResult};
pub use formats::{detect_format, is_archive, ArchiveFormat};
