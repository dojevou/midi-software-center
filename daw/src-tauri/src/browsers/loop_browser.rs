// Loop Browser State and Commands
//
// Handles loop search, filtering, and preview functionality.
// Architecture:
// - LoopBrowserState: Trusty Module (pure state)
// - Implementation functions: Grown-up Script
// - Tauri commands: Task-O-Matic

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoopBrowserError {
    #[error("Loop not found: {0}")]
    LoopNotFound(String),
    #[error("Invalid BPM range: min={0}, max={1}")]
    InvalidBpmRange(f32, f32),
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub bpm: f32,
    pub key: Option<String>,
    pub duration_bars: u32,
    pub tags: Vec<String>,
    pub category: String,
}

impl LoopInfo {
    pub fn new(
        id: String,
        name: String,
        path: String,
        bpm: f32,
        duration_bars: u32,
        category: String,
    ) -> Self {
        Self { id, name, path, bpm, key: None, duration_bars, tags: Vec::new(), category }
    }

    pub fn matches_filter(&self, filter: &LoopFilter) -> bool {
        // BPM range filter
        if let Some(min_bpm) = filter.min_bpm {
            if self.bpm < min_bpm {
                return false;
            }
        }
        if let Some(max_bpm) = filter.max_bpm {
            if self.bpm > max_bpm {
                return false;
            }
        }

        // Key filter
        if let Some(ref filter_key) = filter.key {
            match &self.key {
                Some(loop_key) if loop_key != filter_key => return false,
                None => return false,
                _ => {},
            }
        }

        // Tag filter (all tags must match)
        for tag in &filter.tags {
            if !self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                return false;
            }
        }

        true
    }

    pub fn matches_search(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();
        self.name.to_lowercase().contains(&query_lower)
            || self.category.to_lowercase().contains(&query_lower)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoopFilter {
    pub min_bpm: Option<f32>,
    pub max_bpm: Option<f32>,
    pub key: Option<String>,
    pub tags: Vec<String>,
}

impl LoopFilter {
    pub fn new() -> Self {
        Self { min_bpm: None, max_bpm: None, key: None, tags: Vec::new() }
    }

    pub fn validate(&self) -> Result<(), LoopBrowserError> {
        if let (Some(min), Some(max)) = (self.min_bpm, self.max_bpm) {
            if min > max {
                return Err(LoopBrowserError::InvalidBpmRange(min, max));
            }
        }
        Ok(())
    }
}

impl Default for LoopFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopBrowserState {
    pub loops: HashMap<String, LoopInfo>,
    pub search_query: String,
    pub filter: LoopFilter,
    pub selected_loop: Option<String>,
    pub preview_loop: Option<String>,
}

impl LoopBrowserState {
    pub fn new() -> Self {
        Self {
            loops: HashMap::new(),
            search_query: String::new(),
            filter: LoopFilter::new(),
            selected_loop: None,
            preview_loop: None,
        }
    }

    pub fn add_loop(&mut self, loop_info: LoopInfo) {
        let id = loop_info.id.clone();
        self.loops.insert(id, loop_info);
    }

    pub fn remove_loop(&mut self, loop_id: &str) -> Option<LoopInfo> {
        self.loops.remove(loop_id)
    }

    pub fn search_loops_impl(&mut self, query: String) -> Vec<LoopInfo> {
        self.search_query = query;
        self.get_filtered_loops()
    }

    pub fn set_filter_impl(
        &mut self,
        filter: LoopFilter,
    ) -> Result<Vec<LoopInfo>, LoopBrowserError> {
        filter.validate()?;
        self.filter = filter;
        Ok(self.get_filtered_loops())
    }

    pub fn get_filtered_loops(&self) -> Vec<LoopInfo> {
        self.loops
            .values()
            .filter(|loop_info| {
                loop_info.matches_search(&self.search_query)
                    && loop_info.matches_filter(&self.filter)
            })
            .cloned()
            .collect()
    }

    pub fn preview_loop_impl(&mut self, loop_id: String) -> Result<LoopInfo, LoopBrowserError> {
        let loop_info = self
            .loops
            .get(&loop_id)
            .ok_or_else(|| LoopBrowserError::LoopNotFound(loop_id.clone()))?;
        self.preview_loop = Some(loop_id);
        Ok(loop_info.clone())
    }

    pub fn select_loop_impl(&mut self, loop_id: String) -> Result<LoopInfo, LoopBrowserError> {
        let loop_info = self
            .loops
            .get(&loop_id)
            .ok_or_else(|| LoopBrowserError::LoopNotFound(loop_id.clone()))?;
        self.selected_loop = Some(loop_id);
        Ok(loop_info.clone())
    }

    pub fn stop_preview(&mut self) {
        self.preview_loop = None;
    }

    pub fn clear_selection(&mut self) {
        self.selected_loop = None;
    }

    pub fn get_loop(&self, loop_id: &str) -> Option<&LoopInfo> {
        self.loops.get(loop_id)
    }

    pub fn loop_count(&self) -> usize {
        self.loops.len()
    }

    pub fn filtered_count(&self) -> usize {
        self.get_filtered_loops().len()
    }
}

impl Default for LoopBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Command Handlers (Task-O-Matic)
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn search_loops(
    state: State<'_, Mutex<LoopBrowserState>>,
    query: String,
) -> Result<Vec<LoopInfo>, String> {
    Ok(state.lock().map_err(|e| format!("Lock error: {}", e))?.search_loops_impl(query))
}

#[tauri::command]
pub async fn preview_loop(
    state: State<'_, Mutex<LoopBrowserState>>,
    loop_id: String,
) -> Result<LoopInfo, String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .preview_loop_impl(loop_id)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_loop(id: &str, name: &str, bpm: f32) -> LoopInfo {
        LoopInfo::new(
            id.to_string(),
            name.to_string(),
            format!("/path/to/{}.mid", id),
            bpm,
            4,
            "drums".to_string(),
        )
    }

    #[test]
    fn test_loop_info_new() {
        let loop_info = create_test_loop("loop1", "Drum Loop", 120.0);
        assert_eq!(loop_info.id, "loop1");
        assert_eq!(loop_info.name, "Drum Loop");
        assert_eq!(loop_info.bpm, 120.0);
        assert_eq!(loop_info.duration_bars, 4);
    }

    #[test]
    fn test_loop_info_matches_filter_bpm() {
        let loop_info = create_test_loop("loop1", "Test", 120.0);

        let filter =
            LoopFilter { min_bpm: Some(100.0), max_bpm: Some(140.0), key: None, tags: vec![] };
        assert!(loop_info.matches_filter(&filter));

        let filter =
            LoopFilter { min_bpm: Some(130.0), max_bpm: Some(140.0), key: None, tags: vec![] };
        assert!(!loop_info.matches_filter(&filter));
    }

    #[test]
    fn test_loop_info_matches_filter_key() {
        let mut loop_info = create_test_loop("loop1", "Test", 120.0);
        loop_info.key = Some("C".to_string());

        let filter =
            LoopFilter { min_bpm: None, max_bpm: None, key: Some("C".to_string()), tags: vec![] };
        assert!(loop_info.matches_filter(&filter));

        let filter =
            LoopFilter { min_bpm: None, max_bpm: None, key: Some("D".to_string()), tags: vec![] };
        assert!(!loop_info.matches_filter(&filter));
    }

    #[test]
    fn test_loop_info_matches_filter_tags() {
        let mut loop_info = create_test_loop("loop1", "Test", 120.0);
        loop_info.tags = vec!["rock".to_string(), "heavy".to_string()];

        let filter =
            LoopFilter { min_bpm: None, max_bpm: None, key: None, tags: vec!["rock".to_string()] };
        assert!(loop_info.matches_filter(&filter));

        let filter =
            LoopFilter { min_bpm: None, max_bpm: None, key: None, tags: vec!["jazz".to_string()] };
        assert!(!loop_info.matches_filter(&filter));
    }

    #[test]
    fn test_loop_info_matches_search() {
        let loop_info = create_test_loop("loop1", "Heavy Rock Drum", 120.0);

        assert!(loop_info.matches_search(""));
        assert!(loop_info.matches_search("heavy"));
        assert!(loop_info.matches_search("ROCK"));
        assert!(loop_info.matches_search("drum"));
        assert!(!loop_info.matches_search("jazz"));
    }

    #[test]
    fn test_loop_filter_validate() {
        let filter =
            LoopFilter { min_bpm: Some(100.0), max_bpm: Some(140.0), key: None, tags: vec![] };
        assert!(filter.validate().is_ok());

        let filter =
            LoopFilter { min_bpm: Some(140.0), max_bpm: Some(100.0), key: None, tags: vec![] };
        assert!(matches!(
            filter.validate(),
            Err(LoopBrowserError::InvalidBpmRange(_, _))
        ));
    }

    #[test]
    fn test_loop_browser_state_new() {
        let state = LoopBrowserState::new();
        assert_eq!(state.loops.len(), 0);
        assert_eq!(state.search_query, "");
        assert!(state.selected_loop.is_none());
    }

    #[test]
    fn test_add_loop() {
        let mut state = LoopBrowserState::new();
        let loop_info = create_test_loop("loop1", "Test", 120.0);
        state.add_loop(loop_info);
        assert_eq!(state.loop_count(), 1);
    }

    #[test]
    fn test_remove_loop() {
        let mut state = LoopBrowserState::new();
        let loop_info = create_test_loop("loop1", "Test", 120.0);
        state.add_loop(loop_info);

        let removed = state.remove_loop("loop1");
        assert!(removed.is_some());
        assert_eq!(state.loop_count(), 0);
    }

    #[test]
    fn test_search_loops() {
        let mut state = LoopBrowserState::new();
        state.add_loop(create_test_loop("loop1", "Rock Drum", 120.0));
        state.add_loop(create_test_loop("loop2", "Jazz Bass", 100.0));
        state.add_loop(create_test_loop("loop3", "Rock Guitar", 130.0));

        let results = state.search_loops_impl("rock".to_string());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_loops_empty_query() {
        let mut state = LoopBrowserState::new();
        state.add_loop(create_test_loop("loop1", "Rock Drum", 120.0));
        state.add_loop(create_test_loop("loop2", "Jazz Bass", 100.0));

        let results = state.search_loops_impl("".to_string());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_set_filter() {
        let mut state = LoopBrowserState::new();
        state.add_loop(create_test_loop("loop1", "Test", 110.0));
        state.add_loop(create_test_loop("loop2", "Test", 130.0));

        let filter =
            LoopFilter { min_bpm: Some(120.0), max_bpm: Some(140.0), key: None, tags: vec![] };

        let results = state.set_filter_impl(filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "loop2");
    }

    #[test]
    fn test_set_filter_invalid() {
        let mut state = LoopBrowserState::new();

        let filter =
            LoopFilter { min_bpm: Some(140.0), max_bpm: Some(100.0), key: None, tags: vec![] };

        let result = state.set_filter_impl(filter);
        assert!(matches!(
            result,
            Err(LoopBrowserError::InvalidBpmRange(_, _))
        ));
    }

    #[test]
    fn test_preview_loop() {
        let mut state = LoopBrowserState::new();
        let loop_info = create_test_loop("loop1", "Test", 120.0);
        state.add_loop(loop_info);

        let result = state.preview_loop_impl("loop1".to_string()).unwrap();
        assert_eq!(result.id, "loop1");
        assert_eq!(state.preview_loop, Some("loop1".to_string()));
    }

    #[test]
    fn test_preview_loop_not_found() {
        let mut state = LoopBrowserState::new();
        let result = state.preview_loop_impl("nonexistent".to_string());
        assert!(matches!(result, Err(LoopBrowserError::LoopNotFound(_))));
    }

    #[test]
    fn test_select_loop() {
        let mut state = LoopBrowserState::new();
        let loop_info = create_test_loop("loop1", "Test", 120.0);
        state.add_loop(loop_info);

        let result = state.select_loop_impl("loop1".to_string()).unwrap();
        assert_eq!(result.id, "loop1");
        assert_eq!(state.selected_loop, Some("loop1".to_string()));
    }

    #[test]
    fn test_select_loop_not_found() {
        let mut state = LoopBrowserState::new();
        let result = state.select_loop_impl("nonexistent".to_string());
        assert!(matches!(result, Err(LoopBrowserError::LoopNotFound(_))));
    }

    #[test]
    fn test_stop_preview() {
        let mut state = LoopBrowserState::new();
        state.preview_loop = Some("loop1".to_string());
        state.stop_preview();
        assert!(state.preview_loop.is_none());
    }

    #[test]
    fn test_clear_selection() {
        let mut state = LoopBrowserState::new();
        state.selected_loop = Some("loop1".to_string());
        state.clear_selection();
        assert!(state.selected_loop.is_none());
    }

    #[test]
    fn test_get_loop() {
        let mut state = LoopBrowserState::new();
        let loop_info = create_test_loop("loop1", "Test", 120.0);
        state.add_loop(loop_info);

        let result = state.get_loop("loop1").unwrap();
        assert_eq!(result.id, "loop1");
    }

    #[test]
    fn test_filtered_count() {
        let mut state = LoopBrowserState::new();
        state.add_loop(create_test_loop("loop1", "Test", 110.0));
        state.add_loop(create_test_loop("loop2", "Test", 130.0));

        let filter =
            LoopFilter { min_bpm: Some(120.0), max_bpm: Some(140.0), key: None, tags: vec![] };
        state.set_filter_impl(filter).unwrap();

        assert_eq!(state.loop_count(), 2);
        assert_eq!(state.filtered_count(), 1);
    }
}
