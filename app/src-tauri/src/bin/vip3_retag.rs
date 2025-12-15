//! VIP3 Bulk Retagger CLI
//!
//! High-performance batch tagging for MIDI files using Aho-Corasick pattern matching
//! and Rayon parallel processing.

use clap::Parser;
use midi_app::core::analysis::Vip3Extractor;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// VIP3 Bulk Retagger - Tag all files with timbres, styles, and articulations
#[derive(Parser, Debug)]
#[command(name = "vip3_retag")]
#[command(about = "Bulk retag MIDI files with VIP3 categories", long_about = None)]
struct Args {
    /// Batch size for processing files
    #[arg(short, long, default_value = "10000")]
    batch_size: i64,

    /// Database URL
    #[arg(
        short,
        long,
        default_value = "postgresql://midiuser:145278963@localhost:5433/midi_library"
    )]
    database_url: String,

    /// Dry run - don't write to database
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Update counts after retagging
    #[arg(long, default_value = "true")]
    update_counts: bool,
}

/// File record for processing
#[derive(Debug, Clone)]
struct FileRecord {
    id: i64,
    filename: String,
    filepath: String,
}

/// Category ID lookup maps
struct CategoryMaps {
    timbres: HashMap<String, i16>,
    styles: HashMap<String, i16>,
    articulations: HashMap<String, i16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let start = Instant::now();

    println!("🎵 VIP3 Bulk Retagger");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Database: {}", args.database_url);
    println!("Batch size: {}", args.batch_size);
    println!("Dry run: {}", args.dry_run);
    println!();

    // Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&args.database_url)
        .await?;

    // Load category maps
    println!("📂 Loading category maps...");
    let category_maps = load_category_maps(&pool).await?;
    println!(
        "   Loaded: {} timbres, {} styles, {} articulations",
        category_maps.timbres.len(),
        category_maps.styles.len(),
        category_maps.articulations.len()
    );

    // Create extractor
    println!("🔧 Building Aho-Corasick automata...");
    let extractor = Arc::new(Vip3Extractor::new());

    // Get total file count
    let total_files: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
        .fetch_one(&pool)
        .await?;
    let total_files = total_files.0;
    println!("📊 Total files to process: {}", total_files);
    println!();

    let mut files_processed = 0usize;
    let mut timbres_added = 0usize;
    let mut styles_added = 0usize;
    let mut articulations_added = 0usize;
    let mut offset = 0i64;

    while offset < total_files {
        // Fetch batch
        let files = fetch_file_batch(&pool, offset, args.batch_size).await?;
        if files.is_empty() {
            break;
        }

        let batch_len = files.len();
        let batch_start = Instant::now();

        // Convert to format expected by extractor
        let file_tuples: Vec<(i64, String, String)> = files
            .iter()
            .map(|f| (f.id, f.filename.clone(), f.filepath.clone()))
            .collect();

        // Extract categories in parallel
        let results = extractor.extract_batch(&file_tuples);

        // Collect assignments
        let mut timbre_assignments: Vec<(i64, i16)> = Vec::new();
        let mut style_assignments: Vec<(i64, i16)> = Vec::new();
        let mut articulation_assignments: Vec<(i64, i16)> = Vec::new();

        for (file_id, categories) in &results {
            for timbre in &categories.timbres {
                if let Some(&timbre_id) = category_maps.timbres.get(timbre) {
                    timbre_assignments.push((*file_id, timbre_id));
                }
            }
            for style in &categories.styles {
                if let Some(&style_id) = category_maps.styles.get(style) {
                    style_assignments.push((*file_id, style_id));
                }
            }
            for articulation in &categories.articulations {
                if let Some(&articulation_id) = category_maps.articulations.get(articulation) {
                    articulation_assignments.push((*file_id, articulation_id));
                }
            }
        }

        // Insert if not dry run
        if !args.dry_run {
            let t = insert_timbres(&pool, &timbre_assignments).await?;
            let s = insert_styles(&pool, &style_assignments).await?;
            let a = insert_articulations(&pool, &articulation_assignments).await?;

            timbres_added += t as usize;
            styles_added += s as usize;
            articulations_added += a as usize;
        } else {
            timbres_added += timbre_assignments.len();
            styles_added += style_assignments.len();
            articulations_added += articulation_assignments.len();
        }

        files_processed += batch_len;
        offset += args.batch_size;

        // Progress update
        let batch_elapsed = batch_start.elapsed().as_secs_f64();
        let batch_rate = batch_len as f64 / batch_elapsed;
        let overall_elapsed = start.elapsed().as_secs_f64();
        let overall_rate = files_processed as f64 / overall_elapsed;
        let progress = (files_processed as f64 / total_files as f64) * 100.0;

        print!(
            "\r⏳ Progress: {}/{} ({:.1}%) | Batch: {:.0}/s | Overall: {:.0}/s | Tags: T:{} S:{} A:{}",
            files_processed, total_files, progress, batch_rate, overall_rate,
            timbres_added, styles_added, articulations_added
        );
        std::io::Write::flush(&mut std::io::stdout())?;
    }

    println!();
    println!();

    // Update counts if requested
    if args.update_counts && !args.dry_run {
        println!("📈 Updating category counts...");
        update_counts(&pool).await?;
        println!("   Done!");
    }

    let elapsed = start.elapsed();
    let elapsed_seconds = elapsed.as_secs_f64();
    let files_per_second = files_processed as f64 / elapsed_seconds;

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Complete!");
    println!("   Files processed: {}", files_processed);
    println!("   Timbres added:   {}", timbres_added);
    println!("   Styles added:    {}", styles_added);
    println!("   Articulations:   {}", articulations_added);
    println!("   Total tags:      {}", timbres_added + styles_added + articulations_added);
    println!("   Time elapsed:    {:.2}s", elapsed_seconds);
    println!("   Speed:           {:.0} files/sec", files_per_second);

    if args.dry_run {
        println!();
        println!("⚠️  DRY RUN - No changes written to database");
    }

    Ok(())
}

async fn load_category_maps(
    pool: &sqlx::PgPool,
) -> Result<CategoryMaps, sqlx::Error> {
    let timbre_rows: Vec<(i16, String)> = sqlx::query_as("SELECT id, name FROM timbres")
        .fetch_all(pool)
        .await?;
    let timbres: HashMap<String, i16> = timbre_rows
        .into_iter()
        .map(|(id, name)| (name, id))
        .collect();

    let style_rows: Vec<(i16, String)> = sqlx::query_as("SELECT id, name FROM styles")
        .fetch_all(pool)
        .await?;
    let styles: HashMap<String, i16> = style_rows
        .into_iter()
        .map(|(id, name)| (name, id))
        .collect();

    let articulation_rows: Vec<(i16, String)> =
        sqlx::query_as("SELECT id, name FROM articulations")
            .fetch_all(pool)
            .await?;
    let articulations: HashMap<String, i16> = articulation_rows
        .into_iter()
        .map(|(id, name)| (name, id))
        .collect();

    Ok(CategoryMaps {
        timbres,
        styles,
        articulations,
    })
}

async fn fetch_file_batch(
    pool: &sqlx::PgPool,
    offset: i64,
    limit: i64,
) -> Result<Vec<FileRecord>, sqlx::Error> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        r#"
        SELECT id, filename, filepath
        FROM files
        ORDER BY id
        OFFSET $1 LIMIT $2
        "#,
    )
    .bind(offset)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, filename, filepath)| FileRecord {
            id,
            filename,
            filepath,
        })
        .collect())
}

async fn insert_timbres(
    pool: &sqlx::PgPool,
    assignments: &[(i64, i16)],
) -> Result<u64, sqlx::Error> {
    if assignments.is_empty() {
        return Ok(0);
    }

    let file_ids: Vec<i64> = assignments.iter().map(|(f, _)| *f).collect();
    let timbre_ids: Vec<i16> = assignments.iter().map(|(_, t)| *t).collect();

    let result = sqlx::query(
        r#"
        INSERT INTO midi_file_timbres (file_id, timbre_id)
        SELECT * FROM UNNEST($1::bigint[], $2::smallint[])
        ON CONFLICT (file_id, timbre_id) DO NOTHING
        "#,
    )
    .bind(&file_ids)
    .bind(&timbre_ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

async fn insert_styles(
    pool: &sqlx::PgPool,
    assignments: &[(i64, i16)],
) -> Result<u64, sqlx::Error> {
    if assignments.is_empty() {
        return Ok(0);
    }

    let file_ids: Vec<i64> = assignments.iter().map(|(f, _)| *f).collect();
    let style_ids: Vec<i16> = assignments.iter().map(|(_, s)| *s).collect();

    let result = sqlx::query(
        r#"
        INSERT INTO midi_file_styles (file_id, style_id)
        SELECT * FROM UNNEST($1::bigint[], $2::smallint[])
        ON CONFLICT (file_id, style_id) DO NOTHING
        "#,
    )
    .bind(&file_ids)
    .bind(&style_ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

async fn insert_articulations(
    pool: &sqlx::PgPool,
    assignments: &[(i64, i16)],
) -> Result<u64, sqlx::Error> {
    if assignments.is_empty() {
        return Ok(0);
    }

    let file_ids: Vec<i64> = assignments.iter().map(|(f, _)| *f).collect();
    let articulation_ids: Vec<i16> = assignments.iter().map(|(_, a)| *a).collect();

    let result = sqlx::query(
        r#"
        INSERT INTO midi_file_articulations (file_id, articulation_id)
        SELECT * FROM UNNEST($1::bigint[], $2::smallint[])
        ON CONFLICT (file_id, articulation_id) DO NOTHING
        "#,
    )
    .bind(&file_ids)
    .bind(&articulation_ids)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

async fn update_counts(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE timbres t
        SET file_count = (
            SELECT COUNT(*) FROM midi_file_timbres mft WHERE mft.timbre_id = t.id
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE styles s
        SET file_count = (
            SELECT COUNT(*) FROM midi_file_styles mfs WHERE mfs.style_id = s.id
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE articulations a
        SET file_count = (
            SELECT COUNT(*) FROM midi_file_articulations mfa WHERE mfa.articulation_id = a.id
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
