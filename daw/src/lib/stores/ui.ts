import { writable, type Writable } from 'svelte/store';

/**
 * Filter sidebar visible
 */
export const showFilters: Writable<boolean> = writable(true);

/**
 * Sequencer panel visible
 */
export const showSequencer: Writable<boolean> = writable(false);

/**
 * File details panel visible
 */
export const showDetails: Writable<boolean> = writable(false);

/**
 * Currently selected file for details
 */
export const selectedFileId: Writable<number | null> = writable(null);

/**
 * Export dialog open
 */
export const exportDialogOpen: Writable<boolean> = writable(false);

/**
 * Toast notifications
 */
export interface Toast {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  message: string;
  duration?: number;
}

export const toasts: Writable<Toast[]> = writable([]);

/**
 * Add a toast notification
 */
export function addToast(
  type: Toast['type'],
  message: string,
  duration: number = 3000
) {
  const id = `toast-${Date.now()}-${Math.random()}`;
  const toast: Toast = { id, type, message, duration };

  toasts.update(t => [...t, toast]);

  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }
}

/**
 * Remove a toast notification
 */
export function removeToast(id: string) {
  toasts.update(t => t.filter(toast => toast.id !== id));
}

/**
 * Convenience functions
 */
export const toast = {
  success: (msg: string, duration?: number) => addToast('success', msg, duration),
  error: (msg: string, duration?: number) => addToast('error', msg, duration),
  info: (msg: string, duration?: number) => addToast('info', msg, duration),
  warning: (msg: string, duration?: number) => addToast('warning', msg, duration),
};
