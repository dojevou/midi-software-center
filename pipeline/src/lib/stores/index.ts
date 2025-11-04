/**
 * Store Index - Central export point for all stores
 *
 * This barrel file exports all store modules for easy importing across the application.
 * Usage: import { playbackStore, projectStore } from '$lib/stores';
 */

// Playback state management
export { playbackStore, playbackActions } from './playbackStore';

// Project and track management
export { projectStore, projectActions } from './projectStore';

// Database search and file management
export { databaseStore, databaseActions } from './databaseStore';

// UI state and window management
export { uiStore, uiActions } from './uiStore';

// Type exports for convenience
export type { PlaybackState, Position, TimeSignature } from './playbackStore';
export type { ProjectState, TrackInfo } from './projectStore';
export type { DatabaseState, SearchResult, SearchFilters } from './databaseStore';
export type { UIState, WindowPosition, WindowId } from './uiStore';
