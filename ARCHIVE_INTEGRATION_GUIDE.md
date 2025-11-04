# 🎵 Archive Integration Guide: Pro Tools-Like Window System

**Status**: Implementation Plan for MIDI Software Center
**Date**: 2025-11-03
**Goal**: Extract archive patterns and implement advanced window management system like Pro Tools

---

## 📋 Table of Contents

1. [What to Extract from Archive](#what-to-extract)
2. [Pro Tools Window System Architecture](#pro-tools-architecture)
3. [Implementation Steps](#implementation-steps)
4. [Code Examples](#code-examples)
5. [Integration with Current Project](#integration)

---

## 📦 What to Extract from Archive {#what-to-extract}

### 1. ✅ Menu System Pattern

**Source**: `/tmp/midi-library-system/src-tauri/src/main.rs` (lines 44-103)

**What to Extract**:
```rust
// Archive uses this pattern - we can adapt it
fn setup_menu() -> Menu {
    let file_menu = Submenu::new("File", Menu::new()...);
    let view_menu = Submenu::new("View", Menu::new()...);
    // etc
}
```

**For Current Project**: Adapt to create window management menus
- View → Windows submenu (show/hide windows)
- Arrange windows submenu (tile, cascade, full-screen)
- Save/Load layouts submenu

### 2. ✅ Global Shortcuts Pattern

**Source**: `/tmp/midi-library-system/src-tauri/src/main.rs` (lines 113-148)

**What to Extract**:
```rust
fn setup_global_shortcuts(app: &AppHandle) -> Result<(), AppError> {
    let mut manager = app.global_shortcut_manager();
    manager.register("Space", move || {
        let _ = app.emit_all("shortcut:play-pause", ());
    })
}
```

**For Current Project**: Add window shortcuts
- `Cmd+1` → Show Pipeline
- `Cmd+2` → Show DAW
- `Cmd+3` → Show Database
- `Cmd+~` → Cycle windows

### 3. ✅ System Tray Pattern

**Source**: `/tmp/midi-library-system/src-tauri/src/main.rs` (lines 105-111)

**What to Extract**:
```rust
fn setup_system_tray() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show", "Show"))
        .add_item(CustomMenuItem::new("hide", "Hide"))
}
```

**For Current Project**: Extend with window menu items
- Show Pipeline
- Show DAW
- Show Database
- Minimize/Restore

### 4. ✅ State Management Pattern

**Source**: `/tmp/midi-library-system/src-tauri/src/state.rs`

**What to Extract**:
```rust
pub struct AppState {
    pub db_pool: SqlitePool,
    pub search_client: MeilisearchClient,
    pub midi_controller: Arc<Mutex<MidiController>>,
}
```

**For Current Project**: Extend with window state
```rust
pub struct WindowState {
    pub pipeline_window: Option<Window>,
    pub daw_window: Option<Window>,
    pub database_window: Option<Window>,
    pub layout: WindowLayout,
    pub window_positions: HashMap<String, Position>,
}
```

### 5. ✅ Event Loop Pattern

**Source**: `/tmp/midi-library-system/src-tauri/src/main.rs` (lines 274-293)

**What to Extract**:
```rust
app.run(|app_handle, event| {
    match event {
        RunEvent::WindowEvent { label, event, .. } => {
            match event {
                WindowEvent::CloseRequested { .. } => { ... }
                WindowEvent::Focused(focused) => { ... }
            }
        }
    }
});
```

**For Current Project**: Use for window lifecycle management
- Track window open/close
- Persist window state
- Handle focus changes

---

## 🎨 Pro Tools Window System Architecture {#pro-tools-architecture}

### Multi-Window System Like Pro Tools

```
┌─────────────────────────────────────────────────────┐
│  MAIN APPLICATION WINDOW                            │
│  ┌─────────────────────────────────────────────────┐│
│  │  Menu Bar (File, Edit, View, Windows, Help)     ││
│  ├─────────────────────────────────────────────────┤│
│  │  Toolbar                                         ││
│  ├─────────────┬──────────────────┬────────────────┤│
│  │  Sidebar    │  Main Content    │  Inspector     ││
│  │             │                  │                ││
│  │  Library    │  Pipeline        │  Properties   ││
│  │  ├─ Mixer   │  ├─ Import       │  ├─ File Info  ││
│  │  ├─ Browser │  ├─ Analyze      │  ├─ Tags       ││
│  │  └─ Search  │  └─ Preview      │  └─ Metadata   ││
│  │             │                  │                ││
│  └─────────────┴──────────────────┴────────────────┘│
│                                                     │
│  SECONDARY WINDOWS (Floating/Dockable)              │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐│
│  │  DAW Window  │  │Database Brwsr│  │Settings    ││
│  │              │  │              │  │            ││
│  │ ├─ Timeline  │  │ ├─ Files     │  │ Audio      ││
│  │ ├─ Piano     │  │ ├─ Queries   │  │ MIDI       ││
│  │ ├─ Mixer     │  │ └─ Stats     │  │ Display    ││
│  │ └─ Controls  │  │              │  │            ││
│  └──────────────┘  └──────────────┘  └────────────┘│
└─────────────────────────────────────────────────────┘
```

### Window Types

| Type | Behavior | Example |
|------|----------|---------|
| **Main** | Persistent, can't close | Pipeline/DAW main editor |
| **Dockable** | Can be attached to panels | Inspector, Properties |
| **Floating** | Independent windows | Database Browser, Settings |
| **Modal** | Blocks interaction | Dialogs, Preferences |
| **Palette** | Always-on-top tools | Transport, Tool palettes |

### Layout System

```typescript
interface WindowLayout {
  id: string;
  name: string;
  windows: {
    [key: string]: WindowPosition;
  };
  panels: {
    [key: string]: PanelState;
  };
  timestamp: number;
}

interface WindowPosition {
  x: number;
  y: number;
  width: number;
  height: number;
  maximized: boolean;
  docked?: string; // Dock target
}
```

---

## 🔧 Implementation Steps {#implementation-steps}

### Phase 1: Foundation (Weeks 1-2)

**Step 1.1: Create Window Manager Module**

Location: `pipeline/src-tauri/src/windows/mod.rs`

```rust
pub mod manager;
pub mod layout;
pub mod state;
pub mod events;

pub use manager::WindowManager;
pub use layout::Layout;
```

**Step 1.2: Adapt Archive's State Pattern**

Location: `pipeline/src-tauri/src/windows/state.rs`

```rust
use std::collections::HashMap;
use tauri::Window;

#[derive(Clone)]
pub struct WindowState {
    pub windows: HashMap<String, WindowInfo>,
    pub current_layout: String,
    pub saved_layouts: HashMap<String, Layout>,
}

#[derive(Clone, Debug)]
pub struct WindowInfo {
    pub label: String,
    pub title: String,
    pub window_type: WindowType,
    pub position: Position,
    pub docked_to: Option<String>,
    pub visible: bool,
}

#[derive(Clone, Debug)]
pub enum WindowType {
    Main,
    Dockable,
    Floating,
    Modal,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
```

**Step 1.3: Window Manager Implementation**

Location: `pipeline/src-tauri/src/windows/manager.rs`

```rust
use tauri::{Manager, AppHandle, Window};
use crate::windows::state::{WindowState, WindowInfo, WindowType, Position};

pub struct WindowManager {
    state: WindowState,
}

impl WindowManager {
    pub fn new() -> Self {
        WindowManager {
            state: WindowState {
                windows: Default::default(),
                current_layout: "default".to_string(),
                saved_layouts: Default::default(),
            }
        }
    }

    pub fn open_window(&mut self, app: &AppHandle, label: &str, window_type: WindowType) -> Result<(), String> {
        // Create new window based on type
        match window_type {
            WindowType::Main => self.create_main_window(app, label)?,
            WindowType::Dockable => self.create_dockable_window(app, label)?,
            WindowType::Floating => self.create_floating_window(app, label)?,
            WindowType::Modal => self.create_modal_window(app, label)?,
        }

        Ok(())
    }

    pub fn close_window(&mut self, label: &str) -> Result<(), String> {
        if let Some(window) = self.state.windows.remove(label) {
            Ok(())
        } else {
            Err(format!("Window {} not found", label))
        }
    }

    pub fn toggle_window(&mut self, app: &AppHandle, label: &str) -> Result<(), String> {
        if let Some(window) = self.state.windows.get(label) {
            if window.visible {
                self.hide_window(label)?;
            } else {
                self.show_window(label)?;
            }
        }
        Ok(())
    }

    pub fn save_layout(&mut self, name: String) -> Result<(), String> {
        let layout = Layout {
            name: name.clone(),
            windows: self.get_current_positions()?,
        };
        self.state.saved_layouts.insert(name, layout);
        Ok(())
    }

    pub fn load_layout(&mut self, app: &AppHandle, name: &str) -> Result<(), String> {
        if let Some(layout) = self.state.saved_layouts.get(name) {
            for (label, pos) in &layout.windows {
                self.set_window_position(app, label, pos.clone())?;
            }
            self.state.current_layout = name.to_string();
            Ok(())
        } else {
            Err(format!("Layout {} not found", name))
        }
    }

    // Helper methods
    fn create_main_window(&mut self, app: &AppHandle, label: &str) -> Result<(), String> {
        // Implementation
        Ok(())
    }

    fn create_dockable_window(&mut self, app: &AppHandle, label: &str) -> Result<(), String> {
        // Implementation
        Ok(())
    }

    fn create_floating_window(&mut self, app: &AppHandle, label: &str) -> Result<(), String> {
        // Implementation
        Ok(())
    }

    fn create_modal_window(&mut self, app: &AppHandle, label: &str) -> Result<(), String> {
        // Implementation
        Ok(())
    }

    fn show_window(&mut self, label: &str) -> Result<(), String> {
        if let Some(window) = self.state.windows.get_mut(label) {
            window.visible = true;
            Ok(())
        } else {
            Err(format!("Window {} not found", label))
        }
    }

    fn hide_window(&mut self, label: &str) -> Result<(), String> {
        if let Some(window) = self.state.windows.get_mut(label) {
            window.visible = false;
            Ok(())
        } else {
            Err(format!("Window {} not found", label))
        }
    }

    fn set_window_position(&mut self, app: &AppHandle, label: &str, pos: Position) -> Result<(), String> {
        if let Some(window) = self.state.windows.get_mut(label) {
            window.position = pos;
            Ok(())
        } else {
            Err(format!("Window {} not found", label))
        }
    }

    fn get_current_positions(&self) -> Result<std::collections::HashMap<String, Position>, String> {
        let mut positions = std::collections::HashMap::new();
        for (label, info) in &self.state.windows {
            positions.insert(label.clone(), info.position.clone());
        }
        Ok(positions)
    }
}
```

### Phase 2: Tauri Integration (Weeks 3-4)

**Step 2.1: Adapt Archive's Menu System**

Location: `pipeline/src-tauri/src/windows/menu.rs`

```rust
use tauri::{Menu, Submenu, MenuItem, CustomMenuItem};

pub fn create_window_menu() -> Submenu {
    Submenu::new(
        "Windows",
        Menu::new()
            .add_item(CustomMenuItem::new("show_pipeline", "Show Pipeline"))
            .add_item(CustomMenuItem::new("show_daw", "Show DAW"))
            .add_item(CustomMenuItem::new("show_database", "Show Database"))
            .add_item(CustomMenuItem::new("show_settings", "Show Settings"))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("arrange_tile", "Tile Windows"))
            .add_item(CustomMenuItem::new("arrange_cascade", "Cascade Windows"))
            .add_native_item(MenuItem::Separator)
            .add_item(CustomMenuItem::new("save_layout", "Save Layout..."))
            .add_item(CustomMenuItem::new("load_layout", "Load Layout..."))
    )
}
```

**Step 2.2: Adapt Archive's Global Shortcuts**

Location: `pipeline/src-tauri/src/windows/shortcuts.rs`

```rust
use tauri::{AppHandle, GlobalShortcutManager};

pub fn setup_window_shortcuts(app: &AppHandle) -> Result<(), String> {
    let mut manager = app.global_shortcut_manager();

    // Window navigation shortcuts
    let show_pipeline = app.clone();
    manager.register("CmdOrCtrl+1", move || {
        let _ = show_pipeline.emit_all("command:show-pipeline", ());
    }).map_err(|e| e.to_string())?;

    let show_daw = app.clone();
    manager.register("CmdOrCtrl+2", move || {
        let _ = show_daw.emit_all("command:show-daw", ());
    }).map_err(|e| e.to_string())?;

    let show_database = app.clone();
    manager.register("CmdOrCtrl+3", move || {
        let _ = show_database.emit_all("command:show-database", ());
    }).map_err(|e| e.to_string())?;

    // Cycle windows
    let cycle_windows = app.clone();
    manager.register("CmdOrCtrl+`", move || {
        let _ = cycle_windows.emit_all("command:cycle-windows", ());
    }).map_err(|e| e.to_string())?;

    Ok(())
}
```

**Step 2.3: Adapt Archive's Event Loop**

Location: `pipeline/src-tauri/src/main.rs` modifications

```rust
// In main() function, add to the Tauri builder:

.on_window_event(|event| {
    match event.event() {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            let window = event.window();

            // Save window state before closing
            if let Ok(manager) = window.app_handle().try_invoke_handler("save_window_state") {
                // Window state saved
            }

            // Allow close for non-main windows
            if window.label() != "main" {
                // Save position before closing
            }
        }
        tauri::WindowEvent::Moved(pos) => {
            // Update window position in state
        }
        tauri::WindowEvent::Resized(size) => {
            // Update window size in state
        }
        tauri::WindowEvent::Focused(is_focused) => {
            if *is_focused {
                // Update focused window in state
            }
        }
        _ => {}
    }
})
```

### Phase 3: Tauri Commands (Weeks 5-6)

**Step 3.1: Window Control Commands**

Location: `pipeline/src-tauri/src/windows/commands.rs`

```rust
use tauri::command;
use crate::windows::manager::WindowManager;

#[command]
pub async fn show_window(label: String) -> Result<(), String> {
    // Implementation
    Ok(())
}

#[command]
pub async fn hide_window(label: String) -> Result<(), String> {
    // Implementation
    Ok(())
}

#[command]
pub async fn toggle_window(label: String) -> Result<(), String> {
    // Implementation
    Ok(())
}

#[command]
pub async fn save_layout(name: String) -> Result<(), String> {
    // Implementation
    Ok(())
}

#[command]
pub async fn load_layout(name: String) -> Result<(), String> {
    // Implementation
    Ok(())
}

#[command]
pub async fn get_layout_list() -> Result<Vec<String>, String> {
    // Implementation
    Ok(vec![])
}

#[command]
pub async fn arrange_windows(arrangement: String) -> Result<(), String> {
    match arrangement.as_str() {
        "tile" => {
            // Tile arrangement logic
            Ok(())
        }
        "cascade" => {
            // Cascade arrangement logic
            Ok(())
        }
        _ => Err("Unknown arrangement".to_string())
    }
}

#[command]
pub async fn get_window_state() -> Result<WindowState, String> {
    // Return current window state
    Ok(Default::default())
}
```

### Phase 4: Frontend Integration (Weeks 7-8)

**Step 4.1: Window Management Store**

Location: `pipeline/src/lib/stores/windowStore.ts`

```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface WindowInfo {
  label: string;
  title: string;
  visible: boolean;
  position: { x: number; y: number };
  size: { width: number; height: number };
}

function createWindowStore() {
  const { subscribe, set, update } = writable<{
    windows: Record<string, WindowInfo>;
    layout: string;
  }>({
    windows: {},
    layout: 'default'
  });

  return {
    subscribe,

    async showWindow(label: string) {
      await invoke('show_window', { label });
      update(state => ({
        ...state,
        windows: {
          ...state.windows,
          [label]: {
            ...state.windows[label],
            visible: true
          }
        }
      }));
    },

    async hideWindow(label: string) {
      await invoke('hide_window', { label });
      update(state => ({
        ...state,
        windows: {
          ...state.windows,
          [label]: {
            ...state.windows[label],
            visible: false
          }
        }
      }));
    },

    async toggleWindow(label: string) {
      await invoke('toggle_window', { label });
      // Update will be received via event
    },

    async saveLayout(name: string) {
      await invoke('save_layout', { name });
      update(state => ({ ...state, layout: name }));
    },

    async loadLayout(name: string) {
      await invoke('load_layout', { name });
      update(state => ({ ...state, layout: name }));
    },

    async getLayoutList() {
      return await invoke<string[]>('get_layout_list');
    }
  };
}

export const windowStore = createWindowStore();
```

**Step 4.2: Window Menu Component**

Location: `pipeline/src/lib/components/WindowsMenu.svelte`

```svelte
<script lang="ts">
  import { windowStore } from '../stores/windowStore';

  let layouts: string[] = [];
  let showSaveDialog = false;
  let layoutName = '';

  onMount(async () => {
    layouts = await windowStore.getLayoutList();
  });

  async function saveLayout() {
    if (layoutName.trim()) {
      await windowStore.saveLayout(layoutName);
      layouts = await windowStore.getLayoutList();
      layoutName = '';
      showSaveDialog = false;
    }
  }

  async function loadLayout(name: string) {
    await windowStore.loadLayout(name);
  }
</script>

<div class="windows-menu">
  <button on:click={() => windowStore.showWindow('pipeline')}>
    Show Pipeline
  </button>
  <button on:click={() => windowStore.showWindow('daw')}>
    Show DAW
  </button>
  <button on:click={() => windowStore.showWindow('database')}>
    Show Database
  </button>

  <hr />

  <div class="layouts">
    <h4>Layouts</h4>
    {#each layouts as layout (layout)}
      <button on:click={() => loadLayout(layout)}>
        {layout}
      </button>
    {/each}
    <button on:click={() => showSaveDialog = true}>
      Save Layout
    </button>
  </div>

  {#if showSaveDialog}
    <div class="dialog">
      <input
        bind:value={layoutName}
        placeholder="Layout name"
        on:keydown={e => e.key === 'Enter' && saveLayout()}
      />
      <button on:click={saveLayout}>Save</button>
      <button on:click={() => showSaveDialog = false}>Cancel</button>
    </div>
  {/if}
</div>

<style>
  .windows-menu {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  button {
    padding: 0.5rem 1rem;
    cursor: pointer;
  }

  .layouts {
    border-top: 1px solid #ccc;
    padding-top: 0.5rem;
  }

  .dialog {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
</style>
```

---

## 💻 Code Examples {#code-examples}

### Full Window Manager Example

Create file: `pipeline/src-tauri/src/windows/mod.rs`

```rust
pub mod manager;
pub mod state;
pub mod commands;
pub mod menu;
pub mod shortcuts;

pub use manager::WindowManager;
pub use state::{WindowState, WindowInfo, WindowType, Position};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Layout {
    pub name: String,
    pub windows: std::collections::HashMap<String, Position>,
}
```

### Initialize in main.rs

Modify: `pipeline/src-tauri/src/main.rs`

```rust
mod windows;

use windows::WindowManager;

async fn initialize_app_state(app: &AppHandle) -> Result<AppState, AppError> {
    // ... existing code ...

    // Initialize window manager
    let window_manager = WindowManager::new();
    app.manage(Arc::new(tokio::sync::Mutex::new(window_manager)));
    info!("✅ Window manager initialized");

    Ok(app_state)
}

fn main() {
    let app = tauri::Builder::default()
        .menu(create_app_menu()) // Include window menu
        .setup(|app| {
            // ... existing setup ...

            // Setup window shortcuts
            windows::shortcuts::setup_window_shortcuts(app.handle())?;

            Ok(())
        })
        .on_window_event(|event| {
            // Handle window events
            match event.event() {
                tauri::WindowEvent::CloseRequested { .. } => {
                    // Save state
                }
                tauri::WindowEvent::Moved(_) => {
                    // Update position
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            windows::commands::show_window,
            windows::commands::hide_window,
            windows::commands::toggle_window,
            windows::commands::save_layout,
            windows::commands::load_layout,
            windows::commands::get_layout_list,
            windows::commands::arrange_windows,
            windows::commands::get_window_state,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        match event {
            // ... existing event handling ...
            _ => {}
        }
    });
}
```

---

## 🔗 Integration with Current Project {#integration}

### Current Project Structure

```
pipeline/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs ........................ Entry point
│   │   ├── commands/
│   │   │   ├── file_import.rs ........... (42 tests)
│   │   │   ├── analyze.rs .............. (35 tests)
│   │   │   ├── split_file.rs ........... (27 tests)
│   │   │   └── archive_import.rs ....... (20 tests)
│   │   ├── db/
│   │   │   ├── repositories/
│   │   │   │   ├── file_repository.rs .. (109 tests)
│   │   │   │   ├── tag_repository.rs ... (100 tests)
│   │   │   │   ├── metadata_repository.rs (79 tests)
│   │   │   │   └── search_repository.rs (82 tests)
│   │   ├── core/ ......................... Analysis modules
│   │   └── ????? ......................... NEW: windows/ module
│   └── Cargo.toml
├── src/
│   ├── lib/
│   │   ├── components/ .................. Svelte UI
│   │   ├── stores/ ...................... State management
│   │   └── api/ ......................... API clients
│   └── App.svelte
└── tauri.conf.json
```

### Where to Add Window System

**New Directory Structure**:

```
pipeline/src-tauri/src/windows/
├── mod.rs ........................ Module definition
├── manager.rs ................... Window manager (main logic)
├── state.rs ..................... State structures
├── commands.rs .................. Tauri commands
├── menu.rs ...................... Menu creation
├── shortcuts.rs ................. Global shortcuts
├── layout.rs .................... Layout persistence
└── events.rs .................... Event handlers
```

**Update Files**:

1. `pipeline/src-tauri/src/main.rs` - Add window initialization
2. `pipeline/src/lib/stores/windowStore.ts` - Create new store
3. `pipeline/src/lib/components/WindowMenu.svelte` - Create new component
4. `pipeline/tauri.conf.json` - Define secondary windows

### Integration Checklist

- [ ] Create `windows/` module in `pipeline/src-tauri/src/`
- [ ] Implement `WindowManager` with state tracking
- [ ] Add window management Tauri commands
- [ ] Create Window menu in main menu bar
- [ ] Add keyboard shortcuts (Cmd+1, Cmd+2, etc.)
- [ ] Implement layout save/load persistence
- [ ] Create frontend store for window state
- [ ] Create Windows menu component
- [ ] Add window arrangement logic (tile, cascade)
- [ ] Implement system tray window controls
- [ ] Add window position persistence on app close
- [ ] Write integration tests for window manager
- [ ] Document window system in project README

---

## 📊 Phase Timeline

```
Week 1-2: Foundation
  └─ Windows module structure
  └─ Window manager implementation
  └─ State management

Week 3-4: Tauri Integration
  └─ Menu system adaptation
  └─ Shortcuts setup
  └─ Event loop integration

Week 5-6: Commands & Backend
  └─ Window control commands
  └─ Layout persistence
  └─ Window arrangement logic

Week 7-8: Frontend Integration
  └─ Window store creation
  └─ Menu components
  └─ Layout management UI

Week 9-10: Testing & Refinement
  └─ Integration tests
  └─ UI/UX refinement
  └─ Performance optimization

Week 11-12: Documentation
  └─ API documentation
  └─ User guide
  └─ Developer guide
```

---

## 🎯 Success Criteria

✅ **Functionality**
- [ ] Can open/close 3+ independent windows
- [ ] Can dock windows together
- [ ] Can save and restore layouts
- [ ] Keyboard shortcuts work
- [ ] System tray controls windows

✅ **Performance**
- [ ] Window operations < 100ms
- [ ] Layout switching < 500ms
- [ ] No memory leaks
- [ ] Smooth window animations

✅ **User Experience**
- [ ] Intuitive menu system
- [ ] Responsive shortcuts
- [ ] Persistent layouts
- [ ] Multi-monitor support

✅ **Code Quality**
- [ ] 80%+ test coverage
- [ ] Comprehensive error handling
- [ ] Well-documented API
- [ ] Follows project conventions

---

## 🚀 Quick Start

### To Begin Implementation

1. **Copy archive reference files**:
```bash
cp /tmp/midi-library-system/src-tauri/src/main.rs \
   /tmp/archive-main-reference.rs
```

2. **Review key patterns**:
- Menu system (archive: lines 44-103)
- Shortcuts (archive: lines 113-148)
- Event loop (archive: lines 274-293)

3. **Create windows module**:
```bash
mkdir -p pipeline/src-tauri/src/windows
touch pipeline/src-tauri/src/windows/mod.rs
touch pipeline/src-tauri/src/windows/manager.rs
touch pipeline/src-tauri/src/windows/state.rs
touch pipeline/src-tauri/src/windows/commands.rs
```

4. **Start with Phase 1 (Foundation)**

---

**Next Step**: Would you like me to start implementing the window system by creating the foundation modules?

