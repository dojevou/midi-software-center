use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub aliases: Vec<String>,
    pub keybinding: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandCategory {
    Transport,
    Track,
    Edit,
    View,
    Settings,
    Help,
}

impl CommandCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommandCategory::Transport => "Transport",
            CommandCategory::Track => "Track",
            CommandCategory::Edit => "Edit",
            CommandCategory::View => "View",
            CommandCategory::Settings => "Settings",
            CommandCategory::Help => "Help",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub command: CommandEntry,
    pub score: u32,
}

pub struct CommandPalette {
    commands: Vec<CommandEntry>,
    recently_used: Mutex<VecDeque<String>>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let commands = Self::initialize_commands();
        Self { commands, recently_used: Mutex::new(VecDeque::with_capacity(10)) }
    }

    fn initialize_commands() -> Vec<CommandEntry> {
        vec![
            // Transport commands
            CommandEntry {
                id: "transport.play".to_string(),
                name: "Play".to_string(),
                description: "Start playback".to_string(),
                category: CommandCategory::Transport,
                aliases: vec!["start".to_string(), "resume".to_string()],
                keybinding: Some("Space".to_string()),
            },
            CommandEntry {
                id: "transport.stop".to_string(),
                name: "Stop".to_string(),
                description: "Stop playback".to_string(),
                category: CommandCategory::Transport,
                aliases: vec!["halt".to_string()],
                keybinding: Some("Space".to_string()),
            },
            CommandEntry {
                id: "transport.pause".to_string(),
                name: "Pause".to_string(),
                description: "Pause playback".to_string(),
                category: CommandCategory::Transport,
                aliases: vec![],
                keybinding: Some("Ctrl+P".to_string()),
            },
            CommandEntry {
                id: "transport.record".to_string(),
                name: "Record".to_string(),
                description: "Start recording".to_string(),
                category: CommandCategory::Transport,
                aliases: vec!["rec".to_string()],
                keybinding: Some("Ctrl+R".to_string()),
            },
            CommandEntry {
                id: "transport.loop".to_string(),
                name: "Toggle Loop".to_string(),
                description: "Enable or disable loop mode".to_string(),
                category: CommandCategory::Transport,
                aliases: vec!["repeat".to_string()],
                keybinding: Some("Ctrl+L".to_string()),
            },
            CommandEntry {
                id: "transport.rewind".to_string(),
                name: "Rewind".to_string(),
                description: "Jump to start".to_string(),
                category: CommandCategory::Transport,
                aliases: vec!["beginning".to_string(), "start".to_string()],
                keybinding: Some("Home".to_string()),
            },
            CommandEntry {
                id: "transport.forward".to_string(),
                name: "Fast Forward".to_string(),
                description: "Jump to end".to_string(),
                category: CommandCategory::Transport,
                aliases: vec!["end".to_string()],
                keybinding: Some("End".to_string()),
            },
            CommandEntry {
                id: "transport.metronome".to_string(),
                name: "Toggle Metronome".to_string(),
                description: "Enable or disable metronome click".to_string(),
                category: CommandCategory::Transport,
                aliases: vec!["click".to_string(), "metro".to_string()],
                keybinding: Some("Ctrl+M".to_string()),
            },
            // Track commands
            CommandEntry {
                id: "track.new".to_string(),
                name: "New Track".to_string(),
                description: "Create a new MIDI track".to_string(),
                category: CommandCategory::Track,
                aliases: vec!["add track".to_string(), "nt".to_string()],
                keybinding: Some("Ctrl+T".to_string()),
            },
            CommandEntry {
                id: "track.delete".to_string(),
                name: "Delete Track".to_string(),
                description: "Remove the selected track".to_string(),
                category: CommandCategory::Track,
                aliases: vec!["remove track".to_string()],
                keybinding: Some("Ctrl+Shift+D".to_string()),
            },
            CommandEntry {
                id: "track.duplicate".to_string(),
                name: "Duplicate Track".to_string(),
                description: "Create a copy of the selected track".to_string(),
                category: CommandCategory::Track,
                aliases: vec!["copy track".to_string()],
                keybinding: Some("Ctrl+D".to_string()),
            },
            CommandEntry {
                id: "track.mute".to_string(),
                name: "Mute Track".to_string(),
                description: "Mute the selected track".to_string(),
                category: CommandCategory::Track,
                aliases: vec!["silence".to_string()],
                keybinding: Some("M".to_string()),
            },
            CommandEntry {
                id: "track.solo".to_string(),
                name: "Solo Track".to_string(),
                description: "Solo the selected track".to_string(),
                category: CommandCategory::Track,
                aliases: vec![],
                keybinding: Some("S".to_string()),
            },
            CommandEntry {
                id: "track.arm".to_string(),
                name: "Arm Track".to_string(),
                description: "Arm track for recording".to_string(),
                category: CommandCategory::Track,
                aliases: vec!["enable recording".to_string()],
                keybinding: Some("Ctrl+Shift+R".to_string()),
            },
            CommandEntry {
                id: "track.rename".to_string(),
                name: "Rename Track".to_string(),
                description: "Rename the selected track".to_string(),
                category: CommandCategory::Track,
                aliases: vec![],
                keybinding: Some("F2".to_string()),
            },
            CommandEntry {
                id: "track.color".to_string(),
                name: "Set Track Color".to_string(),
                description: "Change track color".to_string(),
                category: CommandCategory::Track,
                aliases: vec!["change color".to_string()],
                keybinding: None,
            },
            // Edit commands
            CommandEntry {
                id: "edit.undo".to_string(),
                name: "Undo".to_string(),
                description: "Undo last action".to_string(),
                category: CommandCategory::Edit,
                aliases: vec![],
                keybinding: Some("Ctrl+Z".to_string()),
            },
            CommandEntry {
                id: "edit.redo".to_string(),
                name: "Redo".to_string(),
                description: "Redo last undone action".to_string(),
                category: CommandCategory::Edit,
                aliases: vec![],
                keybinding: Some("Ctrl+Shift+Z".to_string()),
            },
            CommandEntry {
                id: "edit.cut".to_string(),
                name: "Cut".to_string(),
                description: "Cut selected notes".to_string(),
                category: CommandCategory::Edit,
                aliases: vec![],
                keybinding: Some("Ctrl+X".to_string()),
            },
            CommandEntry {
                id: "edit.copy".to_string(),
                name: "Copy".to_string(),
                description: "Copy selected notes".to_string(),
                category: CommandCategory::Edit,
                aliases: vec![],
                keybinding: Some("Ctrl+C".to_string()),
            },
            CommandEntry {
                id: "edit.paste".to_string(),
                name: "Paste".to_string(),
                description: "Paste notes from clipboard".to_string(),
                category: CommandCategory::Edit,
                aliases: vec![],
                keybinding: Some("Ctrl+V".to_string()),
            },
            CommandEntry {
                id: "edit.delete".to_string(),
                name: "Delete".to_string(),
                description: "Delete selected notes".to_string(),
                category: CommandCategory::Edit,
                aliases: vec!["remove".to_string()],
                keybinding: Some("Delete".to_string()),
            },
            CommandEntry {
                id: "edit.select_all".to_string(),
                name: "Select All".to_string(),
                description: "Select all notes in current track".to_string(),
                category: CommandCategory::Edit,
                aliases: vec![],
                keybinding: Some("Ctrl+A".to_string()),
            },
            CommandEntry {
                id: "edit.quantize".to_string(),
                name: "Quantize".to_string(),
                description: "Snap notes to grid".to_string(),
                category: CommandCategory::Edit,
                aliases: vec!["snap".to_string()],
                keybinding: Some("Q".to_string()),
            },
            CommandEntry {
                id: "edit.transpose_up".to_string(),
                name: "Transpose Up".to_string(),
                description: "Transpose selected notes up one semitone".to_string(),
                category: CommandCategory::Edit,
                aliases: vec!["pitch up".to_string()],
                keybinding: Some("Ctrl+Up".to_string()),
            },
            CommandEntry {
                id: "edit.transpose_down".to_string(),
                name: "Transpose Down".to_string(),
                description: "Transpose selected notes down one semitone".to_string(),
                category: CommandCategory::Edit,
                aliases: vec!["pitch down".to_string()],
                keybinding: Some("Ctrl+Down".to_string()),
            },
            // View commands
            CommandEntry {
                id: "view.zoom_in".to_string(),
                name: "Zoom In".to_string(),
                description: "Increase timeline zoom".to_string(),
                category: CommandCategory::View,
                aliases: vec!["magnify".to_string()],
                keybinding: Some("Ctrl+=".to_string()),
            },
            CommandEntry {
                id: "view.zoom_out".to_string(),
                name: "Zoom Out".to_string(),
                description: "Decrease timeline zoom".to_string(),
                category: CommandCategory::View,
                aliases: vec!["shrink".to_string()],
                keybinding: Some("Ctrl+-".to_string()),
            },
            CommandEntry {
                id: "view.zoom_fit".to_string(),
                name: "Zoom to Fit".to_string(),
                description: "Fit all content in view".to_string(),
                category: CommandCategory::View,
                aliases: vec!["fit".to_string()],
                keybinding: Some("Ctrl+0".to_string()),
            },
            CommandEntry {
                id: "view.show_mixer".to_string(),
                name: "Show Mixer".to_string(),
                description: "Toggle mixer panel".to_string(),
                category: CommandCategory::View,
                aliases: vec!["mixer".to_string()],
                keybinding: Some("F3".to_string()),
            },
            CommandEntry {
                id: "view.show_piano_roll".to_string(),
                name: "Show Piano Roll".to_string(),
                description: "Toggle piano roll editor".to_string(),
                category: CommandCategory::View,
                aliases: vec!["piano".to_string(), "pr".to_string()],
                keybinding: Some("F4".to_string()),
            },
            CommandEntry {
                id: "view.show_browser".to_string(),
                name: "Show Browser".to_string(),
                description: "Toggle file browser".to_string(),
                category: CommandCategory::View,
                aliases: vec!["browser".to_string()],
                keybinding: Some("F5".to_string()),
            },
            CommandEntry {
                id: "view.fullscreen".to_string(),
                name: "Toggle Fullscreen".to_string(),
                description: "Enter or exit fullscreen mode".to_string(),
                category: CommandCategory::View,
                aliases: vec![],
                keybinding: Some("F11".to_string()),
            },
            // Settings commands
            CommandEntry {
                id: "settings.general".to_string(),
                name: "General Settings".to_string(),
                description: "Open general settings".to_string(),
                category: CommandCategory::Settings,
                aliases: vec!["preferences".to_string()],
                keybinding: Some("Ctrl+,".to_string()),
            },
            CommandEntry {
                id: "settings.audio".to_string(),
                name: "Audio Settings".to_string(),
                description: "Configure audio device and buffer".to_string(),
                category: CommandCategory::Settings,
                aliases: vec!["audio config".to_string()],
                keybinding: None,
            },
            CommandEntry {
                id: "settings.midi".to_string(),
                name: "MIDI Settings".to_string(),
                description: "Configure MIDI devices".to_string(),
                category: CommandCategory::Settings,
                aliases: vec!["midi config".to_string()],
                keybinding: None,
            },
            CommandEntry {
                id: "settings.keyboard".to_string(),
                name: "Keyboard Shortcuts".to_string(),
                description: "Customize keyboard shortcuts".to_string(),
                category: CommandCategory::Settings,
                aliases: vec!["keybindings".to_string(), "shortcuts".to_string()],
                keybinding: None,
            },
            // Help commands
            CommandEntry {
                id: "help.documentation".to_string(),
                name: "Documentation".to_string(),
                description: "Open user manual".to_string(),
                category: CommandCategory::Help,
                aliases: vec!["manual".to_string(), "docs".to_string()],
                keybinding: Some("F1".to_string()),
            },
            CommandEntry {
                id: "help.about".to_string(),
                name: "About".to_string(),
                description: "About MIDI Software Center".to_string(),
                category: CommandCategory::Help,
                aliases: vec!["info".to_string()],
                keybinding: None,
            },
        ]
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.is_empty() {
            return self.get_recently_used();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = self
            .commands
            .iter()
            .filter_map(|cmd| {
                let score = self.calculate_match_score(cmd, &query_lower);
                if score > 0 {
                    Some(SearchResult { command: cmd.clone(), score })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.cmp(&a.score));

        results
    }

    fn calculate_match_score(&self, cmd: &CommandEntry, query: &str) -> u32 {
        let name_lower = cmd.name.to_lowercase();
        let desc_lower = cmd.description.to_lowercase();

        let mut score = 0u32;

        // Exact match on name (highest priority)
        if name_lower == query {
            score = score.saturating_add(1000);
        }

        // Name starts with query
        if name_lower.starts_with(query) {
            score = score.saturating_add(500);
        }

        // Name contains query
        if name_lower.contains(query) {
            score = score.saturating_add(200);
        }

        // Description contains query
        if desc_lower.contains(query) {
            score = score.saturating_add(100);
        }

        // Alias match
        for alias in &cmd.aliases {
            let alias_lower = alias.to_lowercase();
            if alias_lower == query {
                score = score.saturating_add(800);
            } else if alias_lower.starts_with(query) {
                score = score.saturating_add(400);
            } else if alias_lower.contains(query) {
                score = score.saturating_add(150);
            }
        }

        // Abbreviation match (e.g., "nt" matches "New Track")
        if self.matches_abbreviation(&name_lower, query) {
            score = score.saturating_add(300);
        }

        // Category match
        if cmd.category.as_str().to_lowercase().contains(query) {
            score = score.saturating_add(50);
        }

        score
    }

    fn matches_abbreviation(&self, text: &str, abbr: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return false;
        }

        let abbr_chars: Vec<char> = abbr.chars().collect();
        if abbr_chars.len() > words.len() {
            return false;
        }

        // Check if abbreviation matches first letters of words
        words.iter().zip(abbr_chars.iter()).all(|(word, &ch)| word.starts_with(ch))
    }

    pub fn get_recently_used(&self) -> Vec<SearchResult> {
        let recent = self.recently_used.lock().unwrap();
        recent
            .iter()
            .filter_map(|id| {
                self.commands.iter().find(|cmd| &cmd.id == id).map(|cmd| SearchResult {
                    command: cmd.clone(),
                    score: 0, // Recently used items don't need scores
                })
            })
            .collect()
    }

    pub fn record_usage(&self, command_id: &str) -> Result<(), String> {
        let mut recent = self
            .recently_used
            .lock()
            .map_err(|e| format!("Failed to lock recently used: {}", e))?;

        // Remove if already present
        recent.retain(|id| id != command_id);

        // Add to front
        recent.push_front(command_id.to_string());

        // Limit to 10 items
        if recent.len() > 10 {
            recent.pop_back();
        }

        Ok(())
    }

    pub fn get_all_commands(&self) -> Vec<CommandEntry> {
        self.commands.clone()
    }

    pub fn get_command_by_id(&self, id: &str) -> Option<CommandEntry> {
        self.commands.iter().find(|cmd| cmd.id == id).cloned()
    }

    pub fn parse_keybinding(keybinding: &str) -> Result<Vec<String>, String> {
        if keybinding.trim().is_empty() {
            return Err("Empty keybinding".to_string());
        }

        let parts: Vec<String> = keybinding
            .split('+')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            return Err("Empty keybinding".to_string());
        }

        Ok(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_palette_initialization() {
        let palette = CommandPalette::new();
        assert!(!palette.commands.is_empty());
        // We have 42 commands defined
        assert!(
            palette.commands.len() >= 30,
            "Expected at least 30 commands, got {}",
            palette.commands.len()
        );
    }

    #[test]
    fn test_search_exact_match() {
        let palette = CommandPalette::new();
        let results = palette.search("Play");

        assert!(!results.is_empty());
        assert_eq!(results[0].command.name, "Play");
        assert!(results[0].score > 900); // High score for exact match
    }

    #[test]
    fn test_search_partial_match() {
        let palette = CommandPalette::new();
        let results = palette.search("track");

        assert!(!results.is_empty());
        // Should find multiple track-related commands
        assert!(results.len() > 5);
    }

    #[test]
    fn test_search_abbreviation() {
        let palette = CommandPalette::new();
        let results = palette.search("nt");

        assert!(!results.is_empty());
        // "New Track" should match "nt" abbreviation
        let new_track = results.iter().find(|r| r.command.name == "New Track");
        assert!(new_track.is_some());
    }

    #[test]
    fn test_search_alias() {
        let palette = CommandPalette::new();
        let results = palette.search("rec");

        assert!(!results.is_empty());
        // "Record" has "rec" as alias
        let record = results.iter().find(|r| r.command.name == "Record");
        assert!(record.is_some());
        assert!(record.unwrap().score > 700); // High score for alias match
    }

    #[test]
    fn test_search_empty_query() {
        let palette = CommandPalette::new();
        let results = palette.search("");

        // Should return recently used (empty initially)
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let palette = CommandPalette::new();
        let results_lower = palette.search("play");
        let results_upper = palette.search("PLAY");

        assert_eq!(results_lower.len(), results_upper.len());
        assert_eq!(results_lower[0].command.id, results_upper[0].command.id);
    }

    #[test]
    fn test_search_no_results() {
        let palette = CommandPalette::new();
        let results = palette.search("xyzabc123");

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_record_usage() {
        let palette = CommandPalette::new();

        palette.record_usage("transport.play").unwrap();
        palette.record_usage("transport.stop").unwrap();

        let recent = palette.get_recently_used();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].command.id, "transport.stop"); // Most recent first
        assert_eq!(recent[1].command.id, "transport.play");
    }

    #[test]
    fn test_record_usage_limit() {
        let palette = CommandPalette::new();
        let all_commands = palette.get_all_commands();

        // Add 12 real command IDs (more than limit of 10)
        for command in all_commands.iter().take(12.min(all_commands.len())) {
            palette.record_usage(&command.id).ok();
        }

        let recent = palette.get_recently_used();
        assert_eq!(recent.len(), 10); // Should be limited to 10
    }

    #[test]
    fn test_record_usage_duplicate() {
        let palette = CommandPalette::new();

        palette.record_usage("transport.play").unwrap();
        palette.record_usage("transport.stop").unwrap();
        palette.record_usage("transport.play").unwrap(); // Duplicate

        let recent = palette.get_recently_used();
        assert_eq!(recent.len(), 2); // Should not have duplicates
        assert_eq!(recent[0].command.id, "transport.play"); // Moved to front
    }

    #[test]
    fn test_get_all_commands() {
        let palette = CommandPalette::new();
        let all = palette.get_all_commands();

        assert!(!all.is_empty());
        assert!(
            all.len() >= 30,
            "Expected at least 30 commands, got {}",
            all.len()
        );
    }

    #[test]
    fn test_get_command_by_id() {
        let palette = CommandPalette::new();

        let cmd = palette.get_command_by_id("transport.play");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "Play");

        let invalid = palette.get_command_by_id("invalid.id");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_command_categories() {
        let palette = CommandPalette::new();
        let all = palette.get_all_commands();

        let transport =
            all.iter().filter(|c| matches!(c.category, CommandCategory::Transport)).count();
        let track = all.iter().filter(|c| matches!(c.category, CommandCategory::Track)).count();
        let edit = all.iter().filter(|c| matches!(c.category, CommandCategory::Edit)).count();
        let view = all.iter().filter(|c| matches!(c.category, CommandCategory::View)).count();
        let settings =
            all.iter().filter(|c| matches!(c.category, CommandCategory::Settings)).count();
        let help = all.iter().filter(|c| matches!(c.category, CommandCategory::Help)).count();

        assert!(transport > 0);
        assert!(track > 0);
        assert!(edit > 0);
        assert!(view > 0);
        assert!(settings > 0);
        assert!(help > 0);
    }

    #[test]
    fn test_parse_keybinding_simple() {
        let result = CommandPalette::parse_keybinding("Space");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Space"]);
    }

    #[test]
    fn test_parse_keybinding_modifier() {
        let result = CommandPalette::parse_keybinding("Ctrl+S");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Ctrl", "S"]);
    }

    #[test]
    fn test_parse_keybinding_multiple_modifiers() {
        let result = CommandPalette::parse_keybinding("Ctrl+Shift+D");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["Ctrl", "Shift", "D"]);
    }

    #[test]
    fn test_parse_keybinding_empty() {
        let result = CommandPalette::parse_keybinding("");
        assert!(result.is_err());
    }

    #[test]
    fn test_category_as_str() {
        assert_eq!(CommandCategory::Transport.as_str(), "Transport");
        assert_eq!(CommandCategory::Track.as_str(), "Track");
        assert_eq!(CommandCategory::Edit.as_str(), "Edit");
        assert_eq!(CommandCategory::View.as_str(), "View");
        assert_eq!(CommandCategory::Settings.as_str(), "Settings");
        assert_eq!(CommandCategory::Help.as_str(), "Help");
    }

    #[test]
    fn test_search_by_category() {
        let palette = CommandPalette::new();
        let results = palette.search("transport");

        // Should find transport-related commands
        assert!(!results.is_empty());
        // Most results should be Transport category, but may include others with "transport" in description
        let has_transport =
            results.iter().any(|r| matches!(r.command.category, CommandCategory::Transport));
        assert!(has_transport);
    }

    #[test]
    fn test_search_by_description() {
        let palette = CommandPalette::new();
        let results = palette.search("playback");

        // Should find commands with "playback" in description
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_ordering() {
        let palette = CommandPalette::new();
        let results = palette.search("track");

        // Results should be ordered by score (highest first)
        if results.len() > 1 {
            assert!(results[0].score >= results[1].score);
        }
    }

    #[test]
    fn test_matches_abbreviation_valid() {
        let palette = CommandPalette::new();
        assert!(palette.matches_abbreviation("new track", "nt"));
        assert!(palette.matches_abbreviation("zoom in", "zi"));
    }

    #[test]
    fn test_matches_abbreviation_invalid() {
        let palette = CommandPalette::new();
        assert!(!palette.matches_abbreviation("new track", "tx"));
        assert!(!palette.matches_abbreviation("zoom in", "zx"));
        assert!(!palette.matches_abbreviation("play", "abc")); // Too long
    }

    #[test]
    fn test_command_entry_serialization() {
        let entry = CommandEntry {
            id: "test.command".to_string(),
            name: "Test Command".to_string(),
            description: "A test command".to_string(),
            category: CommandCategory::Edit,
            aliases: vec!["test".to_string()],
            keybinding: Some("Ctrl+T".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: CommandEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            command: CommandEntry {
                id: "test.command".to_string(),
                name: "Test".to_string(),
                description: "Test".to_string(),
                category: CommandCategory::Edit,
                aliases: vec![],
                keybinding: None,
            },
            score: 100,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.command.id, deserialized.command.id);
        assert_eq!(result.score, deserialized.score);
    }
}
