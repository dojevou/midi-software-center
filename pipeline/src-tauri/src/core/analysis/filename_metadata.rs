/// Filename metadata extraction module
///
/// Extracts musical metadata embedded in filenames:
/// - BPM values (120, 128, 140, etc.)
/// - Key signatures (Cm, Am, F#, etc.)
/// - Genre tags (house, techno, dnb, etc.)
/// - Structure tags (fill, loop, verse, etc.)
/// - Track numbers
///
/// Based on analysis of 1.49M MIDI files from production collection.
/// Complements content-based analysis with filename-based metadata fallback.
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Filename metadata extracted from a MIDI file name
#[derive(Debug, Clone, PartialEq)]
pub struct FilenameMetadata {
    /// BPM value extracted from filename (30-300 range)
    pub bpm: Option<f32>,
    /// Musical key signature (e.g., "Cm", "Am", "F#")
    pub key: Option<String>,
    /// Genre tags (house, techno, dnb, etc.)
    pub genres: Vec<String>,
    /// Structure tags (fill, loop, verse, etc.)
    pub structure_tags: Vec<String>,
    /// Leading track number (1-999)
    pub track_number: Option<u32>,
}

impl FilenameMetadata {
    /// Extracts all metadata from a filename in one pass
    ///
    /// # Examples
    /// ```
    /// use pipeline::core::analysis::filename_metadata::FilenameMetadata;
    ///
    /// let meta = FilenameMetadata::extract_from_filename("120_bpm_Cm_house_fill.mid");
    /// assert_eq!(meta.bpm, Some(120.0));
    /// assert_eq!(meta.key, Some("Cm".to_string()));
    /// assert!(meta.genres.contains(&"house".to_string()));
    /// assert!(meta.structure_tags.contains(&"fill".to_string()));
    /// ```
    pub fn extract_from_filename(filename: &str) -> Self {
        FilenameMetadata {
            bpm: extract_bpm_from_filename(filename),
            key: extract_key_from_filename(filename),
            genres: extract_genres_from_filename(filename),
            structure_tags: extract_structure_tags(filename),
            track_number: extract_leading_number(filename),
        }
    }
}

// ============================================================================
// BPM Extraction
// ============================================================================

static BPM_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_bpm_regex() -> &'static Regex {
    BPM_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)([0-9]{2,3})[\s_]*(bpm|beats|tempo)|(?:^|_|\s|-|/)([0-9]{2,3})(?:_|\s|-|/|\.)",
        )
        .expect("BPM regex should be valid")
    })
}

/// Extracts BPM value from filename
///
/// Recognizes patterns:
/// - Explicit: "120_BPM_house_loop.mid", "Drums_140bpm.mid"
/// - Implicit: "house_120.mid", "bass_140.mid"
///
/// Valid range: 30-300 BPM
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::extract_bpm_from_filename;
///
/// assert_eq!(extract_bpm_from_filename("house_120_loop.mid"), Some(120.0));
/// assert_eq!(extract_bpm_from_filename("140bpm_trap.mid"), Some(140.0));
/// assert_eq!(extract_bpm_from_filename("drums_128_beats.mid"), Some(128.0));
/// assert_eq!(extract_bpm_from_filename("no_bpm_here.mid"), None);
/// ```
pub fn extract_bpm_from_filename(filename: &str) -> Option<f32> {
    // Use find_iter to get all matches and pick the best one
    // Prioritize explicit BPM notation (with "bpm", "beats", "tempo")
    // over implicit standalone numbers
    let regex = get_bpm_regex();

    // First, try to find explicit BPM notation
    for caps in regex.captures_iter(filename) {
        if let Some(m) = caps.get(1) {
            if let Ok(bpm) = m.as_str().parse::<f32>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    // If no explicit BPM found, try implicit numbers
    for caps in regex.captures_iter(filename) {
        if let Some(m) = caps.get(3) {
            if let Ok(bpm) = m.as_str().parse::<f32>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    None
}

// ============================================================================
// Key Signature Extraction
// ============================================================================

static KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static KEY_NORMALIZATION_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_key_regex() -> &'static Regex {
    KEY_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)(?:^|_|\s|-|/)([A-G](?:#|b)?m?)(?:_|\s|-|/|\.)|([A-G]\s?(maj|min|major|minor))",
        )
        .expect("Key regex should be valid")
    })
}

fn get_key_map() -> &'static HashMap<&'static str, &'static str> {
    KEY_NORMALIZATION_MAP.get_or_init(|| {
        HashMap::from([
            // Minor variants
            ("amin", "Am"),
            ("am", "Am"),
            ("bmin", "Bm"),
            ("bm", "Bm"),
            ("cmin", "Cm"),
            ("cm", "Cm"),
            ("dmin", "Dm"),
            ("dm", "Dm"),
            ("emin", "Em"),
            ("em", "Em"),
            ("fmin", "Fm"),
            ("fm", "Fm"),
            ("gmin", "Gm"),
            ("gm", "Gm"),
            // Major variants
            ("amaj", "A"),
            ("cmaj", "C"),
            ("dmaj", "D"),
            ("emaj", "E"),
            ("fmaj", "F"),
            ("gmaj", "G"),
            ("bmaj", "B"),
            // Flats and sharps
            ("bb", "Bb"),
            ("a#", "A#"),
            ("c#", "C#"),
            ("d#", "D#"),
            ("f#", "F#"),
            ("g#", "G#"),
            ("ab", "Ab"),
            ("db", "Db"),
            ("eb", "Eb"),
            ("gb", "Gb"),
            // Ambiguous single letters (default to major)
            ("a", "A"),
            ("b", "B"),
            ("c", "C"),
            ("d", "D"),
            ("e", "E"),
            ("f", "F"),
            ("g", "G"),
        ])
    })
}

/// Normalizes raw key signature to canonical form
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::normalize_key_signature;
///
/// assert_eq!(normalize_key_signature("amin"), Some("Am".to_string()));
/// assert_eq!(normalize_key_signature("Cmaj"), Some("C".to_string()));
/// assert_eq!(normalize_key_signature("f#"), Some("F#".to_string()));
/// assert_eq!(normalize_key_signature("a"), Some("A".to_string()));
/// ```
pub fn normalize_key_signature(raw_key: &str) -> Option<String> {
    get_key_map().get(raw_key.to_lowercase().as_str()).map(|&s| s.to_string())
}

/// Extracts key signature from filename and normalizes it
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::extract_key_from_filename;
///
/// assert_eq!(extract_key_from_filename("Cm_bass.mid"), Some("Cm".to_string()));
/// assert_eq!(extract_key_from_filename("16_Dm_Bass.mid"), Some("Dm".to_string()));
/// assert_eq!(extract_key_from_filename("melody_in_A_major.mid"), Some("A".to_string()));
/// ```
pub fn extract_key_from_filename(filename: &str) -> Option<String> {
    get_key_regex()
        .captures(filename)
        .and_then(|caps| caps.get(1).or_else(|| caps.get(3)))
        .and_then(|m| normalize_key_signature(m.as_str()))
}

// ============================================================================
// Genre Extraction
// ============================================================================

static GENRE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_genre_regex() -> &'static Regex {
    GENRE_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)(house|techno|trance|hip.?hop|trap|dubstep|dnb|drum.?n.?bass|jazz|funk|soul|rock|pop|edm|ambient|downtempo|break|jungle|garage|electro|acid|minimal|deep|progressive)",
        )
        .expect("Genre regex should be valid")
    })
}

/// Normalizes genre variants to canonical form
fn normalize_genre(raw: &str) -> String {
    match raw {
        "hip hop" | "hiphop" | "hip-hop" => "hip-hop".to_string(),
        "dnb" | "drum n bass" | "drum and bass" | "drum-n-bass" | "drum_n_bass" => {
            "dnb".to_string()
        },
        genre => genre.to_string(),
    }
}

/// Extracts genre tags from filename
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::extract_genres_from_filename;
///
/// let genres = extract_genres_from_filename("deep_house_120.mid");
/// assert!(genres.contains(&"deep".to_string()));
/// assert!(genres.contains(&"house".to_string()));
///
/// let genres = extract_genres_from_filename("drum_n_bass_170.mid");
/// assert!(genres.contains(&"dnb".to_string()));
/// ```
pub fn extract_genres_from_filename(filename: &str) -> Vec<String> {
    get_genre_regex()
        .find_iter(filename)
        .map(|m| m.as_str().to_lowercase())
        .map(|g| normalize_genre(&g))
        .collect()
}

// ============================================================================
// Structure Tag Extraction
// ============================================================================

static STRUCTURE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_structure_regex() -> &'static Regex {
    STRUCTURE_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)(verse|chorus|bridge|intro|outro|drop|build|breakdown|fill|loop|one.?shot|sample|melody|hook|riff|lick|main|full|short|long)",
        )
        .expect("Structure regex should be valid")
    })
}

/// Normalizes structure tag variants
fn normalize_structure_tag(raw: &str) -> String {
    match raw {
        "one shot" | "one-shot" | "oneshot" => "oneshot".to_string(),
        tag => tag.to_string(),
    }
}

/// Extracts structure tags from filename
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::extract_structure_tags;
///
/// let tags = extract_structure_tags("drum_fill_120bpm.mid");
/// assert!(tags.contains(&"fill".to_string()));
///
/// let tags = extract_structure_tags("verse_melody_loop.mid");
/// assert!(tags.contains(&"verse".to_string()));
/// assert!(tags.contains(&"melody".to_string()));
/// assert!(tags.contains(&"loop".to_string()));
/// ```
pub fn extract_structure_tags(filename: &str) -> Vec<String> {
    get_structure_regex()
        .find_iter(filename)
        .map(|m| m.as_str().to_lowercase())
        .map(|s| normalize_structure_tag(&s))
        .collect()
}

// ============================================================================
// Track Number Extraction
// ============================================================================

static LEADING_NUMBER_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_leading_number_regex() -> &'static Regex {
    LEADING_NUMBER_REGEX
        .get_or_init(|| Regex::new(r"^([0-9]+)").expect("Leading number regex should be valid"))
}

/// Extracts leading number from filename
///
/// Common uses:
/// - Track ordering (01-99)
/// - Kit numbers (001-999)
/// - Version numbers (v1, v2, v3)
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::extract_leading_number;
///
/// assert_eq!(extract_leading_number("01_kick.mid"), Some(1));
/// assert_eq!(extract_leading_number("125_melody.mid"), Some(125));
/// assert_eq!(extract_leading_number("kick_01.mid"), None);
/// ```
pub fn extract_leading_number(filename: &str) -> Option<u32> {
    get_leading_number_regex()
        .captures(filename)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())
}

/// Type of leading number (track number vs possible BPM)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType {
    /// Track number (1-99)
    TrackNumber,
    /// Could be BPM value (30-300)
    PossibleBPM,
    /// Unknown purpose
    Unknown,
}

/// Classifies the purpose of a leading number
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::{classify_leading_number, NumberType};
///
/// assert_eq!(classify_leading_number(1), NumberType::TrackNumber);
/// assert_eq!(classify_leading_number(42), NumberType::TrackNumber);
/// assert_eq!(classify_leading_number(120), NumberType::PossibleBPM);
/// assert_eq!(classify_leading_number(500), NumberType::Unknown);
/// ```
pub fn classify_leading_number(num: u32) -> NumberType {
    match num {
        30..=300 => NumberType::PossibleBPM,
        1..=29 => NumberType::TrackNumber,
        _ => NumberType::Unknown,
    }
}

// ============================================================================
// Cross-Validation
// ============================================================================

/// Result of cross-validating analyzed metadata with filename metadata
#[derive(Debug, Clone, PartialEq)]
pub enum KeyValidationResult {
    /// Both sources agree (high confidence)
    Validated(String),
    /// Sources disagree (requires manual review)
    Conflict { analyzed: String, filename: String },
    /// Only analyzed data available
    AnalyzedOnly(String),
    /// Only filename data available
    FilenameOnly(String),
    /// No data available
    Unknown,
}

/// Cross-validates key signature from analysis and filename
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::{validate_key_signature, KeyValidationResult};
///
/// // Agreement - validated
/// match validate_key_signature(Some("Cm"), Some("Cm")) {
///     KeyValidationResult::Validated(key) => assert_eq!(key, "Cm"),
///     _ => panic!("Expected validated"),
/// }
///
/// // Conflict
/// match validate_key_signature(Some("Cm"), Some("Dm")) {
///     KeyValidationResult::Conflict { analyzed, filename } => {
///         assert_eq!(analyzed, "Cm");
///         assert_eq!(filename, "Dm");
///     }
///     _ => panic!("Expected conflict"),
/// }
/// ```
pub fn validate_key_signature(
    analyzed_key: Option<&str>,
    filename_key: Option<&str>,
) -> KeyValidationResult {
    match (analyzed_key, filename_key) {
        (Some(a), Some(f)) if a == f => KeyValidationResult::Validated(a.to_string()),
        (Some(a), Some(f)) => {
            KeyValidationResult::Conflict { analyzed: a.to_string(), filename: f.to_string() }
        },
        (Some(a), None) => KeyValidationResult::AnalyzedOnly(a.to_string()),
        (None, Some(f)) => KeyValidationResult::FilenameOnly(f.to_string()),
        (None, None) => KeyValidationResult::Unknown,
    }
}

/// Validates BPM from analysis against filename metadata
///
/// Tolerance: ±5 BPM (accounts for detection variance)
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::validate_bpm;
///
/// assert!(validate_bpm(Some(120.0), Some(120.0)));  // Exact match
/// assert!(validate_bpm(Some(120.0), Some(123.0)));  // Within tolerance
/// assert!(!validate_bpm(Some(120.0), Some(140.0))); // Out of tolerance
/// ```
pub fn validate_bpm(analyzed_bpm: Option<f32>, filename_bpm: Option<f32>) -> bool {
    match (analyzed_bpm, filename_bpm) {
        (Some(a), Some(f)) => (a - f).abs() <= 5.0,
        _ => false,
    }
}

/// Genre-specific BPM range validation
///
/// # Examples
/// ```
/// use pipeline::core::analysis::filename_metadata::validate_bpm_for_genre;
///
/// assert!(validate_bpm_for_genre(120.0, "house"));   // House: 120-128 BPM
/// assert!(!validate_bpm_for_genre(80.0, "house"));   // Too slow for house
/// assert!(validate_bpm_for_genre(170.0, "dnb"));     // DNB: 160-180 BPM
/// ```
pub fn validate_bpm_for_genre(bpm: f32, genre: &str) -> bool {
    let ranges: HashMap<&str, (f32, f32)> = [
        ("house", (120.0, 128.0)),
        ("techno", (125.0, 135.0)),
        ("trance", (128.0, 140.0)),
        ("dubstep", (138.0, 142.0)),
        ("dnb", (160.0, 180.0)),
        ("drum-n-bass", (160.0, 180.0)),
        ("trap", (135.0, 145.0)),
        ("hip-hop", (70.0, 100.0)),
        ("funk", (90.0, 120.0)),
        ("rock", (100.0, 140.0)),
        ("pop", (100.0, 130.0)),
    ]
    .iter()
    .cloned()
    .collect();

    ranges.get(genre).map(|&(min, max)| bpm >= min && bpm <= max).unwrap_or(true)
    // Unknown genres pass validation
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpm_extraction() {
        assert_eq!(extract_bpm_from_filename("house_120_loop.mid"), Some(120.0));
        assert_eq!(extract_bpm_from_filename("140bpm_trap.mid"), Some(140.0));
        assert_eq!(
            extract_bpm_from_filename("drums_128_beats.mid"),
            Some(128.0)
        );
        assert_eq!(extract_bpm_from_filename("no_bpm_here.mid"), None);
        assert_eq!(extract_bpm_from_filename("track_5.mid"), None);
    }

    #[test]
    fn test_key_extraction() {
        assert_eq!(
            extract_key_from_filename("Cm_bass.mid"),
            Some("Cm".to_string())
        );
        assert_eq!(
            extract_key_from_filename("16_Dm_Bass.mid"),
            Some("Dm".to_string())
        );
        assert_eq!(
            extract_key_from_filename("melody_in_A_major.mid"),
            Some("A".to_string())
        );
        assert_eq!(
            extract_key_from_filename("F#_lead.mid"),
            Some("F#".to_string())
        );
    }

    #[test]
    fn test_genre_extraction() {
        let genres = extract_genres_from_filename("deep_house_120.mid");
        assert!(genres.contains(&"deep".to_string()));
        assert!(genres.contains(&"house".to_string()));

        let genres = extract_genres_from_filename("drum_n_bass_170.mid");
        assert!(genres.contains(&"dnb".to_string()));

        let genres = extract_genres_from_filename("hip-hop_90.mid");
        assert!(genres.contains(&"hip-hop".to_string()));
    }

    #[test]
    fn test_structure_tags() {
        let tags = extract_structure_tags("drum_fill_120bpm.mid");
        assert!(tags.contains(&"fill".to_string()));

        let tags = extract_structure_tags("verse_melody_loop.mid");
        assert!(tags.contains(&"verse".to_string()));
        assert!(tags.contains(&"melody".to_string()));
        assert!(tags.contains(&"loop".to_string()));
    }

    #[test]
    fn test_leading_number() {
        assert_eq!(extract_leading_number("01_kick.mid"), Some(1));
        assert_eq!(extract_leading_number("125_melody.mid"), Some(125));
        assert_eq!(extract_leading_number("kick_01.mid"), None);
    }

    #[test]
    fn test_number_classification() {
        assert_eq!(classify_leading_number(1), NumberType::TrackNumber);
        assert_eq!(classify_leading_number(42), NumberType::TrackNumber);
        assert_eq!(classify_leading_number(120), NumberType::PossibleBPM);
        assert_eq!(classify_leading_number(500), NumberType::Unknown);
    }

    #[test]
    fn test_full_extraction() {
        let meta = FilenameMetadata::extract_from_filename("01_120_bpm_Cm_house_fill.mid");
        assert_eq!(meta.track_number, Some(1));
        assert_eq!(meta.bpm, Some(120.0));
        assert_eq!(meta.key, Some("Cm".to_string()));
        assert!(meta.genres.contains(&"house".to_string()));
        assert!(meta.structure_tags.contains(&"fill".to_string()));
    }

    #[test]
    fn test_bpm_validation() {
        assert!(validate_bpm(Some(120.0), Some(120.0)));
        assert!(validate_bpm(Some(120.0), Some(123.0)));
        assert!(!validate_bpm(Some(120.0), Some(140.0)));
    }

    #[test]
    fn test_genre_bpm_validation() {
        assert!(validate_bpm_for_genre(120.0, "house"));
        assert!(!validate_bpm_for_genre(80.0, "house"));
        assert!(validate_bpm_for_genre(170.0, "dnb"));
        assert!(!validate_bpm_for_genre(120.0, "dnb"));
    }

    #[test]
    fn test_key_validation() {
        match validate_key_signature(Some("Cm"), Some("Cm")) {
            KeyValidationResult::Validated(key) => assert_eq!(key, "Cm"),
            _ => panic!("Expected validated"),
        }

        match validate_key_signature(Some("Cm"), Some("Dm")) {
            KeyValidationResult::Conflict { analyzed, filename } => {
                assert_eq!(analyzed, "Cm");
                assert_eq!(filename, "Dm");
            },
            _ => panic!("Expected conflict"),
        }

        match validate_key_signature(Some("Cm"), None) {
            KeyValidationResult::AnalyzedOnly(key) => assert_eq!(key, "Cm"),
            _ => panic!("Expected analyzed only"),
        }

        match validate_key_signature(None, Some("Cm")) {
            KeyValidationResult::FilenameOnly(key) => assert_eq!(key, "Cm"),
            _ => panic!("Expected filename only"),
        }

        match validate_key_signature(None, None) {
            KeyValidationResult::Unknown => (),
            _ => panic!("Expected unknown"),
        }
    }
}
