<script lang="ts">
  import MenuBar from '$lib/components/MenuBar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import DAWWindow from '$lib/windows/DAWWindow.svelte';
  import MixerWindow from '$lib/windows/MixerWindow.svelte';
  import DatabaseWindow from '$lib/windows/DatabaseWindow.svelte';
  import PipelineWindow from '$lib/windows/PipelineWindow.svelte';

  console.log('App loaded - simple version without event listeners');
</script>

<MenuBar />

<div class="workspace">
  <DAWWindow />
  <MixerWindow />
  <DatabaseWindow />
  <PipelineWindow />
</div>

<StatusBar />

<style>
  .workspace {
    position: relative;
    height: calc(100vh - 4rem); /* Adjust for menu and status bar */
    overflow: hidden;
    background-color: var(--bg-primary);
  }
</style>
