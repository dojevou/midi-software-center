# MIDI Software Center - Architecture Reference

**Complete Technical Reference for Production-Ready Development**

Generated: November 30, 2025

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Database Schema](#database-schema)
3. [Connection Management](#connection-management)
4. [Error Handling](#error-handling)
5. [Data Models](#data-models)
6. [Repository Patterns](#repository-patterns)
7. [Performance Configuration](#performance-configuration)
8. [Query Patterns](#query-patterns)

---

## Project Overview

### Technology Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Backend | Rust | 1.70+ |
| Framework | Tauri | 2.7 |
| Database | PostgreSQL | 16+ |
| Async Runtime | tokio | 1.35 |
| SQL Toolkit | sqlx | 0.7 |
| MIDI Parsing | midly | 0.5 |
| Frontend | Svelte/TypeScript | 4.2/5.3 |

### Cargo Dependencies (shared/rust/Cargo.toml)

```toml
[package]
name = "midi-library-shared"
version = "0.1.0"
edition = "2021"

[dependencies]
# MIDI parsing
midly = "0.5"

# Database (optional feature)
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"], optional = true }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Async runtime (optional feature)
tokio = { version = "1.35", features = ["full"], optional = true }

# Time & UUID
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }

# Numeric types
rust_decimal = { version = "1.33", features = ["serde"] }
hex = "0.4"

# Logging
tracing = "0.1"

[features]
default = []
database = ["sqlx", "tokio"]
full = ["database"]
```

---

## Database Schema

### Core Tables (from 001_initial_schema.sql)

#### files Table
```sql
CREATE TABLE files (
    id BIGSERIAL PRIMARY KEY,

    -- File identification
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL UNIQUE,
    original_filename TEXT NOT NULL,
    content_hash BYTEA NOT NULL,
    file_size_bytes BIGINT NOT NULL,

    -- MIDI format
    format SMALLINT CHECK (format IN (0, 1, 2)),
    num_tracks SMALLINT NOT NULL DEFAULT 1,
    ticks_per_quarter_note INTEGER,

    -- Duration
    duration_seconds NUMERIC(10, 3),
    duration_ticks BIGINT,

    -- Multi-track handling
    is_multi_track BOOLEAN DEFAULT FALSE,
    parent_file_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
    track_number SMALLINT,
    total_tracks SMALLINT,

    -- Extracted context
    manufacturer TEXT,
    collection_name TEXT,
    folder_tags TEXT[],

    -- Full-text search
    search_vector tsvector,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    analyzed_at TIMESTAMPTZ,

    -- Processing
    import_batch_id UUID,

    -- Additional fields from migrations 002, 008, 009
    parent_folder TEXT,
    filename_bpm REAL,
    filename_key TEXT,
    filename_genres TEXT[],
    structure_tags TEXT[],
    metadata_source TEXT,
    track_names TEXT[],
    copyright TEXT,
    instrument_names_text TEXT[],
    markers TEXT[],
    lyrics TEXT[]
);
```

#### musical_metadata Table
```sql
CREATE TABLE musical_metadata (
    file_id BIGINT PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,

    -- Tempo
    bpm NUMERIC(6, 2) CHECK (bpm IS NULL OR (bpm >= 20 AND bpm <= 300)),
    bpm_confidence REAL,
    has_tempo_changes BOOLEAN DEFAULT FALSE,
    tempo_changes JSONB,

    -- Key signature (enum type)
    key_signature musical_key,
    key_confidence REAL,
    has_key_changes BOOLEAN DEFAULT FALSE,
    key_changes JSONB,

    -- Time signature
    time_signature_numerator SMALLINT DEFAULT 4,
    time_signature_denominator SMALLINT DEFAULT 4,
    has_time_signature_changes BOOLEAN DEFAULT FALSE,
    time_signature_changes JSONB,

    -- Note statistics
    total_notes INTEGER NOT NULL DEFAULT 0,
    unique_pitches INTEGER,
    pitch_range_min SMALLINT CHECK (pitch_range_min IS NULL OR (pitch_range_min >= 0 AND pitch_range_min <= 127)),
    pitch_range_max SMALLINT CHECK (pitch_range_max IS NULL OR (pitch_range_max >= 0 AND pitch_range_max <= 127)),
    avg_velocity NUMERIC(5, 2),

    -- Density metrics
    note_density NUMERIC(8, 3),
    polyphony_max SMALLINT,
    polyphony_avg NUMERIC(5, 2),

    -- Musical characteristics
    is_monophonic BOOLEAN DEFAULT FALSE,
    is_polyphonic BOOLEAN DEFAULT TRUE,
    is_percussive BOOLEAN DEFAULT FALSE,

    -- Chord analysis
    has_chords BOOLEAN DEFAULT FALSE,
    chord_complexity REAL,

    -- Melody analysis
    has_melody BOOLEAN DEFAULT FALSE,
    melodic_range SMALLINT,

    -- Harmonic analysis (migration 010)
    chord_progression JSONB,
    chord_types TEXT[],
    has_seventh_chords BOOLEAN,
    has_extended_chords BOOLEAN,
    chord_change_rate NUMERIC,
    chord_complexity_score NUMERIC,

    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### tags Table
```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    category TEXT,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

#### file_tags Table
```sql
CREATE TABLE file_tags (
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    added_by TEXT DEFAULT 'system',

    PRIMARY KEY (file_id, tag_id)
);
```

### Enum Types
```sql
CREATE TYPE file_category AS ENUM (
    'KICK', 'SNARE', 'HIHAT', 'CLAP', 'PERC', 'TOM', 'CYMBAL',
    'DRUM_LOOP', 'DRUM_PATTERN', 'BASS', 'SUB_BASS', 'BASS_LOOP',
    'CHORD', 'PROGRESSION', 'STAB', 'PAD', 'TEXTURE', 'ATMOSPHERE',
    'LEAD', 'MELODY', 'HOOK', 'RIFF', 'ARP', 'SEQUENCE',
    'PIANO', 'KEYS', 'ORGAN', 'STRING', 'BRASS', 'WOODWIND',
    'FX', 'RISER', 'IMPACT', 'SWEEP', 'TRANSITION',
    'VOCAL', 'VOX', 'SAMPLE', 'MOTIF', 'THEME', 'FULL_MIX', 'STEM', 'UNKNOWN'
);

CREATE TYPE musical_key AS ENUM (
    'C', 'Cm', 'C#', 'C#m', 'Db', 'Dbm',
    'D', 'Dm', 'D#', 'D#m', 'Eb', 'Ebm',
    'E', 'Em', 'F', 'Fm', 'F#', 'F#m',
    'Gb', 'Gbm', 'G', 'Gm', 'G#', 'G#m',
    'Ab', 'Abm', 'A', 'Am', 'A#', 'A#m',
    'Bb', 'Bbm', 'B', 'Bm', 'UNKNOWN'
);
```

### Key Indexes
```sql
-- Files indexes
CREATE UNIQUE INDEX idx_files_content_hash ON files(content_hash);
CREATE INDEX idx_files_filepath ON files(filepath);
CREATE INDEX idx_files_search ON files USING gin(search_vector);
CREATE INDEX idx_files_folder_tags ON files USING gin(folder_tags);

-- Metadata indexes
CREATE INDEX idx_metadata_bpm ON musical_metadata(bpm) WHERE bpm IS NOT NULL;
CREATE INDEX idx_metadata_key ON musical_metadata(key_signature) WHERE key_signature != 'UNKNOWN';

-- Tags indexes
CREATE INDEX idx_tags_name_trgm ON tags USING gin(name gin_trgm_ops);
CREATE INDEX idx_file_tags_tag ON file_tags(tag_id);
CREATE INDEX idx_file_tags_file ON file_tags(file_id);
```

---

## Connection Management

### Database Wrapper (pipeline/src-tauri/src/database/mod.rs)

```rust
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Database connection pool wrapper with performance optimizations
#[derive(Clone)]
pub struct Database {
    pool: Arc<RwLock<PgPool>>,
    database_url: String,
    reconnect_attempts: Arc<RwLock<u32>>,
}

impl Database {
    /// Create new database connection pool with optimized settings
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let mut connect_options = PgConnectOptions::from_str(database_url)?;

        // Enable prepared statement caching (100 statements)
        connect_options = connect_options.statement_cache_capacity(100);
        connect_options = connect_options.application_name("midi-library-pipeline");

        // Calculate optimal pool size (based on CPU cores and RAM)
        let (_, pool_size, _) = calculate_all_settings();
        let min_connections = (pool_size as f64 * 0.2).max(5.0) as u32;

        let pool = PgPoolOptions::new()
            .max_connections(pool_size as u32)      // Dynamic (20-200)
            .min_connections(min_connections)        // 20% of max, min 5
            .acquire_timeout(Duration::from_secs(10))
            .max_lifetime(Duration::from_secs(1800)) // 30 minutes
            .idle_timeout(Duration::from_secs(300))  // 5 minutes
            .test_before_acquire(true)               // Validate connection health
            .connect_with(connect_options)
            .await?;

        Ok(Self {
            pool: Arc::new(RwLock::new(pool)),
            database_url: database_url.to_string(),
            reconnect_attempts: Arc::new(RwLock::new(0)),
        })
    }

    /// Get cloned connection pool
    pub async fn pool(&self) -> PgPool {
        self.pool.read().await.clone()
    }

    /// Execute with automatic retry (for transient failures)
    pub async fn execute_with_retry<T, F, Fut>(
        &self,
        max_retries: u32,
        operation: F,
    ) -> Result<T, sqlx::Error>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, sqlx::Error>>,
    {
        let mut retries = 0;
        let mut delay = Duration::from_millis(100);

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries && is_transient_error(&e) => {
                    retries += 1;
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// Check if error is transient (retryable)
fn is_transient_error(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_)
    )
}
```

### Pool Statistics
```rust
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub size: u32,
    pub idle: usize,
    pub active: usize,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub pool_stats: PoolStats,
    pub connection_test: bool,
    pub response_time_ms: u64,
}
```

---

## Error Handling

### Shared DbError (shared/rust/src/db/models/error.rs)

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Duplicate record: {0}")]
    Duplicate(String),

    #[error("Foreign key violation: {0}")]
    ForeignKeyViolation(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Connection pool exhausted")]
    PoolExhausted,

    #[error("Database operation timed out: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Internal database error: {0}")]
    Internal(String),
}

pub type DbResult<T> = Result<T, DbError>;

// SQLx error conversion
#[cfg(feature = "database")]
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Row not found".to_string()),
            sqlx::Error::PoolTimedOut => Self::PoolExhausted,
            sqlx::Error::Database(db_err) => {
                let code = db_err.code().unwrap_or_default();
                match code.as_ref() {
                    "23505" => Self::Duplicate(db_err.message().to_string()),
                    "23503" => Self::ForeignKeyViolation(db_err.message().to_string()),
                    "23502" => Self::Validation(format!("Not null violation: {}", db_err.message())),
                    "23514" => Self::Validation(format!("Check violation: {}", db_err.message())),
                    "08000" | "08003" | "08006" => Self::Connection(db_err.message().to_string()),
                    _ => Self::Query(db_err.message().to_string()),
                }
            }
            _ => Self::Internal(err.to_string()),
        }
    }
}
```

### Pipeline AppError (pipeline/src-tauri/src/error.rs)

```rust
#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    ValidationError(String),
    IOError(std::io::Error),
    MidiError(String),
    Config(String),
    GeneralError(String),
}

pub type AppResult<T> = Result<T, AppError>;
pub type TauriResult<T> = Result<T, String>;

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}
```

---

## Data Models

### File Model (pipeline/src-tauri/src/db/models.rs)

```rust
use chrono::{DateTime, Utc};
use sqlx::types::BigDecimal;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct File {
    pub id: i64,
    pub filename: String,
    pub filepath: String,
    pub original_filename: String,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,
    pub format: Option<i16>,
    pub num_tracks: i16,
    pub ticks_per_quarter_note: Option<i32>,
    pub duration_seconds: Option<BigDecimal>,
    pub duration_ticks: Option<i64>,
    pub is_multi_track: Option<bool>,
    pub parent_file_id: Option<i64>,
    pub track_number: Option<i16>,
    pub total_tracks: Option<i16>,
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub folder_tags: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub analyzed_at: Option<DateTime<Utc>>,
    pub import_batch_id: Option<Uuid>,
    pub parent_folder: Option<String>,
    pub filename_bpm: Option<f32>,
    pub filename_key: Option<String>,
    pub filename_genres: Option<Vec<String>>,
    pub structure_tags: Option<Vec<String>>,
    pub metadata_source: Option<String>,
    pub track_names: Option<Vec<String>>,
    pub copyright: Option<String>,
    pub instrument_names_text: Option<Vec<String>>,
    pub markers: Option<Vec<String>>,
    pub lyrics: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct NewFile {
    pub filename: String,
    pub filepath: String,
    pub original_filename: String,
    pub content_hash: Vec<u8>,
    pub file_size_bytes: i64,
    pub format: Option<i16>,
    pub num_tracks: i16,
    pub ticks_per_quarter_note: Option<i32>,
    pub duration_seconds: Option<BigDecimal>,
    pub duration_ticks: Option<i64>,
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub folder_tags: Option<Vec<String>>,
    pub import_batch_id: Option<Uuid>,
    pub parent_folder: Option<String>,
    pub filename_bpm: Option<f32>,
    pub filename_key: Option<String>,
    pub filename_genres: Option<Vec<String>>,
    pub structure_tags: Option<Vec<String>>,
    pub metadata_source: Option<String>,
    pub track_names: Option<Vec<String>>,
    pub copyright: Option<String>,
    pub instrument_names_text: Option<Vec<String>>,
    pub markers: Option<Vec<String>>,
    pub lyrics: Option<Vec<String>>,
}
```

### MusicalMetadata Model

```rust
#[derive(Debug, Clone, FromRow)]
pub struct MusicalMetadata {
    pub file_id: i64,
    pub bpm: Option<BigDecimal>,
    pub bpm_confidence: Option<f32>,
    pub has_tempo_changes: Option<bool>,
    pub tempo_changes: Option<serde_json::Value>,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f32>,
    pub has_key_changes: Option<bool>,
    pub key_changes: Option<serde_json::Value>,
    pub time_signature_numerator: Option<i16>,
    pub time_signature_denominator: Option<i16>,
    pub has_time_signature_changes: Option<bool>,
    pub time_signature_changes: Option<serde_json::Value>,
    pub total_notes: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_min: Option<i16>,
    pub pitch_range_max: Option<i16>,
    pub avg_velocity: Option<BigDecimal>,
    pub note_density: Option<BigDecimal>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<BigDecimal>,
    pub is_monophonic: Option<bool>,
    pub is_polyphonic: Option<bool>,
    pub is_percussive: Option<bool>,
    pub has_chords: Option<bool>,
    pub chord_complexity: Option<f32>,
    pub has_melody: Option<bool>,
    pub melodic_range: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub chord_progression: Option<serde_json::Value>,
    pub chord_types: Option<Vec<String>>,
    pub has_seventh_chords: Option<bool>,
    pub has_extended_chords: Option<bool>,
    pub chord_change_rate: Option<BigDecimal>,
    pub chord_complexity_score: Option<BigDecimal>,
}
```

### Tag Models

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbTag {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TagWithCount {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub usage_count: i32,
}
```

---

## Repository Patterns

### FileRepository

```rust
pub struct FileRepository;

impl FileRepository {
    /// Insert a new file
    pub async fn insert(pool: &PgPool, new_file: NewFile) -> Result<i64, sqlx::Error> {
        let file_id = sqlx::query_scalar!(
            r#"
            INSERT INTO files (filename, filepath, original_filename, content_hash, ...)
            VALUES ($1, $2, $3, $4, ...)
            RETURNING id
            "#,
            new_file.filename,
            new_file.filepath,
            // ... other fields
        )
        .fetch_one(pool)
        .await?;

        Ok(file_id)
    }

    /// Find by ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<File>, sqlx::Error> {
        sqlx::query_as!(File, "SELECT ... FROM files WHERE id = $1", id)
            .fetch_optional(pool)
            .await
    }

    /// Check duplicate by hash
    pub async fn check_duplicate(pool: &PgPool, content_hash: &[u8]) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM files WHERE content_hash = $1"#,
            content_hash
        )
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// Paginated list
    pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<File>, sqlx::Error> {
        sqlx::query_as!(File, "SELECT ... FROM files ORDER BY created_at DESC LIMIT $1 OFFSET $2", limit, offset)
            .fetch_all(pool)
            .await
    }
}
```

### MetadataRepository

```rust
pub struct MetadataRepository;

impl MetadataRepository {
    /// Insert with upsert
    pub async fn insert(pool: &PgPool, metadata: NewMusicalMetadata) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO musical_metadata (file_id, bpm, key_signature, ...)
            VALUES ($1, $2, $3::text::musical_key, ...)
            ON CONFLICT (file_id) DO UPDATE SET
                bpm = EXCLUDED.bpm,
                key_signature = EXCLUDED.key_signature,
                ...
            "#,
            metadata.file_id,
            metadata.bpm,
            metadata.key_signature,
            // ...
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Find by file ID
    pub async fn find_by_file_id(pool: &PgPool, file_id: i64) -> Result<Option<MusicalMetadata>, sqlx::Error> {
        sqlx::query_as!(
            MusicalMetadata,
            r#"SELECT ..., key_signature::text as key_signature FROM musical_metadata WHERE file_id = $1"#,
            file_id
        )
        .fetch_optional(pool)
        .await
    }
}
```

### TagRepository

```rust
pub struct TagRepository {
    pool: PgPool,
}

impl TagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get or create tag
    pub async fn get_or_create_tag(&self, name: &str, category: Option<&str>) -> Result<i32> {
        sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO tags (name, category, usage_count, created_at)
            VALUES ($1, $2, 0, NOW())
            ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(name)
        .bind(category)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    /// Batch add tags to file
    pub async fn add_tags_to_file(&self, file_id: i64, tag_ids: &[i32]) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO file_tags (file_id, tag_id, added_at, added_by)
            SELECT $1, unnest($2::int[]), NOW(), 'system'
            ON CONFLICT (file_id, tag_id) DO NOTHING
            "#,
        )
        .bind(file_id)
        .bind(tag_ids)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

### SearchRepository

```rust
pub struct SearchRepository;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub min_bpm: Option<f64>,
    pub max_bpm: Option<f64>,
    pub key: Option<String>,
    pub manufacturer: Option<String>,
    pub collection: Option<String>,
}

impl SearchRepository {
    /// Full-text search with filters
    pub async fn search(
        pool: &PgPool,
        query: SearchQuery,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<File>, sqlx::Error> {
        sqlx::query_as!(
            File,
            r#"
            SELECT f.* FROM files f
            LEFT JOIN musical_metadata mm ON f.id = mm.file_id
            WHERE
                ($1::text IS NULL OR f.search_vector @@ plainto_tsquery('english', $1))
                AND ($2::float8 IS NULL OR mm.bpm::float8 >= $2)
                AND ($3::float8 IS NULL OR mm.bpm::float8 <= $3)
                AND ($4::text IS NULL OR mm.key_signature::text = $4)
            ORDER BY ts_rank(f.search_vector, plainto_tsquery('english', $1)) DESC
            LIMIT $5 OFFSET $6
            "#,
            query.text,
            query.min_bpm,
            query.max_bpm,
            query.key,
            limit,
            offset
        )
        .fetch_all(pool)
        .await
    }
}
```

---

## Performance Configuration

### Default Connection Pool Settings

| Setting | Value | Notes |
|---------|-------|-------|
| max_connections | Dynamic (20-200) | Based on CPU cores |
| min_connections | 20% of max, min 5 | Warm pool |
| acquire_timeout | 10 seconds | Fast failure |
| max_lifetime | 30 minutes | Connection recycling |
| idle_timeout | 5 minutes | Resource cleanup |
| statement_cache | 100 | Prepared statement cache |
| test_before_acquire | true | Health validation |

### Batch Operations

| Operation | Batch Size | Notes |
|-----------|------------|-------|
| File Import | 1,000 | Transaction batching |
| Analysis | 200 | CPU-intensive |
| Tag Updates | 100 | Bulk insert with unnest |
| Search Results | 50-100 | Pagination default |

### Query Timeouts

| Query Type | Timeout |
|------------|---------|
| Simple lookup | 5s |
| Full-text search | 10s |
| Batch import | 30s |
| Health check | 1s |

---

## Query Patterns

### Type-Safe Queries with sqlx::query_as!

```rust
// With explicit type annotations for nullable fields
let file = sqlx::query_as!(
    File,
    r#"
    SELECT
        id,
        filename,
        created_at as "created_at!",  -- Not null
        analyzed_at,                   -- Nullable
        key_signature::text            -- Enum to text
    FROM files WHERE id = $1
    "#,
    id
)
.fetch_optional(pool)
.await?;
```

### Enum Type Handling

```rust
// Insert with enum cast
sqlx::query!(
    "INSERT INTO musical_metadata (key_signature) VALUES ($1::text::musical_key)",
    key_string
)

// Select with text cast
sqlx::query_as!(
    MusicalMetadata,
    "SELECT key_signature::text as key_signature FROM musical_metadata"
)
```

### Array Parameters

```rust
// Using unnest for batch operations
sqlx::query(
    r#"
    INSERT INTO file_tags (file_id, tag_id)
    SELECT $1, unnest($2::int[])
    ON CONFLICT DO NOTHING
    "#,
)
.bind(file_id)
.bind(&tag_ids[..])
.execute(pool)
.await?;
```

### JSONB Fields

```rust
// Insert JSONB
sqlx::query!(
    "INSERT INTO musical_metadata (tempo_changes) VALUES ($1)",
    serde_json::json!([{"tick": 0, "bpm": 120}])
)

// Query JSONB
sqlx::query_as!(
    MusicalMetadata,
    "SELECT tempo_changes FROM musical_metadata WHERE file_id = $1",
    file_id
)
```

---

## Database Connection String

```
postgresql://midiuser:145278963@localhost:5433/midi_library
```

| Component | Value |
|-----------|-------|
| User | midiuser |
| Password | 145278963 |
| Host | localhost |
| Port | 5433 |
| Database | midi_library |

---

## AppState Structure

```rust
/// Application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
}

// Usage in Tauri commands
#[tauri::command]
pub async fn get_file(
    state: State<'_, AppState>,
    id: i64,
) -> TauriResult<File> {
    let pool = state.database.pool().await;
    FileRepository::find_by_id(&pool, id)
        .await
        .map_err(|e| AppError::from(e).into())?
        .ok_or_else(|| AppError::not_found(&format!("File {}", id)).into())
}
```

---

## Summary: Key Patterns

1. **Connection**: Use `PgPool` with dynamic sizing
2. **Error Handling**: `DbError` (shared) → `AppError` (pipeline) → `String` (Tauri)
3. **Queries**: Use `sqlx::query_as!` macro for type safety
4. **Transactions**: Use `pool.begin().await?` for atomic operations
5. **Batch Operations**: Use `unnest` for array inserts
6. **Enum Types**: Cast `::text::enum_type` in SQL
7. **JSONB**: Use `serde_json::Value` in Rust models
8. **Performance**: Retry transient errors with exponential backoff
