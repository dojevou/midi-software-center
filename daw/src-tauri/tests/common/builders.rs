
/// Test data builders for fluent test construction
use midi_daw::models::{MidiEvent, MidiEventType};

/// Builder for MIDI files in database
pub struct MidiFileBuilder {
    filepath: String,
    filename: String,
    file_size_bytes: i64,
    num_tracks: i16,
    bpm: Option<f64>,
    key_signature: Option<String>,
}

impl MidiFileBuilder {
    pub fn new(filename: &str) -> Self {
        Self {
            filepath: format!("/test/{}", filename),
            filename: filename.to_string(),
            file_size_bytes: 1024,
            num_tracks: 1,
            bpm: Some(120.0),
            key_signature: Some("C_MAJOR".to_string()),
        }
    }

    pub fn with_bpm(mut self, bpm: f64) -> Self {
        self.bpm = Some(bpm);
        self
    }

    pub fn with_key(mut self, key: &str) -> Self {
        self.key_signature = Some(key.to_string());
        self
    }

    pub fn with_tracks(mut self, count: i16) -> Self {
        self.num_tracks = count;
        self
    }

    pub async fn insert(self, pool: &sqlx::PgPool) -> i64 {
        // Generate a simple hash based on timestamp and thread ID
        let hash_str = generate_test_hash();

        let result = sqlx::query!(
            "INSERT INTO files (filepath, filename, content_hash, file_size_bytes, num_tracks)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id",
            self.filepath,
            self.filename,
            hash_str.as_bytes(),
            self.file_size_bytes,
            self.num_tracks
        )
        .fetch_one(pool)
        .await
        .expect("Failed to insert test file");

        let file_id = result.id;

        // Insert metadata if provided
        if self.bpm.is_some() || self.key_signature.is_some() {
            if let Some(ref key) = self.key_signature {
                let query_str = format!(
                    "INSERT INTO musical_metadata (file_id, bpm, key_signature, total_notes)
                     VALUES ($1, $2, '{}'::music_key, $3)",
                    key
                );
                let _ = sqlx::query(&query_str)
                    .bind(file_id)
                    .bind(self.bpm)
                    .bind(100i32)
                    .execute(pool)
                    .await;
            } else {
                let _ = sqlx::query(
                    "INSERT INTO musical_metadata (file_id, bpm, total_notes)
                     VALUES ($1, $2, $3)",
                )
                .bind(file_id)
                .bind(self.bpm)
                .bind(100i32)
                .execute(pool)
                .await;
            }
        }

        file_id
    }
}

/// Builder for sequencer tracks
pub struct TrackBuilder {
    name: String,
    channel: u8,
    muted: bool,
    solo: bool,
    volume: u8,
    pan: u8,
    events: Vec<MidiEvent>,
}

impl TrackBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            channel: 0,
            muted: false,
            solo: false,
            volume: 100,
            pan: 64,
            events: Vec::new(),
        }
    }

    pub fn with_channel(mut self, channel: u8) -> Self {
        self.channel = channel;
        self
    }

    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    pub fn solo(mut self) -> Self {
        self.solo = true;
        self
    }

    pub fn with_volume(mut self, volume: u8) -> Self {
        self.volume = volume;
        self
    }

    pub fn with_events(mut self, events: Vec<MidiEvent>) -> Self {
        self.events = events;
        self
    }

    pub fn add_note(mut self, tick: u64, note: u8, velocity: u8, duration: u64) -> Self {
        self.events.push(MidiEvent {
            event_type: MidiEventType::NoteOn,
            tick,
            channel: self.channel,
            note: Some(note),
            velocity: Some(velocity),
            controller: None,
            value: None,
            program: None,
        });
        self.events.push(MidiEvent {
            event_type: MidiEventType::NoteOff,
            tick: tick + duration,
            channel: self.channel,
            note: Some(note),
            velocity: Some(0),
            controller: None,
            value: None,
            program: None,
        });
        self
    }

    pub fn build(self) -> (String, u8, Vec<MidiEvent>) {
        (self.name, self.channel, self.events)
    }
}

/// Builder for sequencer state
pub struct SequencerStateBuilder {
    bpm: f32,
    playing: bool,
    position: u64,
}

impl SequencerStateBuilder {
    pub fn new() -> Self {
        Self { bpm: 120.0, playing: false, position: 0 }
    }

    pub fn with_bpm(mut self, bpm: f32) -> Self {
        self.bpm = bpm;
        self
    }

    pub fn playing(mut self) -> Self {
        self.playing = true;
        self
    }

    pub fn at_position(mut self, position: u64) -> Self {
        self.position = position;
        self
    }

    pub fn build(self) -> (f32, bool, u64) {
        (self.bpm, self.playing, self.position)
    }
}

impl Default for SequencerStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a simple deterministic hash for testing
///
/// Uses system time and thread ID to create a pseudo-random but deterministic hash string.
fn generate_test_hash() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    let thread_id = std::thread::current().id();
    let id_val = format!("{:?}", thread_id);

    format!("{:064x}", timestamp ^ id_val.len() as u64)
}
