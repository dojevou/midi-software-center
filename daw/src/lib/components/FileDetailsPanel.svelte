<script lang="ts">
  import { selectedFile } from '../stores';
  import { api } from '../api';
  import type { FileDetails } from '../api';

  let details: FileDetails | null = null;
  let loading = false;
  let error: string | null = null;
  let copiedPath = false;

  // Collapsible section state
  let expandedSections = {
    basic: true,
    musical: true,
    technical: false,
    instruments: false,
    compatibility: false,
    statistics: false,
    actions: true,
  };

  // Watch for file selection changes
  $: if ($selectedFile) {
    loadDetails($selectedFile.id);
  }

  async function loadDetails(fileId: number) {
    loading = true;
    error = null;
    try {
      details = await api.files.getDetails(fileId);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load file details';
      console.error('Failed to load details:', err);
    } finally {
      loading = false;
    }
  }

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }

  function calculateBars(
    durationSeconds: number,
    bpm: number,
    timeSigNumerator: number = 4
  ): number {
    const beatsPerMinute = bpm;
    const totalBeats = (durationSeconds / 60) * beatsPerMinute;
    const barsCount = totalBeats / timeSigNumerator;
    return Math.ceil(barsCount);
  }

  function getKeyColor(key: string | null): string {
    if (!key) return '';
    // Check if key is minor (contains 'm' or 'minor')
    const isMinor = key.toLowerCase().includes('m') || key.toLowerCase().includes('minor');
    return isMinor ? 'minor-key' : 'major-key';
  }

  async function copyPath() {
    if (details) {
      try {
        await navigator.clipboard.writeText(details.file_path);
        copiedPath = true;
        setTimeout(() => (copiedPath = false), 2000);
      } catch (err) {
        console.error('Failed to copy path:', err);
      }
    }
  }

  function toggleSection(section: keyof typeof expandedSections) {
    expandedSections[section] = !expandedSections[section];
  }

  async function toggleFavorite() {
    if (!details) return;
    try {
      if (details.is_favorite) {
        await api.favorites.remove(details.id);
      } else {
        await api.favorites.add(details.id);
      }
      // Reload details to get updated favorite status
      await loadDetails(details.id);
    } catch (err) {
      console.error('Failed to toggle favorite:', err);
    }
  }

  async function loadToSequencer() {
    if (!details) return;
    try {
      await api.sequencer.addTrack(details.id, 0);
      console.log('File loaded to sequencer');
    } catch (err) {
      console.error('Failed to load to sequencer:', err);
    }
  }

  async function exportFile() {
    if (!details) return;
    try {
      // This would need a file picker dialog - simplified for now
      console.log('Export functionality not yet implemented');
    } catch (err) {
      console.error('Failed to export:', err);
    }
  }
</script>

<div class="details-panel">
  {#if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading details...</p>
    </div>
  {:else if error}
    <div class="error-state">
      <div class="error-icon">⚠️</div>
      <p>{error}</p>
    </div>
  {:else if details}
    <div class="details-content">
      <!-- Basic Info Section -->
      <section class="detail-section">
        <button class="section-header" on:click={() => toggleSection('basic')}>
          <span class="icon">📄</span>
          <h3>File Information</h3>
          <span class="toggle">{expandedSections.basic ? '▼' : '▶'}</span>
        </button>
        {#if expandedSections.basic}
          <div class="section-content">
            <div class="info-row">
              <span class="label">Name:</span>
              <span class="value" title={details.file_name}>{details.file_name}</span>
            </div>
            <div class="info-row">
              <span class="label">Path:</span>
              <div class="path-container">
                <span class="value path" title={details.file_path}>
                  {details.file_path.length > 40
                    ? '...' + details.file_path.slice(-37)
                    : details.file_path}
                </span>
                <button class="copy-btn" on:click={copyPath} title="Copy path">
                  {copiedPath ? '✓' : '📋'}
                </button>
              </div>
            </div>
            <div class="info-row">
              <span class="label">Size:</span>
              <span class="value">{formatFileSize(details.file_size)}</span>
            </div>
            <div class="info-row">
              <span class="label">Added:</span>
              <span class="value">{formatDate(details.created_at)}</span>
            </div>
            {#if details.manufacturer}
              <div class="info-row">
                <span class="label">Manufacturer:</span>
                <span class="value">{details.manufacturer}</span>
              </div>
            {/if}
            {#if details.collection}
              <div class="info-row">
                <span class="label">Collection:</span>
                <span class="value">{details.collection}</span>
              </div>
            {/if}
          </div>
        {/if}
      </section>

      <!-- Musical Properties Section -->
      <section class="detail-section">
        <button class="section-header" on:click={() => toggleSection('musical')}>
          <span class="icon">🎵</span>
          <h3>Musical Properties</h3>
          <span class="toggle">{expandedSections.musical ? '▼' : '▶'}</span>
        </button>
        {#if expandedSections.musical}
          <div class="section-content">
            {#if details.bpm}
              <div class="bpm-display">
                <div class="bpm-value">{details.bpm}</div>
                <div class="bpm-label">BPM</div>
              </div>
            {/if}
            {#if details.key}
              <div class="info-row">
                <span class="label">Key:</span>
                <span class="value key-value {getKeyColor(details.key)}">{details.key}</span>
              </div>
            {/if}
            {#if details.time_signature}
              <div class="info-row">
                <span class="label">Time Signature:</span>
                <span class="value">{details.time_signature}</span>
              </div>
            {/if}
            <div class="info-row">
              <span class="label">Duration:</span>
              <span class="value">
                {formatDuration(details.duration_seconds ?? 0)}
                {#if details.bpm}
                  ({calculateBars(details.duration_seconds ?? 0, details.bpm)} bars)
                {/if}
              </span>
            </div>
            <div class="info-row">
              <span class="label">Tracks:</span>
              <span class="value">{details.track_count}</span>
            </div>
            {#if details.category}
              <div class="info-row">
                <span class="label">Category:</span>
                <span class="value category-badge">{details.category}</span>
              </div>
            {/if}
            {#if details.tags && details.tags.length > 0}
              <div class="info-row">
                <span class="label">Tags:</span>
                <div class="tags">
                  {#each details.tags as tag}
                    <span class="tag">{tag}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </section>

      <!-- Technical Data Section -->
      <section class="detail-section">
        <button class="section-header" on:click={() => toggleSection('technical')}>
          <span class="icon">⚙️</span>
          <h3>Technical Data</h3>
          <span class="toggle">{expandedSections.technical ? '▼' : '▶'}</span>
        </button>
        {#if expandedSections.technical}
          <div class="section-content">
            <div class="info-row">
              <span class="label">MIDI Format:</span>
              <span class="value muted">Type 1 (estimated)</span>
            </div>
            <div class="info-row">
              <span class="label">TPQN:</span>
              <span class="value muted">480 (standard)</span>
            </div>
            <div class="info-row">
              <span class="label">Has Notes:</span>
              <span class="value">{details.has_notes ? 'Yes' : 'No'}</span>
            </div>
            <div class="info-row">
              <span class="label">Has Drums:</span>
              <span class="value">{details.has_drums ? 'Yes' : 'No'}</span>
            </div>
            <div class="info-row">
              <span class="label">Channels:</span>
              <span class="value muted">Not available</span>
            </div>
          </div>
        {/if}
      </section>

      <!-- Instruments/Channels Section -->
      <section class="detail-section">
        <button class="section-header" on:click={() => toggleSection('instruments')}>
          <span class="icon">🎹</span>
          <h3>Instruments</h3>
          <span class="toggle">{expandedSections.instruments ? '▼' : '▶'}</span>
        </button>
        {#if expandedSections.instruments}
          <div class="section-content">
            <div class="placeholder">
              <p>Instrument detection coming soon</p>
              <span>GM instrument mapping will be available in a future update</span>
            </div>
          </div>
        {/if}
      </section>

      <!-- Compatibility Section -->
      <section class="detail-section">
        <button class="section-header" on:click={() => toggleSection('compatibility')}>
          <span class="icon">🔗</span>
          <h3>Compatibility</h3>
          <span class="toggle">{expandedSections.compatibility ? '▼' : '▶'}</span>
        </button>
        {#if expandedSections.compatibility}
          <div class="section-content">
            <button class="action-btn secondary"> Find Compatible Files </button>
            <div class="placeholder">
              <span>Files with similar BPM and key will be shown here</span>
            </div>
          </div>
        {/if}
      </section>

      <!-- Statistics Section -->
      <section class="detail-section">
        <button class="section-header" on:click={() => toggleSection('statistics')}>
          <span class="icon">📊</span>
          <h3>Statistics</h3>
          <span class="toggle">{expandedSections.statistics ? '▼' : '▶'}</span>
        </button>
        {#if expandedSections.statistics}
          <div class="section-content">
            <div class="placeholder">
              <p>Usage tracking coming soon</p>
              <span
                >Play count, export history, and usage statistics will be available in a future
                update</span
              >
            </div>
          </div>
        {/if}
      </section>

      <!-- Actions Section -->
      <section class="detail-section">
        <button class="section-header" on:click={() => toggleSection('actions')}>
          <span class="icon">⚡</span>
          <h3>Actions</h3>
          <span class="toggle">{expandedSections.actions ? '▼' : '▶'}</span>
        </button>
        {#if expandedSections.actions}
          <div class="section-content actions">
            <button
              class="action-btn {details.is_favorite ? 'favorite-active' : ''}"
              on:click={toggleFavorite}
            >
              {details.is_favorite ? '★' : '☆'}
              {details.is_favorite ? 'Remove from' : 'Add to'} Favorites
            </button>
            <button class="action-btn" on:click={loadToSequencer}> ▶️ Load to Sequencer </button>
            <button class="action-btn" on:click={exportFile}> 💾 Export </button>
            <button class="action-btn" disabled> ✏️ Open in Editor </button>
          </div>
        {/if}
      </section>
    </div>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">📁</div>
      <p>Select a file to view details</p>
    </div>
  {/if}
</div>

<style>
  .details-panel {
    width: 350px;
    height: 100%;
    background: rgba(0, 0, 0, 0.3);
    border-left: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .loading-state,
  .error-state,
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
    height: 100%;
    color: rgba(255, 255, 255, 0.6);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #ff3e00;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-icon,
  .error-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    opacity: 0.3;
  }

  .error-state {
    color: #ff6b6b;
  }

  .details-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .detail-section {
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .section-header {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.2);
    border: none;
    color: #fff;
    cursor: pointer;
    transition: background 0.2s;
    text-align: left;
  }

  .section-header:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .section-header .icon {
    font-size: 1.25rem;
  }

  .section-header h3 {
    flex: 1;
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .section-header .toggle {
    font-size: 0.75rem;
    opacity: 0.5;
  }

  .section-content {
    padding: 1rem;
    animation: slideDown 0.2s ease-out;
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .info-row {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.5rem 0;
    font-size: 0.875rem;
  }

  .info-row .label {
    min-width: 100px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.6);
  }

  .info-row .value {
    flex: 1;
    color: #fff;
    word-break: break-word;
  }

  .value.muted {
    color: rgba(255, 255, 255, 0.4);
    font-style: italic;
  }

  .path-container {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .value.path {
    flex: 1;
    font-family: monospace;
    font-size: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .copy-btn {
    padding: 0.25rem 0.5rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: #fff;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.875rem;
  }

  .copy-btn:hover {
    background: rgba(255, 255, 255, 0.2);
    border-color: #ff3e00;
  }

  .bpm-display {
    text-align: center;
    padding: 1.5rem;
    margin-bottom: 1rem;
    background: rgba(76, 175, 80, 0.1);
    border: 2px solid rgba(76, 175, 80, 0.3);
    border-radius: 8px;
  }

  .bpm-value {
    font-size: 3rem;
    font-weight: 700;
    color: #4caf50;
    line-height: 1;
  }

  .bpm-label {
    font-size: 0.875rem;
    font-weight: 600;
    color: rgba(76, 175, 80, 0.8);
    margin-top: 0.5rem;
    text-transform: uppercase;
    letter-spacing: 2px;
  }

  .key-value {
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-weight: 600;
  }

  .key-value.major-key {
    background: rgba(33, 150, 243, 0.2);
    color: #2196f3;
  }

  .key-value.minor-key {
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
  }

  .category-badge {
    padding: 0.25rem 0.75rem;
    background: rgba(255, 152, 0, 0.2);
    color: #ff9800;
    border-radius: 4px;
    font-weight: 600;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .tag {
    padding: 0.25rem 0.5rem;
    background: rgba(156, 39, 176, 0.2);
    color: #9c27b0;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .placeholder {
    padding: 1rem;
    text-align: center;
    color: rgba(255, 255, 255, 0.4);
  }

  .placeholder p {
    margin: 0 0 0.5rem 0;
    font-weight: 600;
  }

  .placeholder span {
    font-size: 0.875rem;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .action-btn {
    width: 100%;
    padding: 0.75rem 1rem;
    background: rgba(255, 62, 0, 0.1);
    border: 2px solid rgba(255, 62, 0, 0.3);
    border-radius: 6px;
    color: #ff3e00;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    text-align: center;
  }

  .action-btn:hover:not(:disabled) {
    background: rgba(255, 62, 0, 0.2);
    border-color: #ff3e00;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(255, 62, 0, 0.3);
  }

  .action-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .action-btn.secondary {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.2);
    color: #fff;
  }

  .action-btn.secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.4);
  }

  .action-btn.favorite-active {
    background: rgba(255, 193, 7, 0.1);
    border-color: rgba(255, 193, 7, 0.5);
    color: #ffc107;
  }

  .action-btn.favorite-active:hover {
    background: rgba(255, 193, 7, 0.2);
    border-color: #ffc107;
  }

  /* Scrollbar styling */
  .details-content::-webkit-scrollbar {
    width: 8px;
  }

  .details-content::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
  }

  .details-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
  }

  .details-content::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  /* Responsive */
  @media (max-width: 768px) {
    .details-panel {
      width: 100%;
      border-left: none;
      border-top: 1px solid rgba(255, 255, 255, 0.1);
    }
  }
</style>
