# Frontend Performance Profiling

Phase 7C comprehensive frontend optimization system for MIDI Software Center.

## Quick Start

### 1. Enable Profiling in Component

```svelte
<script lang="ts">
  import { useFullProfiler } from '$lib/profiling';

  const { trackRequest, cleanup } = useFullProfiler('MyComponent');
</script>
```

### 2. Track Network Requests

```typescript
const data = await trackRequest(async () => {
  return await invoke('get_files');
});
```

### 3. Virtual Scrolling for Large Lists

```svelte
<script lang="ts">
  import { VirtualScrollHelper } from '$lib/profiling';

  const virtualScroll = new VirtualScrollHelper(600, 50, items.length);

  $: visibleRange = virtualScroll.calculateVisibleRange(scrollTop);
  $: visibleItems = items.slice(visibleRange.start, visibleRange.end);
</script>
```

### 4. Debounced Store Updates

```typescript
import { createDebouncedStore } from '$lib/profiling';

export const playheadPosition = createDebouncedStore(0, 50);
```

### 5. Bundle Analysis

```typescript
// vite.config.ts
import { bundleAnalyzerPlugin } from './src/lib/profiling';

export default defineConfig({
  plugins: [
    sveltekit(),
    bundleAnalyzerPlugin()
  ]
});
```

## Files

- `performance.ts` - Core profiling engine (1,500 lines)
- `hooks.ts` - Svelte hooks (200 lines)
- `vite-plugin.ts` - Bundle analyzer (300 lines)
- `example-optimized-component.svelte` - Reference implementation

## Documentation

See [PHASE-7C-FRONTEND-OPTIMIZATION-GUIDE.md](../../../../PHASE-7C-FRONTEND-OPTIMIZATION-GUIDE.md) for complete documentation.

## Performance Targets

- Bundle Size: <500KB
- FPS: 60
- Memory Usage: <80%
- Cache Hit Rate: >70%
- Average Latency: <500ms

## Usage in Development

```typescript
// Enable in dev mode only
if (import.meta.env.DEV) {
  import { profiler } from '$lib/profiling';

  profiler.getProfilers().memory.startMonitoring(5000);
  profiler.getProfilers().render.startFPSMonitoring();
}
```

## Generate Performance Report

```typescript
import { profiler } from '$lib/profiling';

const report = profiler.generateReport();
console.log(report);
```

## Architecture

Follows the Three Archetypes Pattern:

- **Trusty Modules**: Pure profiling utilities (`performance.ts`)
- **Grown-up Scripts**: Svelte hooks with side effects (`hooks.ts`)
- **Task-O-Matics**: UI components using profiling (your components)

## License

Part of MIDI Software Center - Production Ready System
