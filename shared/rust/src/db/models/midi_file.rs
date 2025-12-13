//! MIDI file database model
//!
//! Represents the core `files` table in the database schema.
//! Designed for 3,000,000+ MIDI files at production scale.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core MIDI file record matching the `files` database table.
///
/// Uses `BIGSERIAL` (i64) for primary key to support millions of files.
/// Includes multi-track handling and search optimization fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct MidiFile {
    /// Primary key (BIGSERIAL)
    pub id: i64,

    /// Sanitized filename (without path)
    pub filename: String,

    /// Full file path (unique constraint)
    pub filepath: String,

    /// Original filename before sanitization
    pub original_filename: String,

    /// BLAKE3 content hash for deduplication (32 bytes)
    pub content_hash: Vec<u8>,

    /// File size in bytes
    pub file_size_bytes: i64,

    /// MIDI format (0, 1, or 2)
    pub format: Option<i16>,

    /// Number of tracks in the file
    pub num_tracks: i16,

    /// Ticks per quarter note (MIDI resolution)
    pub ticks_per_quarter_note: Option<i32>,

    /// Duration in seconds
    pub duration_seconds: Option<rust_decimal::Decimal>,

    /// Duration in MIDI ticks
    pub duration_ticks: Option<i64>,

    /// Whether this file has multiple tracks that were split
    pub is_multi_track: Option<bool>,

    /// Parent file ID if this is a split track
    pub parent_file_id: Option<i64>,

    /// Track number within parent (if split)
    pub track_number: Option<i16>,

    /// Total tracks in parent (if split)
    pub total_tracks: Option<i16>,

    /// Extracted manufacturer/source
    pub manufacturer: Option<String>,

    /// Collection or pack name
    pub collection_name: Option<String>,

    /// Tags extracted from folder path
    #[cfg_attr(feature = "database", sqlx(default))]
    pub folder_tags: Option<Vec<String>>,

    /// Import batch identifier for tracking
    pub import_batch_id: Option<Uuid>,

    /// When the file was added to the database
    pub created_at: Option<DateTime<Utc>>,

    /// When the file record was last updated
    pub updated_at: Option<DateTime<Utc>>,

    /// When analysis was last performed
    pub analyzed_at: Option<DateTime<Utc>>,
}

/// Lightweight file reference for listings and searches.
///
/// Used when full file data isn't needed (performance optimization).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct FileRef {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
}

/// Data required to create a new MIDI file record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMidiFile {
    pub filename: String,
    pub filepath: String,
    pub original_filename: String,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,
    pub format: Option<i16>,
    pub num_tracks: i16,
    pub ticks_per_quarter_note: Option<i32>,
    pub import_batch_id: Option<Uuid>,
}

/// Optional fields for updating an existing MIDI file record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateMidiFile {
    pub filename: Option<String>,
    pub filepath: Option<String>,
    pub format: Option<i16>,
    pub num_tracks: Option<i16>,
    pub ticks_per_quarter_note: Option<i32>,
    pub duration_seconds: Option<rust_decimal::Decimal>,
    pub duration_ticks: Option<i64>,
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub folder_tags: Option<Vec<String>>,
    pub analyzed_at: Option<DateTime<Utc>>,
}

/// File with split track information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct SplitFileInfo {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub parent_file_id: Option<i64>,
    pub track_number: Option<i16>,
    pub total_tracks: Option<i16>,
    pub is_multi_track: Option<bool>,
}

/// Import batch tracking for bulk operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct ImportBatch {
    pub id: Uuid,
    pub source_directory: String,
    pub total_files: i32,
    pub processed_files: i32,
    pub failed_files: i32,
    pub skipped_files: i32,
    pub status: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

impl MidiFile {
    /// Get the content hash as a hex string.
    #[must_use]
    pub fn hash_hex(&self) -> String {
        hex::encode(&self.content_hash)
    }

    /// Check if this file is a split track from a multi-track parent.
    #[must_use]
    pub fn is_split_track(&self) -> bool {
        self.parent_file_id.is_some()
    }

    /// Get the file extension (lowercase).
    #[must_use]
    pub fn extension(&self) -> Option<String> {
        std::path::Path::new(&self.filename)
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
    }
}

impl CreateMidiFile {
    /// Create a new file creation request with minimal required fields.
    #[must_use]
    pub fn new(
        filename: String,
        filepath: String,
        content_hash: Vec<u8>,
        file_size_bytes: i64,
    ) -> Self {
        Self {
            original_filename: filename.clone(),
            filename,
            filepath,
            content_hash,
            file_size_bytes,
            format: None,
            num_tracks: 1,
            ticks_per_quarter_note: None,
            import_batch_id: None,
        }
    }

    /// Set the import batch ID.
    #[must_use]
    pub fn with_batch(mut self, batch_id: Uuid) -> Self {
        self.import_batch_id = Some(batch_id);
        self
    }

    /// Set MIDI format information.
    #[must_use]
    pub fn with_format(mut self, format: i16, num_tracks: i16, tpqn: i32) -> Self {
        self.format = Some(format);
        self.num_tracks = num_tracks;
        self.ticks_per_quarter_note = Some(tpqn);
        self
    }
}

// Legacy alias for backwards compatibility
pub type File = MidiFile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_midi_file() {
        let create = CreateMidiFile::new(
            "test.mid".to_string(),
            "/path/to/test.mid".to_string(),
            vec![0u8; 32],
            1024,
        );

        assert_eq!(create.filename, "test.mid");
        assert_eq!(create.file_size_bytes, 1024);
        assert_eq!(create.num_tracks, 1);
    }

    #[test]
    fn test_create_with_batch() {
        let batch_id = Uuid::new_v4();
        let create = CreateMidiFile::new(
            "test.mid".to_string(),
            "/path/to/test.mid".to_string(),
            vec![0u8; 32],
            1024,
        )
        .with_batch(batch_id);

        assert_eq!(create.import_batch_id, Some(batch_id));
    }

    #[test]
    fn test_extension() {
        let file = MidiFile {
            id: 1,
            filename: "Test_Song.MIDI".to_string(),
            filepath: "/path/to/Test_Song.MIDI".to_string(),
            original_filename: "Test Song.MIDI".to_string(),
            content_hash: vec![0u8; 32],
            file_size_bytes: 1024,
            format: Some(1),
            num_tracks: 4,
            ticks_per_quarter_note: Some(480),
            duration_seconds: None,
            duration_ticks: None,
            is_multi_track: Some(true),
            parent_file_id: None,
            track_number: None,
            total_tracks: Some(4),
            manufacturer: None,
            collection_name: None,
            folder_tags: None,
            import_batch_id: None,
            created_at: None,
            updated_at: None,
            analyzed_at: None,
        };

        assert_eq!(file.extension(), Some("midi".to_string()));
        assert!(!file.is_split_track());
    }
}
