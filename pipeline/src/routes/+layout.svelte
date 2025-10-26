<script lang="ts">
  import { onMount } from 'svelte';
  import { uiStore } from '$lib/stores/ui';
  import '../app.css';

  onMount(() => {
    // Initialize theme from localStorage
    const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | null;
    if (savedTheme) {
      uiStore.setTheme(savedTheme);
    }
  });

  // Apply theme changes to document
  $: {
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', $uiStore.theme === 'dark');
    }
  }
</script>

<div class="min-h-screen bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-50 transition-colors">
  <slot />
</div>

<!-- Global Notifications -->
<div class="fixed top-4 right-4 z-50 flex flex-col gap-2">
  {#each $uiStore.notifications as notification (notification.id)}
    <div
      class="max-w-sm w-full bg-white dark:bg-slate-800 border rounded-lg shadow-lg p-4 transition-all duration-300 animate-slide-in-right
        {notification.type === 'error' ? 'border-red-500' :
         notification.type === 'warning' ? 'border-yellow-500' :
         notification.type === 'success' ? 'border-green-500' :
         'border-slate-200 dark:border-slate-700'}"
      role="alert"
    >
      <div class="flex items-start gap-3">
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium text-slate-900 dark:text-slate-100">
            {notification.message}
          </p>
        </div>
        <button
          on:click={() => uiStore.removeNotification(notification.id)}
          class="flex-shrink-0 p-1 rounded hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors"
          aria-label="Dismiss notification"
        >
          ✕
        </button>
      </div>
    </div>
  {/each}
</div>

<style>
  @keyframes slide-in-right {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .animate-slide-in-right {
    animation: slide-in-right 0.3s ease-out;
  }
</style>
