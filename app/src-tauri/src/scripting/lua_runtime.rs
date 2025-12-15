#![allow(dead_code)]

// =============================================================================
// LUA SCRIPTING RUNTIME
// =============================================================================
// Embeddable Lua scripting for automation, macros, and MIDI processing.
//
// CLAUDE CODE INSTRUCTIONS:
// 1. Add to Cargo.toml: mlua = { version = "0.9", features = ["lua54", "async"] }
// 2. Location: daw/src-tauri/src/scripting/lua_runtime.rs
// 3. Scripts stored in user scripts folder
//
// FEATURES:
// - MIDI event processing
// - Transport control
// - Parameter automation
// - Custom generators (arpeggiator, chord, etc.)
// - File operations
// - Timer/scheduling
// =============================================================================

use mlua::{Function, Lua, Result as LuaResult, Table};
use parking_lot::RwLock;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Script metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScriptInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub enabled: bool,
    pub path: String,
    pub script_type: ScriptType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ScriptType {
    MidiProcessor, // Transforms MIDI in real-time
    Generator,     // Generates MIDI (arpeggiator, etc.)
    Automation,    // Parameter automation
    Utility,       // General utility
    Action,        // One-shot action
}

/// Action sent from script to application
#[derive(Debug, Clone)]
pub enum ScriptAction {
    // MIDI
    SendMidi { device: String, data: Vec<u8> },

    // Transport
    Play,
    Stop,
    Record,
    SetTempo(f64),
    SetPosition(u64),

    // Parameters
    SetParameter { path: String, value: f64 },

    // UI
    ShowMessage { title: String, message: String },
    Log(String),

    // File
    LoadMidi(String),
    SaveMidi { path: String, data: Vec<u8> },
}

/// Lua scripting runtime
pub struct LuaRuntime {
    lua: Lua,
    scripts: RwLock<HashMap<String, ScriptInfo>>,
    action_tx: mpsc::UnboundedSender<ScriptAction>,
}

impl LuaRuntime {
    /// Create new Lua runtime
    pub fn new() -> LuaResult<(Self, mpsc::UnboundedReceiver<ScriptAction>)> {
        let lua = Lua::new();
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let runtime = Self { lua, scripts: RwLock::new(HashMap::new()), action_tx };

        runtime.register_api()?;

        Ok((runtime, action_rx))
    }

    /// Register the scripting API
    fn register_api(&self) -> LuaResult<()> {
        let globals = self.lua.globals();

        // MIDI API
        self.register_midi_api(&globals)?;

        // Transport API
        self.register_transport_api(&globals)?;

        // Parameter API
        self.register_parameter_api(&globals)?;

        // Music theory API
        self.register_theory_api(&globals)?;

        // Utility API
        self.register_utility_api(&globals)?;

        // Timer API
        self.register_timer_api(&globals)?;

        Ok(())
    }

    fn register_midi_api(&self, globals: &Table) -> LuaResult<()> {
        let midi = self.lua.create_table()?;
        let tx = self.action_tx.clone();

        // midi.send(device, status, data1, data2)
        let tx_clone = tx.clone();
        midi.set(
            "send",
            self.lua.create_function(
                move |_, (device, status, data1, data2): (String, u8, u8, u8)| {
                    let _ = tx_clone
                        .send(ScriptAction::SendMidi { device, data: vec![status, data1, data2] });
                    Ok(())
                },
            )?,
        )?;

        // midi.note_on(device, channel, note, velocity)
        let tx_clone = tx.clone();
        midi.set(
            "note_on",
            self.lua.create_function(
                move |_, (device, channel, note, velocity): (String, u8, u8, u8)| {
                    let status = 0x90 | (channel & 0x0F);
                    let _ = tx_clone.send(ScriptAction::SendMidi {
                        device,
                        data: vec![status, note & 0x7F, velocity & 0x7F],
                    });
                    Ok(())
                },
            )?,
        )?;

        // midi.note_off(device, channel, note)
        let tx_clone = tx.clone();
        midi.set(
            "note_off",
            self.lua.create_function(move |_, (device, channel, note): (String, u8, u8)| {
                let status = 0x80 | (channel & 0x0F);
                let _ = tx_clone
                    .send(ScriptAction::SendMidi { device, data: vec![status, note & 0x7F, 0] });
                Ok(())
            })?,
        )?;

        // midi.cc(device, channel, controller, value)
        let tx_clone = tx.clone();
        midi.set(
            "cc",
            self.lua.create_function(
                move |_, (device, channel, cc, value): (String, u8, u8, u8)| {
                    let status = 0xB0 | (channel & 0x0F);
                    let _ = tx_clone.send(ScriptAction::SendMidi {
                        device,
                        data: vec![status, cc & 0x7F, value & 0x7F],
                    });
                    Ok(())
                },
            )?,
        )?;

        // midi.program_change(device, channel, program)
        let tx_clone = tx.clone();
        midi.set(
            "program_change",
            self.lua
                .create_function(move |_, (device, channel, program): (String, u8, u8)| {
                    let status = 0xC0 | (channel & 0x0F);
                    let _ = tx_clone.send(ScriptAction::SendMidi {
                        device,
                        data: vec![status, program & 0x7F],
                    });
                    Ok(())
                })?,
        )?;

        // midi.pitch_bend(device, channel, value) - value: -8192 to +8191
        let tx_clone = tx.clone();
        midi.set(
            "pitch_bend",
            self.lua
                .create_function(move |_, (device, channel, value): (String, u8, i16)| {
                    let status = 0xE0 | (channel & 0x0F);
                    let unsigned = (value + 8192) as u16;
                    let lsb = (unsigned & 0x7F) as u8;
                    let msb = ((unsigned >> 7) & 0x7F) as u8;
                    let _ = tx_clone
                        .send(ScriptAction::SendMidi { device, data: vec![status, lsb, msb] });
                    Ok(())
                })?,
        )?;

        globals.set("midi", midi)?;
        Ok(())
    }

    fn register_transport_api(&self, globals: &Table) -> LuaResult<()> {
        let transport = self.lua.create_table()?;
        let tx = self.action_tx.clone();

        let tx_clone = tx.clone();
        transport.set(
            "play",
            self.lua.create_function(move |_, ()| {
                let _ = tx_clone.send(ScriptAction::Play);
                Ok(())
            })?,
        )?;

        let tx_clone = tx.clone();
        transport.set(
            "stop",
            self.lua.create_function(move |_, ()| {
                let _ = tx_clone.send(ScriptAction::Stop);
                Ok(())
            })?,
        )?;

        let tx_clone = tx.clone();
        transport.set(
            "record",
            self.lua.create_function(move |_, ()| {
                let _ = tx_clone.send(ScriptAction::Record);
                Ok(())
            })?,
        )?;

        let tx_clone = tx.clone();
        transport.set(
            "set_tempo",
            self.lua.create_function(move |_, bpm: f64| {
                let _ = tx_clone.send(ScriptAction::SetTempo(bpm));
                Ok(())
            })?,
        )?;

        let tx_clone = tx.clone();
        transport.set(
            "set_position",
            self.lua.create_function(move |_, ticks: u64| {
                let _ = tx_clone.send(ScriptAction::SetPosition(ticks));
                Ok(())
            })?,
        )?;

        globals.set("transport", transport)?;
        Ok(())
    }

    fn register_parameter_api(&self, globals: &Table) -> LuaResult<()> {
        let params = self.lua.create_table()?;
        let tx = self.action_tx.clone();

        let tx_clone = tx.clone();
        params.set(
            "set",
            self.lua.create_function(move |_, (path, value): (String, f64)| {
                let _ = tx_clone.send(ScriptAction::SetParameter { path, value });
                Ok(())
            })?,
        )?;

        globals.set("params", params)?;
        Ok(())
    }

    fn register_theory_api(&self, globals: &Table) -> LuaResult<()> {
        let theory = self.lua.create_table()?;

        // Scales
        let scales = self.lua.create_table()?;
        scales.set("major", vec![0, 2, 4, 5, 7, 9, 11])?;
        scales.set("minor", vec![0, 2, 3, 5, 7, 8, 10])?;
        scales.set("harmonic_minor", vec![0, 2, 3, 5, 7, 8, 11])?;
        scales.set("melodic_minor", vec![0, 2, 3, 5, 7, 9, 11])?;
        scales.set("dorian", vec![0, 2, 3, 5, 7, 9, 10])?;
        scales.set("phrygian", vec![0, 1, 3, 5, 7, 8, 10])?;
        scales.set("lydian", vec![0, 2, 4, 6, 7, 9, 11])?;
        scales.set("mixolydian", vec![0, 2, 4, 5, 7, 9, 10])?;
        scales.set("locrian", vec![0, 1, 3, 5, 6, 8, 10])?;
        scales.set("pentatonic_major", vec![0, 2, 4, 7, 9])?;
        scales.set("pentatonic_minor", vec![0, 3, 5, 7, 10])?;
        scales.set("blues", vec![0, 3, 5, 6, 7, 10])?;
        scales.set("chromatic", vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])?;
        theory.set("scales", scales)?;

        // Chords
        let chords = self.lua.create_table()?;
        chords.set("major", vec![0, 4, 7])?;
        chords.set("minor", vec![0, 3, 7])?;
        chords.set("dim", vec![0, 3, 6])?;
        chords.set("aug", vec![0, 4, 8])?;
        chords.set("sus2", vec![0, 2, 7])?;
        chords.set("sus4", vec![0, 5, 7])?;
        chords.set("maj7", vec![0, 4, 7, 11])?;
        chords.set("min7", vec![0, 3, 7, 10])?;
        chords.set("dom7", vec![0, 4, 7, 10])?;
        chords.set("dim7", vec![0, 3, 6, 9])?;
        chords.set("m7b5", vec![0, 3, 6, 10])?;
        chords.set("maj9", vec![0, 4, 7, 11, 14])?;
        chords.set("min9", vec![0, 3, 7, 10, 14])?;
        chords.set("dom9", vec![0, 4, 7, 10, 14])?;
        theory.set("chords", chords)?;

        // Helper functions
        theory.set(
            "note_name",
            self.lua.create_function(|_, note: u8| {
                let names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
                let octave = (note / 12) as i8 - 1;
                Ok(format!("{}{}", names[(note % 12) as usize], octave))
            })?,
        )?;

        theory.set(
            "name_to_note",
            self.lua.create_function(|_, name: String| {
                let notes = [("C", 0), ("D", 2), ("E", 4), ("F", 5), ("G", 7), ("A", 9), ("B", 11)];
                let chars: Vec<char> = name.chars().collect();
                if chars.is_empty() {
                    return Ok(60u8);
                }

                let mut note = 0i8;
                let mut idx = 0;

                // Base note
                for (n, val) in &notes {
                    if chars[0].to_uppercase().to_string() == *n {
                        note = *val as i8;
                        idx = 1;
                        break;
                    }
                }

                // Accidentals
                while idx < chars.len() {
                    match chars[idx] {
                        '#' => note += 1,
                        'b' => note -= 1,
                        _ => break,
                    }
                    idx += 1;
                }

                // Octave
                let octave: i8 = name[idx..].parse().unwrap_or(4);

                Ok(((octave + 1) * 12 + note).clamp(0, 127) as u8)
            })?,
        )?;

        theory.set(
            "transpose",
            self.lua.create_function(|_, (note, semitones): (u8, i8)| {
                Ok((note as i16 + semitones as i16).clamp(0, 127) as u8)
            })?,
        )?;

        theory.set(
            "in_scale",
            self.lua.create_function(|_, (note, root, scale): (u8, u8, Vec<i32>)| {
                let interval = ((note as i32 - root as i32) % 12 + 12) % 12;
                Ok(scale.contains(&interval))
            })?,
        )?;

        globals.set("theory", theory)?;
        Ok(())
    }

    fn register_utility_api(&self, globals: &Table) -> LuaResult<()> {
        let util = self.lua.create_table()?;
        let tx = self.action_tx.clone();

        // Logging
        let tx_clone = tx.clone();
        util.set(
            "log",
            self.lua.create_function(move |_, message: String| {
                let _ = tx_clone.send(ScriptAction::Log(message));
                Ok(())
            })?,
        )?;

        // Message dialog
        let tx_clone = tx.clone();
        util.set(
            "message",
            self.lua.create_function(move |_, (title, message): (String, String)| {
                let _ = tx_clone.send(ScriptAction::ShowMessage { title, message });
                Ok(())
            })?,
        )?;

        // Random
        util.set(
            "random",
            self.lua.create_function(|_, (min, max): (i32, i32)| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
                let range = (max - min + 1) as u64;
                Ok(min + (seed % range) as i32)
            })?,
        )?;

        util.set(
            "random_float",
            self.lua.create_function(|_, ()| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
                Ok((seed % 10000) as f64 / 10000.0)
            })?,
        )?;

        // Clamp
        util.set(
            "clamp",
            self.lua.create_function(|_, (value, min, max): (f64, f64, f64)| {
                Ok(value.clamp(min, max))
            })?,
        )?;

        // Lerp
        util.set(
            "lerp",
            self.lua.create_function(|_, (a, b, t): (f64, f64, f64)| {
                Ok(a + (b - a) * t.clamp(0.0, 1.0))
            })?,
        )?;

        // Map range
        util.set(
            "map",
            self.lua.create_function(
                |_, (value, in_min, in_max, out_min, out_max): (f64, f64, f64, f64, f64)| {
                    let t = (value - in_min) / (in_max - in_min);
                    Ok(out_min + (out_max - out_min) * t)
                },
            )?,
        )?;

        globals.set("util", util)?;
        Ok(())
    }

    fn register_timer_api(&self, globals: &Table) -> LuaResult<()> {
        // Timer functions would need async support
        // Simplified version here
        let timer = self.lua.create_table()?;

        timer.set(
            "sleep_ms",
            self.lua.create_function(|_, ms: u64| {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                Ok(())
            })?,
        )?;

        globals.set("timer", timer)?;
        Ok(())
    }

    /// Load and register a script
    pub fn load_script(&self, info: ScriptInfo, source: &str) -> LuaResult<()> {
        // Compile script
        self.lua.load(source).exec()?;

        // Store info
        self.scripts.write().insert(info.id.clone(), info);

        Ok(())
    }

    /// Execute a script function with no arguments, returning a value
    pub fn call_function_no_args<T: for<'a> mlua::FromLuaMulti<'a>>(
        &self,
        func_name: &str,
    ) -> LuaResult<T> {
        let globals = self.lua.globals();
        let func: Function = globals.get(func_name)?;
        func.call(())
    }

    /// Execute a script function with arguments, returning a value
    pub fn call_function_with_args<A, T>(&self, func_name: &str, args: A) -> LuaResult<T>
    where
        A: for<'a> mlua::IntoLuaMulti<'a>,
        T: for<'a> mlua::FromLuaMulti<'a>,
    {
        let globals = self.lua.globals();
        let func: Function = globals.get(func_name)?;
        func.call(args)
    }

    /// Process MIDI through all processor scripts
    pub fn process_midi(
        &self,
        channel: u8,
        status: u8,
        data1: u8,
        data2: u8,
    ) -> Vec<(u8, u8, u8, u8)> {
        let mut results = vec![(channel, status, data1, data2)];

        // Call on_midi if defined
        if let Ok(func) = self.lua.globals().get::<_, Function>("on_midi") {
            if let Ok(result) = func.call::<_, Vec<Vec<u8>>>((channel, status, data1, data2)) {
                results.clear();
                for msg in result {
                    if msg.len() >= 4 {
                        results.push((msg[0], msg[1], msg[2], msg[3]));
                    }
                }
            }
        }

        results
    }

    /// Get script info
    pub fn list_scripts(&self) -> Vec<ScriptInfo> {
        self.scripts.read().values().cloned().collect()
    }

    /// Enable/disable script
    pub fn set_script_enabled(&self, id: &str, enabled: bool) {
        if let Some(script) = self.scripts.write().get_mut(id) {
            script.enabled = enabled;
        }
    }
}

// =============================================================================
// EXAMPLE SCRIPTS
// =============================================================================

pub const SCRIPT_ARPEGGIATOR: &str = r#"
-- Arpeggiator script
-- Hold notes and they will arpeggiate

local held_notes = {}
local arp_index = 1
local arp_rate = 120  -- BPM
local arp_mode = "up" -- up, down, updown, random

function on_midi(channel, status, data1, data2)
    local msg_type = bit32.band(status, 0xF0)
    
    if msg_type == 0x90 and data2 > 0 then
        -- Note on
        table.insert(held_notes, data1)
        table.sort(held_notes)
        return {}  -- Don't pass through, arp will play
        
    elseif msg_type == 0x80 or (msg_type == 0x90 and data2 == 0) then
        -- Note off
        for i, note in ipairs(held_notes) do
            if note == data1 then
                table.remove(held_notes, i)
                break
            end
        end
        return {}
    end
    
    -- Pass through other messages
    return {{channel, status, data1, data2}}
end

function arp_tick()
    if #held_notes == 0 then return end
    
    local note
    if arp_mode == "up" then
        note = held_notes[((arp_index - 1) % #held_notes) + 1]
    elseif arp_mode == "down" then
        note = held_notes[#held_notes - ((arp_index - 1) % #held_notes)]
    elseif arp_mode == "random" then
        note = held_notes[util.random(1, #held_notes)]
    end
    
    if note then
        midi.note_on("default", 0, note, 100)
        timer.sleep_ms(50)
        midi.note_off("default", 0, note)
    end
    
    arp_index = arp_index + 1
end
"#;

pub const SCRIPT_CHORD_TRIGGER: &str = r#"
-- Chord trigger script
-- Single notes trigger full chords

local chord_type = "maj7"  -- Current chord type

function on_midi(channel, status, data1, data2)
    local msg_type = bit32.band(status, 0xF0)
    
    if msg_type == 0x90 and data2 > 0 then
        -- Get chord intervals
        local chord = theory.chords[chord_type] or theory.chords.major
        local output = {}
        
        for _, interval in ipairs(chord) do
            local note = data1 + interval
            if note <= 127 then
                midi.note_on("default", channel, note, data2)
                table.insert(output, {channel, 0x90, note, data2})
            end
        end
        
        return output
        
    elseif msg_type == 0x80 or (msg_type == 0x90 and data2 == 0) then
        -- Note off for all chord notes
        local chord = theory.chords[chord_type] or theory.chords.major
        local output = {}
        
        for _, interval in ipairs(chord) do
            local note = data1 + interval
            if note <= 127 then
                midi.note_off("default", channel, note)
                table.insert(output, {channel, 0x80, note, 0})
            end
        end
        
        return output
    end
    
    return {{channel, status, data1, data2}}
end
"#;

pub const SCRIPT_VELOCITY_CURVE: &str = r#"
-- Velocity curve script
-- Apply custom velocity curves

local curve_type = "exponential"  -- linear, exponential, logarithmic, s-curve
local min_vel = 20
local max_vel = 127

function apply_curve(velocity)
    local normalized = velocity / 127
    local curved
    
    if curve_type == "linear" then
        curved = normalized
    elseif curve_type == "exponential" then
        curved = normalized * normalized
    elseif curve_type == "logarithmic" then
        curved = math.sqrt(normalized)
    elseif curve_type == "s-curve" then
        curved = 3 * normalized * normalized - 2 * normalized * normalized * normalized
    else
        curved = normalized
    end
    
    return math.floor(util.lerp(min_vel, max_vel, curved))
end

function on_midi(channel, status, data1, data2)
    local msg_type = bit32.band(status, 0xF0)
    
    if msg_type == 0x90 and data2 > 0 then
        local new_vel = apply_curve(data2)
        return {{channel, status, data1, new_vel}}
    end
    
    return {{channel, status, data1, data2}}
end
"#;

impl LuaRuntime {
    /// Load a script from name, code, and type (convenience method for Tauri commands)
    pub fn load_script_from_code(
        &self,
        name: &str,
        code: &str,
        script_type: ScriptType,
    ) -> LuaResult<()> {
        let info = ScriptInfo {
            id: name.to_string(),
            name: name.to_string(),
            description: String::new(),
            author: String::new(),
            version: "1.0.0".to_string(),
            enabled: true,
            path: String::new(),
            script_type,
        };
        self.load_script(info, code)
    }

    /// Unload a script by name
    pub fn unload_script(&self, name: &str) {
        self.scripts.write().remove(name);
    }
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

use std::sync::Mutex;
use tauri::State;

/// Thread-safe scripting state for Tauri
/// Stores script metadata and source code; creates LuaRuntime per operation
/// (Lua is not thread-safe, so we can't persist a single runtime across threads)
pub struct ScriptingState {
    scripts: Mutex<HashMap<String, (ScriptInfo, String)>>,
}

impl ScriptingState {
    pub fn new() -> Self {
        Self { scripts: Mutex::new(HashMap::new()) }
    }

    /// Create a temporary runtime with all scripts loaded
    fn create_runtime_with_scripts(
        &self,
    ) -> Result<(LuaRuntime, mpsc::UnboundedReceiver<ScriptAction>), String> {
        let (runtime, action_rx) = LuaRuntime::new().map_err(|e| e.to_string())?;

        // Load all enabled scripts into the runtime
        let scripts = self.scripts.lock().map_err(|e| e.to_string())?;
        for (info, code) in scripts.values() {
            if info.enabled {
                runtime.load_script(info.clone(), code).map_err(|e| e.to_string())?;
            }
        }

        Ok((runtime, action_rx))
    }
}

impl Default for ScriptingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Load a script (validates by compiling, stores for later execution)
#[tauri::command]
pub fn scripting_load_script(
    state: State<ScriptingState>,
    name: String,
    code: String,
    script_type: String,
) -> Result<(), String> {
    // Validate script by compiling it
    let lua = Lua::new();
    lua.load(&code)
        .into_function()
        .map_err(|e| format!("Script compilation error: {}", e))?;

    let stype = match script_type.as_str() {
        "generator" => ScriptType::Generator,
        "processor" => ScriptType::MidiProcessor,
        "automation" => ScriptType::Automation,
        "action" => ScriptType::Action,
        _ => ScriptType::Utility,
    };

    let info = ScriptInfo {
        id: name.clone(),
        name: name.clone(),
        description: String::new(),
        author: String::new(),
        version: "1.0.0".to_string(),
        enabled: true,
        path: String::new(),
        script_type: stype,
    };

    let mut scripts = state.inner().scripts.lock().map_err(|e| e.to_string())?;
    scripts.insert(name, (info, code));
    Ok(())
}

/// Unload a script
#[tauri::command]
pub fn scripting_unload_script(state: State<ScriptingState>, name: String) -> Result<(), String> {
    let mut scripts = state.inner().scripts.lock().map_err(|e| e.to_string())?;
    scripts.remove(&name);
    Ok(())
}

/// List all loaded scripts
#[tauri::command]
pub fn scripting_list_scripts(state: State<ScriptingState>) -> Result<Vec<ScriptInfo>, String> {
    let scripts = state.inner().scripts.lock().map_err(|e| e.to_string())?;
    Ok(scripts.values().map(|(info, _)| info.clone()).collect())
}

/// Run a function from a loaded script (creates runtime with full MIDI/transport API)
#[tauri::command]
pub fn scripting_run_function(
    state: State<ScriptingState>,
    _script_name: String,
    function_name: String,
) -> Result<(), String> {
    // Create a runtime with all scripts loaded
    let (runtime, _action_rx) = state.create_runtime_with_scripts()?;

    // Call the function with full API access
    runtime
        .call_function_no_args::<()>(&function_name)
        .map_err(|e| format!("Function call error: {}", e))
}

/// Process MIDI through all enabled processor scripts
#[tauri::command]
pub fn scripting_process_midi(
    state: State<ScriptingState>,
    channel: u8,
    status: u8,
    data1: u8,
    data2: u8,
) -> Result<Vec<Vec<u8>>, String> {
    let (runtime, _action_rx) = state.create_runtime_with_scripts()?;
    let results = runtime.process_midi(channel, status, data1, data2);
    Ok(results.iter().map(|(c, s, d1, d2)| vec![*c, *s, *d1, *d2]).collect())
}

/// Enable or disable a script
#[tauri::command]
pub fn scripting_set_enabled(
    state: State<ScriptingState>,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    let mut scripts = state.inner().scripts.lock().map_err(|e| e.to_string())?;
    if let Some((info, _)) = scripts.get_mut(&name) {
        info.enabled = enabled;
    }
    Ok(())
}

/// Get script info by name
#[tauri::command]
pub fn scripting_get_script(
    state: State<ScriptingState>,
    name: String,
) -> Result<Option<ScriptInfo>, String> {
    let scripts = state.inner().scripts.lock().map_err(|e| e.to_string())?;
    Ok(scripts.get(&name).map(|(info, _)| info.clone()))
}

#[tauri::command]
pub fn scripting_get_example_scripts() -> Vec<(String, String, String)> {
    vec![
        (
            "arpeggiator".to_string(),
            "processor".to_string(),
            SCRIPT_ARPEGGIATOR.to_string(),
        ),
        (
            "chord_trigger".to_string(),
            "processor".to_string(),
            SCRIPT_CHORD_TRIGGER.to_string(),
        ),
        (
            "velocity_curve".to_string(),
            "processor".to_string(),
            SCRIPT_VELOCITY_CURVE.to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let result = LuaRuntime::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_simple_script() {
        let (runtime, _rx) = LuaRuntime::new().unwrap();
        let result = runtime.lua.load("return 1 + 1").eval::<i32>();
        assert_eq!(result.unwrap(), 2);
    }
}
