/**
 * Presets Store
 *
 * Manages preset templates including:
 * - Mixer presets (channel configurations, effects chains)
 * - Track templates (MIDI routing, default settings)
 * - Project templates (complete project setups)
 */

import { derived, get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { Commands } from '$lib/api/commands';
import type {
  MixerPreset,
  TrackTemplate,
  TrackType,
  ProjectTemplate,
} from '$lib/types';

// ============================================================================
// TYPES
// ============================================================================

export interface PresetsState {
  // Mixer Presets
  mixerPresets: MixerPreset[];
  mixerPresetsByCategory: Map<string, MixerPreset[]>;
  selectedMixerPreset: MixerPreset | null;

  // Track Templates
  trackTemplates: TrackTemplate[];
  trackTemplatesByCategory: Map<string, TrackTemplate[]>;
  trackTemplatesByType: Map<TrackType, TrackTemplate[]>;
  selectedTrackTemplate: TrackTemplate | null;

  // Project Templates
  projectTemplates: ProjectTemplate[];
  projectTemplatesByCategory: Map<string, ProjectTemplate[]>;
  selectedProjectTemplate: ProjectTemplate | null;

  // UI State
  activeTab: 'mixer' | 'track' | 'project';
  searchQuery: string;
  filterCategory: string | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
}

// ============================================================================
// INITIAL STATE
// ============================================================================

const initialState: PresetsState = {
  mixerPresets: [],
  mixerPresetsByCategory: new Map(),
  selectedMixerPreset: null,
  trackTemplates: [],
  trackTemplatesByCategory: new Map(),
  trackTemplatesByType: new Map(),
  selectedTrackTemplate: null,
  projectTemplates: [],
  projectTemplatesByCategory: new Map(),
  selectedProjectTemplate: null,
  activeTab: 'mixer',
  searchQuery: '',
  filterCategory: null,
  isLoading: false,
  isSaving: false,
  error: null,
};

// ============================================================================
// STORE
// ============================================================================

const { subscribe, set, update } = writable<PresetsState>(initialState);

export const presetsStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const mixerPresetCategories = derived(presetsStore, ($store) =>
  Array.from($store.mixerPresetsByCategory.keys())
);

export const trackTemplateCategories = derived(presetsStore, ($store) =>
  Array.from($store.trackTemplatesByCategory.keys())
);

export const projectTemplateCategories = derived(presetsStore, ($store) =>
  Array.from($store.projectTemplatesByCategory.keys())
);

export const mixerPresetCount = derived(presetsStore, ($store) => $store.mixerPresets.length);

export const trackTemplateCount = derived(presetsStore, ($store) => $store.trackTemplates.length);

export const projectTemplateCount = derived(
  presetsStore,
  ($store) => $store.projectTemplates.length
);

export const filteredMixerPresets = derived(presetsStore, ($store) => {
  let filtered = $store.mixerPresets;

  // Filter by category
  if ($store.filterCategory) {
    filtered = filtered.filter((p) => p.category === $store.filterCategory);
  }

  // Filter by search query
  if ($store.searchQuery) {
    const query = $store.searchQuery.toLowerCase();
    filtered = filtered.filter(
      (p) =>
        p.name.toLowerCase().includes(query) ||
        (p.description && p.description.toLowerCase().includes(query))
    );
  }

  return filtered;
});

export const filteredTrackTemplates = derived(presetsStore, ($store) => {
  let filtered = $store.trackTemplates;

  // Filter by category
  if ($store.filterCategory) {
    filtered = filtered.filter((t) => t.category === $store.filterCategory);
  }

  // Filter by search query
  if ($store.searchQuery) {
    const query = $store.searchQuery.toLowerCase();
    filtered = filtered.filter(
      (t) =>
        t.name.toLowerCase().includes(query) ||
        (t.description && t.description.toLowerCase().includes(query))
    );
  }

  return filtered;
});

export const filteredProjectTemplates = derived(presetsStore, ($store) => {
  let filtered = $store.projectTemplates;

  // Filter by category
  if ($store.filterCategory) {
    filtered = filtered.filter((p) => p.category === $store.filterCategory);
  }

  // Filter by search query
  if ($store.searchQuery) {
    const query = $store.searchQuery.toLowerCase();
    filtered = filtered.filter(
      (p) =>
        p.name.toLowerCase().includes(query) ||
        (p.description && p.description.toLowerCase().includes(query))
    );
  }

  return filtered;
});

// ============================================================================
// ACTIONS
// ============================================================================

export const presetsActions = {
  // ==========================================================================
  // MIXER PRESETS
  // ==========================================================================

  async loadMixerPresets(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const presets = await invoke<MixerPreset[]>(Commands.MIXER_PRESETS_LIST);

      // Group by category
      const byCategory = new Map<string, MixerPreset[]>();
      for (const preset of presets) {
        const category = preset.category || 'general';
        if (!byCategory.has(category)) {
          byCategory.set(category, []);
        }
        byCategory.get(category)!.push(preset);
      }

      update((state) => ({
        ...state,
        mixerPresets: presets,
        mixerPresetsByCategory: byCategory,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load mixer presets:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async loadMixerPresetsByCategory(category: string): Promise<MixerPreset[]> {
    try {
      const presets = await invoke<MixerPreset[]>(Commands.MIXER_PRESETS_LIST_BY_CATEGORY, {
        category,
      });
      update((state) => {
        const byCategory = new Map(state.mixerPresetsByCategory);
        byCategory.set(category, presets);
        return { ...state, mixerPresetsByCategory: byCategory };
      });
      return presets;
    } catch (error) {
      console.error('Failed to load mixer presets by category:', error);
      return [];
    }
  },

  async getMixerPreset(id: number): Promise<MixerPreset | null> {
    try {
      return await invoke<MixerPreset | null>(Commands.MIXER_PRESETS_GET, { id });
    } catch (error) {
      console.error('Failed to get mixer preset:', error);
      return null;
    }
  },

  async searchMixerPresets(query: string): Promise<MixerPreset[]> {
    try {
      return await invoke<MixerPreset[]>(Commands.MIXER_PRESETS_SEARCH, { query });
    } catch (error) {
      console.error('Failed to search mixer presets:', error);
      return [];
    }
  },

  async createMixerPreset(preset: Omit<MixerPreset, 'id' | 'created_at' | 'modified_at'>): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const id = await invoke<number>(Commands.MIXER_PRESETS_CREATE, preset);
      await this.loadMixerPresets();
      update((state) => ({ ...state, isSaving: false }));
      return id;
    } catch (error) {
      console.error('Failed to create mixer preset:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  async updateMixerPreset(id: number, preset: Partial<MixerPreset>): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.MIXER_PRESETS_UPDATE, { id, ...preset });
      await this.loadMixerPresets();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update mixer preset:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async deleteMixerPreset(id: number): Promise<void> {
    try {
      await invoke(Commands.MIXER_PRESETS_DELETE, { id });
      await this.loadMixerPresets();
    } catch (error) {
      console.error('Failed to delete mixer preset:', error);
    }
  },

  selectMixerPreset(preset: MixerPreset | null): void {
    update((state) => ({ ...state, selectedMixerPreset: preset }));
  },

  // ==========================================================================
  // TRACK TEMPLATES
  // ==========================================================================

  async loadTrackTemplates(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const templates = await invoke<TrackTemplate[]>(Commands.TRACK_TEMPLATES_LIST);

      // Group by category
      const byCategory = new Map<string, TrackTemplate[]>();
      for (const template of templates) {
        const category = template.category || 'general';
        if (!byCategory.has(category)) {
          byCategory.set(category, []);
        }
        byCategory.get(category)!.push(template);
      }

      // Group by type
      const byType = new Map<TrackType, TrackTemplate[]>();
      for (const template of templates) {
        if (!byType.has(template.track_type)) {
          byType.set(template.track_type, []);
        }
        byType.get(template.track_type)!.push(template);
      }

      update((state) => ({
        ...state,
        trackTemplates: templates,
        trackTemplatesByCategory: byCategory,
        trackTemplatesByType: byType,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load track templates:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async loadTrackTemplatesByCategory(category: string): Promise<TrackTemplate[]> {
    try {
      const templates = await invoke<TrackTemplate[]>(Commands.TRACK_TEMPLATES_LIST_BY_CATEGORY, {
        category,
      });
      update((state) => {
        const byCategory = new Map(state.trackTemplatesByCategory);
        byCategory.set(category, templates);
        return { ...state, trackTemplatesByCategory: byCategory };
      });
      return templates;
    } catch (error) {
      console.error('Failed to load track templates by category:', error);
      return [];
    }
  },

  async loadTrackTemplatesByType(trackType: TrackType): Promise<TrackTemplate[]> {
    try {
      const templates = await invoke<TrackTemplate[]>(Commands.TRACK_TEMPLATES_LIST_BY_TYPE, {
        track_type: trackType,
      });
      update((state) => {
        const byType = new Map(state.trackTemplatesByType);
        byType.set(trackType, templates);
        return { ...state, trackTemplatesByType: byType };
      });
      return templates;
    } catch (error) {
      console.error('Failed to load track templates by type:', error);
      return [];
    }
  },

  async getTrackTemplate(id: number): Promise<TrackTemplate | null> {
    try {
      return await invoke<TrackTemplate | null>(Commands.TRACK_TEMPLATES_GET, { id });
    } catch (error) {
      console.error('Failed to get track template:', error);
      return null;
    }
  },

  async searchTrackTemplates(query: string): Promise<TrackTemplate[]> {
    try {
      return await invoke<TrackTemplate[]>(Commands.TRACK_TEMPLATES_SEARCH, { query });
    } catch (error) {
      console.error('Failed to search track templates:', error);
      return [];
    }
  },

  async createTrackTemplate(template: Omit<TrackTemplate, 'id' | 'created_at' | 'modified_at'>): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const id = await invoke<number>(Commands.TRACK_TEMPLATES_CREATE, template);
      await this.loadTrackTemplates();
      update((state) => ({ ...state, isSaving: false }));
      return id;
    } catch (error) {
      console.error('Failed to create track template:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  async updateTrackTemplate(id: number, template: Partial<TrackTemplate>): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.TRACK_TEMPLATES_UPDATE, { id, ...template });
      await this.loadTrackTemplates();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update track template:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async deleteTrackTemplate(id: number): Promise<void> {
    try {
      await invoke(Commands.TRACK_TEMPLATES_DELETE, { id });
      await this.loadTrackTemplates();
    } catch (error) {
      console.error('Failed to delete track template:', error);
    }
  },

  selectTrackTemplate(template: TrackTemplate | null): void {
    update((state) => ({ ...state, selectedTrackTemplate: template }));
  },

  // ==========================================================================
  // PROJECT TEMPLATES
  // ==========================================================================

  async loadProjectTemplates(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const templates = await invoke<ProjectTemplate[]>(Commands.PROJECT_TEMPLATES_LIST);

      // Group by category
      const byCategory = new Map<string, ProjectTemplate[]>();
      for (const template of templates) {
        const category = template.category || 'general';
        if (!byCategory.has(category)) {
          byCategory.set(category, []);
        }
        byCategory.get(category)!.push(template);
      }

      update((state) => ({
        ...state,
        projectTemplates: templates,
        projectTemplatesByCategory: byCategory,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load project templates:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async loadProjectTemplatesByCategory(category: string): Promise<ProjectTemplate[]> {
    try {
      const templates = await invoke<ProjectTemplate[]>(
        Commands.PROJECT_TEMPLATES_LIST_BY_CATEGORY,
        { category }
      );
      update((state) => {
        const byCategory = new Map(state.projectTemplatesByCategory);
        byCategory.set(category, templates);
        return { ...state, projectTemplatesByCategory: byCategory };
      });
      return templates;
    } catch (error) {
      console.error('Failed to load project templates by category:', error);
      return [];
    }
  },

  async getProjectTemplate(id: number): Promise<ProjectTemplate | null> {
    try {
      return await invoke<ProjectTemplate | null>(Commands.PROJECT_TEMPLATES_GET, { id });
    } catch (error) {
      console.error('Failed to get project template:', error);
      return null;
    }
  },

  async searchProjectTemplates(query: string): Promise<ProjectTemplate[]> {
    try {
      return await invoke<ProjectTemplate[]>(Commands.PROJECT_TEMPLATES_SEARCH, { query });
    } catch (error) {
      console.error('Failed to search project templates:', error);
      return [];
    }
  },

  async createProjectTemplate(
    template: Omit<ProjectTemplate, 'id' | 'created_at' | 'modified_at'>
  ): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const id = await invoke<number>(Commands.PROJECT_TEMPLATES_CREATE, template);
      await this.loadProjectTemplates();
      update((state) => ({ ...state, isSaving: false }));
      return id;
    } catch (error) {
      console.error('Failed to create project template:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  async updateProjectTemplate(id: number, template: Partial<ProjectTemplate>): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.PROJECT_TEMPLATES_UPDATE, { id, ...template });
      await this.loadProjectTemplates();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update project template:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async deleteProjectTemplate(id: number): Promise<void> {
    try {
      await invoke(Commands.PROJECT_TEMPLATES_DELETE, { id });
      await this.loadProjectTemplates();
    } catch (error) {
      console.error('Failed to delete project template:', error);
    }
  },

  async duplicateProjectTemplate(id: number, newName: string): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const newId = await invoke<number>(Commands.PROJECT_TEMPLATES_DUPLICATE, {
        id,
        new_name: newName,
      });
      await this.loadProjectTemplates();
      update((state) => ({ ...state, isSaving: false }));
      return newId;
    } catch (error) {
      console.error('Failed to duplicate project template:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  selectProjectTemplate(template: ProjectTemplate | null): void {
    update((state) => ({ ...state, selectedProjectTemplate: template }));
  },

  // ==========================================================================
  // UI STATE
  // ==========================================================================

  setActiveTab(tab: PresetsState['activeTab']): void {
    update((state) => ({ ...state, activeTab: tab }));
  },

  setSearchQuery(query: string): void {
    update((state) => ({ ...state, searchQuery: query }));
  },

  setFilterCategory(category: string | null): void {
    update((state) => ({ ...state, filterCategory: category }));
  },

  clearError(): void {
    update((state) => ({ ...state, error: null }));
  },

  reset(): void {
    set(initialState);
  },

  // ==========================================================================
  // INITIALIZATION
  // ==========================================================================

  async initialize(): Promise<void> {
    await Promise.all([
      this.loadMixerPresets(),
      this.loadTrackTemplates(),
      this.loadProjectTemplates(),
    ]);
  },
};

// Auto-initialize if in browser
if (typeof window !== 'undefined') {
  setTimeout(() => {
    presetsActions.initialize().catch(console.error);
  }, 100);
}
