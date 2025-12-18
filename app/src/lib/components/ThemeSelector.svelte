<script lang="ts">
  import { themeStore, themeName, THEME_NAMES, THEME_INFO, type ThemeName } from '$lib/stores/themeStore';

  let isOpen = false;

  function selectTheme(name: ThemeName) {
    themeStore.setTheme(name);
    isOpen = false;
  }

  function toggleDropdown() {
    isOpen = !isOpen;
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      isOpen = false;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="theme-selector">
  <button
    class="theme-button"
    on:click={toggleDropdown}
    aria-expanded={isOpen}
    aria-haspopup="listbox"
  >
    <span class="theme-indicator" data-theme={$themeName}></span>
    <span class="theme-name">{$themeName}</span>
    <svg class="chevron" class:rotated={isOpen} width="12" height="12" viewBox="0 0 12 12">
      <path d="M2 4L6 8L10 4" stroke="currentColor" stroke-width="1.5" fill="none"/>
    </svg>
  </button>

  {#if isOpen}
    <div class="theme-dropdown" role="listbox">
      {#each THEME_NAMES as name}
        <button
          class="theme-option"
          class:selected={$themeName === name}
          on:click={() => selectTheme(name)}
          role="option"
          aria-selected={$themeName === name}
        >
          <span class="theme-indicator" data-theme={name}></span>
          <div class="theme-info">
            <span class="option-name">{THEME_INFO[name].name}</span>
            <span class="option-desc">{THEME_INFO[name].description}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .theme-selector {
    position: relative;
    display: inline-block;
  }

  .theme-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    color: var(--text-primary);
    cursor: pointer;
    font-size: var(--font-size-sm);
    transition: background var(--transition-duration), border-color var(--transition-duration);
  }

  .theme-button:hover {
    background: var(--bg-hover);
    border-color: var(--border-focus);
  }

  .theme-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid var(--border);
  }

  .theme-indicator[data-theme="DARK"] { background: linear-gradient(135deg, #121212, #007bff); }
  .theme-indicator[data-theme="WARM"] { background: linear-gradient(135deg, #1a1512, #e8a54b); }
  .theme-indicator[data-theme="NEON"] { background: linear-gradient(135deg, #0a0a0f, #ff6b2b); }
  .theme-indicator[data-theme="MINT"] { background: linear-gradient(135deg, #0f1614, #26c6a0); }
  .theme-indicator[data-theme="ROSE"] { background: linear-gradient(135deg, #18121a, #e878a0); }
  .theme-indicator[data-theme="BASS"] { background: linear-gradient(135deg, #08080a, #6366f1); }

  .theme-name {
    font-weight: 500;
  }

  .chevron {
    transition: transform var(--transition-duration);
  }

  .chevron.rotated {
    transform: rotate(180deg);
  }

  .theme-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    margin-top: 4px;
    min-width: 220px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 1000;
    overflow: hidden;
  }

  .theme-option {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 10px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    transition: background var(--transition-duration);
  }

  .theme-option:hover {
    background: var(--bg-hover);
  }

  .theme-option.selected {
    background: var(--accent-muted);
  }

  .theme-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .option-name {
    font-weight: 600;
    font-size: var(--font-size-sm);
    letter-spacing: 0.5px;
  }

  .option-desc {
    font-size: 10px;
    color: var(--text-muted);
    letter-spacing: 0.3px;
  }
</style>
