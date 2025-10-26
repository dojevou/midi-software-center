///! Tag Extractor - Trusty Module
//!
//! Extracts tags from file paths and folder names

use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExtractedTags {
    pub manufacturer: Option<String>,
    pub collection: Option<String>,
    pub genres: Vec<String>,
    pub category: Option<String>,
    pub bpm_hint: Option<i16>,
    pub descriptors: Vec<String>,
}

pub fn extract_tags_from_path(filepath: &str) -> ExtractedTags {
    let path = Path::new(filepath);
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    let mut tags = ExtractedTags {
        manufacturer: None,
        collection: None,
        genres: Vec::new(),
        category: None,
        bpm_hint: None,
        descriptors: Vec::new(),
    };

    // Extract manufacturer
    tags.manufacturer = detect_manufacturer(&components);

    // Extract genres
    tags.genres = extract_genres(&components);

    // Extract category
    tags.category = detect_category(&components);

    // Extract BPM hint
    tags.bpm_hint = extract_bpm_hint(filepath);

    // Extract descriptors
    tags.descriptors = extract_descriptors(&components);

    tags
}

fn detect_manufacturer(components: &[&str]) -> Option<String> {
    let manufacturers = [
        "DMS", "Loopmasters", "Vengeance", "Sample Magic", "Singomakers",
        "Hy2rogen", "Production Master", "Function Loops", "Audentity",
        "Chill Samples", "Class A Samples", "Diginoiz", "Prime Loops",
        "Producer Loops", "Sonic Academy", "Black Octopus", "MIDI Focus",
        "Zenhiser"
    ];

    for component in components {
        for mfr in &manufacturers {
            if component.contains(mfr) {
                return Some(mfr.to_string());
            }
        }
    }

    None
}

fn extract_genres(components: &[&str]) -> Vec<String> {
    let genres = [
        "Trance", "House", "Techno", "Drum and Bass", "DnB", "Dubstep",
        "Hardcore", "Progressive", "Electro", "Tech House", "Deep House",
        "Psytrance", "Minimal", "Ambient", "Chill", "Lo-Fi", "Trap",
        "Future Bass", "Liquid"
    ];

    let mut found = Vec::new();
    let path_str = components.join("/").to_lowercase();

    for genre in &genres {
        if path_str.contains(&genre.to_lowercase()) {
            found.push(genre.to_string());
        }
    }

    found
}

fn detect_category(components: &[&str]) -> Option<String> {
    let categories = [
        ("Bass", vec!["bass", "sub", "reese"]),
        ("Melody", vec!["melody", "lead", "melodic"]),
        ("Pad", vec!["pad", "atmosphere", "ambient"]),
        ("Chord", vec!["chord", "progression"]),
        ("Arp", vec!["arp", "arpegg"]),
        ("Drum", vec!["drum", "kick", "snare", "hat", "perc"]),
        ("FX", vec!["fx", "effect", "riser", "sweep"]),
    ];

    let path_str = components.join("/").to_lowercase();

    for (category, keywords) in &categories {
        for keyword in keywords {
            if path_str.contains(keyword) {
                return Some(category.to_string());
            }
        }
    }

    None
}

fn extract_bpm_hint(filepath: &str) -> Option<i16> {
    // Look for patterns like "140 BPM", "140BPM", "140bpm"
    let re = regex::Regex::new(r"(\d{2,3})\s*[Bb][Pp][Mm]").ok()?;

    if let Some(cap) = re.captures(filepath) {
        return cap[1].parse().ok();
    }

    // Also check for folder names like "140 BPM"
    let re2 = regex::Regex::new(r"/(\d{2,3})\s*[Bb][Pp][Mm]/").ok()?;
    if let Some(cap) = re2.captures(filepath) {
        return cap[1].parse().ok();
    }

    None
}

fn extract_descriptors(components: &[&str]) -> Vec<String> {
    let descriptors = [
        "Hard", "Soft", "Deep", "Bright", "Dark", "Warm", "Cold",
        "Fat", "Thin", "Punchy", "Smooth", "Rough", "Clean", "Dirty",
        "Melodic", "Atmospheric", "Epic", "Minimal", "Complex",
        "Liquid", "Uplifting", "Driving", "Bouncy", "Rolling",
        "Variation", "Straight", "Shuffle", "Triplet", "Syncopation"
    ];

    let mut found = Vec::new();
    let path_str = components.join("/").to_lowercase();

    for desc in &descriptors {
        if path_str.contains(&desc.to_lowercase()) && !found.contains(&desc.to_string()) {
            found.push(desc.to_string());
        }
    }

    found
}
