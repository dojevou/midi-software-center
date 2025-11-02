//! Test data builders with fluent API for easy test setup

use sqlx::PgPool;

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
