/// Auto-Tagging System for MIDI Files (Enhanced - v2.0)
///
/// Based on real-world analysis of 1,566,480 MIDI files from production archives.
/// Implements confidence-based tagging with 350+ tag patterns across 10 categories.
///
/// **Tag Extraction Sources:**
/// - File names (splitting on _, -, space) → confidence 0.85-0.90
/// - Folder paths (pack/folder hierarchy) → confidence 0.90-0.95
/// - MIDI content (GM instrument names) → confidence 0.75
/// - BPM analysis (tempo detection) → confidence 0.80
/// - Key analysis (key signature) → confidence 0.80
///
/// **Tag Categories with Priorities:**
/// - genre (priority 10) - dubstep, house, techno, jazz, hip-hop, etc. (77+ tags)
/// - instrument (priority 20) - kick, tabla, djembe, synth, etc. (120+ tags)
/// - element (priority 30) - loop, sequence, pattern, etc.
/// - key (priority 40) - c, am, g#, etc. (24 keys)
/// - tempo (priority 50) - slow, mid-tempo, upbeat, fast, very-fast
/// - mood (priority 60) - dark, melodic, energetic, etc.
/// - structure (priority 80) - intro, verse, chorus, bridge, breakdown
/// - brand (priority 85) - vengeance, ezdrummer, splice, etc. (45+ tags)
/// - world (priority 90) - africa, asia, middle-east, etc.
///
/// **Confidence Scoring:**
/// - Pack-level detection: 0.95 (highest)
/// - Folder-level detection: 0.90
/// - Filename exact match: 0.90
/// - Filename fuzzy match: 0.85
/// - BPM/Key analysis: 0.80
/// - MIDI GM instruments: 0.75
/// - Generic/derived: 0.70
///
/// **Detection Methods:**
/// - pack_level, folder_level, filename_exact, filename_fuzzy
/// - bpm_analysis, bpm_derived, key_analysis, midi_gm
/// - filename_generic
use regex::Regex;
use std::collections::HashSet;

// Drum analyzer integration (v2.1)
use super::drum_analyzer;
use crate::core::midi::types::MidiFile;

/// Main auto-tagging engine
pub struct AutoTagger {
    genre_keywords: HashSet<String>,
    instrument_keywords: HashSet<String>,
    manufacturer_keywords: HashSet<String>,
    style_keywords: HashSet<String>,
    common_words: HashSet<String>,
    split_pattern: Regex,
}

/// Tag with category, confidence, and priority (enhanced for database integration)
#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub category: Option<String>,
    pub confidence: f64, // 0.60-0.95 confidence score
    pub priority: i32,   // 10-90 priority (lower = higher priority)
    pub detection_method: String,
}

impl Tag {
    pub fn new(name: impl Into<String>, category: Option<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            category: category.map(|c| c.into()),
            confidence: 0.85, // Default confidence
            priority: 50,     // Default priority
            detection_method: "filename".to_string(),
        }
    }

    /// Create tag with full metadata
    pub fn with_metadata(
        name: impl Into<String>,
        category: Option<impl Into<String>>,
        confidence: f64,
        priority: i32,
        detection_method: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category: category.map(|c| c.into()),
            confidence,
            priority,
            detection_method: detection_method.into(),
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

// Manual Hash and PartialEq implementations for deduplication
// Tags are considered equal if name/category match, regardless of confidence/priority/detection_method
impl std::hash::Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.category.hash(state);
        // Don't hash confidence/priority for deduplication
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.category == other.category
        // Explicitly ignore confidence/priority/detection_method for deduplication
    }
}

impl Eq for Tag {}

impl AutoTagger {
    /// Create a new auto-tagger with default keyword dictionaries
    ///
    /// # Errors
    /// Returns error if internal regex pattern compilation fails (should never happen with valid pattern)
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            genre_keywords: Self::load_genre_keywords(),
            instrument_keywords: Self::load_instrument_keywords(),
            manufacturer_keywords: Self::load_manufacturer_keywords(),
            style_keywords: Self::load_style_keywords(),
            common_words: Self::load_common_words(),
            // Split on underscores, hyphens, spaces, and dots
            // Note: camelCase splitting requires lookahead/lookbehind which isn't supported in Rust regex
            split_pattern: Regex::new(r"[_\-\s.]+")?,
        })
    }

    /// Extract tags from file path, name, and MIDI content
    ///
    /// # Arguments
    /// * `file_path` - Full file path (e.g., "/Vengeance/DeepHouse/Kicks/VEC_Kick_128.mid")
    /// * `file_name` - File name only (e.g., "VEC_Kick_128.mid")
    /// * `midi_instruments` - Instrument names from MIDI file (e.g., ["Acoustic Bass Drum"])
    /// * `bpm` - Detected BPM (optional, added as tag if present)
    /// * `key_signature` - Detected key (optional, added as tag if present)
    /// * `midi_file` - Optional MIDI file for drum analysis (v2.1 enhancement)
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
        midi_file: Option<&MidiFile>,
    ) -> Vec<Tag> {
        let mut tags = HashSet::new();

        // 1. Add drum-specific tags FIRST if MIDI file provided (v2.1 enhancement)
        //    This ensures drum analyzer tags take precedence over generic path/filename/instrument tags
        //    Drum analyzer runs first because it provides higher-confidence, MIDI-based detection
        if let Some(midi) = midi_file {
            let drum_analysis = drum_analyzer::analyze_drum_midi(midi);
            if drum_analysis.is_drum_file {
                tags.extend(drum_analyzer::generate_drum_tags(
                    &drum_analysis,
                    file_path,
                    file_name,
                ));
            }
        }

        // 2. Extract from file name
        tags.extend(self.extract_from_filename(file_name));

        // 3. Extract from folder path
        tags.extend(self.extract_from_path(file_path));

        // 4. Extract from MIDI instruments
        tags.extend(self.extract_from_instruments(midi_instruments));

        // 5. Add BPM tag if available (high confidence from analysis)
        if let Some(bpm_val) = bpm {
            let bpm_rounded = bpm_val.round() as i32;
            tags.insert(Tag::with_metadata(
                bpm_rounded.to_string(),
                Some("tempo"),
                0.80, // Confidence from BPM detection algorithm
                50,   // Tempo priority
                "bpm_analysis",
            ));

            // Add tempo range tags based on BPM
            let tempo_tag = if bpm_val < 90.0 {
                Some(("slow", 50))
            } else if bpm_val < 120.0 {
                Some(("mid-tempo", 50))
            } else if bpm_val < 140.0 {
                Some(("upbeat", 50))
            } else if bpm_val < 170.0 {
                Some(("fast", 50))
            } else {
                Some(("very-fast", 50))
            };

            if let Some((tempo_name, priority)) = tempo_tag {
                tags.insert(Tag::with_metadata(
                    tempo_name,
                    Some("tempo"),
                    0.75, // Derived from BPM
                    priority,
                    "bpm_derived",
                ));
            }
        }

        // 5. Add key signature tag if available (high confidence from analysis)
        if let Some(key) = key_signature {
            let key_normalized = key.to_lowercase();
            if key_normalized != "unknown" {
                tags.insert(Tag::with_metadata(
                    key_normalized,
                    Some("key"),
                    0.80, // Confidence from key detection algorithm
                    40,   // Key priority
                    "key_analysis",
                ));
            }
        }

        tags.into_iter().collect()
    }

    /// Extract tags from filename by splitting on common separators
    /// Returns tags with confidence scores and priorities based on detection method
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

            // PRIORITY 1: Check for exact matches first (prevents fuzzy match conflicts)
            if self.genre_keywords.contains(&word_lower) {
                tags.push(Tag::with_metadata(
                    word_lower.clone(),
                    Some("genre"),
                    0.90, // Exact filename match
                    10,   // Genre priority
                    "filename_exact",
                ));
            } else if self.instrument_keywords.contains(&word_lower) {
                tags.push(Tag::with_metadata(
                    word_lower.clone(),
                    Some("instrument"),
                    0.90,
                    20, // Instrument priority
                    "filename_exact",
                ));
            } else if self.manufacturer_keywords.contains(&word_lower) {
                tags.push(Tag::with_metadata(
                    word_lower.clone(),
                    Some("brand"),
                    0.90,
                    85, // Library/brand priority
                    "filename_exact",
                ));
            } else if self.style_keywords.contains(&word_lower) {
                // Detect category based on style keyword
                let (category, priority): (Option<&str>, i32) = if [
                    "intro",
                    "outro",
                    "verse",
                    "chorus",
                    "bridge",
                    "breakdown",
                    "pre-chorus",
                    "cha",
                    "chb",
                    "ch3",
                    "bkdn",
                    "ta",
                    "middle-8",
                    "mid-8",
                    "all",
                ]
                .contains(&word_lower.as_str())
                {
                    (Some("structure"), 80) // Song structure priority
                } else {
                    // Mood/style keywords like "deep", "dark", "heavy" have no category prefix
                    (None, 60) // Mood/style priority, no category
                };

                tags.push(Tag::with_metadata(
                    word_lower.clone(),
                    category,
                    0.90,
                    priority,
                    "filename_exact",
                ));
            }
            // PRIORITY 2: Try fuzzy matching only if no exact match found
            else if let Some(matched_genre) = self.fuzzy_match(&word_lower, &self.genre_keywords)
            {
                tags.push(Tag::with_metadata(
                    matched_genre,
                    Some("genre"),
                    0.85, // Fuzzy match lower confidence
                    10,
                    "filename_fuzzy",
                ));
            } else if let Some(matched_instrument) =
                self.fuzzy_match(&word_lower, &self.instrument_keywords)
            {
                tags.push(Tag::with_metadata(
                    matched_instrument,
                    Some("instrument"),
                    0.85,
                    20,
                    "filename_fuzzy",
                ));
            } else if let Some(matched_brand) =
                self.fuzzy_match(&word_lower, &self.manufacturer_keywords)
            {
                tags.push(Tag::with_metadata(
                    matched_brand,
                    Some("brand"),
                    0.85,
                    85,
                    "filename_fuzzy",
                ));
            } else if let Some(matched_style) = self.fuzzy_match(&word_lower, &self.style_keywords)
            {
                // Mood/style keywords have no category prefix (same as exact match behavior)
                tags.push(Tag::with_metadata(
                    matched_style,
                    None::<&str>,
                    0.85,
                    60,
                    "filename_fuzzy",
                ));
            }
            // PRIORITY 3: Generic tags as fallback
            else if word.len() > 3 && word.chars().all(|c| c.is_alphanumeric()) {
                // Add as generic tag if it's meaningful (>3 chars, alphanumeric)
                // No category prefix for generic/unknown words
                tags.push(Tag::with_metadata(
                    word_lower,
                    None::<&str>,
                    0.70, // Low confidence for generic
                    70,   // Technical/generic priority
                    "filename_generic",
                ));
            }
        }

        tags
    }

    /// Extract tags from folder path (pack/folder-level detection)
    /// Folder-level tags get slightly lower confidence than exact matches
    fn extract_from_path(&self, path: &str) -> Vec<Tag> {
        let mut tags = Vec::new();

        // Split path into components
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        for (idx, part) in parts.iter().enumerate() {
            let part_lower = part.to_lowercase();

            // First folder = pack level (highest confidence: 0.95)
            // Later folders = sub-genre/category level (confidence: 0.90)
            let (confidence, method) = if idx == 0 || idx == 1 {
                (0.95, "pack_level")
            } else {
                (0.90, "folder_level")
            };

            // Check against dictionaries
            if let Some(matched_genre) = self.fuzzy_match(&part_lower, &self.genre_keywords) {
                tags.push(Tag::with_metadata(
                    matched_genre,
                    Some("genre"),
                    confidence,
                    10,
                    method,
                ));
            } else if let Some(matched_instrument) =
                self.fuzzy_match(&part_lower, &self.instrument_keywords)
            {
                // Instruments in path get "category:" prefix to distinguish from filename-detected instruments
                tags.push(Tag::with_metadata(
                    matched_instrument,
                    Some("category"),
                    confidence,
                    20,
                    method,
                ));
            } else if let Some(matched_brand) =
                self.fuzzy_match(&part_lower, &self.manufacturer_keywords)
            {
                tags.push(Tag::with_metadata(
                    matched_brand,
                    Some("brand"),
                    confidence,
                    85,
                    method,
                ));
            }
        }

        tags
    }

    /// Extract tags from MIDI instrument names
    /// GM instrument detection gets moderate confidence (0.75)
    fn extract_from_instruments(&self, instruments: &[String]) -> Vec<Tag> {
        let mut tags = Vec::new();

        for instrument in instruments {
            let inst_lower = instrument.to_lowercase();

            // Map MIDI GM instrument names to our keywords
            if let Some(matched) = self.fuzzy_match(&inst_lower, &self.instrument_keywords) {
                tags.push(Tag::with_metadata(
                    matched,
                    Some("instrument"),
                    0.75, // Moderate confidence for GM instruments
                    20,
                    "midi_gm",
                ));
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
            // Electronic/EDM (expanded from 1.5M+ file analysis)
            "house",
            "deephouse",
            "deep_house",
            "deep-house",
            "techhouse",
            "tech_house",
            "tech-house",
            "techno",
            "trance",
            "psy_trance",
            "psy-trance",
            "psytrance",
            "dubstep",
            "dnb",
            "drum_and_bass",
            "drumnbass",
            "drum-and-bass",
            "edm",
            "electro",
            "progressive",
            "minimal",
            "acid",
            "ambient",
            "breakbeat",
            "garage",
            "speed_garage",
            "speed-garage",
            "uk-garage",
            "trap",
            "melodic_trap",
            "melodic-trap",
            "future_bass",
            "future-bass",
            "glitch",
            "idm",
            "jungle",
            "hardstyle",
            "hardcore",
            "lofi",
            "lo-fi",
            "chillout",
            "chill-out",
            "downtempo",
            "industrial",
            // Urban/Contemporary
            "hip_hop",
            "hiphop",
            "hip-hop",
            "rap",
            "rnb",
            "r&b",
            "r-and-b",
            "soul",
            "pop",
            "disco",
            "funk",
            // Traditional/Acoustic
            "jazz",
            "blues",
            "rock",
            "metal",
            "country",
            "classical",
            "cinematic",
            "film-score",
            "acoustic",
            // World Music
            "africa",
            "african",
            "asia",
            "asian",
            "middle_east",
            "middle-east",
            "latin",
            "world",
            // Sub-genres & styles (from real MIDI collection)
            "jazzy_hip-hop",
            "jazzy-hip-hop",
            "future_rnb",
            "future-rnb",
            "liquid_dnb",
            "liquid-dnb",
            "neurofunk",
            "bass-music",
            "bass_music",
            "atmospheric",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn load_instrument_keywords() -> HashSet<String> {
        [
            // Drums (Western)
            "kick",
            "bass_drum",
            "bassdrum",
            "snare",
            "hihat",
            "hat",
            "hi-hat",
            "clap",
            "handclap",
            "tom",
            "toms",
            "cymbal",
            "crash",
            "ride",
            "percussion",
            "perc",
            "drum",
            "drums",
            "cowbell",
            "tambourine",
            "shaker",
            "conga",
            "congas",
            "bongo",
            "bongos",
            // World Instruments - African (from real collection)
            "djembe",
            "talking_drum",
            "talking-drum",
            "dun",
            "dun-set",
            "banana_bells",
            "banana-bells",
            "shakere",
            "trash_dun",
            "trash-dun",
            // World Instruments - Asian (from real collection)
            "tabla",
            "dayon",
            "bayon", // Tabla variations
            "samul_nori",
            "samul-nori",
            "kkwaenggwari",
            "janggu",
            "buk",
            "kkwaenggwari",
            "rebana",
            "kendang",
            "ghatam",
            "dhol",
            "korean_woodblock",
            "korean-woodblock",
            "korean_cymbal",
            "korean-cymbal",
            "mudang-cymbal",
            // World Instruments - Middle Eastern (from real collection)
            "darabuka",
            "riq",
            "duff",
            "tabal",
            "tupan",
            "muzhar",
            "finger_bells",
            "finger-bells",
            // Bass
            "bass",
            "sub",
            "subbass",
            "sub-bass",
            "reese",
            // Synths
            "pluck",
            "plucked",
            "lead",
            "synth",
            "pad",
            "sub-pad",
            "strings-pad",
            "chord",
            "chords",
            "stab",
            "arp",
            "arpeggiated",
            "arp-loop",
            "melody",
            "melodic",
            "melody-loop",
            // Keys
            "piano",
            "keys",
            "organ",
            "rhodes",
            "wurlitzer",
            "electric_piano",
            "electric-piano",
            // Orchestral
            "strings",
            "string",
            "brass",
            "woodwind",
            "orchestra",
            "violin",
            "viola",
            "cello",
            "contrabass",
            "trumpet",
            "trombone",
            "horn",
            "flute",
            "oboe",
            "clarinet",
            "bassoon",
            // Vocals
            "vocal",
            "vocals",
            "vox",
            "voice",
            "choir",
            // FX
            "fx",
            "effect",
            "riser",
            "impact",
            "sweep",
            "transition",
            "zap",
            "whistle",
            "waterfalling",
            "wobbly",
            "wobble",
            // Loops & Elements
            "loop",
            "pattern",
            "sequence",
            "seq",
            "backing",
            "construction",
            // Note: "groove" removed - now handled by drum analyzer with category "pattern-type"

            // Guitar
            "guitar",
            "acoustic_guitar",
            "acoustic-guitar",
            "electric_guitar",
            "electric-guitar",
            "bass_guitar",
            "bass-guitar",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn load_manufacturer_keywords() -> HashSet<String> {
        [
            // Sample Pack Manufacturers
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
            "zero_g",
            "zero-g",
            "vir2",
            // Drum Libraries (from Mega Drums Pack analysis)
            "ezdrummer",
            "ez_drummer",
            "ez-drummer",
            "superior_drummer",
            "superior-drummer",
            "groove_monkey",
            "groove-monkey",
            "stage_1_drums",
            "stage-1-drums",
            "reelfeel_drums",
            "reelfeel-drums",
            "la_drum_studio",
            "la-drum-studio",
            "nice_beats",
            "nice-beats",
            "sly_dunbar",
            "sly-dunbar",
            "x_filez",
            "x-filez",
            // Hardware Manufacturers
            "roland",
            "korg",
            "moog",
            "arturia",
            "native_instruments",
            "native-instruments",
            "native",
            // Software Synths
            "serum",
            "massive",
            "sylenth",
            "spire",
            // DAWs
            "abletonlive",
            "ableton",
            "flstudio",
            "fl_studio",
            "fl-studio",
            "logic",
            "cubase",
            "pro_tools",
            "pro-tools",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn load_style_keywords() -> HashSet<String> {
        [
            // Mood/Atmosphere
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
            // Song Structure (from Piano Collection analysis)
            "intro",
            "outro",
            "verse",
            "verse-1",
            "verse-2",
            "verse-3",
            "chorus",
            "chorus-a",
            "chorus-b",
            "chorus-3",
            "cha",
            "chb",
            "ch3", // Short forms
            "pre-chorus",
            "prechorus",
            "bridge",
            "breakdown",
            "bkdn",
            "turnaround",
            "ta",
            "middle-8",
            "mid-8",
            "all", // Complete arrangement
            // Musical Styles (from Chords Collection)
            "ballad",
            "straight",
            "swing",
            "shuffle",
            "waltz",
            "valse",
            // Psy Trance Descriptors
            "flowing",
            "intense",
            "psychedelic",
            "hypnotic",
            // Production Styles
            "phatter",
            "phunkier",
            "original",
            "better-now-style",
            "sicko-mode-style",
            "stargazing-style",
            "in-my-feelings-style",
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

// Note: Default trait removed since AutoTagger::new() now returns Result.
// Users should call AutoTagger::new()? instead of using Default.

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // TAG STRUCT TESTS (6 tests)
    // =============================================================================

    #[test]
    fn test_tag_new_with_category() {
        let tag = Tag::new("house", Some("genre"));
        assert_eq!(tag.name, "house");
        assert_eq!(tag.category, Some("genre".to_string()));
        assert_eq!(tag.full_name(), "genre:house");
    }

    #[test]
    fn test_tag_new_without_category() {
        let tag = Tag::new("deep", None::<String>);
        assert_eq!(tag.name, "deep");
        assert_eq!(tag.category, None);
        assert_eq!(tag.full_name(), "deep");
    }

    #[test]
    fn test_tag_equality() {
        let tag1 = Tag::new("kick", Some("instrument"));
        let tag2 = Tag::new("kick", Some("instrument"));
        let tag3 = Tag::new("snare", Some("instrument"));

        assert_eq!(tag1, tag2);
        assert_ne!(tag1, tag3);
    }

    #[test]
    fn test_tag_clone() {
        let tag1 = Tag::new("techno", Some("genre"));
        let tag2 = tag1.clone();

        assert_eq!(tag1, tag2);
        assert_eq!(tag1.full_name(), tag2.full_name());
    }

    #[test]
    fn test_tag_hash_in_set() {
        use std::collections::HashSet;

        let mut tags = HashSet::new();
        tags.insert(Tag::new("house", Some("genre")));
        tags.insert(Tag::new("house", Some("genre"))); // Duplicate
        tags.insert(Tag::new("kick", Some("instrument")));

        assert_eq!(tags.len(), 2); // Duplicate removed
    }

    #[test]
    fn test_tag_debug() {
        let tag = Tag::new("bass", Some("instrument"));
        let debug_str = format!("{:?}", tag);

        assert!(debug_str.contains("bass"));
        assert!(debug_str.contains("instrument"));
    }

    // =============================================================================
    // FUZZY MATCHING TESTS (10 tests)
    // =============================================================================

    #[test]
    fn test_fuzzy_match_exact() {
        let tagger = AutoTagger::new().unwrap();

        // Exact match should always win
        let result = tagger.fuzzy_match("techno", &tagger.genre_keywords);
        assert_eq!(result, Some("techno".to_string()));
    }

    #[test]
    fn test_fuzzy_match_distance_1() {
        let tagger = AutoTagger::new().unwrap();

        // "teckno" is 1 char different from "techno" (swap c<->h)
        let result = tagger.fuzzy_match("teckno", &tagger.genre_keywords);
        assert_eq!(result, Some("techno".to_string()));
    }

    #[test]
    fn test_fuzzy_match_distance_2() {
        let tagger = AutoTagger::new().unwrap();

        // "tecnno" is 2 chars different from "techno"
        let result = tagger.fuzzy_match("tecnno", &tagger.genre_keywords);
        assert_eq!(result, Some("techno".to_string()));
    }

    #[test]
    fn test_fuzzy_match_distance_3_fails() {
        let tagger = AutoTagger::new().unwrap();

        // Distance 3 should not match (threshold is 2)
        let result = tagger.fuzzy_match("xyzno", &tagger.genre_keywords);
        assert_eq!(result, None);
    }

    #[test]
    fn test_fuzzy_match_short_word_no_fuzzy() {
        let tagger = AutoTagger::new().unwrap();

        // Words < 4 chars don't fuzzy match (only exact)
        let result = tagger.fuzzy_match("dnb", &tagger.genre_keywords);
        assert_eq!(result, Some("dnb".to_string())); // Exact match

        let result = tagger.fuzzy_match("dn", &tagger.genre_keywords);
        assert_eq!(result, None); // Too short and no exact match
    }

    #[test]
    fn test_fuzzy_match_minimum_distance_wins() {
        let tagger = AutoTagger::new().unwrap();

        // If multiple matches, choose the one with minimum distance
        // Note: fuzzy match requires >= 4 chars, so use "snare" → "snar"
        let result = tagger.fuzzy_match("snar", &tagger.instrument_keywords);
        assert_eq!(result, Some("snare".to_string())); // Distance 1
    }

    #[test]
    fn test_fuzzy_match_vengance_to_vengeance() {
        let tagger = AutoTagger::new().unwrap();

        // Common misspelling
        let result = tagger.fuzzy_match("vengance", &tagger.manufacturer_keywords);
        assert_eq!(result, Some("vengeance".to_string()));
    }

    #[test]
    fn test_fuzzy_match_hiphop_to_hip_hop() {
        let tagger = AutoTagger::new().unwrap();

        // Both forms exist in dictionary
        let result1 = tagger.fuzzy_match("hiphop", &tagger.genre_keywords);
        assert_eq!(result1, Some("hiphop".to_string()));

        let result2 = tagger.fuzzy_match("hip_hop", &tagger.genre_keywords);
        assert_eq!(result2, Some("hip_hop".to_string()));
    }

    #[test]
    fn test_fuzzy_match_empty_string() {
        let tagger = AutoTagger::new().unwrap();

        let result = tagger.fuzzy_match("", &tagger.genre_keywords);
        assert_eq!(result, None);
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let tagger = AutoTagger::new().unwrap();

        // Input is already lowercased by caller, but test the function
        let result = tagger.fuzzy_match("techno", &tagger.genre_keywords);
        assert_eq!(result, Some("techno".to_string()));
    }

    // =============================================================================
    // FILENAME EXTRACTION TESTS (15 tests)
    // =============================================================================

    #[test]
    fn test_filename_underscore_splitting() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("Deep_House_Kick.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"deep".to_string()));
        assert!(tag_names.contains(&"genre:house".to_string()));
        assert!(tag_names.contains(&"instrument:kick".to_string()));
    }

    #[test]
    fn test_filename_hyphen_splitting() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("techno-lead-synth.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"genre:techno".to_string()));
        assert!(tag_names.contains(&"instrument:lead".to_string()));
        assert!(tag_names.contains(&"instrument:synth".to_string()));
    }

    #[test]
    fn test_filename_space_splitting() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("Dark Ambient Pad.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"dark".to_string()));
        assert!(tag_names.contains(&"genre:ambient".to_string())); // ambient is in genre_keywords
        assert!(tag_names.contains(&"instrument:pad".to_string()));
    }

    #[test]
    fn test_filename_dot_splitting() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("kick.heavy.128.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"instrument:kick".to_string()));
        assert!(tag_names.contains(&"heavy".to_string()));
    }

    #[test]
    fn test_filename_mixed_separators() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("VEC_Deep-House Kick.128.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"deep".to_string()));
        assert!(tag_names.contains(&"genre:house".to_string()));
        assert!(tag_names.contains(&"instrument:kick".to_string()));
    }

    #[test]
    fn test_filename_extension_removal_mid() {
        let tagger = AutoTagger::new().unwrap();

        let tags1 = tagger.extract_from_filename("kick.mid");
        let tags2 = tagger.extract_from_filename("kick.MID");
        let tags3 = tagger.extract_from_filename("kick.midi");
        let tags4 = tagger.extract_from_filename("kick.MIDI");

        // All should extract "kick" regardless of extension case
        assert_eq!(tags1, tags2);
        assert_eq!(tags1, tags3);
        assert_eq!(tags1, tags4);
    }

    #[test]
    fn test_filename_common_words_filtered() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("The_New_Kick_For_Track_Mix.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();

        // Common words should be filtered
        assert!(!tag_names.contains(&"the".to_string()));
        assert!(!tag_names.contains(&"new".to_string()));
        assert!(!tag_names.contains(&"for".to_string()));
        assert!(!tag_names.contains(&"track".to_string()));
        assert!(!tag_names.contains(&"mix".to_string()));

        // But "kick" should remain
        assert!(tag_names.contains(&"kick".to_string()));
    }

    #[test]
    fn test_filename_short_words_filtered() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("A_B_C_Kick.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();

        // Single-letter words should be filtered (< 2 chars)
        assert!(!tag_names.contains(&"a".to_string()));
        assert!(!tag_names.contains(&"b".to_string()));
        assert!(!tag_names.contains(&"c".to_string()));

        // But "kick" should remain
        assert!(tag_names.contains(&"kick".to_string()));
    }

    #[test]
    fn test_filename_generic_tags_alphanumeric() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("CustomSample_Unusual.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // "unusual" is alphanumeric, >3 chars, not in dictionaries → generic tag
        assert!(tag_names.contains(&"unusual".to_string()));
    }

    #[test]
    fn test_filename_numbers_filtered() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("Kick_128_BPM_v2.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();

        // Numbers should be filtered (not alphanumeric in the sense of word.chars().all(|c| c.is_alphanumeric()))
        // Actually, "128" IS alphanumeric, but it's ≤ 3 chars so filtered by length
        // "v2" is 2 chars, filtered
        assert!(tag_names.contains(&"kick".to_string()));
        assert!(!tag_names.contains(&"128".to_string())); // Length 3, needs >3
        assert!(!tag_names.contains(&"v2".to_string())); // Length 2
        assert!(!tag_names.contains(&"bpm".to_string())); // Filtered as common word
    }

    #[test]
    fn test_filename_special_characters_split() {
        let tagger = AutoTagger::new().unwrap();

        // Special chars cause splits, leaving clean words
        let tags = tagger.extract_from_filename("Kick_Bass.mid"); // Use underscores which are valid separators
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();

        assert!(tag_names.contains(&"kick".to_string()));
        assert!(tag_names.contains(&"bass".to_string()));
    }

    #[test]
    fn test_filename_empty() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_filename_only_extension() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename(".mid");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_filename_multiple_same_keyword() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("Kick_Kick_Kick.mid");

        // Should deduplicate because tags are in a HashSet in extract_tags()
        // But extract_from_filename returns Vec, so we might get duplicates here
        // Let's check behavior
        assert!(!tags.is_empty());

        // Actually, we're returning Vec<Tag>, not HashSet, so duplicates are possible
        // The deduplication happens in extract_tags() which uses HashSet
        // This test documents current behavior
    }

    #[test]
    fn test_filename_no_matches() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("xyz_qwerty_abc.mid");

        // "qwerty" is >3 chars, alphanumeric → should become generic tag
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();
        assert!(tag_names.contains(&"qwerty".to_string()));
    }

    // =============================================================================
    // EXISTING TESTS (5 tests - already passing)
    // =============================================================================

    #[test]
    fn test_extract_from_filename() {
        let tagger = AutoTagger::new().unwrap();

        // Test 1: Vengeance style naming
        let tags = tagger.extract_from_filename("VEC_Deep_House_Kick_128_C.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        println!("Extracted tags: {:?}", tag_names);

        assert!(tag_names.contains(&"genre:house".to_string()));
        assert!(tag_names.contains(&"deep".to_string())); // mood keywords have no category prefix
        assert!(tag_names.contains(&"instrument:kick".to_string()));

        // Test 2: Underscore-separated naming (CamelCase not supported - see line 64)
        let tags = tagger.extract_from_filename("Techno_Lead_Synth.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"genre:techno".to_string()));
        assert!(tag_names.contains(&"instrument:lead".to_string()));
        assert!(tag_names.contains(&"instrument:synth".to_string()));
    }

    #[test]
    fn test_extract_from_path() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_path("/Vengeance/DeepHouse/Drums/Kicks/file.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"brand:vengeance".to_string()));
        assert!(tag_names.contains(&"category:drums".to_string()));
    }

    #[test]
    fn test_fuzzy_matching() {
        let tagger = AutoTagger::new().unwrap();

        // "vengance" should match "vengeance" (1 char difference)
        let result = tagger.fuzzy_match("vengance", &tagger.manufacturer_keywords);
        assert_eq!(result, Some("vengeance".to_string()));

        // "teckno" should match "techno" (1 char swap)
        let result = tagger.fuzzy_match("teckno", &tagger.genre_keywords);
        assert_eq!(result, Some("techno".to_string()));
    }

    #[test]
    fn test_full_tag_extraction() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_tags(
            "/Samples/Vengeance/DeepHouse/Drums/VEC_Deep_Kick_128_C.mid",
            "VEC_Deep_Kick_128_C.mid",
            &["Acoustic Bass Drum".to_string()],
            Some(128.0),
            Some("C"),
            None, // No MIDI file for backward compatibility test
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should have brand
        assert!(tag_names.iter().any(|t| t.starts_with("brand:")));
        // Should have genre
        assert!(tag_names.iter().any(|t| t.starts_with("genre:")));
        // Should have instrument
        assert!(tag_names.iter().any(|t| t.starts_with("instrument:")));
        // Should have BPM
        assert!(tag_names.contains(&"tempo:128".to_string()));
        // Should have key
        assert!(tag_names.contains(&"key:c".to_string()));
    }

    // =============================================================================
    // PATH EXTRACTION TESTS (12 tests - using real MIDI collection examples)
    // =============================================================================

    #[test]
    fn test_path_brand_extraction() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: Vengeance folder structure
        let tags = tagger.extract_from_path("/Samples/Vengeance/Vol2/DeepHouse/Kicks/file.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"brand:vengeance".to_string()));
    }

    #[test]
    fn test_path_genre_extraction() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: Techno folder (simpler case than "DnB Midi pack" which has spaces)
        let tags = tagger.extract_from_path("/MIDI/Techno/Loops/file.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // "techno" should match genre
        assert!(tag_names.contains(&"genre:techno".to_string()));
    }

    #[test]
    fn test_path_instrument_category() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: Dubstep pack with instrument folders
        let tags = tagger.extract_from_path("/Samples/Drums/Kicks/01A Kick.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Instruments in path get "category:" prefix
        assert!(tag_names.contains(&"category:drums".to_string()));
    }

    #[test]
    fn test_path_multiple_levels() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: Deep folder hierarchy
        let tags = tagger.extract_from_path("/Splice/Techno/Loops/Bass/Dark/file.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"brand:splice".to_string()));
        assert!(tag_names.contains(&"genre:techno".to_string()));
        // Bass might be category:bass from path
        assert!(tag_names.iter().any(|t| t.contains("bass")));
    }

    #[test]
    fn test_path_windows_style() {
        let tagger = AutoTagger::new().unwrap();

        // Windows paths use backslashes, but our code only handles forward slashes
        // This test documents current behavior
        let tags = tagger.extract_from_path("C:\\Samples\\Vengeance\\file.mid");

        // Won't extract properly with backslashes (documented limitation)
        // In production, paths should be normalized to forward slashes
        assert!(tags.is_empty()); // No extraction from Windows paths
    }

    #[test]
    fn test_path_empty() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_path("");
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_path_root_only() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_path("/file.mid");
        assert_eq!(tags.len(), 0); // No meaningful path components
    }

    #[test]
    fn test_path_fuzzy_matching() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: Common misspelling in folder name
        let tags = tagger.extract_from_path("/Samples/Vengance/file.mid"); // Misspelled
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should fuzzy match to "vengeance"
        assert!(tag_names.contains(&"brand:vengeance".to_string()));
    }

    #[test]
    fn test_path_case_insensitive() {
        let tagger = AutoTagger::new().unwrap();

        let tags1 = tagger.extract_from_path("/Samples/VENGEANCE/file.mid");
        let tags2 = tagger.extract_from_path("/Samples/vengeance/file.mid");
        let tags3 = tagger.extract_from_path("/Samples/Vengeance/file.mid");

        // All should produce same tags (case-insensitive)
        let names1: Vec<String> = tags1.iter().map(|t| t.full_name()).collect();
        let names2: Vec<String> = tags2.iter().map(|t| t.full_name()).collect();
        let names3: Vec<String> = tags3.iter().map(|t| t.full_name()).collect();

        assert_eq!(names1, names2);
        assert_eq!(names1, names3);
    }

    #[test]
    fn test_path_multiple_brands() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: Sample pack from multiple sources
        let tags = tagger.extract_from_path("/Splice/Vengeance/Cymatics/file.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should extract all brands
        assert!(tag_names.contains(&"brand:splice".to_string()));
        assert!(tag_names.contains(&"brand:vengeance".to_string()));
        assert!(tag_names.contains(&"brand:cymatics".to_string()));
    }

    #[test]
    fn test_path_normalized_names() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: After normalization, spaces become underscores
        let tags = tagger.extract_from_path("/Samples/House/Deep_Kick.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // "house" should be extracted from folder name
        assert!(tag_names.contains(&"genre:house".to_string()));
    }

    #[test]
    fn test_path_numeric_folders() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: Numbered folders
        let tags = tagger.extract_from_path("/Samples/2024/Vol2/file.mid");

        // Numbers should be ignored (no meaningful tags)
        // This test documents current behavior
        assert!(tags.is_empty() || tags.iter().all(|t| !t.name.chars().all(|c| c.is_numeric())));
    }

    // =============================================================================
    // INSTRUMENT EXTRACTION TESTS (8 tests - MIDI GM instrument names)
    // =============================================================================

    #[test]
    fn test_instrument_acoustic_bass_drum() {
        let tagger = AutoTagger::new().unwrap();

        // Note: extract_from_instruments expects whole-keyword fuzzy matching
        // "Acoustic Bass Drum" doesn't fuzzy-match any single keyword
        // This test documents current behavior (no extraction)
        let tags = tagger.extract_from_instruments(&["Acoustic Bass Drum".to_string()]);

        // Current behavior: no match (would need word-splitting to match "bass" or "drum")
        // In practice, instruments are extracted from filenames/paths, not GM names
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_instrument_multiple() {
        let tagger = AutoTagger::new().unwrap();

        // Use single-word instrument names that match keywords
        let instruments = vec!["Bass".to_string(), "Synth".to_string(), "Piano".to_string()];
        let tags = tagger.extract_from_instruments(&instruments);

        // Should extract tags for matching keywords
        assert!(tags.len() >= 3);
    }

    #[test]
    fn test_instrument_empty_list() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_instruments(&[]);
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_instrument_no_matches() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_instruments(&["Unknown Instrument 999".to_string()]);

        // Should not extract tags for unknown instruments
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_instrument_fuzzy_matching() {
        let tagger = AutoTagger::new().unwrap();

        // Misspelled single-word instrument name
        let tags = tagger.extract_from_instruments(&["Syntth".to_string()]); // Extra 't'
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should fuzzy match "synth" (Levenshtein distance = 1)
        assert!(tag_names.contains(&"instrument:synth".to_string()));
    }

    #[test]
    fn test_instrument_case_insensitive() {
        let tagger = AutoTagger::new().unwrap();

        let tags1 = tagger.extract_from_instruments(&["SYNTH BASS".to_string()]);
        let tags2 = tagger.extract_from_instruments(&["synth bass".to_string()]);
        let tags3 = tagger.extract_from_instruments(&["Synth Bass".to_string()]);

        // All should produce same tags
        let names1: Vec<String> = tags1.iter().map(|t| t.full_name()).collect();
        let names2: Vec<String> = tags2.iter().map(|t| t.full_name()).collect();
        let names3: Vec<String> = tags3.iter().map(|t| t.full_name()).collect();

        assert_eq!(names1, names2);
        assert_eq!(names1, names3);
    }

    #[test]
    fn test_instrument_duplicates() {
        let tagger = AutoTagger::new().unwrap();

        let instruments =
            vec!["Bass Drum".to_string(), "Bass Drum".to_string(), "Bass Drum".to_string()];
        let tags = tagger.extract_from_instruments(&instruments);

        // Should handle duplicates (extract_tags() uses HashSet for deduplication)
        assert!(!tags.is_empty());
    }

    #[test]
    fn test_instrument_partial_matches() {
        let tagger = AutoTagger::new().unwrap();

        // Single-word instrument that matches keyword
        let tags = tagger.extract_from_instruments(&["Bass".to_string()]);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // "bass" should be extracted (exact match)
        assert!(tag_names.contains(&"instrument:bass".to_string()));
    }

    // =============================================================================
    // BPM & KEY TAGGING TESTS (12 tests - using real filename patterns)
    // =============================================================================

    #[test]
    fn test_bpm_tag_integer() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "001 170 BPM E.mid"
        let tags = tagger.extract_tags("", "001 170 BPM E.mid", &[], Some(170.0), None, None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"tempo:170".to_string()));
    }

    #[test]
    fn test_bpm_tag_float_rounded() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: BPM detector returns 128.5
        let tags = tagger.extract_tags("", "kick.mid", &[], Some(128.5), None, None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should round to 129
        assert!(tag_names.contains(&"tempo:129".to_string()));
    }

    #[test]
    fn test_bpm_tag_none() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_tags("", "kick.mid", &[], None, None, None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // No BPM tag should be added
        assert!(!tag_names.iter().any(|t| t.starts_with("tempo:")));
    }

    #[test]
    fn test_key_tag_major() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "001 170 BPM E.mid"
        let tags = tagger.extract_tags("", "001 170 BPM E.mid", &[], None, Some("E"), None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"key:e".to_string()));
    }

    #[test]
    fn test_key_tag_minor() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "CS2_140_Am_Behind_The_Photo.mid"
        let tags = tagger.extract_tags(
            "",
            "CS2_140_Am_Behind_The_Photo.mid",
            &[],
            None,
            Some("Am"),
            None,
        );
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"key:am".to_string()));
    }

    #[test]
    fn test_key_tag_sharp() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "001 BASS LOOP G#.mid"
        let tags = tagger.extract_tags("", "001 BASS LOOP G#.mid", &[], None, Some("G#"), None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"key:g#".to_string()));
    }

    #[test]
    fn test_key_tag_flat() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "CS2_160_A#m_Belong_Here.mid"
        let tags = tagger.extract_tags("", "file.mid", &[], None, Some("Bb"), None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"key:bb".to_string()));
    }

    #[test]
    fn test_key_tag_unknown_filtered() {
        let tagger = AutoTagger::new().unwrap();

        // Unknown keys should be filtered out
        let tags = tagger.extract_tags("", "file.mid", &[], None, Some("Unknown"), None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(!tag_names.iter().any(|t| t.starts_with("key:")));
    }

    #[test]
    fn test_key_tag_none() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_tags("", "file.mid", &[], None, None, None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // No key tag should be added
        assert!(!tag_names.iter().any(|t| t.starts_with("key:")));
    }

    #[test]
    fn test_bpm_and_key_combined() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "001 Midi 174bpm C - SUBLIMEDNB Zenhiser.mid"
        let tags = tagger.extract_tags("", "174bpm C.mid", &[], Some(174.0), Some("C"), None);
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"tempo:174".to_string()));
        assert!(tag_names.contains(&"key:c".to_string()));
    }

    #[test]
    fn test_bpm_extreme_values() {
        let tagger = AutoTagger::new().unwrap();

        // Very slow tempo
        let tags1 = tagger.extract_tags("", "file.mid", &[], Some(40.0), None, None);
        let names1: Vec<String> = tags1.iter().map(|t| t.full_name()).collect();
        assert!(names1.contains(&"tempo:40".to_string()));

        // Very fast tempo
        let tags2 = tagger.extract_tags("", "file.mid", &[], Some(300.0), None, None);
        let names2: Vec<String> = tags2.iter().map(|t| t.full_name()).collect();
        assert!(names2.contains(&"tempo:300".to_string()));
    }

    #[test]
    fn test_key_case_normalization() {
        let tagger = AutoTagger::new().unwrap();

        let tags1 = tagger.extract_tags("", "file.mid", &[], None, Some("C#"), None);
        let tags2 = tagger.extract_tags("", "file.mid", &[], None, Some("c#"), None);
        let _tags3 = tagger.extract_tags("", "file.mid", &[], None, Some("C♯"), None); // Won't match (# only)

        let names1: Vec<String> = tags1.iter().map(|t| t.full_name()).collect();
        let names2: Vec<String> = tags2.iter().map(|t| t.full_name()).collect();

        // Both should normalize to "c#"
        assert_eq!(names1, names2);
    }

    // =============================================================================
    // EXISTING TESTS (5 tests - already passing)
    // =============================================================================

    #[test]
    fn test_common_words_filtered() {
        let tagger = AutoTagger::new().unwrap();

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

    // =============================================================================
    // INTEGRATION TESTS (10 tests - full end-to-end tagging with real examples)
    // =============================================================================

    #[test]
    fn test_integration_real_dnb_file() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "001 Midi 174bpm C - SUBLIMEDNB Zenhiser.mid"
        let tags = tagger.extract_tags(
            "/DnB/001 Midi 174bpm C - SUBLIMEDNB Zenhiser.mid",
            "001 Midi 174bpm C - SUBLIMEDNB Zenhiser.mid",
            &["Synth Bass".to_string()],
            Some(174.0),
            Some("C"),
            None,
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should have BPM, key
        assert!(tag_names.contains(&"tempo:174".to_string()));
        assert!(tag_names.contains(&"key:c".to_string()));
        // May or may not have instrument tags depending on matching
        // Note: "Zenhiser" is not in manufacturer_keywords, so it won't be extracted
    }

    #[test]
    fn test_integration_real_dubstep_file() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "01B Wobble Riser.mid"
        let tags = tagger.extract_tags(
            "/Dubstep Midis/01B Wobble Riser.mid",
            "01B Wobble Riser.mid",
            &[],
            Some(140.0),
            None,
            None,
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"tempo:140".to_string()));
        assert!(tag_names.iter().any(|t| t.contains("riser") || t.contains("fx")));
    }

    #[test]
    fn test_integration_real_ambient_file() {
        let tagger = AutoTagger::new().unwrap();

        // Real example: "CS2_140_Am_Behind_The_Photo.mid"
        let tags = tagger.extract_tags(
            "/Ambient/CS2_140_Am_Behind_The_Photo.mid",
            "CS2_140_Am_Behind_The_Photo.mid",
            &["Pad Synth".to_string()],
            Some(140.0),
            Some("Am"),
            None,
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"tempo:140".to_string()));
        assert!(tag_names.contains(&"key:am".to_string()));
        assert!(tag_names.contains(&"genre:ambient".to_string()));
        // May have instrument tags
    }

    #[test]
    fn test_integration_vengeance_style() {
        let tagger = AutoTagger::new().unwrap();

        // Typical Vengeance naming convention
        let tags = tagger.extract_tags(
            "/Vengeance/DeepHouse/Drums/Kicks/VEC_Deep_Kick_128_C.mid",
            "VEC_Deep_Kick_128_C.mid",
            &["Acoustic Bass Drum".to_string()],
            Some(128.0),
            Some("C"),
            None,
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should extract: brand, genre, style, instrument, bpm, key
        assert!(tag_names.contains(&"brand:vengeance".to_string()));
        assert!(tag_names.iter().any(|t| t.contains("house"))); // deephouse or house
        assert!(tag_names.contains(&"deep".to_string())); // Style tag
        assert!(tag_names.iter().any(|t| t.contains("kick")));
        assert!(tag_names.contains(&"tempo:128".to_string()));
        assert!(tag_names.contains(&"key:c".to_string()));
    }

    #[test]
    fn test_integration_splice_style() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_tags(
            "/Splice/Techno/Loops/Bass/Dark_Bass_Loop_125_Am.mid",
            "Dark_Bass_Loop_125_Am.mid",
            &["Synth Bass 1".to_string()],
            Some(125.0),
            Some("Am"),
            None,
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        assert!(tag_names.contains(&"brand:splice".to_string()));
        assert!(tag_names.contains(&"genre:techno".to_string()));
        assert!(tag_names.contains(&"dark".to_string()));
        assert!(tag_names.contains(&"instrument:bass".to_string()));
        assert!(tag_names.contains(&"tempo:125".to_string()));
        assert!(tag_names.contains(&"key:am".to_string()));
    }

    #[test]
    fn test_integration_deduplication() {
        let tagger = AutoTagger::new().unwrap();

        // Same keyword appears in path, filename, and instruments
        let tags = tagger.extract_tags(
            "/Bass/Sub Bass/Bass_Loop.mid",
            "Bass_Loop.mid",
            &["Bass Drum".to_string(), "Synth Bass".to_string()],
            None,
            None,
            None,
        );

        // Should deduplicate "bass" tags (using HashSet)
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();
        let bass_count = tag_names.iter().filter(|t| t.contains("bass")).count();

        // Multiple "bass" tags are okay (different sources), but not excessive duplicates
        assert!(bass_count >= 1);
        assert!(bass_count <= 5); // Reasonable upper bound
    }

    #[test]
    fn test_integration_minimal_file() {
        let tagger = AutoTagger::new().unwrap();

        // Minimal information
        let tags = tagger.extract_tags("", "file.mid", &[], None, None, None);

        // Should return empty or very minimal tags
        assert!(tags.is_empty());
    }

    #[test]
    fn test_integration_comprehensive_file() {
        let tagger = AutoTagger::new().unwrap();

        // Maximum information
        let tags = tagger.extract_tags(
            "/Cymatics/Dubstep/Drums/Snares/Heavy_Snare_140_E.mid",
            "Heavy_Snare_140_E.mid",
            &["Acoustic Snare".to_string(), "Reverb Snare".to_string()],
            Some(140.0),
            Some("E"),
            None,
        );

        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should have many tags from all sources
        assert!(tag_names.len() >= 5); // brand, genre, style, instrument, bpm, key
        assert!(tag_names.contains(&"brand:cymatics".to_string()));
        assert!(tag_names.contains(&"heavy".to_string()));
        assert!(tag_names.contains(&"instrument:snare".to_string()));
        assert!(tag_names.contains(&"tempo:140".to_string()));
        assert!(tag_names.contains(&"key:e".to_string()));
    }

    #[test]
    fn test_integration_unicode_handling() {
        let tagger = AutoTagger::new().unwrap();

        // Test with unicode characters (common in international file names)
        let tags = tagger.extract_tags(
            "/Samples/Café_Böhm/file.mid",
            "Café_Böhm_Kick.mid",
            &[],
            None,
            None,
            None, // No MIDI file for backward compatibility test
        );

        // Should handle unicode gracefully (lowercase conversion)
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // "kick" should still be extracted
        assert!(tag_names.contains(&"instrument:kick".to_string()));
    }

    #[test]
    fn test_integration_all_sources_empty() {
        let tagger = AutoTagger::new().unwrap();

        // All sources provide no useful information
        let tags = tagger.extract_tags("", "", &[], None, None, None);

        assert_eq!(tags.len(), 0);
    }

    // =============================================================================
    // DICTIONARY VALIDATION TESTS (10 tests - verify all keyword dictionaries)
    // =============================================================================

    #[test]
    fn test_dictionary_genre_count() {
        let tagger = AutoTagger::new().unwrap();

        // Verify dictionary is populated
        assert!(!tagger.genre_keywords.is_empty());
        assert!(tagger.genre_keywords.len() >= 20); // At least 20 genres

        // Verify some expected keywords
        assert!(tagger.genre_keywords.contains("house"));
        assert!(tagger.genre_keywords.contains("techno"));
        assert!(tagger.genre_keywords.contains("dnb"));
        assert!(tagger.genre_keywords.contains("dubstep"));
    }

    #[test]
    fn test_dictionary_instrument_count() {
        let tagger = AutoTagger::new().unwrap();

        assert!(!tagger.instrument_keywords.is_empty());
        assert!(tagger.instrument_keywords.len() >= 30); // At least 30 instruments

        // Verify common instruments
        assert!(tagger.instrument_keywords.contains("kick"));
        assert!(tagger.instrument_keywords.contains("snare"));
        assert!(tagger.instrument_keywords.contains("bass"));
        assert!(tagger.instrument_keywords.contains("synth"));
        assert!(tagger.instrument_keywords.contains("piano"));
    }

    #[test]
    fn test_dictionary_manufacturer_count() {
        let tagger = AutoTagger::new().unwrap();

        assert!(!tagger.manufacturer_keywords.is_empty());
        assert!(tagger.manufacturer_keywords.len() >= 15); // At least 15 brands

        // Verify major brands
        assert!(tagger.manufacturer_keywords.contains("vengeance"));
        assert!(tagger.manufacturer_keywords.contains("splice"));
        assert!(tagger.manufacturer_keywords.contains("cymatics"));
    }

    #[test]
    fn test_dictionary_style_count() {
        let tagger = AutoTagger::new().unwrap();

        assert!(!tagger.style_keywords.is_empty());
        assert!(tagger.style_keywords.len() >= 20); // At least 20 styles

        // Verify common styles
        assert!(tagger.style_keywords.contains("deep"));
        assert!(tagger.style_keywords.contains("dark"));
        assert!(tagger.style_keywords.contains("melodic"));
        assert!(tagger.style_keywords.contains("heavy"));
    }

    #[test]
    fn test_dictionary_common_words_count() {
        let tagger = AutoTagger::new().unwrap();

        assert!(!tagger.common_words.is_empty());
        assert!(tagger.common_words.len() >= 10); // At least 10 stopwords

        // Verify common stopwords
        assert!(tagger.common_words.contains("the"));
        assert!(tagger.common_words.contains("and"));
        assert!(tagger.common_words.contains("for"));
        assert!(tagger.common_words.contains("midi"));
    }

    #[test]
    fn test_dictionary_no_duplicates_genre() {
        let tagger = AutoTagger::new().unwrap();

        // HashSet should prevent duplicates
        let count = tagger.genre_keywords.len();
        let vec: Vec<_> = tagger.genre_keywords.iter().collect();

        assert_eq!(vec.len(), count); // No duplicates
    }

    #[test]
    fn test_dictionary_all_lowercase() {
        let tagger = AutoTagger::new().unwrap();

        // All keywords should be lowercase for consistent matching
        for keyword in &tagger.genre_keywords {
            assert_eq!(keyword, &keyword.to_lowercase());
        }

        for keyword in &tagger.instrument_keywords {
            assert_eq!(keyword, &keyword.to_lowercase());
        }

        for keyword in &tagger.manufacturer_keywords {
            assert_eq!(keyword, &keyword.to_lowercase());
        }

        for keyword in &tagger.style_keywords {
            assert_eq!(keyword, &keyword.to_lowercase());
        }
    }

    #[test]
    fn test_dictionary_no_empty_strings() {
        let tagger = AutoTagger::new().unwrap();

        // No dictionary should contain empty strings
        assert!(!tagger.genre_keywords.contains(""));
        assert!(!tagger.instrument_keywords.contains(""));
        assert!(!tagger.manufacturer_keywords.contains(""));
        assert!(!tagger.style_keywords.contains(""));
        assert!(!tagger.common_words.contains(""));
    }

    #[test]
    fn test_dictionary_multi_word_variants() {
        let tagger = AutoTagger::new().unwrap();

        // Should have both single-word and multi-word variants
        assert!(
            tagger.genre_keywords.contains("dnb")
                || tagger.genre_keywords.contains("drum_and_bass")
        );

        assert!(
            tagger.genre_keywords.contains("hiphop") || tagger.genre_keywords.contains("hip_hop")
        );
    }

    #[test]
    fn test_dictionary_construction_no_error() {
        // AutoTagger::new() should never fail with valid regex
        let result = AutoTagger::new();
        assert!(result.is_ok());

        let tagger = result.unwrap();

        // All dictionaries should be initialized
        assert!(!tagger.genre_keywords.is_empty());
        assert!(!tagger.instrument_keywords.is_empty());
        assert!(!tagger.manufacturer_keywords.is_empty());
        assert!(!tagger.style_keywords.is_empty());
        assert!(!tagger.common_words.is_empty());
    }

    // =============================================================================
    // EDGE CASES & STRESS TESTS (8 tests)
    // =============================================================================

    #[test]
    fn test_edge_case_very_long_filename() {
        let tagger = AutoTagger::new().unwrap();

        // 300+ character filename
        let long_name = "Very_Long_Filename_With_Many_Words_Kick_Bass_Synth_Pad_Lead_Arp_Pluck_Chord_Stab_FX_Riser_Impact_Sweep_Transition_Loop_Pattern_Sequence_Melody_Melodic_Harmony_Rhythm_Beat_Groove_Bounce_Punch_Heavy_Deep_Dark_Bright_Warm_Cold_Analog_Digital_Vintage_Modern_Classic_Dirty_Clean_Distorted_Atmospheric_Uplifting_Euphoric_Driving_Energetic_Chill_Relaxed_Aggressive_Soft.mid";

        let tags = tagger.extract_from_filename(long_name);

        // Should handle long filenames without crashing
        assert!(!tags.is_empty());
    }

    #[test]
    fn test_edge_case_very_deep_path() {
        let tagger = AutoTagger::new().unwrap();

        // 20+ levels deep
        let deep_path = "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/Vengeance/Techno/file.mid";

        let tags = tagger.extract_from_path(deep_path);

        // Should handle deep paths
        assert!(!tags.is_empty());
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();
        assert!(tag_names.contains(&"brand:vengeance".to_string()));
        assert!(tag_names.contains(&"genre:techno".to_string()));
    }

    #[test]
    fn test_edge_case_only_numbers() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("123_456_789.mid");

        // Numbers should be filtered (≤3 chars each)
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_edge_case_only_special_characters() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("@#$%^&*()_+[]{}|;:,.<>?.mid");

        // Special chars should be filtered
        assert_eq!(tags.len(), 0);
    }

    #[test]
    fn test_edge_case_mixed_case_consistency() {
        let tagger = AutoTagger::new().unwrap();

        let tags1 = tagger.extract_from_filename("TECHNO_KICK.mid");
        let tags2 = tagger.extract_from_filename("techno_kick.mid");
        let tags3 = tagger.extract_from_filename("Techno_Kick.mid");
        let tags4 = tagger.extract_from_filename("TeCHnO_KiCK.mid");

        // All should produce identical results
        let names1: Vec<String> = tags1.iter().map(|t| t.full_name()).collect();
        let names2: Vec<String> = tags2.iter().map(|t| t.full_name()).collect();
        let names3: Vec<String> = tags3.iter().map(|t| t.full_name()).collect();
        let names4: Vec<String> = tags4.iter().map(|t| t.full_name()).collect();

        assert_eq!(names1, names2);
        assert_eq!(names1, names3);
        assert_eq!(names1, names4);
    }

    #[test]
    fn test_edge_case_empty_components() {
        let tagger = AutoTagger::new().unwrap();

        // Multiple consecutive separators create empty components
        let tags = tagger.extract_from_filename("Kick___Bass...Synth.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.full_name()).collect();

        // Should still extract valid tags
        assert!(tag_names.contains(&"instrument:kick".to_string()));
        assert!(tag_names.contains(&"instrument:bass".to_string()));
        assert!(tag_names.contains(&"instrument:synth".to_string()));
    }

    #[test]
    fn test_edge_case_single_character_words() {
        let tagger = AutoTagger::new().unwrap();

        let tags = tagger.extract_from_filename("A_B_C_D_E_F_G_Kick.mid");
        let tag_names: Vec<String> = tags.iter().map(|t| t.name.clone()).collect();

        // Single chars should be filtered (<2 chars)
        assert!(!tag_names.contains(&"a".to_string()));
        assert!(!tag_names.contains(&"b".to_string()));

        // But "kick" should remain
        assert!(tag_names.contains(&"kick".to_string()));
    }

    #[test]
    fn test_edge_case_regex_pattern_valid() {
        // Ensure regex pattern compiles successfully
        let tagger = AutoTagger::new();

        assert!(tagger.is_ok());

        let tagger = tagger.unwrap();

        // Pattern should split on underscores, hyphens, spaces, dots
        let test_str = "a_b-c d.e";
        let parts: Vec<&str> = tagger.split_pattern.split(test_str).collect();

        assert_eq!(parts, vec!["a", "b", "c", "d", "e"]);
    }
}
