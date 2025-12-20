# Day 5, Part 5A: MIDI Export

**Duration:** 2 hours
**Prerequisites:** Days 1-4 complete
**Files to create:** 2

---

## Overview

Build MIDI export functionality:
1. MIDI exporter component
2. Export all tracks to MIDI file
3. Render automation to MIDI CC
4. Tempo and time signature handling
5. Tauri commands

---

## Step 1: MIDI Exporter (1 hour)

Create `app/src-tauri/src/daw/export/midi_exporter.rs`:

```rust
use crate::daw::automation::AutomationLane;
use crate::daw::mixer::TrackState;
use std::path::Path;
use std::fs::File;
use std::io::Write;

/// MIDI exporter
pub struct MidiExporter {
    ppq: u16, // Pulses per quarter note (default: 480)
}

impl MidiExporter {
    pub fn new(ppq: u16) -> Self {
        Self { ppq }
    }

    /// Export tracks to MIDI file
    pub fn export(
        &self,
        path: &Path,
        tracks: &[TrackState],
        automation_lanes: &[AutomationLane],
        bpm: f32,
        time_signature: (u8, u8),
    ) -> Result<(), String> {
        log::info!("Exporting to MIDI file: {:?}", path);

        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        // Write MIDI header
        self.write_header(&mut file, tracks.len() + 1)?; // +1 for tempo track

        // Write tempo track (track 0)
        self.write_tempo_track(&mut file, bpm, time_signature)?;

        // Write each track
        for (idx, track) in tracks.iter().enumerate() {
            self.write_track(&mut file, idx + 1, track, automation_lanes)?;
        }

        Ok(())
    }

    fn write_header(&self, file: &mut File, num_tracks: usize) -> Result<(), String> {
        // MThd chunk
        file.write_all(b"MThd")
            .map_err(|e| format!("Write error: {}", e))?;

        // Chunk length (always 6)
        file.write_all(&6u32.to_be_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // Format 1 (multiple tracks, synchronous)
        file.write_all(&1u16.to_be_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // Number of tracks
        file.write_all(&(num_tracks as u16).to_be_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // Division (pulses per quarter note)
        file.write_all(&self.ppq.to_be_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    fn write_tempo_track(
        &self,
        file: &mut File,
        bpm: f32,
        time_signature: (u8, u8),
    ) -> Result<(), String> {
        let mut track_data = Vec::new();

        // Time signature event at tick 0
        track_data.extend_from_slice(&Self::write_variable_length(0)); // Delta time
        track_data.push(0xFF); // Meta event
        track_data.push(0x58); // Time signature
        track_data.push(4); // Length
        track_data.push(time_signature.0); // Numerator
        track_data.push((time_signature.1 as f32).log2() as u8); // Denominator as power of 2
        track_data.push(24); // MIDI clocks per metronome click
        track_data.push(8); // 32nd notes per quarter note

        // Tempo event
        track_data.extend_from_slice(&Self::write_variable_length(0)); // Delta time
        track_data.push(0xFF); // Meta event
        track_data.push(0x51); // Set tempo
        track_data.push(3); // Length

        // Microseconds per quarter note
        let uspq = (60_000_000.0 / bpm) as u32;
        track_data.push(((uspq >> 16) & 0xFF) as u8);
        track_data.push(((uspq >> 8) & 0xFF) as u8);
        track_data.push((uspq & 0xFF) as u8);

        // End of track
        track_data.extend_from_slice(&Self::write_variable_length(0));
        track_data.push(0xFF);
        track_data.push(0x2F);
        track_data.push(0);

        // Write track chunk
        self.write_track_chunk(file, &track_data)?;

        Ok(())
    }

    fn write_track(
        &self,
        file: &mut File,
        track_num: usize,
        track: &TrackState,
        automation_lanes: &[AutomationLane],
    ) -> Result<(), String> {
        let mut track_data = Vec::new();

        // Track name
        track_data.extend_from_slice(&Self::write_variable_length(0)); // Delta time
        track_data.push(0xFF); // Meta event
        track_data.push(0x03); // Track name
        let name_bytes = track.name.as_bytes();
        track_data.extend_from_slice(&Self::write_variable_length(name_bytes.len() as u32));
        track_data.extend_from_slice(name_bytes);

        // Initial volume CC (CC7)
        let volume = ((track.gain_linear() * 127.0) as u8).min(127);
        track_data.extend_from_slice(&Self::write_variable_length(0));
        track_data.push(0xB0 | (track_num as u8 & 0x0F)); // Control change on channel
        track_data.push(7); // CC7 = Volume
        track_data.push(volume);

        // Initial pan CC (CC10)
        let pan = (((track.pan + 1.0) / 2.0) * 127.0) as u8;
        track_data.extend_from_slice(&Self::write_variable_length(0));
        track_data.push(0xB0 | (track_num as u8 & 0x0F));
        track_data.push(10); // CC10 = Pan
        track_data.push(pan);

        // Export automation as CC messages
        let track_automation: Vec<_> = automation_lanes
            .iter()
            .filter(|lane| lane.track_id == track.track_id)
            .collect();

        for lane in track_automation {
            self.write_automation_cc(&mut track_data, lane, track_num)?;
        }

        // End of track
        track_data.extend_from_slice(&Self::write_variable_length(0));
        track_data.push(0xFF);
        track_data.push(0x2F);
        track_data.push(0);

        // Write track chunk
        self.write_track_chunk(file, &track_data)?;

        Ok(())
    }

    fn write_automation_cc(
        &self,
        track_data: &mut Vec<u8>,
        lane: &AutomationLane,
        track_num: usize,
    ) -> Result<(), String> {
        if lane.points.is_empty() {
            return Ok(());
        }

        // Map parameter to CC number
        let cc_num = match lane.parameter_id.as_str() {
            "gain" | "volume" => 7,
            "pan" => 10,
            _ => return Ok(()), // Skip unknown parameters
        };

        let mut last_tick = 0u64;

        for point in &lane.points {
            let delta = (point.time_ticks - last_tick) as u32;
            let value = (point.value * 127.0) as u8;

            track_data.extend_from_slice(&Self::write_variable_length(delta));
            track_data.push(0xB0 | (track_num as u8 & 0x0F)); // Control change
            track_data.push(cc_num);
            track_data.push(value);

            last_tick = point.time_ticks;
        }

        Ok(())
    }

    fn write_track_chunk(&self, file: &mut File, track_data: &[u8]) -> Result<(), String> {
        // MTrk chunk
        file.write_all(b"MTrk")
            .map_err(|e| format!("Write error: {}", e))?;

        // Chunk length
        file.write_all(&(track_data.len() as u32).to_be_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        // Track data
        file.write_all(track_data)
            .map_err(|e| format!("Write error: {}", e))?;

        Ok(())
    }

    fn write_variable_length(mut value: u32) -> Vec<u8> {
        let mut result = Vec::new();
        let mut bytes = Vec::new();

        bytes.push((value & 0x7F) as u8);
        value >>= 7;

        while value > 0 {
            bytes.push(((value & 0x7F) | 0x80) as u8);
            value >>= 7;
        }

        bytes.reverse();
        result.extend_from_slice(&bytes);
        result
    }
}

impl Default for MidiExporter {
    fn default() -> Self {
        Self::new(480)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_length() {
        assert_eq!(MidiExporter::write_variable_length(0), vec![0]);
        assert_eq!(MidiExporter::write_variable_length(127), vec![127]);
        assert_eq!(MidiExporter::write_variable_length(128), vec![0x81, 0x00]);
        assert_eq!(MidiExporter::write_variable_length(16383), vec![0xFF, 0x7F]);
    }

    #[test]
    fn test_exporter_creation() {
        let exporter = MidiExporter::new(480);
        assert_eq!(exporter.ppq, 480);

        let default_exporter = MidiExporter::default();
        assert_eq!(default_exporter.ppq, 480);
    }
}
```

---

## Step 2: Export Module (10 min)

Create `app/src-tauri/src/daw/export/mod.rs`:

```rust
pub mod midi_exporter;

pub use midi_exporter::MidiExporter;
```

Update `app/src-tauri/src/daw/mod.rs`:

```rust
pub mod automation;
pub mod export;     // NEW
pub mod mixer;
pub mod presets;
pub mod project;
pub mod sequencer;
```

---

## Step 3: Tauri Commands (30 min)

Create `app/src-tauri/src/commands/daw/export_commands.rs`:

```rust
use crate::daw::export::MidiExporter;
use crate::AppState;
use tauri::State;
use std::path::PathBuf;

/// Export current session to MIDI file
#[tauri::command]
pub async fn export_to_midi(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Exporting to MIDI: {}", file_path);

    let path = PathBuf::from(&file_path);

    let sequencer = state.sequencer.lock().await;

    // Get current state
    let mixer = sequencer.get_mixer().await;
    let tracks: Vec<_> = mixer.tracks.values().cloned().collect();

    let automation = sequencer.get_automation_recorder();
    let automation_lanes = automation.get_all_lanes().await;

    // TODO: Get BPM and time signature from project
    let bpm = 120.0;
    let time_signature = (4, 4);

    // Export
    let exporter = MidiExporter::default();
    exporter.export(&path, &tracks, &automation_lanes, bpm, time_signature)?;

    log::info!("MIDI export completed: {:?}", path);

    Ok(())
}

/// Export with custom settings
#[tauri::command]
pub async fn export_to_midi_with_settings(
    file_path: String,
    bpm: f32,
    time_signature_num: u8,
    time_signature_denom: u8,
    ppq: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Exporting to MIDI with settings: {} BPM, {}/{}, {} PPQ",
        bpm,
        time_signature_num,
        time_signature_denom,
        ppq
    );

    let path = PathBuf::from(&file_path);

    let sequencer = state.sequencer.lock().await;

    // Get current state
    let mixer = sequencer.get_mixer().await;
    let tracks: Vec<_> = mixer.tracks.values().cloned().collect();

    let automation = sequencer.get_automation_recorder();
    let automation_lanes = automation.get_all_lanes().await;

    // Export
    let exporter = MidiExporter::new(ppq);
    exporter.export(
        &path,
        &tracks,
        &automation_lanes,
        bpm,
        (time_signature_num, time_signature_denom),
    )?;

    log::info!("MIDI export completed: {:?}", path);

    Ok(())
}
```

Register in `app/src-tauri/src/commands/daw/mod.rs`:

```rust
pub mod export_commands;
pub use export_commands::*;
```

Register in `main.rs`:

```rust
use midi_app::commands::daw::{
    export_to_midi,
    export_to_midi_with_settings,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing
            export_to_midi,
            export_to_midi_with_settings,
        ])
        // ...
}
```

---

## Step 4: Frontend Export Dialog (20 min)

Create `app/src/lib/components/DAW/ExportDialog.svelte`:

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';

  export let show = false;
  export let onClose: (() => void) | null = null;

  let exportPath = '';
  let bpm = 120;
  let timeSignatureNum = 4;
  let timeSignatureDenom = 4;
  let ppq = 480;
  let isExporting = false;

  async function handleBrowse() {
    const path = await save({
      filters: [
        {
          name: 'MIDI File',
          extensions: ['mid', 'midi'],
        },
      ],
      defaultPath: 'export.mid',
    });

    if (path) {
      exportPath = path;
    }
  }

  async function handleExport() {
    if (!exportPath) {
      alert('Please select export path');
      return;
    }

    isExporting = true;

    try {
      await invoke('export_to_midi_with_settings', {
        filePath: exportPath,
        bpm,
        timeSignatureNum,
        timeSignatureDenom,
        ppq,
      });

      alert('Export completed successfully!');
      show = false;

      if (onClose) {
        onClose();
      }
    } catch (error) {
      console.error('Export failed:', error);
      alert(`Export failed: ${error}`);
    } finally {
      isExporting = false;
    }
  }
</script>

{#if show}
  <div class="modal-overlay" on:click={() => (show = false)}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Export to MIDI</h3>
        <button class="close-btn" on:click={() => (show = false)}>✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="export-path">Export Path *</label>
          <div class="path-input-group">
            <input
              id="export-path"
              type="text"
              bind:value={exportPath}
              placeholder="/path/to/export.mid"
              class="form-input"
              readonly
            />
            <button class="browse-btn" on:click={handleBrowse}> Browse </button>
          </div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="export-bpm">BPM</label>
            <input
              id="export-bpm"
              type="number"
              bind:value={bpm}
              min="60"
              max="240"
              class="form-input"
            />
          </div>

          <div class="form-group">
            <label for="export-ts-num">Time Signature</label>
            <div class="ts-group">
              <input
                id="export-ts-num"
                type="number"
                bind:value={timeSignatureNum}
                min="1"
                max="16"
                class="form-input small"
              />
              <span>/</span>
              <input
                type="number"
                bind:value={timeSignatureDenom}
                min="1"
                max="16"
                class="form-input small"
              />
            </div>
          </div>
        </div>

        <div class="form-group">
          <label for="export-ppq">Pulses Per Quarter (PPQ)</label>
          <select id="export-ppq" bind:value={ppq} class="form-select">
            <option value={96}>96 PPQ</option>
            <option value={192}>192 PPQ</option>
            <option value={480}>480 PPQ (Recommended)</option>
            <option value={960}>960 PPQ</option>
          </select>
        </div>

        <div class="info-box">
          <p><strong>Note:</strong> Export includes:</p>
          <ul>
            <li>All tracks with initial volume and pan</li>
            <li>Automation rendered as MIDI CC messages</li>
            <li>Tempo and time signature</li>
          </ul>
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" on:click={() => (show = false)} disabled={isExporting}>
          Cancel
        </button>
        <button class="export-btn" on:click={handleExport} disabled={!exportPath || isExporting}>
          {isExporting ? 'Exporting...' : 'Export'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #161616;
    border-bottom: 1px solid #333;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .close-btn {
    background: none;
    color: #999;
    border: none;
    font-size: 20px;
    cursor: pointer;
  }

  .close-btn:hover {
    color: #fff;
  }

  .modal-body {
    padding: 16px;
    overflow-y: auto;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    color: #999;
  }

  .form-input,
  .form-select {
    width: 100%;
    padding: 8px 12px;
    background: #0a0a0a;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 13px;
  }

  .form-input.small {
    width: 60px;
  }

  .path-input-group {
    display: flex;
    gap: 8px;
  }

  .browse-btn {
    padding: 8px 16px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
  }

  .browse-btn:hover {
    background: #2563eb;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .ts-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .ts-group span {
    color: #999;
    font-size: 18px;
  }

  .info-box {
    background: #0a0a0a;
    border: 1px solid #333;
    border-radius: 4px;
    padding: 12px;
    margin-top: 16px;
  }

  .info-box p {
    margin: 0 0 8px 0;
    font-size: 13px;
    color: #999;
  }

  .info-box ul {
    margin: 0;
    padding-left: 20px;
    font-size: 12px;
    color: #666;
  }

  .info-box li {
    margin: 4px 0;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px;
    background: #161616;
    border-top: 1px solid #333;
  }

  .cancel-btn,
  .export-btn {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }

  .cancel-btn {
    background: #333;
    color: #fff;
  }

  .cancel-btn:hover:not(:disabled) {
    background: #3a3a3a;
  }

  .export-btn {
    background: #22c55e;
    color: #fff;
  }

  .export-btn:hover:not(:disabled) {
    background: #16a34a;
  }

  .export-btn:disabled,
  .cancel-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

---

## Verification (10 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib export
```

Test export:

```bash
npm run check
make dev
```

1. Open export dialog
2. Select export path
3. Adjust BPM and time signature
4. Click Export
5. Verify MIDI file created
6. Open exported file in another DAW (Reaper, Ableton, etc.)
7. Verify tempo, tracks, and automation

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| File not created | Check file permissions, verify path is valid |
| Invalid MIDI file | Verify header format, check variable-length encoding |
| Automation not exported | Ensure lanes have points, check CC mapping |
| Tempo wrong in DAW | Verify microseconds per quarter note calculation |

---

## What's Next?

✅ **You've completed:**
- MIDI exporter with automation rendering
- Export dialog with settings
- Tempo and time signature export
- CC7 (volume) and CC10 (pan) export
- Variable-length quantity encoding

**Next:** [Part 5B: Integration Testing](./DAY5_PART_B_INTEGRATION_TESTING.md)
- End-to-end workflow testing
- Automation record/playback verification
- Preset save/load testing
- Project save/load testing
- MIDI export validation
