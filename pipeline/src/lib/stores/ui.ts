/**
 * UI state management
 */

import { writable } from 'svelte/store';

interface UIState {
  sidebarOpen: boolean;
  detailsPanelOpen: boolean;
  theme: 'light' | 'dark';
  notifications: Notification[];
}

interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
  duration?: number;
}

function createUIStore() {
  const { subscribe, update } = writable<UIState>({
    sidebarOpen: true,
    detailsPanelOpen: false,
    theme: 'light',
    notifications: [],
  });

  return {
    subscribe,
    toggleSidebar: () => update(state => ({
      ...state,
      sidebarOpen: !state.sidebarOpen,
    })),
    toggleDetailsPanel: () => update(state => ({
      ...state,
      detailsPanelOpen: !state.detailsPanelOpen,
    })),
    setTheme: (theme: 'light' | 'dark') => update(state => ({
      ...state,
      theme,
    })),
    addNotification: (notification: Omit<Notification, 'id'>) => {
      const id = Math.random().toString(36).substring(7);
      update(state => ({
        ...state,
        notifications: [...state.notifications, { ...notification, id }],
      }));

      // Auto-remove after duration
      if (notification.duration) {
        setTimeout(() => {
          update(state => ({
            ...state,
            notifications: state.notifications.filter(n => n.id !== id),
          }));
        }, notification.duration);
      }
    },
    removeNotification: (id: string) => update(state => ({
      ...state,
      notifications: state.notifications.filter(n => n.id !== id),
    })),
  };
}

export const uiStore = createUIStore();
