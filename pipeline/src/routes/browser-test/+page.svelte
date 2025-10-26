<script lang="ts">
  import FileBrowser from '$lib/components/FileBrowser/FileBrowser.svelte';
</script>

<div class="h-screen flex flex-col">
  <header class="bg-white border-b p-4">
    <h1 class="text-2xl font-bold">File Browser Test</h1>
    <p class="text-sm text-slate-600">Testing FileBrowser component with grid/list toggle and loading states</p>
  </header>

  <main class="flex-1 overflow-hidden">
    <FileBrowser />
  </main>
</div>
