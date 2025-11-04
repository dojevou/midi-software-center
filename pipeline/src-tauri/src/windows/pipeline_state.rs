//! Pipeline Window State
//!
//! Trusty Module: Pure data structures for pipeline processing state including
//! processing status, progress tracking, and statistics. No I/O, no side effects.

use serde::{Deserialize, Serialize};

/// Processing status for pipeline operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessingStatus {
    /// Pipeline is idle, no operations running
    Idle,
    /// Pipeline is actively processing files
    Processing,
    /// Pipeline is paused (can be resumed)
    Paused,
    /// Pipeline has completed processing
    Complete,
    /// Pipeline encountered an error
    Error,
}

impl Default for ProcessingStatus {
    fn default() -> Self {
        ProcessingStatus::Idle
    }
}

impl ProcessingStatus {
    /// Check if pipeline is currently active (processing or paused)
    pub fn is_active(&self) -> bool {
        matches!(self, ProcessingStatus::Processing | ProcessingStatus::Paused)
    }

    /// Check if can be paused
    pub fn can_pause(&self) -> bool {
        *self == ProcessingStatus::Processing
    }

    /// Check if can be resumed
    pub fn can_resume(&self) -> bool {
        *self == ProcessingStatus::Paused
    }

    /// Check if can start new operation
    pub fn can_start(&self) -> bool {
        matches!(
            self,
            ProcessingStatus::Idle | ProcessingStatus::Complete | ProcessingStatus::Error
        )
    }
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    /// Total files to process
    pub total_files: usize,
    /// Files successfully processed
    pub processed_count: usize,
    /// Files skipped (duplicates, already processed)
    pub skipped_count: usize,
    /// Files with errors
    pub error_count: usize,
    /// Currently processing file name
    pub current_file_name: String,
    /// Processing start time (unix timestamp)
    pub start_time: Option<u64>,
    /// Processing end time (unix timestamp)
    pub end_time: Option<u64>,
    /// Estimated time remaining in seconds
    pub estimated_time_remaining: Option<f32>,
}

impl Default for ProcessingStats {
    fn default() -> Self {
        ProcessingStats {
            total_files: 0,
            processed_count: 0,
            skipped_count: 0,
            error_count: 0,
            current_file_name: String::new(),
            start_time: None,
            end_time: None,
            estimated_time_remaining: None,
        }
    }
}

impl ProcessingStats {
    /// Create new processing stats
    pub fn new(total_files: usize) -> Self {
        ProcessingStats {
            total_files,
            ..Default::default()
        }
    }

    /// Calculate progress percentage (0.0 to 100.0)
    pub fn progress_percentage(&self) -> f32 {
        if self.total_files == 0 {
            return 0.0;
        }
        let completed = self.processed_count + self.skipped_count + self.error_count;
        (completed as f32 / self.total_files as f32) * 100.0
    }

    /// Get files remaining to process
    pub fn files_remaining(&self) -> usize {
        let completed = self.processed_count + self.skipped_count + self.error_count;
        self.total_files.saturating_sub(completed)
    }

    /// Calculate success rate percentage
    pub fn success_rate(&self) -> f32 {
        let total_processed = self.processed_count + self.error_count;
        if total_processed == 0 {
            return 100.0;
        }
        (self.processed_count as f32 / total_processed as f32) * 100.0
    }

    /// Calculate elapsed time in seconds
    pub fn elapsed_time(&self) -> Option<u64> {
        let start = self.start_time?;
        let end = self
            .end_time
            .or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs())
            })?;
        Some(end.saturating_sub(start))
    }

    /// Calculate average processing time per file in seconds
    pub fn average_time_per_file(&self) -> Option<f32> {
        if self.processed_count == 0 {
            return None;
        }
        let elapsed = self.elapsed_time()? as f32;
        Some(elapsed / self.processed_count as f32)
    }

    /// Update estimated time remaining
    pub fn update_eta(&mut self) {
        if let Some(avg_time) = self.average_time_per_file() {
            let remaining = self.files_remaining();
            self.estimated_time_remaining = Some(avg_time * remaining as f32);
        }
    }

    /// Check if processing is complete
    pub fn is_complete(&self) -> bool {
        self.files_remaining() == 0
    }

    /// Check if has errors
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

/// Operation type for pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    /// Single file import
    SingleFileImport,
    /// Directory import (batch)
    DirectoryImport,
    /// Archive extraction and import
    ArchiveImport,
    /// File analysis (BPM, key detection)
    FileAnalysis,
    /// Batch file analysis
    BatchAnalysis,
    /// File export/extraction
    FileExport,
    /// Database cleanup/maintenance
    DatabaseMaintenance,
}

impl Default for OperationType {
    fn default() -> Self {
        OperationType::SingleFileImport
    }
}

/// Pipeline window state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineWindowState {
    /// Current processing status
    pub status: ProcessingStatus,
    /// Current operation type
    pub operation_type: OperationType,
    /// Processing statistics
    pub stats: ProcessingStats,
    /// Processing log messages (limited to last 100)
    pub log_messages: Vec<LogMessage>,
    /// Show detailed statistics
    pub show_details: bool,
    /// Auto-scroll log
    pub auto_scroll: bool,
}

impl Default for PipelineWindowState {
    fn default() -> Self {
        PipelineWindowState {
            status: ProcessingStatus::Idle,
            operation_type: OperationType::SingleFileImport,
            stats: ProcessingStats::default(),
            log_messages: Vec::new(),
            show_details: true,
            auto_scroll: true,
        }
    }
}

impl PipelineWindowState {
    /// Create new pipeline window state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start new processing operation
    pub fn start_operation(&mut self, operation_type: OperationType, total_files: usize) {
        self.status = ProcessingStatus::Processing;
        self.operation_type = operation_type;
        self.stats = ProcessingStats::new(total_files);
        self.stats.start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());
        self.log_messages.clear();
        self.add_log(LogLevel::Info, format!("Started {:?}", operation_type));
    }

    /// Pause processing
    pub fn pause(&mut self) -> Result<(), String> {
        if !self.status.can_pause() {
            return Err(format!("Cannot pause when status is {:?}", self.status));
        }
        self.status = ProcessingStatus::Paused;
        self.add_log(LogLevel::Info, "Processing paused".to_string());
        Ok(())
    }

    /// Resume processing
    pub fn resume(&mut self) -> Result<(), String> {
        if !self.status.can_resume() {
            return Err(format!("Cannot resume when status is {:?}", self.status));
        }
        self.status = ProcessingStatus::Processing;
        self.add_log(LogLevel::Info, "Processing resumed".to_string());
        Ok(())
    }

    /// Complete processing
    pub fn complete(&mut self) {
        self.status = ProcessingStatus::Complete;
        self.stats.end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs());
        self.add_log(
            LogLevel::Info,
            format!(
                "Processing complete: {} processed, {} skipped, {} errors",
                self.stats.processed_count, self.stats.skipped_count, self.stats.error_count
            ),
        );
    }

    /// Mark as error
    pub fn set_error(&mut self, error_message: String) {
        self.status = ProcessingStatus::Error;
        self.add_log(LogLevel::Error, error_message);
    }

    /// Update current file being processed
    pub fn update_current_file(&mut self, file_name: String) {
        self.stats.current_file_name = file_name;
    }

    /// Increment processed count
    pub fn increment_processed(&mut self) {
        self.stats.processed_count += 1;
        self.stats.update_eta();
    }

    /// Increment skipped count
    pub fn increment_skipped(&mut self, reason: String) {
        self.stats.skipped_count += 1;
        self.add_log(LogLevel::Warning, format!("Skipped: {}", reason));
        self.stats.update_eta();
    }

    /// Increment error count
    pub fn increment_error(&mut self, error_message: String) {
        self.stats.error_count += 1;
        self.add_log(LogLevel::Error, error_message);
        self.stats.update_eta();
    }

    /// Add log message
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.log_messages.push(LogMessage {
            timestamp,
            level,
            message,
        });

        // Keep only last 100 messages
        if self.log_messages.len() > 100 {
            self.log_messages.drain(0..self.log_messages.len() - 100);
        }
    }

    /// Clear log messages
    pub fn clear_logs(&mut self) {
        self.log_messages.clear();
    }

    /// Reset state to idle
    pub fn reset(&mut self) {
        *self = PipelineWindowState::new();
    }
}

/// Log message for pipeline operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    /// Unix timestamp
    pub timestamp: u64,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
}

impl LogMessage {
    /// Format timestamp as human-readable string
    pub fn formatted_time(&self) -> String {
        // Simple formatting - can be enhanced with chrono
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs())
            .unwrap_or(0)
            .saturating_sub(self.timestamp);

        if elapsed < 60 {
            format!("{}s ago", elapsed)
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else {
            format!("{}h ago", elapsed / 3600)
        }
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Debug information
    Debug,
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_status_transitions() {
        let status = ProcessingStatus::Idle;
        assert!(status.can_start());
        assert!(!status.can_pause());

        let status = ProcessingStatus::Processing;
        assert!(!status.can_start());
        assert!(status.can_pause());
        assert!(status.is_active());
    }

    #[test]
    fn test_processing_stats_progress() {
        let mut stats = ProcessingStats::new(100);
        assert_eq!(stats.progress_percentage(), 0.0);

        stats.processed_count = 50;
        assert_eq!(stats.progress_percentage(), 50.0);

        stats.skipped_count = 25;
        stats.error_count = 25;
        assert_eq!(stats.progress_percentage(), 100.0);
    }

    #[test]
    fn test_processing_stats_success_rate() {
        let mut stats = ProcessingStats::new(100);
        stats.processed_count = 80;
        stats.error_count = 20;

        assert_eq!(stats.success_rate(), 80.0);
    }

    #[test]
    fn test_pipeline_state_operations() {
        let mut state = PipelineWindowState::new();
        assert_eq!(state.status, ProcessingStatus::Idle);

        state.start_operation(OperationType::DirectoryImport, 10);
        assert_eq!(state.status, ProcessingStatus::Processing);
        assert_eq!(state.stats.total_files, 10);

        state.pause().unwrap();
        assert_eq!(state.status, ProcessingStatus::Paused);

        state.resume().unwrap();
        assert_eq!(state.status, ProcessingStatus::Processing);

        state.complete();
        assert_eq!(state.status, ProcessingStatus::Complete);
    }

    #[test]
    fn test_pipeline_state_log_limit() {
        let mut state = PipelineWindowState::new();

        // Add 150 log messages
        for i in 0..150 {
            state.add_log(LogLevel::Info, format!("Message {}", i));
        }

        // Should keep only last 100
        assert_eq!(state.log_messages.len(), 100);
        assert_eq!(state.log_messages[0].message, "Message 50");
    }

    #[test]
    fn test_pipeline_state_stats_updates() {
        let mut state = PipelineWindowState::new();
        state.start_operation(OperationType::BatchAnalysis, 100);

        state.increment_processed();
        assert_eq!(state.stats.processed_count, 1);

        state.increment_skipped("Duplicate".to_string());
        assert_eq!(state.stats.skipped_count, 1);

        state.increment_error("Parse error".to_string());
        assert_eq!(state.stats.error_count, 1);

        assert_eq!(state.stats.files_remaining(), 97);
    }
}
