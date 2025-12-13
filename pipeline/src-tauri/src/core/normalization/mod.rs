/// Filename normalization utilities
///
/// # Archetype: Trusty Module
/// - Normalizes MIDI filenames for database compatibility
/// - Fixes UTF-8 encoding issues
/// - Standardizes extensions and spacing
pub mod filename;

pub use filename::{normalize_directory, NormalizationStats};
