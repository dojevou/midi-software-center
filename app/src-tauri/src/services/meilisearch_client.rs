//! Meilisearch integration for full-text search across MIDI files.
//!
//! Provides high-performance, typo-tolerant search with faceting support for:
//! - Filename search
//! - Tag/instrument filtering
//! - Musical metadata (BPM, key, time signature)
//! - Collection and manufacturer filtering

use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::settings::Settings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Document structure for Meilisearch indexing.
///
/// Represents a searchable MIDI file with all relevant metadata.
/// This structure is flattened for optimal search performance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiSearchDocument {
    /// Primary key (file_id from database)
    pub id: i64,

    /// Sanitized filename (primary search field)
    pub filename: String,

    /// Original filename before sanitization
    pub original_filename: String,

    /// Full file path
    pub filepath: String,

    /// Tags as array (for faceted filtering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Instruments as array (for faceted filtering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruments: Option<Vec<String>>,

    /// BPM (for numeric filtering and sorting)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bpm: Option<f64>,

    /// Musical key (for faceted filtering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_signature: Option<String>,

    /// Time signature as string (e.g., "4/4")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_signature: Option<String>,

    /// Manufacturer/source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,

    /// Collection or pack name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,

    /// Duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,

    /// Number of tracks
    pub num_tracks: i16,

    /// Whether this is a multi-track file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_multi_track: Option<bool>,

    /// Whether file contains percussion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_percussive: Option<bool>,

    /// Timbres (VIP3 sound character)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timbres: Option<Vec<String>>,

    /// Styles (VIP3 musical genres/styles)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<Vec<String>>,

    /// Articulations (VIP3 playing styles)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub articulations: Option<Vec<String>>,
}

/// Search result from Meilisearch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: MidiSearchDocument,
    pub score: Option<f64>,
}

/// Faceted search filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub instruments: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub bpm_min: Option<f64>,
    pub bpm_max: Option<f64>,
    pub key_signature: Option<String>,
    pub manufacturer: Option<String>,
    pub collection_name: Option<String>,
    pub is_percussive: Option<bool>,
    pub timbres: Option<Vec<String>>,
    pub styles: Option<Vec<String>>,
    pub articulations: Option<Vec<String>>,
}

/// Meilisearch client wrapper for MIDI file indexing and search.
pub struct MeilisearchClient {
    client: Client,
    index_name: String,
}

impl MeilisearchClient {
    /// Create a new Meilisearch client.
    ///
    /// # Arguments
    /// * `url` - Meilisearch server URL (default: http://localhost:7700)
    /// * `api_key` - Optional API key for authentication
    /// * `index_name` - Name of the index (default: "midi_files")
    pub fn new(url: &str, api_key: Option<&str>, index_name: Option<&str>) -> Result<Self, String> {
        let client = if let Some(key) = api_key {
            Client::new(url, Some(key))
                .map_err(|e| format!("Failed to create Meilisearch client: {}", e))?
        } else {
            Client::new(url, None::<&str>)
                .map_err(|e| format!("Failed to create Meilisearch client: {}", e))?
        };

        let index_name = index_name.unwrap_or("midi_files").to_string();

        Ok(Self { client, index_name })
    }

    /// Initialize the index with optimal settings for MIDI file search.
    ///
    /// Sets up:
    /// - Searchable attributes (filename, tags, instruments)
    /// - Filterable attributes (bpm, key, instruments, etc.)
    /// - Sortable attributes (bpm, duration)
    /// - Ranking rules optimized for relevance
    pub async fn initialize_index(&self) -> Result<(), String> {
        info!("Initializing Meilisearch index: {}", self.index_name);

        let index = self.get_index().await?;

        // Configure searchable attributes (what to search in)
        let searchable_attributes = vec![
            "filename",
            "original_filename",
            "tags",
            "instruments",
            "manufacturer",
            "collection_name",
            "timbres",
            "styles",
            "articulations",
        ];

        // Configure filterable attributes (what can be used in filters)
        let filterable_attributes = vec![
            "tags",
            "instruments",
            "bpm",
            "key_signature",
            "manufacturer",
            "collection_name",
            "is_percussive",
            "is_multi_track",
            "time_signature",
            "num_tracks",
            "timbres",
            "styles",
            "articulations",
        ];

        // Configure sortable attributes
        let sortable_attributes = ["bpm", "duration_seconds", "filename", "num_tracks"];

        // Configure ranking rules (order matters!)
        let ranking_rules = [
            "words",
            "typo",
            "proximity",
            "attribute",
            "sort",
            "exactness",
        ];

        // Create settings object
        let mut settings = Settings::new();
        settings.searchable_attributes = Some(searchable_attributes.iter().map(|s| s.to_string()).collect());
        settings.filterable_attributes = Some(filterable_attributes.iter().map(|s| s.to_string()).collect());
        settings.sortable_attributes = Some(sortable_attributes.iter().map(|s| s.to_string()).collect());
        settings.ranking_rules = Some(ranking_rules.iter().map(|s| s.to_string()).collect());

        // Apply settings
        index
            .set_settings(&settings)
            .await
            .map_err(|e| format!("Failed to configure index settings: {}", e))?;

        info!("Meilisearch index initialized successfully");
        Ok(())
    }

    /// Get or create the index.
    async fn get_index(&self) -> Result<Index, String> {
        // Try to get existing index
        match self.client.get_index(&self.index_name).await {
            Ok(index) => Ok(index),
            Err(_) => {
                // Create new index
                info!("Creating new Meilisearch index: {}", self.index_name);
                self.client
                    .create_index(&self.index_name, Some("id"))
                    .await
                    .map_err(|e| format!("Failed to create index: {}", e))?;

                // Wait for index creation
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                self.client
                    .get_index(&self.index_name)
                    .await
                    .map_err(|e| format!("Failed to get index after creation: {}", e))
            }
        }
    }

    /// Index a single MIDI file document.
    ///
    /// # Arguments
    /// * `document` - The MIDI file document to index
    pub async fn index_file(&self, document: &MidiSearchDocument) -> Result<(), String> {
        debug!("Indexing file: {} (id: {})", document.filename, document.id);

        let index = self.get_index().await?;

        index
            .add_documents(&[document], Some("id"))
            .await
            .map_err(|e| format!("Failed to index file: {}", e))?;

        Ok(())
    }

    /// Index multiple MIDI file documents in batch.
    ///
    /// More efficient than indexing files one at a time.
    ///
    /// # Arguments
    /// * `documents` - Vector of documents to index
    pub async fn index_files_batch(&self, documents: &[MidiSearchDocument]) -> Result<(), String> {
        if documents.is_empty() {
            return Ok(());
        }

        info!("Batch indexing {} files", documents.len());

        let index = self.get_index().await?;

        index
            .add_documents(documents, Some("id"))
            .await
            .map_err(|e| format!("Failed to batch index files: {}", e))?;

        info!("Batch indexing completed");
        Ok(())
    }

    /// Perform a full-text search.
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `limit` - Maximum number of results (default: 20)
    /// * `offset` - Number of results to skip (for pagination)
    pub async fn search(
        &self,
        query: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<SearchResult>, String> {
        debug!("Searching for: {}", query);

        let index = self.get_index().await?;

        let mut search = index.search();
        search.with_query(query);

        if let Some(limit) = limit {
            search.with_limit(limit);
        }

        if let Some(offset) = offset {
            search.with_offset(offset);
        }

        let results = search
            .execute::<MidiSearchDocument>()
            .await
            .map_err(|e| format!("Search failed: {}", e))?;

        let search_results = results
            .hits
            .into_iter()
            .map(|hit| SearchResult {
                document: hit.result,
                score: hit.ranking_score,
            })
            .collect();

        Ok(search_results)
    }

    /// Perform a faceted search with filters.
    ///
    /// # Arguments
    /// * `query` - Optional search query string
    /// * `filters` - Filter criteria
    /// * `limit` - Maximum number of results
    /// * `offset` - Number of results to skip
    pub async fn faceted_search(
        &self,
        query: Option<&str>,
        filters: &SearchFilters,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<SearchResult>, String> {
        debug!("Faceted search - query: {:?}, filters: {:?}", query, filters);

        let index = self.get_index().await?;

        // Build filter string
        let filter_string = self.build_filter_string(filters);

        let mut search = index.search();

        if let Some(q) = query {
            search.with_query(q);
        }

        if !filter_string.is_empty() {
            search.with_filter(&filter_string);
        }

        if let Some(limit) = limit {
            search.with_limit(limit);
        }

        if let Some(offset) = offset {
            search.with_offset(offset);
        }

        let results = search
            .execute::<MidiSearchDocument>()
            .await
            .map_err(|e| format!("Faceted search failed: {}", e))?;

        let search_results = results
            .hits
            .into_iter()
            .map(|hit| SearchResult {
                document: hit.result,
                score: hit.ranking_score,
            })
            .collect();

        Ok(search_results)
    }

    /// Build a Meilisearch filter string from SearchFilters.
    fn build_filter_string(&self, filters: &SearchFilters) -> String {
        let mut filter_parts = Vec::new();

        // Instrument filters
        if let Some(instruments) = &filters.instruments {
            if !instruments.is_empty() {
                let instrument_filters: Vec<String> = instruments
                    .iter()
                    .map(|i| format!("instruments = '{}'", i))
                    .collect();
                filter_parts.push(format!("({})", instrument_filters.join(" OR ")));
            }
        }

        // Tag filters
        if let Some(tags) = &filters.tags {
            if !tags.is_empty() {
                let tag_filters: Vec<String> = tags
                    .iter()
                    .map(|t| format!("tags = '{}'", t))
                    .collect();
                filter_parts.push(format!("({})", tag_filters.join(" OR ")));
            }
        }

        // BPM range filter
        if let (Some(min), Some(max)) = (filters.bpm_min, filters.bpm_max) {
            filter_parts.push(format!("bpm >= {} AND bpm <= {}", min, max));
        } else if let Some(min) = filters.bpm_min {
            filter_parts.push(format!("bpm >= {}", min));
        } else if let Some(max) = filters.bpm_max {
            filter_parts.push(format!("bpm <= {}", max));
        }

        // Key signature filter
        if let Some(key) = &filters.key_signature {
            filter_parts.push(format!("key_signature = '{}'", key));
        }

        // Manufacturer filter
        if let Some(manufacturer) = &filters.manufacturer {
            filter_parts.push(format!("manufacturer = '{}'", manufacturer));
        }

        // Collection filter
        if let Some(collection) = &filters.collection_name {
            filter_parts.push(format!("collection_name = '{}'", collection));
        }

        // Percussion filter
        if let Some(is_percussive) = filters.is_percussive {
            filter_parts.push(format!("is_percussive = {}", is_percussive));
        }

        // Timbre filters (VIP3)
        if let Some(timbres) = &filters.timbres {
            if !timbres.is_empty() {
                let timbre_filters: Vec<String> = timbres
                    .iter()
                    .map(|t| format!("timbres = '{}'", t))
                    .collect();
                filter_parts.push(format!("({})", timbre_filters.join(" OR ")));
            }
        }

        // Style filters (VIP3)
        if let Some(styles) = &filters.styles {
            if !styles.is_empty() {
                let style_filters: Vec<String> = styles
                    .iter()
                    .map(|s| format!("styles = '{}'", s))
                    .collect();
                filter_parts.push(format!("({})", style_filters.join(" OR ")));
            }
        }

        // Articulation filters (VIP3)
        if let Some(articulations) = &filters.articulations {
            if !articulations.is_empty() {
                let articulation_filters: Vec<String> = articulations
                    .iter()
                    .map(|a| format!("articulations = '{}'", a))
                    .collect();
                filter_parts.push(format!("({})", articulation_filters.join(" OR ")));
            }
        }

        // Join all filters with AND
        filter_parts.join(" AND ")
    }

    /// Delete a file from the index.
    ///
    /// # Arguments
    /// * `file_id` - The ID of the file to delete
    pub async fn delete_file(&self, file_id: i64) -> Result<(), String> {
        debug!("Deleting file from index: {}", file_id);

        let index = self.get_index().await?;

        index
            .delete_document(file_id)
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))?;

        Ok(())
    }

    /// Delete multiple files from the index in batch.
    ///
    /// # Arguments
    /// * `file_ids` - Vector of file IDs to delete
    pub async fn delete_files_batch(&self, file_ids: &[i64]) -> Result<(), String> {
        if file_ids.is_empty() {
            return Ok(());
        }

        info!("Batch deleting {} files from index", file_ids.len());

        let index = self.get_index().await?;

        index
            .delete_documents(file_ids)
            .await
            .map_err(|e| format!("Failed to batch delete files: {}", e))?;

        Ok(())
    }

    /// Clear all documents from the index.
    ///
    /// Use with caution!
    pub async fn clear_index(&self) -> Result<(), String> {
        warn!("Clearing all documents from index: {}", self.index_name);

        let index = self.get_index().await?;

        index
            .delete_all_documents()
            .await
            .map_err(|e| format!("Failed to clear index: {}", e))?;

        info!("Index cleared successfully");
        Ok(())
    }

    /// Get index statistics.
    pub async fn get_stats(&self) -> Result<HashMap<String, serde_json::Value>, String> {
        let index = self.get_index().await?;

        let stats = index
            .get_stats()
            .await
            .map_err(|e| format!("Failed to get stats: {}", e))?;

        let mut result = HashMap::new();
        result.insert(
            "number_of_documents".to_string(),
            serde_json::json!(stats.number_of_documents),
        );
        result.insert(
            "is_indexing".to_string(),
            serde_json::json!(stats.is_indexing),
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // MeilisearchClient Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_meilisearch_client_creation() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_meilisearch_client_creation_with_api_key() {
        let client = MeilisearchClient::new("http://localhost:7700", Some("test_api_key"), None);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_meilisearch_client_creation_with_custom_index() {
        let client =
            MeilisearchClient::new("http://localhost:7700", None, Some("custom_midi_index"));
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.index_name, "custom_midi_index");
    }

    #[tokio::test]
    async fn test_meilisearch_client_default_index_name() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        assert_eq!(client.index_name, "midi_files");
    }

    // ==========================================================================
    // SearchFilters Tests
    // ==========================================================================

    #[test]
    fn test_search_filters_default() {
        let filters = SearchFilters::default();
        assert!(filters.instruments.is_none());
        assert!(filters.tags.is_none());
        assert!(filters.bpm_min.is_none());
        assert!(filters.bpm_max.is_none());
        assert!(filters.key_signature.is_none());
        assert!(filters.manufacturer.is_none());
        assert!(filters.collection_name.is_none());
        assert!(filters.is_percussive.is_none());
        assert!(filters.timbres.is_none());
        assert!(filters.styles.is_none());
        assert!(filters.articulations.is_none());
    }

    #[test]
    fn test_search_filters_with_instruments() {
        let filters = SearchFilters {
            instruments: Some(vec!["piano".to_string(), "guitar".to_string()]),
            ..Default::default()
        };
        assert_eq!(
            filters.instruments,
            Some(vec!["piano".to_string(), "guitar".to_string()])
        );
    }

    #[test]
    fn test_search_filters_serialization() {
        let filters = SearchFilters {
            instruments: Some(vec!["piano".to_string()]),
            bpm_min: Some(100.0),
            bpm_max: Some(150.0),
            key_signature: Some("C Major".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&filters).unwrap();
        let deserialized: SearchFilters = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.instruments, filters.instruments);
        assert_eq!(deserialized.bpm_min, filters.bpm_min);
        assert_eq!(deserialized.bpm_max, filters.bpm_max);
        assert_eq!(deserialized.key_signature, filters.key_signature);
    }

    #[test]
    fn test_search_filters_clone() {
        let filters = SearchFilters {
            instruments: Some(vec!["drums".to_string()]),
            is_percussive: Some(true),
            ..Default::default()
        };

        let cloned = filters.clone();
        assert_eq!(cloned.instruments, filters.instruments);
        assert_eq!(cloned.is_percussive, filters.is_percussive);
    }

    // ==========================================================================
    // Filter String Building Tests
    // ==========================================================================

    #[tokio::test]
    async fn test_filter_string_building() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();

        let filters = SearchFilters {
            instruments: Some(vec!["piano".to_string(), "drums".to_string()]),
            bpm_min: Some(120.0),
            bpm_max: Some(140.0),
            key_signature: Some("C".to_string()),
            ..Default::default()
        };

        let filter_string = client.build_filter_string(&filters);

        assert!(filter_string.contains("instruments = 'piano'"));
        assert!(filter_string.contains("instruments = 'drums'"));
        assert!(filter_string.contains("bpm >= 120"));
        assert!(filter_string.contains("bpm <= 140"));
        assert!(filter_string.contains("key_signature = 'C'"));
    }

    #[tokio::test]
    async fn test_filter_string_empty_filters() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters::default();
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.is_empty());
    }

    #[tokio::test]
    async fn test_filter_string_only_instruments() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            instruments: Some(vec!["bass".to_string()]),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("instruments = 'bass'"));
        assert!(!filter_string.contains("bpm"));
    }

    #[tokio::test]
    async fn test_filter_string_only_tags() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            tags: Some(vec!["jazz".to_string(), "funk".to_string()]),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("tags = 'jazz'"));
        assert!(filter_string.contains("tags = 'funk'"));
        assert!(filter_string.contains("OR"));
    }

    #[tokio::test]
    async fn test_filter_string_only_bpm_min() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            bpm_min: Some(90.0),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("bpm >= 90"));
        assert!(!filter_string.contains("bpm <="));
    }

    #[tokio::test]
    async fn test_filter_string_only_bpm_max() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            bpm_max: Some(180.0),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("bpm <= 180"));
        assert!(!filter_string.contains("bpm >="));
    }

    #[tokio::test]
    async fn test_filter_string_bpm_range() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            bpm_min: Some(100.0),
            bpm_max: Some(120.0),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("bpm >= 100"));
        assert!(filter_string.contains("bpm <= 120"));
    }

    #[tokio::test]
    async fn test_filter_string_is_percussive() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            is_percussive: Some(true),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("is_percussive = true"));
    }

    #[tokio::test]
    async fn test_filter_string_manufacturer() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            manufacturer: Some("Akai".to_string()),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("manufacturer = 'Akai'"));
    }

    #[tokio::test]
    async fn test_filter_string_collection() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            collection_name: Some("MPC Expansion Pack".to_string()),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("collection_name = 'MPC Expansion Pack'"));
    }

    #[tokio::test]
    async fn test_filter_string_timbres() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            timbres: Some(vec!["warm".to_string(), "bright".to_string()]),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("timbres = 'warm'"));
        assert!(filter_string.contains("timbres = 'bright'"));
    }

    #[tokio::test]
    async fn test_filter_string_styles() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            styles: Some(vec!["hip-hop".to_string()]),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("styles = 'hip-hop'"));
    }

    #[tokio::test]
    async fn test_filter_string_articulations() {
        let client = MeilisearchClient::new("http://localhost:7700", None, None).unwrap();
        let filters = SearchFilters {
            articulations: Some(vec!["staccato".to_string(), "legato".to_string()]),
            ..Default::default()
        };
        let filter_string = client.build_filter_string(&filters);
        assert!(filter_string.contains("articulations = 'staccato'"));
        assert!(filter_string.contains("articulations = 'legato'"));
    }

    // ==========================================================================
    // MidiSearchDocument Tests
    // ==========================================================================

    #[test]
    fn test_midi_search_document_creation() {
        let doc = MidiSearchDocument {
            id: 1,
            filename: "test_file.mid".to_string(),
            original_filename: "Test File.mid".to_string(),
            filepath: "/path/to/test_file.mid".to_string(),
            tags: Some(vec!["jazz".to_string()]),
            instruments: Some(vec!["piano".to_string()]),
            bpm: Some(120.0),
            key_signature: Some("C Major".to_string()),
            time_signature: Some("4/4".to_string()),
            manufacturer: Some("Test Manufacturer".to_string()),
            collection_name: Some("Test Collection".to_string()),
            duration_seconds: Some(180.0),
            num_tracks: 4,
            is_multi_track: Some(true),
            is_percussive: Some(false),
            timbres: None,
            styles: None,
            articulations: None,
        };

        assert_eq!(doc.id, 1);
        assert_eq!(doc.filename, "test_file.mid");
        assert_eq!(doc.bpm, Some(120.0));
        assert_eq!(doc.num_tracks, 4);
    }

    #[test]
    fn test_midi_search_document_serialization() {
        let doc = MidiSearchDocument {
            id: 42,
            filename: "beat.mid".to_string(),
            original_filename: "Beat.mid".to_string(),
            filepath: "/beats/beat.mid".to_string(),
            tags: Some(vec!["drums".to_string(), "percussion".to_string()]),
            instruments: Some(vec!["kick".to_string(), "snare".to_string()]),
            bpm: Some(95.0),
            key_signature: None,
            time_signature: Some("4/4".to_string()),
            manufacturer: None,
            collection_name: None,
            duration_seconds: Some(16.0),
            num_tracks: 1,
            is_multi_track: Some(false),
            is_percussive: Some(true),
            timbres: None,
            styles: None,
            articulations: None,
        };

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: MidiSearchDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, doc.id);
        assert_eq!(deserialized.filename, doc.filename);
        assert_eq!(deserialized.bpm, doc.bpm);
        assert_eq!(deserialized.is_percussive, doc.is_percussive);
        assert_eq!(deserialized.tags, doc.tags);
        assert_eq!(deserialized.instruments, doc.instruments);
    }

    #[test]
    fn test_midi_search_document_clone() {
        let doc = MidiSearchDocument {
            id: 100,
            filename: "clone_test.mid".to_string(),
            original_filename: "Clone Test.mid".to_string(),
            filepath: "/test/clone_test.mid".to_string(),
            tags: None,
            instruments: None,
            bpm: Some(140.0),
            key_signature: Some("A Minor".to_string()),
            time_signature: None,
            manufacturer: None,
            collection_name: None,
            duration_seconds: None,
            num_tracks: 2,
            is_multi_track: None,
            is_percussive: None,
            timbres: None,
            styles: None,
            articulations: None,
        };

        let cloned = doc.clone();
        assert_eq!(cloned.id, doc.id);
        assert_eq!(cloned.filename, doc.filename);
        assert_eq!(cloned.bpm, doc.bpm);
        assert_eq!(cloned.key_signature, doc.key_signature);
    }

    #[test]
    fn test_midi_search_document_skips_none_fields() {
        let doc = MidiSearchDocument {
            id: 1,
            filename: "minimal.mid".to_string(),
            original_filename: "Minimal.mid".to_string(),
            filepath: "/minimal.mid".to_string(),
            tags: None,
            instruments: None,
            bpm: None,
            key_signature: None,
            time_signature: None,
            manufacturer: None,
            collection_name: None,
            duration_seconds: None,
            num_tracks: 1,
            is_multi_track: None,
            is_percussive: None,
            timbres: None,
            styles: None,
            articulations: None,
        };

        let json = serde_json::to_string(&doc).unwrap();
        // These fields should be skipped when None
        assert!(!json.contains("tags"));
        assert!(!json.contains("instruments"));
        assert!(!json.contains("bpm"));
        assert!(!json.contains("key_signature"));
        // These required fields should always be present
        assert!(json.contains("id"));
        assert!(json.contains("filename"));
        assert!(json.contains("num_tracks"));
    }

    // ==========================================================================
    // SearchResult Tests
    // ==========================================================================

    #[test]
    fn test_search_result_creation() {
        let doc = MidiSearchDocument {
            id: 1,
            filename: "result.mid".to_string(),
            original_filename: "Result.mid".to_string(),
            filepath: "/result.mid".to_string(),
            tags: None,
            instruments: None,
            bpm: None,
            key_signature: None,
            time_signature: None,
            manufacturer: None,
            collection_name: None,
            duration_seconds: None,
            num_tracks: 1,
            is_multi_track: None,
            is_percussive: None,
            timbres: None,
            styles: None,
            articulations: None,
        };

        let result = SearchResult {
            document: doc,
            score: Some(0.95),
        };

        assert_eq!(result.document.id, 1);
        assert_eq!(result.score, Some(0.95));
    }

    #[test]
    fn test_search_result_serialization() {
        let doc = MidiSearchDocument {
            id: 5,
            filename: "search_result.mid".to_string(),
            original_filename: "Search Result.mid".to_string(),
            filepath: "/search_result.mid".to_string(),
            tags: Some(vec!["test".to_string()]),
            instruments: None,
            bpm: Some(128.0),
            key_signature: None,
            time_signature: None,
            manufacturer: None,
            collection_name: None,
            duration_seconds: None,
            num_tracks: 2,
            is_multi_track: None,
            is_percussive: None,
            timbres: None,
            styles: None,
            articulations: None,
        };

        let result = SearchResult {
            document: doc,
            score: Some(0.87),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.document.id, result.document.id);
        assert_eq!(deserialized.score, result.score);
    }

    #[test]
    fn test_search_result_clone() {
        let doc = MidiSearchDocument {
            id: 10,
            filename: "clone.mid".to_string(),
            original_filename: "Clone.mid".to_string(),
            filepath: "/clone.mid".to_string(),
            tags: None,
            instruments: None,
            bpm: None,
            key_signature: None,
            time_signature: None,
            manufacturer: None,
            collection_name: None,
            duration_seconds: None,
            num_tracks: 1,
            is_multi_track: None,
            is_percussive: None,
            timbres: None,
            styles: None,
            articulations: None,
        };

        let result = SearchResult {
            document: doc,
            score: None,
        };

        let cloned = result.clone();
        assert_eq!(cloned.document.id, result.document.id);
        assert_eq!(cloned.score, result.score);
    }
}
