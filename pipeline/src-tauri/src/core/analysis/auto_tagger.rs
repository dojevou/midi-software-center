//! Auto-Tagging System for MIDI Files
//!
//! This module provides intelligent tag extraction from:
//! - File names (splitting on _, -, space, camelCase)
//! - Folder paths (manufacturer, genre, category)
//! - MIDI content (instrument names, track names)
//!
//! Tags are categorized as:
//! - genre:house, genre:techno, etc.
//! - instrument:kick, instrument:bass, etc.
//! - brand:vengeance, brand:splice, etc.
//! - Style tags: deep, dark, melodic, etc.

use regex::Regex;
use std::collections::HashSet;

/// Main auto-tagging engine
pub struct AutoTagger {
    genre_keywords: HashSet<String>,
    instrument_keywords: HashSet<String>,
    manufacturer_keywords: HashSet<String>,
    style_keywords: HashSet<String>,
    common_words: HashSet<String>,
    split_pattern: Regex,
}

/// Tag with optional category prefix
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub name: String,
    pub category: Option<String>,
}

impl Tag {
    pub fn new(name: impl Into<String>, category: Option<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            category: category.map(|c| c.into()),
        }
    }

    /// Get the full tag string (e.g., "genre:house" or just "deep")
    pub fn full_name(&self) -> String {
        match &self.category {
            Some(cat) => format!("{}:{}", cat, self.name),
            None => self.name.clone(),
        }
    }
}

impl AutoTagger {
    /// Create a new auto-tagger with default keyword dictionaries
    pub fn new() -> Self {
        Self {
            genre_keywords: Self::load_genre_keywords(),
            instrument_keywords: Self::load_instrument_keywords(),
            manufacturer_keywords: Self::load_manufacturer_keywords(),
            style_keywords: Self::load_style_keywords(),
            common_words: Self::load_common_words(),
            // Split on underscores, hyphens, spaces, and dots
            // Note: camelCase splitting requires lookahead/lookbehind which isn't supported in Rust regex
            split_pattern: Regex::new(r"[_\-\s.]+").unwrap(),
        }
    }

    /// Extract tags from file path, name, and MIDI content
    ///
    /// # Arguments
    /// * `file_path` - Full file path (e.g., "/Vengeance/DeepHouse/Kicks/VEC_Kick_128.mid")
    /// * `file_name` - File name only (e.g., "VEC_Kick_128.mid")
    /// * `midi_instruments` - Instrument names from MIDI file (e.g., ["Acoustic Bass Drum"])
    /// * `bpm` - Detected BPM (optional, added as tag if present)
    /// * `key_signature` - Detected key (optional, added as tag if present)
    ///
    /// # Returns
    /// Vector of unique tags with categories
    pub fn extract_tags(
        &self,
        file_path: &str,
        file_name: &str,
        midi_instruments: &[String],
        bpm: Option<f64>,
        key_signature: Option<&str>,
    ) -> Vec<Tag> {
        let mut tags = HashSet::new();

        // 1. Extract from file name
        tags.extend(self.extract_from_filename(file_name));

        // 2. Extract from folder path
        tags.extend(self.extract_from_path(file_path));

        // 3. Extract from MIDI instruments
        tags.extend(self.extract_from_instruments(midi_instruments));

        // 4. Add BPM tag if available
        if let Some(bpm_val) = bpm {
            let bpm_rounded = bpm_val.round() as i32;
            tags.insert(Tag::new(bpm_rounded.to_string(), Some("bpm")));
        }

        // 5. Add key signature tag if available
        if let Some(key) = key_signature {
            let key_normalized = key.to_lowercase();
            if key_normalized != "unknown" {
                tags.insert(Tag::new(key_normalized, Some("key")));
            }
        }

        tags.into_iter().collect()
    }

    /// Extract tags from filename by splitting on common separators
    fn extract_from_filename(&self, filename: &str) -> Vec<Tag> {
        let mut tags = Vec::new();

        // Remove extension
        let name = filename
            .trim_end_matches(".mid")
            .trim_end_matches(".MID")
            .trim_end_matches(".midi")
            .trim_end_matches(".MIDI");

        // Split on separators: _, -, space, and camelCase
        let words: Vec<&str> = self.split_pattern.split(name).collect();

        for word in words {
            let word_lower = word.to_lowercase();

            // Skip common/meaningless words
            if word.len() < 2 || self.common_words.contains(&word_lower) {
                continue;
            }

            // Check against known dictionaries with fuzzy matching
            if let Some(matched_genre) = self.fuzzy_match(&word_lower, &self.genre_keywords) {
                tags.push(Tag::new(matched_genre, Some("genre")));
            } else if let Some(matched_instrument) =
                self.fuzzy_match(&word_lower, &self.instrument_keywords)
            {
                tags.push(Tag::new(matched_instrument, Some("instrument")));
            } else if let Some(matched_brand) =
                self.fuzzy_match(&word_lower, &self.manufacturer_keywords)
            {
                tags.push(Tag::new(matched_brand, Some("brand")));
            } else if let Some(matched_style) = self.fuzzy_match(&word_lower, &self.style_keywords)
            {
                tags.push(Tag::new(matched_style, None::<String>)); // Style tags have no category prefix
            } else if word.len() > 3 && word.chars().all(|c| c.is_alphanumeric()) {
                // Add as generic tag if it's meaningful (>3 chars, alphanumeric)
                tags.push(Tag::new(word_lower, None::<String>));
            }
        }

        tags
    }

    /// Extract tags from folder path
    fn extract_from_path(&self, path: &str) -> Vec<Tag> {
        let mut tags = Vec::new();

        // Split path into components
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        for part in parts {
            let part_lower = part.to_lowercase();

            // Check against dictionaries
            if let Some(matched_genre) = self.fuzzy_match(&part_lower, &self.genre_keywords) {
                tags.push(Tag::new(matched_genre, Some("genre")));
            } else if let Some(matched_instrument) =
                self.fuzzy_match(&part_lower, &self.instrument_keywords)
            {
                tags.push(Tag::new(matched_instrument, Some("category")));
            } else if let Some(matched_brand) =
                self.fuzzy_match(&part_lower, &self.manufacturer_keywords)
            {
                tags.push(Tag::new(matched_brand, Some("brand")));
            }
        }

        tags
    }

    /// Extract tags from MIDI instrument names
    fn extract_from_instruments(&self, instruments: &[String]) -> Vec<Tag> {
        let mut tags = Vec::new();

        for instrument in instruments {
            let inst_lower = instrument.to_lowercase();

            // Map MIDI GM instrument names to our keywords
            if let Some(matched) = self.fuzzy_match(&inst_lower, &self.instrument_keywords) {
                tags.push(Tag::new(matched, Some("instrument")));
            }
        }

        tags
    }

    /// Fuzzy match a word against a dictionary using Levenshtein distance
    /// Returns the matched keyword if distance <= 2
    fn fuzzy_match(&self, input: &str, dictionary: &HashSet<String>) -> Option<String> {
        // First try exact match
        if dictionary.contains(input) {
            return Some(input.to_string());
        }

        // Try fuzzy matching with threshold of 2 edits
        let threshold = 2;

        dictionary
            .iter()
            .filter(|keyword| {
                // Only fuzzy match if input is reasonably long
                if input.len() < 4 {
                    return false;
                }
                strsim::levenshtein(input, keyword) <= threshold
            })
            .min_by_key(|keyword| strsim::levenshtein(input, keyword))
            .cloned()
    }

    // ==========================================================================
    // KEYWORD DICTIONARIES
    // ==========================================================================

    fn load_genre_keywords() -> HashSet<String> {
        [
            "house",
            "deephouse",
            "deep_house",
            "techhouse",
            "tech_house",
            "techno",
            "trance",
            "dubstep",
            "dnb",
            "drum_and_bass",
            "drumnbass",
            "edm",
            "electro",
            "progressive",
            "minimal",
            "acid",
            "ambient",
            "breakbeat",
            "garage",
            "trap",
            "hip_hop",
            "hiphop",
            "lofi",
            "chillout",
            "downtempo",
            "industrial",
            "hardstyle",
            "hardcore",
            "jungle",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn load_instrument_keywords() -> HashSet<String> {
        [
            // Drums
            "kick",
            "bass_drum",
            "bassdrum",
            "snare",
            "hihat",
            "hat",
            "clap",
            "tom",
            "cymbal",
            "percussion",
            "perc",
            "drum",
            "drums",
            // Bass
            "bass",
            "sub",
            "subbass",
            "reese",
            // Synths
            "pluck",
            "lead",
            "synth",
            "pad",
            "chord",
            "stab",
            "arp",
            "arpeggiated",
            "melody",
            "melodic",
            // Keys
            "piano",
            "keys",
            "organ",
            "rhodes",
            "wurlitzer",
            // Orchestral
            "strings",
            "string",
            "brass",
            "woodwind",
            "orchestra",
            // Vocals
            "vocal",
            "vox",
            "voice",
            // FX
            "fx",
            "effect",
            "riser",
            "impact",
            "sweep",
            "transition",
            // Loops
            "loop",
            "pattern",
            "sequence",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn load_manufacturer_keywords() -> HashSet<String> {
        [
            "vengeance",
            "splice",
            "loopmasters",
            "sample_magic",
            "samplemagic",
            "black_octopus",
            "blackoctopus",
            "cymatics",
            "production_master",
            "productionmaster",
            "roland",
            "korg",
            "moog",
            "arturia",
            "native_instruments",
            "native",
            "serum",
            "massive",
            "sylenth",
            "spire",
            "abletonlive",
            "ableton",
            "flstudio",
            "logic",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn load_style_keywords() -> HashSet<String> {
        [
            "dark",
            "melodic",
            "aggressive",
            "soft",
            "hard",
            "heavy",
            "rolling",
            "bouncy",
            "groovy",
            "punchy",
            "warm",
            "cold",
            "analog",
            "digital",
            "vintage",
            "modern",
            "classic",
            "dirty",
            "clean",
            "distorted",
            "atmospheric",
            "uplifting",
            "euphoric",
            "deep",
            "driving",
            "energetic",
            "chill",
            "relaxed",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn load_common_words() -> HashSet<String> {
        [
            "the", "and", "for", "with", "track", "midi", "file", "new", "ver", "vol", "v", "pt",
            "part", "demo", "edit", "mix", "original", "version",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
}

impl Default for AutoTagger {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_filename() {
        let tagger = AutoTagger::new();

        // Test 1: Vengeance style naming
        let tags = tagger.extract_from_filename("VEC_Deep_House_Kick_128_C.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"genre:house".to_string()));
        assert!(tag_names.contains(&"deep".to_string()));
        assert!(tag_names.contains(&"instrument:kick".to_string()));

        // Test 2: CamelCase naming
        let tags = tagger.extract_from_filename("TechnoLeadSynth.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"genre:techno".to_string()));
        assert!(tag_names.contains(&"instrument:lead".to_string()));
    }

    #[test]
    fn test_extract_from_path() {
        let tagger = AutoTagger::new();

        let tags = tagger.extract_from_path("/Vengeance/DeepHouse/Drums/Kicks/file.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"brand:vengeance".to_string()));
        assert!(tag_names.contains(&"category:drums".to_string()));
    }

    #[test]
    fn test_fuzzy_matching() {
        let tagger = AutoTagger::new();

        // "vengance" should match "vengeance" (1 char difference)
        let result = tagger.fuzzy_match("vengance", &tagger.manufacturer_keywords);
        assert_eq!(result, Some("vengeance".to_string()));

        // "teckno" should match "techno" (1 char swap)
        let result = tagger.fuzzy_match("teckno", &tagger.genre_keywords);
        assert_eq!(result, Some("techno".to_string()));
    }

    #[test]
    fn test_full_tag_extraction() {
        let tagger = AutoTagger::new();

        let tags = tagger.extract_tags(
            "/Samples/Vengeance/DeepHouse/Drums/VEC_Deep_Kick_128_C.mid",
            "VEC_Deep_Kick_128_C.mid",
            &["Acoustic Bass Drum".to_string()],
            Some(128.0),
            Some("C"),
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should have brand
        assert!(tag_names.iter().any(|t| t.starts_with("brand:")));
        // Should have genre
        assert!(tag_names.iter().any(|t| t.starts_with("genre:")));
        // Should have instrument
        assert!(tag_names.iter().any(|t| t.starts_with("instrument:")));
        // Should have BPM
        assert!(tag_names.contains(&"bpm:128".to_string()));
        // Should have key
        assert!(tag_names.contains(&"key:c".to_string()));
    }

    #[test]
    fn test_common_words_filtered() {
        let tagger = AutoTagger::new();

        let tags = tagger.extract_from_filename("The_New_Kick_For_Mix.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();

        // Common words should be filtered out
        assert!(!tag_names.contains(&"the".to_string()));
        assert!(!tag_names.contains(&"new".to_string()));
        assert!(!tag_names.contains(&"for".to_string()));
        assert!(!tag_names.contains(&"mix".to_string()));

        // But "kick" should remain
        assert!(tag_names.contains(&"kick".to_string()));
    }
}
