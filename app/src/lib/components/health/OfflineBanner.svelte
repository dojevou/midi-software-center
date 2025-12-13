<script lang="ts">
  import { offlineStore, isOffline, pendingOperationsCount } from '$lib/stores/offlineStore';
  import { slide } from 'svelte/transition';

  export let position: 'top' | 'bottom' = 'top';

  function formatLastOnline(timestamp: number | null): string {
    if (!timestamp) return 'Unknown';

    const diff = Date.now() - timestamp;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return new Date(timestamp).toLocaleDateString();
  }

  async function retrySyncNow() {
    const { synced, failed } = await offlineStore.syncPendingOperations();
    console.log(`Manual sync: ${synced} synced, ${failed} failed`);
  }
</script>

{#if $isOffline}
  <div
    class="offline-banner position-{position}"
    role="alert"
    aria-live="assertive"
    transition:slide={{ duration: 200 }}
  >
    <div class="banner-content">
      <span class="offline-icon">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="1" y1="1" x2="23" y2="23" />
          <path d="M16.72 11.06A10.94 10.94 0 0 1 19 12.55" />
          <path d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39" />
          <path d="M10.71 5.05A16 16 0 0 1 22.58 9" />
          <path d="M1.42 9a15.91 15.91 0 0 1 4.7-2.88" />
          <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
          <line x1="12" y1="20" x2="12.01" y2="20" />
        </svg>
      </span>
      <span class="offline-text">
        You're offline
        {#if $offlineStore.lastOnlineAt}
          <span class="last-online">
            (Last online: {formatLastOnline($offlineStore.lastOnlineAt)})
          </span>
        {/if}
      </span>
    </div>

    {#if $pendingOperationsCount > 0}
      <div class="pending-ops">
        <span>{$pendingOperationsCount} pending change{$pendingOperationsCount !== 1 ? 's' : ''}</span>
        <button class="sync-button" on:click={retrySyncNow}>
          Retry now
        </button>
      </div>
    {/if}

    <div class="banner-info">
      <p>Some features may be limited. Cached data will be shown where available.</p>
    </div>
  </div>
{/if}

<style>
  .offline-banner {
    position: fixed;
    left: 0;
    right: 0;
    padding: 12px 20px;
    background: linear-gradient(135deg, #92400e 0%, #78350f 100%);
    color: white;
    z-index: 9999;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .position-top {
    top: 0;
  }

  .position-bottom {
    bottom: 0;
  }

  .banner-content {
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 600;
  }

  .offline-icon {
    display: flex;
    align-items: center;
  }

  .offline-text {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .last-online {
    font-size: 12px;
    font-weight: normal;
    opacity: 0.8;
  }

  .pending-ops {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    font-size: 13px;
  }

  .sync-button {
    padding: 4px 12px;
    background: rgba(255, 255, 255, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 4px;
    color: white;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .sync-button:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .banner-info {
    margin-top: 8px;
    font-size: 12px;
    opacity: 0.9;
  }

  .banner-info p {
    margin: 0;
  }
</style>
