<script lang="ts">
  /**
   * Tag Suggestions Example
   *
   * Demonstrates how to use the TagSuggestions component
   * in different contexts and configurations.
   */

  import TagSuggestions from '../TagSuggestions.svelte';
  import { onMount } from 'svelte';

  // =============================================================================
  // EXAMPLE 1: Basic Usage - File-Specific Suggestions
  // =============================================================================

  let selectedFileId: number | null = 123; // Replace with actual file ID

  // =============================================================================
  // EXAMPLE 2: Compact Mode - Sidebar Widget
  // =============================================================================

  // Compact mode is useful for sidebars or smaller panels

  // =============================================================================
  // EXAMPLE 3: Browse All Pending Suggestions
  // =============================================================================

  // For a dedicated tag management page

  onMount(() => {
    console.log('TagSuggestions examples mounted');
  });
</script>

<div class="examples-container">
  <h2>Tag Suggestions Component Examples</h2>

  <!-- Example 1: Full Featured -->
  <section class="example-section">
    <h3>Example 1: Full Featured (File-Specific)</h3>
    <p class="description">
      Complete tag suggestions interface for a specific file. Shows all features:
      batch actions, filtering, search, and both list/cloud views.
    </p>

    <div class="example-demo">
      <TagSuggestions
        fileId={selectedFileId}
        showBatchActions={true}
        allowGenerate={true}
        maxHeight="500px"
      />
    </div>

    <details class="code-example">
      <summary>View Code</summary>
      <pre><code>{`<script>
  import TagSuggestions from '$lib/components/TagSuggestions.svelte';

  let fileId = 123; // Your file ID
</script>

<TagSuggestions
  fileId={fileId}
  showBatchActions={true}
  allowGenerate={true}
  maxHeight="500px"
/>`}</code></pre>
    </details>
  </section>

  <!-- Example 2: Compact Mode -->
  <section class="example-section">
    <h3>Example 2: Compact Mode (Sidebar Widget)</h3>
    <p class="description">
      Minimal version suitable for sidebars or smaller panels. Reduces padding
      and spacing for tighter layouts.
    </p>

    <div class="example-demo compact">
      <TagSuggestions
        fileId={selectedFileId}
        compact={true}
        showBatchActions={false}
        allowGenerate={false}
        maxHeight="400px"
      />
    </div>

    <details class="code-example">
      <summary>View Code</summary>
      <pre><code>{`<TagSuggestions
  fileId={fileId}
  compact={true}
  showBatchActions={false}
  allowGenerate={false}
  maxHeight="400px"
/>`}</code></pre>
    </details>
  </section>

  <!-- Example 3: Bulk Review Mode -->
  <section class="example-section">
    <h3>Example 3: Bulk Review Mode</h3>
    <p class="description">
      For reviewing pending suggestions across all files. Set fileId to null
      and the component will load pending suggestions from all files.
    </p>

    <div class="example-demo">
      <TagSuggestions
        fileId={null}
        showBatchActions={true}
        allowGenerate={false}
        maxHeight="600px"
      />
    </div>

    <details class="code-example">
      <summary>View Code</summary>
      <pre><code>{`<TagSuggestions
  fileId={null}  <!-- null = load all pending -->
  showBatchActions={true}
  allowGenerate={false}
  maxHeight="600px"
/>`}</code></pre>
    </details>
  </section>

  <!-- Usage Notes -->
  <section class="usage-notes">
    <h3>Usage Notes</h3>

    <div class="note-card">
      <h4>Props</h4>
      <ul>
        <li><code>fileId</code> - File ID for suggestions (null = all pending)</li>
        <li><code>showBatchActions</code> - Show batch accept/reject controls (default: true)</li>
        <li><code>allowGenerate</code> - Show "Generate" button to create suggestions (default: true)</li>
        <li><code>maxHeight</code> - Maximum height for scrollable areas (default: "600px")</li>
        <li><code>compact</code> - Use compact mode with reduced spacing (default: false)</li>
      </ul>
    </div>

    <div class="note-card">
      <h4>Features</h4>
      <ul>
        <li>Two view modes: List (detailed) and Cloud (visual)</li>
        <li>Real-time search and filtering by tag name or category</li>
        <li>Category-based color coding (genre=blue, instrument=green, etc.)</li>
        <li>Confidence level indicators (high/medium/low with color bars)</li>
        <li>Multi-select for batch operations</li>
        <li>Keyboard-accessible with proper ARIA labels</li>
        <li>Smooth animations using Svelte transitions</li>
      </ul>
    </div>

    <div class="note-card">
      <h4>Store Access</h4>
      <p>
        For advanced use cases, you can directly access the store:
      </p>
      <pre><code>{`import {
  tagSuggestionsState,
  filteredSuggestions,
  fetchTagSuggestions,
  acceptSuggestion
} from '$lib/stores/tagSuggestions';

// Subscribe to state
$: suggestions = $filteredSuggestions;

// Manually trigger actions
await fetchTagSuggestions(fileId);
await acceptSuggestion(suggestionId);`}</code></pre>
    </div>

    <div class="note-card">
      <h4>Backend Requirements</h4>
      <p>The component requires these Tauri commands:</p>
      <ul>
        <li><code>get_tag_categories</code> - Fetch tag category definitions</li>
        <li><code>get_tag_suggestions</code> - Get suggestions for a file</li>
        <li><code>generate_tag_suggestions</code> - Auto-generate suggestions</li>
        <li><code>get_pending_tag_suggestions</code> - Get all pending suggestions</li>
        <li><code>accept_tag_suggestion</code> - Accept a suggestion</li>
        <li><code>reject_tag_suggestion</code> - Reject a suggestion</li>
        <li><code>batch_process_tag_suggestions</code> - Batch accept/reject</li>
      </ul>
    </div>

    <div class="note-card">
      <h4>Integration with File Browser</h4>
      <p>Example integration with a file detail panel:</p>
      <pre><code>{`<script>
  import TagSuggestions from '$lib/components/TagSuggestions.svelte';
  import { selectedFileStore } from '$lib/stores/files';

  $: selectedFile = $selectedFileStore;
</script>

<!-- File Detail Panel -->
<div class="file-details">
  <h2>{selectedFile?.filename}</h2>

  <!-- Other file info... -->

  <section class="tags-section">
    <h3>Tag Suggestions</h3>
    {#if selectedFile}
      <TagSuggestions
        fileId={selectedFile.id}
        compact={true}
        maxHeight="400px"
      />
    {/if}
  </section>
</div>`}</code></pre>
    </div>
  </section>
</div>

<style>
  .examples-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 40px 20px;
    color: var(--color-text, #e5e5e5);
  }

  h2 {
    font-size: 28px;
    font-weight: 700;
    margin-bottom: 32px;
    color: var(--color-text, #e5e5e5);
  }

  .example-section {
    margin-bottom: 48px;
    padding: 24px;
    background: var(--color-surface, #2d2d2d);
    border-radius: 12px;
    border: 1px solid var(--color-border, #3d3d3d);
  }

  h3 {
    font-size: 20px;
    font-weight: 600;
    margin: 0 0 12px 0;
    color: var(--color-text, #e5e5e5);
  }

  .description {
    margin: 0 0 20px 0;
    font-size: 14px;
    color: var(--color-text-secondary, #a3a3a3);
    line-height: 1.6;
  }

  .example-demo {
    margin-bottom: 20px;
  }

  .example-demo.compact {
    max-width: 400px;
  }

  .code-example {
    margin-top: 16px;
  }

  .code-example summary {
    cursor: pointer;
    padding: 10px 16px;
    background: var(--color-bg, #1e1e1e);
    border-radius: 6px;
    font-size: 14px;
    color: var(--color-primary, #3b82f6);
    font-weight: 500;
    transition: background 0.2s;
  }

  .code-example summary:hover {
    background: var(--color-surface, #2d2d2d);
  }

  .code-example pre {
    margin: 12px 0 0 0;
    padding: 16px;
    background: var(--color-bg, #1e1e1e);
    border-radius: 6px;
    overflow-x: auto;
  }

  .code-example code {
    font-family: 'Courier New', monospace;
    font-size: 13px;
    line-height: 1.6;
    color: #9cdcfe;
  }

  .usage-notes {
    margin-top: 48px;
  }

  .note-card {
    margin-bottom: 24px;
    padding: 20px;
    background: var(--color-bg, #1e1e1e);
    border-radius: 8px;
    border-left: 3px solid var(--color-primary, #3b82f6);
  }

  .note-card h4 {
    margin: 0 0 12px 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text, #e5e5e5);
  }

  .note-card p {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: var(--color-text-secondary, #a3a3a3);
    line-height: 1.6;
  }

  .note-card ul {
    margin: 0;
    padding-left: 20px;
    list-style-type: disc;
  }

  .note-card li {
    margin-bottom: 8px;
    font-size: 14px;
    color: var(--color-text-secondary, #a3a3a3);
    line-height: 1.6;
  }

  .note-card code {
    padding: 2px 6px;
    background: var(--color-surface, #2d2d2d);
    border-radius: 3px;
    font-family: 'Courier New', monospace;
    font-size: 13px;
    color: #9cdcfe;
  }

  .note-card pre {
    margin: 12px 0 0 0;
    padding: 16px;
    background: var(--color-surface, #2d2d2d);
    border-radius: 6px;
    overflow-x: auto;
  }

  .note-card pre code {
    background: transparent;
    padding: 0;
  }
</style>
