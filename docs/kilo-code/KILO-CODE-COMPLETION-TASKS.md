# Kilo Code Frontend - Completion Task List

## Mission: Complete the remaining 15% to reach 100% production-ready status

**Current Status:** 85% complete (3,551 lines, 21 files)
**Estimated Time:** 6-8 hours
**Target:** Production-ready frontend with full reactive state management

---

## Phase 1: Missing Stores Implementation (Priority: CRITICAL)

### Task 1.1: Create Pipeline Store
**File:** `app/src/lib/stores/pipelineStore.ts`
**Estimated Time:** 1 hour

Create a new store to manage pipeline import and analysis operations:

```typescript
// Required state
interface PipelineState {
  operation: 'import' | 'analysis' | null;
  isRunning: boolean;
  isPaused: boolean;
  progress: number;
  currentFile: string;
  processed: number;
  totalFiles: number;
  errors: string[];
  lastResult: ImportSummary | null;
}

// Required actions
- startOperation(operation: 'import' | 'analysis')
- pauseOperation()
- stopOperation()
- updateProgress(progress: ImportProgress)
- setComplete(result: ImportSummary)
- clearErrors()
```

**Reference:** Based on `ImportProgress` and `ImportSummary` types in `app/src/lib/types.ts`

---

### Task 1.2: Create Analysis Store
**File:** `app/src/lib/stores/analysisStore.ts`
**Estimated Time:** 45 minutes

Create a store to manage file analysis operations:

```typescript
// Required state
interface AnalysisState {
  isRunning: boolean;
  progress: number;
  currentFile: string;
  processed: number;
  totalFiles: number;
  results: AnalysisResult[];
  errors: string[];
  lastComplete: AnalysisSummary | null;
}

// Required actions
- startAnalysis()
- updateProgress(progress: AnalysisProgress)
- setComplete(result: AnalysisSummary)
- clearResults()
- addError(error: string)
```

**Note:** Define `AnalysisProgress` and `AnalysisSummary` types in `types.ts` if not already present

---

### Task 1.3: Create Archive Store
**File:** `app/src/lib/stores/archiveStore.ts`
**Estimated Time:** 45 minutes

Create a store to manage archive extraction operations:

```typescript
// Required state
interface ArchiveState {
  isExtracting: boolean;
  progress: number;
  currentArchive: string;
  extracted: number;
  totalFiles: number;
  errors: string[];
  extractedPaths: string[];
}

// Required actions
- startExtraction(archivePath: string)
- updateProgress(progress: ArchiveProgress)
- addError(error: ArchiveError)
- setComplete()
- clearState()
```

**Note:** Define `ArchiveProgress` and `ArchiveError` types in `types.ts` if not already present

---

### Task 1.4: Export Stores from Index
**File:** `app/src/lib/stores/index.ts`
**Estimated Time:** 5 minutes

Add exports for the three new stores:

```typescript
export { pipelineStore, pipelineActions } from './pipelineStore';
export { analysisStore, analysisActions } from './analysisStore';
export { archiveStore, archiveActions } from './archiveStore';
```

---

## Phase 2: Wire Event Handlers (Priority: CRITICAL)

### Task 2.1: Update App.svelte Event Handlers
**File:** `app/src/App.svelte`
**Estimated Time:** 30 minutes

Replace all 7 TODO comments with actual store updates:

**Lines 21-24:** Pipeline Progress
```typescript
onPipelineProgress: (progress) => {
  console.log('Pipeline progress event received:', progress);
  pipelineActions.updateProgress(progress);
},
```

**Lines 25-28:** Pipeline Complete
```typescript
onPipelineComplete: (result) => {
  console.log('Pipeline complete event received:', result);
  pipelineActions.setComplete(result);
},
```

**Lines 29-32:** Analysis Progress
```typescript
onAnalysisProgress: (progress) => {
  console.log('Analysis progress event received:', progress);
  analysisActions.updateProgress(progress);
},
```

**Lines 33-36:** Analysis Complete
```typescript
onAnalysisComplete: (result) => {
  console.log('Analysis complete event received:', result);
  analysisActions.setComplete(result);
},
```

**Lines 37-40:** Archive Progress
```typescript
onArchiveProgress: (progress) => {
  console.log('Archive progress event received:', progress);
  archiveActions.updateProgress(progress);
},
```

**Lines 41-44:** Archive Error
```typescript
onArchiveError: (error) => {
  console.log('Archive error event received:', error);
  archiveActions.addError(error);
},
```

**Lines 45-48:** General Progress
```typescript
onProgressUpdate: (update) => {
  console.log('Progress update event received:', update);
  // Delegate to appropriate store based on operation type
  if (update.operation === 'pipeline') {
    pipelineActions.updateProgress(update);
  } else if (update.operation === 'analysis') {
    analysisActions.updateProgress(update);
  }
},
```

**Don't forget:** Import the new stores at the top of `App.svelte`:
```typescript
import { pipelineActions } from '$lib/stores/pipelineStore';
import { analysisActions } from '$lib/stores/analysisStore';
import { archiveActions } from '$lib/stores/archiveStore';
```

---

## Phase 3: Pipeline Window Backend Integration (Priority: HIGH)

### Task 3.1: Implement Pipeline API Commands
**Options:** Choose ONE approach:

#### Option A: Add New Tauri Commands (if they don't exist)
**Files:** Backend Rust files in `pipeline/src-tauri/src/commands/`

Create three new Tauri commands:
- `start_pipeline_operation(operation: String) -> Result<()>`
- `pause_pipeline_operation() -> Result<()>`
- `stop_pipeline_operation() -> Result<()>`

Then add to `app/src/lib/api.ts`:
```typescript
pipeline: {
  start: async (operation: 'import' | 'analysis'): Promise<void> => {
    await invoke('start_pipeline_operation', { operation });
  },
  pause: async (): Promise<void> => {
    await invoke('pause_pipeline_operation');
  },
  stop: async (): Promise<void> => {
    await invoke('stop_pipeline_operation');
  },
}
```

#### Option B: Use Existing Commands (if they exist)
Research if `file_import`, `analyze_files`, or similar commands already exist and can be called directly. Map them to the pipeline API.

---

### Task 3.2: Update PipelineWindow.svelte
**File:** `app/src/lib/windows/PipelineWindow.svelte`
**Estimated Time:** 30 minutes

Replace placeholder functions with actual API calls:

**Lines 50-58:** startPipeline
```typescript
async function startPipeline() {
  try {
    await api.pipeline.start(selectedOperation);
    pipelineActions.startOperation(selectedOperation);
    isRunning = true;
    isPaused = false;
  } catch (error) {
    console.error('Failed to start pipeline:', error);
  }
}
```

**Lines 60-64:** pausePipeline
```typescript
async function pausePipeline() {
  try {
    await api.pipeline.pause();
    pipelineActions.pauseOperation();
    isPaused = true;
  } catch (error) {
    console.error('Failed to pause pipeline:', error);
  }
}
```

**Lines 66-74:** stopPipeline
```typescript
async function stopPipeline() {
  try {
    await api.pipeline.stop();
    pipelineActions.stopOperation();
    isRunning = false;
    progress = 0;
    currentFile = '';
    processed = 0;
    total = 0;
  } catch (error) {
    console.error('Failed to stop pipeline:', error);
  }
}
```

**Don't forget:** Import pipelineStore and api:
```typescript
import { pipelineStore, pipelineActions } from '$lib/stores/pipelineStore';
import { api } from '$lib/api';
```

**Optional Enhancement:** Subscribe to pipelineStore to update UI reactively:
```typescript
$: ({ isRunning, isPaused, progress, currentFile, processed, total } = $pipelineStore);
```

---

## Phase 4: System Monitoring (Priority: MEDIUM, Optional)

### Task 4.1: Add System Monitoring Backend Commands
**Files:** Backend Rust files (likely `daw/src-tauri/src/commands/system.rs` - create if needed)

**Estimated Time:** 1 hour

Implement two Tauri commands using `sysinfo` crate:
```rust
#[tauri::command]
fn get_cpu_usage() -> f32 {
    // Use sysinfo crate to get CPU usage
}

#[tauri::command]
fn get_ram_usage() -> f32 {
    // Use sysinfo crate to get RAM usage
}
```

Add dependency to `Cargo.toml`:
```toml
sysinfo = "0.30"
```

---

### Task 4.2: Add System API to Frontend
**File:** `app/src/lib/api.ts`
**Estimated Time:** 10 minutes

Add new system section:
```typescript
system: {
  cpuUsage: async (): Promise<number> => {
    try {
      return await invoke('get_cpu_usage');
    } catch (error) {
      console.error('Failed to get CPU usage:', error);
      throw error;
    }
  },

  ramUsage: async (): Promise<number> => {
    try {
      return await invoke('get_ram_usage');
    } catch (error) {
      console.error('Failed to get RAM usage:', error);
      throw error;
    }
  },
}
```

---

### Task 4.3: Update StatusBar Component
**File:** `app/src/lib/components/StatusBar.svelte`
**Estimated Time:** 10 minutes

Replace mock data with real API calls (lines 20-24):

```typescript
pollInterval = setInterval(async () => {
  try {
    cpuUsage = await api.system.cpuUsage();
    ramUsage = await api.system.ramUsage();
  } catch (error) {
    console.error('Failed to poll system usage:', error);
    cpuUsage = 0;
    ramUsage = 0;
  }
}, 1000);
```

**Alternative (if Task 4.1-4.2 too complex):** Remove CPU/RAM display entirely:
```svelte
<!-- System Usage -->
<div class="flex items-center space-x-4">
  <!-- Removed CPU/RAM - not critical for MVP -->
</div>
```

---

## Phase 5: Code Cleanup (Priority: LOW)

### Task 5.1: Remove Debug Console Logs
**Files:** Multiple
**Estimated Time:** 30 minutes

**Keep these error logs:**
- All `console.error()` statements in catch blocks
- Critical warnings

**Remove these debug logs:**
- `app/src/main.ts:1` - "Starting Svelte app initialization"
- `app/src/main.ts:5` - "Svelte App imported, mounting to #app"
- `app/src/main.ts:9` - "Svelte app mounted successfully"
- `app/src/App.simple.svelte:9` - "App loaded - simple version"
- `app/src/App.test.svelte:2` - "App component loaded"
- All `console.log()` statements in `App.svelte` event handlers (17 total)
- All `console.log()` statements in `PipelineWindow.svelte` (3 total)
- Fallback warnings in `playbackStore.ts` (10 total) - **Keep these** if backends might not be implemented

**Strategy:**
1. Search: `grep -r "console\.log" app/src --include="*.ts" --include="*.svelte"`
2. Review each occurrence
3. Remove non-critical debug logs
4. Keep error logs and critical warnings

---

### Task 5.2: Add Environment-Based Logging (Optional)
**File:** Create `app/src/lib/utils/logger.ts`
**Estimated Time:** 20 minutes

Create a logging utility:
```typescript
const isDev = import.meta.env.DEV;

export const logger = {
  debug: (...args: any[]) => {
    if (isDev) console.log('[DEBUG]', ...args);
  },
  info: (...args: any[]) => {
    console.log('[INFO]', ...args);
  },
  warn: (...args: any[]) => {
    console.warn('[WARN]', ...args);
  },
  error: (...args: any[]) => {
    console.error('[ERROR]', ...args);
  },
};
```

Then replace remaining console.log statements with `logger.debug()`.

---

## Phase 6: Polish & Enhancements (Priority: LOW, Optional)

### Task 6.1: Master Volume Control
**File:** `app/src/lib/windows/MixerWindow.svelte`
**Estimated Time:** 20 minutes

Replace placeholder master volume (lines 124-134) with working control:

```typescript
let masterVolume = 1.0;

async function updateMasterVolume(volume: number) {
  masterVolume = volume;
  // If backend has set_master_volume command:
  try {
    await api.window.setMasterVolume(volume);
  } catch (error) {
    console.warn('Master volume control not implemented in backend');
  }
}
```

```svelte
<input
  type="range"
  min="0"
  max="1"
  step="0.01"
  bind:value={masterVolume}
  on:input={(e) => updateMasterVolume(parseFloat(e.currentTarget.value))}
  class="dark:bg-input w-32"
/>
<span class="dark:text-gray-300">{Math.round(masterVolume * 100)}%</span>
```

---

### Task 6.2: Add Loading States
**Files:** All window components
**Estimated Time:** 30 minutes

Add loading spinners/skeletons while data loads:

**Example for DatabaseWindow.svelte:**
```svelte
{#if isLoading}
  <div class="loading-spinner">Loading...</div>
{:else}
  <!-- Existing content -->
{/if}
```

---

### Task 6.3: Error Boundaries
**File:** Create `app/src/lib/components/ErrorBoundary.svelte`
**Estimated Time:** 30 minutes

Wrap main components with error handling:
```svelte
<script lang="ts">
  let error: Error | null = null;

  function handleError(e: ErrorEvent) {
    error = e.error;
    console.error('Error boundary caught:', e.error);
  }
</script>

<svelte:window on:error={handleError} />

{#if error}
  <div class="error-display">
    <h2>Something went wrong</h2>
    <pre>{error.message}</pre>
  </div>
{:else}
  <slot />
{/if}
```

---

## Phase 7: Testing (Priority: LOW, Post-Launch)

### Task 7.1: Add Svelte Testing Library
**Estimated Time:** 4 hours

```bash
pnpm add -D @testing-library/svelte @testing-library/jest-dom vitest
```

Create test files for each store and component:
- `playbackStore.test.ts`
- `projectStore.test.ts`
- `databaseStore.test.ts`
- `pipelineStore.test.ts` (new)
- `analysisStore.test.ts` (new)
- `archiveStore.test.ts` (new)

---

## Verification Checklist

After completing all tasks, verify:

- [ ] All 7 TODOs removed from `App.svelte`
- [ ] Pipeline operations update stores reactively
- [ ] Analysis operations update stores reactively
- [ ] Archive operations update stores reactively
- [ ] Pipeline window controls (start/pause/stop) call backend
- [ ] StatusBar shows real CPU/RAM or display removed
- [ ] No console.log statements (except errors/warnings)
- [ ] All event handlers wire to stores correctly
- [ ] All stores exported from `index.ts`
- [ ] TypeScript compilation: `pnpm run check` passes
- [ ] Build: `pnpm run build` succeeds
- [ ] Manual testing: Open app, trigger pipeline operation, verify UI updates

---

## Expected Outcome

**Before:**
- 85% complete
- 7 TODOs
- 3 mock/placeholder sections
- 30+ debug logs
- Events received but not persisted

**After:**
- 100% production-ready
- 0 TODOs
- Full reactive state management
- Clean console output
- Events → Stores → UI updates work end-to-end

**Total Estimated Time:** 6-8 hours (mandatory tasks only)

---

## Priority Order

**If time-constrained, complete in this order:**

1. **Phase 1 (Tasks 1.1-1.4):** Missing stores - CRITICAL
2. **Phase 2 (Task 2.1):** Wire event handlers - CRITICAL
3. **Phase 3 (Tasks 3.1-3.2):** Pipeline backend - HIGH
4. **Phase 5 (Task 5.1):** Remove debug logs - MEDIUM
5. **Phase 4 (Tasks 4.1-4.3):** System monitoring - LOW (optional)
6. **Phase 6:** Polish - LOW (post-launch)
7. **Phase 7:** Testing - LOW (post-launch)

**Minimum viable completion:** Phases 1-3 (4-5 hours)
**Production-ready:** Phases 1-5 (6-8 hours)
**Full polish:** All phases (12-16 hours)
