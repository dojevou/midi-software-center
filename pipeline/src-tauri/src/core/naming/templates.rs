/// Naming Templates
///
/// Provides different template formats for filename generation.
/// Naming template format
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NamingTemplate {
    /// {CATEGORY}_{KEY}_{BPM}BPM_{DESCRIPTION}_{ID}
    #[default]
    Standard,

    /// {CATEGORY}_{KEY}_{BPM}BPM_{ID}
    Compact,

    /// {BPM}BPM_{KEY}_{CATEGORY}_{DESCRIPTION}
    BpmFirst,

    /// Production: {CATEGORY}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{PACK}_{ORIGINAL}
    /// Includes pack name and original filename for full context
    Production,

    /// Custom template with placeholders
    Custom(String),
}

/// Applies template to metadata
///
/// # Arguments
/// * `template` - The naming template to use
/// * `category` - File category (e.g., BASS, KICK, CHORD)
/// * `key` - Musical key (e.g., C, Am, F#)
/// * `bpm` - Beats per minute
/// * `description` - Optional description text
/// * `id` - File identifier
///
/// # Returns
/// * Formatted filename string (without extension)
///
/// # Examples
///
/// ```
/// use pipeline::core::naming::templates::{NamingTemplate, apply_template};
///
/// let result = apply_template(
///     &NamingTemplate::Standard,
///     "BASS",
///     "Cm",
///     140.0,
///     "Deep_Rolling",
///     "001"
/// );
/// assert_eq!(result, "BASS_Cm_140BPM_Deep_Rolling_001");
/// ```
pub fn apply_template(
    template: &NamingTemplate,
    category: &str,
    key: &str,
    bpm: f64,
    description: &str,
    id: &str,
) -> String {
    apply_template_extended(
        template,
        category,
        key,
        bpm,
        description,
        id,
        None,
        None,
        None,
        None,
    )
}

/// Extended template application with additional metadata
///
/// # Arguments
/// * `template` - The naming template to use
/// * `category` - File category
/// * `key` - Musical key
/// * `bpm` - Beats per minute
/// * `description` - Optional description
/// * `id` - File identifier
/// * `timesig` - Optional time signature (e.g., "4-4", "6-8")
/// * `pack` - Optional pack/folder name
/// * `original` - Optional original filename
/// * `layer_info` - Optional (layer_name, layer_number) for split files
///
/// # Returns
/// * Formatted filename string (without extension)
pub fn apply_template_extended(
    template: &NamingTemplate,
    category: &str,
    key: &str,
    bpm: f64,
    description: &str,
    id: &str,
    timesig: Option<&str>,
    pack: Option<&str>,
    original: Option<&str>,
    layer_info: Option<(&str, usize)>,
) -> String {
    match template {
        NamingTemplate::Standard => {
            format!("{}_{}_{:.0}BPM_{}_{}", category, key, bpm, description, id)
        },

        NamingTemplate::Compact => {
            format!("{}_{}_{:.0}BPM_{}", category, key, bpm, id)
        },

        NamingTemplate::BpmFirst => {
            format!("{:.0}BPM_{}_{}_{}", bpm, key, category, description)
        },

        NamingTemplate::Production => {
            let timesig_str = timesig.unwrap_or("4-4");
            let pack_str = pack.unwrap_or("Unknown");

            // Build base: {CATEGORY}_{TIMESIG}_{BPM}BPM_{KEY}_{ID}_{PACK}
            let mut result = format!(
                "{}_{}_{:.0}BPM_{}_{}_{}",
                category, timesig_str, bpm, key, id, pack_str
            );

            // Add layer info if this is a split file
            if let Some((layer_name, layer_num)) = layer_info {
                result.push_str(&format!("_{}_L{:02}", layer_name, layer_num));
            } else if let Some(orig) = original {
                // Add original filename if not a split file
                result.push_str(&format!("_{}", orig));
            }

            result
        },

        NamingTemplate::Custom(template_str) => template_str
            .replace("{CATEGORY}", category)
            .replace("{KEY}", key)
            .replace("{BPM}", &format!("{:.0}", bpm))
            .replace("{DESCRIPTION}", description)
            .replace("{ID}", id)
            .replace("{TIMESIG}", timesig.unwrap_or("4-4"))
            .replace("{PACK}", pack.unwrap_or("Unknown"))
            .replace("{ORIGINAL}", original.unwrap_or("")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_template() {
        let result = apply_template(
            &NamingTemplate::Standard,
            "BASS",
            "Cm",
            140.0,
            "Deep_Rolling",
            "001",
        );

        assert_eq!(result, "BASS_Cm_140BPM_Deep_Rolling_001");
    }

    #[test]
    fn test_compact_template() {
        let result = apply_template(
            &NamingTemplate::Compact,
            "KICK",
            "C",
            128.0,
            "", // Description ignored in compact
            "042",
        );

        assert_eq!(result, "KICK_C_128BPM_042");
    }

    #[test]
    fn test_bpm_first_template() {
        let result = apply_template(
            &NamingTemplate::BpmFirst,
            "LEAD",
            "Am",
            150.0,
            "Energetic",
            "123",
        );

        assert_eq!(result, "150BPM_Am_LEAD_Energetic");
    }

    #[test]
    fn test_custom_template() {
        let custom = NamingTemplate::Custom("{BPM}bpm_{KEY}_{CATEGORY}".to_string());

        let result = apply_template(
            &custom, "LEAD", "Am", 150.0, "", // Not used in this template
            "", // Not used in this template
        );

        assert_eq!(result, "150bpm_Am_LEAD");
    }

    #[test]
    fn test_custom_template_with_all_placeholders() {
        let custom =
            NamingTemplate::Custom("{ID}_{CATEGORY}_{KEY}_{BPM}_{DESCRIPTION}".to_string());

        let result = apply_template(&custom, "BASS", "Dm", 120.0, "Groovy", "999");

        assert_eq!(result, "999_BASS_Dm_120_Groovy");
    }

    #[test]
    fn test_default_template() {
        let default = NamingTemplate::default();
        assert_eq!(default, NamingTemplate::Standard);
    }

    #[test]
    fn test_bpm_rounding() {
        let result = apply_template(&NamingTemplate::Standard, "KICK", "C", 127.8, "desc", "001");

        assert!(result.contains("128BPM"));
    }

    #[test]
    fn test_empty_description() {
        let result = apply_template(&NamingTemplate::Standard, "BASS", "C", 120.0, "", "001");

        assert_eq!(result, "BASS_C_120BPM__001");
    }
}
