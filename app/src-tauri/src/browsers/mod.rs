// DAW Browser Modules
//
// This module contains browser implementations for the DAW application.
// Browsers provide navigation and search functionality for loops, projects, and presets.
// Each browser follows the Three Archetypes pattern:
// - Pure state structures (Trusty Module)
// - Implementation logic (Grown-up Script)
// - Tauri command handlers (Task-O-Matic)

pub mod loop_browser;
pub mod project_browser;

// Re-export common types for convenience
pub use loop_browser::{
    preview_loop, search_loops, LoopBrowserError, LoopBrowserState, LoopFilter, LoopInfo,
};
pub use project_browser::{
    create_project, delete_project, open_project, ProjectBrowserError, ProjectBrowserState,
    ProjectInfo,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports_loop_browser() {
        let state = LoopBrowserState::default();
        assert_eq!(state.search_query, "");
    }

    #[test]
    fn test_module_exports_project_browser() {
        let state = ProjectBrowserState::default();
        assert_eq!(state.recent_projects.len(), 0);
    }

    #[test]
    fn test_loop_filter_serialization() {
        let filter =
            LoopFilter { min_bpm: Some(120.0), max_bpm: Some(140.0), key: None, tags: vec![] };
        let json = serde_json::to_string(&filter).unwrap();
        let deserialized: LoopFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(filter.min_bpm, deserialized.min_bpm);
    }

    #[test]
    fn test_project_info_serialization() {
        let info = ProjectInfo {
            name: "Test Project".to_string(),
            path: "/path/to/project".to_string(),
            last_modified: 12345,
            is_favorite: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ProjectInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.name, deserialized.name);
    }
}
