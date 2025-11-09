/**
 * Type definitions for tag suggestions system
 *
 * ARCHETYPE: TRUSTY MODULE (Type definitions)
 * Purpose: Type-safe interfaces for enhanced tag suggestion features
 */

/**
 * Tag category information from database
 */
export interface TagCategory {
  id: number;
  name: string;
  description: string;
  priority: number;
  color: string;
  createdAt: string;
}

/**
 * Enhanced tag with confidence and priority
 */
export interface EnhancedTag {
  id: number;
  name: string;
  categoryId?: number;
  category?: string;
  categoryColor?: string;
  priority: number;
  autoDetected: boolean;
  confidenceScore: number;
  detectionMethod?: string;
  usageCount: number;
  isActive: boolean;
}

/**
 * Tag suggestion for a specific file
 */
export interface TagSuggestion {
  id: number;
  fileId: number;
  suggestedTagId: number;
  suggestedTag: EnhancedTag;
  confidence: number;
  source: TagSuggestionSource;
  isAccepted?: boolean;
  acceptedAt?: string;
  createdAt: string;
}

/**
 * Source of tag suggestion
 */
export type TagSuggestionSource =
  | 'auto'           // Auto-detected from filename/path
  | 'ml'             // Machine learning model
  | 'user_feedback'  // Based on user patterns
  | 'similar_files'; // From similar MIDI files

/**
 * Tag suggestion filters
 */
export interface TagSuggestionFilters {
  minConfidence?: number;
  maxConfidence?: number;
  categories?: string[];
  sources?: TagSuggestionSource[];
  search?: string;
  onlyPending?: boolean;
}

/**
 * Tag suggestion action
 */
export type TagSuggestionAction = 'accept' | 'reject' | 'defer';

/**
 * Batch tag suggestion operation
 */
export interface BatchTagOperation {
  suggestionIds: number[];
  action: TagSuggestionAction;
}

/**
 * Tag suggestion state
 */
export interface TagSuggestionState {
  suggestions: TagSuggestion[];
  categories: TagCategory[];
  filters: TagSuggestionFilters;
  selectedSuggestionIds: number[];
  loading: boolean;
  error: string | null;
}

/**
 * Tag cloud configuration
 */
export interface TagCloudConfig {
  minFontSize: number;
  maxFontSize: number;
  minConfidence: number;
  sortBy: 'priority' | 'confidence' | 'name' | 'usage';
  groupByCategory: boolean;
}
