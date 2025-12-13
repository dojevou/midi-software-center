/// ML Feature Extraction for MIDI Files
///
/// Ported from ~/midi/original/services/tag_manager/app.py
/// Implements feature extraction compatible with ONNX genre/mood models.
///
/// **Feature Vector (16 features → 128 padded):**
/// - Pitch class histogram (12 dims) - normalized note distribution
/// - BPM/300.0 (1 dim) - tempo normalized to [0, 1] range
/// - density/50.0 (1 dim) - notes per second, capped at 1.0
/// - length_s/600.0 (1 dim) - duration in seconds, capped at 1.0
/// - drum_ratio (1 dim) - fraction of notes on channel 9
///
/// **Genre Labels:** pop, rock, jazz, classical, hiphop, edm, ambient
/// **Mood Labels:** happy, calm, energetic, dark, sad, bright
///
/// This module provides feature extraction and heuristic-based classification.
/// When an ONNX model is available, it can be loaded via the `tract-onnx` crate.
use midi_library_shared::core::midi::types::{Event, MidiFile};

/// Feature vector size expected by ONNX model
pub const MODEL_INPUT_SIZE: usize = 128;

/// Genre labels matching Python tag_manager
pub const GENRE_LABELS: [&str; 7] =
    ["pop", "rock", "jazz", "classical", "hiphop", "edm", "ambient"];

/// Mood labels matching Python tag_manager
pub const MOOD_LABELS: [&str; 6] = ["happy", "calm", "energetic", "dark", "sad", "bright"];

/// Extracted ML features from MIDI file
#[derive(Debug, Clone)]
pub struct MidiFeatures {
    /// 12-dimensional pitch class histogram (C, C#, D, ... B)
    pub pitch_class_histogram: [f32; 12],
    /// BPM (beats per minute)
    pub bpm: f32,
    /// Note density (notes per second)
    pub density: f32,
    /// Duration in seconds
    pub length_s: f32,
    /// Fraction of notes on drum channel (channel 9)
    pub drum_ratio: f32,
    /// Whether this appears to be a drum-focused file
    pub is_drum_file: bool,
}

impl Default for MidiFeatures {
    fn default() -> Self {
        Self {
            pitch_class_histogram: [0.0; 12],
            bpm: 120.0,
            density: 0.0,
            length_s: 0.0,
            drum_ratio: 0.0,
            is_drum_file: false,
        }
    }
}

impl MidiFeatures {
    /// Convert to 128-dimensional feature vector for ONNX model
    pub fn to_feature_vector(&self) -> [f32; MODEL_INPUT_SIZE] {
        let mut feats = [0.0f32; MODEL_INPUT_SIZE];

        // First 12 features: pitch class histogram
        for (i, &val) in self.pitch_class_histogram.iter().enumerate() {
            feats[i] = val;
        }

        // Features 12-15: normalized auxiliary features
        feats[12] = self.bpm / 300.0; // BPM normalized
        feats[13] = (self.density / 50.0).min(1.0); // Density capped
        feats[14] = (self.length_s / 600.0).min(1.0); // Length capped
        feats[15] = self.drum_ratio; // Drum ratio

        // Features 16-127: zero padding
        // (already initialized to 0)

        feats
    }

    /// Extract features from a MidiFile
    pub fn from_midi_file(midi_file: &MidiFile, detected_bpm: Option<f64>) -> Self {
        let mut pcs = [0.0f32; 12];
        let mut total_notes = 0u32;
        let mut drum_notes = 0u32;

        // Count notes and build pitch class histogram
        // Track events contain NoteOn/NoteOff with channel and note information
        for track in &midi_file.tracks {
            for timed_event in &track.events {
                // Extract NoteOn events (NoteOff doesn't add to histogram)
                if let Event::NoteOn { channel, note, velocity } = &timed_event.event {
                    // Skip velocity 0 (which is effectively a NoteOff)
                    if *velocity > 0 {
                        total_notes += 1;

                        // Channel 9 (zero-indexed) is GM drum channel
                        if *channel == 9 {
                            drum_notes += 1;
                        } else {
                            // Add to pitch class histogram (non-drum notes only)
                            let pitch_class = (*note % 12) as usize;
                            pcs[pitch_class] += 1.0;
                        }
                    }
                }
            }
        }

        // Normalize pitch class histogram
        let pcs_sum: f32 = pcs.iter().sum();
        if pcs_sum > 0.0 {
            for val in &mut pcs {
                *val /= pcs_sum;
            }
        }

        // Calculate duration (method takes default_tempo_bpm parameter)
        let default_bpm = detected_bpm.unwrap_or(120.0);
        let length_s = midi_file.duration_seconds(default_bpm) as f32;

        // Calculate density
        let density = if length_s > 0.001 {
            total_notes as f32 / length_s
        } else {
            0.0
        };

        // Calculate drum ratio
        let drum_ratio = if total_notes > 0 {
            drum_notes as f32 / total_notes as f32
        } else {
            0.0
        };

        // Use detected BPM or default
        let bpm = default_bpm as f32;

        Self {
            pitch_class_histogram: pcs,
            bpm,
            density,
            length_s,
            drum_ratio,
            is_drum_file: drum_ratio >= 0.5,
        }
    }
}

/// ML-based genre/mood classification results
#[derive(Debug, Clone)]
pub struct MlClassification {
    /// Predicted genre with confidence
    pub genre: String,
    pub genre_confidence: f32,
    /// Predicted mood with confidence
    pub mood: String,
    pub mood_confidence: f32,
    /// Heuristic key detection from pitch class histogram
    pub key: String,
    /// Primary instrument heuristic
    pub instrument: String,
    /// Complexity score (based on density)
    pub complexity: f32,
}

impl Default for MlClassification {
    fn default() -> Self {
        Self {
            genre: "unknown".to_string(),
            genre_confidence: 0.5,
            mood: "neutral".to_string(),
            mood_confidence: 0.5,
            key: "Cmaj".to_string(),
            instrument: "piano".to_string(),
            complexity: 0.5,
        }
    }
}

/// Heuristic-based genre/mood classifier
///
/// This implementation uses rule-based heuristics based on musical features.
/// Can be replaced with ONNX model inference when a trained model is available.
pub struct HeuristicClassifier;

impl HeuristicClassifier {
    /// Classify genre based on musical features
    pub fn classify_genre(features: &MidiFeatures) -> (String, f32) {
        let bpm = features.bpm;
        let drum_ratio = features.drum_ratio;
        let density = features.density;

        // Heuristic rules based on BPM ranges and characteristics
        // These match common music production conventions

        if drum_ratio >= 0.8 {
            // Very drum-heavy: likely EDM or Hip-Hop
            if (125.0..=145.0).contains(&bpm) {
                return ("edm".to_string(), 0.75);
            } else if (85.0..=100.0).contains(&bpm) {
                return ("hiphop".to_string(), 0.70);
            }
        }

        // BPM-based genre detection
        if bpm <= 70.0 {
            // Very slow: classical, ambient, or ballad
            if density < 2.0 {
                return ("ambient".to_string(), 0.70);
            } else {
                return ("classical".to_string(), 0.65);
            }
        } else if (70.0..=100.0).contains(&bpm) {
            // Medium-slow: jazz, pop ballad, hip-hop
            if drum_ratio >= 0.5 {
                return ("hiphop".to_string(), 0.70);
            } else if density > 5.0 {
                return ("jazz".to_string(), 0.65);
            } else {
                return ("pop".to_string(), 0.60);
            }
        } else if (100.0..=125.0).contains(&bpm) {
            // Medium: pop, rock, jazz
            if density > 8.0 {
                return ("rock".to_string(), 0.65);
            } else if (0.3..0.8).contains(&drum_ratio) {
                return ("pop".to_string(), 0.65);
            } else {
                return ("jazz".to_string(), 0.60);
            }
        } else if (125.0..=145.0).contains(&bpm) {
            // Dance tempo: EDM, house
            return ("edm".to_string(), 0.75);
        } else if (145.0..=180.0).contains(&bpm) {
            // Fast: rock, drum & bass
            return ("rock".to_string(), 0.70);
        } else if bpm > 180.0 {
            // Very fast: punk, metal, drum & bass
            return ("rock".to_string(), 0.65);
        }

        // Default fallback
        ("pop".to_string(), 0.50)
    }

    /// Classify mood based on musical features
    pub fn classify_mood(features: &MidiFeatures) -> (String, f32) {
        let bpm = features.bpm;
        let density = features.density;
        let pcs = &features.pitch_class_histogram;

        // Determine major/minor tendency from pitch class histogram
        // Major keys emphasize pitch classes 0, 4, 7 (C, E, G for C major)
        // Minor keys emphasize pitch classes 0, 3, 7 (C, Eb, G for C minor)
        let major_score = pcs[0] + pcs[4] + pcs[7]; // Root, major 3rd, 5th
        let minor_score = pcs[0] + pcs[3] + pcs[7]; // Root, minor 3rd, 5th
        let is_minor_leaning = minor_score > major_score * 1.1;

        // Heuristic mood classification
        if bpm <= 80.0 {
            if is_minor_leaning {
                return ("sad".to_string(), 0.70);
            } else {
                return ("calm".to_string(), 0.70);
            }
        } else if (80.0..=110.0).contains(&bpm) {
            if is_minor_leaning {
                return ("dark".to_string(), 0.65);
            } else if density > 5.0 {
                return ("happy".to_string(), 0.65);
            } else {
                return ("calm".to_string(), 0.60);
            }
        } else if (110.0..=135.0).contains(&bpm) {
            if is_minor_leaning {
                return ("dark".to_string(), 0.65);
            } else {
                return ("happy".to_string(), 0.70);
            }
        } else if bpm >= 135.0 {
            if density > 10.0 {
                return ("energetic".to_string(), 0.80);
            } else if is_minor_leaning {
                return ("dark".to_string(), 0.70);
            } else {
                return ("bright".to_string(), 0.70);
            }
        }

        // Default fallback
        ("neutral".to_string(), 0.50)
    }

    /// Detect key from pitch class histogram
    pub fn detect_key(features: &MidiFeatures) -> String {
        let pcs = &features.pitch_class_histogram;

        // Find the pitch class with highest weight
        let mut max_idx = 0;
        let mut max_val = 0.0f32;
        for (i, &val) in pcs.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }

        // Key names (assuming major for simplicity)
        const KEY_NAMES: [&str; 12] =
            ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

        // Check if minor by comparing major 3rd vs minor 3rd
        let major_third_idx = (max_idx + 4) % 12;
        let minor_third_idx = (max_idx + 3) % 12;

        let is_minor = pcs[minor_third_idx] > pcs[major_third_idx];

        if is_minor {
            format!("{}m", KEY_NAMES[max_idx])
        } else {
            format!("{}maj", KEY_NAMES[max_idx])
        }
    }

    /// Full classification pipeline
    pub fn classify(features: &MidiFeatures) -> MlClassification {
        let (genre, genre_confidence) = Self::classify_genre(features);
        let (mood, mood_confidence) = Self::classify_mood(features);
        let key = Self::detect_key(features);

        // Instrument heuristic - check both is_drum_file flag and drum_ratio threshold
        let instrument = if features.is_drum_file || features.drum_ratio >= 0.5 {
            "drums".to_string()
        } else if features.density < 2.0 {
            "pad".to_string()
        } else if features.density > 10.0 {
            "lead".to_string()
        } else {
            "piano".to_string()
        };

        // Complexity based on note density
        let complexity = (features.density / 20.0).min(1.0);

        MlClassification {
            genre,
            genre_confidence,
            mood,
            mood_confidence,
            key,
            instrument,
            complexity,
        }
    }
}

/// Combined ML and heuristic tagger
///
/// Extracts features and classifies MIDI files using ML-style features.
/// When ONNX model is available, this can be extended to use it.
pub struct MlTagger;

impl MlTagger {
    /// Tag a MIDI file using ML features and heuristic classification
    pub fn tag_midi_file(midi_file: &MidiFile, detected_bpm: Option<f64>) -> MlClassification {
        let features = MidiFeatures::from_midi_file(midi_file, detected_bpm);
        HeuristicClassifier::classify(&features)
    }

    /// Extract raw features (useful for batch processing or ONNX inference)
    pub fn extract_features(midi_file: &MidiFile, detected_bpm: Option<f64>) -> MidiFeatures {
        MidiFeatures::from_midi_file(midi_file, detected_bpm)
    }

    /// Get feature vector for ONNX model (128 dimensions)
    pub fn get_feature_vector(
        midi_file: &MidiFile,
        detected_bpm: Option<f64>,
    ) -> [f32; MODEL_INPUT_SIZE] {
        let features = MidiFeatures::from_midi_file(midi_file, detected_bpm);
        features.to_feature_vector()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_features() {
        let features = MidiFeatures::default();
        assert_eq!(features.bpm, 120.0);
        assert_eq!(features.drum_ratio, 0.0);
        assert!(!features.is_drum_file);
    }

    #[test]
    fn test_feature_vector_size() {
        let features = MidiFeatures::default();
        let vector = features.to_feature_vector();
        assert_eq!(vector.len(), MODEL_INPUT_SIZE);
        assert_eq!(vector.len(), 128);
    }

    #[test]
    fn test_genre_edm_classification() {
        let mut features = MidiFeatures::default();
        features.bpm = 128.0;
        features.drum_ratio = 0.85;
        features.density = 8.0;

        let (genre, conf) = HeuristicClassifier::classify_genre(&features);
        assert_eq!(genre, "edm");
        assert!(conf >= 0.70);
    }

    #[test]
    fn test_genre_hiphop_classification() {
        let mut features = MidiFeatures::default();
        features.bpm = 90.0;
        features.drum_ratio = 0.85;
        features.density = 5.0;

        let (genre, conf) = HeuristicClassifier::classify_genre(&features);
        assert_eq!(genre, "hiphop");
        assert!(conf >= 0.65);
    }

    #[test]
    fn test_genre_ambient_classification() {
        let mut features = MidiFeatures::default();
        features.bpm = 60.0;
        features.drum_ratio = 0.0;
        features.density = 1.0;

        let (genre, conf) = HeuristicClassifier::classify_genre(&features);
        assert_eq!(genre, "ambient");
        assert!(conf >= 0.65);
    }

    #[test]
    fn test_mood_energetic_classification() {
        let mut features = MidiFeatures::default();
        features.bpm = 150.0;
        features.density = 15.0;

        let (mood, conf) = HeuristicClassifier::classify_mood(&features);
        assert_eq!(mood, "energetic");
        assert!(conf >= 0.75);
    }

    #[test]
    fn test_mood_calm_classification() {
        let mut features = MidiFeatures::default();
        features.bpm = 70.0;
        features.density = 2.0;

        let (mood, _) = HeuristicClassifier::classify_mood(&features);
        assert_eq!(mood, "calm");
    }

    #[test]
    fn test_key_detection_major() {
        let mut features = MidiFeatures::default();
        // Simulate C major: heavy emphasis on C (0), E (4), G (7)
        features.pitch_class_histogram[0] = 0.4; // C
        features.pitch_class_histogram[4] = 0.3; // E (major 3rd)
        features.pitch_class_histogram[7] = 0.2; // G

        let key = HeuristicClassifier::detect_key(&features);
        assert!(key.contains("C"));
        assert!(key.contains("maj"));
    }

    #[test]
    fn test_key_detection_minor() {
        let mut features = MidiFeatures::default();
        // Simulate A minor: heavy emphasis on A (9), C (0), E (4)
        features.pitch_class_histogram[9] = 0.4; // A
        features.pitch_class_histogram[0] = 0.3; // C (minor 3rd from A)
        features.pitch_class_histogram[4] = 0.2; // E

        let key = HeuristicClassifier::detect_key(&features);
        assert!(key.contains("A"));
    }

    #[test]
    fn test_full_classification() {
        let mut features = MidiFeatures::default();
        features.bpm = 128.0;
        features.drum_ratio = 0.7;
        features.density = 10.0;
        features.length_s = 180.0;

        let classification = HeuristicClassifier::classify(&features);

        // Should be EDM
        assert_eq!(classification.genre, "edm");
        // Mood should be happy or energetic at this tempo
        assert!(
            classification.mood == "happy"
                || classification.mood == "energetic"
                || classification.mood == "bright"
        );
        // Should be drums since drum_ratio >= 0.5
        assert_eq!(classification.instrument, "drums");
    }

    #[test]
    fn test_complexity_scaling() {
        let mut features = MidiFeatures::default();

        // Low density = low complexity
        features.density = 2.0;
        let class1 = HeuristicClassifier::classify(&features);

        // High density = high complexity
        features.density = 25.0;
        let class2 = HeuristicClassifier::classify(&features);

        assert!(class2.complexity > class1.complexity);
        assert!(class2.complexity <= 1.0);
    }
}
