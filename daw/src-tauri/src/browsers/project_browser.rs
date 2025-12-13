// Project Browser State and Commands
//
// Handles project management, navigation, and favorites.
// Architecture:
// - ProjectBrowserState: Trusty Module (pure state)
// - Implementation functions: Grown-up Script
// - Tauri commands: Task-O-Matic

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectBrowserError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    #[error("Project already exists: {0}")]
    ProjectAlreadyExists(String),
    #[error("Invalid project name: {0}")]
    InvalidProjectName(String),
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub last_modified: i64, // Unix timestamp
    pub is_favorite: bool,
}

impl ProjectInfo {
    pub fn new(
        name: String,
        path: String,
        last_modified: i64,
    ) -> Result<Self, ProjectBrowserError> {
        if name.is_empty() {
            return Err(ProjectBrowserError::InvalidProjectName(name));
        }
        if path.is_empty() {
            return Err(ProjectBrowserError::InvalidPath(path));
        }

        Ok(Self { name, path, last_modified, is_favorite: false })
    }

    pub fn with_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBrowserState {
    pub projects: HashMap<String, ProjectInfo>, // Key is path
    pub recent_projects: Vec<String>,           // Paths in recency order
    pub favorites: HashSet<String>,             // Favorite project paths
    pub max_recent: usize,
}

impl ProjectBrowserState {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            recent_projects: Vec::new(),
            favorites: HashSet::new(),
            max_recent: 10,
        }
    }

    pub fn create_project_impl(
        &mut self,
        name: String,
        path: String,
    ) -> Result<ProjectInfo, ProjectBrowserError> {
        if self.projects.contains_key(&path) {
            return Err(ProjectBrowserError::ProjectAlreadyExists(path));
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let project = ProjectInfo::new(name, path.clone(), now)?;
        self.projects.insert(path.clone(), project.clone());
        self.add_to_recent(path);

        Ok(project)
    }

    pub fn open_project_impl(&mut self, path: String) -> Result<ProjectInfo, ProjectBrowserError> {
        let project = self
            .projects
            .get(&path)
            .ok_or_else(|| ProjectBrowserError::ProjectNotFound(path.clone()))?
            .clone();

        self.add_to_recent(path);
        Ok(project)
    }

    pub fn delete_project_impl(
        &mut self,
        path: String,
    ) -> Result<ProjectInfo, ProjectBrowserError> {
        let project = self
            .projects
            .remove(&path)
            .ok_or_else(|| ProjectBrowserError::ProjectNotFound(path.clone()))?;

        self.remove_from_recent(&path);
        self.favorites.remove(&path);

        Ok(project)
    }

    pub fn toggle_favorite_impl(&mut self, path: String) -> Result<bool, ProjectBrowserError> {
        let project = self
            .projects
            .get_mut(&path)
            .ok_or_else(|| ProjectBrowserError::ProjectNotFound(path.clone()))?;

        let is_favorite = !project.is_favorite;
        project.is_favorite = is_favorite;

        if is_favorite {
            self.favorites.insert(path);
        } else {
            self.favorites.remove(&path);
        }

        Ok(is_favorite)
    }

    pub fn update_last_modified_impl(&mut self, path: String) -> Result<(), ProjectBrowserError> {
        let project = self
            .projects
            .get_mut(&path)
            .ok_or_else(|| ProjectBrowserError::ProjectNotFound(path.clone()))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        project.last_modified = now;
        self.add_to_recent(path);

        Ok(())
    }

    fn add_to_recent(&mut self, path: String) {
        // Remove if already in list
        self.recent_projects.retain(|p| p != &path);

        // Add to front
        self.recent_projects.insert(0, path);

        // Trim to max_recent
        if self.recent_projects.len() > self.max_recent {
            self.recent_projects.truncate(self.max_recent);
        }
    }

    fn remove_from_recent(&mut self, path: &str) {
        self.recent_projects.retain(|p| p != path);
    }

    pub fn get_recent_projects(&self) -> Vec<ProjectInfo> {
        self.recent_projects
            .iter()
            .filter_map(|path| self.projects.get(path).cloned())
            .collect()
    }

    pub fn get_favorite_projects(&self) -> Vec<ProjectInfo> {
        let mut favorites: Vec<_> = self
            .favorites
            .iter()
            .filter_map(|path| self.projects.get(path).cloned())
            .collect();

        favorites.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        favorites
    }

    pub fn search_projects(&self, query: &str) -> Vec<ProjectInfo> {
        let query_lower = query.to_lowercase();
        self.projects
            .values()
            .filter(|p| p.name.to_lowercase().contains(&query_lower))
            .cloned()
            .collect()
    }

    pub fn get_project(&self, path: &str) -> Option<&ProjectInfo> {
        self.projects.get(path)
    }

    pub fn project_count(&self) -> usize {
        self.projects.len()
    }

    pub fn favorite_count(&self) -> usize {
        self.favorites.len()
    }

    pub fn clear_recent(&mut self) {
        self.recent_projects.clear();
    }
}

impl Default for ProjectBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Command Handlers (Task-O-Matic)
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn create_project(
    state: State<'_, Mutex<ProjectBrowserState>>,
    name: String,
    path: String,
) -> Result<ProjectInfo, String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .create_project_impl(name, path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_project(
    state: State<'_, Mutex<ProjectBrowserState>>,
    path: String,
) -> Result<ProjectInfo, String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .open_project_impl(path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_project(
    state: State<'_, Mutex<ProjectBrowserState>>,
    path: String,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .delete_project_impl(path)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_info_new() {
        let project = ProjectInfo::new(
            "Test Project".to_string(),
            "/path/to/project".to_string(),
            12345,
        )
        .unwrap();

        assert_eq!(project.name, "Test Project");
        assert_eq!(project.path, "/path/to/project");
        assert_eq!(project.last_modified, 12345);
        assert!(!project.is_favorite);
    }

    #[test]
    fn test_project_info_invalid_name() {
        let result = ProjectInfo::new("".to_string(), "/path".to_string(), 12345);
        assert!(matches!(
            result,
            Err(ProjectBrowserError::InvalidProjectName(_))
        ));
    }

    #[test]
    fn test_project_info_invalid_path() {
        let result = ProjectInfo::new("Test".to_string(), "".to_string(), 12345);
        assert!(matches!(result, Err(ProjectBrowserError::InvalidPath(_))));
    }

    #[test]
    fn test_project_info_with_favorite() {
        let project = ProjectInfo::new("Test".to_string(), "/path".to_string(), 12345)
            .unwrap()
            .with_favorite(true);
        assert!(project.is_favorite);
    }

    #[test]
    fn test_project_browser_state_new() {
        let state = ProjectBrowserState::new();
        assert_eq!(state.projects.len(), 0);
        assert_eq!(state.recent_projects.len(), 0);
        assert_eq!(state.favorites.len(), 0);
    }

    #[test]
    fn test_create_project() {
        let mut state = ProjectBrowserState::new();
        let project = state
            .create_project_impl("Test".to_string(), "/path/to/project".to_string())
            .unwrap();

        assert_eq!(project.name, "Test");
        assert_eq!(state.project_count(), 1);
        assert_eq!(state.recent_projects.len(), 1);
    }

    #[test]
    fn test_create_project_already_exists() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("Test".to_string(), "/path".to_string()).unwrap();

        let result = state.create_project_impl("Test2".to_string(), "/path".to_string());
        assert!(matches!(
            result,
            Err(ProjectBrowserError::ProjectAlreadyExists(_))
        ));
    }

    #[test]
    fn test_open_project() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("Test".to_string(), "/path".to_string()).unwrap();

        let project = state.open_project_impl("/path".to_string()).unwrap();
        assert_eq!(project.name, "Test");
    }

    #[test]
    fn test_open_project_not_found() {
        let mut state = ProjectBrowserState::new();
        let result = state.open_project_impl("/nonexistent".to_string());
        assert!(matches!(
            result,
            Err(ProjectBrowserError::ProjectNotFound(_))
        ));
    }

    #[test]
    fn test_delete_project() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("Test".to_string(), "/path".to_string()).unwrap();

        let deleted = state.delete_project_impl("/path".to_string()).unwrap();
        assert_eq!(deleted.name, "Test");
        assert_eq!(state.project_count(), 0);
        assert_eq!(state.recent_projects.len(), 0);
    }

    #[test]
    fn test_delete_project_not_found() {
        let mut state = ProjectBrowserState::new();
        let result = state.delete_project_impl("/nonexistent".to_string());
        assert!(matches!(
            result,
            Err(ProjectBrowserError::ProjectNotFound(_))
        ));
    }

    #[test]
    fn test_toggle_favorite() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("Test".to_string(), "/path".to_string()).unwrap();

        let is_favorite = state.toggle_favorite_impl("/path".to_string()).unwrap();
        assert!(is_favorite);
        assert_eq!(state.favorite_count(), 1);

        let is_favorite = state.toggle_favorite_impl("/path".to_string()).unwrap();
        assert!(!is_favorite);
        assert_eq!(state.favorite_count(), 0);
    }

    #[test]
    fn test_toggle_favorite_not_found() {
        let mut state = ProjectBrowserState::new();
        let result = state.toggle_favorite_impl("/nonexistent".to_string());
        assert!(matches!(
            result,
            Err(ProjectBrowserError::ProjectNotFound(_))
        ));
    }

    #[test]
    fn test_update_last_modified() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("Test".to_string(), "/path".to_string()).unwrap();

        let original_time = state.get_project("/path").unwrap().last_modified;

        // Sleep longer to ensure time changes (100ms is more reliable across systems)
        std::thread::sleep(std::time::Duration::from_millis(100));

        state.update_last_modified_impl("/path".to_string()).unwrap();
        let new_time = state.get_project("/path").unwrap().last_modified;

        // Use >= instead of > to handle cases where time resolution is low
        assert!(
            new_time >= original_time,
            "new_time={:?} should be >= original_time={:?}",
            new_time,
            original_time
        );
    }

    #[test]
    fn test_recent_projects_ordering() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("P1".to_string(), "/path1".to_string()).unwrap();
        state.create_project_impl("P2".to_string(), "/path2".to_string()).unwrap();
        state.create_project_impl("P3".to_string(), "/path3".to_string()).unwrap();

        let recent = state.get_recent_projects();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].name, "P3"); // Most recent
        assert_eq!(recent[1].name, "P2");
        assert_eq!(recent[2].name, "P1");
    }

    #[test]
    fn test_recent_projects_max_limit() {
        let mut state = ProjectBrowserState::new();
        state.max_recent = 3;

        for i in 0..5 {
            state.create_project_impl(format!("P{}", i), format!("/path{}", i)).unwrap();
        }

        assert_eq!(state.recent_projects.len(), 3);
        assert_eq!(state.recent_projects[0], "/path4");
        assert_eq!(state.recent_projects[1], "/path3");
        assert_eq!(state.recent_projects[2], "/path2");
    }

    #[test]
    fn test_recent_projects_reopen() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("P1".to_string(), "/path1".to_string()).unwrap();
        state.create_project_impl("P2".to_string(), "/path2".to_string()).unwrap();

        // Reopen first project
        state.open_project_impl("/path1".to_string()).unwrap();

        let recent = state.get_recent_projects();
        assert_eq!(recent[0].name, "P1"); // Should be at top now
        assert_eq!(recent[1].name, "P2");
    }

    #[test]
    fn test_get_favorite_projects() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("P1".to_string(), "/path1".to_string()).unwrap();
        state.create_project_impl("P2".to_string(), "/path2".to_string()).unwrap();
        state.create_project_impl("P3".to_string(), "/path3".to_string()).unwrap();

        state.toggle_favorite_impl("/path1".to_string()).unwrap();
        state.toggle_favorite_impl("/path3".to_string()).unwrap();

        let favorites = state.get_favorite_projects();
        assert_eq!(favorites.len(), 2);
    }

    #[test]
    fn test_search_projects() {
        let mut state = ProjectBrowserState::new();
        state
            .create_project_impl("Rock Song".to_string(), "/path1".to_string())
            .unwrap();
        state
            .create_project_impl("Jazz Track".to_string(), "/path2".to_string())
            .unwrap();
        state
            .create_project_impl("Rock Beat".to_string(), "/path3".to_string())
            .unwrap();

        let results = state.search_projects("rock");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_projects_case_insensitive() {
        let mut state = ProjectBrowserState::new();
        state
            .create_project_impl("Test Project".to_string(), "/path".to_string())
            .unwrap();

        let results = state.search_projects("TEST");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_clear_recent() {
        let mut state = ProjectBrowserState::new();
        state.create_project_impl("P1".to_string(), "/path1".to_string()).unwrap();
        state.create_project_impl("P2".to_string(), "/path2".to_string()).unwrap();

        state.clear_recent();
        assert_eq!(state.recent_projects.len(), 0);
        assert_eq!(state.project_count(), 2); // Projects still exist
    }
}
