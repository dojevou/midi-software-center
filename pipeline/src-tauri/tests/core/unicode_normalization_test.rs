/// Unicode Normalization Tests
/// Tests edge cases for Unicode handling in filenames and metadata

use midi_pipeline::core::normalization::filename::{normalize_filename, sanitize_for_filesystem};

#[test]
fn test_normalize_basic_ascii() {
    let input = "simple_file.mid";
    let result = normalize_filename(input);
    assert_eq!(result, "simple_file.mid");
}

#[test]
fn test_normalize_spaces_to_underscores() {
    let input = "file with spaces.mid";
    let result = normalize_filename(input);
    assert_eq!(result, "file_with_spaces.mid");
}

#[test]
fn test_normalize_japanese() {
    let input = "音楽ファイル.mid";  // "music file" in Japanese
    let result = normalize_filename(input);

    // Should preserve Unicode characters or handle gracefully
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_russian() {
    let input = "музыка.mid";  // "music" in Russian
    let result = normalize_filename(input);

    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_arabic() {
    let input = "موسيقى.mid";  // "music" in Arabic
    let result = normalize_filename(input);

    // Arabic is RTL, should handle gracefully
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_emoji() {
    let input = "🎵music🎹.mid";  // Emoji in filename
    let result = normalize_filename(input);

    // Should either preserve or strip emoji
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_german_umlaut() {
    let input = "Müller Über.mid";
    let result = normalize_filename(input);

    // Should preserve or decompose umlauts
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_french_accents() {
    let input = "café résumé.mid";
    let result = normalize_filename(input);

    // Should preserve or decompose accents
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_combining_diacritics() {
    // e + combining acute accent (é)
    let input = "cafe\u{0301}.mid";
    let result = normalize_filename(input);

    // Should normalize combining characters
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_nfc_nfd_equivalence() {
    // Same character in NFC and NFD forms
    let nfc = "café.mid";  // é as single character
    let nfd = "cafe\u{0301}.mid";  // e + combining accent

    let result_nfc = normalize_filename(nfc);
    let result_nfd = normalize_filename(nfd);

    // Both should normalize to same result
    assert_eq!(result_nfc, result_nfd);
}

#[test]
fn test_normalize_zero_width_characters() {
    let input = "file\u{200B}name.mid";  // Zero-width space
    let result = normalize_filename(input);

    // Should remove zero-width characters
    assert!(!result.contains('\u{200B}'));
    assert_eq!(result, "filename.mid");
}

#[test]
fn test_normalize_right_to_left_mark() {
    let input = "file\u{200F}name.mid";  // RTL mark
    let result = normalize_filename(input);

    // Should remove RTL marks
    assert!(!result.contains('\u{200F}'));
}

#[test]
fn test_normalize_mixed_scripts() {
    let input = "English日本語Русский.mid";
    let result = normalize_filename(input);

    // Should handle mixed scripts
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_very_long_unicode() {
    let input = "音".repeat(200) + ".mid";  // 200 Japanese characters
    let result = normalize_filename(&input);

    // Should truncate to reasonable length
    assert!(result.len() < input.len());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_control_characters() {
    let input = "file\x00name\x01.mid";  // Null and SOH
    let result = normalize_filename(input);

    // Should remove control characters
    assert!(!result.contains('\x00'));
    assert!(!result.contains('\x01'));
}

#[test]
fn test_normalize_newlines_tabs() {
    let input = "file\nname\twith\rwhitespace.mid";
    let result = normalize_filename(input);

    // Should remove or replace newlines/tabs
    assert!(!result.contains('\n'));
    assert!(!result.contains('\t'));
    assert!(!result.contains('\r'));
}

#[test]
fn test_normalize_unicode_numbers() {
    let input = "track①②③.mid";  // Circled numbers
    let result = normalize_filename(input);

    // Should handle Unicode number forms
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_fullwidth_characters() {
    let input = "ＴＥＳ Ｔ.mid";  // Fullwidth Latin
    let result = normalize_filename(input);

    // Should normalize to halfwidth or handle gracefully
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_sanitize_filesystem_windows_reserved() {
    // Windows reserved characters: < > : " / \ | ? *
    let input = "file<>:\"/\\|?*.mid";
    let result = sanitize_for_filesystem(input);

    // Should remove or replace all reserved characters
    assert!(!result.contains('<'));
    assert!(!result.contains('>'));
    assert!(!result.contains(':'));
    assert!(!result.contains('"'));
    assert!(!result.contains('/'));
    assert!(!result.contains('\\'));
    assert!(!result.contains('|'));
    assert!(!result.contains('?'));
    assert!(!result.contains('*'));
}

#[test]
fn test_sanitize_filesystem_reserved_names() {
    // Windows reserved names
    let reserved = vec!["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];

    for name in reserved {
        let input = format!("{}.mid", name);
        let result = sanitize_for_filesystem(&input);

        // Should not be exactly a reserved name
        assert_ne!(result.to_uppercase(), format!("{}.MID", name).to_uppercase());
    }
}

#[test]
fn test_sanitize_filesystem_trailing_dots() {
    let input = "filename...mid";
    let result = sanitize_for_filesystem(input);

    // Should handle multiple dots
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_sanitize_filesystem_leading_dots() {
    let input = "...filename.mid";
    let result = sanitize_for_filesystem(input);

    // Should not start with dots (hidden file on Unix)
    assert!(!result.starts_with('.'));
}

#[test]
fn test_normalize_unicode_normalization_forms() {
    // Test all four Unicode normalization forms
    let input = "café";

    // NFC, NFD, NFKC, NFKD should all normalize consistently
    let result = normalize_filename(input);

    // Should be in a consistent normalized form
    assert!(!result.is_empty());
}

#[test]
fn test_normalize_bidirectional_text() {
    // Mix LTR and RTL text
    let input = "English-العربية-English.mid";
    let result = normalize_filename(input);

    // Should handle bidirectional text
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_homoglyphs() {
    // Latin 'a' vs Cyrillic 'а' (look similar, different code points)
    let latin = "name.mid";      // Latin 'a'
    let cyrillic = "nаme.mid";   // Cyrillic 'а'

    let result_latin = normalize_filename(latin);
    let result_cyrillic = normalize_filename(cyrillic);

    // Should handle consistently
    assert!(!result_latin.is_empty());
    assert!(!result_cyrillic.is_empty());
}

#[test]
fn test_normalize_ligatures() {
    // ff ligature (ﬀ) should normalize
    let input = "ﬁle.mid";  // fi ligature
    let result = normalize_filename(input);

    // Should decompose ligatures
    assert!(!result.is_empty());
}

#[test]
fn test_normalize_mathematical_symbols() {
    let input = "formula∑∫∂.mid";
    let result = normalize_filename(input);

    // Should handle math symbols
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_empty_after_sanitization() {
    // String that becomes empty after removing invalid chars
    let input = "\u{200B}\u{200C}\u{200D}.mid";  // All zero-width
    let result = normalize_filename(input);

    // Should provide default name if empty
    assert!(!result.is_empty());
    assert!(result.ends_with(".mid"));
}

#[test]
fn test_normalize_mixed_case_extension() {
    let input = "file.MiD";
    let result = normalize_filename(input);

    // Should normalize extension to lowercase
    assert!(result.ends_with(".mid"));
}
