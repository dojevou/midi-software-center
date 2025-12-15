// Phase 2 validation test - verify filename metadata extraction works correctly
#[cfg(test)]
mod phase2_validation {
    use crate::core::analysis::FilenameMetadata;

    #[test]
    fn test_dnb_160_electronic() {
        let filename = "dnb_160_electronic.mid";
        let meta = FilenameMetadata::extract_from_filename(filename);

        println!("\n=== Testing: {} ===", filename);
        println!("BPM:            {:?}", meta.bpm);
        println!("Key:            {:?}", meta.key);
        println!("Genres:         {:?}", meta.genres);
        println!("Structure tags: {:?}", meta.structure_tags);
        println!("Track number:   {:?}", meta.track_number);

        // Expected results
        assert_eq!(meta.bpm, Some(160.0), "Should extract BPM 160");
        assert!(
            meta.genres.contains(&"dnb".to_string())
                || meta.genres.contains(&"electronic".to_string()),
            "Should extract genre (dnb or electronic)"
        );
    }

    #[test]
    fn test_funk_120_shuffle() {
        let filename = "funk_120_shuffle.mid";
        let meta = FilenameMetadata::extract_from_filename(filename);

        println!("\n=== Testing: {} ===", filename);
        println!("BPM:            {:?}", meta.bpm);
        println!("Key:            {:?}", meta.key);
        println!("Genres:         {:?}", meta.genres);
        println!("Structure tags: {:?}", meta.structure_tags);
        println!("Track number:   {:?}", meta.track_number);

        // Expected results
        assert_eq!(meta.bpm, Some(120.0), "Should extract BPM 120");
        assert!(
            meta.genres.contains(&"funk".to_string()),
            "Should extract genre funk"
        );
    }

    #[test]
    fn test_jazz_136_swing() {
        let filename = "jazz_136_swing.mid";
        let meta = FilenameMetadata::extract_from_filename(filename);

        println!("\n=== Testing: {} ===", filename);
        println!("BPM:            {:?}", meta.bpm);
        println!("Key:            {:?}", meta.key);
        println!("Genres:         {:?}", meta.genres);
        println!("Structure tags: {:?}", meta.structure_tags);
        println!("Track number:   {:?}", meta.track_number);

        // Expected results
        assert_eq!(meta.bpm, Some(136.0), "Should extract BPM 136");
        assert!(
            meta.genres.contains(&"jazz".to_string()),
            "Should extract genre jazz"
        );
    }

    #[test]
    fn test_metadata_source_calculation() {
        // Test metadata_source logic
        let test_cases = vec![
            (Some(120.0), Some(120.0), "both"),
            (Some(120.0), None, "analyzed"),
            (None, Some(120.0), "filename"),
            (None, None, "none"),
        ];

        for (analyzed_bpm, filename_bpm, expected_source) in test_cases {
            let metadata_source = match (&analyzed_bpm, &filename_bpm) {
                (Some(_), Some(_)) => "both",
                (Some(_), None) => "analyzed",
                (None, Some(_)) => "filename",
                (None, None) => "none",
            };

            assert_eq!(
                metadata_source, expected_source,
                "BPM ({:?}, {:?}) should result in source '{}'",
                analyzed_bpm, filename_bpm, expected_source
            );
        }
    }

    #[test]
    fn test_database_insert_simulation() {
        // Simulate what would be inserted into the database
        let test_files =
            vec!["dnb_160_electronic.mid", "funk_120_shuffle.mid", "jazz_136_swing.mid"];

        println!("\n=== Database INSERT Simulation ===\n");

        for filename in test_files {
            let meta = FilenameMetadata::extract_from_filename(filename);

            // Simulate what file_import.rs does
            let analyzed_bpm: Option<f32> = None; // Would come from MIDI analysis
            let metadata_source = match (&analyzed_bpm, &meta.bpm) {
                (Some(_), Some(_)) => "both",
                (Some(_), None) => "analyzed",
                (None, Some(_)) => "filename",
                (None, None) => "none",
            };

            println!("File: {}", filename);
            println!("  filename_bpm:      {:?}", meta.bpm);
            println!("  filename_key:      {:?}", meta.key);
            println!("  filename_genres:   {:?}", meta.genres);
            println!("  structure_tags:    {:?}", meta.structure_tags);
            println!("  track_number:      {:?}", meta.track_number);
            println!("  metadata_source:   {}", metadata_source);
            println!();
        }
    }
}
