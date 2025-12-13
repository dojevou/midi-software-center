use super::music::{bpm_compatibility_score, bpm_time_stretchable, key_compatibility_score};
use super::types::{CompatibilityScore, KeySignature};
/// Compatibility scoring - Overall compatibility calculation
///
/// Trusty Module: Pure function that calculates compatibility scores.
/// NO database access - receives file data as parameters.
use crate::models::midi_file::MidiFile;

/// Calculate overall compatibility score between two MIDI files
///
/// This is the main entry point for compatibility calculation.
/// It combines multiple factors with weighted scoring:
/// - 40% Key compatibility (harmonic compatibility)
/// - 40% BPM compatibility (tempo matching)
/// - 20% Category compatibility (style/instrument matching)
///
/// # Arguments
/// * `source` - The source file to compare against
/// * `candidate` - The candidate file to score
///
/// # Returns
/// CompatibilityScore with detailed breakdown
pub fn calculate_compatibility(source: &MidiFile, candidate: &MidiFile) -> CompatibilityScore {
    let mut total_score = 0.0;
    let mut explanations = Vec::new();

    // Key compatibility (40% weight)
    let key_score = if let (Some(key1_str), Some(key2_str)) =
        (&source.key_signature, &candidate.key_signature)
    {
        if let (Some(key1), Some(key2)) =
            (KeySignature::parse(key1_str), KeySignature::parse(key2_str))
        {
            let score = key_compatibility_score(&key1, &key2);

            // Use name() method to show actual key names in explanations
            if score >= 95.0 {
                explanations.push(format!(
                    "Perfect key match ({} → {})",
                    key1.name(),
                    key2.name()
                ));
            } else if score >= 85.0 {
                explanations.push(format!(
                    "Excellent key compatibility ({} → {})",
                    key1.name(),
                    key2.name()
                ));
            } else if score >= 70.0 {
                explanations.push(format!(
                    "Good key compatibility ({} → {})",
                    key1.name(),
                    key2.name()
                ));
            } else if score < 50.0 {
                explanations.push(format!(
                    "Keys may clash ({} → {})",
                    key1.name(),
                    key2.name()
                ));
            }

            total_score += score * 0.4;
            score
        } else {
            50.0 // Unknown keys
        }
    } else {
        50.0 // Missing key information
    };

    // BPM compatibility (40% weight)
    let bpm_score = if let (Some(bpm1), Some(bpm2)) = (source.bpm, candidate.bpm) {
        // Cast f64 to f32 for compatibility functions
        let score = bpm_compatibility_score(bpm1 as f32, bpm2 as f32);

        if score >= 95.0 {
            explanations.push("Nearly identical tempo".to_string());
        } else if score >= 80.0 {
            explanations.push("Similar tempo".to_string());
        } else if bpm_time_stretchable(bpm1 as f32, bpm2 as f32) {
            explanations.push("Tempo can be time-stretched".to_string());
        } else if score < 50.0 {
            explanations.push("Very different tempos".to_string());
        }

        total_score += score * 0.4;
        score
    } else {
        50.0 // Missing BPM information
    };

    // Category compatibility (20% weight)
    let category_score =
        if let (Some(cat1), Some(cat2)) = (&source.primary_category, &candidate.primary_category) {
            let score = if cat1 == cat2 {
                100.0
            } else {
                category_compatibility(cat1, cat2)
            };

            if score >= 90.0 {
                explanations.push("Same or complementary category".to_string());
            }

            total_score += score * 0.2;
            score
        } else {
            50.0
        };

    // Build base explanation
    let base_explanation = if explanations.is_empty() {
        "Limited metadata available".to_string()
    } else {
        explanations.join(". ")
    };

    // Create initial score
    let score = CompatibilityScore {
        total_score,
        key_score,
        bpm_score,
        category_score,
        explanation: base_explanation,
    };

    // Use explain_compatibility to generate enhanced final explanation
    let final_explanation = explain_compatibility(source, candidate, &score);

    CompatibilityScore {
        total_score,
        key_score,
        bpm_score,
        category_score,
        explanation: final_explanation,
    }
}

/// Determine category compatibility
///
/// Scores how well two categories work together musically.
/// Same category = 100, complementary categories = 80, different = 50.
fn category_compatibility(cat1: &str, cat2: &str) -> f32 {
    // Normalize categories
    let cat1 = cat1.to_lowercase();
    let cat2 = cat2.to_lowercase();

    // Same category = perfect
    if cat1 == cat2 {
        return 100.0;
    }

    // Define complementary categories (work well together)
    let complementary_pairs = vec![
        ("kick", "bass"),
        ("kick", "drum"),
        ("snare", "hihat"),
        ("bass", "chord"),
        ("chord", "lead"),
        ("pad", "lead"),
        ("melody", "chord"),
        ("drum", "percussion"),
    ];

    for (a, b) in complementary_pairs {
        if (cat1.contains(a) && cat2.contains(b)) || (cat1.contains(b) && cat2.contains(a)) {
            return 80.0;
        }
    }

    // Different but compatible
    50.0
}

/// Generate human-readable explanation for compatibility score
///
/// Creates a summary explanation based on the overall score.
///
/// # Arguments
/// * `source` - Source file (not currently used, but available for context)
/// * `candidate` - Candidate file (not currently used, but available for context)
/// * `score` - The compatibility score to explain
///
/// # Returns
/// Human-readable explanation string
pub fn explain_compatibility(
    _source: &MidiFile,
    _candidate: &MidiFile,
    score: &CompatibilityScore,
) -> String {
    let mut parts = Vec::new();

    if score.total_score >= 90.0 {
        parts.push("Highly compatible".to_string());
    } else if score.total_score >= 75.0 {
        parts.push("Very compatible".to_string());
    } else if score.total_score >= 60.0 {
        parts.push("Compatible".to_string());
    } else if score.total_score >= 50.0 {
        parts.push("Somewhat compatible".to_string());
    } else {
        parts.push("Limited compatibility".to_string());
    }

    parts.push(score.explanation.clone());

    parts.join(". ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_compatibility_same() {
        assert_eq!(category_compatibility("kick", "kick"), 100.0);
        assert_eq!(category_compatibility("bass", "bass"), 100.0);
    }

    #[test]
    fn test_category_compatibility_complementary() {
        assert_eq!(category_compatibility("kick", "bass"), 80.0);
        assert_eq!(category_compatibility("bass", "kick"), 80.0);
        assert_eq!(category_compatibility("lead", "pad"), 80.0);
        assert_eq!(category_compatibility("chord", "melody"), 80.0);
    }

    #[test]
    fn test_category_compatibility_different() {
        let score = category_compatibility("kick", "melody");
        assert!(score < 80.0);
        assert!(score >= 50.0);
    }

    #[test]
    fn test_explain_compatibility_high_score() {
        let score = CompatibilityScore {
            total_score: 92.0,
            key_score: 95.0,
            bpm_score: 90.0,
            category_score: 90.0,
            explanation: "Perfect key match. Similar tempo".to_string(),
        };

        let file = create_test_file();
        let explanation = explain_compatibility(&file, &file, &score);

        assert!(explanation.contains("Highly compatible"));
        assert!(explanation.contains("Perfect key match"));
    }

    #[test]
    fn test_explain_compatibility_low_score() {
        let score = CompatibilityScore {
            total_score: 45.0,
            key_score: 40.0,
            bpm_score: 50.0,
            category_score: 50.0,
            explanation: "Keys may clash. Very different tempos".to_string(),
        };

        let file = create_test_file();
        let explanation = explain_compatibility(&file, &file, &score);

        assert!(explanation.contains("Limited compatibility"));
    }

    // Helper function to create test MidiFile
    fn create_test_file() -> MidiFile {
        MidiFile {
            id: 1,
            filename: "test.mid".to_string(),
            filepath: "/test/test.mid".to_string(),
            file_size_bytes: 1024,
            content_hash: vec![],
            is_multi_track: false,
            parent_file_id: None,
            track_number: None,
            total_tracks: None,
            manufacturer: None,
            collection_name: None,
            folder_tags: vec![],
            parent_folder: None,
            num_tracks: 1,
            created_at: chrono::Utc::now(),
            analyzed_at: None,
            bpm: Some(120.0),
            key_signature: Some("C".to_string()),
            time_signature: Some("4/4".to_string()),
            duration_seconds: Some(10.0),
            total_notes: 100,
            primary_category: Some("bass".to_string()),
        }
    }
}
