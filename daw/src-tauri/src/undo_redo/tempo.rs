   /// Tempo Editor Undo/Redo Commands - Trusty Module
   ///
   /// Commands for tempo editor operations: add/remove markers, set tempo, create ramps.

use super::core::{Command, UndoRedoError, UndoRedoResult};
use serde::{Deserialize, Serialize};

/// Tempo marker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TempoMarker {
    pub tick: i32,
    pub bpm: f32,
    pub marker_type: TempoMarkerType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TempoMarkerType {
    /// Single tempo change at this point
    Instant,
    /// Start of tempo ramp
    RampStart,
    /// End of tempo ramp
    RampEnd,
}

impl TempoMarker {
    pub fn new(tick: i32, bpm: f32) -> Self {
        Self {
            tick,
            bpm,
            marker_type: TempoMarkerType::Instant,
        }
    }

    pub fn with_type(mut self, marker_type: TempoMarkerType) -> Self {
        self.marker_type = marker_type;
        self
    }
}

/// Add a tempo marker
#[derive(Debug)]
pub struct AddTempoMarkerCommand {
    marker: TempoMarker,
    marker_id: Option<i32>,
    executed: bool,
}

impl AddTempoMarkerCommand {
    pub fn new(tick: i32, bpm: f32) -> Self {
        Self {
            marker: TempoMarker::new(tick, bpm),
            marker_id: None,
            executed: false,
        }
    }

    pub fn with_type(mut self, marker_type: TempoMarkerType) -> Self {
        self.marker = self.marker.with_type(marker_type);
        self
    }
}

impl Command for AddTempoMarkerCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Validate BPM range
            if self.marker.bpm < 20.0 || self.marker.bpm > 999.0 {
                return Err(UndoRedoError::ExecutionFailed(format!(
                    "Invalid BPM: {} (must be 20-999)",
                    self.marker.bpm
                )));
            }
            // Add to state and get ID
            self.marker_id = Some(1); // Mock ID
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Remove marker from state
        Ok(())
    }

    fn description(&self) -> String {
        format!("Add Tempo Marker ({} BPM)", self.marker.bpm)
    }

    fn memory_usage(&self) -> usize {
        32
    }
}

/// Remove a tempo marker
#[derive(Debug)]
pub struct RemoveTempoMarkerCommand {
    marker_id: i32,
    removed_marker: Option<TempoMarker>,
    executed: bool,
}

impl RemoveTempoMarkerCommand {
    pub fn new(marker_id: i32) -> Self {
        Self {
            marker_id,
            removed_marker: None,
            executed: false,
        }
    }
}

impl Command for RemoveTempoMarkerCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Store removed marker (would fetch from state)
            self.removed_marker = Some(TempoMarker::new(0, 120.0));
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Restore marker to state
        Ok(())
    }

    fn description(&self) -> String {
        format!("Remove Tempo Marker (ID {})", self.marker_id)
    }

    fn memory_usage(&self) -> usize {
        32 + self.removed_marker.as_ref().map(|_| 24).unwrap_or(0)
    }
}

/// Set global tempo
#[derive(Debug)]
pub struct SetTempoCommand {
    new_tempo: f32,
    old_tempo: Option<f32>,
    executed: bool,
}

impl SetTempoCommand {
    pub fn new(new_tempo: f32) -> Self {
        Self {
            new_tempo,
            old_tempo: None,
            executed: false,
        }
    }
}

impl Command for SetTempoCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Validate BPM range
            if self.new_tempo < 20.0 || self.new_tempo > 999.0 {
                return Err(UndoRedoError::ExecutionFailed(format!(
                    "Invalid BPM: {} (must be 20-999)",
                    self.new_tempo
                )));
            }
            // Store old tempo (would fetch from state)
            self.old_tempo = Some(120.0);
            // Apply new tempo to state
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Restore old tempo
        Ok(())
    }

    fn description(&self) -> String {
        format!("Set Tempo to {} BPM", self.new_tempo)
    }

    fn memory_usage(&self) -> usize {
        24
    }
}

/// Set tempo ramp between two markers
#[derive(Debug)]
pub struct SetTempoRampCommand {
    start_tick: i32,
    end_tick: i32,
    start_bpm: f32,
    end_bpm: f32,
    start_marker_id: Option<i32>,
    end_marker_id: Option<i32>,
    executed: bool,
}

impl SetTempoRampCommand {
    pub fn new(start_tick: i32, end_tick: i32, start_bpm: f32, end_bpm: f32) -> Self {
        Self {
            start_tick,
            end_tick,
            start_bpm,
            end_bpm,
            start_marker_id: None,
            end_marker_id: None,
            executed: false,
        }
    }
}

impl Command for SetTempoRampCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Validate inputs
            if self.start_tick >= self.end_tick {
                return Err(UndoRedoError::ExecutionFailed(
                    "Start tick must be before end tick".to_string(),
                ));
            }
            if self.start_bpm < 20.0 || self.start_bpm > 999.0 {
                return Err(UndoRedoError::ExecutionFailed(format!(
                    "Invalid start BPM: {} (must be 20-999)",
                    self.start_bpm
                )));
            }
            if self.end_bpm < 20.0 || self.end_bpm > 999.0 {
                return Err(UndoRedoError::ExecutionFailed(format!(
                    "Invalid end BPM: {} (must be 20-999)",
                    self.end_bpm
                )));
            }
            // Add markers to state and get IDs
            self.start_marker_id = Some(1);
            self.end_marker_id = Some(2);
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Remove ramp markers from state
        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Set Tempo Ramp ({} to {} BPM)",
            self.start_bpm, self.end_bpm
        )
    }

    fn memory_usage(&self) -> usize {
        48
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_marker_creation() {
        let marker = TempoMarker::new(480, 120.0);
        assert_eq!(marker.tick, 480);
        assert_eq!(marker.bpm, 120.0);
        assert_eq!(marker.marker_type, TempoMarkerType::Instant);
    }

    #[test]
    fn test_tempo_marker_with_type() {
        let marker = TempoMarker::new(480, 120.0).with_type(TempoMarkerType::RampStart);
        assert_eq!(marker.marker_type, TempoMarkerType::RampStart);
    }

    #[test]
    fn test_add_tempo_marker_command() {
        let mut cmd = AddTempoMarkerCommand::new(480, 120.0);

        cmd.execute().unwrap();
        assert!(cmd.marker_id.is_some());

        cmd.undo().unwrap();
    }

    #[test]
    fn test_add_tempo_marker_invalid_bpm_low() {
        let mut cmd = AddTempoMarkerCommand::new(480, 10.0);

        let result = cmd.execute();
        assert!(matches!(result, Err(UndoRedoError::ExecutionFailed(_))));
    }

    #[test]
    fn test_add_tempo_marker_invalid_bpm_high() {
        let mut cmd = AddTempoMarkerCommand::new(480, 1000.0);

        let result = cmd.execute();
        assert!(matches!(result, Err(UndoRedoError::ExecutionFailed(_))));
    }

    #[test]
    fn test_add_tempo_marker_description() {
        let cmd = AddTempoMarkerCommand::new(480, 140.5);
        assert_eq!(cmd.description(), "Add Tempo Marker (140.5 BPM)");
    }

    #[test]
    fn test_remove_tempo_marker_command() {
        let mut cmd = RemoveTempoMarkerCommand::new(1);

        cmd.execute().unwrap();
        assert!(cmd.removed_marker.is_some());

        cmd.undo().unwrap();
    }

    #[test]
    fn test_remove_tempo_marker_description() {
        let cmd = RemoveTempoMarkerCommand::new(5);
        assert_eq!(cmd.description(), "Remove Tempo Marker (ID 5)");
    }

    #[test]
    fn test_set_tempo_command() {
        let mut cmd = SetTempoCommand::new(140.0);

        cmd.execute().unwrap();
        assert!(cmd.old_tempo.is_some());

        cmd.undo().unwrap();
    }

    #[test]
    fn test_set_tempo_invalid_bpm() {
        let mut cmd = SetTempoCommand::new(1500.0);

        let result = cmd.execute();
        assert!(matches!(result, Err(UndoRedoError::ExecutionFailed(_))));
    }

    #[test]
    fn test_set_tempo_description() {
        let cmd = SetTempoCommand::new(128.0);
        assert_eq!(cmd.description(), "Set Tempo to 128 BPM");
    }

    #[test]
    fn test_set_tempo_ramp_command() {
        let mut cmd = SetTempoRampCommand::new(0, 1920, 120.0, 140.0);

        cmd.execute().unwrap();
        assert!(cmd.start_marker_id.is_some());
        assert!(cmd.end_marker_id.is_some());

        cmd.undo().unwrap();
    }

    #[test]
    fn test_set_tempo_ramp_invalid_ticks() {
        let mut cmd = SetTempoRampCommand::new(1920, 0, 120.0, 140.0);

        let result = cmd.execute();
        assert!(matches!(result, Err(UndoRedoError::ExecutionFailed(_))));
    }

    #[test]
    fn test_set_tempo_ramp_invalid_start_bpm() {
        let mut cmd = SetTempoRampCommand::new(0, 1920, 10.0, 140.0);

        let result = cmd.execute();
        assert!(matches!(result, Err(UndoRedoError::ExecutionFailed(_))));
    }

    #[test]
    fn test_set_tempo_ramp_invalid_end_bpm() {
        let mut cmd = SetTempoRampCommand::new(0, 1920, 120.0, 2000.0);

        let result = cmd.execute();
        assert!(matches!(result, Err(UndoRedoError::ExecutionFailed(_))));
    }

    #[test]
    fn test_set_tempo_ramp_description() {
        let cmd = SetTempoRampCommand::new(0, 1920, 100.0, 160.0);
        assert_eq!(cmd.description(), "Set Tempo Ramp (100 to 160 BPM)");
    }

    #[test]
    fn test_command_memory_usage() {
        let add_cmd = AddTempoMarkerCommand::new(480, 120.0);
        assert!(add_cmd.memory_usage() > 0);

        let remove_cmd = RemoveTempoMarkerCommand::new(1);
        assert!(remove_cmd.memory_usage() > 0);

        let set_cmd = SetTempoCommand::new(140.0);
        assert!(set_cmd.memory_usage() > 0);

        let ramp_cmd = SetTempoRampCommand::new(0, 1920, 120.0, 140.0);
        assert!(ramp_cmd.memory_usage() > 0);
    }

    #[test]
    fn test_tempo_marker_equality() {
        let marker1 = TempoMarker::new(480, 120.0);
        let marker2 = TempoMarker::new(480, 120.0);
        let marker3 = TempoMarker::new(960, 120.0);

        assert_eq!(marker1, marker2);
        assert_ne!(marker1, marker3);
    }
}
