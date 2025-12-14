<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { api } from '$lib/api';
  import type { Project } from '$lib/types';

  export let limit: number = 10;
  export let showEmptyState: boolean = true;
  export let compact: boolean = false;

  const dispatch = createEventDispatcher<{
    projectClick: { project: Project };
    newProject: void;
  }>();

  let recentProjects: Project[] = [];
  let isLoading = true;
  let error: string | null = null;

  onMount(async () => {
    await loadRecentProjects();
  });

  async function loadRecentProjects() {
    isLoading = true;
    error = null;
    try {
      recentProjects = await api.project.getRecent(limit);
    } catch (err) {
      console.error('Failed to load recent projects:', err);
      error = err instanceof Error ? err.message : 'Failed to load projects';
    } finally {
      isLoading = false;
    }
  }

  export async function refresh() {
    await loadRecentProjects();
  }

  function handleProjectClick(project: Project) {
    dispatch('projectClick', { project });
  }

  function handleNewProject() {
    dispatch('newProject');
  }

  function formatDate(dateString: string | null): string {
    if (!dateString) {
      return 'Never';
    }
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffHours < 1) {
      return 'Just now';
    }
    if (diffHours < 24) {
      return `${diffHours}h ago`;
    }
    if (diffDays < 7) {
      return `${diffDays}d ago`;
    }
    return date.toLocaleDateString();
  }

  function getProjectIcon(project: Project): string {
    // Could be based on template or tags in the future
    return '🎹';
  }
</script>

<div class="recent-projects" class:compact>
  <div class="header">
    <h3 class="title">Recent Projects</h3>
    <button type="button" class="refresh-btn" on:click={loadRecentProjects} title="Refresh">
      ↻
    </button>
  </div>

  {#if isLoading}
    <div class="loading">
      <span class="spinner">⏳</span>
      <span>Loading projects...</span>
    </div>
  {:else if error}
    <div class="error">
      <span>⚠️</span>
      <span>{error}</span>
      <button type="button" on:click={loadRecentProjects}>Retry</button>
    </div>
  {:else if recentProjects.length === 0 && showEmptyState}
    <div class="empty-state">
      <span class="empty-icon">📂</span>
      <p class="empty-text">No recent projects</p>
      <button type="button" class="new-project-btn" on:click={handleNewProject}>
        + Create New Project
      </button>
    </div>
  {:else}
    <ul class="project-list">
      {#each recentProjects as project (project.id)}
        <li class="project-item">
          <button
            type="button"
            class="project-button"
            on:click={() => handleProjectClick(project)}
          >
            <span class="project-icon">{getProjectIcon(project)}</span>
            <div class="project-info">
              <span class="project-name">{project.name}</span>
              {#if !compact}
                <span class="project-meta">
                  <span class="meta-item">{formatDate(project.last_opened || project.modified_at)}</span>
                </span>
              {/if}
            </div>
            {#if !compact && project.track_count}
              <span class="track-count">{project.track_count} tracks</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>

    {#if !compact}
      <div class="footer">
        <button type="button" class="new-project-btn" on:click={handleNewProject}>
          + New Project
        </button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .recent-projects {
    display: flex;
    flex-direction: column;
    background: var(--gray-900, #18181b);
    border-radius: 8px;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--gray-850, #1f1f23);
    border-bottom: 1px solid var(--gray-700, #3f3f46);
  }

  .title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--gray-100, #f4f4f5);
  }

  .refresh-btn {
    background: transparent;
    border: none;
    color: var(--gray-400, #a1a1aa);
    cursor: pointer;
    font-size: 16px;
    padding: 4px 8px;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .refresh-btn:hover {
    background: var(--gray-700, #3f3f46);
    color: var(--gray-200, #e4e4e7);
  }

  .loading,
  .error,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px 16px;
    color: var(--gray-400, #a1a1aa);
    gap: 8px;
  }

  .spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .error {
    color: var(--red-400, #f87171);
  }

  .error button {
    margin-top: 8px;
    background: var(--gray-700, #3f3f46);
    border: none;
    color: var(--gray-200, #e4e4e7);
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
  }

  .empty-icon {
    font-size: 48px;
    opacity: 0.5;
  }

  .empty-text {
    margin: 0;
    font-size: 14px;
  }

  .project-list {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 400px;
    overflow-y: auto;
  }

  .project-item {
    border-bottom: 1px solid var(--gray-800, #27272a);
  }

  .project-item:last-child {
    border-bottom: none;
  }

  .project-button {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 16px;
    background: transparent;
    border: none;
    color: var(--gray-100, #f4f4f5);
    cursor: pointer;
    text-align: left;
    transition: background-color 0.15s ease;
  }

  .project-button:hover {
    background: var(--gray-800, #27272a);
  }

  .project-icon {
    font-size: 24px;
    flex-shrink: 0;
  }

  .compact .project-icon {
    font-size: 18px;
  }

  .project-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .project-name {
    font-size: 14px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .compact .project-name {
    font-size: 13px;
  }

  .project-meta {
    display: flex;
    gap: 8px;
    font-size: 11px;
    color: var(--gray-500, #71717a);
  }

  .meta-item::before {
    content: '•';
    margin-right: 8px;
  }

  .meta-item:first-child::before {
    display: none;
  }

  .track-count {
    font-size: 11px;
    color: var(--gray-500, #71717a);
    background: var(--gray-800, #27272a);
    padding: 2px 8px;
    border-radius: 10px;
    flex-shrink: 0;
  }

  .footer {
    padding: 12px 16px;
    border-top: 1px solid var(--gray-700, #3f3f46);
  }

  .new-project-btn {
    width: 100%;
    padding: 10px 16px;
    background: var(--primary-600, #2563eb);
    border: none;
    border-radius: 6px;
    color: white;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .new-project-btn:hover {
    background: var(--primary-500, #3b82f6);
  }

  /* Compact mode adjustments */
  .compact .header {
    padding: 8px 12px;
  }

  .compact .title {
    font-size: 12px;
  }

  .compact .project-button {
    padding: 8px 12px;
    gap: 8px;
  }

  .compact .project-list {
    max-height: 250px;
  }

  /* Scrollbar styling */
  .project-list::-webkit-scrollbar {
    width: 6px;
  }

  .project-list::-webkit-scrollbar-track {
    background: var(--gray-800, #27272a);
  }

  .project-list::-webkit-scrollbar-thumb {
    background: var(--gray-600, #52525b);
    border-radius: 3px;
  }

  .project-list::-webkit-scrollbar-thumb:hover {
    background: var(--gray-500, #71717a);
  }
</style>
