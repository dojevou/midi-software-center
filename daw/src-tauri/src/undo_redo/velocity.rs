/// Velocity Editor Undo/Redo Commands - Trusty Module
///
/// Commands for velocity editor operations: set range, interpolate, reset velocities.
use super::core::{Command, UndoRedoError, UndoRedoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Velocity data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityData {
    pub note_id: i32,
    pub velocity: u8,
}

/// Set velocity for a range of notes (humanize)
#[derive(Debug)]
pub struct SetVelocityRangeCommand {
    note_velocities: HashMap<i32, u8>,
    old_velocities: HashMap<i32, u8>,
    executed: bool,
}

impl SetVelocityRangeCommand {
    pub fn new(note_velocities: HashMap<i32, u8>) -> Self {
        Self { note_velocities, old_velocities: HashMap::new(), executed: false }
    }

    pub fn with_old_velocities(mut self, old_velocities: HashMap<i32, u8>) -> Self {
        self.old_velocities = old_velocities;
        self
    }
}

impl Command for SetVelocityRangeCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Store old velocities if not already set
            if self.old_velocities.is_empty() {
                for note_id in self.note_velocities.keys() {
                    // In a real implementation, this would fetch from state
                    // For now, we store empty values
                    self.old_velocities.insert(*note_id, 100);
                }
            }
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
        // Restore old velocities
        Ok(())
    }

    fn description(&self) -> String {
        format!("Set Velocity for {} notes", self.note_velocities.len())
    }

    fn memory_usage(&self) -> usize {
        48 + (self.note_velocities.len() + self.old_velocities.len()) * 12
    }
}

/// Interpolate velocity between two points
#[derive(Debug)]
pub struct InterpolateVelocityCommand {
    note_ids: Vec<i32>,
    start_velocity: u8,
    end_velocity: u8,
    old_velocities: HashMap<i32, u8>,
    executed: bool,
}

impl InterpolateVelocityCommand {
    pub fn new(note_ids: Vec<i32>, start_velocity: u8, end_velocity: u8) -> Self {
        Self {
            note_ids,
            start_velocity,
            end_velocity,
            old_velocities: HashMap::new(),
            executed: false,
        }
    }

    fn calculate_interpolated_velocity(&self, index: usize) -> u8 {
        if self.note_ids.len() <= 1 {
            return self.start_velocity;
        }

        let ratio = index as f32 / (self.note_ids.len() - 1) as f32;
        let interpolated = self.start_velocity as f32
            + ratio * (self.end_velocity as i16 - self.start_velocity as i16) as f32;

        interpolated.round().clamp(1.0, 127.0) as u8
    }
}

impl Command for InterpolateVelocityCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Calculate and store interpolated values
            for (index, &note_id) in self.note_ids.iter().enumerate() {
                // Store old velocity (would fetch from state in real implementation)
                self.old_velocities.insert(note_id, 100);

                // Calculate new velocity
                let _new_velocity = self.calculate_interpolated_velocity(index);
                // Apply to state (would be done in real implementation)
            }
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
        // Restore old velocities
        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Interpolate Velocity ({} to {}) for {} notes",
            self.start_velocity,
            self.end_velocity,
            self.note_ids.len()
        )
    }

    fn memory_usage(&self) -> usize {
        48 + self.note_ids.len() * 8 + self.old_velocities.len() * 12
    }
}

/// Reset all velocities to a default value
#[derive(Debug)]
pub struct ResetVelocityCommand {
    note_ids: Vec<i32>,
    default_velocity: u8,
    old_velocities: HashMap<i32, u8>,
    executed: bool,
}

impl ResetVelocityCommand {
    pub fn new(note_ids: Vec<i32>, default_velocity: u8) -> Self {
        Self { note_ids, default_velocity, old_velocities: HashMap::new(), executed: false }
    }
}

impl Command for ResetVelocityCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Store old velocities
            for &note_id in &self.note_ids {
                self.old_velocities.insert(note_id, 100);
            }
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
        // Restore old velocities
        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Reset Velocity to {} for {} notes",
            self.default_velocity,
            self.note_ids.len()
        )
    }

    fn memory_usage(&self) -> usize {
        48 + self.note_ids.len() * 8 + self.old_velocities.len() * 12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_velocity_range_command() {
        let mut velocities = HashMap::new();
        velocities.insert(1, 80);
        velocities.insert(2, 90);

        let mut cmd = SetVelocityRangeCommand::new(velocities);

        cmd.execute().unwrap();
        assert!(!cmd.old_velocities.is_empty());

        cmd.undo().unwrap();
    }

    #[test]
    fn test_set_velocity_range_description() {
        let mut velocities = HashMap::new();
        velocities.insert(1, 80);
        velocities.insert(2, 90);

        let cmd = SetVelocityRangeCommand::new(velocities);
        assert_eq!(cmd.description(), "Set Velocity for 2 notes");
    }

    #[test]
    fn test_interpolate_velocity_command() {
        let note_ids = vec![1, 2, 3, 4, 5];
        let mut cmd = InterpolateVelocityCommand::new(note_ids, 50, 100);

        cmd.execute().unwrap();
        assert_eq!(cmd.old_velocities.len(), 5);

        cmd.undo().unwrap();
    }

    #[test]
    fn test_interpolate_velocity_calculation() {
        let note_ids = vec![1, 2, 3, 4, 5];
        let cmd = InterpolateVelocityCommand::new(note_ids, 50, 100);

        assert_eq!(cmd.calculate_interpolated_velocity(0), 50);
        assert_eq!(cmd.calculate_interpolated_velocity(2), 75);
        assert_eq!(cmd.calculate_interpolated_velocity(4), 100);
    }

    #[test]
    fn test_interpolate_velocity_single_note() {
        let note_ids = vec![1];
        let cmd = InterpolateVelocityCommand::new(note_ids, 50, 100);

        assert_eq!(cmd.calculate_interpolated_velocity(0), 50);
    }

    #[test]
    fn test_interpolate_velocity_description() {
        let note_ids = vec![1, 2, 3];
        let cmd = InterpolateVelocityCommand::new(note_ids, 50, 100);

        assert_eq!(
            cmd.description(),
            "Interpolate Velocity (50 to 100) for 3 notes"
        );
    }

    #[test]
    fn test_reset_velocity_command() {
        let note_ids = vec![1, 2, 3, 4, 5];
        let mut cmd = ResetVelocityCommand::new(note_ids, 100);

        cmd.execute().unwrap();
        assert_eq!(cmd.old_velocities.len(), 5);

        cmd.undo().unwrap();
    }

    #[test]
    fn test_reset_velocity_description() {
        let note_ids = vec![1, 2, 3];
        let cmd = ResetVelocityCommand::new(note_ids, 100);

        assert_eq!(cmd.description(), "Reset Velocity to 100 for 3 notes");
    }

    #[test]
    fn test_reset_velocity_undo_before_execute() {
        let note_ids = vec![1, 2, 3];
        let mut cmd = ResetVelocityCommand::new(note_ids, 100);

        let result = cmd.undo();
        assert!(matches!(result, Err(UndoRedoError::UndoFailed(_))));
    }

    #[test]
    fn test_command_memory_usage() {
        let mut velocities = HashMap::new();
        velocities.insert(1, 80);
        velocities.insert(2, 90);

        let set_cmd = SetVelocityRangeCommand::new(velocities);
        assert!(set_cmd.memory_usage() > 0);

        let interp_cmd = InterpolateVelocityCommand::new(vec![1, 2, 3], 50, 100);
        assert!(interp_cmd.memory_usage() > 0);

        let reset_cmd = ResetVelocityCommand::new(vec![1, 2, 3], 100);
        assert!(reset_cmd.memory_usage() > 0);
    }

    #[test]
    fn test_interpolate_velocity_clamping() {
        let note_ids = vec![1, 2, 3];
        let cmd = InterpolateVelocityCommand::new(note_ids, 1, 127);

        let vel = cmd.calculate_interpolated_velocity(1);
        assert!(vel >= 1 && vel <= 127);
    }

    #[test]
    fn test_interpolate_velocity_descending() {
        let note_ids = vec![1, 2, 3, 4, 5];
        let cmd = InterpolateVelocityCommand::new(note_ids, 100, 50);

        assert_eq!(cmd.calculate_interpolated_velocity(0), 100);
        assert_eq!(cmd.calculate_interpolated_velocity(4), 50);
    }

    #[test]
    fn test_set_velocity_range_with_old_velocities() {
        let mut velocities = HashMap::new();
        velocities.insert(1, 80);

        let mut old_velocities = HashMap::new();
        old_velocities.insert(1, 70);

        let mut cmd = SetVelocityRangeCommand::new(velocities).with_old_velocities(old_velocities);

        cmd.execute().unwrap();
        assert_eq!(cmd.old_velocities.get(&1), Some(&70));
    }
}
