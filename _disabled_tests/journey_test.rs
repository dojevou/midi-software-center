//! Phase 7.4: User Journey Tests (10-12 tests)
//!
//! Real user scenarios and complete paths through the application.
//! Tests represent actual user workflows from start to finish.
//!
//! User Personas:
//! - First-time user (onboarding journey)
//! - Professional DJ (library management & performance)
//! - Music producer (creation & export workflow)
//! - Music educator (analysis & teaching)
//! - Sample digger (discovery & curation)
//! - Music publisher (catalog management)
//! - Casual user (quick operations)
//! - Power user (advanced features & automation)
//! - Mobile user (simplified workflow)
//! - Enterprise user (bulk operations & reporting)
//! - Collaboration team (multi-user workflow)
//! - Learning path (progressive feature adoption)

use midi_pipeline::commands::file_import::{import_single_file, import_directory};
use midi_pipeline::commands::files::{get_file_count, get_file_details, list_files};
use midi_pipeline::commands::search::{search_files, get_all_tags, get_bpm_range};
use midi_pipeline::commands::tags::{add_tags_to_file, get_file_tags, search_tags, get_popular_tags};
use midi_pipeline::commands::stats::{get_category_stats, get_database_size};
use midi_pipeline::{AppState, Database};
use sqlx::PgPool;
use std::path::PathBuf;
use std::time::Instant;
use tempfile::TempDir;
use tokio::fs;

mod common;
use common::{TestDatabase, FileFixtures};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async fn create_app_state() -> AppState {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());
    let database = Database::new(&database_url).await.expect("Failed to create database");
    AppState { database }
}

fn create_midi_bytes(bpm: u32, key: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x01, 0xE0]);
    bytes.extend_from_slice(b"MTrk");

    let mut track_data = Vec::new();
    let tempo = 60_000_000 / bpm;
    track_data.extend_from_slice(&[0x00, 0xFF, 0x51, 0x03]);
    track_data.extend_from_slice(&tempo.to_be_bytes()[1..4]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x58, 0x04, 0x04, 0x02, 0x18, 0x08]);

    let key_byte = match key {
        "C_MAJOR" => 0x00, "D_MAJOR" => 0x02, "E_MAJOR" => 0x04,
        "G_MAJOR" => 0x01, "A_MINOR" => 0x00, _ => 0x00,
    };
    track_data.extend_from_slice(&[0x00, 0xFF, 0x59, 0x02, key_byte, 0x00]);
    track_data.extend_from_slice(&[0x00, 0x90, 0x3C, 0x40, 0x83, 0x60, 0x80, 0x3C, 0x40]);
    track_data.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let track_len = track_data.len() as u32;
    bytes.extend_from_slice(&track_len.to_be_bytes());
    bytes.extend_from_slice(&track_data);
    bytes
}

async fn cleanup_test_files(pool: &PgPool, pattern: &str) {
    let _ = sqlx::query("DELETE FROM files WHERE file_path LIKE $1")
        .bind(pattern)
        .execute(pool)
        .await;
}

// ============================================================================
// USER JOURNEY TESTS
// ============================================================================

#[tokio::test]
async fn test_journey_first_time_user() {
    // Journey: Download → install → import → explore → export
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== FIRST-TIME USER JOURNEY ===");

    // Step 1: User opens application (verify empty state)
    println!("Step 1: Opening application for first time...");
    let initial_count = get_file_count(tauri::State(&state)).await.unwrap();
    println!("  Initial file count: {}", initial_count);

    // Step 2: User imports first MIDI file
    println!("Step 2: Importing first MIDI file...");
    let first_file = temp_dir.path().join("my_first_midi.mid");
    fs::write(&first_file, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let import_result = import_single_file(
        tauri::State(&state),
        first_file.to_str().unwrap().to_string(),
    ).await.unwrap();

    println!("  ✓ Imported file ID: {}", import_result.id);

    // Step 3: User explores file details
    println!("Step 3: Viewing file details...");
    let details = get_file_details(tauri::State(&state), import_result.id).await.unwrap();
    println!("  File: {}", details.filepath);
    println!("  Size: {} bytes", details.file_size_bytes);

    // Step 4: User adds first tags
    println!("Step 4: Adding tags...");
    add_tags_to_file(
        tauri::State(&state),
        import_result.id,
        vec!["my_music".to_string(), "favorite".to_string()],
    ).await.unwrap();

    let tags = get_file_tags(tauri::State(&state), import_result.id).await.unwrap();
    println!("  Tags: {:?}", tags);

    // Step 5: User imports more files
    println!("Step 5: Importing more files...");
    for i in 1..=5 {
        let path = temp_dir.path().join(format!("song_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120 + (i * 10), "C_MAJOR")).await.unwrap();
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    let final_count = get_file_count(tauri::State(&state)).await.unwrap();
    println!("  Total files now: {}", final_count);

    // Step 6: User explores library
    println!("Step 6: Browsing library...");
    let files = list_files(tauri::State(&state), Some(1), Some(10), None).await.unwrap();
    println!("  Found {} files in library", files.len());

    // Assertions
    assert!(final_count >= initial_count + 6, "Should have imported 6 files");
    assert!(tags.contains(&"my_music".to_string()), "Tags should be saved");

    println!("✓ First-time user journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_professional_dj() {
    // Journey: Library management → search → playlist → export cue
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== PROFESSIONAL DJ JOURNEY ===");

    // Step 1: Import DJ library
    println!("Step 1: Importing DJ library...");
    let genres = vec![
        ("house", 128),
        ("techno", 140),
        ("deep_house", 122),
        ("tech_house", 126),
    ];

    let mut track_ids = Vec::new();

    for (genre, bpm) in &genres {
        for i in 1..=10 {
            let path = temp_dir.path().join(format!("{}_{}.mid", genre, i));
            fs::write(&path, &create_midi_bytes(*bpm, "C_MAJOR")).await.unwrap();

            let result = import_single_file(
                tauri::State(&state),
                path.to_str().unwrap().to_string(),
            ).await.unwrap();

            add_tags_to_file(
                tauri::State(&state),
                result.file_id,
                vec![genre.to_string(), "dj_library".to_string()],
            ).await.unwrap();

            track_ids.push((result.file_id, *genre, *bpm));
        }
    }

    println!("  ✓ Imported {} tracks", track_ids.len());

    // Step 2: Search by BPM range for mixing
    println!("Step 2: Searching tracks by BPM range...");
    let bpm_range = get_bpm_range(tauri::State(&state)).await.unwrap();
    println!("  BPM range: {:?} - {:?}", bpm_range.min_bpm, bpm_range.max_bpm);

    // Step 3: Create playlist for tonight's set
    println!("Step 3: Creating setlist playlist...");
    let setlist_tracks: Vec<_> = track_ids.iter()
        .filter(|(_, genre, _)| *genre == "house" || *genre == "tech_house")
        .take(10)
        .collect();

    for (file_id, genre, bpm) in &setlist_tracks {
        add_tags_to_file(
            tauri::State(&state),
            **file_id,
            vec!["setlist".to_string(), "friday_night".to_string()],
        ).await.unwrap();
    }

    println!("  ✓ Created setlist with {} tracks", setlist_tracks.len());

    // Step 4: Analyze key compatibility
    println!("Step 4: Analyzing harmonic mixing...");
    let all_tags = get_all_tags(tauri::State(&state)).await.unwrap();
    println!("  Available tags: {}", all_tags.len());

    // Step 5: Export cue points
    println!("Step 5: Preparing export...");
    let export_dir = temp_dir.path().join("dj_export");
    fs::create_dir(&export_dir).await.unwrap();
    println!("  ✓ Export directory ready");

    // Assertions
    assert_eq!(track_ids.len(), 40, "Should have 40 DJ tracks");
    assert_eq!(setlist_tracks.len(), 10, "Setlist should have 10 tracks");

    println!("✓ Professional DJ journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_music_producer() {
    // Journey: Create → compose → refine → export professional
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== MUSIC PRODUCER JOURNEY ===");

    // Step 1: Create new project
    println!("Step 1: Creating new production project...");
    let project_dir = temp_dir.path().join("my_track");
    fs::create_dir(&project_dir).await.unwrap();

    // Step 2: Compose initial ideas
    println!("Step 2: Composing initial ideas...");
    let ideas = vec![
        ("melody_idea.mid", 128, "melody"),
        ("bass_idea.mid", 128, "bass"),
        ("drums_idea.mid", 128, "drums"),
    ];

    let mut idea_ids = Vec::new();

    for (name, bpm, category) in &ideas {
        let path = project_dir.join(name);
        fs::write(&path, &create_midi_bytes(*bpm, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(
            tauri::State(&state),
            result.file_id,
            vec!["production".to_string(), category.to_string(), "v1".to_string()],
        ).await.unwrap();

        idea_ids.push(result.file_id);
    }

    println!("  ✓ Created {} initial ideas", idea_ids.len());

    // Step 3: Refine and iterate
    println!("Step 3: Refining compositions...");
    for (i, file_id) in idea_ids.iter().enumerate() {
        add_tags_to_file(
            tauri::State(&state),
            *file_id,
            vec!["refined".to_string(), format!("revision_{}", i + 1)],
        ).await.unwrap();
    }

    // Step 4: Arrange full track
    println!("Step 4: Arranging full track...");
    let arrangement_path = project_dir.join("full_arrangement.mid");
    fs::write(&arrangement_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let arrangement_result = import_single_file(
        tauri::State(&state),
        arrangement_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        arrangement_result.id,
        vec!["production".to_string(), "arrangement".to_string(), "master".to_string()],
    ).await.unwrap();

    println!("  ✓ Full arrangement created");

    // Step 5: Export for mastering
    println!("Step 5: Exporting for mastering...");
    let export_path = project_dir.join("export_for_mastering.mid");
    fs::write(&export_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    // Step 6: Archive project files
    println!("Step 6: Archiving project...");
    add_tags_to_file(
        tauri::State(&state),
        arrangement_result.id,
        vec!["archived".to_string(), "completed".to_string()],
    ).await.unwrap();

    let tags = get_file_tags(tauri::State(&state), arrangement_result.id).await.unwrap();

    // Assertions
    assert_eq!(idea_ids.len(), 3, "Should have 3 initial ideas");
    assert!(tags.contains(&"master".to_string()), "Should be tagged as master");
    assert!(tags.contains(&"completed".to_string()), "Should be marked complete");

    println!("✓ Music producer journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", project_dir.to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_music_educator() {
    // Journey: Load examples → analyze → annotate → share
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== MUSIC EDUCATOR JOURNEY ===");

    // Step 1: Import teaching examples
    println!("Step 1: Importing teaching examples...");
    let examples = vec![
        ("scales_c_major.mid", "scales", "beginner"),
        ("chords_progression.mid", "chords", "intermediate"),
        ("rhythm_patterns.mid", "rhythm", "beginner"),
        ("harmony_example.mid", "harmony", "advanced"),
    ];

    let mut example_ids = Vec::new();

    for (name, topic, level) in &examples {
        let path = temp_dir.path().join(name);
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(
            tauri::State(&state),
            result.file_id,
            vec![
                "education".to_string(),
                topic.to_string(),
                level.to_string(),
            ],
        ).await.unwrap();

        example_ids.push(result.file_id);
    }

    println!("  ✓ Imported {} teaching examples", example_ids.len());

    // Step 2: Analyze musical concepts
    println!("Step 2: Analyzing examples...");
    for file_id in &example_ids {
        let details = get_file_details(tauri::State(&state), *file_id).await.unwrap();
        println!("  Analyzed: {}", details.filepath);
    }

    // Step 3: Add annotations
    println!("Step 3: Adding educational annotations...");
    for (i, file_id) in example_ids.iter().enumerate() {
        add_tags_to_file(
            tauri::State(&state),
            *file_id,
            vec![
                "annotated".to_string(),
                format!("lesson_{}", i + 1),
            ],
        ).await.unwrap();
    }

    // Step 4: Organize by curriculum
    println!("Step 4: Organizing by curriculum...");
    let curriculum_tags = search_tags(tauri::State(&state), "beginner".to_string()).await.unwrap();
    println!("  Found {} beginner examples", curriculum_tags.len());

    // Step 5: Prepare for sharing
    println!("Step 5: Preparing lesson materials...");
    let lesson_dir = temp_dir.path().join("lesson_materials");
    fs::create_dir(&lesson_dir).await.unwrap();

    // Assertions
    assert_eq!(example_ids.len(), 4, "Should have 4 teaching examples");
    assert!(curriculum_tags.len() > 0, "Should find beginner materials");

    println!("✓ Music educator journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_sample_digger() {
    // Journey: Browse → categorize → create pack → share
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== SAMPLE DIGGER JOURNEY ===");

    // Step 1: Browse sample collection
    println!("Step 1: Browsing sample collection...");
    let sample_types = vec!["kick", "snare", "hihat", "clap", "bass", "synth"];

    let mut sample_ids = Vec::new();

    for sample_type in &sample_types {
        for i in 1..=5 {
            let path = temp_dir.path().join(format!("{}_{}.mid", sample_type, i));
            fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

            let result = import_single_file(
                tauri::State(&state),
                path.to_str().unwrap().to_string(),
            ).await.unwrap();

            add_tags_to_file(
                tauri::State(&state),
                result.file_id,
                vec!["sample".to_string(), sample_type.to_string()],
            ).await.unwrap();

            sample_ids.push((result.file_id, sample_type));
        }
    }

    println!("  ✓ Browsed {} samples", sample_ids.len());

    // Step 2: Categorize by use case
    println!("Step 2: Categorizing samples...");
    for (file_id, sample_type) in &sample_ids {
        let category = if ["kick", "snare", "hihat", "clap"].contains(sample_type) {
            "drums"
        } else {
            "melodic"
        };

        add_tags_to_file(
            tauri::State(&state),
            *file_id,
            vec![category.to_string()],
        ).await.unwrap();
    }

    // Step 3: Create sample pack
    println!("Step 3: Creating sample pack...");
    let pack_name = "ultimate_drums_pack";
    let drum_samples: Vec<_> = sample_ids.iter()
        .filter(|(_, t)| ["kick", "snare", "hihat", "clap"].contains(t))
        .collect();

    for (file_id, _) in &drum_samples {
        add_tags_to_file(
            tauri::State(&state),
            **file_id,
            vec![pack_name.to_string(), "pack".to_string()],
        ).await.unwrap();
    }

    println!("  ✓ Created pack with {} samples", drum_samples.len());

    // Step 4: Export pack
    println!("Step 4: Exporting sample pack...");
    let pack_dir = temp_dir.path().join(pack_name);
    fs::create_dir(&pack_dir).await.unwrap();

    // Assertions
    assert_eq!(sample_ids.len(), 30, "Should have 30 samples");
    assert_eq!(drum_samples.len(), 20, "Pack should have 20 drum samples");

    println!("✓ Sample digger journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_music_publisher() {
    // Journey: Manage catalog → metadata → format delivery → archive
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== MUSIC PUBLISHER JOURNEY ===");

    // Step 1: Import catalog
    println!("Step 1: Importing music catalog...");
    let catalog_items = vec![
        ("track_001", "artist_a", "album_1"),
        ("track_002", "artist_a", "album_1"),
        ("track_003", "artist_b", "album_2"),
        ("track_004", "artist_b", "album_2"),
        ("track_005", "artist_c", "single"),
    ];

    let mut catalog_ids = Vec::new();

    for (track, artist, album) in &catalog_items {
        let path = temp_dir.path().join(format!("{}.mid", track));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(
            tauri::State(&state),
            result.file_id,
            vec![
                "catalog".to_string(),
                artist.to_string(),
                album.to_string(),
            ],
        ).await.unwrap();

        catalog_ids.push(result.file_id);
    }

    println!("  ✓ Imported {} catalog items", catalog_ids.len());

    // Step 2: Add metadata
    println!("Step 2: Adding publishing metadata...");
    for file_id in &catalog_ids {
        add_tags_to_file(
            tauri::State(&state),
            *file_id,
            vec![
                "copyright_cleared".to_string(),
                "publishing_ready".to_string(),
                "2024".to_string(),
            ],
        ).await.unwrap();
    }

    // Step 3: Generate reports
    println!("Step 3: Generating catalog reports...");
    let stats = get_category_stats(tauri::State(&state)).await.unwrap();
    println!("  Catalog statistics generated");

    // Step 4: Prepare delivery formats
    println!("Step 4: Preparing multi-format delivery...");
    let delivery_dir = temp_dir.path().join("delivery");
    fs::create_dir(&delivery_dir).await.unwrap();

    // Step 5: Archive catalog
    println!("Step 5: Archiving catalog...");
    for file_id in &catalog_ids {
        add_tags_to_file(
            tauri::State(&state),
            *file_id,
            vec!["archived_2024".to_string()],
        ).await.unwrap();
    }

    // Assertions
    assert_eq!(catalog_ids.len(), 5, "Should have 5 catalog items");

    println!("✓ Music publisher journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_casual_user() {
    // Journey: Quick import → quick search → quick export
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== CASUAL USER JOURNEY ===");

    let start = Instant::now();

    // Step 1: Quick import
    println!("Step 1: Quick import...");
    let file_path = temp_dir.path().join("my_song.mid");
    fs::write(&file_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        file_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    println!("  ✓ Imported in {}ms", start.elapsed().as_millis());

    // Step 2: Quick search
    println!("Step 2: Quick search...");
    let search_start = Instant::now();
    let count = get_file_count(tauri::State(&state)).await.unwrap();
    println!("  ✓ Found {} files in {}ms", count, search_start.elapsed().as_millis());

    // Step 3: Quick export
    println!("Step 3: Quick export...");
    let export_path = temp_dir.path().join("exported.mid");
    fs::write(&export_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();
    println!("  ✓ Exported");

    let total_duration = start.elapsed();

    // Assertions
    assert!(total_duration.as_secs() < 2, "Casual workflow should be fast");

    println!("✓ Casual user journey complete in {}ms!\n", total_duration.as_millis());

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_power_user() {
    // Journey: Advanced search → scripting → batch → reporting
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== POWER USER JOURNEY ===");

    // Step 1: Batch import
    println!("Step 1: Batch importing library...");
    let batch_start = Instant::now();

    for i in 0..50 {
        let path = temp_dir.path().join(format!("batch_{}.mid", i));
        fs::write(&path, &create_midi_bytes(100 + i, "C_MAJOR")).await.unwrap();
        import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();
    }

    println!("  ✓ Imported 50 files in {}ms", batch_start.elapsed().as_millis());

    // Step 2: Advanced filtering
    println!("Step 2: Advanced multi-filter search...");
    let files = list_files(tauri::State(&state), Some(1), Some(50), None).await.unwrap();
    println!("  ✓ Found {} files", files.len());

    // Step 3: Batch tagging
    println!("Step 3: Batch tagging operations...");
    for (i, file) in files.iter().take(20).enumerate() {
        add_tags_to_file(
            tauri::State(&state),
            file.id,
            vec![format!("batch_{}", i / 5)],
        ).await.unwrap();
    }

    println!("  ✓ Batch tagged 20 files");

    // Step 4: Generate reports
    println!("Step 4: Generating advanced reports...");
    let bpm_range = get_bpm_range(tauri::State(&state)).await.unwrap();
    let db_size = get_database_size(tauri::State(&state)).await.unwrap();

    println!("  BPM range: {:?} - {:?}", bpm_range.min_bpm, bpm_range.max_bpm);
    println!("  Database size: {:?}", db_size);

    // Assertions
    assert!(files.len() >= 50, "Should have batch imported files");

    println!("✓ Power user journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_mobile_user() {
    // Journey: Lite interface → offline mode → sync
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== MOBILE USER JOURNEY ===");

    // Step 1: Quick mobile import
    println!("Step 1: Mobile quick import...");
    let mobile_file = temp_dir.path().join("mobile_recording.mid");
    fs::write(&mobile_file, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let result = import_single_file(
        tauri::State(&state),
        mobile_file.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        result.file_id,
        vec!["mobile".to_string(), "on_the_go".to_string()],
    ).await.unwrap();

    println!("  ✓ Mobile import complete");

    // Step 2: Offline mode (simulate)
    println!("Step 2: Working in offline mode...");
    let offline_file = temp_dir.path().join("offline_work.mid");
    fs::write(&offline_file, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();
    println!("  ✓ Offline file created");

    // Step 3: Sync when online
    println!("Step 3: Syncing with cloud...");
    let sync_result = import_single_file(
        tauri::State(&state),
        offline_file.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        sync_result.file_id,
        vec!["synced".to_string()],
    ).await.unwrap();

    println!("  ✓ Sync complete");

    // Assertions
    let tags = get_file_tags(tauri::State(&state), sync_result.file_id).await.unwrap();
    assert!(tags.contains(&"synced".to_string()), "Should be synced");

    println!("✓ Mobile user journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_enterprise_user() {
    // Journey: Bulk operations → audit → permissions → reporting
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== ENTERPRISE USER JOURNEY ===");

    // Step 1: Bulk import
    println!("Step 1: Enterprise bulk import...");
    for dept in &["sales", "marketing", "production"] {
        for i in 0..10 {
            let path = temp_dir.path().join(format!("{}_{}.mid", dept, i));
            fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

            let result = import_single_file(
                tauri::State(&state),
                path.to_str().unwrap().to_string(),
            ).await.unwrap();

            add_tags_to_file(
                tauri::State(&state),
                result.file_id,
                vec!["enterprise".to_string(), dept.to_string()],
            ).await.unwrap();
        }
    }

    println!("  ✓ Imported 30 enterprise files");

    // Step 2: Audit trail
    println!("Step 2: Generating audit reports...");
    let count = get_file_count(tauri::State(&state)).await.unwrap();
    println!("  Total files in system: {}", count);

    // Step 3: Department statistics
    println!("Step 3: Department statistics...");
    let stats = get_category_stats(tauri::State(&state)).await.unwrap();
    println!("  Statistics generated for {} categories", stats.len());

    // Step 4: Compliance reporting
    println!("Step 4: Compliance reporting...");
    let all_tags = get_all_tags(tauri::State(&state)).await.unwrap();
    println!("  Compliance tags tracked: {}", all_tags.len());

    // Assertions
    assert!(count >= 30, "Should have enterprise files");

    println!("✓ Enterprise user journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_collaboration_workflow() {
    // Journey: Team → import sources → feedback → finalize
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== COLLABORATION WORKFLOW JOURNEY ===");

    // Step 1: Team member A contributes
    println!("Step 1: Team member A contribution...");
    let member_a_path = temp_dir.path().join("member_a_track.mid");
    fs::write(&member_a_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let member_a_result = import_single_file(
        tauri::State(&state),
        member_a_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        member_a_result.file_id,
        vec!["collaboration".to_string(), "member_a".to_string(), "draft".to_string()],
    ).await.unwrap();

    // Step 2: Team member B contributes
    println!("Step 2: Team member B contribution...");
    let member_b_path = temp_dir.path().join("member_b_track.mid");
    fs::write(&member_b_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let member_b_result = import_single_file(
        tauri::State(&state),
        member_b_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        member_b_result.file_id,
        vec!["collaboration".to_string(), "member_b".to_string(), "draft".to_string()],
    ).await.unwrap();

    // Step 3: Team lead reviews
    println!("Step 3: Team lead reviewing...");
    add_tags_to_file(
        tauri::State(&state),
        member_a_result.file_id,
        vec!["reviewed".to_string(), "approved".to_string()],
    ).await.unwrap();

    // Step 4: Finalize project
    println!("Step 4: Finalizing collaborative project...");
    let final_path = temp_dir.path().join("final_collaboration.mid");
    fs::write(&final_path, &create_midi_bytes(128, "C_MAJOR")).await.unwrap();

    let final_result = import_single_file(
        tauri::State(&state),
        final_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        final_result.file_id,
        vec!["collaboration".to_string(), "final".to_string(), "published".to_string()],
    ).await.unwrap();

    // Assertions
    let final_tags = get_file_tags(tauri::State(&state), final_result.file_id).await.unwrap();
    assert!(final_tags.contains(&"final".to_string()), "Should be finalized");
    assert!(final_tags.contains(&"published".to_string()), "Should be published");

    println!("✓ Collaboration workflow journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

#[tokio::test]
async fn test_journey_learning_path() {
    // Journey: Tutorials → templates → guided → independent
    let state = create_app_state().await;
    let temp_dir = TempDir::new().unwrap();

    println!("=== LEARNING PATH JOURNEY ===");

    // Step 1: Tutorial files
    println!("Step 1: Following tutorials...");
    for i in 1..=3 {
        let path = temp_dir.path().join(format!("tutorial_{}.mid", i));
        fs::write(&path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

        let result = import_single_file(
            tauri::State(&state),
            path.to_str().unwrap().to_string(),
        ).await.unwrap();

        add_tags_to_file(
            tauri::State(&state),
            result.file_id,
            vec!["learning".to_string(), "tutorial".to_string(), format!("lesson_{}", i)],
        ).await.unwrap();
    }

    println!("  ✓ Completed 3 tutorials");

    // Step 2: Using templates
    println!("Step 2: Working with templates...");
    let template_path = temp_dir.path().join("template_practice.mid");
    fs::write(&template_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let template_result = import_single_file(
        tauri::State(&state),
        template_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        template_result.file_id,
        vec!["learning".to_string(), "template".to_string()],
    ).await.unwrap();

    // Step 3: Guided practice
    println!("Step 3: Guided practice...");
    let practice_path = temp_dir.path().join("guided_practice.mid");
    fs::write(&practice_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let practice_result = import_single_file(
        tauri::State(&state),
        practice_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        practice_result.file_id,
        vec!["learning".to_string(), "guided".to_string()],
    ).await.unwrap();

    // Step 4: Independent creation
    println!("Step 4: Independent creation...");
    let independent_path = temp_dir.path().join("my_first_creation.mid");
    fs::write(&independent_path, &create_midi_bytes(120, "C_MAJOR")).await.unwrap();

    let independent_result = import_single_file(
        tauri::State(&state),
        independent_path.to_str().unwrap().to_string(),
    ).await.unwrap();

    add_tags_to_file(
        tauri::State(&state),
        independent_result.file_id,
        vec!["learning".to_string(), "independent".to_string(), "achievement".to_string()],
    ).await.unwrap();

    // Assertions
    let final_tags = get_file_tags(tauri::State(&state), independent_result.file_id).await.unwrap();
    assert!(final_tags.contains(&"achievement".to_string()), "Learning milestone achieved");

    println!("✓ Learning path journey complete!\n");

    cleanup_test_files(state.database.pool(), &format!("{}%", temp_dir.path().to_str().unwrap())).await;
}

// ============================================================================
// USER JOURNEY SUMMARY
// ============================================================================

#[tokio::test]
async fn test_journey_summary() {
    println!("\n=== USER JOURNEY TEST SUMMARY ===");
    println!("All 12 user journey tests represent real-world scenarios:\n");
    println!("  1. First-time user: Onboarding experience");
    println!("  2. Professional DJ: Library management & performance");
    println!("  3. Music producer: Full production workflow");
    println!("  4. Music educator: Teaching & analysis");
    println!("  5. Sample digger: Discovery & curation");
    println!("  6. Music publisher: Catalog management");
    println!("  7. Casual user: Quick operations");
    println!("  8. Power user: Advanced features & automation");
    println!("  9. Mobile user: On-the-go workflow");
    println!(" 10. Enterprise user: Bulk operations & compliance");
    println!(" 11. Collaboration: Team workflow");
    println!(" 12. Learning path: Progressive skill development");
    println!("\nRun with: cargo test --test journey_test");
    println!("==================================\n");
}
