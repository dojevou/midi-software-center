/**
 * Track Colors Utility - MPC 3.0 Style Color Palette
 *
 * Provides color management for tracks based on track type.
 * Supports both track type-based colors (MPC 3.0 style) and
 * custom user-defined colors.
 */

import type { SequencerTrackType } from '$lib/types';
import { TRACK_TYPE_COLORS } from '$lib/types';

// ============================================================================
// TRACK TYPE COLORS
// ============================================================================

/**
 * Get the color for a track type
 */
export function getTrackTypeColor(trackType: SequencerTrackType): string {
  return TRACK_TYPE_COLORS[trackType] || TRACK_TYPE_COLORS.midi;
}

/**
 * Get a lighter version of a track type color (for clips)
 */
export function getTrackTypeColorLight(trackType: SequencerTrackType): string {
  const color = getTrackTypeColor(trackType);
  return adjustColorLightness(color, 20);
}

/**
 * Get a darker version of a track type color (for borders)
 */
export function getTrackTypeColorDark(trackType: SequencerTrackType): string {
  const color = getTrackTypeColor(trackType);
  return adjustColorLightness(color, -20);
}

/**
 * Get color with opacity for clip backgrounds
 */
export function getTrackTypeColorWithOpacity(
  trackType: SequencerTrackType,
  opacity: number = 0.8
): string {
  const color = getTrackTypeColor(trackType);
  return hexToRgba(color, opacity);
}

// ============================================================================
// TRACK TYPE DETECTION
// ============================================================================

/**
 * Detect track type from track properties (heuristic-based)
 *
 * Detection rules:
 * 1. MIDI channel 10 (0-indexed: 9) = drums (GM standard)
 * 2. Name contains drum-related keywords = drums
 * 3. Has audio clips = audio
 * 4. Default = midi
 */
export function detectTrackType(track: {
  name?: string;
  midiChannel?: number;
  hasAudioClips?: boolean;
  hasDrums?: boolean;
}): SequencerTrackType {
  // Check explicit drums flag
  if (track.hasDrums) {
    return 'drum';
  }

  // Check MIDI channel 10 (GM drums)
  if (track.midiChannel === 9 || track.midiChannel === 10) {
    return 'drum';
  }

  // Check name for drum keywords
  if (track.name) {
    const drumKeywords =
      /\b(drum|kick|snare|hihat|hi-hat|cymbal|tom|perc|percussion|808|909|kit|beat)\b/i;
    if (drumKeywords.test(track.name)) {
      return 'drum';
    }
  }

  // Check for audio clips
  if (track.hasAudioClips) {
    return 'audio';
  }

  // Default to MIDI
  return 'midi';
}

// ============================================================================
// CUSTOM COLOR PALETTE
// ============================================================================

/**
 * Predefined custom colors for tracks (MPC 3.0 style vibrant palette)
 */
export const CUSTOM_TRACK_COLORS = [
  '#EF4444', // Red
  '#F97316', // Orange
  '#F59E0B', // Amber
  '#EAB308', // Yellow
  '#84CC16', // Lime
  '#22C55E', // Green
  '#10B981', // Emerald
  '#14B8A6', // Teal
  '#06B6D4', // Cyan
  '#0EA5E9', // Sky
  '#3B82F6', // Blue
  '#6366F1', // Indigo
  '#8B5CF6', // Violet
  '#A855F7', // Purple
  '#D946EF', // Fuchsia
  '#EC4899', // Pink
  '#F43F5E', // Rose
];

/**
 * Get a color from the custom palette by index
 */
export function getCustomColor(index: number): string {
  return CUSTOM_TRACK_COLORS[index % CUSTOM_TRACK_COLORS.length];
}

/**
 * Get next available color (for new tracks)
 */
export function getNextAvailableColor(usedColors: string[]): string {
  const available = CUSTOM_TRACK_COLORS.find((c) => !usedColors.includes(c));
  return available || getCustomColor(usedColors.length);
}

// ============================================================================
// COLOR UTILITIES
// ============================================================================

/**
 * Convert hex color to RGBA
 */
export function hexToRgba(hex: string, alpha: number = 1): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) return `rgba(59, 130, 246, ${alpha})`; // Default blue

  const r = parseInt(result[1], 16);
  const g = parseInt(result[2], 16);
  const b = parseInt(result[3], 16);

  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

/**
 * Convert hex to HSL
 */
export function hexToHsl(hex: string): { h: number; s: number; l: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) return { h: 0, s: 0, l: 50 };

  let r = parseInt(result[1], 16) / 255;
  let g = parseInt(result[2], 16) / 255;
  let b = parseInt(result[3], 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  let h = 0;
  let s = 0;
  const l = (max + min) / 2;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

    switch (max) {
      case r:
        h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
        break;
      case g:
        h = ((b - r) / d + 2) / 6;
        break;
      case b:
        h = ((r - g) / d + 4) / 6;
        break;
    }
  }

  return { h: h * 360, s: s * 100, l: l * 100 };
}

/**
 * Convert HSL to hex
 */
export function hslToHex(h: number, s: number, l: number): string {
  h = h / 360;
  s = s / 100;
  l = l / 100;

  let r, g, b;

  if (s === 0) {
    r = g = b = l;
  } else {
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1 / 6) return p + (q - p) * 6 * t;
      if (t < 1 / 2) return q;
      if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
      return p;
    };

    const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
    const p = 2 * l - q;
    r = hue2rgb(p, q, h + 1 / 3);
    g = hue2rgb(p, q, h);
    b = hue2rgb(p, q, h - 1 / 3);
  }

  const toHex = (x: number) => {
    const hex = Math.round(x * 255).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };

  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

/**
 * Adjust color lightness
 */
export function adjustColorLightness(hex: string, amount: number): string {
  const { h, s, l } = hexToHsl(hex);
  const newL = Math.max(0, Math.min(100, l + amount));
  return hslToHex(h, s, newL);
}

/**
 * Get contrasting text color (black or white)
 */
export function getContrastingTextColor(hex: string): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) return '#ffffff';

  const r = parseInt(result[1], 16);
  const g = parseInt(result[2], 16);
  const b = parseInt(result[3], 16);

  // Calculate relative luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;

  return luminance > 0.5 ? '#000000' : '#ffffff';
}

// ============================================================================
// CSS VARIABLE HELPERS
// ============================================================================

/**
 * Generate CSS variables for a track type
 */
export function getTrackTypeCSSVars(trackType: SequencerTrackType): string {
  const color = getTrackTypeColor(trackType);
  const colorLight = getTrackTypeColorLight(trackType);
  const colorDark = getTrackTypeColorDark(trackType);
  const textColor = getContrastingTextColor(color);

  return `
    --track-color: ${color};
    --track-color-light: ${colorLight};
    --track-color-dark: ${colorDark};
    --track-text-color: ${textColor};
    --track-color-20: ${hexToRgba(color, 0.2)};
    --track-color-40: ${hexToRgba(color, 0.4)};
    --track-color-60: ${hexToRgba(color, 0.6)};
    --track-color-80: ${hexToRgba(color, 0.8)};
  `;
}
