// Comprehensive State Management for MIDI Library DAW
// Centralizes all application state with Svelte stores, backend sync, and localStorage persistence

import { writable, derived, get, type Writable, type Readable } from 'svelte/store';
import { api } from './api';
import type { FileDetails, Track, MidiDevice, SearchFilters, TrackProperties, PlaybackPosition } from './api';

// ============ FILE & SELECTION ============

export interface FileData {
  id: number;
  filename: string;
  filepath: string;
  bpm: number;
  key_signature: string;
  duration: number;
  time_signature: string;
  category: string;
}

// Single file selection
export const selectedFile: Writable<FileDetails | null> = writable(null);

// Selected pattern for piano roll editing
export interface MidiPattern {
  events: any[];
  ticks_per_beat: number;
}
export const selectedPattern: Writable<MidiPattern | null> = writable(null);

// Multiple file selection
export const selectedFiles: Writable<number[]> = writable([]);

// Recent files history (max 20)
export const recentFiles: Writable<FileDetails[]> = writable([]);

// Add file to recent history
export function addToRecent(file: FileDetails) {
  recentFiles.update(files => {
    const filtered = files.filter(f => f.id !== file.id);
    const updated = [file, ...filtered].slice(0, 20);
    saveRecentFiles(updated);
    return updated;
  });
}

// Clear recent files
export function clearRecent() {
  recentFiles.set([]);
  localStorage.removeItem('recent_files');
}

// ============ SEARCH & FILTERS ============

export const searchQuery: Writable<string> = writable('');

export const searchFilters: Writable<SearchFilters> = writable({
  sort_by: 'relevance',
  sort_desc: true,
  limit: 50,
  offset: 0,
});

export const searchResults: Writable<FileDetails[]> = writable([]);
export const searchTotal: Writable<number> = writable(0);
export const searchLoading: Writable<boolean> = writable(false);

// Perform search with current filters
export async function performSearch(additionalFilters?: Partial<SearchFilters>) {
  const filters = { ...get(searchFilters), ...additionalFilters };
  searchLoading.set(true);

  try {
    const results = await api.search.files(filters);
    searchResults.set(results.files);
    searchTotal.set(results.total);
    searchFilters.update(f => ({ ...f, ...additionalFilters }));
  } catch (error) {
    console.error('Search failed:', error);
    notify('error', `Search failed: ${error}`);
  } finally {
    searchLoading.set(false);
  }
}

// ============ FAVORITES ============

export const favorites: Writable<number[]> = writable([]);

// Load favorites from backend
export async function loadFavorites() {
  try {
    const favFiles = await api.favorites.getAll();
    favorites.set(favFiles.map(f => f.id));
  } catch (error) {
    console.error('Failed to load favorites:', error);
    notify('error', 'Failed to load favorites');
  }
}

// Toggle favorite status
export async function toggleFavorite(fileId: number) {
  const current = get(favorites);
  const isFavorite = current.includes(fileId);

  try {
    if (isFavorite) {
      await api.favorites.remove(fileId);
      favorites.update(f => f.filter(id => id !== fileId));
      notify('info', 'Removed from favorites');
    } else {
      await api.favorites.add(fileId);
      favorites.update(f => [...f, fileId]);
      notify('success', 'Added to favorites');
    }
  } catch (error) {
    console.error('Failed to toggle favorite:', error);
    notify('error', 'Failed to update favorite status');
  }
}

// Derived: is current file favorited?
export const isCurrentFileFavorited: Readable<boolean> = derived(
  [selectedFile, favorites],
  ([$selectedFile, $favorites]) =>
    $selectedFile ? $favorites.includes($selectedFile.id) : false
);

// ============ MIDI DEVICES ============

export const midiDevices: Writable<MidiDevice[]> = writable([]);
export const connectedDevice: Writable<MidiDevice | null> = writable(null);
export const midiConnected: Writable<boolean> = writable(false);

// Refresh MIDI devices list
export async function refreshMidiDevices() {
  try {
    const devices = await api.midi.listDevices();
    midiDevices.set(devices);

    const connected = await api.midi.isConnected();
    midiConnected.set(connected);

    if (connected) {
      const current = await api.midi.getCurrentDevice();
      connectedDevice.set(current);
    } else {
      connectedDevice.set(null);
    }
  } catch (error) {
    console.error('Failed to refresh MIDI devices:', error);
  }
}

// Connect to MIDI device
export async function connectMidiDevice(deviceName: string) {
  try {
    await api.midi.connect(deviceName);
    await refreshMidiDevices();
    notify('success', `Connected to ${deviceName}`);
  } catch (error) {
    console.error('Failed to connect:', error);
    notify('error', `Failed to connect: ${error}`);
  }
}

// Disconnect MIDI device
export async function disconnectMidiDevice() {
  try {
    await api.midi.disconnect();
    connectedDevice.set(null);
    midiConnected.set(false);
    notify('info', 'MIDI device disconnected');
  } catch (error) {
    console.error('Failed to disconnect:', error);
    notify('error', 'Failed to disconnect');
  }
}

// ============ SEQUENCER & PLAYBACK ============

export const isPlaying: Writable<boolean> = writable(false);
export const isPaused: Writable<boolean> = writable(false);
export const playbackPosition: Writable<PlaybackPosition> = writable({
  current_tick: 0,
  current_bar: 0,
  current_beat: 0,
});
export const tempo: Writable<number> = writable(120);
export const timeSignature: Writable<{ numerator: number; denominator: number }> = writable({
  numerator: 4,
  denominator: 4
});

// Tracks
export const tracks: Writable<Track[]> = writable([]);

// Load tracks from sequencer
export async function loadTracks() {
  try {
    const trackList = await api.sequencer.getTracks();
    tracks.set(trackList);
  } catch (error) {
    console.error('Failed to load tracks:', error);
    notify('error', 'Failed to load tracks');
  }
}

// Add track to sequencer
export async function addTrack(fileId: number, channel: number = 0) {
  try {
    await api.sequencer.addTrack(fileId, channel);
    await loadTracks();
    notify('success', 'Track added');
  } catch (error) {
    console.error('Failed to add track:', error);
    notify('error', 'Failed to add track');
  }
}

// Remove track from sequencer
export async function removeTrack(trackId: number) {
  try {
    await api.sequencer.removeTrack(trackId);
    await loadTracks();
    notify('info', 'Track removed');
  } catch (error) {
    console.error('Failed to remove track:', error);
    notify('error', 'Failed to remove track');
  }
}

// Update track properties
export async function updateTrack(trackId: number, properties: TrackProperties) {
  try {
    await api.sequencer.updateTrack(trackId, properties);
    await loadTracks();
  } catch (error) {
    console.error('Failed to update track:', error);
    notify('error', 'Failed to update track');
  }
}

// Derived: has any solo tracks?
export const hasSoloTracks: Readable<boolean> = derived(
  tracks,
  $tracks => $tracks.some(t => t.solo)
);

// Derived: active tracks (considering solo/mute)
export const activeTracks: Readable<Track[]> = derived(
  tracks,
  $tracks => {
    const hasSolo = $tracks.some(t => t.solo);
    if (hasSolo) {
      return $tracks.filter(t => t.solo);
    }
    return $tracks.filter(t => !t.muted);
  }
);

// ============ UI STATE ============

export const currentView: Writable<'search' | 'piano-roll' | 'sequencer'> = writable('search');
export const viewMode: Writable<'grid' | 'list'> = writable('grid');
export const sidebarOpen: Writable<boolean> = writable(true);
export const detailsPanelOpen: Writable<boolean> = writable(true);
export const zoom: Writable<{ horizontal: number; vertical: number }> = writable({
  horizontal: 1,
  vertical: 1
});

// ============ NOTIFICATIONS ============

export interface Notification {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  message: string;
  duration?: number;
}

export const notifications: Writable<Notification[]> = writable([]);

// Add notification
export function notify(type: Notification['type'], message: string, duration = 3000) {
  const id = `notif-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  const notification: Notification = { id, type, message, duration };

  notifications.update(n => [...n, notification]);

  if (duration > 0) {
    setTimeout(() => {
      notifications.update(n => n.filter(notif => notif.id !== id));
    }, duration);
  }
}

// Remove notification
export function removeNotification(id: string) {
  notifications.update(n => n.filter(notif => notif.id !== id));
}

// ============ PLAYBACK CONTROL ============

// Toggle play/pause
export async function togglePlayback() {
  const playing = get(isPlaying);
  const paused = get(isPaused);

  try {
    if (playing) {
      await api.sequencer.pause();
      isPlaying.set(false);
      isPaused.set(true);
    } else {
      if (paused) {
        await api.sequencer.resume();
      } else {
        await api.sequencer.start();
      }
      isPlaying.set(true);
      isPaused.set(false);
    }
  } catch (error) {
    console.error('Playback toggle failed:', error);
    notify('error', 'Playback control failed');
  }
}

// Stop playback
export async function stopPlayback() {
  try {
    await api.sequencer.stop();
    isPlaying.set(false);
    isPaused.set(false);
    playbackPosition.set({
      current_tick: 0,
      current_bar: 0,
      current_beat: 0,
    });
  } catch (error) {
    console.error('Stop failed:', error);
    notify('error', 'Stop failed');
  }
}

// Update playback position
export async function updatePlaybackPosition() {
  try {
    const position = await api.sequencer.getPosition();
    playbackPosition.set(position);
  } catch (error) {
    console.error('Failed to get position:', error);
  }
}

// Set tempo
export async function setTempo(bpm: number) {
  try {
    await api.playback.setTempo(bpm);
    tempo.set(bpm);
  } catch (error) {
    console.error('Failed to set tempo:', error);
    notify('error', 'Failed to set tempo');
  }
}

// ============ PERSISTENCE ============

// Save UI preferences to localStorage
export function savePreferences() {
  const prefs = {
    sidebarOpen: get(sidebarOpen),
    detailsPanelOpen: get(detailsPanelOpen),
    zoom: get(zoom),
    currentView: get(currentView),
  };

  try {
    localStorage.setItem('ui_preferences', JSON.stringify(prefs));
  } catch (error) {
    console.error('Failed to save preferences:', error);
  }
}

// Load UI preferences from localStorage
export function loadPreferences() {
  try {
    const saved = localStorage.getItem('ui_preferences');
    if (saved) {
      const prefs = JSON.parse(saved);
      sidebarOpen.set(prefs.sidebarOpen ?? true);
      detailsPanelOpen.set(prefs.detailsPanelOpen ?? true);
      zoom.set(prefs.zoom ?? { horizontal: 1, vertical: 1 });
      currentView.set(prefs.currentView ?? 'search');
    }
  } catch (error) {
    console.error('Failed to load preferences:', error);
  }
}

// Save recent files to localStorage
function saveRecentFiles(files: FileDetails[]) {
  try {
    // Store only essential data to avoid localStorage quota
    const simplified = files.map(f => ({
      id: f.id,
      file_name: f.file_name,
      bpm: f.bpm,
      key: f.key,
      category: f.category,
    }));
    localStorage.setItem('recent_files', JSON.stringify(simplified));
  } catch (error) {
    console.error('Failed to save recent files:', error);
  }
}

// Load recent files from localStorage
export async function loadRecentFiles() {
  try {
    const saved = localStorage.getItem('recent_files');
    if (saved) {
      const simplified = JSON.parse(saved);
      // Fetch full details for recent files
      const fullDetails = await Promise.all(
        simplified.map((s: any) =>
          api.files.getDetails(s.id).catch(() => null)
        )
      );
      recentFiles.set(fullDetails.filter(f => f !== null) as FileDetails[]);
    }
  } catch (error) {
    console.error('Failed to load recent files:', error);
  }
}

// ============ INITIALIZATION ============

// Initialize all stores on app mount
export async function initializeStores() {
  console.log('Initializing stores...');

  // Load saved preferences
  loadPreferences();

  // Load data from backend
  await Promise.allSettled([
    loadFavorites(),
    refreshMidiDevices(),
    loadTracks(),
    loadRecentFiles(),
  ]);

  console.log('Stores initialized');
}

// Setup periodic updates
export function startPeriodicUpdates() {
  // Refresh MIDI devices every 2 seconds
  const midiInterval = setInterval(refreshMidiDevices, 2000);

  // Update playback position every 100ms when playing
  const positionInterval = setInterval(() => {
    if (get(isPlaying)) {
      updatePlaybackPosition();
    }
  }, 100);

  // Return cleanup function
  return () => {
    clearInterval(midiInterval);
    clearInterval(positionInterval);
  };
}

// Auto-save preferences when they change
sidebarOpen.subscribe(() => savePreferences());
detailsPanelOpen.subscribe(() => savePreferences());
zoom.subscribe(() => savePreferences());
currentView.subscribe(() => savePreferences());

// ============ DERIVED STORES ============

// Has active selection
export const hasSelectedFile: Readable<boolean> = derived(
  selectedFile,
  $selectedFile => $selectedFile !== null
);

// Has tracks in sequencer
export const hasTracks: Readable<boolean> = derived(
  tracks,
  $tracks => $tracks.length > 0
);

// Can play (has tracks and MIDI connected)
export const canPlay: Readable<boolean> = derived(
  [hasTracks, midiConnected],
  ([$hasTracks, $midiConnected]) => $hasTracks && $midiConnected
);

// Search has results
export const hasSearchResults: Readable<boolean> = derived(
  searchResults,
  $results => $results.length > 0
);

// Has favorites
export const hasFavorites: Readable<boolean> = derived(
  favorites,
  $favorites => $favorites.length > 0
);
