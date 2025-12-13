#![allow(dead_code, unused_imports, unused_variables)]
//! Test fixtures: databases, files, and sample data
use super::database::TestDatabase;
use sqlx::PgPool;

/// Standard test fixtures
pub struct TestFixtures {
    db: TestDatabase,
}

impl TestFixtures {
    pub async fn new() -> Self {
        Self { db: TestDatabase::new().await }
    }

    /// Fixture: 100 files with varied metadata
    pub async fn standard_library() -> Self {
        Self { db: TestDatabase::with_full_dataset().await }
    }

    /// Fixture: Empty database (for insert tests)
    pub async fn empty() -> Self {
        Self::new().await
    }

    /// Fixture: 1000 files (for performance tests)
    pub async fn large_library() -> Self {
        Self { db: TestDatabase::with_files(1000).await }
    }

    pub fn pool(&self) -> &PgPool {
        self.db.pool()
    }
}

/// Filesystem fixtures (test MIDI files)
pub struct FileFixtures {
    temp_dir: tempfile::TempDir,
}

impl FileFixtures {
    pub async fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        Self { temp_dir }
    }

    /// Create a test MIDI file
    pub async fn create_midi_file(&self, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = self.temp_dir.path().join(name);
        tokio::fs::write(&path, content).await.expect("Failed to write test file");
        path
    }

    /// Create multiple test files
    pub async fn create_midi_files(&self, count: usize) -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        for i in 0..count {
            let path = self
                .create_midi_file(&format!("test_{}.mid", i), &self.simple_midi_bytes())
                .await;
            paths.push(path);
        }
        paths
    }

    /// Simple valid MIDI file bytes (120 BPM, C major)
    pub fn simple_midi_bytes(&self) -> Vec<u8> {
        // MIDI header (Format 0, 1 track, 480 ticks/beat)
        let mut bytes = vec![
            0x4D, 0x54, 0x68, 0x64, // "MThd"
            0x00, 0x00, 0x00, 0x06, // Header length
            0x00, 0x00, // Format 0
            0x00, 0x01, // 1 track
            0x01, 0xE0, // 480 ticks/beat
        ];

        // Track header
        bytes.extend_from_slice(&[
            0x4D, 0x54, 0x72, 0x6B, // "MTrk"
            0x00, 0x00, 0x00, 0x0B, // Track length (11 bytes)
        ]);

        // Track data: Tempo (120 BPM) + End of track
        bytes.extend_from_slice(&[
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Tempo: 500000 µs/beat (120 BPM)
            0x00, 0xFF, 0x2F, 0x00, // End of track
        ]);

        bytes
    }

    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

impl Default for FileFixtures {
    fn default() -> Self {
        // Can't use async in Default, so this is a convenience placeholder
        panic!("Use FileFixtures::new().await instead");
    }
}
