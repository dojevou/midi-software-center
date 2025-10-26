// Keyboard Shortcuts Manager
// Provides centralized keyboard shortcut handling for the entire application

export type KeyboardShortcut = {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean; // Command key on Mac
  action: () => void | Promise<void>;
  description: string;
  category?: string; // Group shortcuts by category for help display
  context?: string; // Optional context where shortcut applies
};

export class KeyboardShortcutManager {
  private shortcuts: KeyboardShortcut[] = [];
  private isListening = false;
  private handleKeyDown: (event: KeyboardEvent) => void;

  constructor() {
    this.handleKeyDown = this.createKeyDownHandler();
  }

  register(shortcut: KeyboardShortcut) {
    // Check for duplicate shortcuts
    const existing = this.shortcuts.find(s =>
      s.key.toLowerCase() === shortcut.key.toLowerCase() &&
      !!s.ctrl === !!shortcut.ctrl &&
      !!s.shift === !!shortcut.shift &&
      !!s.alt === !!shortcut.alt &&
      !!s.meta === !!shortcut.meta
    );

    if (existing) {
      console.warn(`Duplicate shortcut registered: ${this.formatShortcut(shortcut)}`);
      // Replace the existing shortcut
      this.unregister(shortcut.key, shortcut.ctrl, shortcut.shift, shortcut.alt, shortcut.meta);
    }

    this.shortcuts.push(shortcut);
  }

  unregister(key: string, ctrl?: boolean, shift?: boolean, alt?: boolean, meta?: boolean) {
    this.shortcuts = this.shortcuts.filter(s =>
      !(s.key.toLowerCase() === key.toLowerCase() &&
        !!s.ctrl === !!ctrl &&
        !!s.shift === !!shift &&
        !!s.alt === !!alt &&
        !!s.meta === !!meta)
    );
  }

  start() {
    if (this.isListening) return;
    this.isListening = true;
    document.addEventListener('keydown', this.handleKeyDown);
  }

  stop() {
    this.isListening = false;
    document.removeEventListener('keydown', this.handleKeyDown);
  }

  clear() {
    this.shortcuts = [];
  }

  private createKeyDownHandler() {
    return (event: KeyboardEvent) => {
      // Don't trigger shortcuts when typing in input fields
      const target = event.target as HTMLElement;
      const isInputField = target.tagName === 'INPUT' ||
                          target.tagName === 'TEXTAREA' ||
                          target.isContentEditable;

      // Exception: Escape key should always work
      if (isInputField && event.key !== 'Escape') {
        return;
      }

      // Normalize key for consistency
      let key = event.key;

      // Handle special keys
      if (key === ' ') key = 'Space';

      // Find matching shortcut
      const shortcut = this.shortcuts.find(s => {
        const keyMatch = s.key.toLowerCase() === key.toLowerCase();
        const ctrlMatch = !!s.ctrl === (event.ctrlKey || event.metaKey); // Support Cmd on Mac
        const shiftMatch = !!s.shift === event.shiftKey;
        const altMatch = !!s.alt === event.altKey;

        return keyMatch && ctrlMatch && shiftMatch && altMatch;
      });

      if (shortcut) {
        event.preventDefault();
        event.stopPropagation();

        try {
          const result = shortcut.action();
          if (result instanceof Promise) {
            result.catch(error => {
              console.error(`Error executing shortcut ${this.formatShortcut(shortcut)}:`, error);
            });
          }
        } catch (error) {
          console.error(`Error executing shortcut ${this.formatShortcut(shortcut)}:`, error);
        }
      }
    };
  }

  getShortcuts(): KeyboardShortcut[] {
    return [...this.shortcuts];
  }

  getShortcutsByCategory(): Map<string, KeyboardShortcut[]> {
    const categorized = new Map<string, KeyboardShortcut[]>();

    for (const shortcut of this.shortcuts) {
      const category = shortcut.category || 'Other';
      if (!categorized.has(category)) {
        categorized.set(category, []);
      }
      categorized.get(category)!.push(shortcut);
    }

    return categorized;
  }

  formatShortcut(shortcut: KeyboardShortcut): string {
    const parts: string[] = [];

    if (shortcut.ctrl || shortcut.meta) parts.push('Ctrl');
    if (shortcut.shift) parts.push('Shift');
    if (shortcut.alt) parts.push('Alt');

    const keyDisplay = shortcut.key === ' ' || shortcut.key === 'Space'
      ? 'Space'
      : shortcut.key.toUpperCase();
    parts.push(keyDisplay);

    return parts.join('+');
  }
}

// Global singleton instance
export const keyboardManager = new KeyboardShortcutManager();
