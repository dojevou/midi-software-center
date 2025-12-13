// Barrel exports for all stores

export * from './playbackStore';
export * from './projectStore';
export * from './databaseStore';
export * from './uiStore';

export { pipelineStore, pipelineActions } from './pipelineStore';
export { analysisStore, analysisActions } from './analysisStore';
export { archiveStore, archiveActions } from './archiveStore';

// New stores for window support
export { tagStore, tagActions, filteredTags, selectedTagCount, hasSelectedTags } from './tagStore';
export {
  pianoRollStore,
  pianoRollActions,
  hasSelection as pianoRollHasSelection,
  sortedNotes,
} from './pianoRollStore';
export {
  automationStore,
  automationActions,
  visibleLanes,
  selectedLaneData,
  hasPointSelection,
} from './automationStore';
export {
  favoritesStore,
  favoritesActions,
  filteredItems,
  itemCount,
  hasSelection as favoritesHasSelection,
} from './favoritesStore';
export {
  settingsStore,
  settingsActions,
  hasPendingChanges,
  currentTheme,
  mergedSettings,
} from './settingsStore';

// Preferences, Gear, and Presets stores (Nov 2025)
export {
  preferencesStore,
  preferencesActions,
  settingsCategories,
  settingsCount,
  shortcutCategories,
  shortcutCount,
  layoutNames,
  layoutCount,
  recentProjectCount,
  pinnedProjectCount,
} from './preferencesStore';
export {
  gearStore,
  gearActions,
  profileCount,
  userGearCount,
  filteredProfiles,
  filteredUserGear,
} from './gearStore';
export {
  presetsStore,
  presetsActions,
  mixerPresetCount,
  trackTemplateCount,
  projectTemplateCount,
  filteredMixerPresets,
  filteredTrackTemplates,
  filteredProjectTemplates,
} from './presetsStore';

// VIP3 Browser stores (Dec 2025)
export {
  browserStore,
  browserActions,
  browserResults,
  browserFilters,
  browserIsLoading,
  browserError,
  selectedFiles,
  previewFile,
} from './browserStore';
export {
  collectionStore,
  collectionActions,
  collections,
  selectedCollection,
  collectionIsLoading,
  collectionError,
  smartCollections,
  regularCollections,
} from './collectionStore';
export {
  categoryStore,
  categoryActions,
  timbres,
  styles,
  styleHierarchy,
  articulations,
  bpmRanges,
  musicalKeys,
  categoryCounts,
  categoryIsLoading,
  categoryError,
  timbresSorted,
  stylesSorted,
  articulationsSorted,
  majorKeys,
  minorKeys,
} from './categoryStore';

// VIP3 Store (unified VIP3 state management)
export {
  vip3Store,
  vip3Actions,
  selectedTimbres,
  selectedStyles,
  selectedArticulations,
  selectedBpmRange,
  selectedKeys,
  hasActiveFilters,
  filterSummary,
  pageCount,
} from './vip3Store';

// Sequencer/Timeline stores (Dec 2025)
export {
  sequencerStore,
  sequencerActions,
  projectName as sequencerProjectName,
  isProjectDirty,
  currentBpm,
  currentTimeSignature,
  isPlaying,
  isRecording,
  formattedPlayhead,
  loopEnabled,
  loopRange,
  selectedTracks,
  selectedClips,
  hasSoloedTracks,
  visibleTracks,
  totalProjectLength,
  pixelsPerTick,
  ticksPerPixel,
} from './sequencerStore';

// Theming & Accessibility stores (Dec 2025)
export {
  themeStore,
  isDarkMode,
  themeMode,
  reducedMotion,
  type ThemeMode,
  type ThemeColors,
  type ThemeConfig,
  type ThemeState,
} from './themeStore';
export {
  a11yStore,
  isKeyboardNav,
  isFocusTrapped,
  type A11yState,
  type A11yAnnouncement,
} from './a11yStore';

// Keyboard Shortcuts store (Dec 2025)
export {
  keyboardStore,
  shortcutsByCategory,
  hasConflicts,
  comboToString,
  matchesCombo,
  type KeyCombo,
  type ShortcutAction,
  type ShortcutCategory,
} from './keyboardStore';

// Drag and Drop store (Dec 2025)
export {
  dndStore,
  isDragging,
  dragType,
  type DragSource,
  type DropTarget,
  type DragData,
  type DragDataType,
  type MidiFileDragPayload,
  type TrackDragPayload,
  type NoteDragPayload,
  type DropZone,
} from './dndStore';

// Ableton Link store (Dec 2025)
export {
  linkState,
  linkEnabled,
  linkPeers,
  linkActions,
  type LinkState,
} from './linkStore';

// MIDI Learn store (Dec 2025)
export {
  learnState,
  isLearning,
  mappingCount,
  learnActions,
  type MidiMapping,
} from './learnStore';

// Notation store (Dec 2025)
export { notationState, notationActions } from './notationStore';

// Scripting store (Dec 2025)
export {
  scriptingState,
  loadedScripts,
  availableActions,
  scriptingActions,
  type ScriptAction,
  type ScriptInfo,
} from './scriptingStore';
