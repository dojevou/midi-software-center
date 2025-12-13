/// Filename Generator
///
/// Generates intelligent filenames from MIDI file metadata.
use crate::core::analysis::{BpmDetectionResult, KeyDetectionResult};
use crate::core::naming::{sanitizer, templates};

/// Configuration for filename generation
#[derive(Debug, Clone)]
pub struct NamingConfig {
    pub template: templates::NamingTemplate,
    pub include_description: bool,
    pub max_description_length: usize,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            template: templates::NamingTemplate::Standard,
            include_description: true,
            max_description_length: 50,
        }
    }
}

/// Input metadata for filename generation
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub category: String,
    pub bpm: f64,
    pub key: String,
    pub description: Option<String>,
    pub file_id: String,
}

/// Generates a new filename from metadata
///
/// # Arguments
/// * `metadata` - File metadata
/// * `config` - Naming configuration
///
/// # Returns
/// * Generated filename with .mid extension
///
/// # Examples
/// ```
/// use pipeline::core::naming::generator::*;
///
/// let metadata = FileMetadata {
///     category: "BASS".to_string(),
///     bpm: 140.0,
///     key: "Cm".to_string(),
///     description: Some("Deep Rolling".to_string()),
///     file_id: "001".to_string(),
/// };
///
/// let filename = generate_filename(&metadata, &NamingConfig::default());
/// // Result: "BASS_Cm_140BPM_Deep_Rolling_001.mid"
/// ```
pub fn generate_filename(metadata: &FileMetadata, config: &NamingConfig) -> String {
    // Sanitize category
    let category = sanitizer::sanitize_filename(&metadata.category.to_uppercase());

    // Sanitize key
    let key = sanitizer::sanitize_filename(&metadata.key);

    // Process description
    let description = if config.include_description {
        process_description(&metadata.description, config.max_description_length)
    } else {
        String::new()
    };

    // Apply template
    let filename_base = templates::apply_template(
        &config.template,
        &category,
        &key,
        metadata.bpm,
        &description,
        &metadata.file_id,
    );

    // Final sanitization
    let sanitized = sanitizer::sanitize_filename(&filename_base);

    // Ensure .mid extension
    sanitizer::ensure_mid_extension(&sanitized)
}

/// Processes description text
fn process_description(description: &Option<String>, max_length: usize) -> String {
    match description {
        None => String::new(),
        Some(desc) => {
            // First sanitize to convert spaces to underscores
            let sanitized = sanitizer::sanitize_filename(desc);

            // Then clean filler words
            let cleaned = sanitizer::clean_description(&sanitized);

            // Truncate if needed
            if cleaned.len() > max_length {
                cleaned[..max_length].to_string()
            } else {
                cleaned
            }
        },
    }
}

/// Generates filename from analysis results (convenience function)
///
/// # Arguments
/// * `category` - File category (BASS, KICK, etc.)
/// * `bpm_result` - BPM detection result
/// * `key_result` - Key detection result
/// * `original_filename` - Original filename to extract description from
/// * `file_id` - Unique file identifier
/// * `config` - Naming configuration
///
/// # Returns
/// * Generated filename with .mid extension
pub fn generate_from_analysis(
    category: &str,
    bpm_result: &BpmDetectionResult,
    key_result: &KeyDetectionResult,
    original_filename: &str,
    file_id: &str,
    config: &NamingConfig,
) -> String {
    // Extract description from original filename if useful
    let description = extract_useful_description(original_filename);

    let metadata = FileMetadata {
        category: category.to_string(),
        bpm: bpm_result.bpm,
        key: key_result.key.clone(),
        description,
        file_id: file_id.to_string(),
    };

    generate_filename(&metadata, config)
}

/// Generates production filename with pack name and original filename
///
/// # Arguments
/// * `category` - File category
/// * `bpm` - Beats per minute
/// * `key` - Musical key
/// * `file_id` - Zero-padded file ID (e.g., "000001")
/// * `timesig` - Time signature (e.g., "4-4", "6-8")
/// * `pack_name` - Name of the pack/folder
/// * `original_name` - Original filename (cleaned)
///
/// # Returns
/// * Formatted filename: {CATEGORY}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{PACK}_{ORIGINAL}.mid
///
/// # Examples
/// ```
/// use pipeline::core::naming::generator::generate_production_filename;
///
/// let filename = generate_production_filename(
///     "KICK",
///     120.0,
///     "C",
///     "000001",
///     "4-4",
///     "DrumPack2024",
///     "Heavy_Boom"
/// );
/// assert_eq!(filename, "KICK_4-4_120BPM_C_000001_DrumPack2024_Heavy_Boom.mid");
/// ```
pub fn generate_production_filename(
    category: &str,
    bpm: f64,
    key: &str,
    file_id: &str,
    timesig: &str,
    pack_name: &str,
    original_name: &str,
) -> String {
    let sanitized_category = sanitizer::sanitize_filename(&category.to_uppercase());
    let sanitized_key = sanitizer::sanitize_filename(key);
    let sanitized_pack = sanitizer::sanitize_filename(pack_name);
    let sanitized_original = sanitizer::sanitize_filename(original_name);

    let filename_base = templates::apply_template_extended(
        &templates::NamingTemplate::Production,
        &sanitized_category,
        &sanitized_key,
        bpm,
        "", // description not used in Production template
        file_id,
        Some(timesig),
        Some(&sanitized_pack),
        Some(&sanitized_original),
        None, // no layer info
    );

    sanitizer::ensure_mid_extension(&filename_base)
}

/// Generates production filename for split/layer files
///
/// # Arguments
/// * `category` - File category
/// * `bpm` - Beats per minute
/// * `key` - Musical key
/// * `file_id` - Zero-padded file ID
/// * `timesig` - Time signature
/// * `pack_name` - Name of the pack/folder
/// * `layer_name` - Name of the layer (e.g., "OpenHat", "ClosedHat")
/// * `layer_number` - Layer number (1-based)
///
/// # Returns
/// * Formatted filename: {CATEGORY}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{PACK}_{LAYER}_L{NUM}.mid
///
/// # Examples
/// ```
/// use pipeline::core::naming::generator::generate_production_layer_filename;
///
/// let filename = generate_production_layer_filename(
///     "HIHAT",
///     140.0,
///     "Am",
///     "000123",
///     "6-8",
///     "VintageDrums",
///     "OpenHat",
///     1
/// );
/// assert_eq!(filename, "HIHAT_6-8_140BPM_Am_000123_VintageDrums_OpenHat_L01.mid");
/// ```
pub fn generate_production_layer_filename(
    category: &str,
    bpm: f64,
    key: &str,
    file_id: &str,
    timesig: &str,
    pack_name: &str,
    layer_name: &str,
    layer_number: usize,
) -> String {
    let sanitized_category = sanitizer::sanitize_filename(&category.to_uppercase());
    let sanitized_key = sanitizer::sanitize_filename(key);
    let sanitized_pack = sanitizer::sanitize_filename(pack_name);
    let sanitized_layer = sanitizer::sanitize_filename(layer_name);

    let filename_base = templates::apply_template_extended(
        &templates::NamingTemplate::Production,
        &sanitized_category,
        &sanitized_key,
        bpm,
        "",
        file_id,
        Some(timesig),
        Some(&sanitized_pack),
        None, // no original for split files
        Some((&sanitized_layer, layer_number)),
    );

    sanitizer::ensure_mid_extension(&filename_base)
}

/// Extracts useful parts from original filename
fn extract_useful_description(original_filename: &str) -> Option<String> {
    // Remove extension
    let without_ext = original_filename.trim_end_matches(".mid").trim_end_matches(".MID");

    // Remove common prefixes
    let prefixes = ["MIDI_", "Track_", "File_", "Song_"];
    let mut cleaned = without_ext.to_string();

    for prefix in &prefixes {
        if cleaned.starts_with(prefix) {
            cleaned = cleaned[prefix.len()..].to_string();
        }
    }

    // If cleaned version is meaningful, use it
    if !cleaned.is_empty() && cleaned.len() > 3 {
        Some(cleaned)
    } else {
        None
    }
}

/// Handles naming conflicts by appending counter
///
/// # Arguments
/// * `base_filename` - The desired filename
/// * `existing_files` - List of existing filenames to check against
///
/// # Returns
/// * Unique filename that doesn't conflict with existing files
pub fn resolve_naming_conflict(base_filename: &str, existing_files: &[String]) -> String {
    let without_ext = base_filename.trim_end_matches(".mid");

    if !existing_files.contains(&base_filename.to_string()) {
        return base_filename.to_string();
    }

    // Try incrementing counter
    for i in 1..1000 {
        let candidate = format!("{}_v{}.mid", without_ext, i);
        if !existing_files.contains(&candidate) {
            return candidate;
        }
    }

    // Fallback with timestamp
    // Note: SystemTime before Unix epoch is impossible on modern systems,
    // but we handle it gracefully per architecture requirements
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();

    format!("{}_{}.mid", without_ext, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename_standard() {
        let metadata = FileMetadata {
            category: "BASS".to_string(),
            bpm: 140.5,
            key: "Cm".to_string(),
            description: Some("Deep Rolling".to_string()),
            file_id: "001".to_string(),
        };

        let filename = generate_filename(&metadata, &NamingConfig::default());

        assert!(filename.starts_with("BASS_Cm_140BPM"));
        assert!(filename.ends_with(".mid"));
    }

    #[test]
    fn test_generate_filename_no_description() {
        let metadata = FileMetadata {
            category: "KICK".to_string(),
            bpm: 128.0,
            key: "C".to_string(),
            description: None,
            file_id: "042".to_string(),
        };

        let config = NamingConfig { include_description: false, ..Default::default() };

        let filename = generate_filename(&metadata, &config);

        // Sanitizer removes double underscores, so result is clean
        assert_eq!(filename, "KICK_C_128BPM_042.mid");
    }

    #[test]
    fn test_extract_useful_description() {
        assert_eq!(
            extract_useful_description("MIDI_Cool_Bass.mid"),
            Some("Cool_Bass".to_string())
        );

        assert_eq!(
            extract_useful_description("Track_1.mid"),
            None // Too short
        );
    }

    #[test]
    fn test_extract_no_prefix() {
        assert_eq!(
            extract_useful_description("Amazing_Lead.mid"),
            Some("Amazing_Lead".to_string())
        );
    }

    #[test]
    fn test_resolve_naming_conflict() {
        let base = "BASS_Cm_140BPM_Deep_001.mid";
        let existing = vec![base.to_string()];

        let resolved = resolve_naming_conflict(base, &existing);

        assert_ne!(resolved, base);
        assert!(resolved.ends_with(".mid"));
        assert!(resolved.contains("_v1"));
    }

    #[test]
    fn test_resolve_no_conflict() {
        let base = "BASS_Cm_140BPM_Deep_001.mid";
        let existing = vec![];

        let resolved = resolve_naming_conflict(base, &existing);

        assert_eq!(resolved, base);
    }

    #[test]
    fn test_category_uppercase() {
        let metadata = FileMetadata {
            category: "bass".to_string(), // lowercase
            bpm: 140.0,
            key: "Cm".to_string(),
            description: None,
            file_id: "001".to_string(),
        };

        let filename = generate_filename(&metadata, &NamingConfig::default());

        assert!(filename.starts_with("BASS")); // Should be uppercase
    }

    #[test]
    fn test_process_description_truncation() {
        let long_desc = Some("A".repeat(100));

        let result = process_description(&long_desc, 20);

        assert!(result.len() <= 20);
    }

    #[test]
    fn test_process_description_none() {
        let result = process_description(&None, 50);
        assert_eq!(result, "");
    }

    #[test]
    fn test_compact_template() {
        let metadata = FileMetadata {
            category: "KICK".to_string(),
            bpm: 128.0,
            key: "C".to_string(),
            description: Some("Heavy".to_string()),
            file_id: "042".to_string(),
        };

        let config = NamingConfig {
            template: templates::NamingTemplate::Compact,
            include_description: true,
            max_description_length: 50,
        };

        let filename = generate_filename(&metadata, &config);

        // Compact template doesn't include description in the template
        assert_eq!(filename, "KICK_C_128BPM_042.mid");
    }

    #[test]
    fn test_invalid_characters_in_metadata() {
        let metadata = FileMetadata {
            category: "BA<SS>".to_string(),
            bpm: 140.0,
            key: "C:m".to_string(),
            description: Some("Deep/Rolling*".to_string()),
            file_id: "001".to_string(),
        };

        let filename = generate_filename(&metadata, &NamingConfig::default());

        // Should sanitize all invalid characters
        assert!(!filename.contains('<'));
        assert!(!filename.contains('>'));
        assert!(!filename.contains(':'));
        assert!(!filename.contains('/'));
        assert!(!filename.contains('*'));
    }

    #[test]
    fn test_description_with_filler_words() {
        let metadata = FileMetadata {
            category: "BASS".to_string(),
            bpm: 140.0,
            key: "Cm".to_string(),
            description: Some("the new bass and track file".to_string()),
            file_id: "001".to_string(),
        };

        let filename = generate_filename(&metadata, &NamingConfig::default());

        println!("Generated filename: {}", filename);

        // Filler words should be removed (the, new, and, track, file)
        // Result should have "bass" but not the filler words
        assert!(filename.contains("bass"));
    }

    #[test]
    fn test_multiple_conflicts() {
        let base = "BASS_Cm_140BPM_Deep_001.mid";
        let existing = vec![
            base.to_string(),
            "BASS_Cm_140BPM_Deep_001_v1.mid".to_string(),
            "BASS_Cm_140BPM_Deep_001_v2.mid".to_string(),
        ];

        let resolved = resolve_naming_conflict(base, &existing);

        assert_eq!(resolved, "BASS_Cm_140BPM_Deep_001_v3.mid");
    }

    #[test]
    fn test_extract_empty_filename() {
        assert_eq!(extract_useful_description(""), None);
        assert_eq!(extract_useful_description(".mid"), None);
    }
}
