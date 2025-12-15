//! Analysis result database models
//!
//! Represents analysis outputs including BPM detection, key detection,
//! drum patterns, and auto-generated tags.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Analysis result record from automated MIDI analysis.
///
/// Stores BPM, key, tags, and confidence scores for each analyzed file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct AnalysisResult {
    /// Primary key (BIGSERIAL)
    pub id: i64,

    /// Foreign key to files table
    pub file_id: i64,

    /// Detected BPM (30-300 range)
    pub bpm: Option<f64>,

    /// BPM detection confidence (0.0-1.0)
    pub bpm_confidence: Option<f64>,

    /// Detected musical key (e.g., "C", "Am", "F#m")
    pub key_signature: Option<String>,

    /// Key detection confidence (0.0-1.0)
    pub key_confidence: Option<f64>,

    /// Auto-generated tags (JSONB array)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub tags: Value,

    /// Overall analysis confidence score
    pub confidence: f64,

    /// Type of analysis performed (e.g., "full", "quick", "drum-only")
    pub analysis_type: String,

    /// Version of analyzer that produced this result
    pub analyzer_version: String,

    /// Controller analysis data (JSONB)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub controller_data: Option<Value>,

    /// Articulation analysis data (JSONB)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub articulation_data: Option<Value>,

    /// Structure/form analysis data (JSONB)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub structure_data: Option<Value>,

    /// When this analysis was created
    pub created_at: Option<DateTime<Utc>>,

    /// When this analysis was last updated
    pub updated_at: Option<DateTime<Utc>>,
}

/// Data required to create a new analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnalysisResult {
    pub file_id: i64,
    pub bpm: Option<f64>,
    pub bpm_confidence: Option<f64>,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f64>,
    pub tags: Value,
    pub confidence: f64,
    pub analysis_type: String,
    pub analyzer_version: String,
    pub controller_data: Option<Value>,
    pub articulation_data: Option<Value>,
    pub structure_data: Option<Value>,
}

/// Optional fields for updating an analysis result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateAnalysisResult {
    pub bpm: Option<f64>,
    pub bpm_confidence: Option<f64>,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f64>,
    pub tags: Option<Value>,
    pub confidence: Option<f64>,
    pub controller_data: Option<Value>,
    pub articulation_data: Option<Value>,
    pub structure_data: Option<Value>,
}

/// Drum pattern analysis result.
///
/// Specialized analysis for drum/percussion MIDI files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct DrumPattern {
    /// Primary key
    pub id: i64,

    /// Foreign key to files table
    pub file_id: i64,

    /// Pattern type (groove, fill, intro, ending, etc.)
    pub pattern_type: Option<String>,

    /// Rhythmic feel (straight, swing, shuffle)
    pub feel: Option<String>,

    /// Detected techniques (ghost notes, double bass, etc.)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub techniques: Option<Vec<String>>,

    /// Instrument types detected (kick, snare, hihat, etc.)
    #[cfg_attr(feature = "database", sqlx(default))]
    pub instruments: Option<Vec<String>>,

    /// Pattern complexity score (0.0-1.0)
    pub complexity: Option<f64>,

    /// Swing amount (0.0-1.0)
    pub swing_factor: Option<f64>,

    /// Whether this is a loop
    pub is_loop: Option<bool>,

    /// Loop length in beats
    pub loop_length_beats: Option<i32>,

    /// Syncopation score
    pub syncopation: Option<f64>,

    /// When this pattern was analyzed
    pub created_at: Option<DateTime<Utc>>,
}

/// Data required to create a drum pattern record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDrumPattern {
    pub file_id: i64,
    pub pattern_type: Option<String>,
    pub feel: Option<String>,
    pub techniques: Option<Vec<String>>,
    pub instruments: Option<Vec<String>>,
    pub complexity: Option<f64>,
    pub swing_factor: Option<f64>,
    pub is_loop: Option<bool>,
    pub loop_length_beats: Option<i32>,
    pub syncopation: Option<f64>,
}

/// Tag definition from the tags table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct Tag {
    /// Primary key (SERIAL)
    pub id: i32,

    /// Tag name (unique)
    pub name: String,

    /// Tag category (instrument, genre, pattern, etc.)
    pub category: Option<String>,

    /// How many files use this tag
    pub usage_count: Option<i32>,

    /// When this tag was created
    pub created_at: Option<DateTime<Utc>>,
}

/// File-tag relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct FileTag {
    pub file_id: i64,
    pub tag_id: i32,
    pub added_at: Option<DateTime<Utc>>,
    pub added_by: Option<String>,
}

/// Data required to create a new tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub category: Option<String>,
}

/// Optional fields for updating a tag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub category: Option<String>,
}

/// Tag with usage count for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "database", derive(sqlx::FromRow))]
pub struct TagWithCount {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i64,
    pub created_at: Option<DateTime<Utc>>,
}

impl AnalysisResult {
    /// Get tags as a Vec of strings.
    pub fn tags_as_vec(&self) -> Vec<String> {
        match &self.tags {
            Value::Array(arr) => arr.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            _ => Vec::new(),
        }
    }

    /// Check if analysis is high confidence (>0.8).
    #[must_use]
    pub fn is_high_confidence(&self) -> bool {
        self.confidence > 0.8
    }

    /// Check if BPM was detected.
    #[must_use]
    pub fn has_bpm(&self) -> bool {
        self.bpm.is_some()
    }

    /// Check if key was detected.
    #[must_use]
    pub fn has_key(&self) -> bool {
        self.key_signature.is_some()
    }
}

impl CreateAnalysisResult {
    /// Create a new analysis result with minimal fields.
    #[must_use]
    pub fn new(file_id: i64, analysis_type: &str, analyzer_version: &str) -> Self {
        Self {
            file_id,
            bpm: None,
            bpm_confidence: None,
            key_signature: None,
            key_confidence: None,
            tags: Value::Array(Vec::new()),
            confidence: 0.0,
            analysis_type: analysis_type.to_string(),
            analyzer_version: analyzer_version.to_string(),
            controller_data: None,
            articulation_data: None,
            structure_data: None,
        }
    }

    /// Set BPM detection result.
    #[must_use]
    pub fn with_bpm(mut self, bpm: f64, confidence: f64) -> Self {
        self.bpm = Some(bpm);
        self.bpm_confidence = Some(confidence);
        self
    }

    /// Set key detection result.
    #[must_use]
    pub fn with_key(mut self, key: &str, confidence: f64) -> Self {
        self.key_signature = Some(key.to_string());
        self.key_confidence = Some(confidence);
        self
    }

    /// Set tags.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Value::Array(tags.into_iter().map(Value::String).collect());
        self
    }

    /// Set overall confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

impl DrumPattern {
    /// Check if this is a groove pattern.
    #[must_use]
    pub fn is_groove(&self) -> bool {
        self.pattern_type.as_ref().is_some_and(|t| t.to_lowercase() == "groove")
    }

    /// Check if this has swing feel.
    #[must_use]
    pub fn has_swing(&self) -> bool {
        self.swing_factor.is_some_and(|f| f > 0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_analysis_result() {
        let result = CreateAnalysisResult::new(1, "full", "1.0.0")
            .with_bpm(120.0, 0.95)
            .with_key("Am", 0.87)
            .with_tags(vec!["drums".to_string(), "rock".to_string()])
            .with_confidence(0.9);

        assert_eq!(result.file_id, 1);
        assert_eq!(result.bpm, Some(120.0));
        assert_eq!(result.key_signature, Some("Am".to_string()));
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_tags_as_vec() {
        let result = AnalysisResult {
            id: 1,
            file_id: 1,
            bpm: Some(120.0),
            bpm_confidence: Some(0.95),
            key_signature: Some("Am".to_string()),
            key_confidence: Some(0.87),
            tags: serde_json::json!(["drums", "rock", "loop"]),
            confidence: 0.9,
            analysis_type: "full".to_string(),
            analyzer_version: "1.0.0".to_string(),
            controller_data: None,
            articulation_data: None,
            structure_data: None,
            created_at: None,
            updated_at: None,
        };

        let tags = result.tags_as_vec();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&"drums".to_string()));
        assert!(result.is_high_confidence());
        assert!(result.has_bpm());
        assert!(result.has_key());
    }

    #[test]
    fn test_create_tag() {
        let tag = CreateTag { name: "drums".to_string(), category: Some("instrument".to_string()) };

        assert_eq!(tag.name, "drums");
        assert_eq!(tag.category, Some("instrument".to_string()));
    }
}
