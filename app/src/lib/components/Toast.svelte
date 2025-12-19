<script lang="ts">
  import { toastStore } from '$lib/stores/toastStore';
  import { fly } from 'svelte/transition';

  $: toasts = $toastStore;

  function getIcon(type: string) {
    switch (type) {
      case 'success':
        return '✓';
      case 'error':
        return '✕';
      case 'warning':
        return '⚠';
      case 'info':
        return 'ℹ';
      default:
        return '';
    }
  }
</script>

<div class="toast-container">
  {#each toasts as toast (toast.id)}
    <div
      class="toast toast-{toast.type}"
      transition:fly={{ y: 20, duration: 200 }}
      on:click={() => toastStore.dismiss(toast.id)}
      role="alert"
    >
      <span class="toast-icon">{getIcon(toast.type)}</span>
      <span class="toast-message">{toast.message}</span>
      <button class="toast-close" on:click={() => toastStore.dismiss(toast.id)}>×</button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    gap: 12px;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: var(--color-bg-secondary, #2a2a2a);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(255, 255, 255, 0.1);
    min-width: 280px;
    max-width: 400px;
    pointer-events: auto;
    cursor: pointer;
    transition: transform 0.2s, opacity 0.2s;
  }

  .toast:hover {
    transform: translateY(-2px);
    opacity: 0.95;
  }

  .toast-icon {
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: bold;
    border-radius: 50%;
  }

  .toast-success {
    border-left: 4px solid #34c759;
  }

  .toast-success .toast-icon {
    background: #34c759;
    color: white;
  }

  .toast-error {
    border-left: 4px solid #ff3b30;
  }

  .toast-error .toast-icon {
    background: #ff3b30;
    color: white;
  }

  .toast-warning {
    border-left: 4px solid #ff9500;
  }

  .toast-warning .toast-icon {
    background: #ff9500;
    color: white;
  }

  .toast-info {
    border-left: 4px solid #007aff;
  }

  .toast-info .toast-icon {
    background: #007aff;
    color: white;
  }

  .toast-message {
    flex: 1;
    font-size: 14px;
    color: var(--color-text, #fff);
    line-height: 1.4;
  }

  .toast-close {
    flex-shrink: 0;
    background: none;
    border: none;
    color: var(--color-text-secondary, #999);
    font-size: 24px;
    line-height: 1;
    cursor: pointer;
    padding: 0;
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.2s;
  }

  .toast-close:hover {
    color: var(--color-text, #fff);
  }
</style>
