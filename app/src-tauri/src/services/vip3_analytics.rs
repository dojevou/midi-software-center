use anyhow::{Context, Result};
use duckdb::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Filter counts for VIP3 UI dropdowns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCounts {
    pub folders: HashMap<String, i64>,
    pub bpm_ranges: HashMap<i32, i64>,
    pub keys: HashMap<i32, i64>,
    pub instruments: HashMap<i32, i64>,
    pub timbres: HashMap<i32, i64>,
    pub styles: HashMap<i32, i64>,
    pub articulations: HashMap<i32, i64>,
    pub channel_counts: HashMap<i16, i64>,
    pub multi_track: i64,
}

/// VIP3 Analytics Service using DuckDB for fast aggregations
///
/// DuckDB queries PostgreSQL directly for 10-20x faster GROUP BY performance:
/// - BPM range counts: 647ms → 50-100ms
/// - Instrument counts: 563ms → 50-100ms
/// - Timbre counts: 210ms → 20-50ms
pub struct VIP3AnalyticsService {
    conn: Arc<Mutex<Connection>>,
    pg_url: String,
}

impl VIP3AnalyticsService {
    /// Create new analytics service
    pub fn new(pg_url: String) -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to create DuckDB in-memory connection")?;

        // Install and load PostgreSQL extension
        conn.execute_batch("INSTALL postgres; LOAD postgres;")
            .context("Failed to install PostgreSQL extension")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            pg_url,
        })
    }

    /// Attach to PostgreSQL database
    async fn attach_postgres(&self) -> Result<()> {
        let conn = self.conn.lock().await;

        let attach_sql = format!(
            "ATTACH '{}' AS pg (TYPE POSTGRES)",
            self.pg_url
        );

        conn.execute(&attach_sql, [])
            .context("Failed to attach PostgreSQL database")?;

        Ok(())
    }

    /// Get all filter counts in a single call
    pub async fn get_filter_counts(&self) -> Result<FilterCounts> {
        // Attach to PostgreSQL if not already attached
        self.attach_postgres().await.ok(); // Ignore error if already attached

        let conn = self.conn.lock().await;

        // Query 1: Folder counts
        let folders = self.get_folder_counts(&conn)?;

        // Query 2: BPM range counts
        let bpm_ranges = self.get_bpm_range_counts(&conn)?;

        // Query 3: Key counts
        let keys = self.get_key_counts(&conn)?;

        // Query 4: Instrument tag counts
        let instruments = self.get_instrument_counts(&conn)?;

        // Query 5: Timbre counts
        let timbres = self.get_timbre_counts(&conn)?;

        // Query 6: Style counts
        let styles = self.get_style_counts(&conn)?;

        // Query 7: Articulation counts
        let articulations = self.get_articulation_counts(&conn)?;

        // Query 8: Channel count distribution
        let channel_counts = self.get_channel_count_distribution(&conn)?;

        // Query 9: Multi-track file count
        let multi_track = self.get_multi_track_count(&conn)?;

        Ok(FilterCounts {
            folders,
            bpm_ranges,
            keys,
            instruments,
            timbres,
            styles,
            articulations,
            channel_counts,
            multi_track,
        })
    }

    fn get_folder_counts(&self, conn: &Connection) -> Result<HashMap<String, i64>> {
        let mut stmt = conn.prepare(
            "SELECT fld.path, COUNT(DISTINCT f.id)::BIGINT as count
             FROM pg.files f
             JOIN pg.folders fld ON f.folder_id = fld.id
             WHERE f.folder_id IS NOT NULL
             GROUP BY fld.path
             ORDER BY count DESC
             LIMIT 1000"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (path, count) = row?;
            map.insert(path, count);
        }

        Ok(map)
    }

    fn get_bpm_range_counts(&self, conn: &Connection) -> Result<HashMap<i32, i64>> {
        let mut stmt = conn.prepare(
            "SELECT bpm_range_id, COUNT(DISTINCT id)::BIGINT as count
             FROM pg.files
             WHERE bpm_range_id IS NOT NULL
             GROUP BY bpm_range_id
             ORDER BY bpm_range_id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (bpm_range_id, count) = row?;
            map.insert(bpm_range_id, count);
        }

        Ok(map)
    }

    fn get_key_counts(&self, conn: &Connection) -> Result<HashMap<i32, i64>> {
        let mut stmt = conn.prepare(
            "SELECT key_id, COUNT(DISTINCT id)::BIGINT as count
             FROM pg.files
             WHERE key_id IS NOT NULL
             GROUP BY key_id
             ORDER BY key_id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (key_id, count) = row?;
            map.insert(key_id, count);
        }

        Ok(map)
    }

    fn get_instrument_counts(&self, conn: &Connection) -> Result<HashMap<i32, i64>> {
        let mut stmt = conn.prepare(
            "SELECT tag_id, COUNT(DISTINCT file_id)::BIGINT as count
             FROM pg.file_tags ft
             JOIN pg.tags t ON ft.tag_id = t.id
             WHERE t.category = 'instrument' AND t.is_active = true
             GROUP BY tag_id
             ORDER BY count DESC
             LIMIT 100"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (tag_id, count) = row?;
            map.insert(tag_id, count);
        }

        Ok(map)
    }

    fn get_timbre_counts(&self, conn: &Connection) -> Result<HashMap<i32, i64>> {
        let mut stmt = conn.prepare(
            "SELECT timbre_id, COUNT(DISTINCT file_id)::BIGINT as count
             FROM pg.midi_file_timbres
             GROUP BY timbre_id
             ORDER BY timbre_id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (timbre_id, count) = row?;
            map.insert(timbre_id, count);
        }

        Ok(map)
    }

    fn get_style_counts(&self, conn: &Connection) -> Result<HashMap<i32, i64>> {
        let mut stmt = conn.prepare(
            "SELECT style_id, COUNT(DISTINCT file_id)::BIGINT as count
             FROM pg.midi_file_styles
             GROUP BY style_id
             ORDER BY style_id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (style_id, count) = row?;
            map.insert(style_id, count);
        }

        Ok(map)
    }

    fn get_articulation_counts(&self, conn: &Connection) -> Result<HashMap<i32, i64>> {
        let mut stmt = conn.prepare(
            "SELECT articulation_id, COUNT(DISTINCT file_id)::BIGINT as count
             FROM pg.midi_file_articulations
             GROUP BY articulation_id
             ORDER BY articulation_id"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (articulation_id, count) = row?;
            map.insert(articulation_id, count);
        }

        Ok(map)
    }

    fn get_channel_count_distribution(&self, conn: &Connection) -> Result<HashMap<i16, i64>> {
        let mut stmt = conn.prepare(
            "SELECT channel_count, COUNT(DISTINCT id)::BIGINT as count
             FROM pg.files
             WHERE channel_count IS NOT NULL AND channel_count > 0
             GROUP BY channel_count
             ORDER BY channel_count"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i16>(0)?, row.get::<_, i64>(1)?))
        })?;

        let mut map = HashMap::new();
        for row in rows {
            let (channel_count, count) = row?;
            map.insert(channel_count, count);
        }

        Ok(map)
    }

    fn get_multi_track_count(&self, conn: &Connection) -> Result<i64> {
        let mut stmt = conn.prepare(
            "SELECT COUNT(DISTINCT id)::BIGINT as count
             FROM pg.files
             WHERE is_multi_track = true"
        )?;

        let count = stmt.query_row([], |row| row.get::<_, i64>(0))?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vip3_analytics_service() {
        let pg_url = "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string();

        let service = VIP3AnalyticsService::new(pg_url).unwrap();
        let counts = service.get_filter_counts().await.unwrap();

        println!("BPM ranges: {:?}", counts.bpm_ranges);
        println!("Keys: {:?}", counts.keys);
        println!("Timbres count: {}", counts.timbres.len());
        println!("Multi-track files: {}", counts.multi_track);

        assert!(counts.bpm_ranges.len() > 0, "Should have BPM range counts");
    }
}
