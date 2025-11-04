<script lang="ts">
  /**
   * ProjectBrowserWindow - Project Manager
   *
   * Task-O-Matic: Browse, open, and manage DAW projects with recent and grid views.
   */

  import { onMount } from 'svelte';
  import { debounce } from '$lib/utils/debounce';
  import WindowBase from '$lib/components/WindowBase.svelte';
  import { projectActions } from '$lib/stores/projectStore';

  // ============================================================================
  // Types
  // ============================================================================

  interface Project {
    id: string;
    name: string;
    path: string;
    lastModified: Date;
    created: Date;
    size: number; // bytes
    tempo: number;
    tracks: number;
    duration: number; // seconds
    thumbnail?: string;
    favorite: boolean;
  }

  type SortBy = 'name' | 'date' | 'size';
  type ViewMode = 'grid' | 'list';

  // ============================================================================
  // State
  // ============================================================================

  let allProjects: Project[] = [];
  let filteredProjects: Project[] = [];
  let recentProjects: Project[] = [];

  let searchQuery = '';
  let sortBy: SortBy = 'date';
  let viewMode: ViewMode = 'grid';
  let showFavoritesOnly = false;

  let selectedProject: Project | null = null;
  let contextMenuProject: Project | null = null;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let showContextMenu = false;

  // ============================================================================
  // Filter and Sort
  // ============================================================================

  function filterAndSortProjects(): void {
    let results = [...allProjects];

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      results = results.filter(p =>
        p.name.toLowerCase().includes(query) ||
        p.path.toLowerCase().includes(query)
      );
    }

    // Filter favorites
    if (showFavoritesOnly) {
      results = results.filter(p => p.favorite);
    }

    // Sort
    results.sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.name.localeCompare(b.name);
        case 'date':
          return b.lastModified.getTime() - a.lastModified.getTime();
        case 'size':
          return b.size - a.size;
        default:
          return 0;
      }
    });

    filteredProjects = results;
  }

  const debouncedFilter = debounce(filterAndSortProjects, 300);

  function handleSearchInput(): void {
    debouncedFilter();
  }

  function updateRecentProjects(): void {
    recentProjects = [...allProjects]
      .sort((a, b) => b.lastModified.getTime() - a.lastModified.getTime())
      .slice(0, 10);
  }

  // ============================================================================
  // Project Actions
  // ============================================================================

  async function openProject(project: Project): Promise<void> {
    try {
      await projectActions.loadProject(project.id);
      console.log('Opened project:', project.name);
      // Update last modified date
      project.lastModified = new Date();
      updateRecentProjects();
      filterAndSortProjects();
    } catch (error) {
      console.error('Failed to open project:', error);
      alert(`Failed to open project: ${error}`);
    }
  }

  async function createNewProject(): Promise<void> {
    const name = prompt('Enter project name:');
    if (!name) return;

    try {
      await projectActions.newProject(name);
      console.log('Created new project:', name);

      // Add to projects list
      const newProject: Project = {
        id: crypto.randomUUID(),
        name,
        path: `/projects/${name}`,
        lastModified: new Date(),
        created: new Date(),
        size: 0,
        tempo: 120,
        tracks: 0,
        duration: 0,
        favorite: false
      };

      allProjects = [...allProjects, newProject];
      updateRecentProjects();
      filterAndSortProjects();
    } catch (error) {
      console.error('Failed to create project:', error);
      alert(`Failed to create project: ${error}`);
    }
  }

  function deleteProject(project: Project): void {
    if (!confirm(`Delete project "${project.name}"? This cannot be undone.`)) {
      return;
    }

    allProjects = allProjects.filter(p => p.id !== project.id);
    if (selectedProject?.id === project.id) {
      selectedProject = null;
    }
    updateRecentProjects();
    filterAndSortProjects();
  }

  function renameProject(project: Project): void {
    const newName = prompt('Enter new name:', project.name);
    if (!newName || newName === project.name) return;

    allProjects = allProjects.map(p =>
      p.id === project.id ? { ...p, name: newName } : p
    );
    filterAndSortProjects();
  }

  function toggleFavorite(project: Project): void {
    allProjects = allProjects.map(p =>
      p.id === project.id ? { ...p, favorite: !p.favorite } : p
    );
    filterAndSortProjects();
  }

  function showInFolder(project: Project): void {
    console.log('Show in folder:', project.path);
    // TODO: Integrate with Tauri to open file explorer
  }

  function showProperties(project: Project): void {
    alert(`Project Properties:\n
Name: ${project.name}
Path: ${project.path}
Size: ${formatSize(project.size)}
Tracks: ${project.tracks}
Tempo: ${project.tempo} BPM
Duration: ${formatDuration(project.duration)}
Created: ${formatDate(project.created)}
Modified: ${formatDate(project.lastModified)}`);
  }

  // ============================================================================
  // Context Menu
  // ============================================================================

  function handleContextMenu(e: MouseEvent, project: Project): void {
    e.preventDefault();
    contextMenuProject = project;
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    showContextMenu = true;
  }

  function closeContextMenu(): void {
    showContextMenu = false;
    contextMenuProject = null;
  }

  function handleContextMenuAction(action: string): void {
    if (!contextMenuProject) return;

    switch (action) {
      case 'open':
        openProject(contextMenuProject);
        break;
      case 'rename':
        renameProject(contextMenuProject);
        break;
      case 'delete':
        deleteProject(contextMenuProject);
        break;
      case 'folder':
        showInFolder(contextMenuProject);
        break;
      case 'properties':
        showProperties(contextMenuProject);
        break;
    }

    closeContextMenu();
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatDate(date: Date): string {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  }

  function getRelativeTime(date: Date): string {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins} min ago`;
    if (diffHours < 24) return `${diffHours} hours ago`;
    if (diffDays < 7) return `${diffDays} days ago`;
    return formatDate(date);
  }

  // ============================================================================
  // Lifecycle
  // ============================================================================

  onMount(() => {
    // Load demo projects
    allProjects = Array.from({ length: 25 }, (_, i) => ({
      id: crypto.randomUUID(),
      name: `Project ${i + 1}`,
      path: `/projects/project_${i + 1}`,
      lastModified: new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000),
      created: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000),
      size: Math.floor(Math.random() * 50 * 1024 * 1024), // 0-50MB
      tempo: 60 + Math.floor(Math.random() * 140),
      tracks: Math.floor(Math.random() * 16) + 1,
      duration: Math.floor(Math.random() * 300) + 30,
      favorite: Math.random() > 0.8
    }));

    updateRecentProjects();
    filterAndSortProjects();

    // Close context menu on click
    window.addEventListener('click', closeContextMenu);

    return () => {
      window.removeEventListener('click', closeContextMenu);
    };
  });

  $: totalSize = allProjects.reduce((sum, p) => sum + p.size, 0);
</script>

<WindowBase windowId="project-browser" title="Project Browser">
  <div class="project-browser-window">
    <!-- Toolbar -->
    <div class="toolbar">
      <button class="new-project-btn" on:click={createNewProject}>
        + New Project
      </button>

      <div class="search-container">
        <input
          type="text"
          placeholder="Search projects..."
          bind:value={searchQuery}
          on:input={handleSearchInput}
          class="search-input"
        />
      </div>

      <div class="toolbar-controls">
        <label class="sort-label">
          Sort by:
          <select bind:value={sortBy} on:change={filterAndSortProjects} class="sort-select">
            <option value="date">Date Modified</option>
            <option value="name">Name</option>
            <option value="size">Size</option>
          </select>
        </label>

        <button
          class="view-btn"
          class:active={viewMode === 'grid'}
          on:click={() => viewMode = 'grid'}
          title="Grid view"
        >
          ⊞
        </button>
        <button
          class="view-btn"
          class:active={viewMode === 'list'}
          on:click={() => viewMode = 'list'}
          title="List view"
        >
          ☰
        </button>

        <button
          class="favorites-btn"
          class:active={showFavoritesOnly}
          on:click={() => { showFavoritesOnly = !showFavoritesOnly; filterAndSortProjects(); }}
          title="Show favorites only"
        >
          ★
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <div class="main-content">
      <!-- Recent Projects Column -->
      <div class="recent-column">
        <div class="section-header">
          <h3 class="section-title">Recent Projects</h3>
          <span class="section-count">{recentProjects.length}</span>
        </div>

        <div class="recent-list">
          {#each recentProjects as project (project.id)}
            <div
              class="recent-item"
              class:selected={selectedProject?.id === project.id}
              on:click={() => selectedProject = project}
              on:dblclick={() => openProject(project)}
              on:contextmenu={(e) => handleContextMenu(e, project)}
              role="button"
              tabindex="0"
            >
              <div class="recent-info">
                <div class="recent-name">{project.name}</div>
                <div class="recent-meta">
                  <span class="recent-time">{getRelativeTime(project.lastModified)}</span>
                  <span class="meta-separator">•</span>
                  <span class="recent-size">{formatSize(project.size)}</span>
                  <span class="meta-separator">•</span>
                  <span class="recent-tempo">{project.tempo} BPM</span>
                </div>
              </div>

              {#if project.favorite}
                <span class="favorite-star">★</span>
              {/if}
            </div>
          {/each}

          {#if recentProjects.length === 0}
            <div class="empty-state">
              <p>No recent projects</p>
            </div>
          {/if}
        </div>
      </div>

      <!-- All Projects Column -->
      <div class="all-column">
        <div class="section-header">
          <h3 class="section-title">All Projects</h3>
          <span class="section-count">{filteredProjects.length}</span>
        </div>

        {#if viewMode === 'grid'}
          <div class="projects-grid">
            {#each filteredProjects as project (project.id)}
              <div
                class="grid-item"
                class:selected={selectedProject?.id === project.id}
                on:click={() => selectedProject = project}
                on:dblclick={() => openProject(project)}
                on:contextmenu={(e) => handleContextMenu(e, project)}
                role="button"
                tabindex="0"
              >
                <div class="grid-thumbnail">
                  {#if project.thumbnail}
                    <img src={project.thumbnail} alt={project.name} />
                  {:else}
                    <div class="thumbnail-placeholder">
                      <span class="project-icon">🎵</span>
                    </div>
                  {/if}

                  {#if project.favorite}
                    <button
                      class="grid-favorite"
                      on:click|stopPropagation={() => toggleFavorite(project)}
                    >
                      ★
                    </button>
                  {/if}
                </div>

                <div class="grid-info">
                  <div class="grid-name" title={project.name}>{project.name}</div>
                  <div class="grid-meta">
                    {formatSize(project.size)} • {project.tracks} tracks
                  </div>
                  <div class="grid-date">{formatDate(project.lastModified)}</div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="projects-list">
            {#each filteredProjects as project (project.id)}
              <div
                class="list-item"
                class:selected={selectedProject?.id === project.id}
                on:click={() => selectedProject = project}
                on:dblclick={() => openProject(project)}
                on:contextmenu={(e) => handleContextMenu(e, project)}
                role="button"
                tabindex="0"
              >
                <button
                  class="list-favorite"
                  class:active={project.favorite}
                  on:click|stopPropagation={() => toggleFavorite(project)}
                >
                  ★
                </button>

                <div class="list-name">{project.name}</div>
                <div class="list-date">{formatDate(project.lastModified)}</div>
                <div class="list-size">{formatSize(project.size)}</div>
                <div class="list-tracks">{project.tracks} tracks</div>
                <div class="list-tempo">{project.tempo} BPM</div>
              </div>
            {/each}
          </div>
        {/if}

        {#if filteredProjects.length === 0}
          <div class="empty-state">
            <p>No projects found</p>
            <p class="hint">Try adjusting your search or filters</p>
          </div>
        {/if}
      </div>
    </div>

    <!-- Context Menu -->
    {#if showContextMenu && contextMenuProject}
      <div
        class="context-menu"
        style="left: {contextMenuX}px; top: {contextMenuY}px"
        on:click|stopPropagation
      >
        <button class="context-item" on:click={() => handleContextMenuAction('open')}>
          Open
        </button>
        <button class="context-item" on:click={() => handleContextMenuAction('rename')}>
          Rename
        </button>
        <div class="context-separator" />
        <button class="context-item" on:click={() => handleContextMenuAction('folder')}>
          Show in Folder
        </button>
        <button class="context-item" on:click={() => handleContextMenuAction('properties')}>
          Properties
        </button>
        <div class="context-separator" />
        <button class="context-item danger" on:click={() => handleContextMenuAction('delete')}>
          Delete
        </button>
      </div>
    {/if}

    <!-- Status Bar -->
    <div class="status-bar">
      <span class="status-item">{filteredProjects.length} projects</span>
      <span class="status-item">{formatSize(totalSize)} total</span>
      {#if selectedProject}
        <span class="status-item">Selected: {selectedProject.name}</span>
      {/if}
    </div>
  </div>
</WindowBase>

<style>
  .project-browser-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--window-bg, #1e1e1e);
  }

  /* Toolbar */
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: var(--toolbar-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .new-project-btn {
    padding: 8px 16px;
    background: var(--primary-color, #52b788);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    transition: all 0.15s ease;
  }

  .new-project-btn:hover {
    background: var(--primary-hover, #40916c);
  }

  .search-container {
    flex: 1;
  }

  .search-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--input-bg, #1a1a1a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    font-size: 13px;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-color, #0078d4);
  }

  .toolbar-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sort-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  .sort-select {
    padding: 6px 8px;
    background: var(--select-bg, #2a2a2a);
    color: var(--text-primary, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
  }

  .view-btn,
  .favorites-btn {
    width: 32px;
    height: 32px;
    background: var(--button-bg, #3a3a3a);
    color: var(--button-text, #e0e0e0);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
    transition: all 0.15s ease;
  }

  .view-btn:hover,
  .favorites-btn:hover {
    background: var(--button-hover, #4a4a4a);
  }

  .view-btn.active,
  .favorites-btn.active {
    background: var(--accent-color, #0078d4);
    border-color: var(--accent-color, #0078d4);
  }

  /* Main Content */
  .main-content {
    flex: 1;
    display: grid;
    grid-template-columns: 300px 1fr;
    overflow: hidden;
  }

  .recent-column,
  .all-column {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .recent-column {
    background: var(--recent-bg, #252525);
    border-right: 1px solid var(--border-color, #3e3e3e);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    background: var(--header-bg, #2d2d2d);
    border-bottom: 1px solid var(--border-color, #3e3e3e);
  }

  .section-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
  }

  .section-count {
    font-size: 12px;
    color: var(--text-secondary, #888);
  }

  /* Recent List */
  .recent-list {
    flex: 1;
    overflow-y: auto;
  }

  .recent-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .recent-item:hover {
    background: var(--item-hover, #2a2a2a);
  }

  .recent-item.selected {
    background: var(--item-selected, #3a3a3a);
  }

  .recent-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .recent-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .recent-meta {
    font-size: 11px;
    color: var(--text-secondary, #888);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .recent-time,
  .recent-size,
  .recent-tempo {
    font-variant-numeric: tabular-nums;
  }

  .meta-separator {
    opacity: 0.5;
  }

  .favorite-star {
    color: var(--star-active, #f7dc6f);
    font-size: 16px;
  }

  /* Grid View */
  .projects-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 16px;
    padding: 16px;
    overflow-y: auto;
  }

  .grid-item {
    display: flex;
    flex-direction: column;
    background: var(--grid-item-bg, #252525);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .grid-item:hover {
    background: var(--grid-item-hover, #2a2a2a);
    border-color: var(--accent-color, #0078d4);
    transform: translateY(-2px);
  }

  .grid-item.selected {
    border-color: var(--accent-color, #0078d4);
    background: var(--grid-item-selected, #2a2a2a);
  }

  .grid-thumbnail {
    position: relative;
    width: 100%;
    height: 120px;
    background: var(--thumbnail-bg, #1a1a1a);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .grid-thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .thumbnail-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .project-icon {
    font-size: 48px;
    opacity: 0.3;
  }

  .grid-favorite {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 28px;
    height: 28px;
    background: rgba(0, 0, 0, 0.7);
    color: var(--star-active, #f7dc6f);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
    transition: all 0.15s ease;
  }

  .grid-favorite:hover {
    background: rgba(0, 0, 0, 0.9);
    transform: scale(1.1);
  }

  .grid-info {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .grid-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .grid-meta,
  .grid-date {
    font-size: 11px;
    color: var(--text-secondary, #888);
  }

  /* List View */
  .projects-list {
    overflow-y: auto;
  }

  .list-item {
    display: grid;
    grid-template-columns: 40px 1fr 180px 100px 100px 100px;
    gap: 12px;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color, #3e3e3e);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .list-item:hover {
    background: var(--item-hover, #2a2a2a);
  }

  .list-item.selected {
    background: var(--item-selected, #3a3a3a);
  }

  .list-favorite {
    width: 28px;
    height: 28px;
    background: transparent;
    color: var(--star-inactive, #666);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 18px;
    transition: all 0.15s ease;
  }

  .list-favorite:hover {
    background: rgba(255, 255, 255, 0.05);
    color: var(--star-hover, #f7dc6f);
  }

  .list-favorite.active {
    color: var(--star-active, #f7dc6f);
  }

  .list-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary, #e0e0e0);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .list-date,
  .list-size,
  .list-tracks,
  .list-tempo {
    font-size: 12px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  /* Empty State */
  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    color: var(--text-secondary, #888);
  }

  .empty-state p {
    margin: 0 0 8px 0;
  }

  .hint {
    font-size: 13px;
    color: var(--text-tertiary, #666);
  }

  /* Context Menu */
  .context-menu {
    position: fixed;
    min-width: 160px;
    background: var(--context-bg, #2d2d2d);
    border: 1px solid var(--border-color, #3e3e3e);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    overflow: hidden;
  }

  .context-item {
    width: 100%;
    padding: 10px 16px;
    background: transparent;
    color: var(--text-primary, #e0e0e0);
    border: none;
    text-align: left;
    cursor: pointer;
    font-size: 13px;
    transition: background 0.15s ease;
  }

  .context-item:hover {
    background: var(--context-hover, #3a3a3a);
  }

  .context-item.danger {
    color: #e81123;
  }

  .context-item.danger:hover {
    background: #e81123;
    color: white;
  }

  .context-separator {
    height: 1px;
    background: var(--border-color, #3e3e3e);
    margin: 4px 0;
  }

  /* Status Bar */
  .status-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 6px 12px;
    background: var(--status-bg, #2d2d2d);
    border-top: 1px solid var(--border-color, #3e3e3e);
  }

  .status-item {
    font-size: 11px;
    color: var(--text-secondary, #888);
    font-variant-numeric: tabular-nums;
  }

  /* Scrollbars */
  .recent-list::-webkit-scrollbar,
  .projects-grid::-webkit-scrollbar,
  .projects-list::-webkit-scrollbar {
    width: 12px;
    height: 12px;
  }

  .recent-list::-webkit-scrollbar-track,
  .projects-grid::-webkit-scrollbar-track,
  .projects-list::-webkit-scrollbar-track {
    background: var(--scrollbar-track, #1e1e1e);
  }

  .recent-list::-webkit-scrollbar-thumb,
  .projects-grid::-webkit-scrollbar-thumb,
  .projects-list::-webkit-scrollbar-thumb {
    background: var(--scrollbar-thumb, #4e4e4e);
    border-radius: 6px;
  }
</style>
