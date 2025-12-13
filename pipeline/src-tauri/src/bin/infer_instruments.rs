// Parallel instrument inference tool
// Combines Rust parallel processing with SQL batch updates for maximum speed

use rayon::prelude::*;
use regex::Regex;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;

#[derive(Debug, Clone)]
struct FileRecord {
    id: i64,
    filename: String,
    filepath: String,
}

#[derive(Debug)]
struct InstrumentPatterns {
    drums: Regex,
    bass: Regex,
    lead: Regex,
    synth: Regex,
    pad: Regex,
    pluck: Regex,
    keys: Regex,
    guitar: Regex,
    vocal: Regex,
    strings: Regex,
    brass: Regex,
    arp: Regex,
    fx: Regex,
    percussion: Regex,
    chord: Regex,
}

impl InstrumentPatterns {
    fn new() -> Self {
        Self {
            drums: Regex::new(r"(?i)(kick|snare|hat|hihat|cymbal|tom|clap|rim|cowbell|shaker|crash|ride|drum|groove|fill|beat|loop|kit)").unwrap(),
            bass: Regex::new(r"(?i)(^|[^a-z])(bass|sub|808)").unwrap(),
            lead: Regex::new(r"(?i)(^|[^a-z])(lead|melody)").unwrap(),
            synth: Regex::new(r"(?i)(^|[^a-z])synth").unwrap(),
            pad: Regex::new(r"(?i)(^|[^a-z])(pad|atmospher|ambient)").unwrap(),
            pluck: Regex::new(r"(?i)(^|[^a-z])(pluck|pizz)").unwrap(),
            keys: Regex::new(r"(?i)(^|[^a-z])(key|keys|piano|keyboard|rhodes|organ)").unwrap(),
            guitar: Regex::new(r"(?i)(^|[^a-z])(guitar|gtr)").unwrap(),
            vocal: Regex::new(r"(?i)(^|[^a-z])(vocal|voice|vox|choir)").unwrap(),
            strings: Regex::new(r"(?i)(^|[^a-z])(string|violin|cello|viola)").unwrap(),
            brass: Regex::new(r"(?i)(^|[^a-z])(brass|trumpet|horn|trombone|sax)").unwrap(),
            arp: Regex::new(r"(?i)(^|[^a-z])(arp|arpeggiat)").unwrap(),
            fx: Regex::new(r"(?i)(^|[^a-z])(fx|effect|sweep|riser|impact|transition)").unwrap(),
            percussion: Regex::new(r"(?i)(^|[^a-z])(perc|percussion|conga|bongo|tabla)").unwrap(),
            chord: Regex::new(r"(?i)(^|[^a-z])chord").unwrap(),
        }
    }

    fn infer_instrument(&self, filename: &str, filepath: &str) -> Option<String> {
        // Priority 1: Check filename (most specific)
        if self.drums.is_match(filename) || self.drums.is_match(filepath) {
            return Some("drums".to_string());
        }
        if self.bass.is_match(filename) {
            return Some("bass".to_string());
        }
        if self.lead.is_match(filename) {
            return Some("lead".to_string());
        }
        if self.synth.is_match(filename) {
            return Some("synth".to_string());
        }
        if self.pad.is_match(filename) {
            return Some("pad".to_string());
        }
        if self.pluck.is_match(filename) {
            return Some("pluck".to_string());
        }
        if self.keys.is_match(filename) || self.chord.is_match(filename) {
            return Some("keys".to_string());
        }
        if self.guitar.is_match(filename) {
            return Some("guitar".to_string());
        }
        if self.vocal.is_match(filename) {
            return Some("vocal".to_string());
        }
        if self.strings.is_match(filename) {
            return Some("strings".to_string());
        }
        if self.brass.is_match(filename) {
            return Some("brass".to_string());
        }
        if self.arp.is_match(filename) {
            return Some("arp".to_string());
        }
        if self.fx.is_match(filename) {
            return Some("fx".to_string());
        }
        if self.percussion.is_match(filename) {
            return Some("percussion".to_string());
        }

        // Priority 2: Check filepath patterns
        if self.bass.is_match(filepath) {
            return Some("bass".to_string());
        }
        if self.lead.is_match(filepath) {
            return Some("lead".to_string());
        }
        if self.synth.is_match(filepath) {
            return Some("synth".to_string());
        }
        if self.pad.is_match(filepath) {
            return Some("pad".to_string());
        }
        if self.pluck.is_match(filepath) {
            return Some("pluck".to_string());
        }
        if self.keys.is_match(filepath) {
            return Some("keys".to_string());
        }
        if self.guitar.is_match(filepath) {
            return Some("guitar".to_string());
        }
        if self.vocal.is_match(filepath) {
            return Some("vocal".to_string());
        }
        if self.strings.is_match(filepath) {
            return Some("strings".to_string());
        }
        if self.brass.is_match(filepath) {
            return Some("brass".to_string());
        }
        if self.arp.is_match(filepath) {
            return Some("arp".to_string());
        }
        if self.fx.is_match(filepath) {
            return Some("fx".to_string());
        }
        if self.percussion.is_match(filepath) {
            return Some("percussion".to_string());
        }

        None
    }
}

async fn batch_update_instruments(
    pool: &PgPool,
    updates: Vec<(i64, String)>,
) -> Result<u64, sqlx::Error> {
    if updates.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;

    // Build batch UPDATE using unnest for maximum performance
    let ids: Vec<i64> = updates.iter().map(|(id, _)| *id).collect();
    let instruments: Vec<String> = updates.iter().map(|(_, inst)| inst.clone()).collect();

    let result = sqlx::query(
        r#"
        UPDATE files
        SET instrument_names_text = ARRAY[data.instrument]
        FROM (
            SELECT unnest($1::bigint[]) as id, unnest($2::text[]) as instrument
        ) as data
        WHERE files.id = data.id
        "#,
    )
    .bind(&ids)
    .bind(&instruments)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(result.rows_affected())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       Parallel Instrument Inference Tool                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Get DATABASE_URL from environment
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("🔌 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(48) // High connection pool for parallel processing
        .connect(&database_url)
        .await?;

    println!("✅ Connected");
    println!();

    // Step 1: Fetch all files missing instrument metadata
    println!("📊 Fetching files missing instrument metadata...");
    let files: Vec<FileRecord> = sqlx::query_as::<_, (i64, String, String)>(
        r#"
        SELECT id, filename, filepath
        FROM files
        WHERE array_length(instrument_names_text, 1) IS NULL
           OR array_length(instrument_names_text, 1) = 0
        "#,
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|(id, filename, filepath)| FileRecord { id, filename, filepath })
    .collect();

    let total_files = files.len();
    println!("   Found {} files to process", total_files);
    println!();

    if total_files == 0 {
        println!("✅ All files already have instrument metadata!");
        return Ok(());
    }

    // Step 2: Process files in parallel using Rayon
    println!("⚡ Processing files in parallel (using all CPU cores)...");
    let patterns = Arc::new(InstrumentPatterns::new());
    let start_time = std::time::Instant::now();

    let results: Vec<(i64, String)> = files
        .par_iter()
        .filter_map(|file| {
            patterns
                .infer_instrument(&file.filename, &file.filepath)
                .map(|instrument| (file.id, instrument))
        })
        .collect();

    let inferred_count = results.len();
    let elapsed = start_time.elapsed();

    println!(
        "   ✅ Inferred instruments for {}/{} files ({:.1}%)",
        inferred_count,
        total_files,
        100.0 * inferred_count as f64 / total_files as f64
    );
    println!("   ⏱️  Processing time: {:.2}s", elapsed.as_secs_f64());
    println!(
        "   🚀 Speed: {:.0} files/sec",
        inferred_count as f64 / elapsed.as_secs_f64()
    );
    println!();

    // Step 3: Batch update database (in chunks of 10,000)
    println!("💾 Updating database in batches...");
    let batch_size = 10_000;
    let total_batches = (results.len() + batch_size - 1) / batch_size;
    let mut total_updated = 0u64;

    for (batch_idx, batch) in results.chunks(batch_size).enumerate() {
        print!(
            "   Batch {}/{} ({} files)... ",
            batch_idx + 1,
            total_batches,
            batch.len()
        );
        std::io::Write::flush(&mut std::io::stdout())?;

        let updated = batch_update_instruments(&pool, batch.to_vec()).await?;
        total_updated += updated;

        println!("✅ {} rows updated", updated);
    }

    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    RESULTS SUMMARY                             ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("   Total files processed:     {}", total_files);
    println!("   Instruments inferred:      {}", inferred_count);
    println!("   Database rows updated:     {}", total_updated);
    println!(
        "   Still missing:             {}",
        total_files - inferred_count
    );
    println!(
        "   Coverage improvement:      +{:.1}%",
        100.0 * inferred_count as f64 / total_files as f64
    );
    println!();

    // Step 4: Show final statistics
    println!("📊 Final instrument distribution:");
    println!();

    let stats: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT
            UNNEST(instrument_names_text) as instrument,
            COUNT(*) as count
        FROM files
        WHERE array_length(instrument_names_text, 1) > 0
        GROUP BY instrument
        ORDER BY count DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    for (instrument, count) in stats {
        println!("   {:12} {:>10} files", instrument, count);
    }

    println!();
    println!("✅ Instrument inference complete!");

    Ok(())
}
