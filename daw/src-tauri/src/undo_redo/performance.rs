//! Performance Optimization - Trusty Module
//!
//! Command compression and memory management for optimal performance.

use super::core::{Command, UndoRedoResult};

/// Command compressor for merging similar commands
pub struct CommandCompressor;

impl CommandCompressor {
    /// Try to compress two commands into one
    ///
    /// Returns true if compression was successful.
    pub fn compress(
        target: &mut Box<dyn Command>,
        source: &Box<dyn Command>,
    ) -> UndoRedoResult<bool> {
        if target.can_merge_with(source.as_ref()) {
            target.merge_with(source.as_ref())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if two commands can be compressed
    pub fn can_compress(cmd1: &Box<dyn Command>, cmd2: &Box<dyn Command>) -> bool {
        cmd1.can_merge_with(cmd2.as_ref())
    }
}

/// Memory limiter for command history
pub struct MemoryLimiter {
    max_memory: usize,
    current_memory: usize,
}

impl MemoryLimiter {
    /// Create a new memory limiter with max memory in bytes
    pub fn new(max_memory: usize) -> Self {
        Self {
            max_memory,
            current_memory: 0,
        }
    }

    /// Add command and update memory usage
    pub fn add_command(&mut self, cmd: &Box<dyn Command>) {
        self.current_memory += cmd.memory_usage();
    }

    /// Remove command and update memory usage
    pub fn remove_command(&mut self, cmd: &Box<dyn Command>) {
        self.current_memory = self.current_memory.saturating_sub(cmd.memory_usage());
    }

    /// Check if adding a command would exceed memory limit
    pub fn would_exceed(&self, cmd: &Box<dyn Command>) -> bool {
        self.current_memory + cmd.memory_usage() > self.max_memory
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.current_memory
    }

    /// Get memory limit
    pub fn max_usage(&self) -> usize {
        self.max_memory
    }

    /// Get percentage of memory used (0-100)
    pub fn usage_percentage(&self) -> f32 {
        if self.max_memory == 0 {
            return 0.0;
        }
        (self.current_memory as f32 / self.max_memory as f32) * 100.0
    }

    /// Reset memory counter
    pub fn reset(&mut self) {
        self.current_memory = 0;
    }

    /// Set new memory limit
    pub fn set_max_memory(&mut self, max_memory: usize) {
        self.max_memory = max_memory;
    }
}

/// Command batching for improved performance
pub struct CommandBatcher {
    batch: Vec<Box<dyn Command>>,
    max_batch_size: usize,
}

impl CommandBatcher {
    /// Create a new command batcher
    pub fn new(max_batch_size: usize) -> Self {
        Self {
            batch: Vec::new(),
            max_batch_size,
        }
    }

    /// Add command to batch
    pub fn add(&mut self, cmd: Box<dyn Command>) {
        self.batch.push(cmd);
    }

    /// Check if batch is full
    pub fn is_full(&self) -> bool {
        self.batch.len() >= self.max_batch_size
    }

    /// Get batch size
    pub fn size(&self) -> usize {
        self.batch.len()
    }

    /// Clear batch and return commands
    pub fn flush(&mut self) -> Vec<Box<dyn Command>> {
        std::mem::take(&mut self.batch)
    }

    /// Get batch without clearing
    pub fn peek(&self) -> &[Box<dyn Command>] {
        &self.batch
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::undo_redo::core::UndoRedoError;

    // Mock command for testing
    #[derive(Debug)]
    struct MockCommand {
        size: usize,
        can_merge: bool,
    }

    impl MockCommand {
        fn new(size: usize) -> Self {
            Self {
                size,
                can_merge: false,
            }
        }

        fn mergeable(size: usize) -> Self {
            Self {
                size,
                can_merge: true,
            }
        }
    }

    impl Command for MockCommand {
        fn execute(&mut self) -> UndoRedoResult<()> {
            Ok(())
        }

        fn undo(&mut self) -> UndoRedoResult<()> {
            Ok(())
        }

        fn description(&self) -> String {
            "Mock Command".to_string()
        }

        fn memory_usage(&self) -> usize {
            self.size
        }

        fn can_merge_with(&self, _other: &dyn Command) -> bool {
            self.can_merge
        }

        fn merge_with(&mut self, other: &dyn Command) -> UndoRedoResult<()> {
            if self.can_merge {
                self.size += other.memory_usage();
                Ok(())
            } else {
                Err(UndoRedoError::InvalidState(
                    "Cannot merge".to_string(),
                ))
            }
        }
    }

    #[test]
    fn test_memory_limiter_new() {
        let limiter = MemoryLimiter::new(1024);
        assert_eq!(limiter.max_usage(), 1024);
        assert_eq!(limiter.current_usage(), 0);
    }

    #[test]
    fn test_memory_limiter_add_command() {
        let mut limiter = MemoryLimiter::new(1024);
        let cmd: Box<dyn Command> = Box::new(MockCommand::new(100));

        limiter.add_command(&cmd);
        assert_eq!(limiter.current_usage(), 100);
    }

    #[test]
    fn test_memory_limiter_remove_command() {
        let mut limiter = MemoryLimiter::new(1024);
        let cmd: Box<dyn Command> = Box::new(MockCommand::new(100));

        limiter.add_command(&cmd);
        limiter.remove_command(&cmd);
        assert_eq!(limiter.current_usage(), 0);
    }

    #[test]
    fn test_memory_limiter_would_exceed() {
        let mut limiter = MemoryLimiter::new(100);
        let cmd1: Box<dyn Command> = Box::new(MockCommand::new(60));
        let cmd2: Box<dyn Command> = Box::new(MockCommand::new(50));

        limiter.add_command(&cmd1);
        assert!(limiter.would_exceed(&cmd2));
    }

    #[test]
    fn test_memory_limiter_usage_percentage() {
        let mut limiter = MemoryLimiter::new(1000);
        let cmd: Box<dyn Command> = Box::new(MockCommand::new(250));

        limiter.add_command(&cmd);
        assert_eq!(limiter.usage_percentage(), 25.0);
    }

    #[test]
    fn test_memory_limiter_reset() {
        let mut limiter = MemoryLimiter::new(1024);
        let cmd: Box<dyn Command> = Box::new(MockCommand::new(100));

        limiter.add_command(&cmd);
        limiter.reset();
        assert_eq!(limiter.current_usage(), 0);
    }

    #[test]
    fn test_memory_limiter_set_max_memory() {
        let mut limiter = MemoryLimiter::new(1024);

        limiter.set_max_memory(2048);
        assert_eq!(limiter.max_usage(), 2048);
    }

    #[test]
    fn test_command_compressor_can_compress() {
        let cmd1: Box<dyn Command> = Box::new(MockCommand::mergeable(100));
        let cmd2: Box<dyn Command> = Box::new(MockCommand::mergeable(50));

        assert!(CommandCompressor::can_compress(&cmd1, &cmd2));
    }

    #[test]
    fn test_command_compressor_cannot_compress() {
        let cmd1: Box<dyn Command> = Box::new(MockCommand::new(100));
        let cmd2: Box<dyn Command> = Box::new(MockCommand::new(50));

        assert!(!CommandCompressor::can_compress(&cmd1, &cmd2));
    }

    #[test]
    fn test_command_compressor_compress() {
        let mut cmd1: Box<dyn Command> = Box::new(MockCommand::mergeable(100));
        let cmd2: Box<dyn Command> = Box::new(MockCommand::mergeable(50));

        let result = CommandCompressor::compress(&mut cmd1, &cmd2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        assert_eq!(cmd1.memory_usage(), 150);
    }

    #[test]
    fn test_command_batcher_new() {
        let batcher = CommandBatcher::new(10);
        assert_eq!(batcher.size(), 0);
        assert!(!batcher.is_full());
    }

    #[test]
    fn test_command_batcher_add() {
        let mut batcher = CommandBatcher::new(10);
        let cmd: Box<dyn Command> = Box::new(MockCommand::new(100));

        batcher.add(cmd);
        assert_eq!(batcher.size(), 1);
    }

    #[test]
    fn test_command_batcher_is_full() {
        let mut batcher = CommandBatcher::new(2);
        let cmd1: Box<dyn Command> = Box::new(MockCommand::new(100));
        let cmd2: Box<dyn Command> = Box::new(MockCommand::new(50));

        batcher.add(cmd1);
        assert!(!batcher.is_full());

        batcher.add(cmd2);
        assert!(batcher.is_full());
    }

    #[test]
    fn test_command_batcher_flush() {
        let mut batcher = CommandBatcher::new(10);
        let cmd1: Box<dyn Command> = Box::new(MockCommand::new(100));
        let cmd2: Box<dyn Command> = Box::new(MockCommand::new(50));

        batcher.add(cmd1);
        batcher.add(cmd2);

        let flushed = batcher.flush();
        assert_eq!(flushed.len(), 2);
        assert_eq!(batcher.size(), 0);
    }

    #[test]
    fn test_command_batcher_peek() {
        let mut batcher = CommandBatcher::new(10);
        let cmd: Box<dyn Command> = Box::new(MockCommand::new(100));

        batcher.add(cmd);

        let peeked = batcher.peek();
        assert_eq!(peeked.len(), 1);
        assert_eq!(batcher.size(), 1); // Size unchanged
    }

    #[test]
    fn test_memory_limiter_multiple_commands() {
        let mut limiter = MemoryLimiter::new(1000);
        let cmd1: Box<dyn Command> = Box::new(MockCommand::new(200));
        let cmd2: Box<dyn Command> = Box::new(MockCommand::new(300));
        let cmd3: Box<dyn Command> = Box::new(MockCommand::new(400));

        limiter.add_command(&cmd1);
        limiter.add_command(&cmd2);
        assert_eq!(limiter.current_usage(), 500);

        limiter.add_command(&cmd3);
        assert_eq!(limiter.current_usage(), 900);

        limiter.remove_command(&cmd2);
        assert_eq!(limiter.current_usage(), 600);
    }

    #[test]
    fn test_memory_limiter_saturating_sub() {
        let mut limiter = MemoryLimiter::new(1000);
        let cmd: Box<dyn Command> = Box::new(MockCommand::new(100));

        // Remove without adding - should not underflow
        limiter.remove_command(&cmd);
        assert_eq!(limiter.current_usage(), 0);
    }

    #[test]
    fn test_memory_limiter_zero_max() {
        let limiter = MemoryLimiter::new(0);
        assert_eq!(limiter.usage_percentage(), 0.0);
    }
}
