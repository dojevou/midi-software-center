# Tag Suggestions UI Component

A comprehensive Svelte component for displaying and managing tag suggestions with confidence scores, built for the MIDI Software Center's enhanced auto-tagging system.

## Architecture

This implementation follows the **Three Archetypes Pattern**:

### 1. Trusty Module (Pure Functions)
**File:** `src/lib/utils/tagUtils.ts`

Pure utility functions with no side effects:
- `getCategoryColor()` - Get category color hex codes
- `getCategoryPriority()` - Get category priority values
- `calculateTagFontSize()` - Calculate font size for tag cloud
- `calculateConfidenceOpacity()` - Calculate opacity from confidence score
- `sortTags()` - Sort tags by various criteria
- `groupTagsByCategory()` - Group tags by category
- `filterTagsBySearch()` - Filter tags by search query
- `formatConfidence()` - Format confidence as percentage
- `getConfidenceLevel()` - Get confidence level (high/medium/low)
- `normalizeTagName()` - Normalize tag names
- And more...

### 2. Grown-up Script (State Management)
**File:** `src/lib/stores/tagSuggestions.ts`

State management with Tauri IPC and side effects:
- Writable stores for suggestions, categories, filters, selection
- Derived stores for filtered results, grouped tags, stats
- Async actions for fetching/modifying suggestions
- Selection management (toggle, select all, clear)
- Filter management (search, category, confidence)

### 3. Task-O-Matic (UI Component)
**File:** `src/lib/components/TagSuggestions.svelte`

Complete UI with user interactions:
- Tag cloud and list views
- Confidence bars and visual indicators
- Multi-select with batch operations
- Search and filtering
- Responsive and accessible

## Type Definitions

**File:** `src/lib/types/tagSuggestions.ts`

```typescript
interface EnhancedTag {
  id: number;
  name: string;
  categoryId?: number;
  category?: string;
  categoryColor?: string;
  priority: number;
  autoDetected: boolean;
  confidenceScore: number; // 0.00-1.00
  detectionMethod?: string;
  usageCount: number;
  isActive: boolean;
}

interface TagSuggestion {
  id: number;
  fileId: number;
  suggestedTagId: number;
  suggestedTag: EnhancedTag;
  confidence: number;
  source: 'auto' | 'ml' | 'user_feedback' | 'similar_files';
  isAccepted?: boolean;
  acceptedAt?: string;
  createdAt: string;
}

interface TagCategory {
  id: number;
  name: string;
  description: string;
  priority: number; // 10-90 (lower = higher priority)
  color: string; // Hex color
  createdAt: string;
}
```

## Category Colors (from Migration 007)

```typescript
const categoryColors = {
  genre: '#3498db',      // Blue
  instrument: '#2ecc71',  // Green
  element: '#e67e22',     // Orange
  key: '#9b59b6',         // Purple
  tempo: '#e74c3c',       // Red
  mood: '#f39c12',        // Orange
  technical: '#95a5a6',   // Gray
  structure: '#1abc9c',   // Teal
  library: '#34495e',     // Dark gray
  world: '#d35400'        // Dark orange
};
```

## Component Props

```typescript
export let fileId: number | null = null;        // File ID (null = all pending)
export let showBatchActions: boolean = true;    // Show batch controls
export let allowGenerate: boolean = true;       // Show generate button
export let maxHeight: string = '600px';         // Max scrollable height
export let compact: boolean = false;            // Compact mode
```

## Features

### 1. Dual View Modes

**List View (Default):**
- Grouped by category
- Detailed confidence bars
- Detection method icons
- Individual accept/reject buttons
- Checkbox selection

**Cloud View:**
- Visual tag cloud
- Font size based on confidence + usage
- Color coded by category
- Opacity based on confidence
- Hover effects

### 2. Filtering & Search

- **Search:** Real-time text search on tag names
- **Category Filter:** Multi-select category chips
- **Confidence Filter:** Slider for minimum confidence (0.5-0.95)
- **Pending Only:** Toggle to show only unreviewed suggestions

### 3. Selection & Batch Operations

- Individual checkbox selection
- "Select All" filtered suggestions
- "Select High Confidence" (≥85%)
- Select by category
- Batch accept/reject selected suggestions
- Selection count indicator

### 4. Confidence Indicators

**Visual Elements:**
- Colored confidence bars (green/orange/red)
- Percentage display
- Opacity based on confidence
- Level badges (High/Medium/Low)

**Thresholds:**
- High: ≥85% (Green #4caf50)
- Medium: 70-84% (Orange #ff9800)
- Low: <70% (Red #f44336)

### 5. Statistics Bar

Real-time stats showing:
- High confidence count
- Medium confidence count
- Low confidence count

### 6. Accessibility

- Proper ARIA labels on all interactive elements
- Keyboard navigation support
- Screen reader friendly
- Focus management
- Semantic HTML

### 7. Animations

Smooth transitions using Svelte's built-in animations:
- Slide transitions for sections
- Fade transitions for cards
- Easing functions (quintOut)
- Smooth confidence bar animations

## Usage Examples

### Basic Usage (File-Specific)

```svelte
<script>
  import TagSuggestions from '$lib/components/TagSuggestions.svelte';

  let fileId = 123;
</script>

<TagSuggestions
  fileId={fileId}
  showBatchActions={true}
  allowGenerate={true}
/>
```

### Compact Sidebar Widget

```svelte
<TagSuggestions
  fileId={fileId}
  compact={true}
  showBatchActions={false}
  allowGenerate={false}
  maxHeight="400px"
/>
```

### Bulk Review Mode

```svelte
<TagSuggestions
  fileId={null}
  showBatchActions={true}
  allowGenerate={false}
  maxHeight="600px"
/>
```

### Integration with File Browser

```svelte
<script>
  import TagSuggestions from '$lib/components/TagSuggestions.svelte';
  import { selectedFileStore } from '$lib/stores/files';

  $: selectedFile = $selectedFileStore;
</script>

<div class="file-details">
  {#if selectedFile}
    <h2>{selectedFile.filename}</h2>

    <section class="tags-section">
      <h3>Suggested Tags</h3>
      <TagSuggestions
        fileId={selectedFile.id}
        compact={true}
        maxHeight="400px"
      />
    </section>
  {/if}
</div>
```

## Required Tauri Commands

The component requires these backend commands:

```rust
// Get all tag categories
#[tauri::command]
async fn get_tag_categories(state: State<'_, AppState>) -> Result<Vec<TagCategory>, String>

// Get suggestions for a specific file
#[tauri::command]
async fn get_tag_suggestions(
    file_id: i64,
    state: State<'_, AppState>
) -> Result<Vec<TagSuggestion>, String>

// Generate new suggestions using auto-tagging
#[tauri::command]
async fn generate_tag_suggestions(
    file_id: i64,
    state: State<'_, AppState>
) -> Result<(), String>

// Get all pending suggestions (isAccepted = NULL)
#[tauri::command]
async fn get_pending_tag_suggestions(
    limit: i32,
    state: State<'_, AppState>
) -> Result<Vec<TagSuggestion>, String>

// Accept a suggestion (applies tag to file)
#[tauri::command]
async fn accept_tag_suggestion(
    suggestion_id: i32,
    state: State<'_, AppState>
) -> Result<(), String>

// Reject a suggestion
#[tauri::command]
async fn reject_tag_suggestion(
    suggestion_id: i32,
    state: State<'_, AppState>
) -> Result<(), String>

// Batch process suggestions
#[tauri::command]
async fn batch_process_tag_suggestions(
    suggestion_ids: Vec<i32>,
    action: String, // "accept" | "reject" | "defer"
    state: State<'_, AppState>
) -> Result<(), String>
```

## Database Schema

Based on migration `007_enhanced_tags.sql`:

```sql
-- Tag categories
CREATE TABLE tag_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 50,
    color VARCHAR(7),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enhanced tags table
ALTER TABLE tags
    ADD COLUMN category_id INTEGER REFERENCES tag_categories(id),
    ADD COLUMN priority INTEGER DEFAULT 50,
    ADD COLUMN auto_detected BOOLEAN DEFAULT FALSE,
    ADD COLUMN confidence_score DECIMAL(3,2) DEFAULT 0.00,
    ADD COLUMN detection_method VARCHAR(50),
    ADD COLUMN parent_tag_id INTEGER REFERENCES tags(id),
    ADD COLUMN is_active BOOLEAN DEFAULT TRUE;

-- Tag suggestions
CREATE TABLE tag_suggestions (
    id SERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id),
    suggested_tag_id INTEGER NOT NULL REFERENCES tags(id),
    confidence DECIMAL(3,2) NOT NULL,
    source VARCHAR(50) NOT NULL,
    is_accepted BOOLEAN,
    accepted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(file_id, suggested_tag_id)
);
```

## Store API

Direct access to the store for advanced use cases:

```typescript
import {
  tagSuggestionsState,
  filteredSuggestions,
  uniqueTags,
  tagsByCategory,
  selectedSuggestions,
  suggestionStats,
  fetchTagSuggestions,
  generateTagSuggestions,
  acceptSuggestion,
  rejectSuggestion,
  batchProcessSuggestions,
  toggleSuggestionSelection,
  selectAllSuggestions,
  clearSelection,
  updateFilters,
  setSearchQuery
} from '$lib/stores/tagSuggestions';

// Subscribe to filtered suggestions
$: suggestions = $filteredSuggestions;

// Fetch suggestions for a file
await fetchTagSuggestions(fileId);

// Generate suggestions
await generateTagSuggestions(fileId);

// Accept individual suggestion
await acceptSuggestion(suggestionId);

// Batch accept selected
const selected = $selectedSuggestions;
await batchProcessSuggestions(
  selected.map(s => s.id),
  'accept'
);

// Update filters
updateFilters({
  minConfidence: 0.85,
  categories: ['genre', 'instrument']
});
```

## Styling & Theming

Uses CSS custom properties for theming:

```css
--color-surface: #2d2d2d;      /* Component background */
--color-bg: #1e1e1e;           /* Section backgrounds */
--color-border: #3d3d3d;       /* Borders */
--color-text: #e5e5e5;         /* Primary text */
--color-text-secondary: #a3a3a3; /* Secondary text */
--color-primary: #3b82f6;      /* Primary accent */
```

## Performance Considerations

1. **Derived Stores:** Automatic memoization of filtered/grouped data
2. **Reactive Updates:** Only re-renders changed sections
3. **Smooth Animations:** Hardware-accelerated CSS transitions
4. **Lazy Loading:** Suggestions loaded on demand
5. **Efficient Filtering:** Client-side filtering with indexed lookups

## Testing

See `examples/TagSuggestionsExample.svelte` for interactive examples and testing scenarios.

## Future Enhancements

Potential improvements:
1. Drag-and-drop tag reordering
2. Custom confidence thresholds per category
3. Tag relationship visualization
4. ML-based tag recommendations
5. Bulk edit mode for multiple files
6. Keyboard shortcuts for power users
7. Export/import tag configurations
8. Tag usage analytics dashboard

## Dependencies

- Svelte 4.2+
- @tauri-apps/api 2.7+
- TypeScript 5.3+

## License

Part of the MIDI Software Center project.

## See Also

- `ARCHITECTURE-REFERENCE.md` - Three Archetypes Pattern
- `database/migrations/007_enhanced_tags.sql` - Database schema
- `pipeline/src-tauri/src/core/analysis/auto_tagger.rs` - Backend auto-tagging
