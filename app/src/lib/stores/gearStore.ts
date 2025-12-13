/**
 * Gear Store
 *
 * Manages MIDI gear profiles and user equipment including:
 * - Gear profiles (synthesizers, drum machines, controllers, etc.)
 * - CC mappings for each gear profile
 * - Program/patch libraries
 * - User's personal gear collection with favorites
 */

import { derived, get, writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { Commands } from '$lib/api/commands';
import type {
  GearProfile,
  GearType,
  CCMapping,
  GearProgram,
  UserGear,
  UserGearWithProfile,
} from '$lib/types';

// ============================================================================
// TYPES
// ============================================================================

export interface GearState {
  // Gear Profiles
  profiles: GearProfile[];
  profilesByType: Map<GearType, GearProfile[]>;
  selectedProfile: GearProfile | null;

  // CC Mappings
  ccMappings: CCMapping[];

  // Programs
  programs: GearProgram[];
  programsByBank: Map<number, GearProgram[]>;

  // User Gear
  userGear: UserGear[];
  favoriteGear: UserGear[];
  selectedUserGear: UserGearWithProfile | null;

  // UI State
  activeTab: 'profiles' | 'user-gear' | 'programs';
  searchQuery: string;
  filterType: GearType | null;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
}

// ============================================================================
// INITIAL STATE
// ============================================================================

const initialState: GearState = {
  profiles: [],
  profilesByType: new Map(),
  selectedProfile: null,
  ccMappings: [],
  programs: [],
  programsByBank: new Map(),
  userGear: [],
  favoriteGear: [],
  selectedUserGear: null,
  activeTab: 'user-gear',
  searchQuery: '',
  filterType: null,
  isLoading: false,
  isSaving: false,
  error: null,
};

// ============================================================================
// STORE
// ============================================================================

const { subscribe, set, update } = writable<GearState>(initialState);

export const gearStore = { subscribe };

// ============================================================================
// DERIVED STORES
// ============================================================================

export const gearTypes = derived(gearStore, ($store) =>
  Array.from($store.profilesByType.keys())
);

export const profileCount = derived(gearStore, ($store) => $store.profiles.length);

export const userGearCount = derived(gearStore, ($store) => $store.userGear.length);

export const favoriteGearCount = derived(gearStore, ($store) => $store.favoriteGear.length);

export const filteredProfiles = derived(gearStore, ($store) => {
  let filtered = $store.profiles;

  // Filter by type
  if ($store.filterType) {
    filtered = filtered.filter((p) => p.gear_type === $store.filterType);
  }

  // Filter by search query
  if ($store.searchQuery) {
    const query = $store.searchQuery.toLowerCase();
    filtered = filtered.filter(
      (p) =>
        p.name.toLowerCase().includes(query) ||
        p.manufacturer.toLowerCase().includes(query) ||
        (p.description && p.description.toLowerCase().includes(query))
    );
  }

  return filtered;
});

export const filteredUserGear = derived(gearStore, ($store) => {
  if (!$store.searchQuery) {
    return $store.userGear;
  }

  const query = $store.searchQuery.toLowerCase();
  return $store.userGear.filter(
    (g) =>
      (g.nickname && g.nickname.toLowerCase().includes(query)) ||
      (g.notes && g.notes.toLowerCase().includes(query))
  );
});

// ============================================================================
// ACTIONS
// ============================================================================

export const gearActions = {
  // ==========================================================================
  // GEAR PROFILES
  // ==========================================================================

  async loadProfiles(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const profiles = await invoke<GearProfile[]>(Commands.GEAR_PROFILES_LIST);

      // Group by type
      const byType = new Map<GearType, GearProfile[]>();
      for (const profile of profiles) {
        if (!byType.has(profile.gear_type)) {
          byType.set(profile.gear_type, []);
        }
        byType.get(profile.gear_type)!.push(profile);
      }

      update((state) => ({
        ...state,
        profiles,
        profilesByType: byType,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load profiles:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async loadProfilesByType(gearType: GearType): Promise<GearProfile[]> {
    try {
      const profiles = await invoke<GearProfile[]>(Commands.GEAR_PROFILES_LIST_BY_TYPE, {
        gear_type: gearType,
      });
      update((state) => {
        const byType = new Map(state.profilesByType);
        byType.set(gearType, profiles);
        return { ...state, profilesByType: byType };
      });
      return profiles;
    } catch (error) {
      console.error('Failed to load profiles by type:', error);
      return [];
    }
  },

  async getProfile(id: number): Promise<GearProfile | null> {
    try {
      return await invoke<GearProfile | null>(Commands.GEAR_PROFILES_GET, { id });
    } catch (error) {
      console.error('Failed to get profile:', error);
      return null;
    }
  },

  async searchProfiles(query: string): Promise<GearProfile[]> {
    try {
      return await invoke<GearProfile[]>(Commands.GEAR_PROFILES_SEARCH, { query });
    } catch (error) {
      console.error('Failed to search profiles:', error);
      return [];
    }
  },

  async createProfile(profile: Omit<GearProfile, 'id' | 'created_at' | 'modified_at'>): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const id = await invoke<number>(Commands.GEAR_PROFILES_CREATE, profile);
      await this.loadProfiles();
      update((state) => ({ ...state, isSaving: false }));
      return id;
    } catch (error) {
      console.error('Failed to create profile:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  async updateProfile(id: number, profile: Partial<GearProfile>): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.GEAR_PROFILES_UPDATE, { id, ...profile });
      await this.loadProfiles();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update profile:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async deleteProfile(id: number): Promise<void> {
    try {
      await invoke(Commands.GEAR_PROFILES_DELETE, { id });
      await this.loadProfiles();
    } catch (error) {
      console.error('Failed to delete profile:', error);
    }
  },

  selectProfile(profile: GearProfile | null): void {
    update((state) => ({ ...state, selectedProfile: profile }));
    if (profile) {
      this.loadCCMappings(profile.id);
      this.loadPrograms(profile.id);
    }
  },

  // ==========================================================================
  // CC MAPPINGS
  // ==========================================================================

  async loadCCMappings(profileId: number): Promise<void> {
    try {
      const mappings = await invoke<CCMapping[]>(Commands.GEAR_CC_LIST, { profile_id: profileId });
      update((state) => ({ ...state, ccMappings: mappings }));
    } catch (error) {
      console.error('Failed to load CC mappings:', error);
    }
  },

  async getCCMapping(id: number): Promise<CCMapping | null> {
    try {
      return await invoke<CCMapping | null>(Commands.GEAR_CC_GET, { id });
    } catch (error) {
      console.error('Failed to get CC mapping:', error);
      return null;
    }
  },

  async getCCMappingByNumber(profileId: number, ccNumber: number): Promise<CCMapping | null> {
    try {
      return await invoke<CCMapping | null>(Commands.GEAR_CC_GET_BY_NUMBER, {
        profile_id: profileId,
        cc_number: ccNumber,
      });
    } catch (error) {
      console.error('Failed to get CC mapping by number:', error);
      return null;
    }
  },

  async createCCMapping(mapping: Omit<CCMapping, 'id'>): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const id = await invoke<number>(Commands.GEAR_CC_CREATE, mapping);
      if (get(gearStore).selectedProfile?.id === mapping.gear_profile_id) {
        await this.loadCCMappings(mapping.gear_profile_id);
      }
      update((state) => ({ ...state, isSaving: false }));
      return id;
    } catch (error) {
      console.error('Failed to create CC mapping:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  async updateCCMapping(id: number, mapping: Partial<CCMapping>): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.GEAR_CC_UPDATE, { id, ...mapping });
      const profileId = get(gearStore).selectedProfile?.id;
      if (profileId) {
        await this.loadCCMappings(profileId);
      }
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update CC mapping:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async deleteCCMapping(id: number): Promise<void> {
    try {
      await invoke(Commands.GEAR_CC_DELETE, { id });
      const profileId = get(gearStore).selectedProfile?.id;
      if (profileId) {
        await this.loadCCMappings(profileId);
      }
    } catch (error) {
      console.error('Failed to delete CC mapping:', error);
    }
  },

  // ==========================================================================
  // PROGRAMS
  // ==========================================================================

  async loadPrograms(profileId: number): Promise<void> {
    try {
      const programs = await invoke<GearProgram[]>(Commands.GEAR_PROGRAMS_LIST, {
        profile_id: profileId,
      });

      // Group by bank
      const byBank = new Map<number, GearProgram[]>();
      for (const program of programs) {
        if (!byBank.has(program.bank_msb)) {
          byBank.set(program.bank_msb, []);
        }
        byBank.get(program.bank_msb)!.push(program);
      }

      update((state) => ({
        ...state,
        programs,
        programsByBank: byBank,
      }));
    } catch (error) {
      console.error('Failed to load programs:', error);
    }
  },

  async loadProgramsByBank(profileId: number, bankMsb: number): Promise<GearProgram[]> {
    try {
      const programs = await invoke<GearProgram[]>(Commands.GEAR_PROGRAMS_LIST_BY_BANK, {
        profile_id: profileId,
        bank_msb: bankMsb,
      });
      update((state) => {
        const byBank = new Map(state.programsByBank);
        byBank.set(bankMsb, programs);
        return { ...state, programsByBank: byBank };
      });
      return programs;
    } catch (error) {
      console.error('Failed to load programs by bank:', error);
      return [];
    }
  },

  async getProgram(id: number): Promise<GearProgram | null> {
    try {
      return await invoke<GearProgram | null>(Commands.GEAR_PROGRAMS_GET, { id });
    } catch (error) {
      console.error('Failed to get program:', error);
      return null;
    }
  },

  async searchPrograms(profileId: number, query: string): Promise<GearProgram[]> {
    try {
      return await invoke<GearProgram[]>(Commands.GEAR_PROGRAMS_SEARCH, {
        profile_id: profileId,
        query,
      });
    } catch (error) {
      console.error('Failed to search programs:', error);
      return [];
    }
  },

  async createProgram(program: Omit<GearProgram, 'id'>): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const id = await invoke<number>(Commands.GEAR_PROGRAMS_CREATE, program);
      if (get(gearStore).selectedProfile?.id === program.gear_profile_id) {
        await this.loadPrograms(program.gear_profile_id);
      }
      update((state) => ({ ...state, isSaving: false }));
      return id;
    } catch (error) {
      console.error('Failed to create program:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  async updateProgram(id: number, program: Partial<GearProgram>): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.GEAR_PROGRAMS_UPDATE, { id, ...program });
      const profileId = get(gearStore).selectedProfile?.id;
      if (profileId) {
        await this.loadPrograms(profileId);
      }
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update program:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async deleteProgram(id: number): Promise<void> {
    try {
      await invoke(Commands.GEAR_PROGRAMS_DELETE, { id });
      const profileId = get(gearStore).selectedProfile?.id;
      if (profileId) {
        await this.loadPrograms(profileId);
      }
    } catch (error) {
      console.error('Failed to delete program:', error);
    }
  },

  // ==========================================================================
  // USER GEAR
  // ==========================================================================

  async loadUserGear(): Promise<void> {
    try {
      update((state) => ({ ...state, isLoading: true, error: null }));
      const userGear = await invoke<UserGear[]>(Commands.USER_GEAR_LIST);
      const favorites = await invoke<UserGear[]>(Commands.USER_GEAR_LIST_FAVORITES);
      update((state) => ({
        ...state,
        userGear,
        favoriteGear: favorites,
        isLoading: false,
      }));
    } catch (error) {
      console.error('Failed to load user gear:', error);
      update((state) => ({
        ...state,
        isLoading: false,
        error: String(error),
      }));
    }
  },

  async getUserGear(id: number): Promise<UserGear | null> {
    try {
      return await invoke<UserGear | null>(Commands.USER_GEAR_GET, { id });
    } catch (error) {
      console.error('Failed to get user gear:', error);
      return null;
    }
  },

  async getUserGearWithProfile(id: number): Promise<UserGearWithProfile | null> {
    try {
      const result = await invoke<UserGearWithProfile | null>(Commands.USER_GEAR_GET_WITH_PROFILE, {
        id,
      });
      if (result) {
        update((state) => ({ ...state, selectedUserGear: result }));
      }
      return result;
    } catch (error) {
      console.error('Failed to get user gear with profile:', error);
      return null;
    }
  },

  async addUserGear(gear: Omit<UserGear, 'id' | 'created_at' | 'last_used'>): Promise<number> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      const id = await invoke<number>(Commands.USER_GEAR_ADD, gear);
      await this.loadUserGear();
      update((state) => ({ ...state, isSaving: false }));
      return id;
    } catch (error) {
      console.error('Failed to add user gear:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
      throw error;
    }
  },

  async updateUserGear(id: number, gear: Partial<UserGear>): Promise<void> {
    try {
      update((state) => ({ ...state, isSaving: true }));
      await invoke(Commands.USER_GEAR_UPDATE, { id, ...gear });
      await this.loadUserGear();
      update((state) => ({ ...state, isSaving: false }));
    } catch (error) {
      console.error('Failed to update user gear:', error);
      update((state) => ({ ...state, isSaving: false, error: String(error) }));
    }
  },

  async setFavorite(id: number, favorite: boolean): Promise<void> {
    try {
      await invoke(Commands.USER_GEAR_SET_FAVORITE, { id, favorite });
      await this.loadUserGear();
    } catch (error) {
      console.error('Failed to set favorite:', error);
    }
  },

  async markUsed(id: number): Promise<void> {
    try {
      await invoke(Commands.USER_GEAR_MARK_USED, { id });
    } catch (error) {
      console.error('Failed to mark used:', error);
    }
  },

  async removeUserGear(id: number): Promise<void> {
    try {
      await invoke(Commands.USER_GEAR_REMOVE, { id });
      await this.loadUserGear();
    } catch (error) {
      console.error('Failed to remove user gear:', error);
    }
  },

  selectUserGear(gear: UserGearWithProfile | null): void {
    update((state) => ({ ...state, selectedUserGear: gear }));
  },

  // ==========================================================================
  // UI STATE
  // ==========================================================================

  setActiveTab(tab: GearState['activeTab']): void {
    update((state) => ({ ...state, activeTab: tab }));
  },

  setSearchQuery(query: string): void {
    update((state) => ({ ...state, searchQuery: query }));
  },

  setFilterType(gearType: GearType | null): void {
    update((state) => ({ ...state, filterType: gearType }));
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
    await Promise.all([this.loadProfiles(), this.loadUserGear()]);
  },
};

// Auto-initialize if in browser
if (typeof window !== 'undefined') {
  setTimeout(() => {
    gearActions.initialize().catch(console.error);
  }, 100);
}
