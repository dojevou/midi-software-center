<script lang="ts">
  import { keyboardManager, type KeyboardShortcut } from '../utils/keyboard';

  export let visible = false;

  function formatShortcut(shortcut: KeyboardShortcut): string {
    return keyboardManager.formatShortcut(shortcut);
  }

  function close() {
    visible = false;
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  // Get shortcuts organized by category
  $: shortcutsByCategory = keyboardManager.getShortcutsByCategory();

  // Define category order for display
  const categoryOrder = [
    'Playback',
    'Navigation',
    'Zoom',
    'File Operations',
    'Editing',
    'Selection',
    'Search/UI',
    'Display',
    'Other',
  ];
</script>

{#if visible}
  <div class="modal-overlay" on:click={handleBackdropClick} role="presentation">
    <div class="modal" on:click|stopPropagation role="dialog" aria-labelledby="shortcuts-title">
      <div class="modal-header">
        <h2 id="shortcuts-title">⌨️ Keyboard Shortcuts</h2>
        <button class="close-btn" on:click={close} aria-label="Close" title="Close (Esc)">
          ✕
        </button>
      </div>

      <div class="shortcuts-container">
        {#each categoryOrder as category}
          {#if shortcutsByCategory.has(category)}
            <div class="shortcuts-category">
              <h3>{category}</h3>
              <div class="shortcuts-list">
                {#each shortcutsByCategory.get(category) || [] as shortcut}
                  <div class="shortcut-item">
                    <kbd class="shortcut-key">{formatShortcut(shortcut)}</kbd>
                    <span class="shortcut-desc">{shortcut.description}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        {/each}
      </div>

      <div class="modal-footer">
        <p class="hint">💡 Press <kbd>?</kbd> to show/hide this dialog</p>
        <button class="primary-btn" on:click={close}>Got it!</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal {
    background: linear-gradient(135deg, #1a1a1a 0%, #242424 100%);
    border: 2px solid rgba(255, 62, 0, 0.5);
    border-radius: 12px;
    max-width: 900px;
    width: 90%;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    animation: slideUp 0.3s ease-out;
  }

  @keyframes slideUp {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-bottom: 2px solid rgba(255, 62, 0, 0.3);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.75rem;
    color: #ff3e00;
    font-weight: 700;
  }

  .close-btn {
    width: 36px;
    height: 36px;
    background: rgba(255, 255, 255, 0.05);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: rgba(255, 255, 255, 0.7);
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-btn:hover {
    background: rgba(244, 67, 54, 0.2);
    border-color: rgba(244, 67, 54, 0.5);
    color: #f44336;
    transform: scale(1.05);
  }

  .shortcuts-container {
    flex: 1;
    overflow-y: auto;
    padding: 2rem;
  }

  .shortcuts-category {
    margin-bottom: 2rem;
  }

  .shortcuts-category:last-child {
    margin-bottom: 0;
  }

  .shortcuts-category h3 {
    color: #3b82f6;
    font-size: 1.125rem;
    font-weight: 600;
    margin: 0 0 1rem 0;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid rgba(59, 130, 246, 0.3);
  }

  .shortcuts-list {
    display: grid;
    gap: 0.75rem;
  }

  .shortcut-item {
    display: grid;
    grid-template-columns: 180px 1fr;
    gap: 1rem;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 6px;
    align-items: center;
    transition: all 0.2s;
  }

  .shortcut-item:hover {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 62, 0, 0.3);
    transform: translateX(4px);
  }

  kbd {
    background: linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.1) 0%,
      rgba(255, 255, 255, 0.05) 100%
    );
    padding: 0.4rem 0.75rem;
    border: 2px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', 'Courier New', monospace;
    font-size: 0.875rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
    display: inline-block;
    text-align: center;
    min-width: 50px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .shortcut-key {
    justify-self: start;
  }

  .shortcut-desc {
    color: rgba(255, 255, 255, 0.8);
    font-size: 0.9375rem;
  }

  .modal-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 2rem;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(0, 0, 0, 0.2);
  }

  .hint {
    margin: 0;
    color: rgba(255, 255, 255, 0.5);
    font-size: 0.875rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .hint kbd {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    min-width: auto;
  }

  .primary-btn {
    padding: 0.75rem 2rem;
    background: linear-gradient(135deg, #ff3e00 0%, #ff6b35 100%);
    border: none;
    border-radius: 6px;
    color: white;
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    box-shadow: 0 4px 12px rgba(255, 62, 0, 0.3);
  }

  .primary-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 16px rgba(255, 62, 0, 0.4);
  }

  .primary-btn:active {
    transform: translateY(0);
  }

  /* Scrollbar styling */
  .shortcuts-container::-webkit-scrollbar {
    width: 10px;
  }

  .shortcuts-container::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 5px;
  }

  .shortcuts-container::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 5px;
  }

  .shortcuts-container::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  /* Responsive design */
  @media (max-width: 768px) {
    .modal {
      width: 95%;
      max-height: 90vh;
    }

    .modal-header {
      padding: 1rem 1.5rem;
    }

    .modal-header h2 {
      font-size: 1.5rem;
    }

    .shortcuts-container {
      padding: 1.5rem;
    }

    .shortcut-item {
      grid-template-columns: 1fr;
      gap: 0.5rem;
    }

    .shortcut-key {
      justify-self: center;
    }

    .shortcut-desc {
      text-align: center;
    }

    .modal-footer {
      flex-direction: column;
      gap: 1rem;
      padding: 1rem 1.5rem;
    }

    .hint {
      order: 2;
    }

    .primary-btn {
      width: 100%;
    }
  }
</style>
