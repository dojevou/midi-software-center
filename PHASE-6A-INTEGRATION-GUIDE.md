# Phase 6A Undo/Redo Backend - Integration Guide

Quick guide to integrate the undo/redo system into the DAW application.

## Step 1: Update main.rs

Add the undo/redo state and commands to the Tauri builder:

```rust
// In daw/src-tauri/src/main.rs

// Add import at top
use commands::undo_redo::UndoRedoState;

// In main() function, add to state management:
let undo_redo_state = UndoRedoState::new();

// Update Tauri builder:
tauri::Builder::default()
    .manage(state)
    .manage(midi_manager)
    .manage(sequencer_engine)
    .manage(daw_state)
    .manage(undo_redo_state)  // Add this line
    .invoke_handler(tauri::generate_handler![
        // ... existing commands ...

        // Add undo/redo commands:
        commands::undo_redo::undo,
        commands::undo_redo::redo,
        commands::undo_redo::can_undo,
        commands::undo_redo::can_redo,
        commands::undo_redo::undo_description,
        commands::undo_redo::redo_description,
        commands::undo_redo::clear_history,
        commands::undo_redo::undo_count,
        commands::undo_redo::redo_count,
        commands::undo_redo::undo_descriptions,
        commands::undo_redo::redo_descriptions,
        commands::undo_redo::memory_usage,
        commands::undo_redo::set_max_depth,
        commands::undo_redo::set_max_memory,
        commands::undo_redo::set_compression,
    ])
```

## Step 2: Update commands/mod.rs

Export the undo_redo commands:

```rust
// In daw/src-tauri/src/commands/mod.rs

pub mod undo_redo {
    pub use crate::undo_redo::commands::*;
}
```

## Step 3: TypeScript Type Definitions

Create a TypeScript types file for the undo/redo API:

```typescript
// daw/src/lib/api/undoRedo.ts

import { invoke } from '@tauri-apps/api/core';

export interface UndoRedoAPI {
  undo(): Promise<string>;
  redo(): Promise<string>;
  canUndo(): Promise<boolean>;
  canRedo(): Promise<boolean>;
  undoDescription(): Promise<string | null>;
  redoDescription(): Promise<string | null>;
  clearHistory(): Promise<void>;
  undoCount(): Promise<number>;
  redoCount(): Promise<number>;
  undoDescriptions(): Promise<string[]>;
  redoDescriptions(): Promise<string[]>;
  memoryUsage(): Promise<number>;
  setMaxDepth(depth: number): Promise<void>;
  setMaxMemory(bytes: number): Promise<void>;
  setCompression(enabled: boolean): Promise<void>;
}

export const undoRedo: UndoRedoAPI = {
  undo: () => invoke<string>('undo'),
  redo: () => invoke<string>('redo'),
  canUndo: () => invoke<boolean>('can_undo'),
  canRedo: () => invoke<boolean>('can_redo'),
  undoDescription: () => invoke<string | null>('undo_description'),
  redoDescription: () => invoke<string | null>('redo_description'),
  clearHistory: () => invoke<void>('clear_history'),
  undoCount: () => invoke<number>('undo_count'),
  redoCount: () => invoke<number>('redo_count'),
  undoDescriptions: () => invoke<string[]>('undo_descriptions'),
  redoDescriptions: () => invoke<string[]>('redo_descriptions'),
  memoryUsage: () => invoke<number>('memory_usage'),
  setMaxDepth: (depth: number) => invoke<void>('set_max_depth', { maxDepth: depth }),
  setMaxMemory: (bytes: number) => invoke<void>('set_max_memory', { maxMemory: bytes }),
  setCompression: (enabled: boolean) => invoke<void>('set_compression', { enabled }),
};
```

## Step 4: Keyboard Shortcuts

Add keyboard shortcuts for undo/redo:

```typescript
// daw/src/lib/shortcuts.ts

import { undoRedo } from './api/undoRedo';

export function setupKeyboardShortcuts() {
  document.addEventListener('keydown', async (e) => {
    // Ctrl+Z or Cmd+Z for undo
    if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
      e.preventDefault();
      try {
        const canUndo = await undoRedo.canUndo();
        if (canUndo) {
          const description = await undoRedo.undo();
          console.log(`Undid: ${description}`);
          // Update UI as needed
        }
      } catch (error) {
        console.error('Undo failed:', error);
      }
    }

    // Ctrl+Shift+Z or Ctrl+Y for redo
    if ((e.ctrlKey || e.metaKey) && (e.key === 'y' || (e.key === 'z' && e.shiftKey))) {
      e.preventDefault();
      try {
        const canRedo = await undoRedo.canRedo();
        if (canRedo) {
          const description = await undoRedo.redo();
          console.log(`Redid: ${description}`);
          // Update UI as needed
        }
      } catch (error) {
        console.error('Redo failed:', error);
      }
    }
  });
}

// Call in your main app initialization
// e.g., in +layout.svelte onMount()
```

## Step 5: Svelte Store (Optional)

Create a Svelte store for reactive undo/redo state:

```typescript
// daw/src/lib/stores/undoRedoStore.ts

import { writable, derived } from 'svelte/store';
import { undoRedo } from '$lib/api/undoRedo';

function createUndoRedoStore() {
  const canUndo = writable(false);
  const canRedo = writable(false);
  const undoDescription = writable<string | null>(null);
  const redoDescription = writable<string | null>(null);

  async function update() {
    canUndo.set(await undoRedo.canUndo());
    canRedo.set(await undoRedo.canRedo());
    undoDescription.set(await undoRedo.undoDescription());
    redoDescription.set(await undoRedo.redoDescription());
  }

  async function undo() {
    try {
      const description = await undoRedo.undo();
      await update();
      return description;
    } catch (error) {
      console.error('Undo failed:', error);
      throw error;
    }
  }

  async function redo() {
    try {
      const description = await undoRedo.redo();
      await update();
      return description;
    } catch (error) {
      console.error('Redo failed:', error);
      throw error;
    }
  }

  // Initialize
  update();

  return {
    canUndo: { subscribe: canUndo.subscribe },
    canRedo: { subscribe: canRedo.subscribe },
    undoDescription: { subscribe: undoDescription.subscribe },
    redoDescription: { subscribe: redoDescription.subscribe },
    undo,
    redo,
    update,
  };
}

export const undoRedoStore = createUndoRedoStore();
```

## Step 6: UI Components

Create undo/redo toolbar buttons:

```svelte
<!-- daw/src/lib/components/UndoRedoButtons.svelte -->

<script lang="ts">
  import { undoRedoStore } from '$lib/stores/undoRedoStore';
  import { ArrowUturnLeft, ArrowUturnRight } from '@heroicons/svelte/24/outline';

  async function handleUndo() {
    try {
      const description = await undoRedoStore.undo();
      console.log(`Undid: ${description}`);
    } catch (error) {
      console.error('Undo failed:', error);
    }
  }

  async function handleRedo() {
    try {
      const description = await undoRedoStore.redo();
      console.log(`Redid: ${description}`);
    } catch (error) {
      console.error('Redo failed:', error);
    }
  }
</script>

<div class="undo-redo-buttons">
  <button
    on:click={handleUndo}
    disabled={!$undoRedoStore.canUndo}
    title={$undoRedoStore.undoDescription || 'Undo'}
    class="btn-undo"
  >
    <ArrowUturnLeft class="w-5 h-5" />
    <span class="sr-only">Undo</span>
  </button>

  <button
    on:click={handleRedo}
    disabled={!$undoRedoStore.canRedo}
    title={$undoRedoStore.redoDescription || 'Redo'}
    class="btn-redo"
  >
    <ArrowUturnRight class="w-5 h-5" />
    <span class="sr-only">Redo</span>
  </button>
</div>

<style>
  .undo-redo-buttons {
    display: flex;
    gap: 0.25rem;
  }

  .btn-undo,
  .btn-redo {
    padding: 0.5rem;
    border-radius: 0.25rem;
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-undo:hover:not(:disabled),
  .btn-redo:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .btn-undo:disabled,
  .btn-redo:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
```

## Step 7: Testing Integration

Test the integration in your browser console:

```javascript
// Check if undo/redo is available
await invoke('can_undo'); // Should return false initially

// Execute some command (placeholder - replace with actual command)
// await invoke('add_note', { pitch: 60, velocity: 100, startTick: 0, duration: 480 });

// Check undo
await invoke('can_undo'); // Should return true

// Get undo description
await invoke('undo_description'); // Should return command description

// Undo
await invoke('undo'); // Should undo the command

// Redo
await invoke('redo'); // Should redo the command
```

## Step 8: Configuration (Optional)

Set custom limits in your app initialization:

```typescript
// daw/src/lib/init.ts

import { undoRedo } from './api/undoRedo';

export async function initializeApp() {
  // Set undo/redo limits
  await undoRedo.setMaxDepth(200); // Keep last 200 commands
  await undoRedo.setMaxMemory(20 * 1024 * 1024); // 20MB limit
  await undoRedo.setCompression(true); // Enable command compression

  console.log('Undo/redo initialized');
}
```

## Troubleshooting

### Commands not appearing in Tauri
- Check that you added the commands to `invoke_handler![]`
- Verify the module is exported in `commands/mod.rs`
- Rebuild the Rust code: `cargo build`

### TypeScript errors
- Check import paths are correct
- Ensure `@tauri-apps/api` is installed
- Verify TypeScript types match Rust return types

### Keyboard shortcuts not working
- Check event listener is registered
- Verify `preventDefault()` is called
- Check browser console for errors

### Undo/redo not working
- Check commands are being executed through CommandHistory
- Verify state is managed correctly
- Check Rust console logs for errors

## Next Steps

After integration:
1. Test undo/redo with all editor operations
2. Add visual feedback for undo/redo actions
3. Implement undo history panel
4. Add command grouping (composite commands)
5. Performance testing with large command histories

## Support

For issues or questions:
- Check the main documentation: `PHASE-6A-UNDO-REDO-BACKEND.md`
- Review test files for usage examples
- Check existing command implementations in `daw/src-tauri/src/undo_redo/`
