# MIDI Software Center - User Guide

> **Status:** Draft - To be completed when Streams A-F finish
> **Last Updated:** 2025-12-17

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [VIP3 Browser](#vip3-browser)
4. [DAW Mixer](#daw-mixer)
5. [Automation](#automation)
6. [Project Management](#project-management)
7. [Collections & Saved Searches](#collections--saved-searches)
8. [Keyboard Shortcuts](#keyboard-shortcuts)
9. [Tips & Tricks](#tips--tricks)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

### What is MIDI Software Center?

MIDI Software Center is a comprehensive MIDI file management and production tool that combines:

- **VIP3 Browser**: Advanced MIDI file browser with intelligent filtering
- **DAW**: Digital Audio Workstation for arranging and mixing MIDI sequences
- **Pipeline**: Batch processing tools for importing, analyzing, and organizing MIDI files

### Key Features

- **2.15M+ Files** indexed with intelligent tagging
- **97 Instrument Tags** automatically detected
- **Real-time MIDI Sequencing** with hardware integration
- **Professional Mixer** with effects and automation
- **Smart Collections** for organizing your library
- **BPM & Key Detection** for all files

---

## Getting Started

### Installation

```bash
# TODO: Add installation instructions
# - Download from releases
# - Run installer
# - First launch setup wizard
```

### First Launch

1. **Database Setup**: The app will create/connect to the PostgreSQL database
2. **Import Files**: Start by importing your MIDI collection (Pipeline → Import)
3. **Browse**: Use VIP3 Browser to explore your library
4. **Create**: Start a new project in the DAW

### System Requirements

- **OS**: Windows 10+, macOS 11+, or Linux
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: Depends on MIDI library size (index is ~10% of collection)
- **Database**: PostgreSQL 16+ (included in setup)

---

## VIP3 Browser

> **Status:** Complete when Stream A (filter counts) and Stream C (collections) finish

### Overview

VIP3 Browser is a powerful MIDI file browser inspired by Native Instruments' Komplete Kontrol. It features:

- **Multi-column filtering** (Folders, Instruments, BPM, Key, etc.)
- **Real-time filter counts** showing how many files match each filter
- **Instant search results** (< 100ms typical)
- **Drag & Drop** to DAW sequencer

### Filtering Files

#### Using the Column Browser

1. **Folders Column**: Click a folder to filter by location
2. **Instruments Column**: Click instruments to filter by tag (piano, drums, bass, etc.)
3. **BPM Column**: Use slider or click BPM ranges
4. **Key Column**: Click musical keys (C, D♭, etc.)
5. **Additional Columns**: Time signature, track count, timbres, styles

#### Filter Counts

Each filter item shows a count badge:

```
[Folders]           [Instruments]      [BPM]
 ├─ Jazz (245) ✓     ├─ Piano (189)     ├─ <100 (45)
 ├─ Classical (12)   ├─ Drums (156) ✓   ├─ 100-120 (89) ✓
 └─ Rock (0)         └─ Bass (78)       └─ 120-140 (67)
```

- **Bold numbers**: Available matches for this filter
- **Grayed out (0)**: No files match this filter with current selection
- **Checkmark (✓)**: Currently active filter

#### Combining Filters

Filters work together (AND logic):

```
Folder: Jazz + Instrument: Piano + BPM: 120-140 = 23 results
```

Each time you add a filter, the counts update in <50ms to show what's still available.

#### Clearing Filters

- **Single filter**: Click the active filter again to remove it
- **All filters**: Click "Clear All" button (top right)
- **Keyboard**: Press `Escape` to clear all filters

### Search Bar

TODO: Add full-text search documentation when implemented

### Results Grid

#### Viewing Results

- **Grid view**: Thumbnails with metadata overlay
- **List view**: Detailed table with sortable columns
- **Toggle**: Switch between views with view mode button

#### File Information

Each result shows:
- Filename
- BPM (if detected)
- Key (if detected)
- Instrument tags
- Duration
- Track count

#### Sorting

Click column headers to sort:
- Name (A-Z)
- BPM (low to high)
- Date added (newest first)
- Duration

### Drag & Drop to DAW

1. **Single file**: Click and drag a file to the DAW sequencer
2. **Multiple files**: Ctrl+click to select multiple, then drag
3. **Track creation**: Each file creates a new track in the DAW

---

## DAW Mixer

> **Status:** Complete when Stream B (mixer commands) finishes

### Overview

The DAW Mixer provides professional mixing capabilities:

- **Up to 128 tracks** (tested with 100+)
- **Real-time VU meters** (60 FPS)
- **Effect chains** per track
- **Flexible routing** with buses
- **Master channel** with limiting

### Channel Strip Controls

#### Gain Fader

- **Range**: -60 dB to +12 dB
- **Default**: 0 dB (unity gain)
- **Controls**:
  - Drag: Adjust gain
  - Shift+Drag: Fine control (0.1 dB steps)
  - Double-click: Reset to 0 dB
  - Scroll wheel: ±1 dB steps

#### Pan Knob

- **Range**: -1.0 (hard left) to +1.0 (hard right)
- **Default**: 0.0 (center)
- **Display**: L50, L25, C, R25, R50, etc.
- **Double-click**: Reset to center

#### Mute Button

- **Click**: Toggle mute on/off
- **Indicator**: Red when muted
- **Keyboard**: `M` key (with channel selected)

#### Solo Button

- **Click**: Solo this track (mute all others)
- **Ctrl+Click**: Add to solo (multiple tracks soloed)
- **Indicator**: Yellow when solo
- **Keyboard**: `S` key

### VU Meters

- **Type**: Peak + RMS metering
- **Update rate**: 60 Hz
- **Range**: -60 dB to +12 dB
- **Peak hold**: 2-second hold time
- **Clipping indicator**: Red >0 dB warning

### Effect Rack

#### Adding Effects

1. Click **FX** button on channel strip
2. Click **+** in effect rack
3. Select effect from menu:
   - EQ (3-band parametric)
   - Compressor
   - Reverb
   - Delay
   - TODO: Additional effects

#### Effect Chain

- Effects process **top to bottom** in chain
- **Drag** effects to reorder
- **Bypass** button: Temporarily disable effect
- **Remove** button (X): Delete effect

#### Effect Parameters

TODO: Document effect parameters when implemented

### Routing & Buses

TODO: Document routing when Stream B completes bus implementation

### Presets

#### Saving Presets

1. Configure channel settings
2. Click preset menu
3. "Save Preset"
4. Enter name
5. Saved for future use

#### Loading Presets

1. Click preset menu
2. Select preset from list
3. All settings applied instantly

---

## Automation

> **Status:** Complete when Stream D (automation) finishes

### Overview

Automation allows you to record and playback parameter changes over time:

- **Supported parameters**: Gain, Pan, Send levels, Effect parameters
- **Recording modes**: Off, Read, Write, Latch, Touch
- **Curve types**: Linear, Stepped, Bezier, Exponential
- **Editing**: Full visual editor with point manipulation

### Automation Modes

#### Off
- Automation is ignored
- Manual control only

#### Read
- Playback recorded automation
- Manual control temporarily overrides

#### Write
- **Always recording**
- All parameter changes create automation points
- Use for initial automation passes

#### Latch
- Starts recording when parameter is first changed
- Continues recording even if control is released
- Good for setting and holding values

#### Touch
- Records only while control is actively being changed
- Stops recording when control is released
- Good for fine-tuning existing automation

### Recording Automation

1. **Select parameter**: Choose what to automate (Gain, Pan, etc.)
2. **Set mode**: Choose Write, Latch, or Touch
3. **Start playback**: Transport → Play
4. **Move parameter**: Adjust fader/knob as desired
5. **Stop playback**: Automation is saved
6. **Set to Read mode**: Play back to hear automation

### Editing Automation

#### Adding Points

- **Click** on automation lane to add point
- **Snap to grid**: Hold `Alt` to disable snap

#### Moving Points

- **Drag** point to new position
- **Shift+Drag**: Constrain to horizontal or vertical
- **Arrow keys**: Nudge selected points

#### Selecting Points

- **Click**: Select single point
- **Ctrl+Click**: Add to selection
- **Box select**: Drag selection rectangle
- **Ctrl+A**: Select all points

#### Deleting Points

- **Backspace/Delete**: Remove selected points
- **Right-click** → Delete

#### Changing Curve Type

- **Right-click point** → Curve Type → Linear/Stepped/Bezier

### Copy/Paste

- **Ctrl+C**: Copy selected points
- **Ctrl+X**: Cut selected points
- **Ctrl+V**: Paste at playhead position

---

## Project Management

> **Status:** Complete when Stream E (projects) finishes

### Creating Projects

1. DAW → File → New Project
2. Enter project name
3. Set BPM (default: 120)
4. Set time signature (default: 4/4)

### Saving Projects

- **Ctrl+S**: Quick save
- **File** → Save As: Save with new name
- **Auto-save**: Every 5 minutes (configurable)

### Loading Projects

- **File** → Open Project
- **Recent projects**: Quick access list
- **Drag & drop**: Drop .midip file onto app

### Exporting

#### Export to MIDI

1. File → Export → MIDI File
2. Choose format:
   - **Type 0**: Single track (all merged)
   - **Type 1**: Multi-track (preserves tracks)
3. Select path and export

#### TODO: Export to Audio (Future)

---

## Collections & Saved Searches

> **Status:** Complete when Stream C finishes

### Saved Searches

#### Creating

1. Apply filters in VIP3 Browser
2. Click **Save Search** button
3. Enter name (e.g., "Jazz Piano 120 BPM")
4. Search saved to sidebar

#### Using

- Click saved search to instantly apply filters
- Edit/Delete via right-click menu

### Collections

#### Creating Collections

1. VIP3 → Collections → New Collection
2. Enter name and description
3. Add files by:
   - Select files → Add to Collection
   - Drag files to collection

#### Managing Collections

- **Reorder files**: Drag within collection
- **Remove files**: Right-click → Remove
- **Export collection**: Save as playlist file

### Favorites

- **Star icon** on any file to add to Favorites
- **Favorites filter** shows all favorited files

---

## Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New project |
| `Ctrl+O` | Open project |
| `Ctrl+S` | Save project |
| `Ctrl+Q` | Quit |
| `F11` | Fullscreen |

### VIP3 Browser

| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Focus search |
| `Escape` | Clear filters |
| `↑↓` | Navigate results |
| `Enter` | Load selected file |

### DAW Mixer

| Shortcut | Action |
|----------|--------|
| `M` | Mute selected track |
| `S` | Solo selected track |
| `←→` | Select previous/next track |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |

### Automation

| Shortcut | Action |
|----------|--------|
| `Ctrl+A` | Select all points |
| `Delete` | Delete selected points |
| `Ctrl+C/X/V` | Copy/Cut/Paste points |

### Transport

| Shortcut | Action |
|----------|--------|
| `Space` | Play/Pause |
| `Home` | Return to start |
| `R` | Start recording |

---

## Tips & Tricks

### Performance Tips

1. **Use filter counts**: They update in real-time to guide your search
2. **Collections for workflows**: Create collections for common combinations
3. **Keyboard shortcuts**: Learn shortcuts for 10x faster workflow
4. **Pin frequently used folders**: Right-click → Pin

### Organization Tips

1. **Tag consistently**: The auto-tagger is good, but manual review helps
2. **Use descriptive folder names**: VIP3 indexes folder structure
3. **Favorites for gems**: Star your best files for easy access

### Mixing Tips

1. **Start with levels**: Set gain before applying effects
2. **Use buses**: Group similar instruments (drums, melody, etc.)
3. **Automate send effects**: Create dynamic reverb/delay
4. **Save presets**: Build a library of go-to channel settings

---

## Troubleshooting

### VIP3 Browser Issues

#### Filters not working
- Check database connection (Status bar)
- Rebuild filter indexes: Settings → Database → Rebuild Indexes
- Check filter counts: If all show (0), filters may be too restrictive

#### Slow filter counts
- Target is <50ms. If slower:
  - Check PostgreSQL performance
  - Verify indexes exist: `\d+ files` in psql
  - Check system resources

### DAW Issues

#### Audio dropouts
- Increase buffer size: Settings → Audio → Buffer Size
- Close other audio applications
- Check CPU usage

#### MIDI not working
- Check MIDI connections: Settings → MIDI Devices
- Verify device permissions (macOS/Linux)
- Restart audio engine

### General Issues

#### Database connection failed
- Verify PostgreSQL is running: `docker-compose ps`
- Check connection string in settings
- Check firewall rules

#### Application crashes
- Check logs: `~/.midi-software-center/logs`
- Report issue with log file

---

## Getting Help

- **Documentation**: https://github.com/yourusername/midi-software-center/docs
- **Issues**: https://github.com/yourusername/midi-software-center/issues
- **Discussions**: https://github.com/yourusername/midi-software-center/discussions

---

**TODO: Complete remaining sections when features are implemented**
