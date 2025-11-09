/**
 * Tag Utility Functions
 *
 * ARCHETYPE: TRUSTY MODULE (Pure functions)
 * Purpose: Pure utility functions for tag operations
 */

import type { EnhancedTag, TagCategory, TagCloudConfig } from '../types/tagSuggestions';

/**
 * Get category color as hex string
 * Matches colors from migration 007_enhanced_tags.sql
 */
export function getCategoryColor(category?: string): string {
  const colors: Record<string, string> = {
    genre: '#3498db',      // blue
    instrument: '#2ecc71',  // green
    element: '#e67e22',     // orange
    key: '#9b59b6',         // purple
    tempo: '#e74c3c',       // red
    mood: '#f39c12',        // orange
    technical: '#95a5a6',   // gray
    structure: '#1abc9c',   // teal
    library: '#34495e',     // dark gray
    world: '#d35400'        // dark orange
  };
  return colors[category || ''] || '#95a5a6'; // Default gray
}

/**
 * Get category priority (lower = higher priority)
 */
export function getCategoryPriority(category?: string): number {
  const priorities: Record<string, number> = {
    genre: 10,
    instrument: 20,
    element: 30,
    key: 40,
    tempo: 50,
    mood: 60,
    technical: 70,
    structure: 80,
    library: 85,
    world: 90
  };
  return priorities[category || ''] || 100;
}

/**
 * Calculate font size for tag cloud based on usage count and confidence
 */
export function calculateTagFontSize(
  tag: EnhancedTag,
  maxUsageCount: number,
  config: TagCloudConfig
): number {
  const { minFontSize, maxFontSize } = config;

  // Weight by both usage count (70%) and confidence (30%)
  const usageRatio = maxUsageCount > 0 ? tag.usageCount / maxUsageCount : 0;
  const confidenceRatio = tag.confidenceScore;
  const combinedRatio = usageRatio * 0.7 + confidenceRatio * 0.3;

  // Use logarithmic scale for better visual distribution
  const logRatio = Math.log(combinedRatio * 10 + 1) / Math.log(11);

  return Math.round(minFontSize + (maxFontSize - minFontSize) * logRatio);
}

/**
 * Calculate opacity based on confidence score
 */
export function calculateConfidenceOpacity(confidence: number): number {
  // Map confidence (0.60-0.95) to opacity (0.6-1.0)
  const minConfidence = 0.60;
  const maxConfidence = 0.95;
  const minOpacity = 0.6;
  const maxOpacity = 1.0;

  const normalized = (confidence - minConfidence) / (maxConfidence - minConfidence);
  const clamped = Math.max(0, Math.min(1, normalized));

  return minOpacity + (maxOpacity - minOpacity) * clamped;
}

/**
 * Format confidence as percentage string
 */
export function formatConfidence(confidence: number): string {
  return `${(confidence * 100).toFixed(0)}%`;
}

/**
 * Sort tags by priority, confidence, and name
 */
export function sortTags(
  tags: EnhancedTag[],
  sortBy: 'priority' | 'confidence' | 'name' | 'usage'
): EnhancedTag[] {
  const sorted = [...tags];

  switch (sortBy) {
    case 'priority':
      sorted.sort((a, b) => {
        // First by category priority
        const catPriorityA = getCategoryPriority(a.category);
        const catPriorityB = getCategoryPriority(b.category);
        if (catPriorityA !== catPriorityB) return catPriorityA - catPriorityB;

        // Then by tag priority
        if (a.priority !== b.priority) return a.priority - b.priority;

        // Finally by usage count (descending)
        return b.usageCount - a.usageCount;
      });
      break;

    case 'confidence':
      sorted.sort((a, b) => {
        if (a.confidenceScore !== b.confidenceScore) {
          return b.confidenceScore - a.confidenceScore;
        }
        return b.usageCount - a.usageCount;
      });
      break;

    case 'name':
      sorted.sort((a, b) => a.name.localeCompare(b.name));
      break;

    case 'usage':
      sorted.sort((a, b) => {
        if (a.usageCount !== b.usageCount) {
          return b.usageCount - a.usageCount;
        }
        return b.confidenceScore - a.confidenceScore;
      });
      break;
  }

  return sorted;
}

/**
 * Group tags by category
 */
export function groupTagsByCategory(tags: EnhancedTag[]): Map<string, EnhancedTag[]> {
  const grouped = new Map<string, EnhancedTag[]>();

  for (const tag of tags) {
    const category = tag.category || 'other';
    if (!grouped.has(category)) {
      grouped.set(category, []);
    }
    grouped.get(category)!.push(tag);
  }

  // Sort categories by priority
  const sortedCategories = Array.from(grouped.keys()).sort(
    (a, b) => getCategoryPriority(a) - getCategoryPriority(b)
  );

  const result = new Map<string, EnhancedTag[]>();
  for (const category of sortedCategories) {
    result.set(category, grouped.get(category)!);
  }

  return result;
}

/**
 * Filter tags by search query
 */
export function filterTagsBySearch(tags: EnhancedTag[], query: string): EnhancedTag[] {
  if (!query || query.length < 2) return tags;

  const lowerQuery = query.toLowerCase().trim();

  return tags.filter(
    (tag) =>
      tag.name.toLowerCase().includes(lowerQuery) ||
      tag.category?.toLowerCase().includes(lowerQuery)
  );
}

/**
 * Filter tags by category names
 */
export function filterTagsByCategories(
  tags: EnhancedTag[],
  categories: string[]
): EnhancedTag[] {
  if (categories.length === 0) return tags;

  return tags.filter((tag) => tag.category && categories.includes(tag.category));
}

/**
 * Filter tags by confidence range
 */
export function filterTagsByConfidence(
  tags: EnhancedTag[],
  minConfidence?: number,
  maxConfidence?: number
): EnhancedTag[] {
  return tags.filter((tag) => {
    if (minConfidence !== undefined && tag.confidenceScore < minConfidence) return false;
    if (maxConfidence !== undefined && tag.confidenceScore > maxConfidence) return false;
    return true;
  });
}

/**
 * Get confidence level label
 */
export function getConfidenceLevel(confidence: number): 'high' | 'medium' | 'low' {
  if (confidence >= 0.85) return 'high';
  if (confidence >= 0.70) return 'medium';
  return 'low';
}

/**
 * Get confidence level color
 */
export function getConfidenceLevelColor(confidence: number): string {
  const level = getConfidenceLevel(confidence);
  switch (level) {
    case 'high':
      return '#4caf50'; // Green
    case 'medium':
      return '#ff9800'; // Orange
    case 'low':
      return '#f44336'; // Red
  }
}

/**
 * Get detection method icon
 */
export function getDetectionMethodIcon(method?: string): string {
  const icons: Record<string, string> = {
    pack_exact: '📦',
    folder_exact: '📁',
    filename_pattern: '🔤',
    bpm_detection: '⏱️',
    key_detection: '🎵',
    contextual: '🧠',
    mood_inference: '😊'
  };
  return icons[method || ''] || '🤖';
}

/**
 * Validate tag name format
 */
export function isValidTagName(name: string): boolean {
  // Tag names should be lowercase, alphanumeric with hyphens
  return /^[a-z0-9-]+$/.test(name);
}

/**
 * Normalize tag name
 */
export function normalizeTagName(name: string): string {
  return name
    .toLowerCase()
    .trim()
    .replace(/\s+/g, '-')
    .replace(/[^a-z0-9-]/g, '');
}

/**
 * Calculate total confidence for multiple tags
 */
export function calculateCombinedConfidence(tags: EnhancedTag[]): number {
  if (tags.length === 0) return 0;

  // Use weighted average based on usage count
  const totalWeight = tags.reduce((sum, tag) => sum + tag.usageCount, 0);
  if (totalWeight === 0) {
    // Fall back to simple average
    return tags.reduce((sum, tag) => sum + tag.confidenceScore, 0) / tags.length;
  }

  const weightedSum = tags.reduce(
    (sum, tag) => sum + tag.confidenceScore * tag.usageCount,
    0
  );

  return weightedSum / totalWeight;
}
