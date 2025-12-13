import { derived, get, writable } from 'svelte/store';
import { timbresApi } from '../api/timbres';
import { stylesApi } from '../api/styles';
import { articulationsApi } from '../api/articulations';
import { collectionsApi } from '../api/collections';
import { savedSearchesApi } from '../api/savedSearches';
import { vip3BrowserApi } from '../api/vip3Browser';

// =============================================================================
// VIP3 BROWSER STATE - Akai VIP3-style category filtering
// =============================================================================

// Category types matching database schema
export interface Timbre {
  id: number;
  name: string;
  description: string;
  file_count: number;
}

export interface Style {
  id: number;
  name: string;
  description: string;
  file_count: number;
}

export interface Articulation {
  id: number;
  name: string;
  description: string;
  file_count: number;
}

export interface BpmRange {
  id: number;
  label: string;
  min_bpm: number;
  max_bpm: number;
  file_count: number;
}

export interface MusicalKey {
  id: number;
  name: string;
  display_name: string;
  is_minor: boolean;
  file_count: number;
}

export interface VIP3File {
  id: number;
  filename: string;
  filepath: string;
  bpm: number | null;
  key_signature: string | null;
  duration_seconds: number | null;
  timbres: string[];
  styles: string[];
  articulations: string[];
}

export interface SavedSearch {
  id: number;
  name: string;
  filters: VIP3Filters;
  created_at: string;
}

export interface Collection {
  id: number;
  name: string;
  description: string;
  file_count: number;
  created_at: string;
}

// Filter state
export interface VIP3Filters {
  timbreIds: number[];
  styleIds: number[];
  articulationIds: number[];
  bpmRangeId: number | null;
  keyIds: number[];
  searchQuery: string;
  sortBy: 'filename' | 'bpm' | 'key_signature' | 'duration' | 'created_at';
  sortOrder: 'asc' | 'desc';
}

// Main store state
export interface VIP3State {
  // Category data
  timbres: Timbre[];
  styles: Style[];
  articulations: Articulation[];
  bpmRanges: BpmRange[];
  musicalKeys: MusicalKey[];

  // Selection state
  filters: VIP3Filters;

  // Results
  files: VIP3File[];
  totalCount: number;
  isLoading: boolean;
  error: string | null;

  // Pagination
  page: number;
  pageSize: number;

  // User data
  savedSearches: SavedSearch[];
  recentSearches: VIP3Filters[];
  collections: Collection[];
  activeCollectionId: number | null;
}

const initialFilters: VIP3Filters = {
  timbreIds: [],
  styleIds: [],
  articulationIds: [],
  bpmRangeId: null,
  keyIds: [],
  searchQuery: '',
  sortBy: 'filename',
  sortOrder: 'asc',
};

const initialState: VIP3State = {
  timbres: [],
  styles: [],
  articulations: [],
  bpmRanges: [],
  musicalKeys: [],
  filters: { ...initialFilters },
  files: [],
  totalCount: 0,
  isLoading: false,
  error: null,
  page: 1,
  pageSize: 50,
  savedSearches: [],
  recentSearches: [],
  collections: [],
  activeCollectionId: null,
};

export const vip3Store = writable<VIP3State>(initialState);

// =============================================================================
// DERIVED STORES
// =============================================================================

export const selectedTimbres = derived(vip3Store, ($state) =>
  $state.timbres.filter((t) => $state.filters.timbreIds.includes(t.id))
);

export const selectedStyles = derived(vip3Store, ($state) =>
  $state.styles.filter((s) => $state.filters.styleIds.includes(s.id))
);

export const selectedArticulations = derived(vip3Store, ($state) =>
  $state.articulations.filter((a) => $state.filters.articulationIds.includes(a.id))
);

export const selectedBpmRange = derived(vip3Store, ($state) =>
  $state.bpmRanges.find((r) => r.id === $state.filters.bpmRangeId) || null
);

export const selectedKeys = derived(vip3Store, ($state) =>
  $state.musicalKeys.filter((k) => $state.filters.keyIds.includes(k.id))
);

export const hasActiveFilters = derived(vip3Store, ($state) => {
  const f = $state.filters;
  return (
    f.timbreIds.length > 0 ||
    f.styleIds.length > 0 ||
    f.articulationIds.length > 0 ||
    f.bpmRangeId !== null ||
    f.keyIds.length > 0 ||
    f.searchQuery.length > 0
  );
});

export const filterSummary = derived(
  [selectedTimbres, selectedStyles, selectedArticulations, selectedBpmRange, selectedKeys],
  ([$timbres, $styles, $articulations, $bpmRange, $keys]) => {
    const parts: string[] = [];
    if ($timbres.length > 0) {parts.push($timbres.map((t) => t.name).join(', '));}
    if ($styles.length > 0) {parts.push($styles.map((s) => s.name).join(', '));}
    if ($articulations.length > 0) {parts.push($articulations.map((a) => a.name).join(', '));}
    if ($bpmRange) {parts.push($bpmRange.label);}
    if ($keys.length > 0) {parts.push($keys.map((k) => k.display_name).join(', '));}
    return parts.join(' \u2022 ') || 'All Files';
  }
);

export const pageCount = derived(vip3Store, ($state) =>
  Math.ceil($state.totalCount / $state.pageSize)
);

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Convert internal VIP3Filters to API-compatible VIP3BrowserFilters
 */
function filtersToApiFormat(filters: VIP3Filters, page: number, pageSize: number) {
  return {
    search_text: filters.searchQuery || undefined,
    timbre_ids: filters.timbreIds.length > 0 ? filters.timbreIds : undefined,
    style_ids: filters.styleIds.length > 0 ? filters.styleIds : undefined,
    articulation_ids: filters.articulationIds.length > 0 ? filters.articulationIds : undefined,
    key_ids: filters.keyIds.length > 0 ? filters.keyIds : undefined,
    sort_by: filters.sortBy as 'filename' | 'bpm' | 'key_signature' | 'duration' | 'created_at',
    sort_desc: filters.sortOrder === 'desc',
    limit: pageSize,
    offset: (page - 1) * pageSize,
  };
}

// =============================================================================
// ACTIONS
// =============================================================================

export const vip3Actions = {
  // Load all category data from backend
  async loadCategories() {
    vip3Store.update((s) => ({ ...s, isLoading: true, error: null }));
    try {
      const categories = await vip3BrowserApi.getAllCategories();

      // Map API response to local types (adding file_count from counts if available)
      const timbres: Timbre[] = categories.timbres.map((t) => ({
        id: t.id,
        name: t.name,
        description: t.description || '',
        file_count: 0, // Will be updated when counts are loaded
      }));

      const styles: Style[] = categories.styles.map((s) => ({
        id: s.id,
        name: s.name,
        description: s.description || '',
        file_count: 0,
      }));

      const articulations: Articulation[] = categories.articulations.map((a) => ({
        id: a.id,
        name: a.name,
        description: a.description || '',
        file_count: 0,
      }));

      const bpmRanges: BpmRange[] = categories.bpmRanges.map((b) => ({
        id: b.id,
        label: b.name,
        min_bpm: b.min_bpm,
        max_bpm: b.max_bpm,
        file_count: 0,
      }));

      const musicalKeys: MusicalKey[] = categories.musicalKeys.map((k) => ({
        id: k.id,
        name: k.name,
        display_name: k.name,
        is_minor: k.mode === 'minor',
        file_count: 0,
      }));

      vip3Store.update((s) => ({
        ...s,
        timbres,
        styles,
        articulations,
        bpmRanges,
        musicalKeys,
        isLoading: false,
      }));

      // Load counts to update file_count values
      await vip3Actions.loadCounts();
    } catch (error) {
      console.error('Failed to load VIP3 categories:', error);
      vip3Store.update((s) => ({
        ...s,
        isLoading: false,
        error: 'Failed to load categories',
      }));
    }
  },

  // Load category counts
  async loadCounts() {
    try {
      const counts = await vip3BrowserApi.getCategoryCounts();

      vip3Store.update((s) => ({
        ...s,
        timbres: s.timbres.map((t) => ({ ...t, file_count: counts.timbres[t.id] || 0 })),
        styles: s.styles.map((st) => ({ ...st, file_count: counts.styles[st.id] || 0 })),
        articulations: s.articulations.map((a) => ({
          ...a,
          file_count: counts.articulations[a.id] || 0,
        })),
      }));
    } catch (error) {
      console.error('Failed to load category counts:', error);
    }
  },

  // Search with current filters
  async search() {
    const state = get(vip3Store);
    vip3Store.update((s) => ({ ...s, isLoading: true, error: null }));

    try {
      const apiFilters = filtersToApiFormat(state.filters, state.page, state.pageSize);
      const result = await vip3BrowserApi.search(apiFilters);

      // Map API files to local VIP3File format
      const files: VIP3File[] = result.files.map((f) => ({
        id: f.id,
        filename: f.filename,
        filepath: f.filepath,
        bpm: f.bpm || null,
        key_signature: f.key_signature || null,
        duration_seconds: f.duration_seconds || null,
        timbres: f.tags?.filter((t: string) => t.startsWith('timbre:')) || [],
        styles: f.tags?.filter((t: string) => t.startsWith('style:')) || [],
        articulations: f.tags?.filter((t: string) => t.startsWith('articulation:')) || [],
      }));

      vip3Store.update((s) => ({
        ...s,
        files,
        totalCount: result.total,
        isLoading: false,
      }));

      // Add to recent searches
      vip3Actions.addRecentSearch(state.filters);
    } catch (error) {
      console.error('VIP3 search failed:', error);
      vip3Store.update((s) => ({
        ...s,
        isLoading: false,
        error: 'Search failed',
      }));
    }
  },

  // Filter toggles
  toggleTimbre(id: number) {
    vip3Store.update((s) => {
      const ids = s.filters.timbreIds.includes(id)
        ? s.filters.timbreIds.filter((x) => x !== id)
        : [...s.filters.timbreIds, id];
      return { ...s, filters: { ...s.filters, timbreIds: ids }, page: 1 };
    });
    vip3Actions.search();
  },

  toggleStyle(id: number) {
    vip3Store.update((s) => {
      const ids = s.filters.styleIds.includes(id)
        ? s.filters.styleIds.filter((x) => x !== id)
        : [...s.filters.styleIds, id];
      return { ...s, filters: { ...s.filters, styleIds: ids }, page: 1 };
    });
    vip3Actions.search();
  },

  toggleArticulation(id: number) {
    vip3Store.update((s) => {
      const ids = s.filters.articulationIds.includes(id)
        ? s.filters.articulationIds.filter((x) => x !== id)
        : [...s.filters.articulationIds, id];
      return { ...s, filters: { ...s.filters, articulationIds: ids }, page: 1 };
    });
    vip3Actions.search();
  },

  setBpmRange(id: number | null) {
    vip3Store.update((s) => ({
      ...s,
      filters: { ...s.filters, bpmRangeId: id },
      page: 1,
    }));
    vip3Actions.search();
  },

  toggleKey(id: number) {
    vip3Store.update((s) => {
      const ids = s.filters.keyIds.includes(id)
        ? s.filters.keyIds.filter((x) => x !== id)
        : [...s.filters.keyIds, id];
      return { ...s, filters: { ...s.filters, keyIds: ids }, page: 1 };
    });
    vip3Actions.search();
  },

  setSearchQuery(query: string) {
    vip3Store.update((s) => ({
      ...s,
      filters: { ...s.filters, searchQuery: query },
      page: 1,
    }));
    // Debounce search for text input
    vip3Actions.search();
  },

  setSorting(sortBy: VIP3Filters['sortBy'], sortOrder: VIP3Filters['sortOrder']) {
    vip3Store.update((s) => ({
      ...s,
      filters: { ...s.filters, sortBy, sortOrder },
    }));
    vip3Actions.search();
  },

  // Clear all filters
  clearFilters() {
    vip3Store.update((s) => ({
      ...s,
      filters: { ...initialFilters },
      page: 1,
    }));
    vip3Actions.search();
  },

  // Pagination
  setPage(page: number) {
    vip3Store.update((s) => ({ ...s, page }));
    vip3Actions.search();
  },

  nextPage() {
    const state = get(vip3Store);
    const maxPage = Math.ceil(state.totalCount / state.pageSize);
    if (state.page < maxPage) {
      vip3Actions.setPage(state.page + 1);
    }
  },

  prevPage() {
    const state = get(vip3Store);
    if (state.page > 1) {
      vip3Actions.setPage(state.page - 1);
    }
  },

  // Saved searches
  async saveCurrentSearch(name: string) {
    const state = get(vip3Store);
    try {
      const apiFilters = filtersToApiFormat(state.filters, 1, state.pageSize);
      const saved = await savedSearchesApi.create({ name, filters: apiFilters });
      if (saved) {
        const localSaved: SavedSearch = {
          id: saved.id,
          name: saved.name,
          filters: state.filters, // Keep the original local format
          created_at: saved.created_at,
        };
        vip3Store.update((s) => ({
          ...s,
          savedSearches: [...s.savedSearches, localSaved],
        }));
      }
    } catch (error) {
      console.error('Failed to save search:', error);
    }
  },

  async loadSavedSearch(id: number) {
    const state = get(vip3Store);
    const saved = state.savedSearches.find((s) => s.id === id);
    if (saved) {
      vip3Store.update((s) => ({
        ...s,
        filters: { ...saved.filters },
        page: 1,
      }));
      vip3Actions.search();
    }
  },

  async deleteSavedSearch(id: number) {
    try {
      await savedSearchesApi.delete(id);
      vip3Store.update((s) => ({
        ...s,
        savedSearches: s.savedSearches.filter((ss) => ss.id !== id),
      }));
    } catch (error) {
      console.error('Failed to delete saved search:', error);
    }
  },

  addRecentSearch(filters: VIP3Filters) {
    vip3Store.update((s) => {
      // Keep last 10 recent searches
      const recent = [filters, ...s.recentSearches.slice(0, 9)];
      return { ...s, recentSearches: recent };
    });
  },

  // Collections
  async loadCollections() {
    try {
      const apiCollections = await collectionsApi.getAll();
      const collections: Collection[] = apiCollections.map((c) => ({
        id: c.id,
        name: c.name,
        description: c.description || '',
        file_count: c.file_count,
        created_at: c.created_at,
      }));
      vip3Store.update((s) => ({ ...s, collections }));
    } catch (error) {
      console.error('Failed to load collections:', error);
    }
  },

  async createCollection(name: string, description: string) {
    try {
      const apiCollection = await collectionsApi.create({ name, description });
      if (apiCollection) {
        const collection: Collection = {
          id: apiCollection.id,
          name: apiCollection.name,
          description: apiCollection.description || '',
          file_count: apiCollection.file_count,
          created_at: apiCollection.created_at,
        };
        vip3Store.update((s) => ({
          ...s,
          collections: [...s.collections, collection],
        }));
        return collection;
      }
      return null;
    } catch (error) {
      console.error('Failed to create collection:', error);
      return null;
    }
  },

  async addFileToCollection(collectionId: number, fileId: number) {
    try {
      await collectionsApi.addFile(collectionId, fileId);
      // Refresh collection file count
      vip3Actions.loadCollections();
    } catch (error) {
      console.error('Failed to add file to collection:', error);
    }
  },

  async removeFileFromCollection(collectionId: number, fileId: number) {
    try {
      await collectionsApi.removeFile(collectionId, fileId);
      vip3Actions.loadCollections();
    } catch (error) {
      console.error('Failed to remove file from collection:', error);
    }
  },

  setActiveCollection(id: number | null) {
    vip3Store.update((s) => ({ ...s, activeCollectionId: id, page: 1 }));
    if (id) {
      vip3Actions.loadCollectionFiles(id);
    } else {
      vip3Actions.search();
    }
  },

  async loadCollectionFiles(collectionId: number) {
    vip3Store.update((s) => ({ ...s, isLoading: true }));
    try {
      const collectionWithFiles = await collectionsApi.getById(collectionId);

      // CollectionWithFiles has file_ids, we need to search with those IDs
      // For now just set an empty files array - in production you'd fetch the actual files
      const fileIds = collectionWithFiles?.file_ids || [];

      // Search for the files by their IDs
      if (fileIds.length > 0) {
        const result = await vip3BrowserApi.search({
          file_ids: fileIds,
          limit: 1000,
        });

        const files: VIP3File[] = result.files.map((f) => ({
          id: f.id,
          filename: f.filename,
          filepath: f.filepath,
          bpm: f.bpm || null,
          key_signature: f.key_signature || null,
          duration_seconds: f.duration_seconds || null,
          timbres: f.tags?.filter((t: string) => t.startsWith('timbre:')) || [],
          styles: f.tags?.filter((t: string) => t.startsWith('style:')) || [],
          articulations: f.tags?.filter((t: string) => t.startsWith('articulation:')) || [],
        }));

        vip3Store.update((s) => ({
          ...s,
          files,
          totalCount: files.length,
          isLoading: false,
        }));
      } else {
        vip3Store.update((s) => ({
          ...s,
          files: [],
          totalCount: 0,
          isLoading: false,
        }));
      }
    } catch (error) {
      console.error('Failed to load collection files:', error);
      vip3Store.update((s) => ({ ...s, isLoading: false }));
    }
  },

  async deleteCollection(id: number) {
    try {
      await collectionsApi.delete(id);
      vip3Store.update((s) => ({
        ...s,
        collections: s.collections.filter((c) => c.id !== id),
        activeCollectionId: s.activeCollectionId === id ? null : s.activeCollectionId,
      }));
    } catch (error) {
      console.error('Failed to delete collection:', error);
    }
  },

  // File categorization
  async assignTimbre(fileId: number, timbreId: number) {
    try {
      await timbresApi.assignToFile(timbreId, fileId);
    } catch (error) {
      console.error('Failed to assign timbre:', error);
    }
  },

  async assignStyle(fileId: number, styleId: number) {
    try {
      await stylesApi.assignToFile(styleId, fileId);
    } catch (error) {
      console.error('Failed to assign style:', error);
    }
  },

  async assignArticulation(fileId: number, articulationId: number) {
    try {
      await articulationsApi.assignToFile(articulationId, fileId);
    } catch (error) {
      console.error('Failed to assign articulation:', error);
    }
  },
};
