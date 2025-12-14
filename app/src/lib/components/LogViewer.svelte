<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { logStore, filteredLogs } from '$lib/stores/logStore';

  export let maxHeight = '400px';
  export let autoScroll = true;

  let containerRef: HTMLDivElement;
  let unlisten: (() => void) | undefined;

  const levels = ['error', 'warn', 'info', 'debug', 'trace'];
  const categories = ['database', 'midi', 'file', 'network', 'system', 'performance'];

  function getLevelColor(level: string): string {
    switch (level.toLowerCase()) {
      case 'error':
        return 'var(--error)';
      case 'warn':
        return 'var(--warning)';
      case 'info':
        return 'var(--info)';
      case 'debug':
        return 'var(--text-muted)';
      case 'trace':
        return 'var(--text-muted)';
      default:
        return 'var(--text-secondary)';
    }
  }

  function formatTimestamp(ts: string): string {
    return new Date(ts).toLocaleTimeString();
  }

  function scrollToBottom() {
    if (autoScroll && containerRef) {
      containerRef.scrollTop = containerRef.scrollHeight;
    }
  }

  $: if ($filteredLogs.length && autoScroll) {
    requestAnimationFrame(scrollToBottom);
  }

  onMount(async () => {
    await logStore.fetchLogs();
    unlisten = await logStore.startStreaming();
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<div class="log-viewer">
  <div class="log-controls">
    <select
      value={$logStore.filter.level || ''}
      on:change={(e) => logStore.setFilter({ level: e.currentTarget.value || null })}
      aria-label="Filter by level"
    >
      <option value="">All Levels</option>
      {#each levels as level (level)}
        <option value={level}>{level.toUpperCase()}</option>
      {/each}
    </select>

    <select
      value={$logStore.filter.category || ''}
      on:change={(e) => logStore.setFilter({ category: e.currentTarget.value || null })}
      aria-label="Filter by category"
    >
      <option value="">All Categories</option>
      {#each categories as category (category)}
        <option value={category}>{category}</option>
      {/each}
    </select>

    <input
      type="text"
      placeholder="Search logs..."
      value={$logStore.filter.search}
      on:input={(e) => logStore.setFilter({ search: e.currentTarget.value })}
      aria-label="Search logs"
    />

    <label class="auto-scroll">
      <input type="checkbox" bind:checked={autoScroll} />
      Auto-scroll
    </label>

    <button on:click={() => logStore.clearLogs()}>Clear</button>
    <button on:click={() => logStore.fetchLogs()}>Refresh</button>
  </div>

  <div
    bind:this={containerRef}
    class="log-entries"
    style="max-height: {maxHeight}"
    role="log"
    aria-live="polite"
  >
    {#each $filteredLogs as entry (entry.timestamp + entry.message)}
      <div
        class="log-entry"
        class:error={entry.level === 'error'}
        class:warn={entry.level === 'warn'}
      >
        <span class="log-time">{formatTimestamp(entry.timestamp)}</span>
        <span class="log-level" style="color: {getLevelColor(entry.level)}">
          [{entry.level.toUpperCase()}]
        </span>
        <span class="log-category">[{entry.category}]</span>
        <span class="log-message">{entry.message}</span>
        {#if entry.target}
          <span class="log-target">{entry.target}</span>
        {/if}
      </div>
    {/each}

    {#if $filteredLogs.length === 0}
      <div class="log-empty">No log entries</div>
    {/if}
  </div>

  <div class="log-footer">
    <span>{$filteredLogs.length} entries</span>
    {#if $logStore.isStreaming}
      <span class="streaming-indicator">Streaming</span>
    {/if}
  </div>
</div>

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    overflow: hidden;
  }

  .log-controls {
    display: flex;
    gap: 8px;
    padding: 8px;
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }

  .log-controls select,
  .log-controls input[type='text'] {
    padding: 4px 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius-sm);
    color: var(--text-primary);
    font-size: var(--font-size-sm);
  }

  .log-controls input[type='text'] {
    flex: 1;
    min-width: 150px;
  }

  .auto-scroll {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .log-controls button {
    padding: 4px 12px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: var(--font-size-sm);
  }

  .log-controls button:hover {
    background: var(--bg-hover);
  }

  .log-entries {
    flex: 1;
    overflow-y: auto;
    font-family: monospace;
    font-size: 12px;
  }

  .log-entry {
    display: flex;
    gap: 8px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }

  .log-entry:hover {
    background: var(--bg-hover);
  }

  .log-entry.error {
    background: rgba(220, 53, 69, 0.1);
  }

  .log-entry.warn {
    background: rgba(255, 193, 7, 0.1);
  }

  .log-time {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .log-level {
    font-weight: 600;
    flex-shrink: 0;
    width: 60px;
  }

  .log-category {
    color: var(--accent);
    flex-shrink: 0;
    width: 100px;
  }

  .log-message {
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .log-target {
    color: var(--text-muted);
    font-size: 10px;
    flex-shrink: 0;
  }

  .log-empty {
    padding: 24px;
    text-align: center;
    color: var(--text-muted);
  }

  .log-footer {
    display: flex;
    justify-content: space-between;
    padding: 4px 8px;
    border-top: 1px solid var(--border);
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }

  .streaming-indicator {
    color: var(--success);
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
</style>
