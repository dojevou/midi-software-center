//! VIP3 Category Retagger - High-performance batch tagging for MIDI files
//!
//! Uses Aho-Corasick for O(n) multi-pattern matching and parallel processing
//! to efficiently tag millions of files with VIP3 categories:
//! - Timbres (sound character: Aggressive, Warm, Bright, etc.)
//! - Styles (genres: Hip-Hop, Techno, Jazz, etc.)
//! - Articulations (patterns: Loop, Chord, Arpeggio, etc.)
//!
//! Performance targets:
//! - 50,000+ files/sec for text extraction (CPU-bound)
//! - 10,000+ tags/sec for database writes (I/O-bound)

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

/// Spelling normalization maps (variant -> canonical)
pub mod normalizations {
    use std::collections::HashMap;
    use std::sync::LazyLock;

    /// Instrument/sound spelling variants
    pub static INSTRUMENT: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
        HashMap::from([
            // Drums
            ("kicks", "kick"),
            ("kik", "kick"),
            ("kck", "kick"),
            ("bd", "kick"),
            ("snares", "snare"),
            ("sn", "snare"),
            ("sd", "snare"),
            ("hihats", "hihat"),
            ("hh", "hihat"),
            ("hi-hats", "hihat"),
            ("hats", "hihat"),
            ("claps", "clap"),
            ("clp", "clap"),
            ("handclap", "clap"),
            ("cymbals", "cymbal"),
            ("cym", "cymbal"),
            ("crashes", "crash"),
            ("crsh", "crash"),
            ("rides", "ride"),
            ("rd", "ride"),
            ("toms", "tom"),
            ("tomtom", "tom"),
            ("tom-tom", "tom"),
            ("percs", "percussion"),
            ("prcs", "percussion"),
            ("drm", "drums"),
            ("drms", "drums"),
            ("drumkit", "drums"),
            ("tambourines", "tambourine"),
            ("tamb", "tambourine"),
            ("shakers", "shaker"),
            ("shkr", "shaker"),
            ("congas", "conga"),
            ("bongos", "bongo"),
            // Bass
            ("basses", "bass"),
            ("bs", "bass"),
            ("bassline", "bass"),
            ("subbass", "sub"),
            ("sub-bass", "sub"),
            ("808s", "808"),
            ("tr808", "808"),
            // Keys
            ("pianos", "piano"),
            ("pno", "piano"),
            ("pian", "piano"),
            ("organs", "organ"),
            ("org", "organ"),
            ("b3", "organ"),
            ("hammond", "organ"),
            ("ep", "electric-piano"),
            ("epiano", "electric-piano"),
            ("rhodes", "electric-piano"),
            ("clavs", "clav"),
            ("clavinet", "clav"),
            ("keyboards", "keyboard"),
            ("keyb", "keyboard"),
            ("kbd", "keyboard"),
            // Guitar
            ("guitars", "guitar"),
            ("gtr", "guitar"),
            ("guit", "guitar"),
            ("acousticguitar", "acoustic-guitar"),
            ("acgtr", "acoustic-guitar"),
            ("electricguitar", "electric-guitar"),
            ("elgtr", "electric-guitar"),
            // Brass
            ("trumpets", "trumpet"),
            ("tpt", "trumpet"),
            ("trombones", "trombone"),
            ("trb", "trombone"),
            ("horns", "horn"),
            ("frenchhorn", "horn"),
            ("saxophones", "saxophone"),
            ("saxes", "saxophone"),
            ("sax", "saxophone"),
            // Synth
            ("synths", "synth"),
            ("syn", "synth"),
            ("synthesizer", "synth"),
            ("leads", "lead"),
            ("ld", "lead"),
            ("pads", "pad"),
            ("pd", "pad"),
            ("arps", "arp"),
            ("arpeggio", "arp"),
            ("arpeggios", "arp"),
            ("plucks", "pluck"),
            ("plk", "pluck"),
            ("stabs", "stab"),
            ("stb", "stab"),
            // Strings
            ("string", "strings"),
            ("str", "strings"),
            ("violins", "violin"),
            ("vln", "violin"),
            ("cellos", "cello"),
            ("vlc", "cello"),
            // Woodwind
            ("flutes", "flute"),
            ("flt", "flute"),
            ("clarinets", "clarinet"),
            ("clar", "clarinet"),
            // Vocal
            ("vocals", "vocal"),
            ("voc", "vocal"),
            ("vx", "vocal"),
            ("voices", "voice"),
            ("choirs", "choir"),
            // Orchestral
            ("orch", "orchestral"),
            ("orchestras", "orchestral"),
            ("cinematic", "orchestral"),
            ("filmscore", "orchestral"),
        ])
    });

    /// Genre/style spelling variants
    pub static GENRE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
        HashMap::from([
            // Rock/Metal
            ("rocks", "rock"),
            ("rck", "rock"),
            ("metals", "metal"),
            ("mtl", "metal"),
            ("heavymetal", "metal"),
            ("punks", "punk"),
            ("pnk", "punk"),
            // Jazz
            ("jazzy", "jazz"),
            ("jz", "jazz"),
            ("swinging", "swing"),
            ("swg", "swing"),
            ("fusions", "fusion"),
            ("jazzfusion", "fusion"),
            // Electronic
            ("houses", "house"),
            ("deephouse", "house"),
            ("deep-house", "house"),
            ("technos", "techno"),
            ("tek", "techno"),
            ("trances", "trance"),
            ("psytrance", "trance"),
            ("dubsteps", "dubstep"),
            ("dub-step", "dubstep"),
            ("dnb", "drum-and-bass"),
            ("drumnbass", "drum-and-bass"),
            ("traps", "trap"),
            ("melodictrap", "trap"),
            ("edms", "edm"),
            ("electronica", "electronic"),
            // Urban
            ("hiphop", "hip-hop"),
            ("hip_hop", "hip-hop"),
            ("hiphops", "hip-hop"),
            ("raps", "rap"),
            ("rapper", "rap"),
            ("rnb", "r&b"),
            ("r-and-b", "r&b"),
            ("randb", "r&b"),
            ("souls", "soul"),
            ("neo-soul", "soul"),
            // Other
            ("pops", "pop"),
            ("disco", "disco"),
            ("funks", "funk"),
            ("funky", "funk"),
            ("lofi", "lo-fi"),
            ("lo_fi", "lo-fi"),
            ("chillhop", "lo-fi"),
            ("ambients", "ambient"),
            ("atmospheres", "ambient"),
            ("cinematic", "orchestral"),
            ("filmscore", "orchestral"),
            ("latins", "latin"),
            ("salsa", "latin"),
            ("bossa", "latin"),
            ("worlds", "world"),
            ("ethnic", "world"),
            ("tribal", "world"),
            ("countrys", "country"),
            ("bluegrass", "country"),
        ])
    });
}

/// Keyword dictionaries for each category
pub mod keywords {
    /// Timbre keywords (map to VIP3 timbres table)
    pub const TIMBRES: &[(&str, &[&str])] = &[
        (
            "Aggressive",
            &["aggressive", "angry", "hard", "heavy", "brutal", "harsh", "fierce"],
        ),
        (
            "Airy",
            &["airy", "breathy", "light", "ethereal", "floating", "spacey"],
        ),
        (
            "Bright",
            &["bright", "crisp", "sharp", "clear", "shiny", "sparkling"],
        ),
        (
            "Clean",
            &["clean", "pure", "pristine", "crystal", "transparent"],
        ),
        (
            "Dark",
            &["dark", "sinister", "evil", "menacing", "ominous", "scary", "creepy"],
        ),
        (
            "Dirty",
            &["dirty", "gritty", "raw", "nasty", "grimy", "filthy"],
        ),
        (
            "Distorted",
            &["distorted", "distortion", "fuzz", "fuzzy", "overdrive", "overdriven"],
        ),
        ("Fat", &["fat", "thick", "huge", "massive", "big", "beefy"]),
        (
            "Gritty",
            &["gritty", "grainy", "textured", "rough", "coarse"],
        ),
        ("Hard", &["hard", "punchy", "tight", "snappy", "aggressive"]),
        (
            "Metallic",
            &["metallic", "metal", "iron", "steel", "tinny", "bell"],
        ),
        ("Muted", &["muted", "dampened", "subdued", "quiet", "soft"]),
        (
            "Punchy",
            &["punchy", "punch", "impact", "transient", "attack"],
        ),
        (
            "Soft",
            &["soft", "gentle", "smooth", "mellow", "delicate", "tender"],
        ),
        ("Thin", &["thin", "narrow", "weak", "tiny", "small"]),
        ("Warm", &["warm", "lush", "rich", "full", "round", "cozy"]),
        (
            "Wide",
            &["wide", "stereo", "spread", "spacious", "expansive"],
        ),
        (
            "Analog",
            &["analog", "analogue", "vintage", "retro", "classic", "old-school"],
        ),
        (
            "Digital",
            &["digital", "modern", "clean", "precise", "hi-fi"],
        ),
        (
            "Organic",
            &["organic", "natural", "acoustic", "real", "live"],
        ),
        (
            "Synthetic",
            &["synthetic", "synth", "electronic", "artificial", "processed"],
        ),
    ];

    /// Style keywords (map to VIP3 styles table)
    pub const STYLES: &[(&str, &[&str])] = &[
        (
            "Ambient",
            &["ambient", "atmosphere", "atmospheric", "drone", "soundscape"],
        ),
        (
            "Cinematic",
            &["cinematic", "film", "movie", "epic", "trailer", "score", "soundtrack"],
        ),
        ("Dance", &["dance", "club", "disco", "dancefloor"]),
        ("EDM", &["edm", "electro", "electronic", "electronica"]),
        ("Funk", &["funk", "funky", "groovy", "groove"]),
        (
            "Hip-Hop",
            &["hip-hop", "hiphop", "hip_hop", "rap", "trap", "boom-bap", "boombap"],
        ),
        (
            "House",
            &[
                "house",
                "deep-house",
                "deephouse",
                "tech-house",
                "techhouse",
                "progressive-house",
            ],
        ),
        (
            "Jazz",
            &["jazz", "jazzy", "swing", "bebop", "fusion", "smooth-jazz"],
        ),
        (
            "Latin",
            &["latin", "salsa", "bossa", "samba", "tango", "cumbia", "reggaeton"],
        ),
        (
            "Lo-Fi",
            &["lofi", "lo-fi", "lo_fi", "chillhop", "jazzhop", "study"],
        ),
        (
            "Metal",
            &["metal", "heavy-metal", "thrash", "death-metal", "black-metal", "doom"],
        ),
        ("Pop", &["pop", "poppy", "mainstream", "commercial"]),
        (
            "R&B",
            &["rnb", "r&b", "r-and-b", "randb", "soul", "neo-soul"],
        ),
        ("Reggae", &["reggae", "dub", "ska", "dancehall", "roots"]),
        (
            "Rock",
            &["rock", "alternative", "indie", "punk", "grunge", "classic-rock"],
        ),
        ("Soul", &["soul", "motown", "gospel", "neo-soul"]),
        (
            "Techno",
            &["techno", "minimal", "detroit", "berlin", "industrial-techno"],
        ),
        ("Trap", &["trap", "melodic-trap", "drill", "uk-drill"]),
        (
            "Orchestral",
            &["orchestral", "orchestra", "classical", "symphonic", "strings", "brass"],
        ),
        (
            "Electronic",
            &["electronic", "synth", "synthesizer", "analog"],
        ),
        ("Acoustic", &["acoustic", "unplugged", "folk"]),
        (
            "World",
            &["world", "ethnic", "tribal", "african", "asian", "middle-east", "arabic"],
        ),
        (
            "Experimental",
            &["experimental", "avant-garde", "noise", "glitch", "idm"],
        ),
        (
            "Chillout",
            &["chillout", "chill", "relaxed", "downtempo", "lounge"],
        ),
    ];

    /// Articulation keywords (map to VIP3 articulations table)
    pub const ARTICULATIONS: &[(&str, &[&str])] = &[
        ("Arpeggio", &["arpeggio", "arpeggiated", "arp", "arps"]),
        ("Chord", &["chord", "chords", "progression", "chordprog"]),
        ("Fill", &["fill", "fills", "drum-fill", "drumfill"]),
        ("Loop", &["loop", "loops", "looped", "looping"]),
        (
            "Melody",
            &["melody", "melodic", "melodious", "tune", "topline"],
        ),
        (
            "One-Shot",
            &["one-shot", "oneshot", "hit", "single", "shot"],
        ),
        ("Phrase", &["phrase", "phrases", "lick", "riff"]),
        ("Riff", &["riff", "riffs", "hook", "hooks", "lick"]),
        ("Stab", &["stab", "stabs", "stabby"]),
        ("Sustain", &["sustain", "sustained", "long", "held"]),
        (
            "Sequence",
            &["sequence", "seq", "seqs", "pattern", "sequenced"],
        ),
        ("Pattern", &["pattern", "patterns", "motif"]),
        ("Groove", &["groove", "grooves", "groovy", "grooving"]),
        ("Break", &["break", "breakdown", "breakbeat", "breaks"]),
        ("Intro", &["intro", "introduction", "opener", "opening"]),
        ("Outro", &["outro", "ending", "closer", "closing"]),
        ("Verse", &["verse", "verses"]),
        ("Chorus", &["chorus", "refrain", "hook"]),
        ("Bridge", &["bridge", "middle-8", "middle8"]),
        ("Drop", &["drop", "drops", "buildup", "build"]),
    ];
}

/// Extracted VIP3 categories for a file
#[derive(Debug, Clone, Default)]
pub struct Vip3Categories {
    pub timbres: Vec<String>,
    pub styles: Vec<String>,
    pub articulations: Vec<String>,
}

/// High-performance VIP3 category extractor using Aho-Corasick
pub struct Vip3Extractor {
    timbre_matcher: AhoCorasick,
    timbre_map: HashMap<usize, String>,

    style_matcher: AhoCorasick,
    style_map: HashMap<usize, String>,

    articulation_matcher: AhoCorasick,
    articulation_map: HashMap<usize, String>,

    /// Spelling normalizations for future fuzzy matching enhancement
    #[allow(dead_code)]
    normalizations: HashMap<String, String>,
}

impl Vip3Extractor {
    /// Create a new VIP3 extractor with pre-built Aho-Corasick automata
    pub fn new() -> Self {
        // Build timbre matcher
        let (timbre_patterns, timbre_map) = Self::build_pattern_map(keywords::TIMBRES);
        let timbre_matcher = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostFirst)
            .ascii_case_insensitive(true)
            .build(&timbre_patterns)
            .expect("Failed to build timbre matcher");

        // Build style matcher
        let (style_patterns, style_map) = Self::build_pattern_map(keywords::STYLES);
        let style_matcher = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostFirst)
            .ascii_case_insensitive(true)
            .build(&style_patterns)
            .expect("Failed to build style matcher");

        // Build articulation matcher
        let (articulation_patterns, articulation_map) =
            Self::build_pattern_map(keywords::ARTICULATIONS);
        let articulation_matcher = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostFirst)
            .ascii_case_insensitive(true)
            .build(&articulation_patterns)
            .expect("Failed to build articulation matcher");

        // Build combined normalizations map
        let mut normalizations = HashMap::new();
        for (k, v) in normalizations::INSTRUMENT.iter() {
            normalizations.insert(k.to_string(), v.to_string());
        }
        for (k, v) in normalizations::GENRE.iter() {
            normalizations.insert(k.to_string(), v.to_string());
        }

        Self {
            timbre_matcher,
            timbre_map,
            style_matcher,
            style_map,
            articulation_matcher,
            articulation_map,
            normalizations,
        }
    }

    /// Build pattern list and index-to-category map for Aho-Corasick
    fn build_pattern_map(categories: &[(&str, &[&str])]) -> (Vec<String>, HashMap<usize, String>) {
        let mut patterns = Vec::new();
        let mut map = HashMap::new();

        for (category, keywords) in categories {
            for keyword in *keywords {
                map.insert(patterns.len(), category.to_string());
                patterns.push(keyword.to_string());
            }
        }

        (patterns, map)
    }

    /// Normalize text for matching (lowercase, replace separators)
    fn normalize_text(text: &str) -> String {
        text.to_lowercase().replace(['_', '-', '.', '/', '\\'], " ")
    }

    /// Normalize a word using the normalization map (for future fuzzy matching)
    #[allow(dead_code)]
    fn normalize_word(&self, word: &str) -> String {
        let lower = word.to_lowercase();
        self.normalizations.get(&lower).cloned().unwrap_or(lower)
    }

    /// Extract VIP3 categories from filename and filepath
    /// This is the hot path - optimized for speed
    pub fn extract(&self, filename: &str, filepath: &str) -> Vip3Categories {
        // Combine and normalize text
        let text = Self::normalize_text(&format!("{} {}", filepath, filename));

        let mut timbres = HashSet::new();
        let mut styles = HashSet::new();
        let mut articulations = HashSet::new();

        // Single-pass Aho-Corasick matching for each category
        for mat in self.timbre_matcher.find_iter(&text) {
            if let Some(category) = self.timbre_map.get(&mat.pattern().as_usize()) {
                timbres.insert(category.clone());
            }
        }

        for mat in self.style_matcher.find_iter(&text) {
            if let Some(category) = self.style_map.get(&mat.pattern().as_usize()) {
                styles.insert(category.clone());
            }
        }

        for mat in self.articulation_matcher.find_iter(&text) {
            if let Some(category) = self.articulation_map.get(&mat.pattern().as_usize()) {
                articulations.insert(category.clone());
            }
        }

        Vip3Categories {
            timbres: timbres.into_iter().collect(),
            styles: styles.into_iter().collect(),
            articulations: articulations.into_iter().collect(),
        }
    }

    /// Extract categories from a batch of files in parallel
    pub fn extract_batch(&self, files: &[(i64, String, String)]) -> Vec<(i64, Vip3Categories)> {
        files
            .par_iter()
            .map(|(id, filename, filepath)| {
                let categories = self.extract(filename, filepath);
                (*id, categories)
            })
            .collect()
    }
}

impl Default for Vip3Extractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics from a retag operation
#[derive(Debug, Clone, Default)]
pub struct RetagStats {
    pub files_processed: usize,
    pub timbres_added: usize,
    pub styles_added: usize,
    pub articulations_added: usize,
    pub elapsed_ms: u64,
}

impl RetagStats {
    pub fn total_tags(&self) -> usize {
        self.timbres_added + self.styles_added + self.articulations_added
    }

    pub fn files_per_second(&self) -> f64 {
        if self.elapsed_ms > 0 {
            (self.files_processed as f64) / (self.elapsed_ms as f64 / 1000.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let extractor = Vip3Extractor::new();
        assert!(!extractor.timbre_map.is_empty());
        assert!(!extractor.style_map.is_empty());
        assert!(!extractor.articulation_map.is_empty());
    }

    #[test]
    fn test_timbre_extraction() {
        let extractor = Vip3Extractor::new();

        let result = extractor.extract("Dark_Aggressive_Synth_Loop.mid", "/samples/edm/");
        assert!(result.timbres.contains(&"Dark".to_string()));
        assert!(result.timbres.contains(&"Aggressive".to_string()));
    }

    #[test]
    fn test_style_extraction() {
        let extractor = Vip3Extractor::new();

        let result = extractor.extract("HipHop_Beat_01.mid", "/samples/hip-hop/trap/");
        assert!(
            result.styles.contains(&"Hip-Hop".to_string())
                || result.styles.contains(&"Trap".to_string())
        );
    }

    #[test]
    fn test_articulation_extraction() {
        let extractor = Vip3Extractor::new();

        let result = extractor.extract("Piano_Arpeggio_Loop_Am.mid", "/samples/keys/");
        assert!(result.articulations.contains(&"Arpeggio".to_string()));
        assert!(result.articulations.contains(&"Loop".to_string()));
    }

    #[test]
    fn test_batch_extraction() {
        let extractor = Vip3Extractor::new();

        let files = vec![
            (1, "Dark_Pad.mid".to_string(), "/synth/".to_string()),
            (2, "Funky_Bass_Loop.mid".to_string(), "/bass/".to_string()),
            (3, "Jazz_Piano_Chord.mid".to_string(), "/piano/".to_string()),
        ];

        let results = extractor.extract_batch(&files);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_normalize_text() {
        assert_eq!(
            Vip3Extractor::normalize_text("Dark_Synth-Loop.mid"),
            "dark synth loop mid"
        );
        assert_eq!(
            Vip3Extractor::normalize_text("/path/to/File"),
            " path to file"
        );
    }
}
