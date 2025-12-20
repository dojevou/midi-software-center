<script lang="ts">
  import type { SavedSearchResponse } from '$lib/types/vip3';

  export let search: SavedSearchResponse;
  export let onLoad: () => void = () => {};
  export let onDelete: () => void = () => {};
  export let onTogglePin: () => void = () => {};

  function formatDate(dateStr: string | undefined): string {
    if (!dateStr) return 'Never';
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function getFilterSummary(filters: any): string {
    if (!filters) return 'No filters';

    const parts: string[] = [];
    if (filters.folder_ids?.length) parts.push(`${filters.folder_ids.length} folders`);
    if (filters.instrument_ids?.length) parts.push(`${filters.instrument_ids.length} instruments`);
    if (filters.timbre_ids?.length) parts.push(`${filters.timbre_ids.length} timbres`);
    if (filters.style_ids?.length) parts.push(`${filters.style_ids.length} styles`);
    if (filters.bpm_range_ids?.length) parts.push('BPM filter');
    if (filters.key_ids?.length) parts.push('Key filter');
    if (filters.favorites_only) parts.push('Favorites only');

    return parts.length > 0 ? parts.join(', ') : 'No filters';
  }
</script>

<div class="saved-search-item" style:border-left-color={search.color || '#007aff'}>
  <div class="content">
    <div class="header-row">
      <div class="title-section">
        {#if search.icon}
          <span class="icon">{search.icon}</span>
        {/if}
        <h4>{search.name}</h4>
        {#if search.is_pinned}
          <span class="pin-badge" title="Pinned">📌</span>
        {/if}
      </div>
      <div class="actions">
        <button
          class="btn-icon"
          on:click|stopPropagation={onTogglePin}
          title={search.is_pinned ? 'Unpin' : 'Pin'}
        >
          {search.is_pinned ? '📌' : '📍'}
        </button>
        <button
          class="btn-icon btn-delete"
          on:click|stopPropagation={onDelete}
          title="Delete"
        >
          🗑️
        </button>
      </div>
    </div>

    {#if search.description}
      <p class="description">{search.description}</p>
    {/if}

    <div class="filter-summary">
      {getFilterSummary(search.filters)}
    </div>

    <div class="meta">
      <span class="use-count" title="Times used">
        🔄 {search.use_count || 0}
      </span>
      <span class="last-used" title="Last used">
        🕒 {formatDate(search.last_used)}
      </span>
    </div>
  </div>

  <button class="btn-load" on:click={onLoad}>
    Load
  </button>
</div>

<style>
  .saved-search-item {
    display: flex;
    align-items: stretch;
    background: var(--color-bg-tertiary, #333);
    border-radius: 8px;
    border-left: 4px solid;
    margin-bottom: 8px;
    overflow: hidden;
    transition: all 0.2s;
  }

  .saved-search-item:hover {
    transform: translateX(2px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .content {
    flex: 1;
    padding: 12px;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 8px;
  }

  .title-section {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }

  .icon {
    font-size: 20px;
  }

  h4 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text, #fff);
  }

  .pin-badge {
    font-size: 14px;
    opacity: 0.7;
  }

  .actions {
    display: flex;
    gap: 4px;
  }

  .btn-icon {
    padding: 4px 8px;
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 16px;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .btn-icon:hover {
    opacity: 1;
  }

  .btn-delete:hover {
    opacity: 1;
    filter: brightness(1.2);
  }

  .description {
    margin: 0 0 8px 0;
    font-size: 13px;
    color: var(--color-text-secondary, #999);
    line-height: 1.4;
  }

  .filter-summary {
    padding: 6px 10px;
    background: var(--color-bg-secondary, #2a2a2a);
    border-radius: 4px;
    font-size: 12px;
    color: var(--color-text-secondary, #999);
    margin-bottom: 8px;
  }

  .meta {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--color-text-tertiary, #666);
  }

  .use-count,
  .last-used {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .btn-load {
    padding: 0 20px;
    background: var(--color-primary, #007aff);
    color: white;
    border: none;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    transition: background 0.2s;
    align-self: stretch;
  }

  .btn-load:hover {
    background: var(--color-primary-hover, #0056b3);
  }
</style>
