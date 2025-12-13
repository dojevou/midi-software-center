<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { message, open, save } from '@tauri-apps/plugin-dialog';
  import FileBrowser from '$lib/components/FileBrowser.svelte';
  import type { Project } from '$lib/types';

  // FileItem interface matching FileBrowser component expectations
  interface FileItem {
    id: number;
    name: string;
    path: string;
    type: 'file' | 'folder';
    size?: number;
    modified?: string;
    duration?: number;
    bpm?: number;
    key?: string;
    tags?: string[];
  }

  // Convert Project[] to FileItem[] for FileBrowser compatibility
  function projectsToFileItems(projects: Project[]): FileItem[] {
    return projects.map((project) => ({
      id: project.id,
      name: project.name,
      path: project.file_path || '',
      type: 'file' as const,
      size: project.file_size,
      modified: project.modified_at,
      duration: project.duration,
      tags: project.tags,
    }));
  }

  interface NewProjectData {
    name: string;
    template: string;
    bpm: number;
    timeSignature: string;
    keySignature: string;
    sampleRate: number;
    bitDepth: number;
    location: string;
  }

  // State using Svelte 4 let bindings
  let projects: Project[] = [];
  let filteredProjects: Project[] = [];
  let selectedProject: Project | null = null;
  let viewMode: 'grid' | 'list' | 'details' = 'grid';
  let sortBy = 'modified';
  let sortDesc = true;
  let searchQuery = '';
  let isLoading = false;
  let showNewProjectDialog = false;
  let newProjectData: NewProjectData = {
    name: '',
    template: 'empty',
    bpm: 120,
    timeSignature: '4/4',
    keySignature: 'C',
    sampleRate: 44100,
    bitDepth: 24,
    location: '',
  };

  const templates = [
    { id: 'empty', name: 'Empty Project', description: 'Start from scratch' },
    { id: 'drum_pattern', name: 'Drum Pattern', description: 'Pre-configured drum tracks' },
    { id: 'piano_trio', name: 'Piano Trio', description: 'Piano, bass, drums' },
    { id: 'electronic', name: 'Electronic', description: 'Synth and drum machine setup' },
    { id: 'orchestral', name: 'Orchestral', description: 'Orchestra section templates' },
    { id: 'jazz_combo', name: 'Jazz Combo', description: 'Jazz quartet setup' },
    { id: 'film_score', name: 'Film Score', description: 'Cinematic template' },
    { id: 'pop_song', name: 'Pop Song', description: 'Modern pop arrangement' },
  ];

  const timeSignatures = [
    '1/4',
    '2/4',
    '3/4',
    '4/4',
    '5/4',
    '6/4',
    '7/4',
    '8/4',
    '3/8',
    '6/8',
    '9/8',
    '12/8',
    '7/8',
    '5/8',
    '11/8',
  ];

  const keySignatures = [
    'C',
    'G',
    'D',
    'A',
    'E',
    'B',
    'F#',
    'C#',
    'F',
    'Bb',
    'Eb',
    'Ab',
    'Db',
    'Gb',
    'Cb',
    'Am',
    'Em',
    'Bm',
    'F#m',
    'C#m',
    'G#m',
    'D#m',
    'A#m',
    'Dm',
    'Gm',
    'Cm',
    'Fm',
    'Bbm',
    'Ebm',
    'Abm',
  ];

  onMount(async () => {
    await loadProjects();
  });

  async function loadProjects() {
    isLoading = true;
    try {
      projects = await api.project.getProjects();
      filterAndSortProjects();
    } catch (error) {
      console.error('Failed to load projects:', error);
      await message(`Failed to load projects: ${error}`, { title: 'Error', kind: 'error' });
    } finally {
      isLoading = false;
    }
  }

  function filterAndSortProjects() {
    const filtered = projects.filter((project) => {
      const matchesSearch =
        searchQuery === '' ||
        project.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        project.description?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        project.tags?.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()));

      return matchesSearch;
    });

    filtered.sort((a, b) => {
      let valA: string | number;
      let valB: string | number;
      switch (sortBy) {
        case 'name':
          valA = a.name.toLowerCase();
          valB = b.name.toLowerCase();
          break;
        case 'created':
          valA = new Date(a.created_at).getTime();
          valB = new Date(b.created_at).getTime();
          break;
        case 'modified':
          valA = new Date(a.modified_at).getTime();
          valB = new Date(b.modified_at).getTime();
          break;
        case 'size':
          valA = a.file_size || 0;
          valB = b.file_size || 0;
          break;
        case 'tracks':
          valA = a.track_count || 0;
          valB = b.track_count || 0;
          break;
        default:
          // Type-safe fallback for dynamic property access
          valA = ((a as unknown as Record<string, unknown>)[sortBy] as string | number) || '';
          valB = ((b as unknown as Record<string, unknown>)[sortBy] as string | number) || '';
      }

      return sortDesc ? (valA > valB ? -1 : 1) : valA < valB ? -1 : 1;
    });

    filteredProjects = filtered;
  }

  // Reactive statement to re-filter when search or sort changes
  $: {
    searchQuery;
    sortBy;
    sortDesc;
    filterAndSortProjects();
  }

  async function openProject(projectId: number) {
    try {
      await api.project.open(projectId);
      await message('Project loaded successfully', { title: 'Success', kind: 'info' });
    } catch (error) {
      console.error('Failed to open project:', error);
      await message(`Failed to open project: ${error}`, { title: 'Error', kind: 'error' });
    }
  }

  async function createProject() {
    if (!newProjectData.name.trim()) {
      await message('Please enter a project name', { title: 'Error', kind: 'error' });
      return;
    }

    try {
      const created = await api.project.create(newProjectData);
      projects = [...projects, created];
      showNewProjectDialog = false;
      newProjectData = {
        name: '',
        template: 'empty',
        bpm: 120,
        timeSignature: '4/4',
        keySignature: 'C',
        sampleRate: 44100,
        bitDepth: 24,
        location: '',
      };

      await message('Project created successfully', { title: 'Success', kind: 'info' });
    } catch (error) {
      console.error('Failed to create project:', error);
      await message(`Failed to create project: ${error}`, { title: 'Error', kind: 'error' });
    }
  }

  async function deleteProject(projectId: number) {
    if (!confirm('Delete this project? This action cannot be undone.')) {
      return;
    }

    try {
      await api.project.delete(projectId);
      projects = projects.filter((p) => p.id !== projectId);
      if (selectedProject?.id === projectId) {
        selectedProject = null;
      }
    } catch (error) {
      console.error('Failed to delete project:', error);
      await message(`Failed to delete project: ${error}`, { title: 'Error', kind: 'error' });
    }
  }

  async function duplicateProject(projectId: number) {
    try {
      const duplicated = await api.project.duplicate(projectId);
      projects = [...projects, duplicated];
    } catch (error) {
      console.error('Failed to duplicate project:', error);
      await message(`Failed to duplicate project: ${error}`, { title: 'Error', kind: 'error' });
    }
  }

  async function exportProject(projectId: number) {
    try {
      const project = projects.find((p) => p.id === projectId);
      const exportPath = await save({
        filters: [
          {
            name: 'Project Archive',
            extensions: ['zip'],
          },
        ],
        defaultPath: `project_${project?.name || projectId}_${new Date().toISOString().split('T')[0]}.zip`,
      });

      if (exportPath) {
        await api.project.exportArchive(projectId, exportPath);
        await message('Project exported successfully', { title: 'Success', kind: 'info' });
      }
    } catch (error) {
      console.error('Failed to export project:', error);
      await message(`Failed to export project: ${error}`, { title: 'Error', kind: 'error' });
    }
  }

  async function importProject() {
    const selected = await open({
      filters: [
        {
          name: 'Project Files',
          extensions: ['zip', 'mid', 'midi'],
        },
      ],
      multiple: false,
    });

    if (selected) {
      try {
        const imported = await api.project.import(selected);
        projects = [...projects, imported];
        await message('Project imported successfully', { title: 'Success', kind: 'info' });
      } catch (error) {
        console.error('Failed to import project:', error);
        await message(`Failed to import project: ${error}`, { title: 'Error', kind: 'error' });
      }
    }
  }

  async function selectProjectLocation() {
    const selected = await open({
      directory: true,
      multiple: false,
    });

    if (selected) {
      newProjectData.location = selected;
    }
  }

  function formatFileSize(bytes: number | undefined): string {
    if (!bytes) {
      return '0 B';
    }

    const units = ['B', 'KB', 'MB', 'GB'];
    let size = bytes;
    let unitIndex = 0;

    while (size >= 1024 && unitIndex < units.length - 1) {
      size /= 1024;
      unitIndex++;
    }

    return `${size.toFixed(1)} ${units[unitIndex]}`;
  }

  function formatDate(dateString: string | undefined, full = false): string {
    if (!dateString) {
      return 'Unknown';
    }

    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (!full) {
      if (diffDays === 0) {
        return 'Today';
      }
      if (diffDays === 1) {
        return 'Yesterday';
      }
      if (diffDays < 7) {
        return `${diffDays} days ago`;
      }
      if (diffDays < 30) {
        return `${Math.floor(diffDays / 7)} weeks ago`;
      }
      return date.toLocaleDateString();
    } else {
      return date.toLocaleString();
    }
  }

  function formatDuration(seconds: number | undefined): string {
    if (!seconds) {
      return '0:00';
    }
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<div class="project-browser-window dark:bg-window dark:text-app-text h-full flex flex-col">
  <!-- Header -->
  <div class="header p-6 border-b dark:border-window-border">
    <div class="flex justify-between items-center mb-4">
      <h2 class="text-2xl dark:text-gray-200">Project Browser</h2>

      <div class="flex gap-3">
        <button
          on:click={importProject}
          class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80 flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M9 12h2v-2h2v2h2V8h2l-4-4-4 4h2v4zM5 14h10v2H5v-2z" />
          </svg>
          Import
        </button>

        <button
          on:click={() => (showNewProjectDialog = true)}
          class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80 flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path d="M11 9h4v2h-4v4H9v-4H5V9h4V5h2v4z" />
          </svg>
          New Project
        </button>
      </div>
    </div>

    <!-- Search & Filter -->
    <div class="search-filters">
      <div class="flex gap-4">
        <div class="search flex-1">
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search projects..."
            class="w-full px-4 py-2 dark:bg-input dark:border-window-border rounded"
          />
        </div>

        <div class="sort-controls flex items-center gap-2">
          <span class="text-sm dark:text-gray-400">Sort by:</span>
          <select
            bind:value={sortBy}
            class="px-3 py-2 dark:bg-input dark:border-window-border rounded"
          >
            <option value="name">Name</option>
            <option value="created">Date Created</option>
            <option value="modified">Last Modified</option>
            <option value="size">File Size</option>
            <option value="tracks">Track Count</option>
          </select>
          <button
            on:click={() => (sortDesc = !sortDesc)}
            class="px-3 py-2 dark:bg-secondary rounded"
          >
            {sortDesc ? '↓' : '↑'}
          </button>
        </div>

        <div class="view-toggle flex border dark:border-window-border rounded overflow-hidden">
          <button
            on:click={() => (viewMode = 'grid')}
            class="px-3 py-2"
            class:dark:bg-secondary={viewMode === 'grid'}
            title="Grid View"
          >
            ⏹
          </button>
          <button
            on:click={() => (viewMode = 'list')}
            class="px-3 py-2"
            class:dark:bg-secondary={viewMode === 'list'}
            title="List View"
          >
            ≡
          </button>
          <button
            on:click={() => (viewMode = 'details')}
            class="px-3 py-2"
            class:dark:bg-secondary={viewMode === 'details'}
            title="Details View"
          >
            ⧉
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- Projects Content -->
  <div class="content flex-1 overflow-auto p-6">
    {#if isLoading}
      <div class="loading dark:text-gray-500 text-center py-12">
        <div
          class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 dark:border-gray-400 mb-2"
        ></div>
        <div>Loading projects...</div>
      </div>
    {:else if projects.length === 0}
      <div class="empty-state text-center py-12">
        <div class="text-6xl mb-4 dark:text-gray-600">📁</div>
        <h3 class="text-xl dark:text-gray-300 mb-2">No projects yet</h3>
        <p class="dark:text-gray-500 mb-6">Create your first project to get started.</p>
        <button
          on:click={() => (showNewProjectDialog = true)}
          class="px-6 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
        >
          Create New Project
        </button>
      </div>
    {:else if viewMode === 'grid'}
      <!-- Grid View -->
      <div
        class="projects-grid grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6"
      >
        {#each filteredProjects as project (project.id)}
          <div
            class="project-card dark:bg-window-subtle rounded-lg border dark:border-window-border overflow-hidden hover:shadow-lg transition-shadow cursor-pointer"
            class:dark:border-primary={selectedProject?.id === project.id}
            on:click={() => (selectedProject = project)}
            on:dblclick={() => openProject(project.id)}
            on:keydown={(e) => e.key === 'Enter' && openProject(project.id)}
            role="button"
            tabindex="0"
          >
            <!-- Project Thumbnail -->
            <div class="aspect-video dark:bg-menu relative group">
              <div class="absolute inset-0 flex items-center justify-center">
                <div class="text-4xl dark:text-gray-600">🎵</div>
              </div>

              <!-- Quick Actions Overlay -->
              <div
                class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-40 transition-all flex items-center justify-center opacity-0 group-hover:opacity-100"
              >
                <div class="flex gap-2">
                  <button
                    on:click|stopPropagation={() => openProject(project.id)}
                    class="p-3 dark:bg-primary rounded-full hover:scale-110 transition-transform"
                    title="Open"
                  >
                    ▶
                  </button>
                  <button
                    on:click|stopPropagation={() => duplicateProject(project.id)}
                    class="p-3 dark:bg-secondary rounded-full hover:scale-110 transition-transform"
                    title="Duplicate"
                  >
                    ⎘
                  </button>
                  <button
                    on:click|stopPropagation={() => exportProject(project.id)}
                    class="p-3 dark:bg-secondary rounded-full hover:scale-110 transition-transform"
                    title="Export"
                  >
                    ⤓
                  </button>
                </div>
              </div>

              <!-- Project Info Badge -->
              <div class="absolute top-2 right-2">
                <div class="flex gap-1">
                  <span class="px-2 py-1 text-xs dark:bg-menu rounded-full dark:text-gray-400">
                    {project.track_count || 0} trk
                  </span>
                  <span class="px-2 py-1 text-xs dark:bg-menu rounded-full dark:text-gray-400">
                    {formatFileSize(project.file_size)}
                  </span>
                </div>
              </div>
            </div>

            <!-- Project Details -->
            <div class="p-4">
              <h3 class="font-medium dark:text-gray-200 truncate mb-1" title={project.name}>
                {project.name}
              </h3>

              {#if project.description}
                <p class="text-sm dark:text-gray-400 mb-3 line-clamp-2">
                  {project.description}
                </p>
              {/if}

              <div class="text-xs dark:text-gray-500 space-y-1">
                <div class="flex justify-between">
                  <span>Modified:</span>
                  <span class="dark:text-gray-300">{formatDate(project.modified_at)}</span>
                </div>
                <div class="flex justify-between">
                  <span>Created:</span>
                  <span class="dark:text-gray-300">{formatDate(project.created_at)}</span>
                </div>
              </div>

              <!-- Tags -->
              {#if project.tags && project.tags.length > 0}
                <div class="tags mt-3 flex flex-wrap gap-1">
                  {#each project.tags.slice(0, 3) as tag (tag)}
                    <span class="text-xs px-2 py-0.5 dark:bg-menu rounded-full dark:text-gray-400">
                      {tag}
                    </span>
                  {/each}
                  {#if project.tags.length > 3}
                    <span class="text-xs px-2 py-0.5 dark:text-gray-500">
                      +{project.tags.length - 3}
                    </span>
                  {/if}
                </div>
              {/if}

              <!-- Actions -->
              <div class="actions mt-4 flex justify-end gap-1">
                <button
                  on:click|stopPropagation={() => openProject(project.id)}
                  class="px-3 py-1 text-xs dark:bg-primary dark:text-white rounded hover:opacity-80"
                >
                  Open
                </button>
                <button
                  on:click|stopPropagation={() => deleteProject(project.id)}
                  class="px-3 py-1 text-xs dark:bg-error rounded hover:opacity-80"
                >
                  Delete
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else if viewMode === 'list'}
      <!-- List View -->
      <div class="projects-list">
        <table class="w-full">
          <thead>
            <tr class="border-b dark:border-window-border">
              <th class="p-3 text-left dark:text-gray-400">Name</th>
              <th class="p-3 text-left dark:text-gray-400">Tracks</th>
              <th class="p-3 text-left dark:text-gray-400">Size</th>
              <th class="p-3 text-left dark:text-gray-400">Modified</th>
              <th class="p-3 text-left dark:text-gray-400">Created</th>
              <th class="p-3 text-left dark:text-gray-400">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each filteredProjects as project (project.id)}
              <tr
                class="border-b dark:border-window-border hover:dark:bg-window-subtle cursor-pointer"
                class:dark:bg-window-subtle={selectedProject?.id === project.id}
                on:click={() => (selectedProject = project)}
                on:dblclick={() => openProject(project.id)}
                on:keydown={(e) => e.key === 'Enter' && openProject(project.id)}
                tabindex="0"
              >
                <td class="p-3">
                  <div class="flex items-center gap-3">
                    <div class="w-8 h-8 dark:bg-menu rounded flex items-center justify-center">
                      <span class="text-sm">🎵</span>
                    </div>
                    <div>
                      <div class="font-medium dark:text-gray-200">{project.name}</div>
                      {#if project.description}
                        <div class="text-xs dark:text-gray-400 truncate max-w-xs">
                          {project.description}
                        </div>
                      {/if}
                    </div>
                  </div>
                </td>
                <td class="p-3 dark:text-gray-300">
                  {project.track_count || 0}
                </td>
                <td class="p-3 dark:text-gray-300">
                  {formatFileSize(project.file_size)}
                </td>
                <td class="p-3 dark:text-gray-300">
                  {formatDate(project.modified_at)}
                </td>
                <td class="p-3 dark:text-gray-300">
                  {formatDate(project.created_at)}
                </td>
                <td class="p-3">
                  <div class="flex gap-1">
                    <button
                      on:click|stopPropagation={() => openProject(project.id)}
                      class="px-2 py-1 text-xs dark:bg-primary dark:text-white rounded hover:opacity-80"
                    >
                      Open
                    </button>
                    <button
                      on:click|stopPropagation={() => duplicateProject(project.id)}
                      class="px-2 py-1 text-xs dark:bg-secondary rounded hover:opacity-80"
                    >
                      Copy
                    </button>
                    <button
                      on:click|stopPropagation={() => deleteProject(project.id)}
                      class="px-2 py-1 text-xs dark:bg-error rounded hover:opacity-80"
                    >
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <!-- Details View -->
      <FileBrowser
        files={projectsToFileItems(filteredProjects)}
        on:open={(e) => openProject(e.detail)}
        on:delete={(e) => deleteProject(e.detail)}
        on:duplicate={(e) => duplicateProject(e.detail)}
        on:export={(e) => exportProject(e.detail)}
      />
    {/if}
  </div>

  <!-- Selected Project Details -->
  {#if selectedProject}
    <div class="selected-details border-t dark:border-window-border p-6">
      <div class="max-w-4xl mx-auto">
        <div class="flex justify-between items-start mb-6">
          <div>
            <h3 class="text-xl dark:text-gray-200 mb-2">{selectedProject.name}</h3>
            <p class="dark:text-gray-400">{selectedProject.description || 'No description'}</p>
          </div>
          <div class="flex gap-2">
            <button
              on:click={() => openProject(selectedProject?.id || 0)}
              class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
            >
              Open Project
            </button>
            <button
              on:click={() => exportProject(selectedProject?.id || 0)}
              class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
            >
              Export
            </button>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-6">
          <div class="space-y-4">
            <div>
              <h4 class="text-sm dark:text-gray-400 mb-2">File Information</h4>
              <div class="space-y-1 text-sm">
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Size:</span>
                  <span class="dark:text-gray-300">{formatFileSize(selectedProject.file_size)}</span
                  >
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Location:</span>
                  <span
                    class="dark:text-gray-300 truncate max-w-xs"
                    title={selectedProject.file_path}
                  >
                    {selectedProject.file_path || 'Unknown'}
                  </span>
                </div>
                <div class="flex justify-between">
                  <span class="dark:text-gray-400">Format:</span>
                  <span class="dark:text-gray-300">{selectedProject.format || 'MIDI Project'}</span>
                </div>
              </div>
            </div>
          </div>

          <div class="space-y-4">
            <h4 class="text-sm dark:text-gray-400 mb-2">Project Statistics</h4>
            <div class="space-y-1 text-sm">
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Tracks:</span>
                <span class="dark:text-gray-300">{selectedProject.track_count || 0}</span>
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Events:</span>
                <span class="dark:text-gray-300">{selectedProject.event_count || 0}</span>
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Notes:</span>
                <span class="dark:text-gray-300">{selectedProject.note_count || 0}</span>
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Duration:</span>
                <span class="dark:text-gray-300">
                  {selectedProject.duration ? formatDuration(selectedProject.duration) : 'Unknown'}
                </span>
              </div>
            </div>
          </div>

          <div class="space-y-4">
            <h4 class="text-sm dark:text-gray-400 mb-2">Timestamps</h4>
            <div class="space-y-1 text-sm">
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Created:</span>
                <span class="dark:text-gray-300"
                  >{formatDate(selectedProject.created_at, true)}</span
                >
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Modified:</span>
                <span class="dark:text-gray-300"
                  >{formatDate(selectedProject.modified_at, true)}</span
                >
              </div>
              <div class="flex justify-between">
                <span class="dark:text-gray-400">Last Opened:</span>
                <span class="dark:text-gray-300">
                  {selectedProject.last_opened
                    ? formatDate(selectedProject.last_opened, true)
                    : 'Never'}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Project Tags -->
        {#if selectedProject.tags && selectedProject.tags.length > 0}
          <div class="mt-6">
            <h4 class="text-sm dark:text-gray-400 mb-2">Tags</h4>
            <div class="flex flex-wrap gap-2">
              {#each selectedProject.tags as tag (tag)}
                <span class="px-3 py-1 dark:bg-secondary rounded-full text-sm dark:text-gray-300">
                  {tag}
                </span>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- New Project Dialog -->
{#if showNewProjectDialog}
  <div
    class="modal-overlay fixed inset-0 dark:bg-black dark:bg-opacity-50 flex items-center justify-center z-50"
  >
    <div class="modal dark:bg-window p-6 rounded-lg w-[600px] max-h-[90vh] overflow-auto">
      <h3 class="text-xl dark:text-gray-200 mb-6">Create New Project</h3>

      <form on:submit|preventDefault={createProject}>
        <div class="space-y-6">
          <!-- Basic Info -->
          <div>
            <label class="block text-sm dark:text-gray-400 mb-2">Project Name</label>
            <input
              type="text"
              bind:value={newProjectData.name}
              class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
              placeholder="My Awesome Project"
              required
            />
          </div>

          <!-- Template Selection -->
          <div>
            <label class="block text-sm dark:text-gray-400 mb-2">Template</label>
            <div class="grid grid-cols-2 gap-3">
              {#each templates as template (template.id)}
                <label
                  class="template-option flex flex-col p-3 border dark:border-window-border rounded cursor-pointer hover:dark:bg-window-subtle transition-colors"
                >
                  <div class="flex items-center gap-2 mb-1">
                    <input
                      type="radio"
                      bind:group={newProjectData.template}
                      value={template.id}
                      name="template"
                      class="rounded"
                    />
                    <span class="font-medium dark:text-gray-200">{template.name}</span>
                  </div>
                  <span class="text-xs dark:text-gray-400">{template.description}</span>
                </label>
              {/each}
            </div>
          </div>

          <!-- Project Settings -->
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm dark:text-gray-400 mb-2">BPM</label>
              <input
                type="number"
                bind:value={newProjectData.bpm}
                min="30"
                max="300"
                step="1"
                class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
              />
            </div>

            <div>
              <label class="block text-sm dark:text-gray-400 mb-2">Time Signature</label>
              <select
                bind:value={newProjectData.timeSignature}
                class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
              >
                {#each timeSignatures as ts (ts)}
                  <option value={ts}>{ts}</option>
                {/each}
              </select>
            </div>

            <div>
              <label class="block text-sm dark:text-gray-400 mb-2">Key Signature</label>
              <select
                bind:value={newProjectData.keySignature}
                class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
              >
                {#each keySignatures as key (key)}
                  <option value={key}>{key}</option>
                {/each}
              </select>
            </div>

            <div>
              <label class="block text-sm dark:text-gray-400 mb-2">Sample Rate</label>
              <select
                bind:value={newProjectData.sampleRate}
                class="w-full px-3 py-2 dark:bg-input dark:border-window-border rounded"
              >
                <option value={44100}>44.1 kHz</option>
                <option value={48000}>48 kHz</option>
                <option value={88200}>88.2 kHz</option>
                <option value={96000}>96 kHz</option>
              </select>
            </div>
          </div>

          <!-- Location -->
          <div>
            <label class="block text-sm dark:text-gray-400 mb-2">Project Location</label>
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={newProjectData.location}
                readonly
                class="flex-1 px-3 py-2 dark:bg-input dark:border-window-border rounded"
                placeholder="Default location will be used"
              />
              <button
                type="button"
                on:click={selectProjectLocation}
                class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
              >
                Browse
              </button>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex gap-3 justify-end">
            <button
              type="button"
              on:click={() => (showNewProjectDialog = false)}
              class="px-4 py-2 dark:bg-secondary rounded hover:opacity-80"
            >
              Cancel
            </button>
            <button
              type="submit"
              class="px-4 py-2 dark:bg-primary dark:text-white rounded hover:opacity-80"
            >
              Create Project
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .project-card {
    transition:
      transform 0.2s,
      box-shadow 0.2s;
  }

  .project-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.2);
  }

  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .template-option:hover {
    border-color: #3b82f6;
  }

  .modal-overlay {
    backdrop-filter: blur(4px);
  }
</style>
