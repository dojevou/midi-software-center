import { writable, derived, get } from 'svelte/store';

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
  if (combo.modifiers.includes('meta')) {parts.push('\u2318');}
  if (combo.modifiers.includes('ctrl')) {parts.push('Ctrl');}
  if (combo.modifiers.includes('alt')) {parts.push('Alt');}
  if (combo.modifiers.includes('shift')) {parts.push('Shift');}
  parts.push(combo.key.toUpperCase());
  return parts.join('+');
}

export function stringToCombo(str: string): KeyCombo {
  const parts = str.split('+').map(p => p.trim().toLowerCase());
  const modifiers: ModifierKey[] = [];
  let key = '';

  for (const part of parts) {
    if (part === '\u2318' || part === 'cmd' || part === 'meta') {modifiers.push('meta');}
    else if (part === 'ctrl' || part === 'control') {modifiers.push('ctrl');}
    else if (part === 'alt' || part === 'option') {modifiers.push('alt');}
    else if (part === 'shift') {modifiers.push('shift');}
    else {key = part;}
  }

  return { key, modifiers };
}

export function matchesCombo(event: KeyboardEvent, combo: KeyCombo): boolean {
  const key = event.key.toLowerCase();
  const eventModifiers = new Set<ModifierKey>();

  if (event.metaKey) {eventModifiers.add('meta');}
  if (event.ctrlKey) {eventModifiers.add('ctrl');}
  if (event.altKey) {eventModifiers.add('alt');}
  if (event.shiftKey) {eventModifiers.add('shift');}

  if (key !== combo.key.toLowerCase()) {return false;}
  if (eventModifiers.size !== combo.modifiers.length) {return false;}

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
  { id: 'window.score', name: 'Score View', description: 'Open Score View window', category: 'windows', defaultCombo: { key: 'n', modifiers: ['ctrl', 'shift'] }, currentCombo: { key: 'n', modifiers: ['ctrl', 'shift'] }, enabled: true },
  { id: 'window.scriptEditor', name: 'Script Editor', description: 'Open Script Editor window', category: 'windows', defaultCombo: { key: 's', modifiers: ['ctrl', 'shift'] }, currentCombo: { key: 's', modifiers: ['ctrl', 'shift'] }, enabled: true },
  { id: 'window.midiLearn', name: 'MIDI Learn', description: 'Open MIDI Learn window', category: 'windows', defaultCombo: { key: 'm', modifiers: ['ctrl'] }, currentCombo: { key: 'm', modifiers: ['ctrl'] }, enabled: true },
  { id: 'window.linkSync', name: 'Ableton Link', description: 'Open Ableton Link window', category: 'windows', defaultCombo: { key: 'l', modifiers: ['ctrl', 'shift'] }, currentCombo: { key: 'l', modifiers: ['ctrl', 'shift'] }, enabled: true },

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
      if (isTypingInInput(event)) {return false;}

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
        if (!shortcut.enabled) {continue;}
        if (shortcut.context && shortcut.context !== state.activeContext) {continue;}
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
        if (!state.shortcuts[id]) {return state;}

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
        if (!state.shortcuts[id]) {return state;}

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
        if (!state.shortcuts[id]) {return state;}
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
  if (event.metaKey) {modifiers.push('meta');}
  if (event.ctrlKey) {modifiers.push('ctrl');}
  if (event.altKey) {modifiers.push('alt');}
  if (event.shiftKey) {modifiers.push('shift');}

  return { key: event.key.toLowerCase(), modifiers };
}

function detectConflicts(shortcuts: Record<string, ShortcutAction>): ShortcutConflict[] {
  const comboMap = new Map<string, string[]>();

  for (const [id, shortcut] of Object.entries(shortcuts)) {
    if (!shortcut.enabled) {continue;}
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
