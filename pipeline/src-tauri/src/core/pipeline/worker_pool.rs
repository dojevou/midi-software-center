// pipeline/src-tauri/src/core/pipeline/worker_pool.rs
//! Worker pool for parallel pipeline stage execution

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Worker pool for a single pipeline stage
pub struct WorkerPool {
    /// Number of worker threads
    worker_count: usize,

    /// Running flag (atomic for safe concurrent access)
    running: Arc<AtomicBool>,

    /// Total items processed by this pool
    processed_count: Arc<AtomicU64>,

    /// Worker thread handles
    handles: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    /// Create a new worker pool with specified number of workers
    pub fn new(worker_count: usize) -> Self {
        Self {
            worker_count,
            running: Arc::new(AtomicBool::new(false)),
            processed_count: Arc::new(AtomicU64::new(0)),
            handles: Vec::with_capacity(worker_count),
        }
    }

    /// Get number of workers in this pool
    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    /// Check if pool is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// Get total items processed
    pub fn processed_count(&self) -> u64 {
        self.processed_count.load(Ordering::Acquire)
    }

    /// Increment processed count
    pub fn increment_processed(&self) {
        self.processed_count.fetch_add(1, Ordering::AcqRel);
    }

    /// Start the worker pool
    pub fn start(&mut self) {
        self.running.store(true, Ordering::Release);
    }

    /// Stop the worker pool
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Add a worker task handle
    pub fn add_handle(&mut self, handle: JoinHandle<()>) {
        self.handles.push(handle);
    }

    /// Wait for all workers to complete
    pub async fn join_all(self) {
        for handle in self.handles {
            let _ = handle.await;
        }
    }

    /// Get clone of running flag for workers
    pub fn running_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.running)
    }

    /// Get clone of processed counter for workers
    pub fn processed_counter(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.processed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_pool_creation() {
        let pool = WorkerPool::new(16);
        assert_eq!(pool.worker_count(), 16);
        assert!(!pool.is_running());
        assert_eq!(pool.processed_count(), 0);
    }

    #[test]
    fn test_worker_pool_lifecycle() {
        let mut pool = WorkerPool::new(8);

        // Initially stopped
        assert!(!pool.is_running());

        // Start pool
        pool.start();
        assert!(pool.is_running());

        // Stop pool
        pool.stop();
        assert!(!pool.is_running());
    }

    #[test]
    fn test_processed_counter() {
        let pool = WorkerPool::new(4);

        assert_eq!(pool.processed_count(), 0);
        pool.increment_processed();
        assert_eq!(pool.processed_count(), 1);
        pool.increment_processed();
        pool.increment_processed();
        assert_eq!(pool.processed_count(), 3);
    }

    #[tokio::test]
    async fn test_worker_pool_with_tasks() {
        let mut pool = WorkerPool::new(4);
        pool.start();

        let running = pool.running_flag();
        let counter = pool.processed_counter();

        // Spawn 4 workers that increment counter
        for _ in 0..4 {
            let running = Arc::clone(&running);
            let counter = Arc::clone(&counter);

            let handle = tokio::spawn(async move {
                while running.load(Ordering::Acquire) {
                    counter.fetch_add(1, Ordering::AcqRel);
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            });

            pool.add_handle(handle);
        }

        // Let workers run briefly
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Stop workers
        pool.stop();

        // Wait for completion
        pool.join_all().await;

        // Should have processed some items
        assert!(counter.load(Ordering::Acquire) > 0);
    }
}
