/// Tauri Command Handlers - Task-O-Matic
///
/// Entry points for undo/redo operations exposed to frontend.
use super::core::CommandHistory;
use std::sync::Mutex;
use tauri::State;

/// Undo/Redo state managed by Tauri
pub struct UndoRedoState {
    history: Mutex<CommandHistory>,
}

impl UndoRedoState {
    pub fn new() -> Self {
        Self { history: Mutex::new(CommandHistory::new()) }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self { history: Mutex::new(CommandHistory::with_max_depth(max_depth)) }
    }
}

impl Default for UndoRedoState {
    fn default() -> Self {
        Self::new()
    }
}

/// Undo the last command
#[tauri::command]
pub async fn undo(state: State<'_, UndoRedoState>) -> Result<String, String> {
    let mut history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    history.undo().map_err(|e| e.to_string())
}

/// Redo the last undone command
#[tauri::command]
pub async fn redo(state: State<'_, UndoRedoState>) -> Result<String, String> {
    let mut history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    history.redo().map_err(|e| e.to_string())
}

/// Check if undo is available
#[tauri::command]
pub async fn can_undo(state: State<'_, UndoRedoState>) -> Result<bool, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.can_undo())
}

/// Check if redo is available
#[tauri::command]
pub async fn can_redo(state: State<'_, UndoRedoState>) -> Result<bool, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.can_redo())
}

/// Get description of next command to undo
#[tauri::command]
pub async fn undo_description(state: State<'_, UndoRedoState>) -> Result<Option<String>, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.undo_description())
}

/// Get description of next command to redo
#[tauri::command]
pub async fn redo_description(state: State<'_, UndoRedoState>) -> Result<Option<String>, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.redo_description())
}

/// Clear all undo/redo history
#[tauri::command]
pub async fn clear_history(state: State<'_, UndoRedoState>) -> Result<(), String> {
    let mut history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    history.clear();
    Ok(())
}

/// Get number of commands in undo stack
#[tauri::command]
pub async fn undo_count(state: State<'_, UndoRedoState>) -> Result<usize, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.undo_count())
}

/// Get number of commands in redo stack
#[tauri::command]
pub async fn redo_count(state: State<'_, UndoRedoState>) -> Result<usize, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.redo_count())
}

/// Get all undo descriptions
#[tauri::command]
pub async fn undo_descriptions(state: State<'_, UndoRedoState>) -> Result<Vec<String>, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.undo_descriptions())
}

/// Get all redo descriptions
#[tauri::command]
pub async fn redo_descriptions(state: State<'_, UndoRedoState>) -> Result<Vec<String>, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.redo_descriptions())
}

/// Get current memory usage
#[tauri::command]
pub async fn memory_usage(state: State<'_, UndoRedoState>) -> Result<usize, String> {
    let history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(history.memory_usage())
}

/// Set maximum depth
#[tauri::command]
pub async fn set_max_depth(
    state: State<'_, UndoRedoState>,
    max_depth: usize,
) -> Result<(), String> {
    let mut history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    history.set_max_depth(max_depth);
    Ok(())
}

/// Set maximum memory
#[tauri::command]
pub async fn set_max_memory(
    state: State<'_, UndoRedoState>,
    max_memory: usize,
) -> Result<(), String> {
    let mut history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    history.set_max_memory(max_memory);
    Ok(())
}

/// Enable or disable command compression
#[tauri::command]
pub async fn set_compression(state: State<'_, UndoRedoState>, enabled: bool) -> Result<(), String> {
    let mut history = state.history.lock().map_err(|e| format!("Lock error: {}", e))?;

    history.set_compression(enabled);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo_state_new() {
        let state = UndoRedoState::new();
        let history = state.history.lock().unwrap();
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_undo_redo_state_with_max_depth() {
        let state = UndoRedoState::with_max_depth(25);
        let history = state.history.lock().unwrap();
        assert_eq!(history.max_depth(), 25);
    }

    #[test]
    fn test_undo_redo_state_default() {
        let state = UndoRedoState::default();
        let history = state.history.lock().unwrap();
        assert_eq!(history.max_depth(), crate::undo_redo::DEFAULT_MAX_DEPTH);
    }

    #[test]
    fn test_state_can_undo_empty() {
        let state = UndoRedoState::new();
        let history = state.history.lock().unwrap();
        assert!(!history.can_undo());
    }

    #[test]
    fn test_state_can_redo_empty() {
        let state = UndoRedoState::new();
        let history = state.history.lock().unwrap();
        assert!(!history.can_redo());
    }

    #[test]
    fn test_state_descriptions_empty() {
        let state = UndoRedoState::new();
        let history = state.history.lock().unwrap();
        assert_eq!(history.undo_description(), None);
        assert_eq!(history.redo_description(), None);
    }

    #[test]
    fn test_state_memory_usage() {
        let state = UndoRedoState::new();
        let history = state.history.lock().unwrap();
        assert_eq!(history.memory_usage(), 0);
    }

    #[test]
    fn test_state_clear() {
        let state = UndoRedoState::new();
        let mut history = state.history.lock().unwrap();
        history.clear();
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_state_set_max_depth() {
        let state = UndoRedoState::new();
        let mut history = state.history.lock().unwrap();
        history.set_max_depth(50);
        assert_eq!(history.max_depth(), 50);
    }

    #[test]
    fn test_state_set_max_memory() {
        let state = UndoRedoState::new();
        let mut history = state.history.lock().unwrap();
        history.set_max_memory(5000);
        assert_eq!(history.max_memory(), 5000);
    }

    #[test]
    fn test_state_compression() {
        let state = UndoRedoState::new();
        let mut history = state.history.lock().unwrap();
        history.set_compression(true);
        // No direct way to verify, but should not error
    }
}
