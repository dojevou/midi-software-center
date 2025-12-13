// Notation store - score rendering and export
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

interface NotationState {
  svgContent: string | null;
  loading: boolean;
  error: string | null;
  currentFile: string | null;
  quantizeLevel: 'whole' | 'half' | 'quarter' | 'eighth' | 'sixteenth' | '32nd';
}

export const notationState = writable<NotationState>({
  svgContent: null,
  loading: false,
  error: null,
  currentFile: null,
  quantizeLevel: 'sixteenth',
});

export const notationActions = {
  async renderScore(midiBytes: number[], title: string, width = 800, height = 400) {
    notationState.update((s) => ({ ...s, loading: true, error: null }));
    try {
      const svg = await invoke<string>('render_score_svg', {
        midiBytes,
        title,
        width,
        height,
      });
      notationState.update((s) => ({ ...s, svgContent: svg, loading: false }));
    } catch (e) {
      notationState.update((s) => ({
        ...s,
        error: String(e),
        loading: false,
      }));
    }
  },

  async exportMusicXML(midiBytes: number[], title: string): Promise<string> {
    return invoke<string>('export_musicxml', { midiBytes, title });
  },

  setQuantizeLevel(level: NotationState['quantizeLevel']) {
    notationState.update((s) => ({ ...s, quantizeLevel: level }));
  },

  clear() {
    notationState.set({
      svgContent: null,
      loading: false,
      error: null,
      currentFile: null,
      quantizeLevel: 'sixteenth',
    });
  },
};
