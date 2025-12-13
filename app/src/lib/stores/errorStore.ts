import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// TYPES
// ============================================================================

export type ErrorSeverity = 'info' | 'warning' | 'error' | 'critical';
export type ErrorCategory = 'network' | 'database' | 'file' | 'midi' | 'ui' | 'unknown';

export interface AppError {
  id: string;
  message: string;
  details?: string;
  severity: ErrorSeverity;
  category: ErrorCategory;
  timestamp: number;
  stack?: string;
  component?: string;
  action?: ErrorAction;
  dismissed: boolean;
  retryable: boolean;
}

export interface ErrorAction {
  label: string;
  handler: () => void | Promise<void>;
}

export interface ErrorState {
  errors: AppError[];
  globalError: AppError | null;
  showErrorDialog: boolean;
  errorCount: number;
}

// ============================================================================
// ERROR UTILITIES
// ============================================================================

export function createErrorId(): string {
  return `err-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

export function categorizeError(error: unknown): ErrorCategory {
  if (error instanceof Error) {
    const msg = error.message.toLowerCase();
    if (msg.includes('network') || msg.includes('fetch') || msg.includes('timeout')) {
      return 'network';
    }
    if (msg.includes('database') || msg.includes('sql') || msg.includes('query')) {
      return 'database';
    }
    if (msg.includes('file') || msg.includes('path') || msg.includes('permission')) {
      return 'file';
    }
    if (msg.includes('midi') || msg.includes('device') || msg.includes('port')) {
      return 'midi';
    }
  }
  return 'unknown';
}

export function formatErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === 'string') {
    return error;
  }
  if (error && typeof error === 'object' && 'message' in error) {
    return String((error as { message: unknown }).message);
  }
  return 'An unexpected error occurred';
}

// ============================================================================
// STORE IMPLEMENTATION
// ============================================================================

const MAX_ERRORS = 100;

const initialState: ErrorState = {
  errors: [],
  globalError: null,
  showErrorDialog: false,
  errorCount: 0,
};

function createErrorStore() {
  const { subscribe, set, update } = writable<ErrorState>(initialState);

  function getState(): ErrorState {
    let state: ErrorState = initialState;
    subscribe(s => { state = s; })();
    return state;
  }

  const store = {
    subscribe,

    // Add a new error
    addError(
      error: unknown,
      options: {
        severity?: ErrorSeverity;
        category?: ErrorCategory;
        component?: string;
        action?: ErrorAction;
        retryable?: boolean;
      } = {}
    ): string {
      const id = createErrorId();
      const appError: AppError = {
        id,
        message: formatErrorMessage(error),
        details: error instanceof Error ? error.stack : undefined,
        severity: options.severity || 'error',
        category: options.category || categorizeError(error),
        timestamp: Date.now(),
        stack: error instanceof Error ? error.stack : undefined,
        component: options.component,
        action: options.action,
        dismissed: false,
        retryable: options.retryable ?? false,
      };

      update(state => {
        const errors = [appError, ...state.errors].slice(0, MAX_ERRORS);
        const showDialog = appError.severity === 'critical' || appError.severity === 'error';
        return {
          ...state,
          errors,
          globalError: showDialog ? appError : state.globalError,
          showErrorDialog: showDialog || state.showErrorDialog,
          errorCount: state.errorCount + 1,
        };
      });

      // Log to backend
      store.logToBackend(appError);

      return id;
    },

    // Dismiss an error
    dismiss(errorId: string) {
      update(state => ({
        ...state,
        errors: state.errors.map(e =>
          e.id === errorId ? { ...e, dismissed: true } : e
        ),
        globalError: state.globalError?.id === errorId ? null : state.globalError,
        showErrorDialog: state.globalError?.id === errorId ? false : state.showErrorDialog,
      }));
    },

    // Dismiss all errors
    dismissAll() {
      update(state => ({
        ...state,
        errors: state.errors.map(e => ({ ...e, dismissed: true })),
        globalError: null,
        showErrorDialog: false,
      }));
    },

    // Clear all errors
    clearAll() {
      set(initialState);
    },

    // Close error dialog
    closeDialog() {
      update(state => ({
        ...state,
        showErrorDialog: false,
        globalError: null,
      }));
    },

    // Retry action for an error
    async retry(errorId: string) {
      const state = getState();
      const error = state.errors.find(e => e.id === errorId);
      if (error?.action) {
        try {
          await error.action.handler();
          store.dismiss(errorId);
        } catch (e) {
          // Error persists, update message if different
          const newMessage = formatErrorMessage(e);
          if (newMessage !== error.message) {
            update(s => ({
              ...s,
              errors: s.errors.map(err =>
                err.id === errorId ? { ...err, message: newMessage } : err
              ),
            }));
          }
        }
      }
    },

    // Log error to Rust backend
    async logToBackend(error: AppError) {
      try {
        await invoke('log_frontend_error', {
          error: {
            message: error.message,
            severity: error.severity,
            category: error.category,
            component: error.component,
            stack: error.stack,
            timestamp: error.timestamp,
          },
        });
      } catch (e) {
        console.error('Failed to log error to backend:', e);
      }
    },
  };

  return store;
}

export const errorStore = createErrorStore();

// Derived stores
export const activeErrors = derived(errorStore, $errors =>
  $errors.errors.filter(e => !e.dismissed)
);

export const criticalErrors = derived(errorStore, $errors =>
  $errors.errors.filter(e => e.severity === 'critical' && !e.dismissed)
);

export const hasErrors = derived(errorStore, $errors =>
  $errors.errors.some(e => !e.dismissed)
);

// Global error handler
if (typeof window !== 'undefined') {
  window.addEventListener('error', (event) => {
    errorStore.addError(event.error || event.message, {
      severity: 'error',
      component: 'window',
    });
  });

  window.addEventListener('unhandledrejection', (event) => {
    errorStore.addError(event.reason, {
      severity: 'error',
      component: 'promise',
    });
  });
}
