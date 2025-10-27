use serde::{Deserialize, Serialize};

/// Represents a complete MIDI file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiFile {
    pub header: Header,
    pub tracks: Vec<Track>,
}

/// MIDI header chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub format: u16, // 0, 1, or 2
    pub num_tracks: u16,
    pub ticks_per_quarter_note: u16,
}

/// A single MIDI track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub events: Vec<TimedEvent>,
}

/// Event with delta time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    pub delta_ticks: u32,
    pub event: Event,
}

/// MIDI events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Channel events
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    Aftertouch {
        channel: u8,
        note: u8,
        pressure: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ChannelAftertouch {
        channel: u8,
        pressure: u8,
    },
    PitchBend {
        channel: u8,
        value: i16,
    },

    // Meta events
    TempoChange {
        microseconds_per_quarter: u32,
    },
    TimeSignature {
        numerator: u8,
        denominator: u8,
        clocks_per_click: u8,
        thirty_seconds_per_quarter: u8,
    },
    KeySignature {
        sharps_flats: i8,
        is_minor: bool,
    },
    Text {
        text_type: TextType,
        text: String,
    },
    EndOfTrack,

    // SysEx
    SysEx {
        data: Vec<u8>,
    },

    // Unknown/unsupported
    Unknown {
        status: u8,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextType {
    Text,
    Copyright,
    TrackName,
    InstrumentName,
    Lyric,
    Marker,
    CuePoint,
}

impl MidiFile {
    /// Calculate total duration in seconds
    pub fn duration_seconds(&self, _default_tempo_bpm: f64) -> f64 {
        let mut total_ticks = 0u64;
        let mut current_tempo_us_per_qn = 500_000u32; // Default: 120 BPM

        for track in &self.tracks {
            let mut track_ticks = 0u64;

            for timed_event in &track.events {
                track_ticks += timed_event.delta_ticks as u64;

                // Update tempo if we encounter a tempo change
                if let Event::TempoChange {
                    microseconds_per_quarter,
                } = timed_event.event
                {
                    current_tempo_us_per_qn = microseconds_per_quarter;
                }
            }

            total_ticks = total_ticks.max(track_ticks);
        }

        // Convert ticks to seconds
        let seconds_per_tick = (current_tempo_us_per_qn as f64 / 1_000_000.0)
            / self.header.ticks_per_quarter_note as f64;
        total_ticks as f64 * seconds_per_tick
    }

    /// Count total notes across all tracks
    pub fn total_notes(&self) -> usize {
        self.tracks
            .iter()
            .flat_map(|track| &track.events)
            .filter(|event| matches!(event.event, Event::NoteOn { velocity, .. } if velocity > 0))
            .count()
    }

    /// Get all unique MIDI channels used
    pub fn channels_used(&self) -> Vec<u8> {
        let mut channels = std::collections::HashSet::new();

        for track in &self.tracks {
            for timed_event in &track.events {
                if let Some(channel) = timed_event.event.channel() {
                    channels.insert(channel);
                }
            }
        }

        let mut result: Vec<u8> = channels.into_iter().collect();
        result.sort();
        result
    }
}

impl Event {
    /// Get the MIDI channel for channel events, None for meta/sysex
    pub fn channel(&self) -> Option<u8> {
        match self {
            Event::NoteOn { channel, .. }
            | Event::NoteOff { channel, .. }
            | Event::Aftertouch { channel, .. }
            | Event::ControlChange { channel, .. }
            | Event::ProgramChange { channel, .. }
            | Event::ChannelAftertouch { channel, .. }
            | Event::PitchBend { channel, .. } => Some(*channel),
            _ => None,
        }
    }

    /// Check if this is a note event
    pub fn is_note(&self) -> bool {
        matches!(self, Event::NoteOn { .. } | Event::NoteOff { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a basic MIDI file for testing
    fn create_basic_midi() -> MidiFile {
        MidiFile {
            header: Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track { events: vec![] }],
        }
    }

    /// Helper function to create a MIDI file with notes
    fn create_midi_with_notes() -> MidiFile {
        MidiFile {
            header: Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 480,
            },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn {
                            channel: 0,
                            note: 60,
                            velocity: 100,
                        },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOff {
                            channel: 0,
                            note: 60,
                            velocity: 0,
                        },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn {
                            channel: 1,
                            note: 64,
                            velocity: 80,
                        },
                    },
                ],
            }],
        }
    }

    mod midi_file_tests {
        use super::*;

        #[test]
        fn test_duration_seconds_empty_file() {
            let midi = create_basic_midi();
            let duration = midi.duration_seconds(120.0);

            // Empty file should have 0 duration
            assert_eq!(duration, 0.0);
        }

        #[test]
        fn test_duration_seconds_with_default_tempo() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 60,
                                velocity: 100,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 1920, // 4 quarters = 1 bar at 480 tpq
                            event: Event::NoteOff {
                                channel: 0,
                                note: 60,
                                velocity: 0,
                            },
                        },
                    ],
                }],
            };

            let duration = midi.duration_seconds(120.0);

            // At 120 BPM (500,000 μs/quarter), 1920 ticks = 4 quarters = 2 seconds
            // seconds_per_tick = 500_000 / 1_000_000 / 480 = 0.00104166...
            // duration = 1920 * 0.00104166... ≈ 2.0 seconds
            assert!(
                (duration - 2.0).abs() < 0.01,
                "Duration should be ~2.0 seconds, got {}",
                duration
            );
        }

        #[test]
        fn test_duration_seconds_with_tempo_change() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange {
                                microseconds_per_quarter: 600_000, // 100 BPM
                            },
                        },
                        TimedEvent {
                            delta_ticks: 1920, // 4 quarters at 100 BPM
                            event: Event::NoteOff {
                                channel: 0,
                                note: 60,
                                velocity: 0,
                            },
                        },
                    ],
                }],
            };

            let duration = midi.duration_seconds(120.0);

            // At 100 BPM (600,000 μs/quarter), 1920 ticks = 4 quarters = 2.4 seconds
            // seconds_per_tick = 600_000 / 1_000_000 / 480 = 0.00125
            // duration = 1920 * 0.00125 = 2.4 seconds
            assert!(
                (duration - 2.4).abs() < 0.01,
                "Duration should be ~2.4 seconds, got {}",
                duration
            );
        }

        #[test]
        fn test_duration_seconds_multiple_tracks() {
            // Duration should be the length of the longest track
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 2,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 960, // Short track
                            event: Event::EndOfTrack,
                        }],
                    },
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 1920, // Longer track
                            event: Event::EndOfTrack,
                        }],
                    },
                ],
            };

            let duration = midi.duration_seconds(120.0);

            // Should use longest track (1920 ticks)
            assert!(
                (duration - 2.0).abs() < 0.01,
                "Duration should be ~2.0 seconds (longest track), got {}",
                duration
            );
        }

        #[test]
        fn test_total_notes_empty_file() {
            let midi = create_basic_midi();
            assert_eq!(midi.total_notes(), 0);
        }

        #[test]
        fn test_total_notes_with_notes() {
            let midi = create_midi_with_notes();

            // Should count 2 NoteOn events (velocity > 0)
            assert_eq!(midi.total_notes(), 2);
        }

        #[test]
        fn test_total_notes_excludes_zero_velocity() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 60,
                                velocity: 0, // Zero velocity = note off
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 64,
                                velocity: 100, // Real note on
                            },
                        },
                    ],
                }],
            };

            // Should only count note with velocity > 0
            assert_eq!(midi.total_notes(), 1);
        }

        #[test]
        fn test_total_notes_excludes_note_off() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 60,
                                velocity: 100,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 480,
                            event: Event::NoteOff {
                                channel: 0,
                                note: 60,
                                velocity: 64,
                            },
                        },
                    ],
                }],
            };

            // Should only count NoteOn events
            assert_eq!(midi.total_notes(), 1);
        }

        #[test]
        fn test_total_notes_multiple_tracks() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 2,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![
                    Track {
                        events: vec![
                            TimedEvent {
                                delta_ticks: 0,
                                event: Event::NoteOn {
                                    channel: 0,
                                    note: 60,
                                    velocity: 100,
                                },
                            },
                            TimedEvent {
                                delta_ticks: 0,
                                event: Event::NoteOn {
                                    channel: 0,
                                    note: 64,
                                    velocity: 80,
                                },
                            },
                        ],
                    },
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 1,
                                note: 67,
                                velocity: 90,
                            },
                        }],
                    },
                ],
            };

            // Should count notes across all tracks
            assert_eq!(midi.total_notes(), 3);
        }

        #[test]
        fn test_channels_used_empty_file() {
            let midi = create_basic_midi();
            assert_eq!(midi.channels_used(), Vec::<u8>::new());
        }

        #[test]
        fn test_channels_used_single_channel() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 5,
                                note: 60,
                                velocity: 100,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 480,
                            event: Event::NoteOff {
                                channel: 5,
                                note: 60,
                                velocity: 0,
                            },
                        },
                    ],
                }],
            };

            assert_eq!(midi.channels_used(), vec![5]);
        }

        #[test]
        fn test_channels_used_multiple_channels() {
            let midi = create_midi_with_notes();

            // Should return sorted unique channels
            assert_eq!(midi.channels_used(), vec![0, 1]);
        }

        #[test]
        fn test_channels_used_all_16_channels() {
            let mut events = Vec::new();

            // Add events on all 16 MIDI channels (0-15)
            for channel in 0..16 {
                events.push(TimedEvent {
                    delta_ticks: 0,
                    event: Event::NoteOn {
                        channel,
                        note: 60,
                        velocity: 100,
                    },
                });
            }

            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track { events }],
            };

            assert_eq!(midi.channels_used(), (0..16).collect::<Vec<u8>>());
        }

        #[test]
        fn test_channels_used_excludes_meta_events() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 60,
                                velocity: 100,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange {
                                microseconds_per_quarter: 500_000,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::EndOfTrack,
                        },
                    ],
                }],
            };

            // Should only include channel from NoteOn
            assert_eq!(midi.channels_used(), vec![0]);
        }

        #[test]
        fn test_channels_used_deduplicates() {
            let midi = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 1,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 3,
                                note: 60,
                                velocity: 100,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 1,
                                note: 64,
                                velocity: 80,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 3, // Duplicate
                                note: 67,
                                velocity: 90,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 1, // Duplicate
                                note: 70,
                                velocity: 85,
                            },
                        },
                    ],
                }],
            };

            // Should return sorted unique channels
            assert_eq!(midi.channels_used(), vec![1, 3]);
        }
    }

    mod event_tests {
        use super::*;

        #[test]
        fn test_channel_note_on() {
            let event = Event::NoteOn {
                channel: 5,
                note: 60,
                velocity: 100,
            };
            assert_eq!(event.channel(), Some(5));
        }

        #[test]
        fn test_channel_note_off() {
            let event = Event::NoteOff {
                channel: 3,
                note: 60,
                velocity: 0,
            };
            assert_eq!(event.channel(), Some(3));
        }

        #[test]
        fn test_channel_aftertouch() {
            let event = Event::Aftertouch {
                channel: 7,
                note: 60,
                pressure: 50,
            };
            assert_eq!(event.channel(), Some(7));
        }

        #[test]
        fn test_channel_control_change() {
            let event = Event::ControlChange {
                channel: 10,
                controller: 7,
                value: 100,
            };
            assert_eq!(event.channel(), Some(10));
        }

        #[test]
        fn test_channel_program_change() {
            let event = Event::ProgramChange {
                channel: 15,
                program: 0,
            };
            assert_eq!(event.channel(), Some(15));
        }

        #[test]
        fn test_channel_channel_aftertouch() {
            let event = Event::ChannelAftertouch {
                channel: 2,
                pressure: 64,
            };
            assert_eq!(event.channel(), Some(2));
        }

        #[test]
        fn test_channel_pitch_bend() {
            let event = Event::PitchBend {
                channel: 8,
                value: 0,
            };
            assert_eq!(event.channel(), Some(8));
        }

        #[test]
        fn test_channel_tempo_change_returns_none() {
            let event = Event::TempoChange {
                microseconds_per_quarter: 500_000,
            };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_time_signature_returns_none() {
            let event = Event::TimeSignature {
                numerator: 4,
                denominator: 4,
                clocks_per_click: 24,
                thirty_seconds_per_quarter: 8,
            };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_key_signature_returns_none() {
            let event = Event::KeySignature {
                sharps_flats: 0,
                is_minor: false,
            };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_text_returns_none() {
            let event = Event::Text {
                text_type: TextType::TrackName,
                text: "Piano".to_string(),
            };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_end_of_track_returns_none() {
            let event = Event::EndOfTrack;
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_sysex_returns_none() {
            let event = Event::SysEx {
                data: vec![0xF0, 0x7E, 0x7F, 0xF7],
            };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_unknown_returns_none() {
            let event = Event::Unknown {
                status: 0xFF,
                data: vec![0x01, 0x02],
            };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_is_note_for_note_on() {
            let event = Event::NoteOn {
                channel: 0,
                note: 60,
                velocity: 100,
            };
            assert!(event.is_note());
        }

        #[test]
        fn test_is_note_for_note_off() {
            let event = Event::NoteOff {
                channel: 0,
                note: 60,
                velocity: 0,
            };
            assert!(event.is_note());
        }

        #[test]
        fn test_is_note_for_control_change() {
            let event = Event::ControlChange {
                channel: 0,
                controller: 7,
                value: 100,
            };
            assert!(!event.is_note());
        }

        #[test]
        fn test_is_note_for_program_change() {
            let event = Event::ProgramChange {
                channel: 0,
                program: 5,
            };
            assert!(!event.is_note());
        }

        #[test]
        fn test_is_note_for_tempo_change() {
            let event = Event::TempoChange {
                microseconds_per_quarter: 500_000,
            };
            assert!(!event.is_note());
        }

        #[test]
        fn test_is_note_for_aftertouch() {
            let event = Event::Aftertouch {
                channel: 0,
                note: 60,
                pressure: 50,
            };
            assert!(!event.is_note());
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_serialize_deserialize_midi_file() {
            let original = MidiFile {
                header: Header {
                    format: 1,
                    num_tracks: 2,
                    ticks_per_quarter_note: 480,
                },
                tracks: vec![
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 60,
                                velocity: 100,
                            },
                        }],
                    },
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange {
                                microseconds_per_quarter: 500_000,
                            },
                        }],
                    },
                ],
            };

            let json = serde_json::to_string(&original).expect("Failed to serialize");
            let deserialized: MidiFile =
                serde_json::from_str(&json).expect("Failed to deserialize");

            // Verify structure
            assert_eq!(deserialized.header.format, 1);
            assert_eq!(deserialized.header.num_tracks, 2);
            assert_eq!(deserialized.header.ticks_per_quarter_note, 480);
            assert_eq!(deserialized.tracks.len(), 2);
            assert_eq!(deserialized.tracks[0].events.len(), 1);
            assert_eq!(deserialized.tracks[1].events.len(), 1);
        }

        #[test]
        fn test_serialize_deserialize_all_event_types() {
            let events = vec![
                Event::NoteOn {
                    channel: 0,
                    note: 60,
                    velocity: 100,
                },
                Event::NoteOff {
                    channel: 0,
                    note: 60,
                    velocity: 64,
                },
                Event::Aftertouch {
                    channel: 0,
                    note: 60,
                    pressure: 50,
                },
                Event::ControlChange {
                    channel: 0,
                    controller: 7,
                    value: 100,
                },
                Event::ProgramChange {
                    channel: 0,
                    program: 5,
                },
                Event::ChannelAftertouch {
                    channel: 0,
                    pressure: 64,
                },
                Event::PitchBend {
                    channel: 0,
                    value: 8192,
                },
                Event::TempoChange {
                    microseconds_per_quarter: 500_000,
                },
                Event::TimeSignature {
                    numerator: 4,
                    denominator: 4,
                    clocks_per_click: 24,
                    thirty_seconds_per_quarter: 8,
                },
                Event::KeySignature {
                    sharps_flats: -2,
                    is_minor: true,
                },
                Event::Text {
                    text_type: TextType::TrackName,
                    text: "Piano".to_string(),
                },
                Event::EndOfTrack,
                Event::SysEx {
                    data: vec![0xF0, 0x7E, 0x7F, 0xF7],
                },
                Event::Unknown {
                    status: 0xFF,
                    data: vec![0x01, 0x02],
                },
            ];

            for original_event in events {
                let json =
                    serde_json::to_string(&original_event).expect("Failed to serialize event");
                let _deserialized: Event =
                    serde_json::from_str(&json).expect("Failed to deserialize event");
                // If we get here, serialization round-trip succeeded
            }
        }

        #[test]
        fn test_serialize_text_types() {
            let text_types = vec![
                TextType::Text,
                TextType::Copyright,
                TextType::TrackName,
                TextType::InstrumentName,
                TextType::Lyric,
                TextType::Marker,
                TextType::CuePoint,
            ];

            for original_type in text_types {
                let json =
                    serde_json::to_string(&original_type).expect("Failed to serialize TextType");
                let _deserialized: TextType =
                    serde_json::from_str(&json).expect("Failed to deserialize TextType");
                // If we get here, serialization round-trip succeeded
            }
        }
    }

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_header_format_0() {
            let header = Header {
                format: 0,
                num_tracks: 1,
                ticks_per_quarter_note: 96,
            };
            assert_eq!(header.format, 0);
        }

        #[test]
        fn test_header_format_2() {
            let header = Header {
                format: 2,
                num_tracks: 5,
                ticks_per_quarter_note: 960,
            };
            assert_eq!(header.format, 2);
        }

        #[test]
        fn test_high_ticks_per_quarter() {
            let header = Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 960, // High resolution
            };
            assert_eq!(header.ticks_per_quarter_note, 960);
        }

        #[test]
        fn test_low_ticks_per_quarter() {
            let header = Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 96, // Low resolution
            };
            assert_eq!(header.ticks_per_quarter_note, 96);
        }

        #[test]
        fn test_pitch_bend_positive() {
            let event = Event::PitchBend {
                channel: 0,
                value: 8192, // Center position
            };
            if let Event::PitchBend { value, .. } = event {
                assert_eq!(value, 8192);
            }
        }

        #[test]
        fn test_pitch_bend_negative() {
            let event = Event::PitchBend {
                channel: 0,
                value: -8192, // Max down
            };
            if let Event::PitchBend { value, .. } = event {
                assert_eq!(value, -8192);
            }
        }

        #[test]
        fn test_key_signature_sharps() {
            let event = Event::KeySignature {
                sharps_flats: 4, // E major
                is_minor: false,
            };
            if let Event::KeySignature {
                sharps_flats,
                is_minor,
            } = event
            {
                assert_eq!(sharps_flats, 4);
                assert!(!is_minor);
            }
        }

        #[test]
        fn test_key_signature_flats() {
            let event = Event::KeySignature {
                sharps_flats: -3, // Eb major
                is_minor: false,
            };
            if let Event::KeySignature {
                sharps_flats,
                is_minor,
            } = event
            {
                assert_eq!(sharps_flats, -3);
                assert!(!is_minor);
            }
        }

        #[test]
        fn test_very_large_delta_ticks() {
            let event = TimedEvent {
                delta_ticks: u32::MAX,
                event: Event::EndOfTrack,
            };
            assert_eq!(event.delta_ticks, u32::MAX);
        }

        #[test]
        fn test_empty_sysex() {
            let event = Event::SysEx { data: vec![] };
            if let Event::SysEx { data } = event {
                assert!(data.is_empty());
            }
        }

        #[test]
        fn test_empty_text() {
            let event = Event::Text {
                text_type: TextType::Text,
                text: String::new(),
            };
            if let Event::Text { text, .. } = event {
                assert!(text.is_empty());
            }
        }

        #[test]
        fn test_unicode_text() {
            let event = Event::Text {
                text_type: TextType::TrackName,
                text: "Piano Track".to_string(),
            };
            if let Event::Text { text, .. } = event {
                assert_eq!(text, "Piano Track");
            }
        }
    }
}
