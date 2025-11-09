   /// Command Serialization - Trusty Module
   ///
   /// Support for serializing/deserializing commands for persistence across sessions.

use super::core::{Command, UndoRedoError, UndoRedoResult};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Trait for commands that can be serialized
pub trait SerializableCommand: Command + Debug {
    /// Serialize command to JSON
    fn to_json(&self) -> UndoRedoResult<String>;

    /// Deserialize command from JSON
    fn from_json(json: &str) -> UndoRedoResult<Box<dyn SerializableCommand>>
    where
        Self: Sized;

    /// Get command type identifier
    fn command_type(&self) -> &'static str;
}

/// Wrapper for serializable commands
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedCommand {
    pub command_type: String,
    pub data: String,
    pub timestamp: u64,
}

impl SerializedCommand {
    pub fn new(command_type: String, data: String) -> Self {
        Self {
            command_type,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Command serializer/deserializer
pub struct CommandSerializer;

impl CommandSerializer {
    /// Serialize a command history to JSON
    pub fn serialize_history(commands: &[SerializedCommand]) -> UndoRedoResult<String> {
        serde_json::to_string(commands)
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Serialization failed: {}", e)))
    }

    /// Deserialize a command history from JSON
    pub fn deserialize_history(json: &str) -> UndoRedoResult<Vec<SerializedCommand>> {
        serde_json::from_str(json).map_err(|e| {
            UndoRedoError::ExecutionFailed(format!("Deserialization failed: {}", e))
        })
    }

    /// Save command history to file
    pub fn save_to_file(commands: &[SerializedCommand], path: &str) -> UndoRedoResult<()> {
        let json = Self::serialize_history(commands)?;
        std::fs::write(path, json)
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Failed to write file: {}", e)))
    }

    /// Load command history from file
    pub fn load_from_file(path: &str) -> UndoRedoResult<Vec<SerializedCommand>> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Failed to read file: {}", e)))?;
        Self::deserialize_history(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialized_command_creation() {
        let cmd = SerializedCommand::new("AddNote".to_string(), r#"{"pitch":60}"#.to_string());

        assert_eq!(cmd.command_type, "AddNote");
        assert_eq!(cmd.data, r#"{"pitch":60}"#);
        assert!(cmd.timestamp > 0);
    }

    #[test]
    fn test_serialize_history() {
        let commands = vec![
            SerializedCommand::new("AddNote".to_string(), r#"{"pitch":60}"#.to_string()),
            SerializedCommand::new("DeleteNote".to_string(), r#"{"id":1}"#.to_string()),
        ];

        let json = CommandSerializer::serialize_history(&commands).unwrap();
        assert!(json.contains("AddNote"));
        assert!(json.contains("DeleteNote"));
    }

    #[test]
    fn test_deserialize_history() {
        let commands = vec![
            SerializedCommand::new("AddNote".to_string(), r#"{"pitch":60}"#.to_string()),
        ];

        let json = CommandSerializer::serialize_history(&commands).unwrap();
        let deserialized = CommandSerializer::deserialize_history(&json).unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0].command_type, "AddNote");
    }

    #[test]
    fn test_serialize_empty_history() {
        let commands: Vec<SerializedCommand> = vec![];
        let json = CommandSerializer::serialize_history(&commands).unwrap();
        assert_eq!(json, "[]");
    }

    #[test]
    fn test_deserialize_empty_history() {
        let deserialized = CommandSerializer::deserialize_history("[]").unwrap();
        assert_eq!(deserialized.len(), 0);
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let result = CommandSerializer::deserialize_history("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_to_file() {
        let commands = vec![
            SerializedCommand::new("AddNote".to_string(), r#"{"pitch":60}"#.to_string()),
        ];

        let temp_file = "/tmp/test_undo_history.json";
        CommandSerializer::save_to_file(&commands, temp_file).unwrap();

        // Verify file exists
        assert!(std::path::Path::new(temp_file).exists());

        // Clean up
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_load_from_file() {
        let commands = vec![
            SerializedCommand::new("AddNote".to_string(), r#"{"pitch":60}"#.to_string()),
        ];

        let temp_file = "/tmp/test_undo_history_load.json";
        CommandSerializer::save_to_file(&commands, temp_file).unwrap();

        let loaded = CommandSerializer::load_from_file(temp_file).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].command_type, "AddNote");

        // Clean up
        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = CommandSerializer::load_from_file("/tmp/nonexistent_file.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_serialization() {
        let commands = vec![
            SerializedCommand::new("AddNote".to_string(), r#"{"pitch":60}"#.to_string()),
            SerializedCommand::new("DeleteNote".to_string(), r#"{"id":1}"#.to_string()),
            SerializedCommand::new("MoveNote".to_string(), r#"{"id":2,"tick":960}"#.to_string()),
        ];

        let json = CommandSerializer::serialize_history(&commands).unwrap();
        let deserialized = CommandSerializer::deserialize_history(&json).unwrap();

        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0].command_type, "AddNote");
        assert_eq!(deserialized[1].command_type, "DeleteNote");
        assert_eq!(deserialized[2].command_type, "MoveNote");
    }

    #[test]
    fn test_serialized_command_preserves_data() {
        let cmd = SerializedCommand::new(
            "ComplexCommand".to_string(),
            r#"{"field1":"value1","field2":42,"field3":true}"#.to_string(),
        );

        let json = CommandSerializer::serialize_history(&[cmd]).unwrap();
        let deserialized = CommandSerializer::deserialize_history(&json).unwrap();

        assert_eq!(deserialized[0].data, r#"{"field1":"value1","field2":42,"field3":true}"#);
    }
}
