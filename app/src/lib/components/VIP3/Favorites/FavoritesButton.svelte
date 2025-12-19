<script lang="ts">
  import { Vip3BrowserApi } from '$lib/api/vip3BrowserApi';

  export let fileId: number;
  export let isFavorite: boolean = false;
  export let onToggle: (newState: boolean) => void = () => {};
  export let size: 'small' | 'medium' | 'large' = 'medium';
  export let showLabel: boolean = false;

  let loading = false;

  const sizes = {
    small: { button: '24px', icon: '16px' },
    medium: { button: '32px', icon: '20px' },
    large: { button: '40px', icon: '24px' },
  };

  async function handleToggle() {
    if (loading) return;

    try {
      loading = true;
      const newState = await Vip3BrowserApi.toggleFavorite(fileId);
      isFavorite = newState;
      onToggle(newState);
    } catch (e) {
      console.error('Error toggling favorite:', e);
    } finally {
      loading = false;
    }
  }
</script>

<button
  class="favorites-button"
  class:favorite={isFavorite}
  class:loading
  on:click|stopPropagation={handleToggle}
  disabled={loading}
  title={isFavorite ? 'Remove from favorites' : 'Add to favorites'}
  style:width={sizes[size].button}
  style:height={sizes[size].button}
>
  <span class="icon" style:font-size={sizes[size].icon}>
    {isFavorite ? '❤️' : '🤍'}
  </span>
  {#if showLabel}
    <span class="label">{isFavorite ? 'Favorited' : 'Favorite'}</span>
  {/if}
</button>

<style>
  .favorites-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 0;
    background: transparent;
    border: none;
    cursor: pointer;
    border-radius: 6px;
    transition: all 0.2s;
    position: relative;
  }

  .favorites-button:hover {
    background: var(--color-bg-hover, rgba(255, 255, 255, 0.1));
    transform: scale(1.1);
  }

  .favorites-button:active {
    transform: scale(0.95);
  }

  .favorites-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .favorites-button.loading .icon {
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .icon {
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s;
  }

  .favorites-button.favorite .icon {
    animation: heartbeat 0.3s ease-in-out;
  }

  @keyframes heartbeat {
    0%, 100% {
      transform: scale(1);
    }
    25% {
      transform: scale(1.2);
    }
    50% {
      transform: scale(1);
    }
    75% {
      transform: scale(1.1);
    }
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text, #fff);
    white-space: nowrap;
  }

  .favorites-button.favorite .label {
    color: var(--color-error, #ff3b30);
  }
</style>
