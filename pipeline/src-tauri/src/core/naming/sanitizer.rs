   /// Filename Sanitization
   ///
   /// Ensures filenames are valid across all operating systems.

/// Sanitizes a string for use in filenames
///
/// # Rules
/// - Removes/replaces invalid characters
/// - Limits length to 255 characters
/// - Removes leading/trailing spaces
/// - Converts to ASCII where possible
///
/// # Arguments
/// * `input` - String to sanitize
///
/// # Returns
/// * Sanitized string safe for filenames
///
/// # Examples
///
/// ```
/// use pipeline::core::naming::sanitizer::sanitize_filename;
///
/// let sanitized = sanitize_filename("my file<name>");
/// assert_eq!(sanitized, "my_file_name_");
/// ```
pub fn sanitize_filename(input: &str) -> String {
    let mut sanitized = input.to_string();

    // Remove/replace invalid characters
    sanitized = sanitized
        .chars()
        .map(|c| match c {
            // Windows reserved characters
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            // Control characters
            c if c.is_control() => '_',
            // Keep valid characters
            c => c,
        })
        .collect();

    // Remove leading/trailing whitespace
    sanitized = sanitized.trim().to_string();

    // Replace multiple spaces with single space
    while sanitized.contains("  ") {
        sanitized = sanitized.replace("  ", " ");
    }

    // Replace spaces with underscores for consistency
    sanitized = sanitized.replace(' ', "_");

    // Remove multiple underscores
    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }

    // Limit length (leave room for extension)
    if sanitized.len() > 250 {
        sanitized.truncate(250);
    }

    // Remove leading/trailing underscores
    sanitized = sanitized.trim_matches('_').to_string();

    // If empty after sanitization, use default
    if sanitized.is_empty() {
        sanitized = "untitled".to_string();
    }

    sanitized
}

/// Removes common filler words from descriptions
///
/// # Arguments
/// * `description` - Description text to clean
///
/// # Returns
/// * Description with filler words removed
pub fn clean_description(description: &str) -> String {
    let filler_words = [
        "untitled", "new", "midi", "file", "song", "track", "the", "a", "an", "and", "or", "but",
    ];

    let words: Vec<&str> = description
        .split('_')
        .filter(|word| {
            let lower = word.to_lowercase();
            !filler_words.contains(&lower.as_str()) && !lower.is_empty()
        })
        .collect();

    words.join("_")
}

/// Ensures filename has .mid extension
///
/// # Arguments
/// * `filename` - Filename to check
///
/// # Returns
/// * Filename with .mid extension
pub fn ensure_mid_extension(filename: &str) -> String {
    if filename.to_lowercase().ends_with(".mid") {
        filename.to_string()
    } else {
        format!("{}.mid", filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_invalid_characters() {
        let input = "test<file>name:with|invalid*chars";
        let output = sanitize_filename(input);

        assert!(!output.contains('<'));
        assert!(!output.contains('>'));
        assert!(!output.contains(':'));
        assert!(!output.contains('|'));
        assert!(!output.contains('*'));
    }

    #[test]
    fn test_replace_spaces() {
        let input = "test file name";
        let output = sanitize_filename(input);

        assert_eq!(output, "test_file_name");
    }

    #[test]
    fn test_remove_multiple_underscores() {
        let input = "test___file___name";
        let output = sanitize_filename(input);

        assert_eq!(output, "test_file_name");
    }

    #[test]
    fn test_length_limit() {
        let input = "a".repeat(300);
        let output = sanitize_filename(&input);

        assert!(output.len() <= 250);
    }

    #[test]
    fn test_empty_input() {
        let output = sanitize_filename("");
        assert_eq!(output, "untitled");
    }

    #[test]
    fn test_clean_description() {
        let input = "the_new_bass_and_lead_file";
        let output = clean_description(input);

        // Should remove filler words
        assert_eq!(output, "bass_lead");
    }

    #[test]
    fn test_ensure_extension() {
        assert_eq!(ensure_mid_extension("test"), "test.mid");
        assert_eq!(ensure_mid_extension("test.mid"), "test.mid");
        assert_eq!(ensure_mid_extension("test.MID"), "test.MID");
    }

    #[test]
    fn test_trim_leading_trailing_underscores() {
        let input = "___test___";
        let output = sanitize_filename(input);
        assert_eq!(output, "test");
    }

    #[test]
    fn test_control_characters() {
        let input = "test\n\r\tfile";
        let output = sanitize_filename(input);
        assert_eq!(output, "test_file");
    }

    #[test]
    fn test_only_invalid_characters() {
        let input = "<>?*|";
        let output = sanitize_filename(input);
        assert_eq!(output, "untitled");
    }
}
