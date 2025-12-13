# Key Profiles Reference

Comprehensive documentation for musical key detection in the MIDI Software Center, including Krumhansl-Schmuckler algorithm implementation, key profiles, MIDI processing, and genre-specific analysis.

## Table of Contents

1. [Overview](#overview)
2. [Krumhansl-Schmuckler Algorithm](#krumhansl-schmuckler-algorithm)
3. [Key Profiles](#key-profiles)
4. [MIDI Processing](#midi-processing)
5. [Key Detection Implementation](#key-detection-implementation)
6. [Data Structures](#data-structures)
7. [Performance Characteristics](#performance-characteristics)
8. [Integration Examples](#integration-examples)

---

## Overview

### Technology Stack
- **Language**: Rust 1.70+
- **MIDI Library**: midly 0.5 (parsing), custom implementation (analysis)
- **Algorithm**: Krumhansl-Schmuckler key-finding algorithm (1990)
- **Profile Source**: Krumhansl-Kessler (1982) probe-tone ratings

### Key Detection Output
- **Key Name**: String (e.g., "C", "F#", "Bb")
- **Scale Type**: Major or Minor
- **Confidence**: 0.0 to 1.0 (based on correlation gap)
- **Alternatives**: Top alternative keys with confidence scores
- **Pitch Class Distribution**: 12-element array of note frequencies

---

## Krumhansl-Schmuckler Algorithm

### Algorithm Overview

The Krumhansl-Schmuckler key-finding algorithm determines the musical key by:

1. **Build Pitch Class Histogram**: Count occurrences of each pitch class (0-11) from MIDI note events
2. **Normalize to Distribution**: Convert counts to probability distribution (sum = 1.0)
3. **Correlate with Profiles**: Calculate Pearson correlation between distribution and each of 24 key profiles (12 major + 12 minor)
4. **Select Best Match**: Key with highest correlation wins
5. **Calculate Confidence**: Based on gap between best and second-best correlations

### Pitch Class Mapping

```
Pitch Class | Note Names
------------|------------
     0      | C, B#
     1      | C#, Db
     2      | D
     3      | D#, Eb
     4      | E, Fb
     5      | F, E#
     6      | F#, Gb
     7      | G
     8      | G#, Ab
     9      | A
    10      | A#, Bb
    11      | B, Cb
```

### Conversion Formula
```rust
pitch_class = midi_note % 12
// Example: MIDI note 60 (Middle C) → 60 % 12 = 0 (C)
// Example: MIDI note 69 (A4 = 440Hz) → 69 % 12 = 9 (A)
```

---

## Key Profiles

### Krumhansl-Kessler Major Profile (1982)

Probe-tone ratings for major keys, indexed by pitch class relative to tonic:

```rust
pub const MAJOR_PROFILE: [f64; 12] = [
    6.35,  // Tonic (I) - C in C major
    2.23,  // Minor 2nd - C# in C major
    3.48,  // Major 2nd - D in C major
    2.33,  // Minor 3rd - D# in C major
    4.38,  // Major 3rd - E in C major
    4.09,  // Perfect 4th - F in C major
    2.52,  // Tritone - F# in C major
    5.19,  // Perfect 5th - G in C major
    2.39,  // Minor 6th - G# in C major
    3.66,  // Major 6th - A in C major
    2.29,  // Minor 7th - A# in C major
    2.88,  // Major 7th - B in C major
];
```

### Krumhansl-Kessler Minor Profile (1982)

Probe-tone ratings for minor keys:

```rust
pub const MINOR_PROFILE: [f64; 12] = [
    6.33,  // Tonic (i) - A in A minor
    2.68,  // Minor 2nd
    3.52,  // Major 2nd
    5.38,  // Minor 3rd (characteristic)
    2.60,  // Major 3rd
    3.53,  // Perfect 4th
    2.54,  // Tritone
    4.75,  // Perfect 5th
    3.98,  // Minor 6th
    2.69,  // Major 6th
    3.34,  // Minor 7th
    3.17,  // Major 7th
];
```

### Profile Characteristics

| Scale Degree | Major Weight | Minor Weight | Musical Significance |
|-------------|-------------|-------------|---------------------|
| Tonic (1)   | 6.35        | 6.33        | Strongest - home base |
| Perfect 5th | 5.19        | 4.75        | Second strongest - dominant |
| Major/Minor 3rd | 4.38/5.38 | 2.60/5.38 | Defines major/minor quality |
| Perfect 4th | 4.09        | 3.53        | Subdominant strength |
| Major 6th   | 3.66        | 2.69        | Common scale degree |
| Major 2nd   | 3.48        | 3.52        | Supertonic |
| Major 7th   | 2.88        | 3.17        | Leading tone (major) |
| Tritone     | 2.52        | 2.54        | Weakest - chromatic |

### Profile Rotation

To test different keys, rotate the profile array:

```rust
fn rotate_profile(profile: &[f64; 12], rotation: usize) -> [f64; 12] {
    let mut rotated = [0.0; 12];
    for i in 0..12 {
        rotated[i] = profile[(i + rotation) % 12];
    }
    rotated
}

// Example: Testing G major (rotation = 7)
// Original C major profile: [6.35, 2.23, 3.48, ...]
// After rotation by 7:      [5.19, 2.39, 3.66, ...] (G becomes tonic position)
```

---

## MIDI Processing

### MIDI File Structure

```rust
pub struct MidiFile {
    pub format: MidiFormat,           // 0, 1, or 2
    pub ticks_per_quarter_note: u16,  // Time resolution
    pub tracks: Vec<Track>,           // Track data
}

pub enum MidiFormat {
    SingleTrack,      // Format 0: All events in one track
    MultiTrack,       // Format 1: Multiple synchronized tracks
    MultiSequence,    // Format 2: Multiple independent sequences
}

pub struct Track {
    pub events: Vec<TimedEvent>,
}

pub struct TimedEvent {
    pub absolute_time: u64,  // Ticks from start
    pub event: Event,
}
```

### Event Types

```rust
pub enum Event {
    // Note Events (used for key detection)
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8, velocity: u8 },

    // Control Events
    ControlChange { channel: u8, controller: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    PitchBend { channel: u8, value: i16 },
    ChannelPressure { channel: u8, pressure: u8 },
    PolyPressure { channel: u8, note: u8, pressure: u8 },

    // Meta Events
    TempoChange { microseconds_per_beat: u32 },
    TimeSignature { numerator: u8, denominator: u8, clocks_per_click: u8, notated_32nd_notes: u8 },
    KeySignature { key: i8, scale: u8 },  // MIDI key signature (often unreliable)
    Text { text_type: u8, text: String },
    TrackName { name: String },
    EndOfTrack,

    // System Events
    SysEx { data: Vec<u8> },
}
```

### MIDI Parsing Implementation

```rust
/// Parse complete MIDI file from bytes
pub fn parse_midi_file(data: &[u8]) -> Result<MidiFile, MidiError> {
    let mut cursor = 0;

    // Parse header chunk "MThd"
    let header = parse_header(data, &mut cursor)?;

    // Parse track chunks "MTrk"
    let mut tracks = Vec::with_capacity(header.num_tracks as usize);
    for _ in 0..header.num_tracks {
        let track = parse_track(data, &mut cursor)?;
        tracks.push(track);
    }

    Ok(MidiFile {
        format: header.format,
        ticks_per_quarter_note: header.ticks_per_quarter_note,
        tracks,
    })
}

/// Parse variable-length quantity (VLQ) encoding
fn read_vlq(data: &[u8], cursor: &mut usize) -> Result<u32, MidiError> {
    let mut value: u32 = 0;
    loop {
        if *cursor >= data.len() {
            return Err(MidiError::UnexpectedEof);
        }
        let byte = data[*cursor];
        *cursor += 1;
        value = (value << 7) | (byte & 0x7F) as u32;
        if byte & 0x80 == 0 {
            break;
        }
    }
    Ok(value)
}
```

### Extracting Notes for Key Detection

```rust
/// Build pitch class histogram from MIDI events
fn build_pitch_class_histogram(midi_file: &MidiFile) -> [u32; 12] {
    let mut histogram = [0u32; 12];

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::NoteOn { note, velocity, channel } = &timed_event.event {
                // Skip drum channel (channel 9 in 0-indexed, channel 10 in MIDI)
                if *channel == 9 {
                    continue;
                }
                // Skip note-off encoded as note-on with velocity 0
                if *velocity == 0 {
                    continue;
                }
                let pitch_class = (*note % 12) as usize;
                histogram[pitch_class] = histogram[pitch_class].saturating_add(1);
            }
        }
    }

    histogram
}
```

---

## Key Detection Implementation

### Full Implementation (Pipeline)

Location: `pipeline/src-tauri/src/core/analysis/key_detector.rs`

```rust
use crate::core::analysis::key_profiles::{MAJOR_PROFILE, MINOR_PROFILE};
use crate::core::midi::MidiFile;

/// Scale type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    Major,
    Minor,
}

/// Alternative key suggestion
#[derive(Debug, Clone)]
pub struct KeyAlternative {
    pub key: String,
    pub scale_type: ScaleType,
    pub confidence: f64,
}

/// Complete key detection result
#[derive(Debug, Clone)]
pub struct KeyDetectionResult {
    pub key: String,                          // e.g., "C", "F#", "Bb"
    pub scale_type: ScaleType,                // Major or Minor
    pub confidence: f64,                      // 0.0 to 1.0
    pub alternatives: Vec<KeyAlternative>,    // Top 3 alternatives
    pub pitch_class_distribution: [f64; 12],  // Normalized distribution
}

/// Detect musical key from MIDI file
pub fn detect_key(midi_file: &MidiFile) -> KeyDetectionResult {
    // Step 1: Build pitch class histogram
    let histogram = build_pitch_class_histogram(midi_file);

    // Step 2: Check minimum note count
    let total_notes: u32 = histogram.iter().sum();
    if total_notes < 10 {
        return KeyDetectionResult {
            key: "Unknown".to_string(),
            scale_type: ScaleType::Major,
            confidence: 0.0,
            alternatives: vec![],
            pitch_class_distribution: [0.0; 12],
        };
    }

    // Step 3: Normalize to probability distribution
    let distribution = normalize_histogram(&histogram);

    // Step 4: Calculate correlations for all 24 keys
    let mut correlations: Vec<(String, ScaleType, f64)> = Vec::with_capacity(24);

    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
    ];

    for (root, note_name) in NOTE_NAMES.iter().enumerate() {
        // Test major key
        let major_profile = rotate_profile(&MAJOR_PROFILE, root);
        let major_corr = calculate_correlation(&distribution, &major_profile);
        correlations.push((note_name.to_string(), ScaleType::Major, major_corr));

        // Test minor key
        let minor_profile = rotate_profile(&MINOR_PROFILE, root);
        let minor_corr = calculate_correlation(&distribution, &minor_profile);
        correlations.push((note_name.to_string(), ScaleType::Minor, minor_corr));
    }

    // Step 5: Sort by correlation (highest first)
    correlations.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Step 6: Calculate confidence
    let confidence = calculate_confidence(&correlations);

    // Step 7: Build result
    let (best_key, best_scale, _) = &correlations[0];

    let alternatives: Vec<KeyAlternative> = correlations[1..4]
        .iter()
        .map(|(key, scale, corr)| KeyAlternative {
            key: key.clone(),
            scale_type: *scale,
            confidence: *corr,
        })
        .collect();

    KeyDetectionResult {
        key: best_key.clone(),
        scale_type: *best_scale,
        confidence,
        alternatives,
        pitch_class_distribution: distribution,
    }
}
```

### Correlation Calculation

```rust
/// Calculate Pearson correlation coefficient between two arrays
fn calculate_correlation(distribution: &[f64; 12], profile: &[f64; 12]) -> f64 {
    // Calculate means
    let dist_mean: f64 = distribution.iter().sum::<f64>() / 12.0;
    let prof_mean: f64 = profile.iter().sum::<f64>() / 12.0;

    // Calculate correlation components
    let mut numerator = 0.0;
    let mut dist_sq_sum = 0.0;
    let mut prof_sq_sum = 0.0;

    for i in 0..12 {
        let dist_diff = distribution[i] - dist_mean;
        let prof_diff = profile[i] - prof_mean;

        numerator += dist_diff * prof_diff;
        dist_sq_sum += dist_diff * dist_diff;
        prof_sq_sum += prof_diff * prof_diff;
    }

    let denominator = (dist_sq_sum * prof_sq_sum).sqrt();

    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}
```

### Confidence Calculation

```rust
/// Calculate confidence based on gap between best and second-best correlations
fn calculate_confidence(sorted_correlations: &[(String, ScaleType, f64)]) -> f64 {
    if sorted_correlations.len() < 2 {
        return 0.0;
    }

    let best = sorted_correlations[0].2;
    let second_best = sorted_correlations[1].2;

    // Confidence is proportional to the gap
    // Large gap = high confidence, small gap = ambiguous key
    let gap = best - second_best;

    // Normalize gap to 0-1 range (empirically, gaps > 0.1 are confident)
    let confidence = (gap / 0.1).min(1.0).max(0.0);

    // Also factor in absolute correlation strength
    let strength = (best + 1.0) / 2.0; // Convert -1..1 to 0..1

    // Combined confidence
    (confidence * 0.7 + strength * 0.3).min(1.0)
}
```

### Simplified Wrapper (Shared Library)

Location: `shared/rust/src/core/analysis/key_detector.rs`

```rust
/// Simple key detection returning optional string
/// Returns None if confidence < 0.5 or insufficient notes
pub fn detect_key(midi_file: &crate::core::midi::MidiFile) -> Option<String> {
    // Build pitch class histogram
    let mut pitch_class_counts = [0u32; 12];

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let crate::core::midi::Event::NoteOn { note, .. } = &timed_event.event {
                let pitch_class = (note % 12) as usize;
                pitch_class_counts[pitch_class] = pitch_class_counts[pitch_class].saturating_add(1);
            }
        }
    }

    // Check minimum notes
    let total_notes: u32 = pitch_class_counts.iter().sum();
    if total_notes < 10 {
        return None;
    }

    // Normalize
    let mut distribution = [0.0; 12];
    for (i, &count) in pitch_class_counts.iter().enumerate() {
        distribution[i] = count as f64 / total_notes as f64;
    }

    // Inline profiles (Krumhansl-Schmuckler)
    const MAJOR_PROFILE: [f64; 12] = [
        6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88
    ];
    const MINOR_PROFILE: [f64; 12] = [
        6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17
    ];

    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
    ];

    // Find best key
    let mut best_correlation = -1.0;
    let mut best_key = String::new();

    for root in 0..12 {
        let major_corr = calculate_correlation(&distribution, &MAJOR_PROFILE, root);
        if major_corr > best_correlation {
            best_correlation = major_corr;
            best_key = format!("{} major", NOTE_NAMES[root]);
        }

        let minor_corr = calculate_correlation(&distribution, &MINOR_PROFILE, root);
        if minor_corr > best_correlation {
            best_correlation = minor_corr;
            best_key = format!("{} minor", NOTE_NAMES[root]);
        }
    }

    // Return if confidence is reasonable
    if best_correlation > 0.5 {
        Some(best_key)
    } else {
        None
    }
}
```

---

## Data Structures

### Database Schema for Key Detection Results

```sql
-- Musical metadata table stores key detection results
CREATE TABLE musical_metadata (
    id SERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,

    -- Key detection fields
    key_signature VARCHAR(10),           -- e.g., "C", "F#", "Bb"
    key_confidence REAL,                 -- 0.0 to 1.0
    scale_type VARCHAR(10),              -- "major" or "minor"

    -- BPM fields
    bpm REAL,
    bpm_confidence REAL,

    -- Duration and time signature
    duration_seconds REAL,
    time_signature_numerator INTEGER,
    time_signature_denominator INTEGER,

    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(file_id)
);

-- Index for key-based queries
CREATE INDEX idx_musical_metadata_key ON musical_metadata(key_signature);
CREATE INDEX idx_musical_metadata_scale ON musical_metadata(scale_type);
```

### Rust Models

```rust
/// Database model for musical metadata
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MusicalMetadata {
    pub id: i32,
    pub file_id: i64,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f32>,
    pub scale_type: Option<String>,
    pub bpm: Option<f32>,
    pub bpm_confidence: Option<f32>,
    pub duration_seconds: Option<f32>,
    pub time_signature_numerator: Option<i32>,
    pub time_signature_denominator: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Insert or update key detection results
pub async fn upsert_key_detection(
    pool: &PgPool,
    file_id: i64,
    result: &KeyDetectionResult,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO musical_metadata (file_id, key_signature, key_confidence, scale_type)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (file_id) DO UPDATE
        SET key_signature = EXCLUDED.key_signature,
            key_confidence = EXCLUDED.key_confidence,
            scale_type = EXCLUDED.scale_type,
            updated_at = NOW()
        "#,
        file_id,
        result.key,
        result.confidence as f32,
        match result.scale_type {
            ScaleType::Major => "major",
            ScaleType::Minor => "minor",
        }
    )
    .execute(pool)
    .await?;

    Ok(())
}
```

---

## Performance Characteristics

### Benchmarks

| Operation | Speed | Notes |
|-----------|-------|-------|
| MIDI Parsing | 88,656 files/sec | BLAKE3 hashing included |
| Key Detection (per file) | ~0.5ms | 1,000+ notes typical |
| Key Detection (batch) | 181-360 files/sec | Full analysis pipeline |
| Correlation Calculation | ~50 ns | 24 correlations per file |
| Profile Rotation | ~20 ns | Array copy operation |

### Memory Usage

- Pitch class histogram: 48 bytes (12 × u32)
- Normalized distribution: 96 bytes (12 × f64)
- KeyDetectionResult: ~200 bytes
- Profile arrays (static): 192 bytes (2 × 12 × f64)

### Optimization Notes

1. **Skip Drum Channel**: Channel 9 (index 9) contains percussion, not pitched notes
2. **Minimum Note Threshold**: Require 10+ notes for reliable detection
3. **Confidence Threshold**: Return "Unknown" if confidence < 0.5
4. **Batch Processing**: Process multiple files in parallel with rayon

---

## Integration Examples

### Tauri Command Integration

```rust
#[tauri::command]
pub async fn analyze_key(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<KeyDetectionResult, String> {
    // Read MIDI file
    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Parse MIDI
    let midi_file = parse_midi_file(&data)
        .map_err(|e| format!("Failed to parse MIDI: {}", e))?;

    // Detect key
    let result = detect_key(&midi_file);

    // Store in database
    let pool = state.pool();
    upsert_key_detection(pool, file_id, &result)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(result)
}
```

### Search by Key

```rust
/// Search files by musical key
pub async fn search_by_key(
    pool: &PgPool,
    key: &str,
    scale_type: Option<&str>,
    limit: i64,
) -> Result<Vec<File>, sqlx::Error> {
    sqlx::query_as!(
        File,
        r#"
        SELECT f.*
        FROM files f
        JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE mm.key_signature = $1
          AND ($2::text IS NULL OR mm.scale_type = $2)
        ORDER BY mm.key_confidence DESC
        LIMIT $3
        "#,
        key,
        scale_type,
        limit
    )
    .fetch_all(pool)
    .await
}
```

### Frontend Integration (TypeScript/Svelte)

```typescript
import { invoke } from '@tauri-apps/api/core';

interface KeyDetectionResult {
  key: string;
  scale_type: 'Major' | 'Minor';
  confidence: number;
  alternatives: Array<{
    key: string;
    scale_type: 'Major' | 'Minor';
    confidence: number;
  }>;
  pitch_class_distribution: number[];
}

// Analyze single file
async function analyzeKey(filePath: string): Promise<KeyDetectionResult> {
  return await invoke('analyze_key', { filePath });
}

// Search by key
async function searchByKey(key: string, scaleType?: string): Promise<File[]> {
  return await invoke('search_files', {
    key,
    scaleType,
    limit: 100
  });
}

// Display key with confidence indicator
function formatKey(result: KeyDetectionResult): string {
  const scale = result.scale_type === 'Major' ? 'maj' : 'min';
  const confidence = Math.round(result.confidence * 100);
  return `${result.key} ${scale} (${confidence}% confidence)`;
}
```

---

## Alternative Profiles (Reference)

### Temperley Profile (2001)

Alternative profile emphasizing leading tone resolution:

```rust
pub const TEMPERLEY_MAJOR: [f64; 12] = [
    5.0, 2.0, 3.5, 2.0, 4.5, 4.0, 2.0, 4.5, 2.0, 3.5, 1.5, 4.0
];

pub const TEMPERLEY_MINOR: [f64; 12] = [
    5.0, 2.0, 3.5, 4.5, 2.0, 4.0, 2.0, 4.5, 3.5, 2.0, 1.5, 4.0
];
```

### Simple Binary Profile

For rapid classification when accuracy is less critical:

```rust
pub const SIMPLE_MAJOR: [f64; 12] = [
    1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0
];

pub const SIMPLE_MINOR: [f64; 12] = [
    1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0
];
```

### Genre-Specific Profiles (Future Enhancement)

These profiles could be trained on genre-specific datasets:

```rust
// Blues: Emphasizes blue notes (b3, b5, b7)
pub const BLUES_PROFILE: [f64; 12] = [
    6.0, 2.0, 3.0, 5.0, 3.5, 4.0, 4.5, 5.0, 2.0, 3.5, 5.0, 2.5
];

// Jazz: More chromatic, emphasizes 9ths, 11ths, 13ths
pub const JAZZ_MAJOR_PROFILE: [f64; 12] = [
    5.5, 3.0, 4.0, 2.5, 4.5, 4.0, 3.5, 5.0, 3.0, 4.0, 3.0, 4.0
];

// Electronic/EDM: Simpler harmonic content
pub const EDM_PROFILE: [f64; 12] = [
    6.5, 1.5, 3.0, 2.0, 4.0, 3.5, 2.0, 5.5, 2.0, 3.0, 2.0, 3.0
];
```

---

## File Locations

| Component | Path |
|-----------|------|
| Full Key Detector | `pipeline/src-tauri/src/core/analysis/key_detector.rs` |
| Key Profiles | `pipeline/src-tauri/src/core/analysis/key_profiles.rs` |
| Simplified Wrapper | `shared/rust/src/core/analysis/key_detector.rs` |
| MIDI Parser | `shared/rust/src/core/midi/parser.rs` |
| Database Models | `pipeline/src-tauri/src/db/models.rs` |
| Search Repository | `pipeline/src-tauri/src/db/repositories/search_repository.rs` |

---

## References

1. Krumhansl, C. L. (1990). *Cognitive Foundations of Musical Pitch*. Oxford University Press.
2. Krumhansl, C. L., & Kessler, E. J. (1982). Tracing the dynamic changes in perceived tonal organization in a spatial representation of musical keys. *Psychological Review*, 89(4), 334-368.
3. Temperley, D. (2001). *The Cognition of Basic Musical Structures*. MIT Press.
4. MIDI Manufacturers Association. (1996). *The Complete MIDI 1.0 Detailed Specification*.

---

*Generated for MIDI Software Center - Key Detection Reference*
*Last Updated: December 2025*
