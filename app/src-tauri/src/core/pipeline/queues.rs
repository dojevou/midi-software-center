// pipeline/src-tauri/src/core/pipeline/queues.rs
//! Lock-free MPMC queues for pipeline stage communication

use crossbeam_queue::ArrayQueue;
use std::sync::Arc;

/// Queue capacity - balance between memory usage and throughput
pub const QUEUE_CAPACITY: usize = 10_000;

/// File record passed between pipeline stages
#[derive(Debug, Clone)]
pub struct FileRecord {
    pub id: i64,
    pub filepath: String,
    pub filename: String,
    pub parent_folder: Option<String>,
    pub is_multi_track: bool,
    pub analyzed: bool,
}

/// Lock-free MPMC queues connecting all pipeline stages
#[derive(Clone)]
pub struct PipelineQueues {
    /// Stage 1 → Stage 2: Import → Sanitize
    pub import_to_sanitize: Arc<ArrayQueue<FileRecord>>,

    /// Stage 2 → Stage 3: Sanitize → Split
    pub sanitize_to_split: Arc<ArrayQueue<FileRecord>>,

    /// Stage 3 → Stage 4: Split → Analyze
    pub split_to_analyze: Arc<ArrayQueue<FileRecord>>,

    /// Stage 4 → Stage 5: Analyze → Rename (optional)
    pub analyze_to_rename: Arc<ArrayQueue<FileRecord>>,

    /// Stage 5 → Stage 6: Rename → Export (or Analyze → Export if skip rename)
    pub rename_to_export: Arc<ArrayQueue<FileRecord>>,
}

impl PipelineQueues {
    /// Create new pipeline queues with default capacity
    pub fn new() -> Self {
        Self::with_capacity(QUEUE_CAPACITY)
    }

    /// Create pipeline queues with custom capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            import_to_sanitize: Arc::new(ArrayQueue::new(capacity)),
            sanitize_to_split: Arc::new(ArrayQueue::new(capacity)),
            split_to_analyze: Arc::new(ArrayQueue::new(capacity)),
            analyze_to_rename: Arc::new(ArrayQueue::new(capacity)),
            rename_to_export: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    /// Get total number of items across all queues (for progress tracking)
    pub fn total_queued(&self) -> usize {
        self.import_to_sanitize.len()
            + self.sanitize_to_split.len()
            + self.split_to_analyze.len()
            + self.analyze_to_rename.len()
            + self.rename_to_export.len()
    }

    /// Check if all queues are empty (pipeline drained)
    pub fn is_empty(&self) -> bool {
        self.import_to_sanitize.is_empty()
            && self.sanitize_to_split.is_empty()
            && self.split_to_analyze.is_empty()
            && self.analyze_to_rename.is_empty()
            && self.rename_to_export.is_empty()
    }
}

impl Default for PipelineQueues {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queues_creation() {
        let queues = PipelineQueues::new();
        assert_eq!(queues.total_queued(), 0);
        assert!(queues.is_empty());
    }

    #[test]
    fn test_custom_capacity() {
        let queues = PipelineQueues::with_capacity(5000);
        assert_eq!(queues.import_to_sanitize.capacity(), 5000);
    }

    #[test]
    fn test_queue_operations() {
        let queues = PipelineQueues::new();

        let record = FileRecord {
            id: 1,
            filepath: "/test/file.mid".to_string(),
            filename: "file.mid".to_string(),
            parent_folder: Some("/test".to_string()),
            is_multi_track: false,
            analyzed: false,
        };

        // Push to first queue
        assert!(queues.import_to_sanitize.push(record.clone()).is_ok());
        assert_eq!(queues.total_queued(), 1);
        assert!(!queues.is_empty());

        // Pop from first queue
        let popped = queues.import_to_sanitize.pop();
        assert!(popped.is_some());
        assert_eq!(queues.total_queued(), 0);
        assert!(queues.is_empty());
    }
}
