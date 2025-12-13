//! Preferences Commands - Settings, Window Layouts, Keyboard Shortcuts, Recent Projects
//!
//! This module provides Tauri commands for managing user preferences including:
//! - Application settings (audio, MIDI, display, library, etc.)
//! - Window layouts (save/restore window positions)
//! - Keyboard shortcuts (customizable hotkeys)
//! - Recent projects list
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{command, State};

// =============================================================================
// SETTINGS
// =============================================================================

/// Application setting with category and value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub category: String,
    pub value_type: String, // "string", "integer", "float", "boolean", "json"
    pub value_string: Option<String>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,
    pub value_json: Option<serde_json::Value>,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub is_user_modified: bool,
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            key: String::new(),
            category: "general".to_string(),
            value_type: "string".to_string(),
            value_string: None,
            value_int: None,
            value_float: None,
            value_bool: None,
            value_json: None,
            default_value: None,
            description: None,
            is_user_modified: false,
        }
    }
}

/// Settings state management for preferences
/// Named AppSettingsState to avoid conflict with settings.rs::SettingsState
pub struct AppSettingsState {
    pub settings: Mutex<HashMap<String, Setting>>,
}

impl Default for AppSettingsState {
    fn default() -> Self {
        let mut settings = HashMap::new();

        // Default audio settings
        settings.insert(
            "audio.sample_rate".to_string(),
            Setting {
                key: "audio.sample_rate".to_string(),
                category: "audio".to_string(),
                value_type: "integer".to_string(),
                value_int: Some(44100),
                default_value: Some("44100".to_string()),
                description: Some("Audio sample rate in Hz".to_string()),
                ..Default::default()
            },
        );
        settings.insert(
            "audio.buffer_size".to_string(),
            Setting {
                key: "audio.buffer_size".to_string(),
                category: "audio".to_string(),
                value_type: "integer".to_string(),
                value_int: Some(256),
                default_value: Some("256".to_string()),
                description: Some("Audio buffer size in samples".to_string()),
                ..Default::default()
            },
        );
        settings.insert(
            "audio.driver".to_string(),
            Setting {
                key: "audio.driver".to_string(),
                category: "audio".to_string(),
                value_type: "string".to_string(),
                value_string: Some("default".to_string()),
                default_value: Some("default".to_string()),
                description: Some("Audio driver to use".to_string()),
                ..Default::default()
            },
        );

        // Default MIDI settings
        settings.insert(
            "midi.clock_ppqn".to_string(),
            Setting {
                key: "midi.clock_ppqn".to_string(),
                category: "midi".to_string(),
                value_type: "integer".to_string(),
                value_int: Some(24),
                default_value: Some("24".to_string()),
                description: Some("MIDI clock pulses per quarter note".to_string()),
                ..Default::default()
            },
        );
        settings.insert(
            "midi.send_clock".to_string(),
            Setting {
                key: "midi.send_clock".to_string(),
                category: "midi".to_string(),
                value_type: "boolean".to_string(),
                value_bool: Some(true),
                default_value: Some("true".to_string()),
                description: Some("Send MIDI clock to outputs".to_string()),
                ..Default::default()
            },
        );

        // Default display settings
        settings.insert(
            "display.theme".to_string(),
            Setting {
                key: "display.theme".to_string(),
                category: "display".to_string(),
                value_type: "string".to_string(),
                value_string: Some("dark".to_string()),
                default_value: Some("dark".to_string()),
                description: Some("UI theme".to_string()),
                ..Default::default()
            },
        );
        settings.insert(
            "display.font_size".to_string(),
            Setting {
                key: "display.font_size".to_string(),
                category: "display".to_string(),
                value_type: "integer".to_string(),
                value_int: Some(12),
                default_value: Some("12".to_string()),
                description: Some("Base font size in points".to_string()),
                ..Default::default()
            },
        );

        // Default library settings
        settings.insert(
            "library.auto_analyze".to_string(),
            Setting {
                key: "library.auto_analyze".to_string(),
                category: "library".to_string(),
                value_type: "boolean".to_string(),
                value_bool: Some(true),
                default_value: Some("true".to_string()),
                description: Some("Automatically analyze imported files".to_string()),
                ..Default::default()
            },
        );
        settings.insert(
            "library.watch_folders".to_string(),
            Setting {
                key: "library.watch_folders".to_string(),
                category: "library".to_string(),
                value_type: "boolean".to_string(),
                value_bool: Some(false),
                default_value: Some("false".to_string()),
                description: Some("Watch folders for new files".to_string()),
                ..Default::default()
            },
        );

        Self { settings: Mutex::new(settings) }
    }
}

/// Get all settings
#[command]
pub async fn settings_get_all(state: State<'_, AppSettingsState>) -> Result<Vec<Setting>, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.values().cloned().collect())
}

/// Get settings by category
#[command]
pub async fn settings_get_by_category(
    state: State<'_, AppSettingsState>,
    category: String,
) -> Result<Vec<Setting>, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.values().filter(|s| s.category == category).cloned().collect())
}

/// Get a single setting by key
#[command]
pub async fn settings_get(
    state: State<'_, AppSettingsState>,
    key: String,
) -> Result<Option<Setting>, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.get(&key).cloned())
}

/// Set a string setting
#[command]
pub async fn settings_set_string(
    state: State<'_, AppSettingsState>,
    key: String,
    value: String,
) -> Result<Setting, String> {
    let mut settings = state.settings.lock().unwrap();

    let setting = settings.entry(key.clone()).or_insert_with(|| Setting {
        key: key.clone(),
        category: key.split('.').next().unwrap_or("general").to_string(),
        value_type: "string".to_string(),
        ..Default::default()
    });

    setting.value_string = Some(value);
    setting.value_type = "string".to_string();
    setting.is_user_modified = true;

    Ok(setting.clone())
}

/// Set an integer setting
#[command]
pub async fn settings_set_int(
    state: State<'_, AppSettingsState>,
    key: String,
    value: i64,
) -> Result<Setting, String> {
    let mut settings = state.settings.lock().unwrap();

    let setting = settings.entry(key.clone()).or_insert_with(|| Setting {
        key: key.clone(),
        category: key.split('.').next().unwrap_or("general").to_string(),
        value_type: "integer".to_string(),
        ..Default::default()
    });

    setting.value_int = Some(value);
    setting.value_type = "integer".to_string();
    setting.is_user_modified = true;

    Ok(setting.clone())
}

/// Set a float setting
#[command]
pub async fn settings_set_float(
    state: State<'_, AppSettingsState>,
    key: String,
    value: f64,
) -> Result<Setting, String> {
    let mut settings = state.settings.lock().unwrap();

    let setting = settings.entry(key.clone()).or_insert_with(|| Setting {
        key: key.clone(),
        category: key.split('.').next().unwrap_or("general").to_string(),
        value_type: "float".to_string(),
        ..Default::default()
    });

    setting.value_float = Some(value);
    setting.value_type = "float".to_string();
    setting.is_user_modified = true;

    Ok(setting.clone())
}

/// Set a boolean setting
#[command]
pub async fn settings_set_bool(
    state: State<'_, AppSettingsState>,
    key: String,
    value: bool,
) -> Result<Setting, String> {
    let mut settings = state.settings.lock().unwrap();

    let setting = settings.entry(key.clone()).or_insert_with(|| Setting {
        key: key.clone(),
        category: key.split('.').next().unwrap_or("general").to_string(),
        value_type: "boolean".to_string(),
        ..Default::default()
    });

    setting.value_bool = Some(value);
    setting.value_type = "boolean".to_string();
    setting.is_user_modified = true;

    Ok(setting.clone())
}

/// Set a JSON setting
#[command]
pub async fn settings_set_json(
    state: State<'_, AppSettingsState>,
    key: String,
    value: serde_json::Value,
) -> Result<Setting, String> {
    let mut settings = state.settings.lock().unwrap();

    let setting = settings.entry(key.clone()).or_insert_with(|| Setting {
        key: key.clone(),
        category: key.split('.').next().unwrap_or("general").to_string(),
        value_type: "json".to_string(),
        ..Default::default()
    });

    setting.value_json = Some(value);
    setting.value_type = "json".to_string();
    setting.is_user_modified = true;

    Ok(setting.clone())
}

/// Reset a setting to its default value
#[command]
pub async fn settings_reset(
    state: State<'_, AppSettingsState>,
    key: String,
) -> Result<Option<Setting>, String> {
    let mut settings = state.settings.lock().unwrap();

    if let Some(setting) = settings.get_mut(&key) {
        // Reset based on default value
        if let Some(default) = &setting.default_value {
            match setting.value_type.as_str() {
                "string" => setting.value_string = Some(default.clone()),
                "integer" => setting.value_int = default.parse().ok(),
                "float" => setting.value_float = default.parse().ok(),
                "boolean" => setting.value_bool = default.parse().ok(),
                _ => {},
            }
        }
        setting.is_user_modified = false;
        Ok(Some(setting.clone()))
    } else {
        Ok(None)
    }
}

/// Reset all settings in a category
#[command]
pub async fn settings_reset_category(
    state: State<'_, AppSettingsState>,
    category: String,
) -> Result<u32, String> {
    let mut settings = state.settings.lock().unwrap();
    let mut count = 0;

    for setting in settings.values_mut() {
        if setting.category == category {
            if let Some(default) = &setting.default_value {
                match setting.value_type.as_str() {
                    "string" => setting.value_string = Some(default.clone()),
                    "integer" => setting.value_int = default.parse().ok(),
                    "float" => setting.value_float = default.parse().ok(),
                    "boolean" => setting.value_bool = default.parse().ok(),
                    _ => {},
                }
            }
            setting.is_user_modified = false;
            count += 1;
        }
    }

    Ok(count)
}

/// Delete a user-defined setting
#[command]
pub async fn settings_delete(
    state: State<'_, AppSettingsState>,
    key: String,
) -> Result<bool, String> {
    let mut settings = state.settings.lock().unwrap();
    Ok(settings.remove(&key).is_some())
}

// =============================================================================
// WINDOW LAYOUTS
// =============================================================================

/// Window position and size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition {
    pub window_id: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_maximized: bool,
    pub is_visible: bool,
    pub z_order: i32,
}

/// Complete window layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowLayout {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub windows: Vec<WindowPosition>,
    pub created_at: String,
    pub updated_at: String,
}

/// Window layouts state
pub struct WindowLayoutState {
    pub layouts: Mutex<Vec<WindowLayout>>,
    pub current_layout_id: Mutex<Option<u64>>,
    next_id: Mutex<u64>,
}

impl Default for WindowLayoutState {
    fn default() -> Self {
        let default_layout = WindowLayout {
            id: 1,
            name: "Default".to_string(),
            description: Some("Default window layout".to_string()),
            is_default: true,
            windows: vec![
                WindowPosition {
                    window_id: "main".to_string(),
                    x: 100,
                    y: 100,
                    width: 1200,
                    height: 800,
                    is_maximized: false,
                    is_visible: true,
                    z_order: 0,
                },
                WindowPosition {
                    window_id: "mixer".to_string(),
                    x: 100,
                    y: 500,
                    width: 1000,
                    height: 300,
                    is_maximized: false,
                    is_visible: true,
                    z_order: 1,
                },
            ],
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        Self {
            layouts: Mutex::new(vec![default_layout]),
            current_layout_id: Mutex::new(Some(1)),
            next_id: Mutex::new(2),
        }
    }
}

/// List all window layouts
#[command]
pub async fn layouts_list(
    state: State<'_, WindowLayoutState>,
) -> Result<Vec<WindowLayout>, String> {
    let layouts = state.layouts.lock().unwrap();
    Ok(layouts.clone())
}

/// Get current layout
#[command]
pub async fn layouts_get_current(
    state: State<'_, WindowLayoutState>,
) -> Result<Option<WindowLayout>, String> {
    let layouts = state.layouts.lock().unwrap();
    let current_id = state.current_layout_id.lock().unwrap();

    if let Some(id) = *current_id {
        Ok(layouts.iter().find(|l| l.id == id).cloned())
    } else {
        Ok(None)
    }
}

/// Get a specific layout by ID
#[command]
pub async fn layouts_get(
    state: State<'_, WindowLayoutState>,
    id: u64,
) -> Result<Option<WindowLayout>, String> {
    let layouts = state.layouts.lock().unwrap();
    Ok(layouts.iter().find(|l| l.id == id).cloned())
}

/// Save current window positions as a new layout
#[command]
pub async fn layouts_save(
    state: State<'_, WindowLayoutState>,
    name: String,
    description: Option<String>,
    windows: Vec<WindowPosition>,
) -> Result<WindowLayout, String> {
    let mut layouts = state.layouts.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();

    let layout = WindowLayout {
        id: *next_id,
        name,
        description,
        is_default: false,
        windows,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    *next_id += 1;
    layouts.push(layout.clone());

    Ok(layout)
}

/// Update an existing layout
#[command]
pub async fn layouts_update(
    state: State<'_, WindowLayoutState>,
    id: u64,
    name: Option<String>,
    description: Option<String>,
    windows: Option<Vec<WindowPosition>>,
) -> Result<Option<WindowLayout>, String> {
    let mut layouts = state.layouts.lock().unwrap();

    if let Some(layout) = layouts.iter_mut().find(|l| l.id == id) {
        if let Some(n) = name {
            layout.name = n;
        }
        if let Some(d) = description {
            layout.description = Some(d);
        }
        if let Some(w) = windows {
            layout.windows = w;
        }
        layout.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(Some(layout.clone()))
    } else {
        Ok(None)
    }
}

/// Set the default layout
#[command]
pub async fn layouts_set_default(
    state: State<'_, WindowLayoutState>,
    id: u64,
) -> Result<bool, String> {
    let mut layouts = state.layouts.lock().unwrap();

    // Clear existing default
    for layout in layouts.iter_mut() {
        layout.is_default = layout.id == id;
    }

    Ok(layouts.iter().any(|l| l.id == id))
}

/// Apply a layout (set as current)
#[command]
pub async fn layouts_apply(state: State<'_, WindowLayoutState>, id: u64) -> Result<bool, String> {
    let layouts = state.layouts.lock().unwrap();
    let mut current = state.current_layout_id.lock().unwrap();

    if layouts.iter().any(|l| l.id == id) {
        *current = Some(id);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Delete a layout
#[command]
pub async fn layouts_delete(state: State<'_, WindowLayoutState>, id: u64) -> Result<bool, String> {
    let mut layouts = state.layouts.lock().unwrap();

    // Don't delete default layout
    if layouts.iter().any(|l| l.id == id && l.is_default) {
        return Err("Cannot delete default layout".to_string());
    }

    let len_before = layouts.len();
    layouts.retain(|l| l.id != id);

    // Clear current if deleted
    if layouts.len() < len_before {
        let mut current = state.current_layout_id.lock().unwrap();
        if *current == Some(id) {
            *current = layouts.iter().find(|l| l.is_default).map(|l| l.id);
        }
    }

    Ok(layouts.len() < len_before)
}

// =============================================================================
// KEYBOARD SHORTCUTS
// =============================================================================

/// Keyboard shortcut definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    pub id: u64,
    pub action: String,
    pub category: String,
    pub key_combo: String, // e.g., "Ctrl+S", "Cmd+Shift+P"
    pub description: Option<String>,
    pub is_global: bool,
    pub is_user_modified: bool,
    pub default_key_combo: Option<String>,
}

/// Keyboard shortcuts state
pub struct KeyboardShortcutsState {
    pub shortcuts: Mutex<Vec<KeyboardShortcut>>,
    next_id: Mutex<u64>,
}

impl Default for KeyboardShortcutsState {
    fn default() -> Self {
        let shortcuts = vec![
            // File operations
            KeyboardShortcut {
                id: 1,
                action: "file.new".to_string(),
                category: "file".to_string(),
                key_combo: "Ctrl+N".to_string(),
                description: Some("Create new project".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Ctrl+N".to_string()),
            },
            KeyboardShortcut {
                id: 2,
                action: "file.open".to_string(),
                category: "file".to_string(),
                key_combo: "Ctrl+O".to_string(),
                description: Some("Open project".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Ctrl+O".to_string()),
            },
            KeyboardShortcut {
                id: 3,
                action: "file.save".to_string(),
                category: "file".to_string(),
                key_combo: "Ctrl+S".to_string(),
                description: Some("Save project".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Ctrl+S".to_string()),
            },
            // Edit operations
            KeyboardShortcut {
                id: 4,
                action: "edit.undo".to_string(),
                category: "edit".to_string(),
                key_combo: "Ctrl+Z".to_string(),
                description: Some("Undo last action".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Ctrl+Z".to_string()),
            },
            KeyboardShortcut {
                id: 5,
                action: "edit.redo".to_string(),
                category: "edit".to_string(),
                key_combo: "Ctrl+Y".to_string(),
                description: Some("Redo last undone action".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Ctrl+Y".to_string()),
            },
            KeyboardShortcut {
                id: 6,
                action: "edit.copy".to_string(),
                category: "edit".to_string(),
                key_combo: "Ctrl+C".to_string(),
                description: Some("Copy selection".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Ctrl+C".to_string()),
            },
            KeyboardShortcut {
                id: 7,
                action: "edit.paste".to_string(),
                category: "edit".to_string(),
                key_combo: "Ctrl+V".to_string(),
                description: Some("Paste clipboard".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Ctrl+V".to_string()),
            },
            KeyboardShortcut {
                id: 8,
                action: "edit.delete".to_string(),
                category: "edit".to_string(),
                key_combo: "Delete".to_string(),
                description: Some("Delete selection".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("Delete".to_string()),
            },
            // Transport
            KeyboardShortcut {
                id: 9,
                action: "transport.play".to_string(),
                category: "transport".to_string(),
                key_combo: "Space".to_string(),
                description: Some("Play/Pause".to_string()),
                is_global: true,
                is_user_modified: false,
                default_key_combo: Some("Space".to_string()),
            },
            KeyboardShortcut {
                id: 10,
                action: "transport.stop".to_string(),
                category: "transport".to_string(),
                key_combo: "Enter".to_string(),
                description: Some("Stop playback".to_string()),
                is_global: true,
                is_user_modified: false,
                default_key_combo: Some("Enter".to_string()),
            },
            KeyboardShortcut {
                id: 11,
                action: "transport.record".to_string(),
                category: "transport".to_string(),
                key_combo: "R".to_string(),
                description: Some("Toggle recording".to_string()),
                is_global: true,
                is_user_modified: false,
                default_key_combo: Some("R".to_string()),
            },
            // View
            KeyboardShortcut {
                id: 12,
                action: "view.mixer".to_string(),
                category: "view".to_string(),
                key_combo: "F3".to_string(),
                description: Some("Show/hide mixer".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("F3".to_string()),
            },
            KeyboardShortcut {
                id: 13,
                action: "view.piano_roll".to_string(),
                category: "view".to_string(),
                key_combo: "F4".to_string(),
                description: Some("Show/hide piano roll".to_string()),
                is_global: false,
                is_user_modified: false,
                default_key_combo: Some("F4".to_string()),
            },
        ];

        Self { shortcuts: Mutex::new(shortcuts), next_id: Mutex::new(14) }
    }
}

/// List all keyboard shortcuts
#[command]
pub async fn shortcuts_list(
    state: State<'_, KeyboardShortcutsState>,
) -> Result<Vec<KeyboardShortcut>, String> {
    let shortcuts = state.shortcuts.lock().unwrap();
    Ok(shortcuts.clone())
}

/// List shortcuts by category
#[command]
pub async fn shortcuts_list_by_category(
    state: State<'_, KeyboardShortcutsState>,
    category: String,
) -> Result<Vec<KeyboardShortcut>, String> {
    let shortcuts = state.shortcuts.lock().unwrap();
    Ok(shortcuts.iter().filter(|s| s.category == category).cloned().collect())
}

/// Get shortcut by action
#[command]
pub async fn shortcuts_get_by_action(
    state: State<'_, KeyboardShortcutsState>,
    action: String,
) -> Result<Option<KeyboardShortcut>, String> {
    let shortcuts = state.shortcuts.lock().unwrap();
    Ok(shortcuts.iter().find(|s| s.action == action).cloned())
}

/// Get shortcut by key combo
#[command]
pub async fn shortcuts_get_by_combo(
    state: State<'_, KeyboardShortcutsState>,
    key_combo: String,
) -> Result<Option<KeyboardShortcut>, String> {
    let shortcuts = state.shortcuts.lock().unwrap();
    Ok(shortcuts.iter().find(|s| s.key_combo == key_combo).cloned())
}

/// Set a keyboard shortcut
#[command]
pub async fn shortcuts_set(
    state: State<'_, KeyboardShortcutsState>,
    action: String,
    key_combo: String,
) -> Result<KeyboardShortcut, String> {
    let mut shortcuts = state.shortcuts.lock().unwrap();

    // Check for conflicts
    if shortcuts.iter().any(|s| s.key_combo == key_combo && s.action != action) {
        return Err(format!("Key combo '{}' is already assigned", key_combo));
    }

    if let Some(shortcut) = shortcuts.iter_mut().find(|s| s.action == action) {
        shortcut.key_combo = key_combo;
        shortcut.is_user_modified = true;
        Ok(shortcut.clone())
    } else {
        Err(format!("Action '{}' not found", action))
    }
}

/// Reset a shortcut to default
#[command]
pub async fn shortcuts_reset(
    state: State<'_, KeyboardShortcutsState>,
    action: String,
) -> Result<Option<KeyboardShortcut>, String> {
    let mut shortcuts = state.shortcuts.lock().unwrap();

    if let Some(shortcut) = shortcuts.iter_mut().find(|s| s.action == action) {
        if let Some(default) = &shortcut.default_key_combo {
            shortcut.key_combo = default.clone();
            shortcut.is_user_modified = false;
        }
        Ok(Some(shortcut.clone()))
    } else {
        Ok(None)
    }
}

/// Reset all shortcuts to defaults
#[command]
pub async fn shortcuts_reset_all(state: State<'_, KeyboardShortcutsState>) -> Result<u32, String> {
    let mut shortcuts = state.shortcuts.lock().unwrap();
    let mut count = 0;

    for shortcut in shortcuts.iter_mut() {
        if let Some(default) = &shortcut.default_key_combo {
            shortcut.key_combo = default.clone();
            shortcut.is_user_modified = false;
            count += 1;
        }
    }

    Ok(count)
}

/// Add a custom shortcut
#[command]
pub async fn shortcuts_add(
    state: State<'_, KeyboardShortcutsState>,
    action: String,
    category: String,
    key_combo: String,
    description: Option<String>,
    is_global: bool,
) -> Result<KeyboardShortcut, String> {
    let mut shortcuts = state.shortcuts.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();

    // Check for conflicts
    if shortcuts.iter().any(|s| s.key_combo == key_combo) {
        return Err(format!("Key combo '{}' is already assigned", key_combo));
    }

    if shortcuts.iter().any(|s| s.action == action) {
        return Err(format!("Action '{}' already exists", action));
    }

    let shortcut = KeyboardShortcut {
        id: *next_id,
        action,
        category,
        key_combo: key_combo.clone(),
        description,
        is_global,
        is_user_modified: true,
        default_key_combo: Some(key_combo),
    };

    *next_id += 1;
    shortcuts.push(shortcut.clone());

    Ok(shortcut)
}

/// Delete a custom shortcut
#[command]
pub async fn shortcuts_delete(
    state: State<'_, KeyboardShortcutsState>,
    action: String,
) -> Result<bool, String> {
    let mut shortcuts = state.shortcuts.lock().unwrap();

    // Only allow deleting user-created shortcuts
    let can_delete = shortcuts
        .iter()
        .find(|s| s.action == action)
        .map(|s| s.default_key_combo.is_none() || s.is_user_modified)
        .unwrap_or(false);

    if !can_delete {
        return Err("Cannot delete built-in shortcuts".to_string());
    }

    let len_before = shortcuts.len();
    shortcuts.retain(|s| s.action != action);
    Ok(shortcuts.len() < len_before)
}

// =============================================================================
// RECENT PROJECTS
// =============================================================================

/// Recent project entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub id: u64,
    pub project_id: Option<u64>,
    pub filepath: String,
    pub name: String,
    pub last_opened: String,
    pub thumbnail_path: Option<String>,
    pub is_pinned: bool,
}

/// Recent projects state
pub struct RecentProjectsState {
    pub projects: Mutex<Vec<RecentProject>>,
    pub max_items: usize,
    next_id: Mutex<u64>,
}

impl Default for RecentProjectsState {
    fn default() -> Self {
        Self { projects: Mutex::new(Vec::new()), max_items: 20, next_id: Mutex::new(1) }
    }
}

/// List recent projects
#[command]
pub async fn recent_list(
    state: State<'_, RecentProjectsState>,
) -> Result<Vec<RecentProject>, String> {
    let projects = state.projects.lock().unwrap();
    Ok(projects.clone())
}

/// Add or update a recent project
#[command]
pub async fn recent_add(
    state: State<'_, RecentProjectsState>,
    filepath: String,
    name: String,
    project_id: Option<u64>,
    thumbnail_path: Option<String>,
) -> Result<RecentProject, String> {
    let mut projects = state.projects.lock().unwrap();
    let mut next_id = state.next_id.lock().unwrap();

    // Check if already exists
    if let Some(existing) = projects.iter_mut().find(|p| p.filepath == filepath) {
        existing.last_opened = chrono::Utc::now().to_rfc3339();
        existing.name = name;
        if let Some(thumb) = thumbnail_path {
            existing.thumbnail_path = Some(thumb);
        }
        return Ok(existing.clone());
    }

    // Add new entry
    let project = RecentProject {
        id: *next_id,
        project_id,
        filepath,
        name,
        last_opened: chrono::Utc::now().to_rfc3339(),
        thumbnail_path,
        is_pinned: false,
    };

    *next_id += 1;
    projects.insert(0, project.clone());

    // Trim to max items (keep pinned items)
    let max = state.max_items;
    while projects.len() > max {
        if let Some(pos) = projects.iter().rposition(|p| !p.is_pinned) {
            if pos >= max {
                projects.remove(pos);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(project)
}

/// Pin/unpin a recent project
#[command]
pub async fn recent_set_pinned(
    state: State<'_, RecentProjectsState>,
    id: u64,
    pinned: bool,
) -> Result<Option<RecentProject>, String> {
    let mut projects = state.projects.lock().unwrap();

    if let Some(project) = projects.iter_mut().find(|p| p.id == id) {
        project.is_pinned = pinned;
        Ok(Some(project.clone()))
    } else {
        Ok(None)
    }
}

/// Remove a recent project
#[command]
pub async fn recent_remove(state: State<'_, RecentProjectsState>, id: u64) -> Result<bool, String> {
    let mut projects = state.projects.lock().unwrap();
    let len_before = projects.len();
    projects.retain(|p| p.id != id);
    Ok(projects.len() < len_before)
}

/// Clear all recent projects (except pinned)
#[command]
pub async fn recent_clear(state: State<'_, RecentProjectsState>) -> Result<u32, String> {
    let mut projects = state.projects.lock().unwrap();
    let len_before = projects.len();
    projects.retain(|p| p.is_pinned);
    Ok((len_before - projects.len()) as u32)
}

/// Clear all recent projects including pinned
#[command]
pub async fn recent_clear_all(state: State<'_, RecentProjectsState>) -> Result<u32, String> {
    let mut projects = state.projects.lock().unwrap();
    let count = projects.len() as u32;
    projects.clear();
    Ok(count)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Setting Tests
    // =========================================================================

    #[test]
    fn test_setting_default() {
        let setting = Setting::default();
        assert_eq!(setting.category, "general");
        assert_eq!(setting.value_type, "string");
        assert!(!setting.is_user_modified);
    }

    #[test]
    fn test_app_settings_state_default() {
        let state = AppSettingsState::default();
        let settings = state.settings.lock().unwrap();

        // Check that default settings exist
        assert!(settings.contains_key("audio.sample_rate"));
        assert!(settings.contains_key("audio.buffer_size"));
        assert!(settings.contains_key("audio.driver"));
        assert!(settings.contains_key("midi.clock_ppqn"));
        assert!(settings.contains_key("midi.send_clock"));
        assert!(settings.contains_key("display.theme"));
        assert!(settings.contains_key("display.font_size"));
        assert!(settings.contains_key("library.auto_analyze"));
        assert!(settings.contains_key("library.watch_folders"));

        // Check default values
        let sample_rate = settings.get("audio.sample_rate").unwrap();
        assert_eq!(sample_rate.value_int, Some(44100));
        assert_eq!(sample_rate.category, "audio");

        let theme = settings.get("display.theme").unwrap();
        assert_eq!(theme.value_string, Some("dark".to_string()));
    }

    #[test]
    fn test_settings_set_and_get() {
        let state = AppSettingsState::default();

        // Set a string setting
        {
            let mut settings = state.settings.lock().unwrap();
            let key = "test.custom_setting".to_string();
            settings.insert(
                key.clone(),
                Setting {
                    key: key.clone(),
                    category: "test".to_string(),
                    value_type: "string".to_string(),
                    value_string: Some("test_value".to_string()),
                    is_user_modified: true,
                    ..Default::default()
                },
            );
        }

        // Verify it was set
        {
            let settings = state.settings.lock().unwrap();
            let setting = settings.get("test.custom_setting").unwrap();
            assert_eq!(setting.value_string, Some("test_value".to_string()));
            assert!(setting.is_user_modified);
        }
    }

    #[test]
    fn test_settings_by_category() {
        let state = AppSettingsState::default();
        let settings = state.settings.lock().unwrap();

        let audio_settings: Vec<_> = settings.values().filter(|s| s.category == "audio").collect();

        assert_eq!(audio_settings.len(), 3); // sample_rate, buffer_size, driver

        let midi_settings: Vec<_> = settings.values().filter(|s| s.category == "midi").collect();

        assert_eq!(midi_settings.len(), 2); // clock_ppqn, send_clock
    }

    #[test]
    fn test_settings_reset() {
        let state = AppSettingsState::default();

        // Modify a setting
        {
            let mut settings = state.settings.lock().unwrap();
            let setting = settings.get_mut("audio.sample_rate").unwrap();
            setting.value_int = Some(96000);
            setting.is_user_modified = true;
        }

        // Verify modification
        {
            let settings = state.settings.lock().unwrap();
            let setting = settings.get("audio.sample_rate").unwrap();
            assert_eq!(setting.value_int, Some(96000));
        }

        // Reset to default
        {
            let mut settings = state.settings.lock().unwrap();
            let setting = settings.get_mut("audio.sample_rate").unwrap();
            if let Some(default) = &setting.default_value {
                setting.value_int = default.parse().ok();
            }
            setting.is_user_modified = false;
        }

        // Verify reset
        {
            let settings = state.settings.lock().unwrap();
            let setting = settings.get("audio.sample_rate").unwrap();
            assert_eq!(setting.value_int, Some(44100));
            assert!(!setting.is_user_modified);
        }
    }

    // =========================================================================
    // Window Layout Tests
    // =========================================================================

    #[test]
    fn test_window_position_default() {
        let pos = WindowPosition {
            window_id: "test".to_string(),
            x: 100,
            y: 200,
            width: 800,
            height: 600,
            is_maximized: false,
            is_visible: true,
            z_order: 0,
        };

        assert_eq!(pos.window_id, "test");
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
        assert_eq!(pos.width, 800);
        assert_eq!(pos.height, 600);
    }

    #[test]
    fn test_window_layout_state_default() {
        let state = WindowLayoutState::default();

        let layouts = state.layouts.lock().unwrap();
        assert_eq!(layouts.len(), 1);

        let default_layout = &layouts[0];
        assert_eq!(default_layout.name, "Default");
        assert!(default_layout.is_default);
        assert_eq!(default_layout.windows.len(), 2); // main and mixer

        let current_id = state.current_layout_id.lock().unwrap();
        assert_eq!(*current_id, Some(1));
    }

    #[test]
    fn test_layout_save() {
        let state = WindowLayoutState::default();

        // Save a new layout
        {
            let mut layouts = state.layouts.lock().unwrap();
            let mut next_id = state.next_id.lock().unwrap();

            let layout = WindowLayout {
                id: *next_id,
                name: "Custom Layout".to_string(),
                description: Some("My custom layout".to_string()),
                is_default: false,
                windows: vec![WindowPosition {
                    window_id: "piano_roll".to_string(),
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                    is_maximized: true,
                    is_visible: true,
                    z_order: 0,
                }],
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };

            *next_id += 1;
            layouts.push(layout);
        }

        // Verify layout was saved
        let layouts = state.layouts.lock().unwrap();
        assert_eq!(layouts.len(), 2);

        let custom = layouts.iter().find(|l| l.name == "Custom Layout").unwrap();
        assert_eq!(custom.id, 2);
        assert_eq!(custom.windows.len(), 1);
        assert!(!custom.is_default);
    }

    #[test]
    fn test_layout_set_default() {
        let state = WindowLayoutState::default();

        // Add a second layout
        {
            let mut layouts = state.layouts.lock().unwrap();
            let mut next_id = state.next_id.lock().unwrap();

            layouts.push(WindowLayout {
                id: *next_id,
                name: "Second".to_string(),
                description: None,
                is_default: false,
                windows: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            });
            *next_id += 1;
        }

        // Set second layout as default
        {
            let mut layouts = state.layouts.lock().unwrap();
            for layout in layouts.iter_mut() {
                layout.is_default = layout.id == 2;
            }
        }

        // Verify
        let layouts = state.layouts.lock().unwrap();
        let first = layouts.iter().find(|l| l.id == 1).unwrap();
        let second = layouts.iter().find(|l| l.id == 2).unwrap();

        assert!(!first.is_default);
        assert!(second.is_default);
    }

    // =========================================================================
    // Keyboard Shortcuts Tests
    // =========================================================================

    #[test]
    fn test_keyboard_shortcut_struct() {
        let shortcut = KeyboardShortcut {
            id: 1,
            action: "test.action".to_string(),
            category: "test".to_string(),
            key_combo: "Ctrl+T".to_string(),
            description: Some("Test action".to_string()),
            is_global: false,
            is_user_modified: false,
            default_key_combo: Some("Ctrl+T".to_string()),
        };

        assert_eq!(shortcut.action, "test.action");
        assert_eq!(shortcut.key_combo, "Ctrl+T");
    }

    #[test]
    fn test_keyboard_shortcuts_state_default() {
        let state = KeyboardShortcutsState::default();

        let shortcuts = state.shortcuts.lock().unwrap();
        assert!(!shortcuts.is_empty());

        // Check for common shortcuts
        assert!(shortcuts.iter().any(|s| s.action == "file.new"));
        assert!(shortcuts.iter().any(|s| s.action == "file.open"));
        assert!(shortcuts.iter().any(|s| s.action == "file.save"));
        assert!(shortcuts.iter().any(|s| s.action == "edit.undo"));
        assert!(shortcuts.iter().any(|s| s.action == "edit.redo"));
        assert!(shortcuts.iter().any(|s| s.action == "transport.play"));
    }

    #[test]
    fn test_shortcuts_categories() {
        let state = KeyboardShortcutsState::default();
        let shortcuts = state.shortcuts.lock().unwrap();

        // Count by category
        let file_count = shortcuts.iter().filter(|s| s.category == "file").count();
        let edit_count = shortcuts.iter().filter(|s| s.category == "edit").count();
        let transport_count = shortcuts.iter().filter(|s| s.category == "transport").count();
        let view_count = shortcuts.iter().filter(|s| s.category == "view").count();

        assert!(file_count >= 3); // new, open, save
        assert!(edit_count >= 5); // undo, redo, copy, paste, delete
        assert!(transport_count >= 3); // play, stop, record
        assert!(view_count >= 2); // mixer, piano_roll
    }

    #[test]
    fn test_shortcut_modify() {
        let state = KeyboardShortcutsState::default();

        // Modify a shortcut
        {
            let mut shortcuts = state.shortcuts.lock().unwrap();
            let shortcut = shortcuts.iter_mut().find(|s| s.action == "file.save").unwrap();
            shortcut.key_combo = "Ctrl+Shift+S".to_string();
            shortcut.is_user_modified = true;
        }

        // Verify modification
        let shortcuts = state.shortcuts.lock().unwrap();
        let shortcut = shortcuts.iter().find(|s| s.action == "file.save").unwrap();
        assert_eq!(shortcut.key_combo, "Ctrl+Shift+S");
        assert!(shortcut.is_user_modified);
    }

    #[test]
    fn test_shortcut_reset() {
        let state = KeyboardShortcutsState::default();

        // Modify then reset
        {
            let mut shortcuts = state.shortcuts.lock().unwrap();
            let shortcut = shortcuts.iter_mut().find(|s| s.action == "file.save").unwrap();
            shortcut.key_combo = "Ctrl+Shift+S".to_string();
            shortcut.is_user_modified = true;
        }

        // Reset
        {
            let mut shortcuts = state.shortcuts.lock().unwrap();
            let shortcut = shortcuts.iter_mut().find(|s| s.action == "file.save").unwrap();
            if let Some(default) = &shortcut.default_key_combo {
                shortcut.key_combo = default.clone();
                shortcut.is_user_modified = false;
            }
        }

        // Verify reset
        let shortcuts = state.shortcuts.lock().unwrap();
        let shortcut = shortcuts.iter().find(|s| s.action == "file.save").unwrap();
        assert_eq!(shortcut.key_combo, "Ctrl+S");
        assert!(!shortcut.is_user_modified);
    }

    #[test]
    fn test_global_shortcuts() {
        let state = KeyboardShortcutsState::default();
        let shortcuts = state.shortcuts.lock().unwrap();

        // Transport shortcuts should be global
        let play = shortcuts.iter().find(|s| s.action == "transport.play").unwrap();
        let stop = shortcuts.iter().find(|s| s.action == "transport.stop").unwrap();
        let record = shortcuts.iter().find(|s| s.action == "transport.record").unwrap();

        assert!(play.is_global);
        assert!(stop.is_global);
        assert!(record.is_global);

        // File shortcuts should not be global
        let save = shortcuts.iter().find(|s| s.action == "file.save").unwrap();
        assert!(!save.is_global);
    }

    // =========================================================================
    // Recent Projects Tests
    // =========================================================================

    #[test]
    fn test_recent_project_struct() {
        let project = RecentProject {
            id: 1,
            project_id: Some(42),
            filepath: "/path/to/project.mid".to_string(),
            name: "My Project".to_string(),
            last_opened: chrono::Utc::now().to_rfc3339(),
            thumbnail_path: Some("/path/to/thumb.png".to_string()),
            is_pinned: false,
        };

        assert_eq!(project.name, "My Project");
        assert_eq!(project.project_id, Some(42));
        assert!(!project.is_pinned);
    }

    #[test]
    fn test_recent_projects_state_default() {
        let state = RecentProjectsState::default();

        let projects = state.projects.lock().unwrap();
        assert!(projects.is_empty());
        assert_eq!(state.max_items, 20);
    }

    #[test]
    fn test_recent_add() {
        let state = RecentProjectsState::default();

        // Add a project
        {
            let mut projects = state.projects.lock().unwrap();
            let mut next_id = state.next_id.lock().unwrap();

            projects.push(RecentProject {
                id: *next_id,
                project_id: None,
                filepath: "/test/project1.mid".to_string(),
                name: "Project 1".to_string(),
                last_opened: chrono::Utc::now().to_rfc3339(),
                thumbnail_path: None,
                is_pinned: false,
            });
            *next_id += 1;
        }

        // Verify
        let projects = state.projects.lock().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Project 1");
    }

    #[test]
    fn test_recent_pin_unpin() {
        let state = RecentProjectsState::default();

        // Add a project
        {
            let mut projects = state.projects.lock().unwrap();
            let mut next_id = state.next_id.lock().unwrap();

            projects.push(RecentProject {
                id: *next_id,
                project_id: None,
                filepath: "/test/project1.mid".to_string(),
                name: "Project 1".to_string(),
                last_opened: chrono::Utc::now().to_rfc3339(),
                thumbnail_path: None,
                is_pinned: false,
            });
            *next_id += 1;
        }

        // Pin the project
        {
            let mut projects = state.projects.lock().unwrap();
            let project = projects.iter_mut().find(|p| p.id == 1).unwrap();
            project.is_pinned = true;
        }

        // Verify pinned
        {
            let projects = state.projects.lock().unwrap();
            let project = projects.iter().find(|p| p.id == 1).unwrap();
            assert!(project.is_pinned);
        }

        // Unpin
        {
            let mut projects = state.projects.lock().unwrap();
            let project = projects.iter_mut().find(|p| p.id == 1).unwrap();
            project.is_pinned = false;
        }

        // Verify unpinned
        let projects = state.projects.lock().unwrap();
        let project = projects.iter().find(|p| p.id == 1).unwrap();
        assert!(!project.is_pinned);
    }

    #[test]
    fn test_recent_clear_keeps_pinned() {
        let state = RecentProjectsState::default();

        // Add multiple projects, some pinned
        {
            let mut projects = state.projects.lock().unwrap();
            let mut next_id = state.next_id.lock().unwrap();

            for i in 1..=5 {
                projects.push(RecentProject {
                    id: *next_id,
                    project_id: None,
                    filepath: format!("/test/project{}.mid", i),
                    name: format!("Project {}", i),
                    last_opened: chrono::Utc::now().to_rfc3339(),
                    thumbnail_path: None,
                    is_pinned: i % 2 == 0, // Pin even-numbered projects
                });
                *next_id += 1;
            }
        }

        // Clear (keeps pinned)
        {
            let mut projects = state.projects.lock().unwrap();
            projects.retain(|p| p.is_pinned);
        }

        // Verify only pinned remain
        let projects = state.projects.lock().unwrap();
        assert_eq!(projects.len(), 2); // Project 2 and 4 (pinned)
        assert!(projects.iter().all(|p| p.is_pinned));
    }

    #[test]
    fn test_recent_clear_all() {
        let state = RecentProjectsState::default();

        // Add multiple projects, some pinned
        {
            let mut projects = state.projects.lock().unwrap();
            let mut next_id = state.next_id.lock().unwrap();

            for i in 1..=5 {
                projects.push(RecentProject {
                    id: *next_id,
                    project_id: None,
                    filepath: format!("/test/project{}.mid", i),
                    name: format!("Project {}", i),
                    last_opened: chrono::Utc::now().to_rfc3339(),
                    thumbnail_path: None,
                    is_pinned: i % 2 == 0,
                });
                *next_id += 1;
            }
        }

        // Clear all
        {
            let mut projects = state.projects.lock().unwrap();
            projects.clear();
        }

        // Verify empty
        let projects = state.projects.lock().unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_recent_max_items() {
        let state = RecentProjectsState::default();

        // Add more than max items
        {
            let mut projects = state.projects.lock().unwrap();
            let mut next_id = state.next_id.lock().unwrap();

            for i in 1..=25 {
                projects.insert(
                    0,
                    RecentProject {
                        id: *next_id,
                        project_id: None,
                        filepath: format!("/test/project{}.mid", i),
                        name: format!("Project {}", i),
                        last_opened: chrono::Utc::now().to_rfc3339(),
                        thumbnail_path: None,
                        is_pinned: false,
                    },
                );
                *next_id += 1;

                // Trim to max
                while projects.len() > state.max_items {
                    if let Some(pos) = projects.iter().rposition(|p| !p.is_pinned) {
                        projects.remove(pos);
                    } else {
                        break;
                    }
                }
            }
        }

        // Verify max is respected
        let projects = state.projects.lock().unwrap();
        assert!(projects.len() <= state.max_items);
    }
}
