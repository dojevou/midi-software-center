//! Common test utilities and constants
//!
//! Shared setup, constants, and utilities used across all tests.

use blake3;
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

// ============================================================================
// Test Configuration Constants
// ============================================================================

/// Default database URL for testing
pub const DEFAULT_DATABASE_URL: &str =
    "postgresql://midiuser:145278963@localhost:5433/midi_library";

/// Maximum database connections for test pool
pub const MAX_TEST_CONNECTIONS: u32 = 5;

/// Query performance threshold (milliseconds)
/// Queries exceeding this threshold should be investigated
pub const QUERY_PERFORMANCE_THRESHOLD_MS: u128 = 100;

/// Batch insert threshold for performance tests
pub const BATCH_INSERT_THRESHOLD_MS: u128 = 500;

/// Maximum file size for test files (100MB)
pub const MAX_TEST_FILE_SIZE: i64 = 100 * 1024 * 1024;

// ============================================================================
// Test Data Constants
// ============================================================================

/// Common MIDI file extensions
pub const MIDI_EXTENSIONS: &[&str] = &[".mid", ".midi", ".MID", ".MIDI"];

/// Common manufacturer names for test data
pub const TEST_MANUFACTURERS: &[&str] = &[
    "Ableton",
    "Native Instruments",
    "Spectrasonics",
    "Arturia",
    "Splice",
    "Loopmasters",
    "Vengeance",
    "Sample Magic",
];

/// Common collection names for test data
pub const TEST_COLLECTIONS: &[&str] = &[
    "Core Library",
    "The Gentleman",
    "Trilian",
    "Analog Lab",
    "Essential MIDI",
    "Studio Essentials",
    "Producer Pack",
    "Ultimate Collection",
];

/// Common musical keys for test data
pub const MUSICAL_KEYS: &[&str] = &[
    "C", "Cm", "C#", "C#m", "D", "Dm", "D#", "D#m", "E", "Em", "F", "Fm", "F#", "F#m", "G", "Gm",
    "G#", "G#m", "A", "Am", "A#", "A#m", "B", "Bm",
];

/// Common BPM values for test data
pub const COMMON_BPMS: &[f64] = &[
    80.0, 85.0, 90.0, 95.0, 100.0, 110.0, 120.0, 125.0, 128.0, 130.0, 135.0, 140.0, 150.0, 160.0,
    170.0, 174.0,
];

/// Common time signatures (numerator, denominator)
pub const TIME_SIGNATURES: &[(i16, i16)] = &[(4, 4), (3, 4), (6, 8), (5, 4), (7, 8), (2, 4)];

// ============================================================================
// Random Data Generators
// ============================================================================

/// Generate a random BLAKE3 hash (32 bytes)
///
/// Creates a cryptographically strong hash for test file deduplication.
pub fn random_hash() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let random_data: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    blake3::hash(&random_data).as_bytes().to_vec()
}

/// Generate a random filename with MIDI extension
///
/// Creates realistic MIDI filenames for testing.
///
/// # Example
///
/// ```
/// let filename = random_filename();
/// // Returns something like: "Bass_Loop_Am_120BPM.mid"
/// ```
pub fn random_filename() -> String {
    let mut rng = rand::thread_rng();

    let prefixes = &[
        "Bass", "Drum", "Lead", "Pad", "Chord", "Melody", "Arp", "FX", "Vocal", "Piano",
    ];
    let types = &["Loop", "One_Shot", "Pattern", "Progression", "Sequence", "Riff"];
    let keys = &["C", "D", "E", "F", "G", "A", "B", "Am", "Dm", "Em"];
    let bpms = &["80", "100", "120", "128", "140", "160"];

    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let type_str = types[rng.gen_range(0..types.len())];
    let key = keys[rng.gen_range(0..keys.len())];
    let bpm = bpms[rng.gen_range(0..bpms.len())];

    format!("{}_{}_{}_{}_BPM.mid", prefix, type_str, key, bpm)
}

/// Generate a random filepath
///
/// Creates realistic file paths for testing.
pub fn random_filepath() -> String {
    let filename = random_filename();
    let mut rng = rand::thread_rng();

    let categories = &["drums", "bass", "keys", "leads", "pads", "fx"];
    let manufacturers = TEST_MANUFACTURERS;

    let category = categories[rng.gen_range(0..categories.len())];
    let manufacturer = manufacturers[rng.gen_range(0..manufacturers.len())];

    format!("/library/{}/{}/{}", manufacturer, category, filename)
}

/// Generate a random UUID
pub fn random_uuid() -> Uuid {
    Uuid::new_v4()
}

/// Generate a random BPM value
pub fn random_bpm() -> f64 {
    let mut rng = rand::thread_rng();
    COMMON_BPMS[rng.gen_range(0..COMMON_BPMS.len())]
}

/// Generate a random musical key
pub fn random_key() -> String {
    let mut rng = rand::thread_rng();
    MUSICAL_KEYS[rng.gen_range(0..MUSICAL_KEYS.len())].to_string()
}

/// Generate a random time signature
pub fn random_time_signature() -> (i16, i16) {
    let mut rng = rand::thread_rng();
    TIME_SIGNATURES[rng.gen_range(0..TIME_SIGNATURES.len())]
}

/// Generate a random manufacturer name
pub fn random_manufacturer() -> String {
    let mut rng = rand::thread_rng();
    TEST_MANUFACTURERS[rng.gen_range(0..TEST_MANUFACTURERS.len())].to_string()
}

/// Generate a random collection name
pub fn random_collection() -> String {
    let mut rng = rand::thread_rng();
    TEST_COLLECTIONS[rng.gen_range(0..TEST_COLLECTIONS.len())].to_string()
}

// ============================================================================
// Database Connection Pool Management
// ============================================================================

/// Get or create a test database pool
///
/// Returns a shared connection pool for the current thread.
/// The pool is created on first access and reused for subsequent calls.
pub async fn get_test_pool() -> PgPool {
    // Note: This is a simplified version. In practice, you'd want to use
    // a proper async OnceCell or similar pattern.
    crate::helpers::db::setup_test_pool().await
}

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a batch of test hashes
///
/// Generates unique hashes for batch testing scenarios.
pub fn create_test_hashes(count: usize) -> Vec<Vec<u8>> {
    (0..count).map(|_| random_hash()).collect()
}

/// Create a batch of test filenames
///
/// Generates unique filenames for batch testing scenarios.
pub fn create_test_filenames(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("test_file_{}.mid", i)).collect()
}

/// Create a batch of test filepaths
///
/// Generates unique filepaths for batch testing scenarios.
pub fn create_test_filepaths(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("/tmp/test_file_{}.mid", i))
        .collect()
}

/// Measure database pool health
///
/// Returns statistics about the connection pool state.
pub fn measure_pool_health(pool: &PgPool) -> PoolHealth {
    PoolHealth {
        size: pool.size(),
        idle: pool.num_idle(),
        is_closed: pool.is_closed(),
    }
}

/// Pool health statistics
#[derive(Debug, Clone)]
pub struct PoolHealth {
    pub size: u32,
    pub idle: usize,
    pub is_closed: bool,
}

impl PoolHealth {
    /// Check if pool is healthy
    pub fn is_healthy(&self) -> bool {
        !self.is_closed && self.size > 0
    }

    /// Check if pool has available connections
    pub fn has_available_connections(&self) -> bool {
        self.idle > 0
    }
}

// ============================================================================
// Test Data Validation
// ============================================================================

/// Validate a BLAKE3 hash
pub fn is_valid_hash(hash: &[u8]) -> bool {
    hash.len() == 32
}

/// Validate a filename
pub fn is_valid_filename(filename: &str) -> bool {
    !filename.is_empty()
        && filename.len() <= 255
        && !filename.contains('\0')
        && !filename.contains('/')
}

/// Validate a filepath
pub fn is_valid_filepath(filepath: &str) -> bool {
    !filepath.is_empty() && filepath.len() <= 4096 && !filepath.contains('\0')
}

/// Validate a BPM value
pub fn is_valid_bpm(bpm: f64) -> bool {
    (20.0..=300.0).contains(&bpm)
}

/// Validate a MIDI pitch value
pub fn is_valid_pitch(pitch: i16) -> bool {
    (0..=127).contains(&pitch)
}

// ============================================================================
// Performance Testing Utilities
// ============================================================================

/// Simple performance timer
pub struct PerfTimer {
    start: std::time::Instant,
    label: String,
}

impl PerfTimer {
    /// Create a new performance timer
    pub fn new(label: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
            label: label.to_string(),
        }
    }

    /// Stop the timer and print elapsed time
    pub fn stop(self) -> u128 {
        let elapsed = self.start.elapsed().as_millis();
        println!("⏱️  {} took {}ms", self.label, elapsed);
        elapsed
    }

    /// Get elapsed time without stopping
    pub fn elapsed(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_hash() {
        let hash1 = random_hash();
        let hash2 = random_hash();

        assert_eq!(hash1.len(), 32);
        assert_eq!(hash2.len(), 32);
        assert_ne!(hash1, hash2); // Should be unique
    }

    #[test]
    fn test_random_filename() {
        let filename = random_filename();
        assert!(filename.ends_with(".mid"));
        assert!(!filename.is_empty());
    }

    #[test]
    fn test_random_filepath() {
        let filepath = random_filepath();
        assert!(filepath.starts_with("/library/"));
        assert!(filepath.ends_with(".mid"));
    }

    #[test]
    fn test_random_bpm() {
        let bpm = random_bpm();
        assert!(COMMON_BPMS.contains(&bpm));
        assert!(is_valid_bpm(bpm));
    }

    #[test]
    fn test_random_key() {
        let key = random_key();
        assert!(MUSICAL_KEYS.contains(&key.as_str()));
    }

    #[test]
    fn test_create_test_hashes() {
        let hashes = create_test_hashes(10);
        assert_eq!(hashes.len(), 10);
        assert!(hashes.iter().all(|h| h.len() == 32));

        // All should be unique
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j]);
            }
        }
    }

    #[test]
    fn test_create_test_filenames() {
        let filenames = create_test_filenames(5);
        assert_eq!(filenames.len(), 5);
        assert_eq!(filenames[0], "test_file_0.mid");
        assert_eq!(filenames[4], "test_file_4.mid");
    }

    #[test]
    fn test_is_valid_hash() {
        let valid_hash = vec![0u8; 32];
        let invalid_hash = vec![0u8; 16];

        assert!(is_valid_hash(&valid_hash));
        assert!(!is_valid_hash(&invalid_hash));
    }

    #[test]
    fn test_is_valid_filename() {
        assert!(is_valid_filename("test.mid"));
        assert!(!is_valid_filename(""));
        assert!(!is_valid_filename("invalid/path.mid"));
        assert!(!is_valid_filename("invalid\0null.mid"));
    }

    #[test]
    fn test_is_valid_filepath() {
        assert!(is_valid_filepath("/tmp/test.mid"));
        assert!(!is_valid_filepath(""));
        assert!(!is_valid_filepath("invalid\0null.mid"));
    }

    #[test]
    fn test_is_valid_bpm() {
        assert!(is_valid_bpm(120.0));
        assert!(is_valid_bpm(20.0));
        assert!(is_valid_bpm(300.0));
        assert!(!is_valid_bpm(19.9));
        assert!(!is_valid_bpm(300.1));
    }

    #[test]
    fn test_is_valid_pitch() {
        assert!(is_valid_pitch(0));
        assert!(is_valid_pitch(60));
        assert!(is_valid_pitch(127));
        assert!(!is_valid_pitch(-1));
        assert!(!is_valid_pitch(128));
    }

    #[test]
    fn test_pool_health() {
        let health = PoolHealth {
            size: 5,
            idle: 3,
            is_closed: false,
        };

        assert!(health.is_healthy());
        assert!(health.has_available_connections());

        let unhealthy = PoolHealth {
            size: 0,
            idle: 0,
            is_closed: true,
        };

        assert!(!unhealthy.is_healthy());
        assert!(!unhealthy.has_available_connections());
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::new("test operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.stop();
        assert!(elapsed >= 10);
    }
}
