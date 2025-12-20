# Stream F: load_file_to_daw - Quick Reference

## Command Signature

```rust
#[tauri::command]
pub async fn load_file_to_daw(
    file_id: i32,
    state: State<'_, DawAppState>,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<i32, String>
```

## Frontend Usage

### TypeScript/JavaScript

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Load single file
const trackId = await invoke<number>('load_file_to_daw', { fileId: 123 });
console.log('Loaded track:', trackId); // Output: 1

// Load with error handling
try {
  const trackId = await invoke<number>('load_file_to_daw', { fileId: 456 });
  console.log('✓ Track loaded:', trackId);
} catch (error) {
  console.error('✗ Failed to load:', error);
}
```

### Svelte Component

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';

  async function loadFile(fileId: number) {
    try {
      const trackId = await invoke<number>('load_file_to_daw', { fileId });
      console.log('Track ID:', trackId);
      // Refresh UI or show success notification
    } catch (error) {
      console.error('Error:', error);
      // Show error notification
    }
  }
</script>

<button on:click={() => loadFile(123)}>
  Load File
</button>
```

## Drag & Drop Example

### Source (VIP3Results.svelte)

```svelte
<script lang="ts">
  function handleDragStart(event: DragEvent, fileId: number) {
    event.dataTransfer?.setData('application/midi-file-id', fileId.toString());
    event.dataTransfer!.effectAllowed = 'copy';
  }
</script>

{#each files as file}
  <div
    class="file-item"
    draggable="true"
    on:dragstart={(e) => handleDragStart(e, file.id)}
  >
    {file.name}
  </div>
{/each}

<style>
  .file-item {
    cursor: grab;
  }
  .file-item:active {
    cursor: grabbing;
  }
</style>
```

### Target (Sequencer.svelte)

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';

  let isDragOver = false;

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave() {
    isDragOver = false;
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;

    const fileId = parseInt(
      event.dataTransfer?.getData('application/midi-file-id') || '0'
    );

    if (fileId > 0) {
      try {
        const trackId = await invoke<number>('load_file_to_daw', { fileId });
        console.log(`✓ Loaded file ${fileId} to track ${trackId}`);
        // Refresh track list
        await refreshTracks();
      } catch (error) {
        console.error('Failed to load file:', error);
      }
    }
  }

  async function refreshTracks() {
    const tracks = await invoke('get_tracks');
    console.log('Tracks:', tracks);
    // Update UI
  }
</script>

<div
  class="sequencer"
  class:drag-over={isDragOver}
  on:drop={handleDrop}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
>
  <!-- Sequencer content -->
</div>

<style>
  .sequencer {
    border: 2px dashed transparent;
    transition: all 0.2s;
  }
  .sequencer.drag-over {
    border-color: #4a90e2;
    background-color: rgba(74, 144, 226, 0.1);
  }
</style>
```

## Error Handling

| Error | Cause | User Action |
|-------|-------|-------------|
| `"File not found: {id}"` | Invalid file_id | Check file exists in database |
| `"Failed to load MIDI file: ..."` | Corrupted MIDI | Show error, skip file |
| `"Database pool not initialized"` | DB connection issue | Retry or restart app |

## Related Commands

```typescript
// Get all tracks
const tracks = await invoke('get_tracks');

// Remove a track
await invoke('remove_track', { trackId: 1 });

// Update track properties
await invoke('update_track', {
  trackId: 1,
  properties: { mute: false, solo: false, volume: 1.0, pan: 0.0 }
});
```

## Performance Tips

1. **Batch Loading**: Load files in parallel for better performance
   ```typescript
   const trackIds = await Promise.all(
     fileIds.map(id => invoke('load_file_to_daw', { fileId: id }))
   );
   ```

2. **Debounce Drag Events**: Prevent excessive re-renders during drag
   ```typescript
   let dragTimeout: number;
   function handleDragOver(event: DragEvent) {
     clearTimeout(dragTimeout);
     dragTimeout = setTimeout(() => {
       // Update UI
     }, 50);
   }
   ```

3. **Show Loading State**: Use loading indicator for large files
   ```svelte
   {#if isLoading}
     <div class="loading">Loading track...</div>
   {/if}
   ```

## Testing

### Browser Console

```javascript
// Test load
invoke('load_file_to_daw', { fileId: 1 })
  .then(id => console.log('Track ID:', id))
  .catch(err => console.error('Error:', err));

// Verify
invoke('get_tracks')
  .then(tracks => console.table(tracks));
```

### Manual Test Script

```bash
./scripts/test-load-file-to-daw.sh
```

## Debug Tips

1. **Check File ID**: Verify file exists
   ```sql
   SELECT id, filepath, filename FROM files WHERE id = 123;
   ```

2. **Check File Path**: Verify file is accessible
   ```bash
   ls -lh /path/to/file.mid
   ```

3. **Enable Logging**: Check Tauri logs
   ```typescript
   // In Tauri config, set log level to 'debug'
   ```

4. **Inspect Track**: After loading
   ```javascript
   const tracks = await invoke('get_tracks');
   console.log('Latest track:', tracks[tracks.length - 1]);
   ```

## Support

- Command source: `app/src-tauri/src/commands/daw/sequencer.rs:144-152`
- Tests: `app/src-tauri/tests/test_load_file_to_daw.rs`
- Docs: `docs/STREAM_F_BACKEND_COMPLETE.md`
