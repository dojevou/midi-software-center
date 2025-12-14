import { writable, derived } from 'svelte/store';

// ============================================================================
// TYPES
// ============================================================================

export interface A11yState {
  announcements: string[];
  focusTrapActive: boolean;
  focusTrapElement: HTMLElement | null;
  screenReaderMode: boolean;
  keyboardNavigationActive: boolean;
  lastFocusedElement: HTMLElement | null;
}

export interface A11yAnnouncement {
  message: string;
  priority: 'polite' | 'assertive';
  timeout?: number;
}

// ============================================================================
// STORE
// ============================================================================

const initialState: A11yState = {
  announcements: [],
  focusTrapActive: false,
  focusTrapElement: null,
  screenReaderMode: false,
  keyboardNavigationActive: false,
  lastFocusedElement: null,
};

// Focus trap handler
function handleFocusTrap(event: KeyboardEvent) {
  if (event.key !== 'Tab') {return;}

  const element = event.currentTarget as HTMLElement;
  const focusable = getFocusableElements(element);
  if (focusable.length === 0) {return;}

  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}

// Get focusable elements within container
function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selector = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(', ');

  return Array.from(container.querySelectorAll(selector));
}

function createA11yStore() {
  const { subscribe, set, update } = writable<A11yState>(initialState);

  let liveRegion: HTMLElement | null = null;

  // Create live region on init
  if (typeof document !== 'undefined') {
    liveRegion = document.createElement('div');
    liveRegion.setAttribute('role', 'status');
    liveRegion.setAttribute('aria-live', 'polite');
    liveRegion.setAttribute('aria-atomic', 'true');
    liveRegion.className = 'sr-only';
    liveRegion.id = 'a11y-announcer';
    document.body.appendChild(liveRegion);
  }

  return {
    subscribe,

    // Screen reader announcements
    announce(message: string, priority: 'polite' | 'assertive' = 'polite') {
      if (!liveRegion) {return;}

      liveRegion.setAttribute('aria-live', priority);
      // Clear then set to trigger announcement
      liveRegion.textContent = '';
      requestAnimationFrame(() => {
        if (liveRegion) {
          liveRegion.textContent = message;
        }
      });

      update(state => ({
        ...state,
        announcements: [...state.announcements.slice(-9), message],
      }));
    },

    // Focus management
    trapFocus(element: HTMLElement) {
      update(state => ({
        ...state,
        focusTrapActive: true,
        focusTrapElement: element,
        lastFocusedElement: document.activeElement as HTMLElement,
      }));

      // Focus first focusable element
      const focusable = getFocusableElements(element);
      if (focusable.length > 0) {
        focusable[0].focus();
      }

      // Add trap handler
      element.addEventListener('keydown', handleFocusTrap);
    },

    releaseFocus() {
      update(state => {
        if (state.focusTrapElement) {
          state.focusTrapElement.removeEventListener('keydown', handleFocusTrap);
        }
        if (state.lastFocusedElement) {
          state.lastFocusedElement.focus();
        }
        return {
          ...state,
          focusTrapActive: false,
          focusTrapElement: null,
        };
      });
    },

    // Skip link support
    skipToContent(targetId: string) {
      const target = document.getElementById(targetId);
      if (target) {
        target.setAttribute('tabindex', '-1');
        target.focus();
        target.scrollIntoView({ behavior: 'smooth' });
      }
    },

    // Keyboard navigation detection
    enableKeyboardMode() {
      update(state => ({ ...state, keyboardNavigationActive: true }));
      document.body.classList.add('keyboard-navigation');
    },

    disableKeyboardMode() {
      update(state => ({ ...state, keyboardNavigationActive: false }));
      document.body.classList.remove('keyboard-navigation');
    },

    // Screen reader mode toggle
    setScreenReaderMode(enabled: boolean) {
      update(state => ({ ...state, screenReaderMode: enabled }));
      if (enabled) {
        document.body.classList.add('screen-reader-mode');
      } else {
        document.body.classList.remove('screen-reader-mode');
      }
    },
  };
}

export const a11yStore = createA11yStore();

// Derived stores
export const isKeyboardNav = derived(a11yStore, $a11y => $a11y.keyboardNavigationActive);
export const isFocusTrapped = derived(a11yStore, $a11y => $a11y.focusTrapActive);
