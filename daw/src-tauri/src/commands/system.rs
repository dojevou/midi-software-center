//! System monitoring and settings commands
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::{Disks, System};
use tauri::{command, Emitter, State, Window};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percentage: f32,
    pub disk_used: u64,
    pub disk_total: u64,
    pub disk_percentage: f32,
    pub platform: String,
    pub os_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AudioDevice {
    pub name: String,
    pub id: String,
    pub input_channels: u32,
    pub output_channels: u32,
    pub sample_rate: u32,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MidiDevice {
    pub name: String,
    pub id: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_virtual: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub audio_buffer_size: u32,
    pub sample_rate: u32,
    pub theme: String,
    pub language: String,
    pub auto_save: bool,
    pub auto_save_interval: u32,
    pub default_bpm: f32,
    pub default_time_signature: (u8, u8),
}

/// MIDI device routing configuration
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MidiRouteConfig {
    /// Source channel (None = all channels)
    pub source_channel: Option<u8>,
    /// Destination channel (None = pass-through)
    pub dest_channel: Option<u8>,
    /// Transpose semitones
    pub transpose: i8,
    /// Velocity scaling (1.0 = no change)
    pub velocity_scale: f32,
}

/// Individual MIDI device settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MidiDeviceConfig {
    pub device_id: String,
    pub device_name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub enabled: bool,
    pub auto_connect: bool,
    /// Port assignments (track_id -> device_id)
    pub port_assignments: HashMap<i32, String>,
    /// Routing configuration
    pub routing: MidiRouteConfig,
}

/// Persisted MIDI device settings
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MidiDeviceSettings {
    /// Last used input device
    pub last_input_device: Option<String>,
    /// Last used output device
    pub last_output_device: Option<String>,
    /// All configured devices
    pub devices: HashMap<String, MidiDeviceConfig>,
    /// Global MIDI thru enabled
    pub midi_thru_enabled: bool,
    /// Clock sync source device
    pub clock_source_device: Option<String>,
}

impl MidiDeviceSettings {
    /// Get config directory path
    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("midi-software-center")
    }

    /// Get settings file path
    fn settings_path() -> PathBuf {
        Self::config_dir().join("midi_devices.json")
    }

    /// Load settings from disk
    pub fn load() -> Self {
        let path = Self::settings_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(settings) => {
                        info!("Loaded MIDI device settings from {:?}", path);
                        return settings;
                    },
                    Err(e) => {
                        warn!("Failed to parse MIDI settings: {}", e);
                    },
                },
                Err(e) => {
                    warn!("Failed to read MIDI settings file: {}", e);
                },
            }
        }
        Self::default()
    }

    /// Save settings to disk
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config dir: {}", e))?;

        let path = Self::settings_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;

        debug!("Saved MIDI device settings to {:?}", path);
        Ok(())
    }

    /// Update device configuration
    pub fn set_device(&mut self, config: MidiDeviceConfig) {
        let device_id = config.device_id.clone();
        if config.is_input && config.auto_connect {
            self.last_input_device = Some(device_id.clone());
        }
        if config.is_output && config.auto_connect {
            self.last_output_device = Some(device_id.clone());
        }
        self.devices.insert(device_id, config);
    }

    /// Get device configuration
    pub fn get_device(&self, device_id: &str) -> Option<&MidiDeviceConfig> {
        self.devices.get(device_id)
    }
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub system: Arc<Mutex<System>>,
    pub settings: Arc<Mutex<AppSettings>>,
    pub midi_settings: Arc<Mutex<MidiDeviceSettings>>,
    pub monitoring_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Default for SystemState {
    fn default() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Load persisted MIDI settings from disk
        let midi_settings = MidiDeviceSettings::load();

        Self {
            system: Arc::new(Mutex::new(sys)),
            settings: Arc::new(Mutex::new(AppSettings {
                audio_buffer_size: 512,
                sample_rate: 44100,
                theme: "dark".to_string(),
                language: "en".to_string(),
                auto_save: true,
                auto_save_interval: 300,
                default_bpm: 120.0,
                default_time_signature: (4, 4),
            })),
            midi_settings: Arc::new(Mutex::new(midi_settings)),
            monitoring_handle: Arc::new(Mutex::new(None)),
        }
    }
}

#[command]
pub async fn get_system_info(state: State<'_, SystemState>) -> Result<SystemInfo, String> {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let memory_percentage = (memory_used as f32 / memory_total as f32) * 100.0;

    // Get disk usage from all mounted disks
    let disks = Disks::new_with_refreshed_list();
    let (disk_used, disk_total) = disks.iter().fold((0u64, 0u64), |(used, total), disk| {
        (
            used + (disk.total_space() - disk.available_space()),
            total + disk.total_space(),
        )
    });
    let disk_percentage = if disk_total > 0 {
        (disk_used as f32 / disk_total as f32) * 100.0
    } else {
        0.0
    };

    Ok(SystemInfo {
        cpu_usage,
        memory_used,
        memory_total,
        memory_percentage,
        disk_used,
        disk_total,
        disk_percentage,
        platform: std::env::consts::OS.to_string(),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
    })
}

// Note: get_audio_devices moved to settings.rs to avoid duplicate command definitions

// Note: get_midi_devices moved to settings.rs to avoid duplicate command definitions

// Note: get_settings moved to settings.rs (comprehensive implementation) to avoid duplicate command definitions

#[command]
pub async fn get_system_settings(state: State<'_, SystemState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.clone())
}

#[command]
pub async fn update_settings(
    state: State<'_, SystemState>,
    updates: std::collections::HashMap<String, serde_json::Value>,
) -> Result<AppSettings, String> {
    let mut settings = state.settings.lock().unwrap();

    for (key, value) in updates {
        match key.as_str() {
            "audio_buffer_size" => {
                if let Ok(size) = serde_json::from_value::<u32>(value) {
                    settings.audio_buffer_size = size.clamp(64, 4096);
                }
            },
            "sample_rate" => {
                if let Ok(rate) = serde_json::from_value::<u32>(value) {
                    settings.sample_rate = rate;
                }
            },
            "theme" => {
                if let Ok(theme) = serde_json::from_value::<String>(value) {
                    settings.theme = theme;
                }
            },
            "language" => {
                if let Ok(lang) = serde_json::from_value::<String>(value) {
                    settings.language = lang;
                }
            },
            "auto_save" => {
                if let Ok(auto_save) = serde_json::from_value::<bool>(value) {
                    settings.auto_save = auto_save;
                }
            },
            "auto_save_interval" => {
                if let Ok(interval) = serde_json::from_value::<u32>(value) {
                    settings.auto_save_interval = interval.clamp(60, 3600);
                }
            },
            "default_bpm" => {
                if let Ok(bpm) = serde_json::from_value::<f32>(value) {
                    settings.default_bpm = bpm.clamp(30.0, 300.0);
                }
            },
            "default_time_signature" => {
                if let Ok(ts) = serde_json::from_value::<(u8, u8)>(value) {
                    settings.default_time_signature = ts;
                }
            },
            _ => {},
        }
    }

    Ok(settings.clone())
}

#[command]
pub async fn start_system_monitoring(
    state: State<'_, SystemState>,
    window: Window,
) -> Result<(), String> {
    let mut handle = state.monitoring_handle.lock().unwrap();

    if handle.is_none() {
        let window_clone = window.clone();
        let state_clone = Arc::new(state.inner().clone());
        let spawn_handle = tokio::spawn(monitoring_loop(window_clone, state_clone));
        *handle = Some(spawn_handle);
    }

    Ok(())
}

#[command]
pub async fn stop_system_monitoring(state: State<'_, SystemState>) -> Result<(), String> {
    let mut handle = state.monitoring_handle.lock().unwrap();
    *handle = None;
    Ok(())
}

// =============================================================================
// DEVICE MANAGEMENT COMMANDS
// =============================================================================

#[command]
pub async fn set_audio_device(
    state: State<'_, SystemState>,
    device_id: String,
) -> Result<(), String> {
    // Store the selected audio device in MIDI settings (for routing purposes)
    let mut midi_settings = state.midi_settings.lock().unwrap();

    // Create or update audio output routing config
    let config = MidiDeviceConfig {
        device_id: device_id.clone(),
        device_name: format!("Audio: {}", device_id),
        is_input: false,
        is_output: true,
        enabled: true,
        auto_connect: true,
        port_assignments: HashMap::new(),
        routing: MidiRouteConfig::default(),
    };
    midi_settings.set_device(config);
    midi_settings.save()?;

    info!("Set audio device: {}", device_id);
    Ok(())
}

#[command]
pub async fn set_midi_device(
    state: State<'_, SystemState>,
    device_id: String,
    is_input: bool,
) -> Result<(), String> {
    let mut midi_settings = state.midi_settings.lock().unwrap();

    // Update last used device
    if is_input {
        midi_settings.last_input_device = Some(device_id.clone());
    } else {
        midi_settings.last_output_device = Some(device_id.clone());
    }

    // Create or update device config
    let config = MidiDeviceConfig {
        device_id: device_id.clone(),
        device_name: device_id.clone(), // Will be updated when device is enumerated
        is_input,
        is_output: !is_input,
        enabled: true,
        auto_connect: true,
        port_assignments: HashMap::new(),
        routing: MidiRouteConfig {
            source_channel: None, // All channels
            dest_channel: None,   // Pass-through
            transpose: 0,
            velocity_scale: 1.0,
        },
    };
    midi_settings.set_device(config);
    midi_settings.save()?;

    info!(
        "Set MIDI {} device: {}",
        if is_input { "input" } else { "output" },
        device_id
    );
    Ok(())
}

/// Get persisted MIDI device settings
#[command]
pub async fn get_midi_device_settings(
    state: State<'_, SystemState>,
) -> Result<MidiDeviceSettings, String> {
    let midi_settings = state.midi_settings.lock().unwrap();
    Ok(midi_settings.clone())
}

/// Update MIDI device configuration
#[command]
pub async fn update_midi_device_config(
    state: State<'_, SystemState>,
    config: MidiDeviceConfig,
) -> Result<(), String> {
    let mut midi_settings = state.midi_settings.lock().unwrap();
    midi_settings.set_device(config);
    midi_settings.save()?;
    Ok(())
}

/// Get last used MIDI devices for auto-reconnect
#[command]
pub async fn get_last_midi_devices(
    state: State<'_, SystemState>,
) -> Result<(Option<String>, Option<String>), String> {
    let midi_settings = state.midi_settings.lock().unwrap();
    Ok((
        midi_settings.last_input_device.clone(),
        midi_settings.last_output_device.clone(),
    ))
}

// =============================================================================
// RESOURCE MONITORING COMMANDS
// =============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuUsage {
    pub global: f32,
    pub per_core: Vec<f32>,
    pub process: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryUsage {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub swap_used: u64,
    pub swap_total: u64,
}

#[command]
pub async fn get_cpu_usage(state: State<'_, SystemState>) -> Result<CpuUsage, String> {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_all();

    let global = sys.global_cpu_info().cpu_usage();
    let per_core: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    // Process CPU usage (approximation)
    let process = global * 0.1; // Placeholder - real impl would use process-specific data

    Ok(CpuUsage { global, per_core, process })
}

#[command]
pub async fn get_memory_usage(state: State<'_, SystemState>) -> Result<MemoryUsage, String> {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_memory();

    let used_bytes = sys.used_memory();
    let total_bytes = sys.total_memory();
    let percentage = (used_bytes as f32 / total_bytes as f32) * 100.0;
    let swap_used = sys.used_swap();
    let swap_total = sys.total_swap();

    Ok(MemoryUsage { used_bytes, total_bytes, percentage, swap_used, swap_total })
}

// Monitoring loop function
async fn monitoring_loop(window: Window, state: Arc<SystemState>) {
    let mut interval = interval(Duration::from_secs(2));

    loop {
        interval.tick().await;

        // Check if monitoring should continue
        {
            let handle = state.monitoring_handle.lock().unwrap();
            if handle.is_none() {
                break;
            }
        }

        let sys_info = {
            let mut sys = state.system.lock().unwrap();
            sys.refresh_all();

            SystemInfo {
                cpu_usage: sys.global_cpu_info().cpu_usage(),
                memory_used: sys.used_memory(),
                memory_total: sys.total_memory(),
                memory_percentage: (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0,
                disk_used: sys.used_memory(),   // Placeholder
                disk_total: sys.total_memory(), // Placeholder
                disk_percentage: (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0,
                platform: std::env::consts::OS.to_string(),
                os_version: "Unknown".to_string(),
            }
        };

        let _ = window.emit("system::info-update", sys_info);
    }
}

// =============================================================================
// FRONTEND ERROR LOGGING
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct FrontendError {
    pub message: String,
    pub severity: String,
    pub category: String,
    pub component: Option<String>,
    pub stack: Option<String>,
    pub timestamp: i64,
}

#[command]
pub async fn log_frontend_error(error: FrontendError) -> Result<(), String> {
    let component = error.component.as_deref().unwrap_or("unknown");

    match error.severity.as_str() {
        "critical" | "error" => {
            error!(
                category = %error.category,
                component = %component,
                timestamp = %error.timestamp,
                stack = ?error.stack,
                "Frontend error: {}",
                error.message
            );
        },
        "warning" => {
            warn!(
                category = %error.category,
                component = %component,
                "Frontend warning: {}",
                error.message
            );
        },
        _ => {
            info!(
                category = %error.category,
                component = %component,
                "Frontend info: {}",
                error.message
            );
        },
    }

    Ok(())
}
