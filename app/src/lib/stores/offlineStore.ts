import { writable, derived, get } from 'svelte/store';
import { healthStore } from './healthStore';

// Types
export interface CachedItem<T> {
  data: T;
  cachedAt: number;
  expiresAt: number;
  key: string;
}

export interface OfflineState {
  isOffline: boolean;
  lastOnlineAt: number | null;
  pendingOperations: PendingOperation[];
  cacheStats: CacheStats;
}

export interface PendingOperation {
  id: string;
  type: 'create' | 'update' | 'delete';
  entity: string;
  data: unknown;
  createdAt: number;
  retryCount: number;
}

export interface CacheStats {
  itemCount: number;
  totalSize: number;
  oldestItem: number | null;
  newestItem: number | null;
}

// Constants
const CACHE_PREFIX = 'midi_cache_';
const CACHE_INDEX_KEY = 'midi_cache_index';
const DEFAULT_TTL = 1000 * 60 * 60; // 1 hour
const MAX_CACHE_SIZE = 50 * 1024 * 1024; // 50MB
const MAX_PENDING_OPERATIONS = 100;

// Cache Index for tracking all cached items
interface CacheIndex {
  keys: string[];
  lastUpdated: number;
}

function getCacheIndex(): CacheIndex {
  try {
    const stored = localStorage.getItem(CACHE_INDEX_KEY);
    return stored ? JSON.parse(stored) : { keys: [], lastUpdated: Date.now() };
  } catch {
    return { keys: [], lastUpdated: Date.now() };
  }
}

function saveCacheIndex(index: CacheIndex): void {
  try {
    localStorage.setItem(CACHE_INDEX_KEY, JSON.stringify(index));
  } catch (e) {
    console.error('Failed to save cache index:', e);
  }
}

// Store
function createOfflineStore() {
  const { subscribe, set, update } = writable<OfflineState>({
    isOffline: false,
    lastOnlineAt: null,
    pendingOperations: [],
    cacheStats: {
      itemCount: 0,
      totalSize: 0,
      oldestItem: null,
      newestItem: null,
    },
  });

  // Load pending operations from storage
  function loadPendingOperations(): PendingOperation[] {
    try {
      const stored = localStorage.getItem('midi_pending_ops');
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  // Save pending operations to storage
  function savePendingOperations(ops: PendingOperation[]): void {
    try {
      localStorage.setItem('midi_pending_ops', JSON.stringify(ops));
    } catch (e) {
      console.error('Failed to save pending operations:', e);
    }
  }

  // Cache operations
  function cacheItem<T>(key: string, data: T, ttl: number = DEFAULT_TTL): void {
    const cacheKey = CACHE_PREFIX + key;
    const item: CachedItem<T> = {
      data,
      cachedAt: Date.now(),
      expiresAt: Date.now() + ttl,
      key,
    };

    try {
      const serialized = JSON.stringify(item);

      // Check if we need to evict items
      const currentSize = getCacheSize();
      if (currentSize + serialized.length > MAX_CACHE_SIZE) {
        evictOldestItems(serialized.length);
      }

      localStorage.setItem(cacheKey, serialized);

      // Update index
      const index = getCacheIndex();
      if (!index.keys.includes(key)) {
        index.keys.push(key);
      }
      index.lastUpdated = Date.now();
      saveCacheIndex(index);

      updateCacheStats();
    } catch (e) {
      console.error('Failed to cache item:', e);
      // Try to evict and retry
      evictOldestItems(MAX_CACHE_SIZE / 4);
      try {
        localStorage.setItem(cacheKey, JSON.stringify(item));
      } catch {
        console.error('Cache storage full, unable to cache item');
      }
    }
  }

  function getCachedItem<T>(key: string): T | null {
    const cacheKey = CACHE_PREFIX + key;
    try {
      const stored = localStorage.getItem(cacheKey);
      if (!stored) return null;

      const item: CachedItem<T> = JSON.parse(stored);

      // Check expiration
      if (Date.now() > item.expiresAt) {
        removeCachedItem(key);
        return null;
      }

      return item.data;
    } catch {
      return null;
    }
  }

  function removeCachedItem(key: string): void {
    const cacheKey = CACHE_PREFIX + key;
    localStorage.removeItem(cacheKey);

    // Update index
    const index = getCacheIndex();
    index.keys = index.keys.filter(k => k !== key);
    index.lastUpdated = Date.now();
    saveCacheIndex(index);

    updateCacheStats();
  }

  function getCacheSize(): number {
    let size = 0;
    const index = getCacheIndex();

    for (const key of index.keys) {
      const item = localStorage.getItem(CACHE_PREFIX + key);
      if (item) {
        size += item.length * 2; // UTF-16 encoding
      }
    }

    return size;
  }

  function evictOldestItems(bytesNeeded: number): void {
    const index = getCacheIndex();
    const items: { key: string; cachedAt: number; size: number }[] = [];

    for (const key of index.keys) {
      const stored = localStorage.getItem(CACHE_PREFIX + key);
      if (stored) {
        try {
          const item = JSON.parse(stored);
          items.push({
            key,
            cachedAt: item.cachedAt,
            size: stored.length * 2,
          });
        } catch {
          // Invalid item, remove it
          localStorage.removeItem(CACHE_PREFIX + key);
        }
      }
    }

    // Sort by age (oldest first)
    items.sort((a, b) => a.cachedAt - b.cachedAt);

    let freedBytes = 0;
    const keysToRemove: string[] = [];

    for (const item of items) {
      if (freedBytes >= bytesNeeded) break;
      keysToRemove.push(item.key);
      freedBytes += item.size;
    }

    for (const key of keysToRemove) {
      localStorage.removeItem(CACHE_PREFIX + key);
    }

    // Update index
    index.keys = index.keys.filter(k => !keysToRemove.includes(k));
    index.lastUpdated = Date.now();
    saveCacheIndex(index);

    console.log(`Evicted ${keysToRemove.length} cache items, freed ${freedBytes} bytes`);
  }

  function updateCacheStats(): void {
    const index = getCacheIndex();
    let totalSize = 0;
    let oldestItem: number | null = null;
    let newestItem: number | null = null;

    for (const key of index.keys) {
      const stored = localStorage.getItem(CACHE_PREFIX + key);
      if (stored) {
        totalSize += stored.length * 2;
        try {
          const item = JSON.parse(stored);
          if (oldestItem === null || item.cachedAt < oldestItem) {
            oldestItem = item.cachedAt;
          }
          if (newestItem === null || item.cachedAt > newestItem) {
            newestItem = item.cachedAt;
          }
        } catch {
          // Skip invalid items
        }
      }
    }

    update(s => ({
      ...s,
      cacheStats: {
        itemCount: index.keys.length,
        totalSize,
        oldestItem,
        newestItem,
      },
    }));
  }

  // Pending operations
  function addPendingOperation(
    type: PendingOperation['type'],
    entity: string,
    data: unknown
  ): string {
    const id = crypto.randomUUID();
    const operation: PendingOperation = {
      id,
      type,
      entity,
      data,
      createdAt: Date.now(),
      retryCount: 0,
    };

    update(s => {
      let ops = [...s.pendingOperations, operation];

      // Limit pending operations
      if (ops.length > MAX_PENDING_OPERATIONS) {
        ops = ops.slice(-MAX_PENDING_OPERATIONS);
      }

      savePendingOperations(ops);
      return { ...s, pendingOperations: ops };
    });

    return id;
  }

  function removePendingOperation(id: string): void {
    update(s => {
      const ops = s.pendingOperations.filter(op => op.id !== id);
      savePendingOperations(ops);
      return { ...s, pendingOperations: ops };
    });
  }

  async function syncPendingOperations(): Promise<{ synced: number; failed: number }> {
    const state = get({ subscribe });
    let synced = 0;
    let failed = 0;

    for (const op of state.pendingOperations) {
      try {
        await executePendingOperation(op);
        removePendingOperation(op.id);
        synced++;
      } catch (error) {
        console.error(`Failed to sync operation ${op.id}:`, error);

        // Increment retry count
        update(s => ({
          ...s,
          pendingOperations: s.pendingOperations.map(o =>
            o.id === op.id ? { ...o, retryCount: o.retryCount + 1 } : o
          ),
        }));

        failed++;

        // Remove operations that have failed too many times
        if (op.retryCount >= 5) {
          removePendingOperation(op.id);
          console.warn(`Operation ${op.id} removed after 5 failed retries`);
        }
      }
    }

    return { synced, failed };
  }

  async function executePendingOperation(op: PendingOperation): Promise<void> {
    // This should be implemented based on your actual API
    const { invoke } = await import('@tauri-apps/api/core');

    switch (op.type) {
      case 'create':
        await invoke(`create_${op.entity}`, { data: op.data });
        break;
      case 'update':
        await invoke(`update_${op.entity}`, { data: op.data });
        break;
      case 'delete':
        await invoke(`delete_${op.entity}`, { data: op.data });
        break;
    }
  }

  function clearCache(): void {
    const index = getCacheIndex();
    for (const key of index.keys) {
      localStorage.removeItem(CACHE_PREFIX + key);
    }
    localStorage.removeItem(CACHE_INDEX_KEY);
    updateCacheStats();
  }

  // Initialize
  const pendingOps = loadPendingOperations();
  set({
    isOffline: false,
    lastOnlineAt: Date.now(),
    pendingOperations: pendingOps,
    cacheStats: {
      itemCount: 0,
      totalSize: 0,
      oldestItem: null,
      newestItem: null,
    },
  });
  updateCacheStats();

  // Listen for health changes
  healthStore.subscribe(health => {
    const wasOffline = get({ subscribe }).isOffline;
    const isNowOffline = health.health?.overall_status === 'unhealthy';

    if (wasOffline && !isNowOffline) {
      // Coming back online - sync pending operations
      syncPendingOperations().then(({ synced, failed }) => {
        if (synced > 0) {
          console.log(`Synced ${synced} pending operations`);
        }
        if (failed > 0) {
          console.warn(`Failed to sync ${failed} operations`);
        }
      });
    }

    update(s => ({
      ...s,
      isOffline: isNowOffline,
      lastOnlineAt: isNowOffline ? s.lastOnlineAt : Date.now(),
    }));
  });

  return {
    subscribe,
    cacheItem,
    getCachedItem,
    removeCachedItem,
    addPendingOperation,
    removePendingOperation,
    syncPendingOperations,
    clearCache,
    getCacheSize,

    // Convenience methods
    cacheSearchResults: (query: string, results: unknown) =>
      cacheItem(`search:${query}`, results, DEFAULT_TTL),
    getCachedSearch: (query: string) =>
      getCachedItem(`search:${query}`),
    cacheFileDetails: (fileId: number, details: unknown) =>
      cacheItem(`file:${fileId}`, details, DEFAULT_TTL * 24),
    getCachedFileDetails: (fileId: number) =>
      getCachedItem(`file:${fileId}`),
  };
}

export const offlineStore = createOfflineStore();

// Derived stores
export const isOffline = derived(offlineStore, $store => $store.isOffline);

export const pendingOperationsCount = derived(
  offlineStore,
  $store => $store.pendingOperations.length
);

export const hasPendingOperations = derived(
  offlineStore,
  $store => $store.pendingOperations.length > 0
);
