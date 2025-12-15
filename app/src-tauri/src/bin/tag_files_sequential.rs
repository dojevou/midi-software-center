/// Sequential File Tagger - Process one keyword at a time with visible progress
///
/// Advantages over single-query approach:
/// 1. Visible progress (shows each keyword as it completes)
/// 2. Can resume if interrupted
/// 3. More predictable performance
/// 4. Lower memory usage
use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use sqlx::postgres::PgPoolOptions;

#[derive(Parser, Debug)]
#[command(name = "tag_files_sequential")]
#[command(about = "Tag files with instruments sequentially (one keyword at a time)")]
struct Args {
    /// Database connection string
    #[arg(short = 'D', long, env = "DATABASE_URL")]
    database_url: String,

    /// Skip first N keywords (for resuming)
    #[arg(short = 's', long, default_value_t = 0)]
    skip: usize,
}

// All 97 instrument keywords
const KEYWORDS: &[(&str, &str)] = &[
    // Drums (23)
    ("ride", "drums"),
    ("fill", "drums"),
    ("kick", "drums"),
    ("tom", "drums"),
    ("crash", "drums"),
    ("snare", "drums"),
    ("stick", "drums"),
    ("hihat", "drums"),
    ("drums", "drums"),
    ("toms", "drums"),
    ("clap", "drums"),
    ("china", "drums"),
    ("conga", "drums"),
    ("cymbal", "drums"),
    ("rim", "drums"),
    ("cowbell", "drums"),
    ("bongo", "drums"),
    ("percussion", "drums"),
    ("shaker", "drums"),
    ("tambourine", "drums"),
    ("splash", "drums"),
    ("hi-hat", "drums"),
    ("drum", "drums"),
    // Bass (5)
    ("bass", "bass"),
    ("bassline", "bass"),
    ("sub", "bass"),
    ("808", "bass"),
    ("909", "bass"),
    // Synths & Keys (14)
    ("synth", "synth"),
    ("piano", "keys"),
    ("lead", "synth"),
    ("pad", "synth"),
    ("keys", "keys"),
    ("arp", "synth"),
    ("pluck", "synth"),
    ("organ", "keys"),
    ("brass", "brass"),
    ("rhodes", "keys"),
    ("wurlitzer", "keys"),
    ("clav", "keys"),
    ("electric-piano", "keys"),
    ("harpsichord", "keys"),
    // Guitars (6)
    ("guitar", "guitar"),
    ("acoustic", "guitar"),
    ("electric", "guitar"),
    ("12-string", "guitar"),
    ("slide", "guitar"),
    ("muted", "guitar"),
    // Strings (6)
    ("strings", "strings"),
    ("violin", "strings"),
    ("cello", "strings"),
    ("viola", "strings"),
    ("ensemble", "strings"),
    ("orchestra", "orchestral"),
    // Brass & Woodwinds (9)
    ("trumpet", "brass"),
    ("sax", "brass"),
    ("trombone", "brass"),
    ("horn", "brass"),
    ("flute", "woodwind"),
    ("clarinet", "woodwind"),
    ("oboe", "woodwind"),
    ("bassoon", "woodwind"),
    // Vocals (5)
    ("vocal", "vocal"),
    ("vox", "vocal"),
    ("choir", "vocal"),
    ("voice", "vocal"),
    ("chant", "vocal"),
    // FX (7)
    ("fx", "fx"),
    ("bell", "fx"),
    ("hit", "fx"),
    ("sfx", "fx"),
    ("sweep", "fx"),
    ("riser", "fx"),
    ("impact", "fx"),
    // Musical Elements (9)
    ("loop", "pattern"),
    ("melody", "melody"),
    ("chord", "harmony"),
    ("groove", "pattern"),
    ("break", "pattern"),
    ("progression", "harmony"),
    ("pattern", "pattern"),
    ("harmonic", "harmony"),
    ("melodic", "melody"),
    // Genres (14)
    ("rock", "genre"),
    ("funk", "genre"),
    ("jazz", "genre"),
    ("dnb", "genre"),
    ("house", "genre"),
    ("trance", "genre"),
    ("techno", "genre"),
    ("edm", "genre"),
    ("soul", "genre"),
    ("trap", "genre"),
    ("reggae", "genre"),
    ("dubstep", "genre"),
    ("hip-hop", "genre"),
    ("r&b", "genre"),
];

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Connect to database
    let pool = PgPoolOptions::new().max_connections(5).connect(&args.database_url).await?;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Sequential File Tagging (97 keywords)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Create progress bar
    let total_keywords = KEYWORDS.len() - args.skip;
    let progress = ProgressBar::new(total_keywords as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) - {msg} [{elapsed_precise}]")?
            .progress_chars("█▓▒░ "),
    );

    let mut total_tagged = 0u64;

    // Process each keyword sequentially
    for (_idx, &(keyword, category)) in KEYWORDS.iter().enumerate().skip(args.skip) {
        progress.set_message(format!("Tagging: {} ({})", keyword, category));

        // Get or create tag
        let tag_result = sqlx::query!(
            r#"
            INSERT INTO tags (name, category)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET category = EXCLUDED.category
            RETURNING id
            "#,
            keyword,
            category
        )
        .fetch_one(&pool)
        .await?;

        let tag_id = tag_result.id as i64;

        // Tag files matching this keyword (filename OR filepath)
        let rows_affected = sqlx::query!(
            r#"
            INSERT INTO file_tags (file_id, tag_id, added_by)
            SELECT DISTINCT
                f.id,
                $1::bigint,
                'sequential_tagger'
            FROM files f
            WHERE (
                LOWER(f.filename) LIKE '%' || LOWER($2) || '%'
                OR LOWER(f.filepath) LIKE '%' || LOWER($2) || '%'
            )
            ON CONFLICT (file_id, tag_id) DO NOTHING
            "#,
            tag_id,
            keyword
        )
        .execute(&pool)
        .await?
        .rows_affected();

        total_tagged += rows_affected;

        progress.set_message(format!(
            "✓ {} ({}) - {} files",
            keyword, category, rows_affected
        ));
        progress.inc(1);

        // Brief pause to avoid overwhelming DB
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    progress.finish_with_message("Complete!");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ TAGGING COMPLETE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Keywords processed:  {}", total_keywords);
    println!("New tags created:    {}", total_tagged);

    // Summary stats
    let stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total_relationships,
            COUNT(DISTINCT file_id) as unique_files
        FROM file_tags
        "#
    )
    .fetch_one(&pool)
    .await?;

    println!(
        "Total relationships: {}",
        stats.total_relationships.unwrap_or(0)
    );
    println!("Unique files tagged: {}", stats.unique_files.unwrap_or(0));

    Ok(())
}
