# Pipeline Frontend Testing Architecture Report

**Date:** 2025-11-03  
**Project:** MIDI Software Center - Pipeline Frontend (`pipeline/src`)  
**Report Focus:** Testing Framework, Patterns, Mocking Utilities

---

## 1. Testing Framework & Configuration

### 1.1 Framework Stack

**Testing Framework:** **Vitest 1.0+**
- Package: `vitest@^1.0.0`
- UI Testing: `@vitest/ui@^1.0.0`
- No Jest (not in dependencies)

**Related:**
- Coverage Provider: **v8** (via `vitest`)
- Coverage Reporter: text, json, html
- Environment: **jsdom** (browser-like DOM)
- Globals Enabled: true (no need to import `describe`, `it`, `expect`)

### 1.2 Configuration Files

**Primary Config:**
- **Location:** `/home/dojevou/projects/midi-software-center/pipeline/vitest.config.ts`
- **Scope:** Frontend tests only (Svelte components, stores, utilities)

**Vite Config Integration:**
- **Location:** `/home/dojevou/projects/midi-software-center/pipeline/vite.config.ts`
- **Test Config:** Lines 65-80 (Vitest setup within Vite)
- **Test Pattern:** `src/**/*.{test,spec}.{js,ts}`
- **Test Timeout:** 10s (default)

### 1.3 NPM Scripts

```json
{
  "test": "vitest",              // Watch mode
  "test:ui": "vitest --ui",      // Vitest UI dashboard
  "test:coverage": "vitest run --coverage"
}
```

---

## 2. Current Test Status

### 2.1 Existing Tests

**CRITICAL:** No frontend tests exist yet.

```
✗ NO .test.ts files
✗ NO .spec.ts files
✗ NO __tests__ directories
✓ Test infrastructure configured but unused
```

**Directories:**
- `/home/dojevou/projects/midi-software-center/pipeline/tests/` — Empty
- `/home/dojevou/projects/midi-software-center/pipeline/src/` — No test files

### 2.2 Coverage Goals

**Configuration (vitest.config.ts, lines 31-36):**
```typescript
thresholds: {
  lines: 70,
  functions: 70,
  branches: 70,
  statements: 70,
}
```

Target: **70% minimum coverage** across all metrics

---

## 3. Architecture & Code Patterns

### 3.1 Svelte Store Patterns (Testable)

**Store Type:** Writable stores with custom methods

**Example 1: Pure State Manager** (`processing.ts`)
```typescript
// ARCHETYPE: MANAGER (State Management Only - NO I/O)
// Location: pipeline/src/lib/stores/processing.ts

function createProcessingStore() {
  const { subscribe, set, update } = writable<ProcessingState>(initialState);

  return {
    subscribe,
    
    // Pure state mutations (testable without I/O)
    setProcessing: (isProcessing: boolean) => {
      update(state => ({ ...state, isProcessing }));
    },
    
    start: (jobId: string) => {
      update(state => ({
        ...state,
        isProcessing: true,
        startTime: new Date(),
        progress: initialProgress,
      }));
    },
    
    getDuration: (): number => {
      // Synchronous calculation
      let duration = 0;
      update(state => {
        if (state.startTime) {
          duration = Math.floor((endTime.getTime() - state.startTime.getTime()) / 1000);
        }
        return state;
      });
      return duration;
    },
  };
}

export const processingStore = createProcessingStore();
```

**Key Features for Testing:**
- Pure state mutations
- Synchronous operations
- No external dependencies
- Predictable, deterministic behavior
- Factory pattern for store creation

---

**Example 2: Store with I/O** (`import.ts`)
```typescript
// ARCHETYPE: Grown-up Script (State Management + I/O)
// Location: pipeline/src/lib/stores/import.ts

function createImportStore() {
  const { subscribe, set, update } = writable<ImportState>(initialState);
  let unlistenProgress: UnlistenFn | null = null;

  return {
    subscribe,
    
    // File I/O operations (requires mocking)
    pickFile: async (): Promise<string | null> => {
      try {
        const result = await open({ /* ... */ });
        return result;
      } catch (error) {
        update(state => ({
          ...state,
          error: error instanceof Error ? error.message : 'Failed'
        }));
        return null;
      }
    },
    
    // Tauri Event Listener (requires mocking)
    importFile: async (filePath: string): Promise<number | null> => {
      update(state => ({ ...state, importing: true, error: null }));
      
      try {
        await startProgressListener(update); // Registers listener
        const fileId = await importSingleFile(filePath); // API call
        update(state => ({ ...state, importing: false }));
        if (unlistenProgress) unlistenProgress(); // Cleanup
        return fileId;
      } catch (error) {
        // Error handling + cleanup
      }
    },
  };
}
```

**Mocking Requirements:**
- `@tauri-apps/plugin-dialog` → `open()` dialog
- `@tauri-apps/api/event` → `listen()` event listeners
- API calls → `importSingleFile()` function
- Cleanup → `UnlistenFn` callbacks

---

### 3.2 API Layer Patterns (Testable with Mocks)

**Client Layer:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/api/client.ts`

```typescript
// Base API client with error handling and retry logic
export class ApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: unknown,
    public isRetryable: boolean = false
  ) {
    super(message);
  }
}

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  const MAX_RETRIES = 3;
  const TIMEOUT_MS = 10000;
  
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    try {
      // Exponential backoff
      if (attempt > 0) {
        const delayMs = 1000 * Math.pow(2, attempt - 1);
        await sleep(delayMs);
      }

      // Timeout handling with Promise.race
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(new Error('Request timeout'));
        }, TIMEOUT_MS);
      });

      const result = await Promise.race([
        invoke<T>(command, args), // Tauri IPC
        timeoutPromise
      ]);

      setConnectionStatus('connected');
      return result;
    } catch (error) {
      const shouldRetry = attempt < MAX_RETRIES && isRetryableError(error);
      if (!shouldRetry) throw createUserFriendlyError(error, command);
    }
  }
}

// Connection monitoring
export function startConnectionMonitoring(intervalMs = 30000): () => void {
  const intervalId = setInterval(() => testConnection(), intervalMs);
  return () => clearInterval(intervalId);
}
```

**Test Requirements:**
- Mock `invoke()` from `@tauri-apps/api/core`
- Test retry logic (3 attempts, exponential backoff)
- Test timeout handling (10s timeout)
- Test connection status store updates
- Test error classification (retryable vs non-retryable)
- Test error message transformation

---

**Processing API:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/api/processing.ts`

```typescript
// Simple wrapper around invokeCommand
export async function startProcessing(config: ProcessingConfig): Promise<string> {
  return invokeCommand<string>('start_processing', { config });
}

export async function getProcessingProgress(): Promise<ProcessingProgress> {
  return invokeCommand<ProcessingProgress>('get_processing_progress');
}

export async function pauseProcessing(): Promise<void> {
  return invokeCommand<void>('pause_processing');
}
```

**Test Requirements:**
- Mock `invokeCommand()`
- Verify correct command names passed
- Verify response types
- Test error propagation

---

### 3.3 Svelte Component Patterns (Testable)

**Container Component:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/components/Import/ImportProgress.svelte`

```svelte
<script lang="ts">
  import { importStore } from '$lib/stores/import';

  export let onComplete: () => void;

  // Reactive declarations (pure data flow, no I/O)
  $: progress = $importStore.progress;
  $: importing = $importStore.importing;

  // Computed values
  $: percentage = progress
    ? (progress.current / progress.total) * 100
    : 0;

  // Reactive side effect
  $: if (progress && progress.current === progress.total && !importing) {
    setTimeout(onComplete, 1000);
  }
</script>

{#if progress}
  <div class="card p-4">
    <div class="flex items-center justify-between mb-2">
      <span class="text-sm font-medium">
        Importing files... ({progress.current}/{progress.total})
      </span>
      <span class="text-sm text-slate-600">{Math.round(percentage)}%</span>
    </div>
    <div class="w-full bg-slate-200 rounded-full h-2">
      <div
        class="bg-primary-600 h-2 rounded-full transition-all"
        style="width: {percentage}%"
      />
    </div>
    <p class="mt-2 text-xs text-slate-600 truncate">{progress.fileName}</p>
  </div>
{/if}
```

**ARCHETYPE:** CONTAINER (composition + layout, NO I/O)
- Reactive to store changes
- Computed display values
- Callback-based completion handling
- No direct API calls

**Test Requirements:**
- Render component with store data
- Verify progress bar width calculation
- Test completion callback
- Test conditional rendering

---

### 3.4 Utility Functions (Pure)

**Validators:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/utils/validators.ts`
**Formatters:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/utils/formatters.ts`

These are likely pure functions (no side effects, no I/O) → Straightforward unit tests

---

## 4. Mocking Strategy for Tauri IPC

### 4.1 Mocking `invoke()` Command

**Source:** `@tauri-apps/api/core`

**Setup Pattern:**

```typescript
import { vi } from 'vitest';

// Mock the Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

// In tests:
const mockInvoke = invoke as ReturnType<typeof vi.fn>;

// Mock successful response
mockInvoke.mockResolvedValueOnce({
  id: 123,
  name: 'test.mid',
  bpm: 120,
});

// Mock error
mockInvoke.mockRejectedValueOnce(new Error('Connection timeout'));

// Verify calls
expect(mockInvoke).toHaveBeenCalledWith('start_processing', { config });
```

**Key Points:**
- Tauri's `invoke()` is the IPC bridge
- Every Rust command is invoked via `invoke(command, args)`
- Mock at the module level (before importing client)
- Verify command names and arguments

---

### 4.2 Mocking `listen()` Events

**Source:** `@tauri-apps/api/event`

**Setup Pattern:**

```typescript
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { listen } from '@tauri-apps/api/event';

const mockListen = listen as ReturnType<typeof vi.fn>;

// Mock event listener registration
mockListen.mockResolvedValueOnce(() => {
  // unlisten callback
});

// In store tests:
const unlistenFn = await listen('import-progress', callback);
expect(mockListen).toHaveBeenCalledWith('import-progress', expect.any(Function));

// Simulate event emission
const mockCallback = vi.fn();
mockListen.mockImplementation(async (eventName, callback) => {
  // Immediately fire event for testing
  callback({ payload: { current: 50, total: 100 } });
  return () => {}; // unlisten function
});
```

**Key Points:**
- Events are subscribed via `listen(eventName, callback)`
- Returns an async unlisten function
- Test both listener registration and event handling
- Simulate event emissions in tests

---

### 4.3 Mocking File Dialog

**Source:** `@tauri-apps/plugin-dialog`

**Setup Pattern:**

```typescript
import { vi } from 'vitest';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

import { open } from '@tauri-apps/plugin-dialog';

const mockOpen = open as ReturnType<typeof vi.fn>;

// Mock file selection
mockOpen.mockResolvedValueOnce('/path/to/file.mid');

// Mock directory selection
mockOpen.mockResolvedValueOnce('/path/to/directory');

// Mock cancelled dialog
mockOpen.mockResolvedValueOnce(null);

// Mock error
mockOpen.mockRejectedValueOnce(new Error('Dialog cancelled'));
```

**Key Points:**
- `open()` returns string (single file) or string[] (multiple)
- Returns null if user cancels
- Verify filter options in call arguments

---

### 4.4 Mocking Svelte Stores

**Pattern for Store Testing:**

```typescript
import { get } from 'svelte/store';

// Subscribe to store
let storeValue: ProcessingState;
const unsubscribe = processingStore.subscribe(value => {
  storeValue = value;
});

// Trigger store mutation
processingStore.start('job-123');

// Verify new state
expect(storeValue.isProcessing).toBe(true);
expect(storeValue.jobId).toBe('job-123');

// Cleanup
unsubscribe();
```

**Advanced Pattern with Writable:**

```typescript
import { get } from 'svelte/store';

// Get current value synchronously
const currentState = get(processingStore);
expect(currentState.phase).toBe('idle');

// Update via store method
processingStore.setPhase('analyzing');

// Verify
const updated = get(processingStore);
expect(updated.phase).toBe('analyzing');
```

---

## 5. Test File Structure & Patterns

### 5.1 Recommended Directory Layout

```
pipeline/src/
├── __tests__/
│   ├── unit/
│   │   ├── utils/
│   │   │   ├── validators.test.ts
│   │   │   └── formatters.test.ts
│   │   ├── stores/
│   │   │   ├── processing.test.ts
│   │   │   ├── import.test.ts
│   │   │   ├── files.test.ts
│   │   │   └── ui.test.ts
│   │   └── api/
│   │       ├── client.test.ts
│   │       ├── processing.test.ts
│   │       └── files.test.ts
│   ├── integration/
│   │   ├── import-workflow.test.ts
│   │   ├── search-workflow.test.ts
│   │   └── processing-workflow.test.ts
│   └── fixtures/
│       └── mock-data.ts
├── lib/
├── routes/
└── App.svelte
```

**Alternative Pattern (per-file):**
```
pipeline/src/
├── lib/
│   ├── stores/
│   │   ├── processing.ts
│   │   ├── processing.test.ts      ← Next to source
│   │   └── ...
│   └── api/
│       ├── client.ts
│       ├── client.test.ts           ← Next to source
│       └── ...
```

---

### 5.2 Store Test Pattern

**File:** `pipeline/src/__tests__/unit/stores/processing.test.ts`

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { processingStore } from '$lib/stores/processing';

describe('processingStore', () => {
  beforeEach(() => {
    processingStore.reset();
  });

  describe('initialization', () => {
    it('should initialize with idle state', () => {
      const state = get(processingStore);
      expect(state.isProcessing).toBe(false);
      expect(state.phase).toBe('idle');
      expect(state.error).toBe(null);
    });
  });

  describe('start()', () => {
    it('should start processing with job ID', () => {
      processingStore.start('job-123');
      const state = get(processingStore);
      expect(state.isProcessing).toBe(true);
      expect(state.jobId).toBe('job-123');
      expect(state.startTime).toBeInstanceOf(Date);
    });

    it('should reset progress to initial state', () => {
      processingStore.start('job-123');
      const state = get(processingStore);
      expect(state.progress.percentage).toBe(0);
      expect(state.progress.phase).toBe('idle');
    });
  });

  describe('setPhase()', () => {
    it('should update phase while preserving other state', () => {
      processingStore.start('job-123');
      processingStore.setPhase('analyzing');
      
      const state = get(processingStore);
      expect(state.progress.phase).toBe('analyzing');
      expect(state.jobId).toBe('job-123');
    });
  });

  describe('complete()', () => {
    it('should mark processing as complete', () => {
      processingStore.start('job-123');
      processingStore.complete();
      
      const state = get(processingStore);
      expect(state.isProcessing).toBe(false);
      expect(state.progress.phase).toBe('completed');
      expect(state.progress.percentage).toBe(100);
      expect(state.endTime).toBeInstanceOf(Date);
    });
  });

  describe('getDuration()', () => {
    it('should calculate duration in seconds', async () => {
      processingStore.start('job-123');
      
      // Simulate delay
      await new Promise(resolve => setTimeout(resolve, 100));
      
      processingStore.complete();
      const duration = processingStore.getDuration();
      
      expect(duration).toBeGreaterThanOrEqual(0);
      expect(typeof duration).toBe('number');
    });
  });

  describe('error handling', () => {
    it('should set error and fail state', () => {
      processingStore.start('job-123');
      processingStore.fail('Processing failed: invalid format');
      
      const state = get(processingStore);
      expect(state.error).toBe('Processing failed: invalid format');
      expect(state.progress.phase).toBe('error');
      expect(state.isProcessing).toBe(false);
    });

    it('should allow clearing errors', () => {
      processingStore.fail('Error message');
      processingStore.clearError();
      
      const state = get(processingStore);
      expect(state.error).toBeNull();
    });
  });

  describe('subscription', () => {
    it('should notify subscribers on state changes', () => {
      const values: ProcessingState[] = [];
      const unsubscribe = processingStore.subscribe(value => {
        values.push(value);
      });

      processingStore.start('job-123');
      processingStore.setPhase('analyzing');

      expect(values.length).toBeGreaterThan(1);
      expect(values[1].isProcessing).toBe(true);
      
      unsubscribe();
    });
  });
});
```

---

### 5.3 API Client Test Pattern

**File:** `pipeline/src/__tests__/unit/api/client.test.ts`

```typescript
import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { invokeCommand, ApiError, isApiError, canRetry } from '$lib/api/client';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('API Client', () => {
  const mockInvoke = invoke as ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockInvoke.mockClear();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('invokeCommand()', () => {
    it('should invoke Tauri command and return result', async () => {
      mockInvoke.mockResolvedValueOnce({ id: 123, name: 'test' });

      const result = await invokeCommand('test_command', { arg: 'value' });

      expect(result).toEqual({ id: 123, name: 'test' });
      expect(mockInvoke).toHaveBeenCalledWith('test_command', { arg: 'value' });
    });

    it('should retry on transient errors', async () => {
      mockInvoke
        .mockRejectedValueOnce(new Error('Connection timeout'))
        .mockRejectedValueOnce(new Error('Pool busy'))
        .mockResolvedValueOnce({ id: 123 });

      const result = await invokeCommand('test_command');

      expect(result).toEqual({ id: 123 });
      expect(mockInvoke).toHaveBeenCalledTimes(3);
    });

    it('should fail immediately on non-retryable errors', async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error('Not found')
      );

      await expect(
        invokeCommand('test_command')
      ).rejects.toThrow('not found');
    });

    it('should apply exponential backoff between retries', async () => {
      mockInvoke
        .mockRejectedValueOnce(new Error('Connection timeout'))
        .mockResolvedValueOnce({ id: 123 });

      const startTime = Date.now();
      await invokeCommand('test_command');
      
      // Should have waited ~1000ms for first retry
      vi.advanceTimersByTime(1000);
      
      expect(Date.now() - startTime).toBeGreaterThanOrEqual(1000);
    });

    it('should timeout after 10 seconds', async () => {
      mockInvoke.mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 15000))
      );

      const promise = invokeCommand('test_command');
      vi.advanceTimersByTime(10000);

      await expect(promise).rejects.toThrow('timeout');
    });
  });

  describe('ApiError', () => {
    it('should create error with message and code', () => {
      const error = new ApiError('Test error', 'TEST_CODE');
      expect(error.message).toBe('Test error');
      expect(error.code).toBe('TEST_CODE');
      expect(error.isRetryable).toBe(false);
    });

    it('should mark retryable errors', () => {
      const error = new ApiError('Timeout', undefined, undefined, true);
      expect(error.isRetryable).toBe(true);
    });
  });

  describe('isApiError()', () => {
    it('should identify ApiError instances', () => {
      const error = new ApiError('Test');
      expect(isApiError(error)).toBe(true);
      expect(isApiError(new Error('Test'))).toBe(false);
      expect(isApiError('string')).toBe(false);
    });
  });

  describe('canRetry()', () => {
    it('should return true for retryable errors', () => {
      const error = new ApiError('Timeout', undefined, undefined, true);
      expect(canRetry(error)).toBe(true);
    });

    it('should return false for non-retryable errors', () => {
      const error = new ApiError('Not found');
      expect(canRetry(error)).toBe(false);
    });

    it('should detect retryable error messages', () => {
      expect(canRetry(new Error('Connection reset'))).toBe(true);
      expect(canRetry(new Error('Pool unavailable'))).toBe(true);
    });
  });
});
```

---

### 5.4 Store with I/O Test Pattern (Import Store)

**File:** `pipeline/src/__tests__/unit/stores/import.test.ts`

```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { importStore } from '$lib/stores/import';

// Mock Tauri APIs
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Mock API functions
vi.mock('$lib/api/files', () => ({
  importSingleFile: vi.fn(),
  importDirectory: vi.fn(),
}));

import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { importSingleFile, importDirectory } from '$lib/api/files';

describe('importStore', () => {
  const mockOpen = open as ReturnType<typeof vi.fn>;
  const mockListen = listen as ReturnType<typeof vi.fn>;
  const mockImportSingleFile = importSingleFile as ReturnType<typeof vi.fn>;
  const mockImportDirectory = importDirectory as ReturnType<typeof vi.fn>;

  beforeEach(() => {
    importStore.reset();
    mockOpen.mockClear();
    mockListen.mockClear();
    mockImportSingleFile.mockClear();
    mockImportDirectory.mockClear();
  });

  describe('pickFile()', () => {
    it('should open file dialog and return selected file', async () => {
      mockOpen.mockResolvedValueOnce('/path/to/file.mid');

      const result = await importStore.pickFile();

      expect(result).toBe('/path/to/file.mid');
      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        filters: [{ name: 'MIDI Files', extensions: ['mid', 'midi'] }],
      });
    });

    it('should handle dialog cancellation', async () => {
      mockOpen.mockResolvedValueOnce(null);

      const result = await importStore.pickFile();

      expect(result).toBeNull();
    });

    it('should set error on dialog failure', async () => {
      mockOpen.mockRejectedValueOnce(new Error('Dialog failed'));

      const result = await importStore.pickFile();

      expect(result).toBeNull();
      const state = get(importStore);
      expect(state.error).toBe('Dialog failed');
    });

    it('should handle array return from open()', async () => {
      mockOpen.mockResolvedValueOnce(['/path/to/file.mid']);

      const result = await importStore.pickFile();

      expect(result).toBe('/path/to/file.mid');
    });
  });

  describe('importFile()', () => {
    it('should set importing flag and call API', async () => {
      mockListen.mockResolvedValueOnce(() => {}); // unlisten fn
      mockImportSingleFile.mockResolvedValueOnce(123);

      const result = await importStore.importFile('/path/to/file.mid');

      expect(result).toBe(123);
      const state = get(importStore);
      expect(state.importing).toBe(false);
    });

    it('should listen for progress events', async () => {
      mockListen.mockResolvedValueOnce(() => {});
      mockImportSingleFile.mockResolvedValueOnce(123);

      await importStore.importFile('/path/to/file.mid');

      expect(mockListen).toHaveBeenCalledWith(
        'import-progress',
        expect.any(Function)
      );
    });

    it('should update store on progress event', async () => {
      let progressCallback: ((event: any) => void) | null = null;

      mockListen.mockImplementationOnce(async (eventName, callback) => {
        progressCallback = callback;
        return () => {}; // unlisten
      });

      mockImportSingleFile.mockImplementationOnce(async () => {
        // Simulate progress event
        if (progressCallback) {
          progressCallback({ payload: { current: 50, total: 100 } });
        }
        return 123;
      });

      await importStore.importFile('/path/to/file.mid');

      const state = get(importStore);
      expect(state.progress?.current).toBe(50);
      expect(state.progress?.total).toBe(100);
    });

    it('should handle import errors', async () => {
      mockListen.mockResolvedValueOnce(() => {});
      mockImportSingleFile.mockRejectedValueOnce(
        new Error('Import failed: invalid format')
      );

      const result = await importStore.importFile('/path/to/file.mid');

      expect(result).toBeNull();
      const state = get(importStore);
      expect(state.error).toBe('Import failed: invalid format');
      expect(state.importing).toBe(false);
    });

    it('should cleanup listeners on error', async () => {
      const mockUnlisten = vi.fn();
      mockListen.mockResolvedValueOnce(mockUnlisten);
      mockImportSingleFile.mockRejectedValueOnce(new Error('Failed'));

      await importStore.importFile('/path/to/file.mid');

      expect(mockUnlisten).toHaveBeenCalled();
    });
  });

  describe('pickAndImport()', () => {
    it('should pick file and import it', async () => {
      mockOpen.mockResolvedValueOnce('/path/to/file.mid');
      mockListen.mockResolvedValueOnce(() => {});
      mockImportSingleFile.mockResolvedValueOnce(123);

      const result = await importStore.pickAndImport();

      expect(result).toBe(123);
    });

    it('should return null if user cancels file picker', async () => {
      mockOpen.mockResolvedValueOnce(null);

      const result = await importStore.pickAndImport();

      expect(result).toBeNull();
      expect(mockImportSingleFile).not.toHaveBeenCalled();
    });
  });

  describe('clearError()', () => {
    it('should clear error message', () => {
      importStore.reset();
      let state = get(importStore);
      // Set error via internal mechanism first
      // Then test clear
      importStore.clearError();
      state = get(importStore);
      expect(state.error).toBeNull();
    });
  });
});
```

---

### 5.5 Svelte Component Test Pattern

**File:** `pipeline/src/__tests__/unit/components/ImportProgress.test.ts`

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import ImportProgress from '$lib/components/Import/ImportProgress.svelte';
import { importStore } from '$lib/stores/import';

// Note: @testing-library/svelte is NOT currently installed
// This shows the pattern when/if component testing is added

describe('ImportProgress.svelte', () => {
  it('should not render when no progress', () => {
    const { container } = render(ImportProgress, {
      props: { onComplete: vi.fn() }
    });
    expect(container.querySelector('.card')).toBeNull();
  });

  it('should display progress bar and percentage', () => {
    // Update store with progress
    importStore.reset();
    // Programmatically set progress (would need store modification)
    
    render(ImportProgress, {
      props: { onComplete: vi.fn() }
    });

    // Check progress display
    expect(screen.getByText(/Importing files.../)).toBeInTheDocument();
  });

  it('should calculate percentage correctly', () => {
    render(ImportProgress, {
      props: { onComplete: vi.fn() }
    });

    // Verify width calculation: (current / total) * 100
    const progressBar = screen.getByRole('progressbar');
    expect(progressBar).toHaveStyle('width: 50%'); // 50/100 = 50%
  });

  it('should call onComplete when done', async () => {
    const onComplete = vi.fn();
    render(ImportProgress, {
      props: { onComplete }
    });

    // Simulate completion
    // Wait for timeout in component
    await new Promise(resolve => setTimeout(resolve, 1100));

    expect(onComplete).toHaveBeenCalled();
  });
});
```

**Note:** Component testing requires additional setup (see section 6.2)

---

## 6. Testing Setup Requirements

### 6.1 Vitest Configuration Enhancement

**Current `vitest.config.ts` is good.** Additional dependencies needed:

```json
{
  "devDependencies": {
    "vitest": "^1.0.0",
    "@vitest/ui": "^1.0.0",
    "@testing-library/svelte": "^4.0.0",  // For component tests
    "@testing-library/dom": "^9.0.0",      // DOM utilities
    "jsdom": "^22.0.0",                    // DOM emulation (already in vite.config)
    "@types/node": "^20.10.0",             // Node types
    "happy-dom": "^12.0.0"                 // Optional: lighter DOM alternative
  }
}
```

### 6.2 Setup Files

**Create:** `pipeline/src/__tests__/setup.ts`

```typescript
import { expect, afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/svelte';

// Cleanup after each test
afterEach(() => {
  cleanup();
});

// Global test utilities
global.matchMedia = global.matchMedia || function() {
  return {
    addListener: vi.fn(),
    removeListener: vi.fn(),
  };
};

// Mock window.alert
global.alert = vi.fn();

// Mock console methods if needed
const originalError = console.error;
beforeAll(() => {
  console.error = vi.fn();
});

afterAll(() => {
  console.error = originalError;
});
```

**Update `vitest.config.ts`:**

```typescript
export default defineConfig({
  // ... existing config ...
  test: {
    // ... existing config ...
    setupFiles: ['src/__tests__/setup.ts'],  // Add this
  },
});
```

---

### 6.3 Mock Data Fixtures

**Create:** `pipeline/src/__tests__/fixtures/mock-data.ts`

```typescript
import type { ProcessingProgress, ProcessingState } from '$lib/stores/processing';
import type { ImportState } from '$lib/stores/import';

export const mockProcessingProgress: ProcessingProgress = {
  phase: 'analyzing',
  currentFile: 'song.mid',
  currentFileIndex: 50,
  totalFiles: 100,
  percentage: 50,
  speed: 2.5,
  estimatedTimeRemaining: 20,
  stats: {
    filesScanned: 100,
    filesDecompressed: 0,
    filesAnalyzed: 50,
    filesSplit: 0,
    filesRenamed: 0,
    filesIndexed: 50,
    duplicatesSkipped: 0,
    errorsEncountered: 0,
  },
};

export const mockProcessingState: ProcessingState = {
  isProcessing: true,
  isPaused: false,
  progress: mockProcessingProgress,
  jobId: 'job-123',
  error: null,
  startTime: new Date(),
  endTime: null,
};

export const mockImportState: ImportState = {
  importing: false,
  progress: null,
  error: null,
};
```

---

## 7. Integration Test Patterns

### 7.1 Import Workflow Test

**File:** `pipeline/src/__tests__/integration/import-workflow.test.ts`

```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { get } from 'svelte/store';
import { importStore } from '$lib/stores/import';
import { processingStore } from '$lib/stores/processing';

vi.mock('@tauri-apps/plugin-dialog');
vi.mock('@tauri-apps/api/event');
vi.mock('@tauri-apps/api/core');
vi.mock('$lib/api/files');

describe('Import Workflow', () => {
  beforeEach(() => {
    importStore.reset();
    processingStore.reset();
  });

  it('should complete full import workflow', async () => {
    // 1. User picks file
    // 2. File is imported
    // 3. Progress updates
    // 4. Completion callback fires
    
    const onComplete = vi.fn();
    
    // ... workflow steps ...
  });

  it('should handle errors gracefully', async () => {
    // ... error handling steps ...
  });
});
```

---

## 8. Summary & Recommendations

### Current State
- ✅ **Vitest configured** — Ready to write tests
- ✅ **Vite + SvelteKit integration** — No additional build setup needed
- ✅ **Store architecture designed for testing** — Pure state managers + I/O managers separated
- ✅ **Tauri IPC layer abstracted** — Mockable at module level
- ❌ **No frontend tests exist** — Green field for test creation

### Test Implementation Path

**Phase 1: Unit Tests (Utilities & Stores)**
- 10-15 tests for pure utility functions (formatters, validators)
- 20-30 tests for processing store (pure state manager)
- 15-25 tests for client API (retry logic, error handling)
- **Estimated:** 50-70 tests, ~1-2 hours

**Phase 2: Store with I/O Tests (Import Store)**
- 15-20 tests for import store (file dialog, listener, API calls)
- Mock Tauri `open()`, `listen()`, and API functions
- **Estimated:** 15-20 tests, ~1-2 hours

**Phase 3: Component Tests (ImportProgress, others)**
- Requires `@testing-library/svelte` installation
- 10-15 tests for container components
- 5-10 tests for presentational components
- **Estimated:** 20-30 tests, ~2-3 hours

**Phase 4: Integration Tests (Workflows)**
- End-to-end workflows: import → progress → completion
- Search workflow
- File management workflow
- **Estimated:** 10-15 tests, ~2-3 hours

**Total Estimate:** 95-150 tests, 8-10 hours of development

### Coverage Target

**Goal:** 70%+ across lines, functions, branches, statements

**Key Files to Test:**
- `$lib/stores/*.ts` (7 files) → ~150 lines
- `$lib/api/*.ts` (8 files) → ~300 lines
- `$lib/utils/*.ts` (2 files) → ~100 lines
- Selected components → ~500 lines
- **Total:** ~1,050 testable lines (excluding node_modules, type definitions)

### Mocking Libraries Needed

```json
{
  "devDependencies": {
    "vitest": "^1.0.0",
    "@vitest/ui": "^1.0.0",
    "@testing-library/svelte": "^4.0.0",
    "@testing-library/dom": "^9.0.0",
    "jsdom": "^22.0.0"
  }
}
```

### Key Testing Patterns

1. **Store testing:** Use `get(store)` + `subscribe()` + expect mutations
2. **API testing:** Mock `invoke()` at module level, test retry logic
3. **Event testing:** Mock `listen()`, simulate event callbacks
4. **Dialog testing:** Mock `open()` with various return types
5. **Component testing:** Use `@testing-library/svelte` + reactive assertions

---

## File Paths Reference

**Configuration:**
- `/home/dojevou/projects/midi-software-center/pipeline/vitest.config.ts`
- `/home/dojevou/projects/midi-software-center/pipeline/vite.config.ts`
- `/home/dojevou/projects/midi-software-center/pipeline/package.json`
- `/home/dojevou/projects/midi-software-center/pipeline/tsconfig.json`

**Source Files:**
- Stores: `pipeline/src/lib/stores/*.ts`
- API: `pipeline/src/lib/api/*.ts`
- Utils: `pipeline/src/lib/utils/*.ts`
- Components: `pipeline/src/lib/components/` (Svelte)

**Test Location (recommended):**
- `pipeline/src/__tests__/unit/` — Unit tests
- `pipeline/src/__tests__/integration/` — Integration tests
- `pipeline/src/__tests__/fixtures/` — Mock data

---

**Report Generated:** 2025-11-03  
**Test Framework:** Vitest 1.0  
**Coverage Target:** 70%  
**Estimated Effort:** 8-10 hours (95-150 tests)
