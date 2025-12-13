import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { offlineStore, isOffline } from '$lib/stores/offlineStore';
import { healthStore } from '$lib/stores/healthStore';

export interface OfflineApiOptions {
  cacheKey?: string;
  cacheTtl?: number;
  allowOffline?: boolean;
  queueIfOffline?: boolean;
}

export class OfflineApiError extends Error {
  constructor(
    message: string,
    public readonly isOfflineError: boolean = false,
    public readonly cachedData?: unknown
  ) {
    super(message);
    this.name = 'OfflineApiError';
  }
}

export async function offlineInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
  options: OfflineApiOptions = {}
): Promise<T> {
  const {
    cacheKey,
    cacheTtl,
    allowOffline = true,
    queueIfOffline = false,
  } = options;

  const offline = get(isOffline);

  // If online, make the request
  if (!offline) {
    try {
      const result = await invoke<T>(command, args);

      // Cache the result if cacheKey is provided
      if (cacheKey) {
        offlineStore.cacheItem(cacheKey, result, cacheTtl);
      }

      return result;
    } catch (error) {
      // Check if this is a connection error
      if (isConnectionError(error)) {
        // Re-check health
        await healthStore.checkHealth();

        // If we're now offline, try cache
        if (allowOffline && cacheKey) {
          const cached = offlineStore.getCachedItem<T>(cacheKey);
          if (cached !== null) {
            console.log(`Returning cached data for ${command}`);
            return cached;
          }
        }
      }
      throw error;
    }
  }

  // We're offline
  if (!allowOffline) {
    throw new OfflineApiError(
      `Cannot execute ${command}: system is offline`,
      true
    );
  }

  // Try to get cached data
  if (cacheKey) {
    const cached = offlineStore.getCachedItem<T>(cacheKey);
    if (cached !== null) {
      console.log(`Returning cached data for ${command} (offline)`);
      return cached;
    }
  }

  // Queue the operation if it's a mutation
  if (queueIfOffline && args) {
    const entity = command.replace(/^(create_|update_|delete_)/, '');
    const type = command.startsWith('create_')
      ? 'create'
      : command.startsWith('update_')
      ? 'update'
      : command.startsWith('delete_')
      ? 'delete'
      : null;

    if (type) {
      offlineStore.addPendingOperation(type, entity, args);
      throw new OfflineApiError(
        `Operation queued for sync when online`,
        true
      );
    }
  }

  throw new OfflineApiError(
    `Cannot execute ${command}: no cached data available`,
    true
  );
}

function isConnectionError(error: unknown): boolean {
  if (error instanceof Error) {
    const message = error.message.toLowerCase();
    return (
      message.includes('connection') ||
      message.includes('network') ||
      message.includes('offline') ||
      message.includes('timeout') ||
      message.includes('econnrefused')
    );
  }
  return false;
}

// Convenience wrappers for common operations
export const offlineApi = {
  async searchFiles(query: string, filters?: Record<string, unknown>) {
    return offlineInvoke('search_files', { query, filters }, {
      cacheKey: `search:${query}:${JSON.stringify(filters || {})}`,
      cacheTtl: 1000 * 60 * 30, // 30 minutes
      allowOffline: true,
    });
  },

  async getFileDetails(fileId: number) {
    return offlineInvoke('get_file_details', { fileId }, {
      cacheKey: `file:${fileId}`,
      cacheTtl: 1000 * 60 * 60 * 24, // 24 hours
      allowOffline: true,
    });
  },

  async getTags() {
    return offlineInvoke('get_all_tags', undefined, {
      cacheKey: 'tags:all',
      cacheTtl: 1000 * 60 * 60, // 1 hour
      allowOffline: true,
    });
  },

  async updateFileTags(fileId: number, tagIds: number[]) {
    return offlineInvoke('update_file_tags', { fileId, tagIds }, {
      queueIfOffline: true,
    });
  },

  async createTag(name: string, category: string) {
    return offlineInvoke('create_tag', { name, category }, {
      queueIfOffline: true,
    });
  },
};
