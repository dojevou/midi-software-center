<script lang="ts">
  import { searchResults, selectedFile, searchTotal } from '../stores';
  import type { FileDetails } from '../api';

  export let viewMode: 'grid' | 'list' = 'grid';

  let currentPage = 1;
  const itemsPerPage = 20;

  $: totalPages = Math.ceil($searchTotal / itemsPerPage);
  $: startIndex = (currentPage - 1) * itemsPerPage;
  $: endIndex = Math.min(startIndex + itemsPerPage, $searchTotal);

  function selectFile(file: FileDetails) {
    $selectedFile = file;
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatFileSize(bytes: number): string {
    const kb = bytes / 1024;
    if (kb < 1024) return `${kb.toFixed(1)} KB`;
    return `${(kb / 1024).toFixed(1)} MB`;
  }

  function goToPage(page: number) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
      // Scroll to top of file list
      window.scrollTo({ top: 0, behavior: 'smooth' });
    }
  }

  function getPageNumbers(): (number | string)[] {
    const pages: (number | string)[] = [];
    const maxVisible = 7;

    if (totalPages <= maxVisible) {
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
    } else {
      pages.push(1);

      if (currentPage > 3) {
        pages.push('...');
      }

      const start = Math.max(2, currentPage - 1);
      const end = Math.min(totalPages - 1, currentPage + 1);

      for (let i = start; i <= end; i++) {
        pages.push(i);
      }

      if (currentPage < totalPages - 2) {
        pages.push('...');
      }

      pages.push(totalPages);
    }

    return pages;
  }

  // Reset to page 1 when search results change
  $: if ($searchResults) {
    currentPage = 1;
  }
</script>

<div class="file-list-container">
  <!-- Results Header -->
  <div class="results-header">
    <span class="results-count">
      Showing {startIndex + 1}-{endIndex} of {$searchTotal} files
    </span>
  </div>

  <!-- File List -->
  <div class="file-list {viewMode}">
    {#if $searchResults.length === 0}
      <div class="empty-state">
        <div class="empty-icon">🔍</div>
        <p>No files found</p>
        <span>Try adjusting your search filters</span>
      </div>
    {:else}
      {#each $searchResults as file (file.id)}
        <div
          class="file-card {$selectedFile?.id === file.id ? 'selected' : ''}"
          on:click={() => selectFile(file)}
          role="button"
          tabindex="0"
          on:keypress={(e) => e.key === 'Enter' && selectFile(file)}
        >
          <div class="file-icon">🎵</div>

          <div class="file-info">
            <h3 class="file-name" title={file.file_name}>{file.file_name}</h3>

            <div class="file-meta">
              {#if file.bpm}
                <span class="meta-tag bpm">{file.bpm} BPM</span>
              {/if}
              {#if file.key}
                <span class="meta-tag key">{file.key}</span>
              {/if}
              {#if file.category}
                <span class="meta-tag category">{file.category}</span>
              {/if}
            </div>

            <div class="file-details">
              <span>⏱️ {formatDuration(file.duration_seconds ?? 0)}</span>
              <span>🎹 {file.track_count} tracks</span>
              <span>💾 {formatFileSize(file.file_size)}</span>
              {#if file.has_drums}
                <span>🥁 Drums</span>
              {/if}
            </div>

            {#if file.manufacturer}
              <div class="manufacturer">
                {file.manufacturer}
              </div>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </div>

  <!-- Pagination -->
  {#if totalPages > 1}
    <div class="pagination">
      <button
        class="page-btn"
        on:click={() => goToPage(currentPage - 1)}
        disabled={currentPage === 1}
        aria-label="Previous page"
      >
        ‹ Prev
      </button>

      <div class="page-numbers">
        {#each getPageNumbers() as page}
          {#if page === '...'}
            <span class="page-ellipsis">...</span>
          {:else}
            <button
              class="page-btn {currentPage === page ? 'active' : ''}"
              on:click={() => goToPage(Number(page))}
            >
              {page}
            </button>
          {/if}
        {/each}
      </div>

      <button
        class="page-btn"
        on:click={() => goToPage(currentPage + 1)}
        disabled={currentPage === totalPages}
        aria-label="Next page"
      >
        Next ›
      </button>
    </div>
  {/if}
</div>

<style>
  .file-list-container {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .results-header {
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .results-count {
    font-size: 0.875rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.7);
  }

  .file-list {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
  }

  .file-list.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
    align-content: start;
  }

  .file-list.list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .file-card {
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    gap: 1rem;
  }

  .file-card:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 62, 0, 0.5);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .file-card.selected {
    background: rgba(255, 62, 0, 0.15);
    border-color: #ff3e00;
    box-shadow: 0 0 20px rgba(255, 62, 0, 0.3);
  }

  .file-icon {
    font-size: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 60px;
  }

  .file-info {
    flex: 1;
    min-width: 0;
  }

  .file-name {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    font-weight: 600;
    color: #fff;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .meta-tag {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .meta-tag.bpm {
    background: rgba(76, 175, 80, 0.2);
    color: #4caf50;
  }

  .meta-tag.key {
    background: rgba(33, 150, 243, 0.2);
    color: #2196f3;
  }

  .meta-tag.category {
    background: rgba(255, 152, 0, 0.2);
    color: #ff9800;
  }

  .file-details {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
  }

  .manufacturer {
    margin-top: 0.5rem;
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
    font-style: italic;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    text-align: center;
    color: rgba(255, 255, 255, 0.5);
    grid-column: 1 / -1;
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
    opacity: 0.3;
  }

  .empty-state p {
    margin: 0 0 0.5rem 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .empty-state span {
    font-size: 0.875rem;
  }

  /* Pagination */
  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 1.5rem 1rem;
    background: rgba(0, 0, 0, 0.2);
    border-top: 1px solid rgba(255, 255, 255, 0.1);
  }

  .page-numbers {
    display: flex;
    gap: 0.25rem;
  }

  .page-btn {
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    color: #fff;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    min-width: 40px;
  }

  .page-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    border-color: #ff3e00;
  }

  .page-btn.active {
    background: rgba(255, 62, 0, 0.2);
    border-color: #ff3e00;
    color: #ff3e00;
  }

  .page-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .page-ellipsis {
    padding: 0.5rem;
    color: rgba(255, 255, 255, 0.3);
    font-weight: 600;
  }

  /* List view specific styles */
  .file-list.list .file-card {
    flex-direction: row;
  }

  .file-list.list .file-icon {
    font-size: 2rem;
    min-width: 50px;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .file-list.grid {
      grid-template-columns: 1fr;
    }

    .pagination {
      flex-wrap: wrap;
    }

    .page-numbers {
      order: 3;
      width: 100%;
      justify-content: center;
      margin-top: 0.5rem;
    }
  }
</style>
