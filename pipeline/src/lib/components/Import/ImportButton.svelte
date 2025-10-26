<!--
  ImportButton.svelte - Buttons to trigger file/folder import
  Location: /pipeline/src/lib/components/Import/ImportButton.svelte
  Archetype: CONTAINER (composition + layout, NO I/O)

  Container responsibilities:
  - UI composition (buttons)
  - User interaction handling
  - Delegates I/O to importStore (Manager)
-->

<script lang="ts">
  import { Upload, FolderOpen } from 'lucide-svelte';
  import { importStore } from '$lib/stores/import';
  import { filesStore } from '$lib/stores/files';
  import { uiStore } from '$lib/stores/ui';

  // Read state from store (reactive)
  $: importing = $importStore.importing;

  async function handleImportFile() {
    // Delegate to Manager store (handles file dialog + import I/O)
    const fileId = await importStore.pickAndImport();

    if (fileId) {
      uiStore.addNotification({
        type: 'success',
        message: 'File imported successfully',
        duration: 3000,
      });
      // Reload files to show new import
      filesStore.loadFiles();
    } else if ($importStore.error) {
      uiStore.addNotification({
        type: 'error',
        message: $importStore.error,
        duration: 5000,
      });
    }
  }

  async function handleImportFolder() {
    // Delegate to Manager store (handles folder dialog + import I/O)
    const success = await importStore.pickAndImportDirectory(true);

    if (success) {
      uiStore.addNotification({
        type: 'success',
        message: 'Folder imported successfully',
        duration: 3000,
      });
      // Reload files to show new imports
      filesStore.loadFiles();
    } else if ($importStore.error) {
      uiStore.addNotification({
        type: 'error',
        message: $importStore.error,
        duration: 5000,
      });
    }
  }
</script>

<div class="flex gap-2">
  <button
    on:click={handleImportFile}
    disabled={importing}
    class="btn-primary px-4 py-2 flex items-center gap-2"
    title="Import a single MIDI file"
  >
    <Upload class="h-4 w-4" />
    {importing ? 'Importing...' : 'Import File'}
  </button>

  <button
    on:click={handleImportFolder}
    disabled={importing}
    class="btn-secondary px-4 py-2 flex items-center gap-2"
    title="Import entire folder of MIDI files"
  >
    <FolderOpen class="h-4 w-4" />
    Folder
  </button>
</div>
