
/// Hash calculation module for file content deduplication.
///
/// This module provides BLAKE3 hashing functionality for calculating
/// file content hashes to detect and prevent duplicate files in the
/// MIDI library system.
///
/// # Architecture Pattern: Trusty Module
///
/// This module follows the **Trusty Module** pattern:
/// - Pure, stateless hash calculation functions
/// - Comprehensive test coverage
/// - No side effects (except convenience file I/O wrapper)
/// - Single responsibility: hash calculation
///
/// # Performance
///
/// BLAKE3 provides 7x performance improvement over SHA-256:
/// - Single-threaded: ~3,000 MB/s (vs SHA-256 ~400 MB/s)
/// - Multi-threaded: ~10,000 MB/s (automatic tree hashing)
///
/// # Usage
///
/// ```rust
/// use pipeline::core::hash::blake3::{calculate_content_hash, hash_to_hex};
///
/// let data = b"MIDI file content";
/// let hash = calculate_content_hash(data);
/// let hex_string = hash_to_hex(&hash);
/// ```
pub mod blake3;

// Re-export commonly used items
pub use self::blake3::{
    calculate_content_hash, calculate_file_hash, hash_to_hex, hex_to_hash, HashError, Result,
};
