<script lang="ts">
  import { themeStore, isDarkMode, themeMode } from '$lib/stores/themeStore';
  import type { ThemeMode } from '$lib/stores/themeStore';

  export let showLabel = false;
  export let size: 'small' | 'medium' | 'large' = 'medium';

  const sizes = {
    small: { button: '24px', icon: '14px' },
    medium: { button: '32px', icon: '18px' },
    large: { button: '40px', icon: '22px' },
  };

  function cycleMode() {
    const modes: ThemeMode[] = ['light', 'dark', 'system'];
    const currentIndex = modes.indexOf($themeMode);
    const nextMode = modes[(currentIndex + 1) % modes.length];
    themeStore.setMode(nextMode);
  }

  function getIcon(mode: ThemeMode, resolved: boolean): string {
    if (mode === 'system') return '💻';
    return resolved ? '🌙' : '☀️';
  }

  function getLabel(mode: ThemeMode): string {
    switch (mode) {
      case 'light': return 'Light mode';
      case 'dark': return 'Dark mode';
      case 'system': return 'System preference';
    }
  }
</script>

<button
  class="theme-toggle"
  class:small={size === 'small'}
  class:large={size === 'large'}
  on:click={cycleMode}
  aria-label={getLabel($themeMode)}
  title={getLabel($themeMode)}
  style="--btn-size: {sizes[size].button}; --icon-size: {sizes[size].icon}"
>
  <span class="icon" aria-hidden="true">
    {getIcon($themeMode, $isDarkMode)}
  </span>
  {#if showLabel}
    <span class="label">{getLabel($themeMode)}</span>
  {/if}
</button>

<style>
  .theme-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: 4px 8px;
    min-width: var(--btn-size);
    min-height: var(--btn-size);
    cursor: pointer;
    transition: background-color var(--transition-duration) ease,
                border-color var(--transition-duration) ease;
  }

  .theme-toggle:hover {
    background: var(--bg-hover);
    border-color: var(--border-focus);
  }

  .theme-toggle:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .icon {
    font-size: var(--icon-size);
    line-height: 1;
  }

  .label {
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
  }

  .small {
    padding: 2px 4px;
  }

  .large {
    padding: 8px 12px;
  }
</style>
