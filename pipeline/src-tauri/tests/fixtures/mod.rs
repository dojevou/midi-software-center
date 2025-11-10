#[allow(dead_code, unused_imports, unused_variables)]
#[allow(dead_code, unused_imports, unused_variables)]
/// Test fixtures for database testing
///
/// Provides builder patterns for creating realistic test data for MIDI files,
/// musical metadata, tags, and other database entities.
use sqlx::types::BigDecimal;
use std::str::FromStr;

// Import actual models from the crate
use midi_pipeline::db::models::NewFile;

// ============================================================================
// File Fixtures
// ============================================================================

/// Builder for creating test MIDI file records
///
/// Provides sensible defaults for all fields with the ability to override
/// specific values for test scenarios.
pub struct NewFileBuilder {
    filename: String,
    filepath: String,
    original_filename: String,
    content_hash: Vec<u8>,
    file_size_bytes: i64,
    format: Option<i16>,
    num_tracks: i16,
    ticks_per_quarter_note: Option<i32>,
    duration_seconds: Option<BigDecimal>,
    duration_ticks: Option<i64>,
    manufacturer: Option<String>,
    collection_name: Option<String>,
    folder_tags: Option<Vec<String>>,
    import_batch_id: Option<uuid::Uuid>,
}

impl NewFileBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            filename: "test.mid".to_string(),
            filepath: "/tmp/test.mid".to_string(),
            original_filename: "test.mid".to_string(),
            content_hash: vec![0u8; 32], // 32-byte hash
            file_size_bytes: 1024,
            format: Some(1),
            num_tracks: 1,
            ticks_per_quarter_note: Some(480),
            duration_seconds: Some(BigDecimal::from_str("10.0").expect("Valid decimal")),
            duration_ticks: Some(4800),
            manufacturer: None,
            collection_name: None,
            folder_tags: None,
            import_batch_id: None,
        }
    }

    /// Set the filename
    pub fn filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }

    /// Set the filepath
    pub fn filepath(mut self, filepath: &str) -> Self {
        self.filepath = filepath.to_string();
        self
    }

    /// Set the original filename
    pub fn original_filename(mut self, original_filename: &str) -> Self {
        self.original_filename = original_filename.to_string();
        self
    }

    /// Set the content hash
    pub fn content_hash(mut self, hash: Vec<u8>) -> Self {
        self.content_hash = hash;
        self
    }

    /// Set the file size
    pub fn file_size_bytes(mut self, size: i64) -> Self {
        self.file_size_bytes = size;
        self
    }

    /// Set the MIDI format (0, 1, or 2)
    pub fn format(mut self, format: i16) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the number of tracks
    pub fn num_tracks(mut self, num_tracks: i16) -> Self {
        self.num_tracks = num_tracks;
        self
    }

    /// Set ticks per quarter note
    pub fn ticks_per_quarter_note(mut self, ticks: i32) -> Self {
        self.ticks_per_quarter_note = Some(ticks);
        self
    }

    /// Set duration in seconds
    pub fn duration_seconds(mut self, duration: f64) -> Self {
        self.duration_seconds = BigDecimal::from_str(&duration.to_string()).ok();
        self
    }

    /// Set duration in ticks
    pub fn duration_ticks(mut self, ticks: i64) -> Self {
        self.duration_ticks = Some(ticks);
        self
    }

    /// Set manufacturer
    pub fn manufacturer(mut self, manufacturer: &str) -> Self {
        self.manufacturer = Some(manufacturer.to_string());
        self
    }

    /// Set collection name
    pub fn collection_name(mut self, collection: &str) -> Self {
        self.collection_name = Some(collection.to_string());
        self
    }

    /// Set folder tags
    pub fn folder_tags(mut self, tags: Vec<String>) -> Self {
        self.folder_tags = Some(tags);
        self
    }

    /// Set import batch ID
    pub fn import_batch_id(mut self, batch_id: uuid::Uuid) -> Self {
        self.import_batch_id = Some(batch_id);
        self
    }

    /// Build the file record (returns NewFile for INSERT)
    pub fn build(self) -> NewFile {
        NewFile {
            filename: self.filename,
            filepath: self.filepath,
            original_filename: self.original_filename,
            content_hash: self.content_hash,
            file_size_bytes: self.file_size_bytes,
            format: self.format,
            num_tracks: self.num_tracks,
            ticks_per_quarter_note: self.ticks_per_quarter_note,
            duration_seconds: self.duration_seconds,
            duration_ticks: self.duration_ticks,
            manufacturer: self.manufacturer,
            collection_name: self.collection_name,
            folder_tags: self.folder_tags,
            import_batch_id: self.import_batch_id,
        }
    }
}

impl Default for NewFileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Musical Metadata Fixtures
// ============================================================================

/// Builder for creating test musical metadata records
pub struct NewMusicalMetadataBuilder {
    file_id: i64,
    bpm: Option<BigDecimal>,
    bpm_confidence: Option<f32>,
    has_tempo_changes: bool,
    tempo_changes: Option<serde_json::Value>,
    key_signature: Option<String>,
    key_confidence: Option<f32>,
    has_key_changes: bool,
    key_changes: Option<serde_json::Value>,
    time_signature_numerator: i16,
    time_signature_denominator: i16,
    has_time_signature_changes: bool,
    time_signature_changes: Option<serde_json::Value>,
    total_notes: i32,
    unique_pitches: Option<i32>,
    pitch_range_min: Option<i16>,
    pitch_range_max: Option<i16>,
    avg_velocity: Option<BigDecimal>,
    note_density: Option<BigDecimal>,
    polyphony_max: Option<i16>,
    polyphony_avg: Option<BigDecimal>,
    is_monophonic: bool,
    is_polyphonic: bool,
    is_percussive: bool,
    has_chords: bool,
    chord_complexity: Option<f32>,
    has_melody: bool,
    melodic_range: Option<i16>,
}

impl NewMusicalMetadataBuilder {
    /// Create a new builder with default values for a given file_id
    pub fn new(file_id: i64) -> Self {
        Self {
            file_id,
            bpm: Some(BigDecimal::from_str("120.0").expect("Valid decimal")),
            bpm_confidence: Some(0.85),
            has_tempo_changes: false,
            tempo_changes: None,
            key_signature: Some("C".to_string()),
            key_confidence: Some(0.90),
            has_key_changes: false,
            key_changes: None,
            time_signature_numerator: 4,
            time_signature_denominator: 4,
            has_time_signature_changes: false,
            time_signature_changes: None,
            total_notes: 100,
            unique_pitches: Some(12),
            pitch_range_min: Some(60),
            pitch_range_max: Some(84),
            avg_velocity: Some(BigDecimal::from_str("80.0").expect("Valid decimal")),
            note_density: Some(BigDecimal::from_str("5.5").expect("Valid decimal")),
            polyphony_max: Some(4),
            polyphony_avg: Some(BigDecimal::from_str("2.5").expect("Valid decimal")),
            is_monophonic: false,
            is_polyphonic: true,
            is_percussive: false,
            has_chords: true,
            chord_complexity: Some(0.6),
            has_melody: true,
            melodic_range: Some(24),
        }
    }

    /// Set BPM
    pub fn bpm(mut self, bpm: f64, confidence: f32) -> Self {
        self.bpm = BigDecimal::from_str(&bpm.to_string()).ok();
        self.bpm_confidence = Some(confidence);
        self
    }

    /// Set key signature
    pub fn key_signature(mut self, key: &str, confidence: f32) -> Self {
        self.key_signature = Some(key.to_string());
        self.key_confidence = Some(confidence);
        self
    }

    /// Set time signature
    pub fn time_signature(mut self, numerator: i16, denominator: i16) -> Self {
        self.time_signature_numerator = numerator;
        self.time_signature_denominator = denominator;
        self
    }

    /// Set note statistics
    pub fn notes(mut self, total: i32, unique_pitches: i32, min: i16, max: i16) -> Self {
        self.total_notes = total;
        self.unique_pitches = Some(unique_pitches);
        self.pitch_range_min = Some(min);
        self.pitch_range_max = Some(max);
        self
    }

    /// Set polyphony
    pub fn polyphony(mut self, max: i16, avg: f64) -> Self {
        self.polyphony_max = Some(max);
        self.polyphony_avg = BigDecimal::from_str(&avg.to_string()).ok();
        self
    }

    /// Set as monophonic
    pub fn monophonic(mut self) -> Self {
        self.is_monophonic = true;
        self.is_polyphonic = false;
        self.polyphony_max = Some(1);
        self.polyphony_avg = Some(BigDecimal::from_str("1.0").expect("Valid decimal"));
        self
    }

    /// Set as percussive
    pub fn percussive(mut self) -> Self {
        self.is_percussive = true;
        self
    }

    /// Set chord properties
    pub fn chords(mut self, has_chords: bool, complexity: f32) -> Self {
        self.has_chords = has_chords;
        self.chord_complexity = Some(complexity);
        self
    }

    /// Set melody properties
    pub fn melody(mut self, has_melody: bool, range: i16) -> Self {
        self.has_melody = has_melody;
        self.melodic_range = Some(range);
        self
    }

    /// Build the metadata record
    pub fn build(self) -> NewMusicalMetadata {
        NewMusicalMetadata {
            file_id: self.file_id,
            bpm: self.bpm,
            bpm_confidence: self.bpm_confidence,
            has_tempo_changes: self.has_tempo_changes,
            tempo_changes: self.tempo_changes,
            key_signature: self.key_signature,
            key_confidence: self.key_confidence,
            has_key_changes: self.has_key_changes,
            key_changes: self.key_changes,
            time_signature_numerator: self.time_signature_numerator,
            time_signature_denominator: self.time_signature_denominator,
            has_time_signature_changes: self.has_time_signature_changes,
            time_signature_changes: self.time_signature_changes,
            total_notes: self.total_notes,
            unique_pitches: self.unique_pitches,
            pitch_range_min: self.pitch_range_min,
            pitch_range_max: self.pitch_range_max,
            avg_velocity: self.avg_velocity,
            note_density: self.note_density,
            polyphony_max: self.polyphony_max,
            polyphony_avg: self.polyphony_avg,
            is_monophonic: self.is_monophonic,
            is_polyphonic: self.is_polyphonic,
            is_percussive: self.is_percussive,
            has_chords: self.has_chords,
            chord_complexity: self.chord_complexity,
            has_melody: self.has_melody,
            melodic_range: self.melodic_range,
        }
    }
}

/// Represents a new musical metadata record to be inserted
#[derive(Debug, Clone)]
pub struct NewMusicalMetadata {
    pub file_id: i64,
    pub bpm: Option<BigDecimal>,
    pub bpm_confidence: Option<f32>,
    pub has_tempo_changes: bool,
    pub tempo_changes: Option<serde_json::Value>,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f32>,
    pub has_key_changes: bool,
    pub key_changes: Option<serde_json::Value>,
    pub time_signature_numerator: i16,
    pub time_signature_denominator: i16,
    pub has_time_signature_changes: bool,
    pub time_signature_changes: Option<serde_json::Value>,
    pub total_notes: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_min: Option<i16>,
    pub pitch_range_max: Option<i16>,
    pub avg_velocity: Option<BigDecimal>,
    pub note_density: Option<BigDecimal>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<BigDecimal>,
    pub is_monophonic: bool,
    pub is_polyphonic: bool,
    pub is_percussive: bool,
    pub has_chords: bool,
    pub chord_complexity: Option<f32>,
    pub has_melody: bool,
    pub melodic_range: Option<i16>,
}

// ============================================================================
// Tag Fixtures
// ============================================================================

/// Builder for creating test tag records
pub struct NewTagBuilder {
    name: String,
    category: Option<String>,
}

impl NewTagBuilder {
    /// Create a new tag builder
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), category: None }
    }

    /// Set the category
    pub fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Build the tag record
    pub fn build(self) -> NewTag {
        NewTag { name: self.name, category: self.category }
    }
}

/// Represents a new tag record to be inserted
#[derive(Debug, Clone)]
pub struct NewTag {
    pub name: String,
    pub category: Option<String>,
}

// ============================================================================
// Preset Fixtures (Realistic Test Data)
// ============================================================================

/// Provides realistic preset fixtures for common test scenarios
pub struct Fixtures;

impl Fixtures {
    /// Create a realistic drum loop file
    pub fn drum_loop() -> NewFile {
        NewFileBuilder::new()
            .filename("Drum_Loop_120BPM.mid")
            .filepath("/samples/drums/Drum_Loop_120BPM.mid")
            .original_filename("Drum Loop 120 BPM.mid")
            .content_hash(vec![1u8; 32]) // Unique hash for drum loop
            .file_size_bytes(2048)
            .duration_seconds(8.0)
            .duration_ticks(3840)
            .manufacturer("Ableton")
            .collection_name("Core Library")
            .folder_tags(vec!["drums".to_string(), "loops".to_string()])
            .build()
    }

    /// Create metadata for a drum loop
    pub fn drum_loop_metadata(file_id: i64) -> NewMusicalMetadata {
        NewMusicalMetadataBuilder::new(file_id)
            .bpm(120.0, 0.99)
            .key_signature("C", 0.5) // Drums have weak key
            .notes(256, 8, 35, 51) // GM drum range
            .percussive()
            .polyphony(3, 2.1)
            .chords(false, 0.0)
            .melody(false, 0)
            .build()
    }

    /// Create a realistic piano chord progression
    pub fn piano_chords() -> NewFile {
        NewFileBuilder::new()
            .filename("Piano_Cmaj_Progression.mid")
            .filepath("/samples/keys/Piano_Cmaj_Progression.mid")
            .original_filename("Piano C Major Progression.mid")
            .content_hash(vec![2u8; 32]) // Unique hash for piano chords
            .file_size_bytes(1536)
            .duration_seconds(16.0)
            .duration_ticks(7680)
            .manufacturer("Native Instruments")
            .collection_name("The Gentleman")
            .folder_tags(vec!["piano".to_string(), "chords".to_string()])
            .build()
    }

    /// Create metadata for piano chords
    pub fn piano_chords_metadata(file_id: i64) -> NewMusicalMetadata {
        NewMusicalMetadataBuilder::new(file_id)
            .bpm(85.0, 0.95)
            .key_signature("C", 0.98)
            .time_signature(4, 4)
            .notes(64, 24, 48, 84)
            .polyphony(6, 4.2)
            .chords(true, 0.75)
            .melody(false, 0)
            .build()
    }

    /// Create a realistic bass line
    pub fn bass_line() -> NewFile {
        NewFileBuilder::new()
            .filename("Bass_Funky_Aminor.mid")
            .filepath("/samples/bass/Bass_Funky_Aminor.mid")
            .original_filename("Funky Bass A Minor.mid")
            .content_hash(vec![3u8; 32]) // Unique hash for bass line
            .file_size_bytes(896)
            .duration_seconds(4.0)
            .duration_ticks(1920)
            .manufacturer("Spectrasonics")
            .collection_name("Trilian")
            .folder_tags(vec!["bass".to_string(), "funk".to_string()])
            .build()
    }

    /// Create metadata for bass line
    pub fn bass_line_metadata(file_id: i64) -> NewMusicalMetadata {
        NewMusicalMetadataBuilder::new(file_id)
            .bpm(100.0, 0.97)
            .key_signature("Am", 0.93)
            .time_signature(4, 4)
            .notes(32, 7, 28, 52)
            .monophonic()
            .chords(false, 0.0)
            .melody(true, 24)
            .build()
    }

    /// Common tag presets
    pub fn common_tags() -> Vec<NewTag> {
        vec![
            NewTagBuilder::new("drums").category("instrument").build(),
            NewTagBuilder::new("bass").category("instrument").build(),
            NewTagBuilder::new("piano").category("instrument").build(),
            NewTagBuilder::new("loop").category("type").build(),
            NewTagBuilder::new("one-shot").category("type").build(),
            NewTagBuilder::new("house").category("genre").build(),
            NewTagBuilder::new("techno").category("genre").build(),
            NewTagBuilder::new("ambient").category("genre").build(),
        ]
    }
}

// ============================================================================
// Utility Functions for Tests
// ============================================================================

/// Generate a random 32-byte hash for testing
///
/// Returns a Vec<u8> with random bytes suitable for use as a content hash
/// in test data. Each call generates a new unique hash.
pub fn random_hash() -> Vec<u8> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Use system time and iteration to create pseudo-random hashes
    // This is deterministic but different enough for most test purposes
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let mut hash = vec![0u8; 32];

    // Seed with timestamp
    let bytes = timestamp.to_le_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        hash[i % 32] ^= byte;
    }

    // Add some variation - use thread ID and iteration
    let thread_id = std::thread::current().id();
    let id_val = format!("{:?}", thread_id);
    for (i, byte) in id_val.bytes().enumerate() {
        hash[(i + 8) % 32] ^= byte;
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_builder_defaults() {
        let file = NewFileBuilder::new().build();
        assert_eq!(file.filename, "test.mid");
        assert_eq!(file.num_tracks, 1);
    }

    #[test]
    fn test_file_builder_custom() {
        let file = NewFileBuilder::new()
            .filename("custom.mid")
            .file_size_bytes(4096)
            .manufacturer("Ableton")
            .build();

        assert_eq!(file.filename, "custom.mid");
        assert_eq!(file.file_size_bytes, 4096);
        assert_eq!(file.manufacturer, Some("Ableton".to_string()));
    }

    #[test]
    fn test_metadata_builder_defaults() {
        let metadata = NewMusicalMetadataBuilder::new(1).build();
        assert_eq!(metadata.file_id, 1);
        assert_eq!(metadata.time_signature_numerator, 4);
        assert_eq!(metadata.time_signature_denominator, 4);
    }

    #[test]
    fn test_fixtures_drum_loop() {
        let file = Fixtures::drum_loop();
        assert!(file.filename.contains("Drum"));
        assert!(file.folder_tags.is_some());

        let metadata = Fixtures::drum_loop_metadata(1);
        assert!(metadata.is_percussive);
        assert!(!metadata.has_chords);
    }

    #[test]
    fn test_fixtures_piano_chords() {
        let metadata = Fixtures::piano_chords_metadata(1);
        assert!(metadata.has_chords);
        assert!(!metadata.is_percussive);
        assert!(metadata.polyphony_max.is_some());
    }

    #[test]
    fn test_fixtures_bass_line() {
        let metadata = Fixtures::bass_line_metadata(1);
        assert!(metadata.is_monophonic);
        assert!(!metadata.is_polyphonic);
        assert!(metadata.has_melody);
    }
}
