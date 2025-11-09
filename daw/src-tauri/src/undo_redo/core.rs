   /// Core Undo/Redo System - Trusty Module
   ///
   /// Pure command pattern implementation with no I/O or side effects.
   /// Provides Command trait and CommandHistory for managing undo/redo operations.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

/// Errors that can occur during undo/redo operations
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UndoRedoError {
    #[error("No commands available to undo")]
    NothingToUndo,
    #[error("No commands available to redo")]
    NothingToRedo,
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Undo operation failed: {0}")]
    UndoFailed(String),
    #[error("Redo operation failed: {0}")]
    RedoFailed(String),
    #[error("Command history is full (max depth: {0})")]
    HistoryFull(usize),
    #[error("Invalid command state: {0}")]
    InvalidState(String),
}

/// Result type for undo/redo operations
pub type UndoRedoResult<T> = Result<T, UndoRedoError>;

/// Command trait for undo/redo operations
///
/// All editor commands must implement this trait to support undo/redo.
/// Commands should be pure data transformations without side effects.
pub trait Command: std::fmt::Debug + Send + Sync {
    /// Execute the command (apply the change)
    ///
    /// This method should modify the target state and return Ok(())
    /// on success, or an error if the operation cannot be completed.
    fn execute(&mut self) -> UndoRedoResult<()>;

    /// Undo the command (reverse the change)
    ///
    /// This method should restore the state to what it was before execute()
    /// was called. It must be the exact inverse of execute().
    fn undo(&mut self) -> UndoRedoResult<()>;

    /// Redo the command (reapply the change)
    ///
    /// This method should reapply the change after an undo.
    /// Default implementation calls execute() again.
    fn redo(&mut self) -> UndoRedoResult<()> {
        self.execute()
    }

    /// Get human-readable description of the command
    ///
    /// This is displayed in the UI for undo/redo menu items.
    /// Use imperative form: "Add Note", "Delete Track", etc.
    fn description(&self) -> String;

    /// Get the estimated memory usage of this command in bytes
    ///
    /// Used for memory limiting. Default implementation returns 0.
    fn memory_usage(&self) -> usize {
        0
    }

    /// Check if this command can be merged with another command
    ///
    /// Used for command compression. Default implementation returns false.
    fn can_merge_with(&self, _other: &dyn Command) -> bool {
        false
    }

    /// Merge this command with another command
    ///
    /// Used for command compression. Default implementation does nothing.
    fn merge_with(&mut self, _other: &dyn Command) -> UndoRedoResult<()> {
        Err(UndoRedoError::InvalidState(
            "Command does not support merging".to_string(),
        ))
    }
}

/// Command history manager
///
/// Manages undo and redo stacks with configurable depth limits.
/// Trusty Module: Pure logic, no I/O, fully testable.
#[derive(Debug)]
pub struct CommandHistory {
    /// Stack of commands that can be undone
    undo_stack: VecDeque<Box<dyn Command>>,
    /// Stack of commands that can be redone
    redo_stack: VecDeque<Box<dyn Command>>,
    /// Maximum number of commands to keep in history
    max_depth: usize,
    /// Current memory usage in bytes
    current_memory: usize,
    /// Maximum memory usage in bytes
    max_memory: usize,
    /// Whether compression is enabled
    compression_enabled: bool,
}

impl CommandHistory {
    /// Create a new command history with default settings
    pub fn new() -> Self {
        Self::with_max_depth(crate::undo_redo::DEFAULT_MAX_DEPTH)
    }

    /// Create a new command history with custom max depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_depth,
            current_memory: 0,
            max_memory: crate::undo_redo::DEFAULT_MAX_MEMORY,
            compression_enabled: false,
        }
    }

    /// Create a new command history with custom memory limit
    pub fn with_memory_limit(max_memory: usize) -> Self {
        let mut history = Self::new();
        history.max_memory = max_memory;
        history
    }

    /// Enable or disable command compression
    pub fn set_compression(&mut self, enabled: bool) {
        self.compression_enabled = enabled;
    }

    /// Execute a command and add it to the undo stack
    ///
    /// This clears the redo stack since we're creating a new timeline.
    pub fn execute_command(&mut self, mut command: Box<dyn Command>) -> UndoRedoResult<()> {
        // Execute the command
        command.execute()?;

        // Clear redo stack (we're on a new timeline)
        self.clear_redo();

        // Try to merge with previous command if compression is enabled
        if self.compression_enabled {
            if let Some(last) = self.undo_stack.back_mut() {
                if last.can_merge_with(command.as_ref()) {
                    last.merge_with(command.as_ref())?;
                    return Ok(());
                }
            }
        }

        // Update memory usage
        let command_memory = command.memory_usage();
        self.current_memory += command_memory;

        // Add command to undo stack
        self.undo_stack.push_back(command);

        // Enforce memory limit
        self.enforce_memory_limit();

        // Enforce depth limit
        if self.undo_stack.len() > self.max_depth {
            if let Some(removed) = self.undo_stack.pop_front() {
                self.current_memory = self.current_memory.saturating_sub(removed.memory_usage());
            }
        }

        Ok(())
    }

    /// Undo the last command
    ///
    /// Moves the command from the undo stack to the redo stack.
    pub fn undo(&mut self) -> UndoRedoResult<String> {
        let mut command = self
            .undo_stack
            .pop_back()
            .ok_or(UndoRedoError::NothingToUndo)?;

        let description = command.description();

        if let Err(e) = command.undo() {
            // Put command back on undo stack if undo fails
            self.undo_stack.push_back(command);
            return Err(e);
        }

        self.redo_stack.push_back(command);

        Ok(description)
    }

    /// Redo the last undone command
    ///
    /// Moves the command from the redo stack to the undo stack.
    pub fn redo(&mut self) -> UndoRedoResult<String> {
        let mut command = self
            .redo_stack
            .pop_back()
            .ok_or(UndoRedoError::NothingToRedo)?;

        let description = command.description();

        if let Err(e) = command.redo() {
            // Put command back on redo stack if redo fails
            self.redo_stack.push_back(command);
            return Err(e);
        }

        self.undo_stack.push_back(command);

        Ok(description)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get description of the next command to undo
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.back().map(|cmd| cmd.description())
    }

    /// Get description of the next command to redo
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }

    /// Clear all undo history
    pub fn clear_undo(&mut self) {
        self.current_memory = self
            .current_memory
            .saturating_sub(self.undo_stack.iter().map(|c| c.memory_usage()).sum());
        self.undo_stack.clear();
    }

    /// Clear all redo history
    pub fn clear_redo(&mut self) {
        self.current_memory = self
            .current_memory
            .saturating_sub(self.redo_stack.iter().map(|c| c.memory_usage()).sum());
        self.redo_stack.clear();
    }

    /// Clear both undo and redo history
    pub fn clear(&mut self) {
        self.clear_undo();
        self.clear_redo();
    }

    /// Get the number of commands in the undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of commands in the redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Get maximum depth
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Set maximum depth
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
        // Enforce new limit
        while self.undo_stack.len() > self.max_depth {
            if let Some(removed) = self.undo_stack.pop_front() {
                self.current_memory = self.current_memory.saturating_sub(removed.memory_usage());
            }
        }
    }

    /// Get maximum memory limit
    pub fn max_memory(&self) -> usize {
        self.max_memory
    }

    /// Set maximum memory limit
    pub fn set_max_memory(&mut self, max_memory: usize) {
        self.max_memory = max_memory;
        self.enforce_memory_limit();
    }

    /// Enforce memory limit by removing oldest commands
    fn enforce_memory_limit(&mut self) {
        while self.current_memory > self.max_memory && !self.undo_stack.is_empty() {
            if let Some(removed) = self.undo_stack.pop_front() {
                self.current_memory = self.current_memory.saturating_sub(removed.memory_usage());
            }
        }
    }

    /// Get all undo descriptions (newest first)
    pub fn undo_descriptions(&self) -> Vec<String> {
        self.undo_stack
            .iter()
            .rev()
            .map(|cmd| cmd.description())
            .collect()
    }

    /// Get all redo descriptions (newest first)
    pub fn redo_descriptions(&self) -> Vec<String> {
        self.redo_stack
            .iter()
            .rev()
            .map(|cmd| cmd.description())
            .collect()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock command for testing
    #[derive(Debug)]
    struct MockCommand {
        value: i32,
        target: std::sync::Arc<std::sync::Mutex<i32>>,
        old_value: Option<i32>,
        description: String,
        memory: usize,
        should_fail_execute: bool,
        should_fail_undo: bool,
    }

    impl MockCommand {
        fn new(value: i32, target: std::sync::Arc<std::sync::Mutex<i32>>) -> Self {
            Self {
                value,
                target,
                old_value: None,
                description: format!("Set to {}", value),
                memory: 32,
                should_fail_execute: false,
                should_fail_undo: false,
            }
        }

        fn with_memory(mut self, memory: usize) -> Self {
            self.memory = memory;
            self
        }

        fn with_fail_execute(mut self) -> Self {
            self.should_fail_execute = true;
            self
        }

        fn with_fail_undo(mut self) -> Self {
            self.should_fail_undo = true;
            self
        }
    }

    impl Command for MockCommand {
        fn execute(&mut self) -> UndoRedoResult<()> {
            if self.should_fail_execute {
                return Err(UndoRedoError::ExecutionFailed("Mock failure".to_string()));
            }
            let mut target = self.target.lock().unwrap();
            self.old_value = Some(*target);
            *target = self.value;
            Ok(())
        }

        fn undo(&mut self) -> UndoRedoResult<()> {
            if self.should_fail_undo {
                return Err(UndoRedoError::UndoFailed("Mock undo failure".to_string()));
            }
            let mut target = self.target.lock().unwrap();
            if let Some(old) = self.old_value {
                *target = old;
            }
            Ok(())
        }

        fn description(&self) -> String {
            self.description.clone()
        }

        fn memory_usage(&self) -> usize {
            self.memory
        }
    }

    #[test]
    fn test_command_history_new() {
        let history = CommandHistory::new();
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_execute_command() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        let cmd = MockCommand::new(42, target.clone());
        history.execute_command(Box::new(cmd)).unwrap();

        assert_eq!(*target.lock().unwrap(), 42);
        assert_eq!(history.undo_count(), 1);
        assert!(history.can_undo());
    }

    #[test]
    fn test_undo_command() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        let cmd = MockCommand::new(42, target.clone());
        history.execute_command(Box::new(cmd)).unwrap();

        let description = history.undo().unwrap();
        assert_eq!(description, "Set to 42");
        assert_eq!(*target.lock().unwrap(), 0);
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 1);
    }

    #[test]
    fn test_redo_command() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        let cmd = MockCommand::new(42, target.clone());
        history.execute_command(Box::new(cmd)).unwrap();
        history.undo().unwrap();

        let description = history.redo().unwrap();
        assert_eq!(description, "Set to 42");
        assert_eq!(*target.lock().unwrap(), 42);
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_undo_nothing_to_undo() {
        let mut history = CommandHistory::new();
        let result = history.undo();
        assert!(matches!(result, Err(UndoRedoError::NothingToUndo)));
    }

    #[test]
    fn test_redo_nothing_to_redo() {
        let mut history = CommandHistory::new();
        let result = history.redo();
        assert!(matches!(result, Err(UndoRedoError::NothingToRedo)));
    }

    #[test]
    fn test_execute_clears_redo_stack() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        // Execute, undo, then execute again
        history
            .execute_command(Box::new(MockCommand::new(42, target.clone())))
            .unwrap();
        history.undo().unwrap();
        assert!(history.can_redo());

        history
            .execute_command(Box::new(MockCommand::new(99, target.clone())))
            .unwrap();
        assert!(!history.can_redo());
    }

    #[test]
    fn test_max_depth_limit() {
        let mut history = CommandHistory::with_max_depth(3);
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        // Add 5 commands
        for i in 1..=5 {
            history
                .execute_command(Box::new(MockCommand::new(i, target.clone())))
                .unwrap();
        }

        // Should only keep last 3
        assert_eq!(history.undo_count(), 3);

        // Undo all 3
        history.undo().unwrap(); // Set to 5
        history.undo().unwrap(); // Set to 4
        history.undo().unwrap(); // Set to 3

        // No more undo available
        assert!(!history.can_undo());
    }

    #[test]
    fn test_clear_undo() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone())))
            .unwrap();
        history.clear_undo();

        assert!(!history.can_undo());
        assert_eq!(history.undo_count(), 0);
    }

    #[test]
    fn test_clear_redo() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone())))
            .unwrap();
        history.undo().unwrap();
        history.clear_redo();

        assert!(!history.can_redo());
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_clear_all() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone())))
            .unwrap();
        history.undo().unwrap();
        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_undo_description() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        assert_eq!(history.undo_description(), None);

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone())))
            .unwrap();
        assert_eq!(history.undo_description(), Some("Set to 42".to_string()));
    }

    #[test]
    fn test_redo_description() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        assert_eq!(history.redo_description(), None);

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone())))
            .unwrap();
        history.undo().unwrap();
        assert_eq!(history.redo_description(), Some("Set to 42".to_string()));
    }

    #[test]
    fn test_memory_tracking() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        assert_eq!(history.memory_usage(), 0);

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone()).with_memory(100)))
            .unwrap();
        assert_eq!(history.memory_usage(), 100);

        history
            .execute_command(Box::new(MockCommand::new(99, target.clone()).with_memory(50)))
            .unwrap();
        assert_eq!(history.memory_usage(), 150);

        history.undo().unwrap();
        assert_eq!(history.memory_usage(), 150); // Command moves to redo stack

        history.clear();
        assert_eq!(history.memory_usage(), 0);
    }

    #[test]
    fn test_memory_limit() {
        let mut history = CommandHistory::with_memory_limit(100);
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        // Add commands totaling 150 bytes
        history
            .execute_command(Box::new(MockCommand::new(1, target.clone()).with_memory(50)))
            .unwrap();
        history
            .execute_command(Box::new(MockCommand::new(2, target.clone()).with_memory(50)))
            .unwrap();
        history
            .execute_command(Box::new(MockCommand::new(3, target.clone()).with_memory(50)))
            .unwrap();

        // Should have dropped oldest command
        assert!(history.memory_usage() <= 100);
        assert!(history.undo_count() <= 2);
    }

    #[test]
    fn test_set_max_depth() {
        let mut history = CommandHistory::with_max_depth(5);
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        for i in 1..=5 {
            history
                .execute_command(Box::new(MockCommand::new(i, target.clone())))
                .unwrap();
        }
        assert_eq!(history.undo_count(), 5);

        // Reduce max depth
        history.set_max_depth(3);
        assert_eq!(history.undo_count(), 3);
    }

    #[test]
    fn test_set_max_memory() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        for i in 1..=5 {
            history
                .execute_command(Box::new(MockCommand::new(i, target.clone()).with_memory(50)))
                .unwrap();
        }
        assert_eq!(history.memory_usage(), 250);

        // Reduce max memory
        history.set_max_memory(100);
        assert!(history.memory_usage() <= 100);
    }

    #[test]
    fn test_multiple_undo_redo() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        // Execute 3 commands
        history
            .execute_command(Box::new(MockCommand::new(1, target.clone())))
            .unwrap();
        history
            .execute_command(Box::new(MockCommand::new(2, target.clone())))
            .unwrap();
        history
            .execute_command(Box::new(MockCommand::new(3, target.clone())))
            .unwrap();

        assert_eq!(*target.lock().unwrap(), 3);

        // Undo 2 times
        history.undo().unwrap();
        history.undo().unwrap();
        assert_eq!(*target.lock().unwrap(), 1);

        // Redo 1 time
        history.redo().unwrap();
        assert_eq!(*target.lock().unwrap(), 2);

        // Undo 2 times
        history.undo().unwrap();
        history.undo().unwrap();
        assert_eq!(*target.lock().unwrap(), 0);
    }

    #[test]
    fn test_undo_descriptions() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        history
            .execute_command(Box::new(MockCommand::new(1, target.clone())))
            .unwrap();
        history
            .execute_command(Box::new(MockCommand::new(2, target.clone())))
            .unwrap();
        history
            .execute_command(Box::new(MockCommand::new(3, target.clone())))
            .unwrap();

        let descriptions = history.undo_descriptions();
        assert_eq!(descriptions.len(), 3);
        assert_eq!(descriptions[0], "Set to 3");
        assert_eq!(descriptions[1], "Set to 2");
        assert_eq!(descriptions[2], "Set to 1");
    }

    #[test]
    fn test_redo_descriptions() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        history
            .execute_command(Box::new(MockCommand::new(1, target.clone())))
            .unwrap();
        history
            .execute_command(Box::new(MockCommand::new(2, target.clone())))
            .unwrap();
        history.undo().unwrap();
        history.undo().unwrap();

        let descriptions = history.redo_descriptions();
        assert_eq!(descriptions.len(), 2);
        assert_eq!(descriptions[0], "Set to 1");
        assert_eq!(descriptions[1], "Set to 2");
    }

    #[test]
    fn test_failed_execute_not_added() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        let result = history.execute_command(Box::new(MockCommand::new(42, target.clone()).with_fail_execute()));
        assert!(result.is_err());
        assert_eq!(history.undo_count(), 0);
    }

    #[test]
    fn test_failed_undo_keeps_command() {
        let mut history = CommandHistory::new();
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone()).with_fail_undo()))
            .unwrap();

        let result = history.undo();
        assert!(result.is_err());
        // Command should still be on undo stack
        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn test_command_history_with_max_depth_zero() {
        let mut history = CommandHistory::with_max_depth(0);
        let target = std::sync::Arc::new(std::sync::Mutex::new(0));

        history
            .execute_command(Box::new(MockCommand::new(42, target.clone())))
            .unwrap();

        // Should not store any commands
        assert_eq!(history.undo_count(), 0);
        assert!(!history.can_undo());
    }
}
