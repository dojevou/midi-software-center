/// Test DuckDB analytics integration and performance
use midi_app::services::VIP3AnalyticsService;
use std::time::Instant;

#[tokio::test]
async fn test_duckdb_analytics_service() {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("Initializing VIP3 Analytics Service...");
    let service =
        VIP3AnalyticsService::new(database_url).expect("Failed to create analytics service");

    println!("Testing filter counts aggregation...");
    let start = Instant::now();
    let counts = service.get_filter_counts().await.expect("Failed to get filter counts");
    let duration = start.elapsed();

    println!("\n✅ DuckDB Analytics Performance:");
    println!("   Query time: {:?}", duration);
    println!("\n📊 Results:");
    println!("   BPM ranges: {} categories", counts.bpm_ranges.len());
    println!("   Keys: {} categories", counts.keys.len());
    println!("   Folders: {} categories", counts.folders.len());
    println!("   Instruments: {} categories", counts.instruments.len());
    println!("   Timbres: {} categories", counts.timbres.len());
    println!("   Styles: {} categories", counts.styles.len());
    println!(
        "   Articulations: {} categories",
        counts.articulations.len()
    );
    println!(
        "   Channel counts: {} categories",
        counts.channel_counts.len()
    );
    println!("   Multi-track files: {}", counts.multi_track);

    // Verify we got reasonable results
    assert!(!counts.bpm_ranges.is_empty(), "Should have BPM ranges");
    assert!(!counts.keys.is_empty(), "Should have keys");

    println!("\n✅ All assertions passed!");

    // Performance target check
    if duration.as_millis() < 100 {
        println!("🚀 EXCELLENT: Query completed in <100ms (target achieved!)");
    } else if duration.as_millis() < 200 {
        println!("✅ GOOD: Query completed in <200ms (within acceptable range)");
    } else {
        println!(
            "⚠️  SLOW: Query took {}ms (target was <100ms)",
            duration.as_millis()
        );
    }
}
