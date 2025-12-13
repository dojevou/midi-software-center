import { derived, writable } from 'svelte/store';
import { api } from '$lib/api';
import type { FileDetails, SearchFilters } from '$lib/types';

console.log('🔧 databaseStore.ts: Store module loading');

// Mock data for browser development (when not in Tauri)
const mockFiles: FileDetails[] = [
  {
    id: 1,
    filename: 'demo_beat_120bpm.mid',
    filepath: '/demo/drums/demo_beat_120bpm.mid',
    file_size_bytes: 2048,
    bpm: 120,
    key_signature: 'C Major',
    time_signature: '4/4',
    duration_seconds: 32.5,
    total_notes: 256,
    primary_category: 'Drums',
    parent_folder: 'drums',
    created_at: new Date().toISOString(),
    is_favorite: true,
    tags: ['drums', 'beat', 'electronic'],
    manufacturer: 'Demo',
    collection_name: 'Demo Collection',
    track_count: 2,
    has_notes: true,
    has_drums: true,
    content_hash: 'mock_hash_1',
  },
  {
    id: 2,
    filename: 'piano_melody_cmaj.mid',
    filepath: '/demo/piano/piano_melody_cmaj.mid',
    file_size_bytes: 4096,
    bpm: 90,
    key_signature: 'C Major',
    time_signature: '4/4',
    duration_seconds: 45.0,
    total_notes: 128,
    primary_category: 'Piano',
    parent_folder: 'piano',
    created_at: new Date().toISOString(),
    is_favorite: false,
    tags: ['piano', 'melody', 'classical'],
    manufacturer: 'Demo',
    collection_name: 'Demo Collection',
    track_count: 1,
    has_notes: true,
    has_drums: false,
    content_hash: 'mock_hash_2',
  },
  {
    id: 3,
    filename: 'synth_pad_ambient.mid',
    filepath: '/demo/synth/synth_pad_ambient.mid',
    file_size_bytes: 3072,
    bpm: 80,
    key_signature: 'A Minor',
    time_signature: '4/4',
    duration_seconds: 60.0,
    total_notes: 64,
    primary_category: 'Synth',
    parent_folder: 'synth',
    created_at: new Date().toISOString(),
    is_favorite: true,
    tags: ['synth', 'pad', 'ambient'],
    manufacturer: 'Demo',
    collection_name: 'Demo Collection',
    track_count: 3,
    has_notes: true,
    has_drums: false,
    content_hash: 'mock_hash_3',
  },
  {
    id: 4,
    filename: 'bass_groove_funky.mid',
    filepath: '/demo/bass/bass_groove_funky.mid',
    file_size_bytes: 1536,
    bpm: 110,
    key_signature: 'E Minor',
    time_signature: '4/4',
    duration_seconds: 16.0,
    total_notes: 48,
    primary_category: 'Bass',
    parent_folder: 'bass',
    created_at: new Date().toISOString(),
    is_favorite: false,
    tags: ['bass', 'groove', 'funky'],
    manufacturer: 'Demo',
    collection_name: 'Demo Collection',
    track_count: 1,
    has_notes: true,
    has_drums: false,
    content_hash: 'mock_hash_4',
  },
  {
    id: 5,
    filename: 'orchestral_strings.mid',
    filepath: '/demo/orchestra/orchestral_strings.mid',
    file_size_bytes: 8192,
    bpm: 72,
    key_signature: 'D Major',
    time_signature: '3/4',
    duration_seconds: 120.0,
    total_notes: 512,
    primary_category: 'Orchestra',
    parent_folder: 'orchestra',
    created_at: new Date().toISOString(),
    is_favorite: true,
    tags: ['orchestra', 'strings', 'cinematic'],
    manufacturer: 'Demo',
    collection_name: 'Demo Collection',
    track_count: 4,
    has_notes: true,
    has_drums: false,
    content_hash: 'mock_hash_5',
  },
];

export type { FileDetails, SearchFilters };

export interface DatabaseState {
  searchResults: FileDetails[];
  currentPage: number;
  totalPages: number;
  totalFiles: number;
  isLoading: boolean;
  selectedFileId: number | null;
}

// Store
const initialState: DatabaseState = {
  searchResults: [],
  currentPage: 0,
  totalPages: 0,
  totalFiles: 0,
  isLoading: false,
  selectedFileId: null,
};

const { subscribe, set, update } = writable<DatabaseState>(initialState);

console.log('🔧 databaseStore.ts: Store created with initial state');

export const databaseStore = { subscribe };
console.log('🔧 databaseStore.ts: databaseStore exported with subscribe only');

// Derived store for totalPages
export const totalPages = derived(databaseStore, ($store) => $store.totalPages);

// Actions (separate export)
export const databaseActions = {
  search: async (params?: SearchFilters) => {
    console.log('🔧 databaseActions.search called with params:', params);

    try {
      update((state) => {
        console.log('🔧 databaseActions.search: updating state to loading');
        return { ...state, isLoading: true };
      });

      // Build search filters with pagination
      const searchFilters: SearchFilters = {
        search_text: params?.search_text,
        min_bpm: params?.min_bpm,
        max_bpm: params?.max_bpm,
        key_signature: params?.key_signature,
        time_signature: params?.time_signature,
        category: params?.category,
        min_notes: params?.min_notes,
        max_notes: params?.max_notes,
        min_duration: params?.min_duration,
        max_duration: params?.max_duration,
        instruments: params?.instruments,
        sort_by: params?.sort_by || 'created_at',
        sort_desc: params?.sort_desc,
        limit: params?.limit || 50,
        offset: (params?.offset || 0) * (params?.limit || 50), // Convert page number to offset
      };

      console.log(
        '🔧 databaseActions.search: calling api.search.files with filters:',
        searchFilters
      );
      const response = await api.search.files(searchFilters);
      console.log('🔧 databaseActions.search: received response:', response);

      const totalPages = Math.ceil(response.total / (searchFilters.limit || 50));

      update((state) => ({
        ...state,
        searchResults: response.files,
        totalPages,
        totalFiles: response.total,
        isLoading: false,
        currentPage: params?.offset || 0,
      }));

      console.log(
        '🔧 databaseActions.search: state updated with',
        response.files.length,
        'files from database, total:',
        response.total
      );
    } catch (error) {
      console.error('❌ databaseActions.search: search failed:', error);
      // Fall back to mock data on error (browser mode or backend error)
      console.log('🔧 databaseActions.search: Falling back to mock data');

      // Filter mock data based on search params for browser development
      let filteredFiles = [...mockFiles];

      if (params?.search_text) {
        const searchLower = params.search_text.toLowerCase();
        filteredFiles = filteredFiles.filter(
          (f) =>
            f.filename.toLowerCase().includes(searchLower) ||
            f.tags.some((t) => t.toLowerCase().includes(searchLower)) ||
            f.primary_category?.toLowerCase().includes(searchLower)
        );
      }

      if (params?.category) {
        filteredFiles = filteredFiles.filter(
          (f) => f.primary_category?.toLowerCase() === params.category?.toLowerCase()
        );
      }

      if (params?.key_signature) {
        filteredFiles = filteredFiles.filter((f) => f.key_signature === params.key_signature);
      }

      if (params?.min_bpm !== undefined) {
        filteredFiles = filteredFiles.filter((f) => (f.bpm || 0) >= (params.min_bpm || 0));
      }

      if (params?.max_bpm !== undefined) {
        filteredFiles = filteredFiles.filter((f) => (f.bpm || 999) <= (params.max_bpm || 999));
      }

      const limit = params?.limit || 50;
      const offset = (params?.offset || 0) * limit;
      const paginatedFiles = filteredFiles.slice(offset, offset + limit);
      const totalPages = Math.ceil(filteredFiles.length / limit);

      update((state) => ({
        ...state,
        searchResults: paginatedFiles,
        totalPages,
        totalFiles: filteredFiles.length,
        isLoading: false,
        currentPage: params?.offset || 0,
      }));

      console.log('🔧 databaseActions.search: Mock data loaded -', paginatedFiles.length, 'files, total:', filteredFiles.length);
    }
  },

  nextPage: async () => {
    console.log('🔧 databaseActions.nextPage called');
    const state = await new Promise<DatabaseState>((resolve) => {
      subscribe((s: DatabaseState) => resolve(s))();
    });

    if (state.currentPage < state.totalPages - 1) {
      // Re-run search with next page offset
      const newOffset = state.currentPage + 1;
      console.log('🔧 databaseActions.nextPage: fetching page', newOffset);
      // This would need the previous search params - should be stored in state
      update((s: DatabaseState) => ({ ...s, currentPage: newOffset }));
    }
  },

  previousPage: async () => {
    console.log('🔧 databaseActions.previousPage called');
    const state = await new Promise<DatabaseState>((resolve) => {
      subscribe((s: DatabaseState) => resolve(s))();
    });

    if (state.currentPage > 0) {
      const newOffset = Math.max(0, state.currentPage - 1);
      console.log('🔧 databaseActions.previousPage: fetching page', newOffset);
      update((s: DatabaseState) => ({ ...s, currentPage: newOffset }));
    }
  },

  goToPage: (page: number) => {
    console.log('🔧 databaseActions.goToPage called with page:', page);
    update((s: DatabaseState) => {
      // Validate page bounds
      const validPage = Math.max(0, Math.min(page, s.totalPages - 1));
      return { ...s, currentPage: validPage };
    });
  },

  getFilters: async () => {
    console.log('🔧 databaseActions.getFilters called');

    try {
      // Get available keys from database
      const keys = await api.search.getSuggestions('', 'key_signature');
      // Get available categories from database
      const categories = await api.search.getSuggestions('', 'category');

      console.log(
        '🔧 databaseActions.getFilters: got filters from database -',
        keys.length,
        'keys,',
        categories.length,
        'categories'
      );
      return {
        keys: keys,
        categories: categories,
      };
    } catch (error) {
      console.error('❌ databaseActions.getFilters: failed to get filters:', error);
      // Fall back to mock filters on error (browser mode or backend error)
      console.log('🔧 databaseActions.getFilters: Falling back to mock filters');
      return {
        keys: ['C Major', 'A Minor', 'E Minor', 'D Major', 'G Major', 'F Major'],
        categories: ['Drums', 'Piano', 'Synth', 'Bass', 'Orchestra', 'Guitar', 'Vocals'],
      };
    }
  },

  refresh: async () => {
    console.log('🔧 databaseActions.refresh called');
    // Get current state and re-run the search with current filters
    const state = await new Promise<DatabaseState>((resolve) => {
      subscribe((s: DatabaseState) => resolve(s))();
    });
    // Re-run search with current page
    await databaseActions.search({ offset: state.currentPage });
  },

  selectFile: (fileId: number) => {
    console.log('🔧 databaseActions.selectFile called with fileId:', fileId);
    update((state) => ({
      ...state,
      selectedFileId: fileId,
    }));
  },
};
