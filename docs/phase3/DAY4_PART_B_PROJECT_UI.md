# Day 4, Part 4B: Project Management UI

**Duration:** 2 hours
**Prerequisites:** Part 4A complete (project models)
**Files to create:** 2

---

## Overview

Build project management UI:
1. TypeScript project types
2. Project API client
3. ProjectManager component
4. New/open/save/save-as dialogs
5. Recent projects list

---

## Step 1: TypeScript Types (15 min)

Create `app/src/lib/types/project.ts`:

```typescript
export interface Project {
  id: string;
  name: string;
  file_path?: string;
  bpm: number;
  time_signature: [number, number];
  sample_rate: number;
  session_data: string;
  created_at: string;
  updated_at: string;
  last_opened_at?: string;
}

export interface RecentProject {
  id: string;
  name: string;
  file_path?: string;
  last_opened_at: string;
}

export interface SessionState {
  project_name: string;
  bpm: number;
  time_signature: [number, number];
  sample_rate: number;
  mixer: any;
  routing: any;
  automation_lanes: any[];
  playhead_position: number;
  loop_enabled: boolean;
  loop_start_ticks: number;
  loop_end_ticks: number;
  midi_files: Record<number, string>;
}

export class ProjectUtils {
  static formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString();
  }

  static getProjectIcon(project: Project | RecentProject): string {
    return '📁';
  }
}
```

---

## Step 2: Project API (15 min)

Create `app/src/lib/api/projectApi.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { Project, RecentProject } from '../types/project';

export class ProjectApi {
  static async create(name: string): Promise<Project> {
    return await invoke<Project>('create_project', { name });
  }

  static async save(projectId: string): Promise<void> {
    await invoke('save_project', { projectId });
  }

  static async load(projectId: string): Promise<Project> {
    return await invoke<Project>('load_project', { projectId });
  }

  static async delete(projectId: string): Promise<void> {
    await invoke('delete_project', { projectId });
  }

  static async listAll(): Promise<Project[]> {
    return await invoke<Project[]>('list_projects');
  }

  static async getRecent(limit: number = 10): Promise<RecentProject[]> {
    return await invoke<RecentProject[]>('get_recent_projects', { limit });
  }
}
```

---

## Step 3: Project Manager Component (1 hour 15 min)

Create `app/src/lib/components/DAW/ProjectManager.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { ProjectApi } from '../../api/projectApi';
  import { ProjectUtils } from '../../types/project';
  import type { Project, RecentProject } from '../../types/project';

  export let currentProject: Project | null = null;
  export let onProjectChange: ((project: Project | null) => void) | null = null;

  let showNewDialog = false;
  let showOpenDialog = false;
  let showSaveAsDialog = false;

  let recentProjects: RecentProject[] = [];
  let allProjects: Project[] = [];

  // New project form
  let newProjectName = '';
  let newProjectBpm = 120;

  // Save As form
  let saveAsName = '';

  // Unsaved changes flag
  let hasUnsavedChanges = false;

  onMount(async () => {
    await loadRecentProjects();
  });

  async function loadRecentProjects() {
    try {
      recentProjects = await ProjectApi.getRecent(10);
    } catch (error) {
      console.error('Failed to load recent projects:', error);
    }
  }

  async function loadAllProjects() {
    try {
      allProjects = await ProjectApi.listAll();
    } catch (error) {
      console.error('Failed to load all projects:', error);
    }
  }

  async function handleNewProject() {
    if (!newProjectName.trim()) {
      alert('Please enter a project name');
      return;
    }

    if (hasUnsavedChanges) {
      if (!confirm('You have unsaved changes. Continue?')) {
        return;
      }
    }

    try {
      const project = await ProjectApi.create(newProjectName);
      currentProject = project;
      hasUnsavedChanges = false;
      showNewDialog = false;
      newProjectName = '';

      if (onProjectChange) {
        onProjectChange(project);
      }

      await loadRecentProjects();
    } catch (error) {
      console.error('Failed to create project:', error);
      alert('Failed to create project');
    }
  }

  async function handleOpenProject(projectId: string) {
    if (hasUnsavedChanges) {
      if (!confirm('You have unsaved changes. Continue?')) {
        return;
      }
    }

    try {
      const project = await ProjectApi.load(projectId);
      currentProject = project;
      hasUnsavedChanges = false;
      showOpenDialog = false;

      if (onProjectChange) {
        onProjectChange(project);
      }

      await loadRecentProjects();
    } catch (error) {
      console.error('Failed to open project:', error);
      alert('Failed to open project');
    }
  }

  async function handleSaveProject() {
    if (!currentProject) {
      // No current project, trigger Save As
      handleOpenSaveAsDialog();
      return;
    }

    try {
      await ProjectApi.save(currentProject.id);
      hasUnsavedChanges = false;
      alert('Project saved successfully');
    } catch (error) {
      console.error('Failed to save project:', error);
      alert('Failed to save project');
    }
  }

  async function handleSaveAs() {
    if (!saveAsName.trim()) {
      alert('Please enter a project name');
      return;
    }

    try {
      // Create new project with new name
      const newProject = await ProjectApi.create(saveAsName);

      // Save current state to new project
      await ProjectApi.save(newProject.id);

      currentProject = newProject;
      hasUnsavedChanges = false;
      showSaveAsDialog = false;
      saveAsName = '';

      if (onProjectChange) {
        onProjectChange(newProject);
      }

      await loadRecentProjects();
    } catch (error) {
      console.error('Failed to save as:', error);
      alert('Failed to save as');
    }
  }

  async function handleCloseProject() {
    if (hasUnsavedChanges) {
      const response = confirm('Save changes before closing?');
      if (response && currentProject) {
        await handleSaveProject();
      }
    }

    currentProject = null;
    hasUnsavedChanges = false;

    if (onProjectChange) {
      onProjectChange(null);
    }
  }

  async function handleDeleteProject(projectId: string, projectName: string) {
    if (!confirm(`Delete project "${projectName}"? This cannot be undone.`)) {
      return;
    }

    try {
      await ProjectApi.delete(projectId);

      if (currentProject?.id === projectId) {
        currentProject = null;
        hasUnsavedChanges = false;

        if (onProjectChange) {
          onProjectChange(null);
        }
      }

      await loadRecentProjects();
      await loadAllProjects();
    } catch (error) {
      console.error('Failed to delete project:', error);
      alert('Failed to delete project');
    }
  }

  function handleOpenNewDialog() {
    showNewDialog = true;
    newProjectName = '';
    newProjectBpm = 120;
  }

  async function handleOpenOpenDialog() {
    showOpenDialog = true;
    await loadAllProjects();
  }

  function handleOpenSaveAsDialog() {
    showSaveAsDialog = true;
    saveAsName = currentProject?.name || '';
  }

  // Mark as changed when user makes edits
  export function markAsChanged() {
    hasUnsavedChanges = true;
  }
</script>

<div class="project-manager">
  <div class="toolbar">
    <div class="file-menu">
      <button class="menu-btn" on:click={handleOpenNewDialog}> 📄 New </button>

      <button class="menu-btn" on:click={handleOpenOpenDialog}> 📂 Open </button>

      <button class="menu-btn" on:click={handleSaveProject} disabled={!currentProject}>
        💾 Save
      </button>

      <button class="menu-btn" on:click={handleOpenSaveAsDialog}> 💾 Save As </button>

      <button class="menu-btn" on:click={handleCloseProject} disabled={!currentProject}>
        ✕ Close
      </button>
    </div>

    <div class="project-info">
      {#if currentProject}
        <span class="project-name">{currentProject.name}</span>
        {#if hasUnsavedChanges}
          <span class="unsaved-indicator" title="Unsaved changes">●</span>
        {/if}
      {:else}
        <span class="no-project">No project open</span>
      {/if}
    </div>
  </div>

  {#if !currentProject && recentProjects.length > 0}
    <div class="recent-projects">
      <h4>Recent Projects</h4>

      <div class="project-list">
        {#each recentProjects as project (project.id)}
          <button class="recent-project-item" on:click={() => handleOpenProject(project.id)}>
            <div class="project-icon">{ProjectUtils.getProjectIcon(project)}</div>
            <div class="project-details">
              <div class="project-name">{project.name}</div>
              <div class="project-time">
                {ProjectUtils.formatDate(project.last_opened_at)}
              </div>
            </div>
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- New Project Dialog -->
{#if showNewDialog}
  <div class="modal-overlay" on:click={() => (showNewDialog = false)}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3>New Project</h3>
        <button class="close-btn" on:click={() => (showNewDialog = false)}>✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="new-project-name">Project Name *</label>
          <input
            id="new-project-name"
            type="text"
            bind:value={newProjectName}
            placeholder="My Awesome Project"
            class="form-input"
          />
        </div>

        <div class="form-group">
          <label for="new-project-bpm">BPM</label>
          <input
            id="new-project-bpm"
            type="number"
            bind:value={newProjectBpm}
            min="60"
            max="240"
            class="form-input"
          />
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" on:click={() => (showNewDialog = false)}> Cancel </button>
        <button class="confirm-btn" on:click={handleNewProject} disabled={!newProjectName.trim()}>
          Create Project
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Open Project Dialog -->
{#if showOpenDialog}
  <div class="modal-overlay" on:click={() => (showOpenDialog = false)}>
    <div class="modal-content large" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Open Project</h3>
        <button class="close-btn" on:click={() => (showOpenDialog = false)}>✕</button>
      </div>

      <div class="modal-body">
        {#if allProjects.length === 0}
          <div class="empty-state">
            <p>No projects found.</p>
            <p>Create a new project to get started.</p>
          </div>
        {:else}
          <div class="project-grid">
            {#each allProjects as project (project.id)}
              <div class="project-card">
                <div class="card-header">
                  <div class="project-icon">{ProjectUtils.getProjectIcon(project)}</div>
                  <button
                    class="delete-btn"
                    on:click|stopPropagation={() => handleDeleteProject(project.id, project.name)}
                    title="Delete project"
                  >
                    ✕
                  </button>
                </div>

                <div class="card-body" on:click={() => handleOpenProject(project.id)}>
                  <div class="project-name">{project.name}</div>
                  <div class="project-meta">
                    <span>{project.bpm} BPM</span>
                    <span>•</span>
                    <span>{project.time_signature[0]}/{project.time_signature[1]}</span>
                  </div>
                  <div class="project-time">
                    Updated {ProjectUtils.formatDate(project.updated_at)}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<!-- Save As Dialog -->
{#if showSaveAsDialog}
  <div class="modal-overlay" on:click={() => (showSaveAsDialog = false)}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3>Save As</h3>
        <button class="close-btn" on:click={() => (showSaveAsDialog = false)}>✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label for="save-as-name">Project Name *</label>
          <input
            id="save-as-name"
            type="text"
            bind:value={saveAsName}
            placeholder="Project Copy"
            class="form-input"
          />
        </div>
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" on:click={() => (showSaveAsDialog = false)}> Cancel </button>
        <button class="confirm-btn" on:click={handleSaveAs} disabled={!saveAsName.trim()}>
          Save As
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .project-manager {
    background: #1a1a1a;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #161616;
    border-bottom: 1px solid #333;
  }

  .file-menu {
    display: flex;
    gap: 4px;
  }

  .menu-btn {
    padding: 6px 12px;
    background: #252525;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
    transition: background 0.2s;
  }

  .menu-btn:hover:not(:disabled) {
    background: #2a2a2a;
    border-color: #444;
  }

  .menu-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .project-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .project-name {
    font-size: 14px;
    font-weight: 600;
    color: #fff;
  }

  .unsaved-indicator {
    color: #f59e0b;
    font-size: 20px;
  }

  .no-project {
    font-size: 13px;
    color: #666;
  }

  .recent-projects {
    padding: 24px;
  }

  .recent-projects h4 {
    margin: 0 0 16px 0;
    font-size: 14px;
    color: #999;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .project-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .recent-project-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    background: #252525;
    border: 1px solid #333;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    text-align: left;
    width: 100%;
  }

  .recent-project-item:hover {
    background: #2a2a2a;
    border-color: #3b82f6;
  }

  .project-icon {
    font-size: 32px;
  }

  .project-details {
    flex: 1;
  }

  .project-time {
    font-size: 12px;
    color: #666;
    margin-top: 2px;
  }

  /* Modal styles */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .modal-content.large {
    max-width: 800px;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px;
    background: #161616;
    border-bottom: 1px solid #333;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .close-btn {
    background: none;
    color: #999;
    border: none;
    font-size: 20px;
    cursor: pointer;
  }

  .close-btn:hover {
    color: #fff;
  }

  .modal-body {
    padding: 16px;
    overflow-y: auto;
  }

  .form-group {
    margin-bottom: 16px;
  }

  .form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    color: #999;
  }

  .form-input {
    width: 100%;
    padding: 8px 12px;
    background: #0a0a0a;
    color: #fff;
    border: 1px solid #333;
    border-radius: 4px;
    font-size: 13px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px;
    background: #161616;
    border-top: 1px solid #333;
  }

  .cancel-btn,
  .confirm-btn {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    font-size: 13px;
    cursor: pointer;
  }

  .cancel-btn {
    background: #333;
    color: #fff;
  }

  .cancel-btn:hover {
    background: #3a3a3a;
  }

  .confirm-btn {
    background: #22c55e;
    color: #fff;
  }

  .confirm-btn:hover {
    background: #16a34a;
  }

  .confirm-btn:disabled {
    background: #333;
    color: #666;
    cursor: not-allowed;
  }

  .project-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 12px;
  }

  .project-card {
    background: #252525;
    border: 1px solid #333;
    border-radius: 6px;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s;
  }

  .project-card:hover {
    border-color: #3b82f6;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px;
    background: #1e1e1e;
  }

  .card-body {
    padding: 12px;
  }

  .project-meta {
    font-size: 12px;
    color: #999;
    margin: 4px 0;
  }

  .delete-btn {
    width: 24px;
    height: 24px;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .delete-btn:hover {
    background: #b91c1c;
  }

  .empty-state {
    text-align: center;
    padding: 48px 24px;
    color: #666;
  }

  .empty-state p {
    margin: 4px 0;
  }
</style>
```

---

## Verification (15 min)

```bash
npm run check
make dev
```

Test project management:
1. Click "New" to create a project
2. Make changes (mark as changed)
3. Click "Save" to save changes
4. Click "Save As" to create a copy
5. Click "Open" to see all projects
6. Click recent project to open
7. Verify unsaved indicator appears
8. Click "Close" and confirm save prompt

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Recent projects not showing | Verify last_opened_at is set on load |
| Save doesn't capture state | Ensure serialize_session is called |
| Unsaved indicator stuck | Reset hasUnsavedChanges after save |
| Delete fails | Check project isn't currently open |

---

## What's Next?

✅ **Day 4 Complete! You've built:**
- Complete project management system
- New/open/save/save-as workflows
- Recent projects list
- Unsaved changes tracking
- Project browser with delete functionality

**Next:** [Day 5, Part 5A: MIDI Export](./DAY5_PART_A_MIDI_EXPORT.md)
- Export to MIDI file
- Render automation to MIDI CC
- Export all tracks
- Tempo/time signature handling
