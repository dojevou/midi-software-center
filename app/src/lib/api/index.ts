/**
 * API Module Index
 *
 * Re-exports all API-related types and utilities.
 */

// Command registry and type-safe invoker
export {
  Commands,
  CommandInvoker,
  cmd,
  type CommandName,
  getAllCommandNames,
  getCommandCount,
  isValidCommand,
  getCommandsByCategory,
  printCommandSummary,
} from './commands';

// VIP3 Browser APIs
export { timbresApi, type CreateTimbreParams, type UpdateTimbreParams, type TimbreCounts } from './timbres';
export { stylesApi, type CreateStyleParams, type UpdateStyleParams, type StyleWithChildren, type StyleCounts } from './styles';
export { articulationsApi, type CreateArticulationParams, type UpdateArticulationParams, type ArticulationCounts } from './articulations';
export { collectionsApi, type CreateCollectionParams, type CreateSmartCollectionParams, type UpdateCollectionParams, type CollectionExportOptions } from './collections';
export { savedSearchesApi, type CreateSavedSearchParams, type UpdateSavedSearchParams } from './savedSearches';
export { vip3BrowserApi, type VIP3Categories, type SearchSuggestions, type VIP3FilterOptions } from './vip3Browser';
