// MIDI Learn store - CC mapping and parameter control
import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface MidiMapping {
  id: string;
  channel: number;
  ccNumber: number;
  targetPath: string;
  minValue: number;
  maxValue: number;
  scalingMode: string;
}

interface LearnState {
  isLearning: boolean;
  targetPath: string | null;
  mappings: MidiMapping[];
}

export const learnState = writable<LearnState>({
  isLearning: false,
  targetPath: null,
  mappings: [],
});

export const isLearning = derived(learnState, ($s) => $s.isLearning);
export const mappingCount = derived(learnState, ($s) => $s.mappings.length);

export const learnActions = {
  async startLearning(targetPath: string) {
    await invoke('learn_start', { targetPath });
    learnState.update((s) => ({ ...s, isLearning: true, targetPath }));
  },

  async cancelLearning() {
    await invoke('learn_cancel');
    learnState.update((s) => ({ ...s, isLearning: false, targetPath: null }));
  },

  async refreshMappings() {
    const mappings = await invoke<MidiMapping[]>('learn_list_mappings');
    learnState.update((s) => ({ ...s, mappings }));
  },

  async removeMapping(id: string) {
    await invoke('learn_remove_mapping', { mappingId: id });
    learnState.update((s) => ({
      ...s,
      mappings: s.mappings.filter((m) => m.id !== id),
    }));
  },

  async exportMappings(): Promise<string> {
    return invoke<string>('learn_export_mappings');
  },

  async importMappings(json: string): Promise<number> {
    const count = await invoke<number>('learn_import_mappings', { json });
    await this.refreshMappings();
    return count;
  },
};
