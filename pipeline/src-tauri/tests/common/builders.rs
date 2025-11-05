//! Test data builders with fluent API for easy test setup

use sqlx::PgPool;

//=============================================================================
// HELPER FUNCTIONS (exported for test use)
//=============================================================================

/// Create a test file with default parameters
pub async fn create_test_file(pool: &PgPool, filename: &str) -> i64 {
    MidiFileBuilder::new()
        .with_path(&format!("/test/path/{}", filename))
        .insert(pool)
        .await
}

/// Insert metadata for a test file
pub async fn insert_metadata(
    pool: &PgPool,
    file_id: i64,
    bpm: Option<f64>,
    key: Option<&str>,
    duration: Option<i32>,
) -> i64 {
    let mut builder = MetadataBuilder::new(file_id);

    if let Some(bpm_val) = bpm {
        builder = builder.with_bpm(bpm_val);
    }

    if let Some(key_val) = key {
        builder = builder.with_key(key_val);
    }

    builder.insert(pool).await
}

/// Create a test file with metadata in one call
pub async fn create_test_file_with_metadata(
    pool: &PgPool,
    filename: &str,
    bpm: Option<f64>,
    key: Option<&str>,
) -> (i64, i64) {
    let file_id = create_test_file(pool, filename).await;
    let metadata_id = insert_metadata(pool, file_id, bpm, key, None).await;
    (file_id, metadata_id)
}

/// Create multiple test files
pub async fn create_test_files(pool: &PgPool, count: usize) -> Vec<i64> {
    let mut ids = Vec::new();
    for i in 0..count {
        let file_id = create_test_file(pool, &format!("test_file_{}.mid", i)).await;
        ids.push(file_id);
    }
    ids
}

/// Setup test app state with database connection
pub async fn setup_test_state() -> midi_pipeline::AppState {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Verify connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .expect("Failed to verify database connection");

    midi_pipeline::AppState {
        db_pool: Some(pool),
        // Add other AppState fields as needed with defaults
    }
}

/// Import and analyze a file in one operation
pub async fn import_and_analyze_file(
    _state: &midi_pipeline::AppState,
    _file_path: String,
) -> Result<(), String> {
    // Placeholder implementation - replace with your actual import/analyze logic
    Ok(())
}

/// Builder for test MIDI file metadata
pub struct MidiFileBuilder {
    file_path: String,
    blake3_hash: String,
    file_size_bytes: i64,
    manufacturer: Option<String>,
    collection_name: Option<String>,
}

impl MidiFileBuilder {
    pub fn new() -> Self {
        Self {
            file_path: "/test/default.mid".to_string(),
            blake3_hash: format!("{:064x}", 0),
            file_size_bytes: 1024,
            manufacturer: None,
            collection_name: None,
        }
    }
    
    pub fn with_path(mut self, path: &str) -> Self {
        self.file_path = path.to_string();
        self
    }
    
    pub fn with_hash(mut self, hash: &str) -> Self {
        self.blake3_hash = hash.to_string();
        self
    }
    
    pub fn with_size(mut self, size: i64) -> Self {
        self.file_size_bytes = size;
        self
    }
    
    pub fn with_manufacturer(mut self, manufacturer: &str) -> Self {
        self.manufacturer = Some(manufacturer.to_string());
        self
    }
    
    pub fn with_collection(mut self, collection: &str) -> Self {
        self.collection_name = Some(collection.to_string());
        self
    }
    
    pub async fn insert(self, pool: &PgPool) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO files (file_path, blake3_hash, file_size_bytes, manufacturer, collection_name) 
             VALUES ($1, $2, $3, $4, $5) RETURNING file_id"
        )
        .bind(self.file_path)
        .bind(self.blake3_hash)
        .bind(self.file_size_bytes)
        .bind(self.manufacturer)
        .bind(self.collection_name)
        .fetch_one(pool)
        .await
        .expect("Failed to insert test file")
    }
}

impl Default for MidiFileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for test musical metadata
pub struct MetadataBuilder {
    file_id: i64,
    bpm: Option<f64>,
    detected_key: Option<String>,
    time_signature: Option<String>,
}

impl MetadataBuilder {
    pub fn new(file_id: i64) -> Self {
        Self {
            file_id,
            bpm: None,
            detected_key: None,
            time_signature: None,
        }
    }
    
    pub fn with_bpm(mut self, bpm: f64) -> Self {
        self.bpm = Some(bpm);
        self
    }
    
    pub fn with_key(mut self, key: &str) -> Self {
        self.detected_key = Some(key.to_string());
        self
    }
    
    pub fn with_time_signature(mut self, time_sig: &str) -> Self {
        self.time_signature = Some(time_sig.to_string());
        self
    }
    
    pub async fn insert(self, pool: &PgPool) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO musical_metadata (file_id, bpm, detected_key, time_signature) 
             VALUES ($1, $2, $3, $4) RETURNING metadata_id"
        )
        .bind(self.file_id)
        .bind(self.bpm)
        .bind(self.detected_key)
        .bind(self.time_signature)
        .fetch_one(pool)
        .await
        .expect("Failed to insert test metadata")
    }
}

/// Builder for test tags
pub struct TagBuilder {
    tag_name: String,
    tag_category: Option<String>,
}

impl TagBuilder {
    pub fn new(tag_name: &str) -> Self {
        Self {
            tag_name: tag_name.to_string(),
            tag_category: None,
        }
    }
    
    pub fn with_category(mut self, category: &str) -> Self {
        self.tag_category = Some(category.to_string());
        self
    }
    
    pub async fn insert(self, pool: &PgPool) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO tags (tag_name, tag_category) VALUES ($1, $2) 
             ON CONFLICT (tag_name) DO UPDATE SET tag_name = EXCLUDED.tag_name
             RETURNING tag_id"
        )
        .bind(self.tag_name)
        .bind(self.tag_category)
        .fetch_one(pool)
        .await
        .expect("Failed to insert test tag")
    }
}
