/// BLAKE3 hashing module for file content deduplication and integrity verification.
///
/// This is a **Trusty Module** with pure hashing logic.
///
/// # Architecture Pattern
///
/// Core functions are pure (no I/O):
/// - `calculate_content_hash()` - Pure hash calculation
/// - `hash_to_hex()` - Pure conversion
///
/// Convenience wrapper (does I/O):
/// - `calculate_file_hash()` - Reads file and calculates hash
///
/// # Performance
///
/// BLAKE3 provides significant performance improvements over SHA-256:
/// - **Single-threaded**: ~3,000 MB/s (vs SHA-256 ~400 MB/s)
/// - **Multi-threaded**: ~10,000 MB/s with parallel tree hashing
/// - **7x faster** than SHA-256 for typical file sizes
///
/// # Examples
///
/// ```rust
/// use pipeline::core::hash::blake3::{calculate_content_hash, hash_to_hex};
///
/// let data = b"Hello, MIDI Library System!";
/// let hash = calculate_content_hash(data);
/// let hex_string = hash_to_hex(&hash);
/// println!("Hash: {}", hex_string);
/// ```
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use thiserror::Error;

/// Hash calculation errors
#[derive(Error, Debug)]
pub enum HashError {
    /// File could not be opened or read
    #[error("Failed to read file: {0}")]
    IoError(#[from] io::Error),

    /// File path is invalid
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

pub type Result<T> = std::result::Result<T, HashError>;

/// Calculate BLAKE3 hash of byte content.
///
/// This is a **pure function** with no side effects (TRUSTY MODULE pattern).
///
/// # Arguments
///
/// * `data` - Byte slice to hash
///
/// # Returns
///
/// 32-byte BLAKE3 hash
///
/// # Performance
///
/// - Single-threaded: ~3,000 MB/s
/// - For data larger than 128 KB, BLAKE3 automatically uses parallel tree hashing
/// - Significantly faster than SHA-256 (~400 MB/s)
///
/// # Examples
///
/// ```rust
/// use pipeline::core::hash::blake3::calculate_content_hash;
///
/// let data = b"MIDI file content";
/// let hash = calculate_content_hash(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn calculate_content_hash(data: &[u8]) -> [u8; 32] {
    // BLAKE3 uses parallel tree hashing automatically for large inputs
    // This provides multi-threaded performance without explicit parallelism
    blake3::hash(data).into()
}

/// Calculate BLAKE3 hash of a file.
///
/// This is a **convenience wrapper** that performs file I/O.
/// For pure hashing logic, use `calculate_content_hash()`.
///
/// # Arguments
///
/// * `path` - Path to file to hash
///
/// # Returns
///
/// 32-byte BLAKE3 hash or error if file cannot be read
///
/// # Errors
///
/// - `HashError::IoError` - File cannot be opened or read
/// - `HashError::InvalidPath` - Path is invalid or does not exist
///
/// # Performance
///
/// For large files (>10 MB), consider using memory-mapped files for better performance.
/// This implementation reads the file into memory, which is optimal for files <100 MB.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
/// use pipeline::core::hash::blake3::calculate_file_hash;
///
/// let path = Path::new("test.mid");
/// let hash = calculate_file_hash(path)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn calculate_file_hash(path: &Path) -> Result<[u8; 32]> {
    // Validate path
    if !path.exists() {
        return Err(HashError::InvalidPath(format!(
            "File does not exist: {}",
            path.display()
        )));
    }

    if !path.is_file() {
        return Err(HashError::InvalidPath(format!(
            "Path is not a file: {}",
            path.display()
        )));
    }

    // Open file
    let mut file = File::open(path)?;

    // For small to medium files (<100 MB), read entire file into memory
    // This is fastest approach for most MIDI files which are typically <10 MB
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Use pure hash function
    Ok(calculate_content_hash(&buffer))
}

/// Convert 32-byte hash to hexadecimal string.
///
/// This is a **pure function** with no side effects (TRUSTY MODULE pattern).
///
/// # Arguments
///
/// * `hash` - 32-byte hash to convert
///
/// # Returns
///
/// 64-character lowercase hexadecimal string
///
/// # Examples
///
/// ```rust
/// use pipeline::core::hash::blake3::{calculate_content_hash, hash_to_hex};
///
/// let data = b"test";
/// let hash = calculate_content_hash(data);
/// let hex = hash_to_hex(&hash);
///
/// assert_eq!(hex.len(), 64); // 32 bytes = 64 hex characters
/// assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
pub fn hash_to_hex(hash: &[u8; 32]) -> String {
    // Use blake3's built-in hex encoding for efficiency
    blake3::Hash::from(*hash).to_hex().to_string()
}

/// Convert hexadecimal string back to 32-byte hash.
///
/// This is a **pure function** with no side effects (TRUSTY MODULE pattern).
///
/// # Arguments
///
/// * `hex` - 64-character hexadecimal string
///
/// # Returns
///
/// 32-byte hash or error if hex string is invalid
///
/// # Errors
///
/// Returns error if:
/// - String is not exactly 64 characters
/// - String contains non-hexadecimal characters
///
/// # Examples
///
/// ```rust
/// use pipeline::core::hash::blake3::{hash_to_hex, hex_to_hash};
///
/// let original = [0u8; 32];
/// let hex = hash_to_hex(&original);
/// let decoded = hex_to_hash(&hex)?;
/// assert_eq!(original, decoded);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn hex_to_hash(hex: &str) -> Result<[u8; 32]> {
    if hex.len() != 64 {
        return Err(HashError::InvalidPath(format!(
            "Hex string must be exactly 64 characters, got {}",
            hex.len()
        )));
    }

    let mut hash = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex[i * 2..i * 2 + 2];
        hash[i] = u8::from_str_radix(byte_str, 16).map_err(|_| {
            HashError::InvalidPath(format!("Invalid hex character in string: {}", byte_str))
        })?;
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_calculate_content_hash_empty() {
        let data = b"";
        let hash = calculate_content_hash(data);

        // BLAKE3 hash of empty string (known value)
        let expected =
            hex_to_hash("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262")
                .unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_calculate_content_hash_hello_world() {
        let data = b"Hello, World!";
        let hash = calculate_content_hash(data);

        // BLAKE3 hash of "Hello, World!" (verified with blake3 crate)
        let expected =
            hex_to_hash("288a86a79f20a3d6dccdca7713beaed178798296bdfa7913fa2a62d9727bf8f8")
                .unwrap();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_calculate_content_hash_consistency() {
        let data = b"Consistent hashing test";

        let hash1 = calculate_content_hash(data);
        let hash2 = calculate_content_hash(data);
        let hash3 = calculate_content_hash(data);

        // Same input must always produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_calculate_content_hash_different_inputs() {
        let data1 = b"First input";
        let data2 = b"Second input";

        let hash1 = calculate_content_hash(data1);
        let hash2 = calculate_content_hash(data2);

        // Different inputs must produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_calculate_content_hash_large_data() {
        // Test with 1 MB of data (triggers parallel tree hashing)
        let data = vec![0xAB; 1_000_000];
        let hash = calculate_content_hash(&data);

        // Just verify we get a valid hash
        assert_eq!(hash.len(), 32);

        // Verify consistency
        let hash2 = calculate_content_hash(&data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_to_hex() {
        let hash = [0u8; 32]; // All zeros
        let hex = hash_to_hex(&hash);

        assert_eq!(hex.len(), 64);
        assert_eq!(hex, "0".repeat(64));
    }

    #[test]
    fn test_hash_to_hex_mixed_values() {
        let hash = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98,
            0x76, 0x54, 0x32, 0x10,
        ];
        let hex = hash_to_hex(&hash);

        assert_eq!(hex.len(), 64);
        assert_eq!(
            hex,
            "0123456789abcdeffedcba9876543210\
             0123456789abcdeffedcba9876543210"
        );
    }

    #[test]
    fn test_hex_to_hash_valid() {
        let hex = "0123456789abcdef\
                   fedcba9876543210\
                   0123456789abcdef\
                   fedcba9876543210";
        let hash = hex_to_hash(hex).unwrap();

        let expected = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98,
            0x76, 0x54, 0x32, 0x10,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hex_to_hash_roundtrip() {
        let original = [0xAB; 32];
        let hex = hash_to_hex(&original);
        let decoded = hex_to_hash(&hex).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_hex_to_hash_invalid_length() {
        let hex = "too_short";
        let result = hex_to_hash(hex);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("64 characters"));
    }

    #[test]
    fn test_hex_to_hash_invalid_characters() {
        let hex = "0123456789abcdefg123456789abcdef0123456789abcdef0123456789abcdef"; // 'g' is invalid
        let result = hex_to_hash(hex);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_file_hash_nonexistent() {
        let path = Path::new("/nonexistent/file.mid");
        let result = calculate_file_hash(path);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, HashError::InvalidPath(_)));
    }

    #[test]
    fn test_calculate_file_hash_directory() {
        // Try to hash a directory (should fail)
        let path = Path::new("/tmp");
        let result = calculate_file_hash(path);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_file_hash_real_file() {
        // Create temporary test file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("blake3_test.txt");

        // Write test data
        let test_data = b"Test MIDI file content for hashing";
        fs::write(&test_file, test_data).unwrap();

        // Calculate hash from file
        let file_hash = calculate_file_hash(&test_file).unwrap();

        // Calculate hash from data directly
        let content_hash = calculate_content_hash(test_data);

        // Should match
        assert_eq!(file_hash, content_hash);

        // Cleanup
        fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_calculate_file_hash_consistency() {
        // Create temporary test file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("blake3_consistency_test.txt");

        // Write test data
        let test_data = b"Consistency test for file hashing";
        fs::write(&test_file, test_data).unwrap();

        // Calculate hash multiple times
        let hash1 = calculate_file_hash(&test_file).unwrap();
        let hash2 = calculate_file_hash(&test_file).unwrap();
        let hash3 = calculate_file_hash(&test_file).unwrap();

        // All hashes must match
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);

        // Cleanup
        fs::remove_file(&test_file).unwrap();
    }

    #[test]
    fn test_integration_full_workflow() {
        // Test the complete workflow: data -> hash -> hex -> hash -> verify
        let original_data = b"Complete integration test for BLAKE3 hashing";

        // Step 1: Calculate hash
        let hash = calculate_content_hash(original_data);
        assert_eq!(hash.len(), 32);

        // Step 2: Convert to hex
        let hex = hash_to_hex(&hash);
        assert_eq!(hex.len(), 64);

        // Step 3: Convert back to hash
        let decoded_hash = hex_to_hash(&hex).unwrap();
        assert_eq!(hash, decoded_hash);

        // Step 4: Verify hash matches content
        let verification_hash = calculate_content_hash(original_data);
        assert_eq!(hash, verification_hash);
    }

    #[test]
    fn test_hash_collision_resistance() {
        // Test that very similar inputs produce different hashes
        let data1 = b"test1";
        let data2 = b"test2";

        let hash1 = calculate_content_hash(data1);
        let hash2 = calculate_content_hash(data2);

        // Even single bit difference should produce completely different hash
        assert_ne!(hash1, hash2);

        // Count number of different bytes (should be high due to avalanche effect)
        let differences = hash1.iter().zip(hash2.iter()).filter(|(a, b)| a != b).count();

        // Expect at least 50% of bytes to be different (avalanche effect)
        assert!(differences > 16, "Only {} bytes different", differences);
    }
}
