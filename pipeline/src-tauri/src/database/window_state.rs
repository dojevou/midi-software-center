
/// Database Window State
///
/// Trusty Module: Pure data structures for database window state including
/// search filters, results, and pagination. No I/O, no side effects.
use serde::{Deserialize, Serialize};

/// Search filters for database window
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    /// Text query for file name, path, or tags
    pub query: Option<String>,
    /// BPM range filter (min, max)
    pub bpm_range: Option<(f32, f32)>,
    /// Key signature filter (e.g., "C", "Am", "F#")
    pub key: Option<String>,
    /// Category filter (e.g., "drum_loop", "melody", "bass")
    pub category: Option<String>,
    /// Tags filter (all must match)
    pub tags: Vec<String>,
    /// Manufacturer filter
    pub manufacturer: Option<String>,
    /// Duration range in seconds (min, max)
    pub duration_range: Option<(f32, f32)>,
    /// Time signature filter (e.g., "4/4", "3/4")
    pub time_signature: Option<String>,
    /// Sort by field
    pub sort_by: SortField,
    /// Sort order
    pub sort_order: SortOrder,
    /// Results per page
    pub page_size: usize,
    /// Current page (0-based)
    pub page: usize,
}

impl SearchFilters {
    /// Create new default filters
    pub fn new() -> Self {
        SearchFilters {
            page_size: 50,
            sort_by: SortField::DateAdded,
            sort_order: SortOrder::Descending,
            ..Default::default()
        }
    }

    /// Create with query
    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    /// Create with BPM range
    pub fn with_bpm_range(mut self, min: f32, max: f32) -> Self {
        self.bpm_range = Some((min, max));
        self
    }

    /// Create with category
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// Add tag filter
    pub fn add_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }

    /// Check if filters are empty (no active filters)
    pub fn is_empty(&self) -> bool {
        self.query.is_none()
            && self.bpm_range.is_none()
            && self.key.is_none()
            && self.category.is_none()
            && self.tags.is_empty()
            && self.manufacturer.is_none()
            && self.duration_range.is_none()
            && self.time_signature.is_none()
    }

    /// Count active filters
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if self.query.is_some() {
            count += 1;
        }
        if self.bpm_range.is_some() {
            count += 1;
        }
        if self.key.is_some() {
            count += 1;
        }
        if self.category.is_some() {
            count += 1;
        }
        if !self.tags.is_empty() {
            count += 1;
        }
        if self.manufacturer.is_some() {
            count += 1;
        }
        if self.duration_range.is_some() {
            count += 1;
        }
        if self.time_signature.is_some() {
            count += 1;
        }
        count
    }
}

/// Sort field options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SortField {
    /// Sort by file name
    FileName,
    /// Sort by date added to database
    #[default]
    DateAdded,
    /// Sort by BPM
    Bpm,
    /// Sort by duration
    Duration,
    /// Sort by file size
    FileSize,
    /// Sort by last access time
    LastAccessed,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SortOrder {
    /// Ascending order (A-Z, 0-9, oldest-newest)
    Ascending,
    /// Descending order (Z-A, 9-0, newest-oldest)
    #[default]
    Descending,
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Database file ID
    pub file_id: i32,
    /// File name
    pub file_name: String,
    /// Full file path
    pub file_path: String,
    /// BPM (if detected)
    pub bpm: Option<f32>,
    /// Key signature (if detected)
    pub key: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// Duration in seconds
    pub duration: f32,
    /// File size in bytes
    pub file_size: i64,
    /// Date added to database
    pub date_added: String,
    /// Last accessed timestamp
    pub last_accessed: Option<String>,
}

impl SearchResult {
    /// Create new search result
    pub fn new(file_id: i32, file_name: String, file_path: String) -> Self {
        SearchResult {
            file_id,
            file_name,
            file_path,
            bpm: None,
            key: None,
            category: None,
            tags: Vec::new(),
            duration: 0.0,
            file_size: 0,
            date_added: String::new(),
            last_accessed: None,
        }
    }

    /// Check if result matches text query
    pub fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.file_name.to_lowercase().contains(&query_lower)
            || self.file_path.to_lowercase().contains(&query_lower)
            || self.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
    }
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Total number of results
    pub total_results: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Current page (0-based)
    pub current_page: usize,
    /// Results per page
    pub page_size: usize,
    /// Has previous page
    pub has_previous: bool,
    /// Has next page
    pub has_next: bool,
}

impl PaginationInfo {
    /// Create pagination info
    pub fn new(total_results: usize, current_page: usize, page_size: usize) -> Self {
        let total_pages = if page_size > 0 {
            total_results.div_ceil(page_size)
        } else {
            0
        };

        PaginationInfo {
            total_results,
            total_pages,
            current_page,
            page_size,
            has_previous: current_page > 0,
            has_next: current_page + 1 < total_pages,
        }
    }

    /// Get start index for current page
    pub fn start_index(&self) -> usize {
        self.current_page * self.page_size
    }

    /// Get end index for current page
    pub fn end_index(&self) -> usize {
        ((self.current_page + 1) * self.page_size).min(self.total_results)
    }

    /// Check if page number is valid
    pub fn is_valid_page(&self, page: usize) -> bool {
        page < self.total_pages
    }
}

/// Database window state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseWindowState {
    /// Current search filters
    pub filters: SearchFilters,
    /// Current search results
    pub results: Vec<SearchResult>,
    /// Pagination info
    pub pagination: PaginationInfo,
    /// Selected file IDs
    pub selected_files: Vec<i32>,
    /// View mode (list, grid, details)
    pub view_mode: ViewMode,
    /// Show preview panel
    pub show_preview: bool,
}

impl Default for DatabaseWindowState {
    fn default() -> Self {
        DatabaseWindowState {
            filters: SearchFilters::new(),
            results: Vec::new(),
            pagination: PaginationInfo::new(0, 0, 50),
            selected_files: Vec::new(),
            view_mode: ViewMode::List,
            show_preview: true,
        }
    }
}

impl DatabaseWindowState {
    /// Create new database window state
    pub fn new() -> Self {
        Self::default()
    }

    /// Update results and pagination
    pub fn set_results(&mut self, results: Vec<SearchResult>, total_count: usize) {
        self.results = results;
        self.pagination =
            PaginationInfo::new(total_count, self.filters.page, self.filters.page_size);
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_files.clear();
    }

    /// Select file
    pub fn select_file(&mut self, file_id: i32) {
        if !self.selected_files.contains(&file_id) {
            self.selected_files.push(file_id);
        }
    }

    /// Deselect file
    pub fn deselect_file(&mut self, file_id: i32) {
        self.selected_files.retain(|&id| id != file_id);
    }

    /// Toggle file selection
    pub fn toggle_selection(&mut self, file_id: i32) {
        if self.selected_files.contains(&file_id) {
            self.deselect_file(file_id);
        } else {
            self.select_file(file_id);
        }
    }

    /// Get selected files count
    pub fn selected_count(&self) -> usize {
        self.selected_files.len()
    }

    /// Check if file is selected
    pub fn is_selected(&self, file_id: i32) -> bool {
        self.selected_files.contains(&file_id)
    }

    /// Go to next page
    pub fn next_page(&mut self) -> bool {
        if self.pagination.has_next {
            self.filters.page += 1;
            true
        } else {
            false
        }
    }

    /// Go to previous page
    pub fn previous_page(&mut self) -> bool {
        if self.pagination.has_previous {
            self.filters.page -= 1;
            true
        } else {
            false
        }
    }

    /// Go to specific page
    pub fn go_to_page(&mut self, page: usize) -> bool {
        if self.pagination.is_valid_page(page) {
            self.filters.page = page;
            true
        } else {
            false
        }
    }
}

/// View mode for database window
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ViewMode {
    /// List view (compact rows)
    #[default]
    List,
    /// Grid view (thumbnails/cards)
    Grid,
    /// Details view (full information table)
    Details,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_filters_empty() {
        let filters = SearchFilters::new();
        assert!(filters.is_empty());
        assert_eq!(filters.active_count(), 0);
    }

    #[test]
    fn test_search_filters_active_count() {
        let mut filters = SearchFilters::new();
        filters.query = Some("test".to_string());
        filters.bpm_range = Some((120.0, 140.0));
        assert_eq!(filters.active_count(), 2);
    }

    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo::new(100, 0, 25);
        assert_eq!(pagination.total_pages, 4);
        assert!(!pagination.has_previous);
        assert!(pagination.has_next);
        assert_eq!(pagination.start_index(), 0);
        assert_eq!(pagination.end_index(), 25);
    }

    #[test]
    fn test_pagination_last_page() {
        let pagination = PaginationInfo::new(100, 3, 25);
        assert!(pagination.has_previous);
        assert!(!pagination.has_next);
        assert_eq!(pagination.start_index(), 75);
        assert_eq!(pagination.end_index(), 100);
    }

    #[test]
    fn test_database_window_selection() {
        let mut state = DatabaseWindowState::new();
        assert_eq!(state.selected_count(), 0);

        state.select_file(1);
        assert_eq!(state.selected_count(), 1);
        assert!(state.is_selected(1));

        state.toggle_selection(1);
        assert_eq!(state.selected_count(), 0);
        assert!(!state.is_selected(1));
    }

    #[test]
    fn test_database_window_pagination() {
        let mut state = DatabaseWindowState::new();
        state.set_results(vec![], 100);

        assert_eq!(state.pagination.current_page, 0);

        assert!(state.next_page());
        assert_eq!(state.filters.page, 1);

        assert!(state.previous_page());
        assert_eq!(state.filters.page, 0);

        assert!(!state.previous_page());
    }

    #[test]
    fn test_search_result_matches_query() {
        let result = SearchResult {
            file_id: 1,
            file_name: "Piano_Loop.mid".to_string(),
            file_path: "/path/to/Piano_Loop.mid".to_string(),
            bpm: Some(120.0),
            key: Some("C".to_string()),
            category: Some("melody".to_string()),
            tags: vec!["piano".to_string(), "loop".to_string()],
            duration: 30.0,
            file_size: 1024,
            date_added: "2025-11-03".to_string(),
            last_accessed: None,
        };

        assert!(result.matches_query("piano"));
        assert!(result.matches_query("LOOP"));
        assert!(result.matches_query("path"));
        assert!(!result.matches_query("guitar"));
    }
}
