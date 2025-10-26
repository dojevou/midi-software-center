/**
 * Validation utilities
 */

import type { FileCategory, MusicalKey } from '$lib/types';

export function isValidFileCategory(value: string): value is FileCategory {
  const categories: FileCategory[] = [
    'KICK', 'SNARE', 'HIHAT', 'PERCUSSION', 'BASS', 'LEAD',
    'PAD', 'CHORD', 'ARP', 'FX', 'VOCAL', 'LOOP', 'UNKNOWN'
  ];
  return categories.includes(value as FileCategory);
}

export function isValidMusicalKey(value: string): value is MusicalKey {
  const keys: MusicalKey[] = [
    'C', 'Cm', 'C#', 'C#m', 'D', 'Dm', 'D#', 'D#m',
    'E', 'Em', 'F', 'Fm', 'F#', 'F#m', 'G', 'Gm',
    'G#', 'G#m', 'A', 'Am', 'A#', 'A#m', 'B', 'Bm', 'UNKNOWN'
  ];
  return keys.includes(value as MusicalKey);
}

export function isValidBpm(bpm: number): boolean {
  return bpm >= 20 && bpm <= 300;
}

export function isValidTag(tag: string): boolean {
  return tag.length > 0 && tag.length <= 50;
}
