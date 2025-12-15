/// Text Metadata Extractor - Trusty Module
///
/// Extracts text metadata from MIDI files:
/// - Track names
/// - Copyright notices
/// - Lyrics
/// - Markers
/// - Cue points
/// - Instrument names
///
/// This module complements the musical analysis by capturing human-readable
/// text information embedded in MIDI files.
use super::types::{Event, MidiFile, TextType};

/// Text metadata extracted from a MIDI file
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextMetadata {
    /// Track names from each track
    pub track_names: Vec<String>,

    /// Copyright notice (usually in first track)
    pub copyright: Option<String>,

    /// Instrument names assigned to tracks
    pub instrument_names: Vec<String>,

    /// Lyrics embedded in the MIDI file
    pub lyrics: Vec<String>,

    /// Markers (section labels, rehearsal marks)
    pub markers: Vec<String>,

    /// Cue points (timing references)
    pub cue_points: Vec<String>,

    /// General text messages
    pub text_messages: Vec<String>,
}

impl TextMetadata {
    /// Extract all text metadata from a MIDI file
    ///
    /// # Examples
    /// ```ignore
    /// use midi_library_shared::core::midi::parser::parse_midi_file;
    /// use midi_library_shared::core::midi::text_metadata::TextMetadata;
    ///
    /// let data = std::fs::read("song.mid").unwrap();
    /// let midi_file = parse_midi_file(&data).unwrap();
    /// let metadata = TextMetadata::extract(&midi_file);
    ///
    /// println!("Track names: {:?}", metadata.track_names);
    /// println!("Copyright: {:?}", metadata.copyright);
    /// ```
    pub fn extract(midi_file: &MidiFile) -> Self {
        let mut metadata = TextMetadata::default();

        for track in &midi_file.tracks {
            for timed_event in &track.events {
                if let Event::Text { text_type, text } = &timed_event.event {
                    Self::process_text_event(&mut metadata, text_type, text);
                }
            }
        }

        // Deduplicate all vectors
        metadata.track_names.sort();
        metadata.track_names.dedup();
        metadata.instrument_names.sort();
        metadata.instrument_names.dedup();
        metadata.lyrics.dedup();
        metadata.markers.dedup();
        metadata.cue_points.dedup();
        metadata.text_messages.sort();
        metadata.text_messages.dedup();

        metadata
    }

    /// Process a single text event
    fn process_text_event(metadata: &mut TextMetadata, text_type: &TextType, text: &str) {
        // Skip empty strings
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        match text_type {
            TextType::TrackName => {
                if !metadata.track_names.contains(&text.to_string()) {
                    metadata.track_names.push(text.to_string());
                }
            },
            TextType::Copyright => {
                // Keep first copyright notice found
                if metadata.copyright.is_none() {
                    metadata.copyright = Some(text.to_string());
                }
            },
            TextType::InstrumentName => {
                if !metadata.instrument_names.contains(&text.to_string()) {
                    metadata.instrument_names.push(text.to_string());
                }
            },
            TextType::Lyric => {
                metadata.lyrics.push(text.to_string());
            },
            TextType::Marker => {
                if !metadata.markers.contains(&text.to_string()) {
                    metadata.markers.push(text.to_string());
                }
            },
            TextType::CuePoint => {
                if !metadata.cue_points.contains(&text.to_string()) {
                    metadata.cue_points.push(text.to_string());
                }
            },
            TextType::Text => {
                if !metadata.text_messages.contains(&text.to_string()) {
                    metadata.text_messages.push(text.to_string());
                }
            },
        }
    }

    /// Check if any text metadata exists
    pub fn is_empty(&self) -> bool {
        self.track_names.is_empty()
            && self.copyright.is_none()
            && self.instrument_names.is_empty()
            && self.lyrics.is_empty()
            && self.markers.is_empty()
            && self.cue_points.is_empty()
            && self.text_messages.is_empty()
    }

    /// Get a summary of text metadata for display
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.track_names.is_empty() {
            parts.push(format!("{} tracks", self.track_names.len()));
        }

        if let Some(copyright) = &self.copyright {
            parts.push(format!("© {}", copyright));
        }

        if !self.instrument_names.is_empty() {
            parts.push(format!("{} instruments", self.instrument_names.len()));
        }

        if !self.markers.is_empty() {
            parts.push(format!("{} markers", self.markers.len()));
        }

        if !self.lyrics.is_empty() {
            parts.push(format!("{} lyric lines", self.lyrics.len()));
        }

        if parts.is_empty() {
            "No text metadata".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::{Header, MidiFile, TimedEvent, Track};

    fn create_test_midi_with_text() -> MidiFile {
        MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::TrackName,
                                text: "Piano Track".to_string(),
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::Copyright,
                                text: "2025 Test Artist".to_string(),
                            },
                        },
                        TimedEvent {
                            delta_ticks: 100,
                            event: Event::Text {
                                text_type: TextType::Marker,
                                text: "Verse 1".to_string(),
                            },
                        },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::TrackName,
                                text: "Bass Track".to_string(),
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::InstrumentName,
                                text: "Electric Bass".to_string(),
                            },
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_extract_track_names() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.track_names.len(), 2);
        assert!(metadata.track_names.contains(&"Piano Track".to_string()));
        assert!(metadata.track_names.contains(&"Bass Track".to_string()));
    }

    #[test]
    fn test_extract_copyright() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.copyright, Some("2025 Test Artist".to_string()));
    }

    #[test]
    fn test_extract_instrument_names() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.instrument_names.len(), 1);
        assert_eq!(metadata.instrument_names[0], "Electric Bass");
    }

    #[test]
    fn test_extract_markers() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.markers.len(), 1);
        assert_eq!(metadata.markers[0], "Verse 1");
    }

    #[test]
    fn test_empty_metadata() {
        let midi = MidiFile {
            header: Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }],
        };

        let metadata = TextMetadata::extract(&midi);
        assert!(metadata.is_empty());
    }

    #[test]
    fn test_deduplication() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![TimedEvent {
                        delta_ticks: 0,
                        event: Event::Text {
                            text_type: TextType::TrackName,
                            text: "Duplicate".to_string(),
                        },
                    }],
                },
                Track {
                    events: vec![TimedEvent {
                        delta_ticks: 0,
                        event: Event::Text {
                            text_type: TextType::TrackName,
                            text: "Duplicate".to_string(),
                        },
                    }],
                },
            ],
        };

        let metadata = TextMetadata::extract(&midi);
        assert_eq!(metadata.track_names.len(), 1);
        assert_eq!(metadata.track_names[0], "Duplicate");
    }

    #[test]
    fn test_summary() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        let summary = metadata.summary();
        assert!(summary.contains("2 tracks"));
        assert!(summary.contains("© 2025 Test Artist"));
        assert!(summary.contains("1 instruments"));
        assert!(summary.contains("1 markers"));
    }

    #[test]
    fn test_empty_summary() {
        let midi = MidiFile {
            header: Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }],
        };

        let metadata = TextMetadata::extract(&midi);
        assert_eq!(metadata.summary(), "No text metadata");
    }
}
