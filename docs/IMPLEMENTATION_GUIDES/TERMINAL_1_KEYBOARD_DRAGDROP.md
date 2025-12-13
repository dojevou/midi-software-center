# TERMINAL 1: Keyboard Shortcuts & Drag/Drop System

## Owner: Claude Instance #1
## Components: Keyboard Shortcuts, Inter-Window Drag & Drop

---

## PART A: KEYBOARD SHORTCUTS SYSTEM

### A1. Create Keyboard Store (`app/src/lib/stores/keyboardStore.ts`)

```typescript
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TYPES
// ============================================================================

export type ModifierKey = 'ctrl' | 'alt' | 'shift' | 'meta';
export type KeyCombo = {
  key: string;
  modifiers: ModifierKey[];
};

export interface ShortcutAction {
  id: string;
  name: string;
  description: string;
  category: ShortcutCategory;
  defaultCombo: KeyCombo;
  currentCombo: KeyCombo;
  action: () => void | Promise<void>;
  enabled: boolean;
  context?: string; // Optional context restriction (e.g., 'piano-roll', 'sequencer')
}

export type ShortcutCategory =
  | 'global'
  | 'transport'
  | 'editing'
  | 'navigation'
  | 'windows'
  | 'file'
  | 'view'
  | 'tools';

export interface KeyboardState {
  shortcuts: Record<string, ShortcutAction>;
  activeContext: string | null;
  isRecording: boolean;
  recordingTarget: string | null;
  conflicts: ShortcutConflict[];
  modifiersPressed: Set<ModifierKey>;
}

export interface ShortcutConflict {
  combo: string;
  shortcutIds: string[];
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

export function comboToString(combo: KeyCombo): string {
  const parts: string[] = [];
  if (combo.modifiers.includes('meta')) parts.push('⌘');
  if (combo.modifiers.includes('ctrl')) parts.push('Ctrl');
  if (combo.modifiers.includes('alt')) parts.push('Alt');
  if (combo.modifiers.includes('shift')) parts.push('Shift');
  parts.push(combo.key.toUpperCase());
  return parts.join('+');
}

export function stringToCombo(str: string): KeyCombo {
  const parts = str.split('+').map(p => p.trim().toLowerCase());
  const modifiers: ModifierKey[] = [];
  let key = '';

  for (const part of parts) {
    if (part === '⌘' || part === 'cmd' || part === 'meta') modifiers.push('meta');
    else if (part === 'ctrl' || part === 'control') modifiers.push('ctrl');
    else if (part === 'alt' || part === 'option') modifiers.push('alt');
    else if (part === 'shift') modifiers.push('shift');
    else key = part;
  }

  return { key, modifiers };
}

export function matchesCombo(event: KeyboardEvent, combo: KeyCombo): boolean {
  const key = event.key.toLowerCase();
  const eventModifiers = new Set<ModifierKey>();

  if (event.metaKey) eventModifiers.add('meta');
  if (event.ctrlKey) eventModifiers.add('ctrl');
  if (event.altKey) eventModifiers.add('alt');
  if (event.shiftKey) eventModifiers.add('shift');

  if (key !== combo.key.toLowerCase()) return false;
  if (eventModifiers.size !== combo.modifiers.length) return false;

  return combo.modifiers.every(mod => eventModifiers.has(mod));
}

// ============================================================================
// DEFAULT SHORTCUTS
// ============================================================================

const DEFAULT_SHORTCUTS: Omit<ShortcutAction, 'action'>[] = [
  // Transport
  { id: 'transport.play', name: 'Play/Pause', description: 'Toggle playback', category: 'transport', defaultCombo: { key: 'space', modifiers: [] }, currentCombo: { key: 'space', modifiers: [] }, enabled: true },
  { id: 'transport.stop', name: 'Stop', description: 'Stop playback and return to start', category: 'transport', defaultCombo: { key: 'space', modifiers: ['shift'] }, currentCombo: { key: 'space', modifiers: ['shift'] }, enabled: true },
  { id: 'transport.record', name: 'Record', description: 'Toggle recording', category: 'transport', defaultCombo: { key: 'r', modifiers: ['meta'] }, currentCombo: { key: 'r', modifiers: ['meta'] }, enabled: true },
  { id: 'transport.rewind', name: 'Rewind', description: 'Go to beginning', category: 'transport', defaultCombo: { key: 'home', modifiers: [] }, currentCombo: { key: 'home', modifiers: [] }, enabled: true },
  { id: 'transport.forward', name: 'Forward', description: 'Go to end', category: 'transport', defaultCombo: { key: 'end', modifiers: [] }, currentCombo: { key: 'end', modifiers: [] }, enabled: true },
  { id: 'transport.loop', name: 'Toggle Loop', description: 'Toggle loop mode', category: 'transport', defaultCombo: { key: 'l', modifiers: ['meta'] }, currentCombo: { key: 'l', modifiers: ['meta'] }, enabled: true },

  // File
  { id: 'file.new', name: 'New Project', description: 'Create new project', category: 'file', defaultCombo: { key: 'n', modifiers: ['meta'] }, currentCombo: { key: 'n', modifiers: ['meta'] }, enabled: true },
  { id: 'file.open', name: 'Open', description: 'Open file/project', category: 'file', defaultCombo: { key: 'o', modifiers: ['meta'] }, currentCombo: { key: 'o', modifiers: ['meta'] }, enabled: true },
  { id: 'file.save', name: 'Save', description: 'Save current project', category: 'file', defaultCombo: { key: 's', modifiers: ['meta'] }, currentCombo: { key: 's', modifiers: ['meta'] }, enabled: true },
  { id: 'file.saveAs', name: 'Save As', description: 'Save project as new file', category: 'file', defaultCombo: { key: 's', modifiers: ['meta', 'shift'] }, currentCombo: { key: 's', modifiers: ['meta', 'shift'] }, enabled: true },
  { id: 'file.export', name: 'Export', description: 'Export MIDI file', category: 'file', defaultCombo: { key: 'e', modifiers: ['meta'] }, currentCombo: { key: 'e', modifiers: ['meta'] }, enabled: true },

  // Editing
  { id: 'edit.undo', name: 'Undo', description: 'Undo last action', category: 'editing', defaultCombo: { key: 'z', modifiers: ['meta'] }, currentCombo: { key: 'z', modifiers: ['meta'] }, enabled: true },
  { id: 'edit.redo', name: 'Redo', description: 'Redo last undone action', category: 'editing', defaultCombo: { key: 'z', modifiers: ['meta', 'shift'] }, currentCombo: { key: 'z', modifiers: ['meta', 'shift'] }, enabled: true },
  { id: 'edit.cut', name: 'Cut', description: 'Cut selection', category: 'editing', defaultCombo: { key: 'x', modifiers: ['meta'] }, currentCombo: { key: 'x', modifiers: ['meta'] }, enabled: true },
  { id: 'edit.copy', name: 'Copy', description: 'Copy selection', category: 'editing', defaultCombo: { key: 'c', modifiers: ['meta'] }, currentCombo: { key: 'c', modifiers: ['meta'] }, enabled: true },
  { id: 'edit.paste', name: 'Paste', description: 'Paste from clipboard', category: 'editing', defaultCombo: { key: 'v', modifiers: ['meta'] }, currentCombo: { key: 'v', modifiers: ['meta'] }, enabled: true },
  { id: 'edit.duplicate', name: 'Duplicate', description: 'Duplicate selection', category: 'editing', defaultCombo: { key: 'd', modifiers: ['meta'] }, currentCombo: { key: 'd', modifiers: ['meta'] }, enabled: true },
  { id: 'edit.delete', name: 'Delete', description: 'Delete selection', category: 'editing', defaultCombo: { key: 'backspace', modifiers: [] }, currentCombo: { key: 'backspace', modifiers: [] }, enabled: true },
  { id: 'edit.selectAll', name: 'Select All', description: 'Select all items', category: 'editing', defaultCombo: { key: 'a', modifiers: ['meta'] }, currentCombo: { key: 'a', modifiers: ['meta'] }, enabled: true },
  { id: 'edit.deselect', name: 'Deselect', description: 'Deselect all', category: 'editing', defaultCombo: { key: 'd', modifiers: ['meta', 'shift'] }, currentCombo: { key: 'd', modifiers: ['meta', 'shift'] }, enabled: true },

  // Navigation
  { id: 'nav.zoomIn', name: 'Zoom In', description: 'Zoom in timeline', category: 'navigation', defaultCombo: { key: '=', modifiers: ['meta'] }, currentCombo: { key: '=', modifiers: ['meta'] }, enabled: true },
  { id: 'nav.zoomOut', name: 'Zoom Out', description: 'Zoom out timeline', category: 'navigation', defaultCombo: { key: '-', modifiers: ['meta'] }, currentCombo: { key: '-', modifiers: ['meta'] }, enabled: true },
  { id: 'nav.zoomFit', name: 'Zoom to Fit', description: 'Fit content in view', category: 'navigation', defaultCombo: { key: '0', modifiers: ['meta'] }, currentCombo: { key: '0', modifiers: ['meta'] }, enabled: true },
  { id: 'nav.scrollLeft', name: 'Scroll Left', description: 'Scroll timeline left', category: 'navigation', defaultCombo: { key: 'arrowleft', modifiers: ['alt'] }, currentCombo: { key: 'arrowleft', modifiers: ['alt'] }, enabled: true },
  { id: 'nav.scrollRight', name: 'Scroll Right', description: 'Scroll timeline right', category: 'navigation', defaultCombo: { key: 'arrowright', modifiers: ['alt'] }, currentCombo: { key: 'arrowright', modifiers: ['alt'] }, enabled: true },

  // Windows (extend existing)
  { id: 'window.database', name: 'Database', description: 'Toggle Database window', category: 'windows', defaultCombo: { key: '1', modifiers: ['meta'] }, currentCombo: { key: '1', modifiers: ['meta'] }, enabled: true },
  { id: 'window.daw', name: 'DAW', description: 'Toggle DAW window', category: 'windows', defaultCombo: { key: '2', modifiers: ['meta'] }, currentCombo: { key: '2', modifiers: ['meta'] }, enabled: true },
  { id: 'window.mixer', name: 'Mixer', description: 'Toggle Mixer window', category: 'windows', defaultCombo: { key: '3', modifiers: ['meta'] }, currentCombo: { key: '3', modifiers: ['meta'] }, enabled: true },
  { id: 'window.pipeline', name: 'Pipeline', description: 'Toggle Pipeline window', category: 'windows', defaultCombo: { key: '4', modifiers: ['meta'] }, currentCombo: { key: '4', modifiers: ['meta'] }, enabled: true },
  { id: 'window.midiMixer', name: 'MIDI Mixer', description: 'Toggle MIDI Mixer', category: 'windows', defaultCombo: { key: '5', modifiers: ['meta'] }, currentCombo: { key: '5', modifiers: ['meta'] }, enabled: true },
  { id: 'window.pianoRoll', name: 'Piano Roll', description: 'Toggle Piano Roll', category: 'windows', defaultCombo: { key: '6', modifiers: ['meta'] }, currentCombo: { key: '6', modifiers: ['meta'] }, enabled: true },
  { id: 'window.minimizeAll', name: 'Minimize All', description: 'Minimize all windows', category: 'windows', defaultCombo: { key: 'm', modifiers: ['meta'] }, currentCombo: { key: 'm', modifiers: ['meta'] }, enabled: true },
  { id: 'window.restoreAll', name: 'Restore All', description: 'Restore all windows', category: 'windows', defaultCombo: { key: 'm', modifiers: ['meta', 'shift'] }, currentCombo: { key: 'm', modifiers: ['meta', 'shift'] }, enabled: true },
  { id: 'window.tile', name: 'Tile Windows', description: 'Tile all visible windows', category: 'windows', defaultCombo: { key: 't', modifiers: ['meta'] }, currentCombo: { key: 't', modifiers: ['meta'] }, enabled: true },
  { id: 'window.commandPalette', name: 'Command Palette', description: 'Open command palette', category: 'windows', defaultCombo: { key: 'p', modifiers: ['meta', 'shift'] }, currentCombo: { key: 'p', modifiers: ['meta', 'shift'] }, enabled: true },

  // View
  { id: 'view.toggleSidebar', name: 'Toggle Sidebar', description: 'Show/hide sidebar', category: 'view', defaultCombo: { key: 'b', modifiers: ['meta'] }, currentCombo: { key: 'b', modifiers: ['meta'] }, enabled: true },
  { id: 'view.toggleInspector', name: 'Toggle Inspector', description: 'Show/hide inspector', category: 'view', defaultCombo: { key: 'i', modifiers: ['meta'] }, currentCombo: { key: 'i', modifiers: ['meta'] }, enabled: true },
  { id: 'view.fullscreen', name: 'Fullscreen', description: 'Toggle fullscreen mode', category: 'view', defaultCombo: { key: 'f', modifiers: ['meta', 'shift'] }, currentCombo: { key: 'f', modifiers: ['meta', 'shift'] }, enabled: true },

  // Tools (Piano Roll context)
  { id: 'tool.select', name: 'Select Tool', description: 'Activate selection tool', category: 'tools', defaultCombo: { key: 'v', modifiers: [] }, currentCombo: { key: 'v', modifiers: [] }, enabled: true, context: 'piano-roll' },
  { id: 'tool.pencil', name: 'Pencil Tool', description: 'Activate pencil/draw tool', category: 'tools', defaultCombo: { key: 'p', modifiers: [] }, currentCombo: { key: 'p', modifiers: [] }, enabled: true, context: 'piano-roll' },
  { id: 'tool.eraser', name: 'Eraser Tool', description: 'Activate eraser tool', category: 'tools', defaultCombo: { key: 'e', modifiers: [] }, currentCombo: { key: 'e', modifiers: [] }, enabled: true, context: 'piano-roll' },
  { id: 'tool.velocity', name: 'Velocity Tool', description: 'Activate velocity editing', category: 'tools', defaultCombo: { key: 'u', modifiers: [] }, currentCombo: { key: 'u', modifiers: [] }, enabled: true, context: 'piano-roll' },
  { id: 'tool.split', name: 'Split Tool', description: 'Activate split tool', category: 'tools', defaultCombo: { key: 'x', modifiers: [] }, currentCombo: { key: 'x', modifiers: [] }, enabled: true, context: 'piano-roll' },
];

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

const initialState: KeyboardState = {
  shortcuts: {},
  activeContext: null,
  isRecording: false,
  recordingTarget: null,
  conflicts: [],
  modifiersPressed: new Set(),
};

function createKeyboardStore() {
  const { subscribe, set, update } = writable<KeyboardState>(initialState);

  return {
    subscribe,

    // Initialize shortcuts with action callbacks
    init(actionMap: Record<string, () => void | Promise<void>>) {
      update(state => {
        const shortcuts: Record<string, ShortcutAction> = {};

        for (const shortcut of DEFAULT_SHORTCUTS) {
          shortcuts[shortcut.id] = {
            ...shortcut,
            action: actionMap[shortcut.id] || (() => console.warn(`No action for ${shortcut.id}`)),
          };
        }

        // Load user customizations from localStorage
        const saved = localStorage.getItem('keyboard-shortcuts');
        if (saved) {
          try {
            const customized = JSON.parse(saved) as Record<string, KeyCombo>;
            for (const [id, combo] of Object.entries(customized)) {
              if (shortcuts[id]) {
                shortcuts[id].currentCombo = combo;
              }
            }
          } catch (e) {
            console.error('Failed to load keyboard shortcuts:', e);
          }
        }

        return { ...state, shortcuts, conflicts: detectConflicts(shortcuts) };
      });
    },

    // Handle keyboard event
    handleKeyDown(event: KeyboardEvent): boolean {
      const state = get({ subscribe });

      // Don't intercept if typing in input
      if (isTypingInInput(event)) return false;

      // Handle recording mode
      if (state.isRecording && state.recordingTarget) {
        event.preventDefault();
        const combo = eventToCombo(event);
        if (combo) {
          this.setShortcut(state.recordingTarget, combo);
          this.stopRecording();
        }
        return true;
      }

      // Find matching shortcut
      for (const shortcut of Object.values(state.shortcuts)) {
        if (!shortcut.enabled) continue;
        if (shortcut.context && shortcut.context !== state.activeContext) continue;
        if (matchesCombo(event, shortcut.currentCombo)) {
          event.preventDefault();
          event.stopPropagation();
          shortcut.action();
          return true;
        }
      }

      return false;
    },

    // Set custom shortcut
    setShortcut(id: string, combo: KeyCombo) {
      update(state => {
        if (!state.shortcuts[id]) return state;

        const shortcuts = {
          ...state.shortcuts,
          [id]: { ...state.shortcuts[id], currentCombo: combo },
        };

        // Save to localStorage
        const customized: Record<string, KeyCombo> = {};
        for (const [key, shortcut] of Object.entries(shortcuts)) {
          if (comboToString(shortcut.currentCombo) !== comboToString(shortcut.defaultCombo)) {
            customized[key] = shortcut.currentCombo;
          }
        }
        localStorage.setItem('keyboard-shortcuts', JSON.stringify(customized));

        return { ...state, shortcuts, conflicts: detectConflicts(shortcuts) };
      });
    },

    // Reset shortcut to default
    resetShortcut(id: string) {
      update(state => {
        if (!state.shortcuts[id]) return state;

        const shortcuts = {
          ...state.shortcuts,
          [id]: { ...state.shortcuts[id], currentCombo: state.shortcuts[id].defaultCombo },
        };

        return { ...state, shortcuts, conflicts: detectConflicts(shortcuts) };
      });
    },

    // Reset all shortcuts
    resetAll() {
      update(state => {
        const shortcuts: Record<string, ShortcutAction> = {};
        for (const [id, shortcut] of Object.entries(state.shortcuts)) {
          shortcuts[id] = { ...shortcut, currentCombo: shortcut.defaultCombo };
        }
        localStorage.removeItem('keyboard-shortcuts');
        return { ...state, shortcuts, conflicts: [] };
      });
    },

    // Context management
    setContext(context: string | null) {
      update(state => ({ ...state, activeContext: context }));
    },

    // Recording mode for capturing new shortcuts
    startRecording(targetId: string) {
      update(state => ({ ...state, isRecording: true, recordingTarget: targetId }));
    },

    stopRecording() {
      update(state => ({ ...state, isRecording: false, recordingTarget: null }));
    },

    // Enable/disable shortcut
    setEnabled(id: string, enabled: boolean) {
      update(state => {
        if (!state.shortcuts[id]) return state;
        return {
          ...state,
          shortcuts: {
            ...state.shortcuts,
            [id]: { ...state.shortcuts[id], enabled },
          },
        };
      });
    },
  };
}

// Helper functions
function isTypingInInput(event: KeyboardEvent): boolean {
  const target = event.target as HTMLElement;
  const tagName = target.tagName.toLowerCase();
  return tagName === 'input' || tagName === 'textarea' || target.isContentEditable;
}

function eventToCombo(event: KeyboardEvent): KeyCombo | null {
  // Don't capture modifier-only keypresses
  if (['control', 'alt', 'shift', 'meta'].includes(event.key.toLowerCase())) {
    return null;
  }

  const modifiers: ModifierKey[] = [];
  if (event.metaKey) modifiers.push('meta');
  if (event.ctrlKey) modifiers.push('ctrl');
  if (event.altKey) modifiers.push('alt');
  if (event.shiftKey) modifiers.push('shift');

  return { key: event.key.toLowerCase(), modifiers };
}

function detectConflicts(shortcuts: Record<string, ShortcutAction>): ShortcutConflict[] {
  const comboMap = new Map<string, string[]>();

  for (const [id, shortcut] of Object.entries(shortcuts)) {
    if (!shortcut.enabled) continue;
    const comboStr = comboToString(shortcut.currentCombo);
    const contextKey = shortcut.context ? `${comboStr}@${shortcut.context}` : comboStr;

    if (!comboMap.has(contextKey)) {
      comboMap.set(contextKey, []);
    }
    comboMap.get(contextKey)!.push(id);
  }

  const conflicts: ShortcutConflict[] = [];
  for (const [combo, ids] of comboMap) {
    if (ids.length > 1) {
      conflicts.push({ combo, shortcutIds: ids });
    }
  }

  return conflicts;
}

export const keyboardStore = createKeyboardStore();

// Derived stores
export const shortcutsByCategory = derived(keyboardStore, ($keyboard) => {
  const byCategory: Record<ShortcutCategory, ShortcutAction[]> = {
    global: [],
    transport: [],
    editing: [],
    navigation: [],
    windows: [],
    file: [],
    view: [],
    tools: [],
  };

  for (const shortcut of Object.values($keyboard.shortcuts)) {
    byCategory[shortcut.category].push(shortcut);
  }

  return byCategory;
});

export const hasConflicts = derived(keyboardStore, ($keyboard) => $keyboard.conflicts.length > 0);
```

### A2. Create Keyboard Shortcuts Component (`app/src/lib/components/KeyboardShortcuts.svelte`)

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { keyboardStore, shortcutsByCategory, hasConflicts, comboToString } from '$lib/stores/keyboardStore';
  import type { ShortcutAction, ShortcutCategory } from '$lib/stores/keyboardStore';

  export let visible = false;

  let searchQuery = '';
  let selectedCategory: ShortcutCategory | 'all' = 'all';
  let editingId: string | null = null;

  $: filteredShortcuts = Object.values($keyboardStore.shortcuts).filter(shortcut => {
    if (selectedCategory !== 'all' && shortcut.category !== selectedCategory) return false;
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return shortcut.name.toLowerCase().includes(query) ||
             shortcut.description.toLowerCase().includes(query) ||
             comboToString(shortcut.currentCombo).toLowerCase().includes(query);
    }
    return true;
  });

  function handleKeyDown(event: KeyboardEvent) {
    // Escape to close modal or cancel editing
    if (event.key === 'Escape') {
      if (editingId) {
        keyboardStore.stopRecording();
        editingId = null;
      } else {
        visible = false;
      }
      return;
    }

    // Let keyboard store handle it when editing
    if (editingId && $keyboardStore.isRecording) {
      keyboardStore.handleKeyDown(event);
      editingId = null;
    }
  }

  function startEditing(id: string) {
    editingId = id;
    keyboardStore.startRecording(id);
  }

  function resetShortcut(id: string) {
    keyboardStore.resetShortcut(id);
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });
</script>

{#if visible}
  <div class="keyboard-shortcuts-overlay" on:click={() => visible = false} role="dialog" aria-modal="true">
    <div class="keyboard-shortcuts-modal" on:click|stopPropagation role="document">
      <header>
        <h2>Keyboard Shortcuts</h2>
        <button class="close-btn" on:click={() => visible = false} aria-label="Close">×</button>
      </header>

      {#if $hasConflicts}
        <div class="conflicts-warning" role="alert">
          <span class="icon">⚠️</span>
          <span>Some shortcuts have conflicts. Review highlighted items.</span>
        </div>
      {/if}

      <div class="search-bar">
        <input
          type="text"
          placeholder="Search shortcuts..."
          bind:value={searchQuery}
          aria-label="Search shortcuts"
        />
        <select bind:value={selectedCategory} aria-label="Filter by category">
          <option value="all">All Categories</option>
          <option value="transport">Transport</option>
          <option value="file">File</option>
          <option value="editing">Editing</option>
          <option value="navigation">Navigation</option>
          <option value="windows">Windows</option>
          <option value="view">View</option>
          <option value="tools">Tools</option>
        </select>
      </div>

      <div class="shortcuts-list" role="list">
        {#each filteredShortcuts as shortcut (shortcut.id)}
          {@const isConflict = $keyboardStore.conflicts.some(c => c.shortcutIds.includes(shortcut.id))}
          {@const isEditing = editingId === shortcut.id}
          <div
            class="shortcut-row"
            class:conflict={isConflict}
            class:editing={isEditing}
            class:disabled={!shortcut.enabled}
            role="listitem"
          >
            <div class="shortcut-info">
              <span class="name">{shortcut.name}</span>
              <span class="description">{shortcut.description}</span>
              {#if shortcut.context}
                <span class="context">{shortcut.context}</span>
              {/if}
            </div>

            <div class="shortcut-combo">
              {#if isEditing}
                <span class="recording">Press keys...</span>
              {:else}
                <kbd class="combo">{comboToString(shortcut.currentCombo)}</kbd>
              {/if}
            </div>

            <div class="shortcut-actions">
              <button
                class="edit-btn"
                on:click={() => startEditing(shortcut.id)}
                disabled={isEditing}
              >
                Edit
              </button>
              {#if comboToString(shortcut.currentCombo) !== comboToString(shortcut.defaultCombo)}
                <button
                  class="reset-btn"
                  on:click={() => resetShortcut(shortcut.id)}
                >
                  Reset
                </button>
              {/if}
              <label class="enable-toggle">
                <input
                  type="checkbox"
                  checked={shortcut.enabled}
                  on:change={(e) => keyboardStore.setEnabled(shortcut.id, e.currentTarget.checked)}
                />
              </label>
            </div>
          </div>
        {/each}
      </div>

      <footer>
        <button class="reset-all-btn" on:click={() => keyboardStore.resetAll()}>
          Reset All to Defaults
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .keyboard-shortcuts-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
  }

  .keyboard-shortcuts-modal {
    background: var(--bg-secondary, #1e1e1e);
    border-radius: 8px;
    width: 800px;
    max-width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color, #333);
  }

  header h2 {
    margin: 0;
    font-size: 18px;
    color: var(--text-primary, #fff);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #888);
    font-size: 24px;
    cursor: pointer;
  }

  .conflicts-warning {
    background: rgba(255, 193, 7, 0.15);
    border: 1px solid #ffc107;
    color: #ffc107;
    padding: 10px 16px;
    margin: 16px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .search-bar {
    display: flex;
    gap: 12px;
    padding: 16px 20px;
  }

  .search-bar input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    background: var(--bg-tertiary, #2a2a2a);
    color: var(--text-primary, #fff);
  }

  .search-bar select {
    padding: 8px 12px;
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    background: var(--bg-tertiary, #2a2a2a);
    color: var(--text-primary, #fff);
  }

  .shortcuts-list {
    flex: 1;
    overflow-y: auto;
    padding: 0 20px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    padding: 12px;
    border-bottom: 1px solid var(--border-color, #333);
    gap: 16px;
  }

  .shortcut-row.conflict {
    background: rgba(220, 53, 69, 0.1);
    border-color: rgba(220, 53, 69, 0.3);
  }

  .shortcut-row.editing {
    background: rgba(0, 123, 255, 0.1);
    border-color: rgba(0, 123, 255, 0.3);
  }

  .shortcut-row.disabled {
    opacity: 0.5;
  }

  .shortcut-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .shortcut-info .name {
    font-weight: 500;
    color: var(--text-primary, #fff);
  }

  .shortcut-info .description {
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .shortcut-info .context {
    font-size: 10px;
    color: var(--accent-color, #007bff);
    background: rgba(0, 123, 255, 0.1);
    padding: 2px 6px;
    border-radius: 3px;
    width: fit-content;
    margin-top: 4px;
  }

  .shortcut-combo {
    min-width: 120px;
    text-align: center;
  }

  .combo {
    background: var(--bg-tertiary, #2a2a2a);
    border: 1px solid var(--border-color, #444);
    border-radius: 4px;
    padding: 4px 8px;
    font-family: monospace;
    font-size: 12px;
    color: var(--text-primary, #fff);
  }

  .recording {
    color: var(--accent-color, #007bff);
    font-style: italic;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .shortcut-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .edit-btn, .reset-btn {
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .edit-btn {
    background: var(--accent-color, #007bff);
    border: none;
    color: white;
  }

  .reset-btn {
    background: none;
    border: 1px solid var(--border-color, #444);
    color: var(--text-secondary, #888);
  }

  footer {
    padding: 16px 20px;
    border-top: 1px solid var(--border-color, #333);
    display: flex;
    justify-content: flex-end;
  }

  .reset-all-btn {
    background: rgba(220, 53, 69, 0.2);
    border: 1px solid #dc3545;
    color: #dc3545;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
  }
</style>
```

### A3. Initialize in App.svelte

Add to `App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { keyboardStore } from '$lib/stores/keyboardStore';
  import { uiActions } from '$lib/stores/uiStore';
  import { playbackStore } from '$lib/stores/playbackStore';
  import KeyboardShortcuts from '$lib/components/KeyboardShortcuts.svelte';

  let showShortcuts = false;

  onMount(() => {
    // Initialize keyboard shortcuts with action mappings
    keyboardStore.init({
      // Transport
      'transport.play': () => playbackStore.togglePlay(),
      'transport.stop': () => playbackStore.stop(),
      'transport.record': () => playbackStore.toggleRecord(),
      'transport.rewind': () => playbackStore.rewind(),
      'transport.loop': () => playbackStore.toggleLoop(),

      // Windows
      'window.database': () => uiActions.toggleWindow('database'),
      'window.daw': () => uiActions.toggleWindow('daw'),
      'window.mixer': () => uiActions.toggleWindow('mixer'),
      'window.pipeline': () => uiActions.toggleWindow('pipeline'),
      'window.midiMixer': () => uiActions.toggleWindow('midi-mixer'),
      'window.minimizeAll': () => uiActions.minimizeAll(),
      'window.restoreAll': () => uiActions.restoreAll(),
      'window.tile': () => uiActions.tileWindows(),
      'window.commandPalette': () => showCommandPalette = true,

      // View
      'view.toggleSidebar': () => uiActions.toggleSidebar(),
      'view.toggleInspector': () => uiActions.toggleInspector(),

      // File
      'file.new': () => createNewProject(),
      'file.open': () => openFile(),
      'file.save': () => saveProject(),
      'file.saveAs': () => saveProjectAs(),
      'file.export': () => exportMidi(),

      // Edit
      'edit.undo': () => undoStore.undo(),
      'edit.redo': () => undoStore.redo(),
    });

    // Global keyboard handler
    const handleKeyDown = (event: KeyboardEvent) => {
      keyboardStore.handleKeyDown(event);
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  });
</script>

<KeyboardShortcuts bind:visible={showShortcuts} />
```

---

## PART B: DRAG & DROP SYSTEM

### B1. Create DnD Store (`app/src/lib/stores/dndStore.ts`)

```typescript
import { writable, derived, get } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

export type DragSource =
  | 'file-browser'
  | 'database-list'
  | 'sequencer-track'
  | 'mixer-channel'
  | 'piano-roll-note'
  | 'loop-browser'
  | 'external';

export type DropTarget =
  | 'sequencer-track'
  | 'mixer-channel'
  | 'piano-roll'
  | 'trash'
  | 'folder'
  | 'window-dock';

export interface DragData {
  type: DragDataType;
  source: DragSource;
  payload: unknown;
  preview?: HTMLElement;
}

export type DragDataType =
  | 'midi-file'
  | 'midi-files'
  | 'audio-file'
  | 'track'
  | 'clip'
  | 'note'
  | 'notes'
  | 'channel'
  | 'window';

export interface MidiFileDragPayload {
  fileId: number;
  filename: string;
  filepath: string;
  bpm?: number;
  key?: string;
}

export interface TrackDragPayload {
  trackId: number;
  channel: number;
  name: string;
}

export interface NoteDragPayload {
  noteIds: number[];
  originalPositions: { pitch: number; tick: number }[];
}

export interface DropZone {
  id: string;
  element: HTMLElement;
  accepts: DragDataType[];
  target: DropTarget;
  onDrop: (data: DragData, position: { x: number; y: number }) => void;
  onDragOver?: (data: DragData) => boolean;
  priority?: number;
}

export interface DnDState {
  isDragging: boolean;
  dragData: DragData | null;
  dropZones: Map<string, DropZone>;
  activeDropZone: string | null;
  cursorPosition: { x: number; y: number };
  dragOffset: { x: number; y: number };
}

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

const initialState: DnDState = {
  isDragging: false,
  dragData: null,
  dropZones: new Map(),
  activeDropZone: null,
  cursorPosition: { x: 0, y: 0 },
  dragOffset: { x: 0, y: 0 },
};

function createDnDStore() {
  const { subscribe, set, update } = writable<DnDState>(initialState);

  let previewElement: HTMLElement | null = null;
  let boundMouseMove: ((e: MouseEvent) => void) | null = null;
  let boundMouseUp: ((e: MouseEvent) => void) | null = null;

  const store = {
    subscribe,

    // Register a drop zone
    registerDropZone(zone: DropZone) {
      update(state => {
        const zones = new Map(state.dropZones);
        zones.set(zone.id, zone);
        return { ...state, dropZones: zones };
      });

      return () => store.unregisterDropZone(zone.id);
    },

    // Unregister a drop zone
    unregisterDropZone(id: string) {
      update(state => {
        const zones = new Map(state.dropZones);
        zones.delete(id);
        return { ...state, dropZones: zones };
      });
    },

    // Start dragging
    startDrag(data: DragData, event: MouseEvent) {
      const offset = { x: 0, y: 0 };

      // Create preview element using safe DOM methods
      if (data.preview) {
        previewElement = data.preview.cloneNode(true) as HTMLElement;
      } else {
        previewElement = createDefaultPreview(data);
      }

      previewElement.classList.add('dnd-preview');
      previewElement.style.position = 'fixed';
      previewElement.style.pointerEvents = 'none';
      previewElement.style.zIndex = '100000';
      document.body.appendChild(previewElement);

      update(state => ({
        ...state,
        isDragging: true,
        dragData: data,
        cursorPosition: { x: event.clientX, y: event.clientY },
        dragOffset: offset,
      }));

      store.updatePreviewPosition(event.clientX, event.clientY);

      // Create bound handlers
      boundMouseMove = (e: MouseEvent) => store.handleMouseMove(e);
      boundMouseUp = (e: MouseEvent) => store.handleMouseUp(e);

      // Add document-level listeners
      document.addEventListener('mousemove', boundMouseMove);
      document.addEventListener('mouseup', boundMouseUp);
      document.body.style.cursor = 'grabbing';
      document.body.classList.add('is-dragging');
    },

    // Update drag position
    handleMouseMove(event: MouseEvent) {
      const state = get({ subscribe });
      if (!state.isDragging) return;

      update(s => ({ ...s, cursorPosition: { x: event.clientX, y: event.clientY } }));

      // Update preview position
      if (previewElement) {
        previewElement.style.left = `${event.clientX + 12}px`;
        previewElement.style.top = `${event.clientY + 12}px`;
      }

      // Find active drop zone
      const activeZone = findActiveDropZone(state.dropZones, state.dragData!, event);
      update(s => ({ ...s, activeDropZone: activeZone?.id || null }));

      // Update drop zone highlighting
      state.dropZones.forEach((zone, id) => {
        if (id === activeZone?.id) {
          zone.element.classList.add('drop-zone-active');
        } else {
          zone.element.classList.remove('drop-zone-active');
        }
      });
    },

    // Handle drop
    handleMouseUp(event: MouseEvent) {
      const state = get({ subscribe });
      if (!state.isDragging || !state.dragData) return;

      // Clean up preview
      if (previewElement) {
        previewElement.remove();
        previewElement = null;
      }

      // Execute drop if over valid zone
      if (state.activeDropZone) {
        const zone = state.dropZones.get(state.activeDropZone);
        if (zone) {
          zone.onDrop(state.dragData, { x: event.clientX, y: event.clientY });
        }
      }

      // Clean up all drop zones
      state.dropZones.forEach(zone => {
        zone.element.classList.remove('drop-zone-active');
      });

      // Remove listeners
      if (boundMouseMove) {
        document.removeEventListener('mousemove', boundMouseMove);
        boundMouseMove = null;
      }
      if (boundMouseUp) {
        document.removeEventListener('mouseup', boundMouseUp);
        boundMouseUp = null;
      }
      document.body.style.cursor = '';
      document.body.classList.remove('is-dragging');

      // Reset state
      set({ ...initialState, dropZones: state.dropZones });
    },

    updatePreviewPosition(x: number, y: number) {
      if (previewElement) {
        previewElement.style.left = `${x + 12}px`;
        previewElement.style.top = `${y + 12}px`;
      }
    },

    // Cancel drag
    cancelDrag() {
      if (previewElement) {
        previewElement.remove();
        previewElement = null;
      }

      const state = get({ subscribe });
      state.dropZones.forEach(zone => {
        zone.element.classList.remove('drop-zone-active');
      });

      if (boundMouseMove) {
        document.removeEventListener('mousemove', boundMouseMove);
        boundMouseMove = null;
      }
      if (boundMouseUp) {
        document.removeEventListener('mouseup', boundMouseUp);
        boundMouseUp = null;
      }

      document.body.style.cursor = '';
      document.body.classList.remove('is-dragging');

      set({ ...initialState, dropZones: state.dropZones });
    },
  };

  return store;
}

// Helper function - creates preview using safe DOM methods (no innerHTML)
function createDefaultPreview(data: DragData): HTMLElement {
  const el = document.createElement('div');
  el.className = 'dnd-default-preview';
  el.style.background = 'var(--bg-secondary, #2a2a2a)';
  el.style.border = '1px solid var(--accent-color, #007bff)';
  el.style.borderRadius = '4px';
  el.style.padding = '8px 12px';
  el.style.fontSize = '12px';
  el.style.color = 'var(--text-primary, #fff)';
  el.style.boxShadow = '0 4px 12px rgba(0,0,0,0.3)';
  el.style.maxWidth = '200px';
  el.style.overflow = 'hidden';
  el.style.textOverflow = 'ellipsis';
  el.style.whiteSpace = 'nowrap';
  el.style.display = 'flex';
  el.style.alignItems = 'center';
  el.style.gap = '6px';

  const icon = document.createElement('span');
  const text = document.createElement('span');

  switch (data.type) {
    case 'midi-file':
      const filePayload = data.payload as MidiFileDragPayload;
      icon.textContent = '🎵';
      text.textContent = filePayload.filename;
      break;
    case 'midi-files':
      const files = data.payload as MidiFileDragPayload[];
      icon.textContent = '🎵';
      text.textContent = `${files.length} files`;
      break;
    case 'track':
      const trackPayload = data.payload as TrackDragPayload;
      icon.textContent = '🎸';
      text.textContent = trackPayload.name;
      break;
    case 'clip':
      icon.textContent = '🎬';
      text.textContent = 'Clip';
      break;
    case 'note':
    case 'notes':
      icon.textContent = '🎹';
      text.textContent = data.type === 'note' ? 'Note' : 'Notes';
      break;
    default:
      icon.textContent = '📦';
      text.textContent = `Dragging ${data.type}`;
  }

  el.appendChild(icon);
  el.appendChild(text);

  return el;
}

function findActiveDropZone(
  zones: Map<string, DropZone>,
  dragData: DragData,
  event: MouseEvent
): DropZone | null {
  const candidates: DropZone[] = [];

  zones.forEach(zone => {
    // Check if zone accepts this drag type
    if (!zone.accepts.includes(dragData.type)) return;

    // Check if cursor is over element
    const rect = zone.element.getBoundingClientRect();
    if (
      event.clientX >= rect.left &&
      event.clientX <= rect.right &&
      event.clientY >= rect.top &&
      event.clientY <= rect.bottom
    ) {
      // Check custom onDragOver validation
      if (zone.onDragOver && !zone.onDragOver(dragData)) return;

      candidates.push(zone);
    }
  });

  // Sort by priority (higher priority wins) or z-index
  candidates.sort((a, b) => (b.priority || 0) - (a.priority || 0));

  return candidates[0] || null;
}

export const dndStore = createDnDStore();

// Derived stores
export const isDragging = derived(dndStore, $dnd => $dnd.isDragging);
export const dragType = derived(dndStore, $dnd => $dnd.dragData?.type || null);
```

### B2. Create Draggable Action (`app/src/lib/utils/draggable.ts`)

```typescript
import { dndStore } from '$lib/stores/dndStore';
import type { DragData } from '$lib/stores/dndStore';

export interface DraggableOptions {
  data: () => DragData;
  disabled?: boolean;
  handle?: string; // CSS selector for drag handle
  onDragStart?: () => void;
  onDragEnd?: () => void;
}

export function draggable(node: HTMLElement, options: DraggableOptions) {
  function getHandle(): HTMLElement {
    if (options.handle) {
      return node.querySelector(options.handle) || node;
    }
    return node;
  }

  function handleMouseDown(event: MouseEvent) {
    if (options.disabled) return;
    if (event.button !== 0) return; // Only left click

    // Check if clicked on handle
    const handle = getHandle();
    if (handle !== node && !handle.contains(event.target as Node)) return;

    event.preventDefault();

    const data = options.data();
    options.onDragStart?.();
    dndStore.startDrag(data, event);

    // One-time cleanup handler
    const cleanup = () => {
      options.onDragEnd?.();
      document.removeEventListener('mouseup', cleanup);
    };
    document.addEventListener('mouseup', cleanup);
  }

  const handle = getHandle();
  handle.style.cursor = options.disabled ? 'default' : 'grab';
  handle.addEventListener('mousedown', handleMouseDown);

  return {
    update(newOptions: DraggableOptions) {
      options = newOptions;
      const h = getHandle();
      h.style.cursor = options.disabled ? 'default' : 'grab';
    },
    destroy() {
      const h = getHandle();
      h.removeEventListener('mousedown', handleMouseDown);
    },
  };
}
```

### B3. Create Droppable Action (`app/src/lib/utils/droppable.ts`)

```typescript
import { dndStore } from '$lib/stores/dndStore';
import type { DragData, DragDataType, DropTarget } from '$lib/stores/dndStore';

export interface DroppableOptions {
  accepts: DragDataType[];
  target: DropTarget;
  onDrop: (data: DragData, position: { x: number; y: number }) => void;
  onDragOver?: (data: DragData) => boolean;
  onDragEnter?: (data: DragData) => void;
  onDragLeave?: () => void;
  disabled?: boolean;
  priority?: number;
}

export function droppable(node: HTMLElement, options: DroppableOptions) {
  const zoneId = `drop-zone-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

  let unregister: (() => void) | null = null;

  function register() {
    if (options.disabled) return;

    unregister = dndStore.registerDropZone({
      id: zoneId,
      element: node,
      accepts: options.accepts,
      target: options.target,
      onDrop: options.onDrop,
      onDragOver: options.onDragOver,
      priority: options.priority,
    });
  }

  register();

  return {
    update(newOptions: DroppableOptions) {
      unregister?.();
      options = newOptions;
      register();
    },
    destroy() {
      unregister?.();
    },
  };
}
```

### B4. Usage Examples

**In DatabaseWindow.svelte (drag source):**

```svelte
<script lang="ts">
  import { draggable } from '$lib/utils/draggable';
  import type { FileDetails } from '$lib/types';

  export let files: FileDetails[] = [];
  let selectedFiles: Set<number> = new Set();
</script>

{#each files as file (file.id)}
  <div
    class="file-row"
    use:draggable={{
      data: () => ({
        type: selectedFiles.size > 1 ? 'midi-files' : 'midi-file',
        source: 'database-list',
        payload: selectedFiles.size > 1
          ? files.filter(f => selectedFiles.has(f.id)).map(f => ({
              fileId: f.id,
              filename: f.filename,
              filepath: f.filepath,
              bpm: f.bpm,
              key: f.key_signature,
            }))
          : {
              fileId: file.id,
              filename: file.filename,
              filepath: file.filepath,
              bpm: file.bpm,
              key: file.key_signature,
            },
      }),
      disabled: false,
    }}
  >
    {file.filename}
  </div>
{/each}
```

**In SequencerTrack.svelte (drop target):**

```svelte
<script lang="ts">
  import { droppable } from '$lib/utils/droppable';
  import { invoke } from '@tauri-apps/api/core';
  import type { DragData, MidiFileDragPayload } from '$lib/stores/dndStore';

  export let trackId: number;
  export let channel: number;

  async function handleDrop(data: DragData, position: { x: number; y: number }) {
    if (data.type === 'midi-file') {
      const payload = data.payload as MidiFileDragPayload;
      await invoke('add_file_to_track', {
        trackId,
        fileId: payload.fileId,
        position: calculateTickPosition(position.x),
      });
    } else if (data.type === 'midi-files') {
      const files = data.payload as MidiFileDragPayload[];
      for (const file of files) {
        await invoke('add_file_to_track', {
          trackId,
          fileId: file.fileId,
          position: calculateTickPosition(position.x),
        });
      }
    }
  }

  function calculateTickPosition(x: number): number {
    // Implementation depends on timeline scale/offset
    return 0;
  }
</script>

<div
  class="sequencer-track"
  use:droppable={{
    accepts: ['midi-file', 'midi-files', 'clip'],
    target: 'sequencer-track',
    onDrop: handleDrop,
    priority: 10,
  }}
>
  <slot />
</div>

<style>
  .sequencer-track {
    min-height: 60px;
    transition: background-color 0.15s ease;
  }

  :global(.sequencer-track.drop-zone-active) {
    background-color: rgba(0, 123, 255, 0.15);
    outline: 2px dashed var(--accent-color, #007bff);
    outline-offset: -2px;
  }
</style>
```

### B5. Global DnD Styles (`app/src/app.css`)

Add these styles:

```css
/* Drag and Drop Styles */
body.is-dragging {
  user-select: none;
}

body.is-dragging * {
  cursor: grabbing !important;
}

.dnd-preview {
  pointer-events: none;
  z-index: 100000;
  opacity: 0.95;
  transform: scale(1.02);
  animation: dnd-preview-appear 0.15s ease-out;
}

@keyframes dnd-preview-appear {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 0.95;
    transform: scale(1.02);
  }
}

.drop-zone-active {
  background-color: rgba(0, 123, 255, 0.1) !important;
  outline: 2px dashed var(--accent-color, #007bff) !important;
  outline-offset: -2px;
}

/* Drag handle cursor */
[data-drag-handle] {
  cursor: grab;
}

[data-drag-handle]:active {
  cursor: grabbing;
}
```

---

## TESTING CHECKLIST

### Keyboard Shortcuts
- [ ] All default shortcuts work correctly
- [ ] Custom shortcuts persist across sessions
- [ ] Conflict detection warns users
- [ ] Recording mode captures new shortcuts
- [ ] Context-specific shortcuts only fire in correct context
- [ ] Escape cancels recording mode
- [ ] Reset to defaults works
- [ ] Shortcuts don't fire when typing in inputs

### Drag & Drop
- [ ] Files drag from database to sequencer
- [ ] Multi-file selection drag works
- [ ] Visual preview follows cursor
- [ ] Drop zones highlight on hover
- [ ] Invalid drop zones don't highlight
- [ ] Escape cancels drag operation
- [ ] Drop position is correctly calculated
- [ ] Cleanup happens on mouseup outside valid zones

---

## FILES TO CREATE/MODIFY

| File | Action |
|------|--------|
| `app/src/lib/stores/keyboardStore.ts` | CREATE |
| `app/src/lib/components/KeyboardShortcuts.svelte` | CREATE |
| `app/src/lib/utils/draggable.ts` | CREATE |
| `app/src/lib/utils/droppable.ts` | CREATE |
| `app/src/lib/stores/dndStore.ts` | CREATE |
| `app/src/App.svelte` | MODIFY - Add keyboard init |
| `app/src/app.css` | MODIFY - Add DnD styles |
| `app/src/lib/stores/index.ts` | MODIFY - Export new stores |
