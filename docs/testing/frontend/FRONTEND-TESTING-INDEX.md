# Frontend Testing Documentation Index

## Overview

This project has **NO frontend tests** currently, but has:
- ✓ Full Vitest configuration (1.0+)
- ✓ Well-organized, testable code (stores, API layer, components)
- ✓ Comprehensive testing infrastructure ready to use
- ✓ Clear separation of concerns (pure state vs I/O)

## Documents

### 1. Quick Start Guide (START HERE)
**File:** `FRONTEND-TESTING-QUICK-START.md`

Fast reference with:
- Testing framework overview (Vitest)
- Mocking strategies (invoke, listen, open)
- Code patterns to test
- Running tests commands
- Recommended test structure

**Time to read:** 5-10 minutes

---

### 2. Comprehensive Architecture Guide
**File:** `FRONTEND-TESTING-ARCHITECTURE.md`

Deep dive covering:
- Complete Vitest configuration (vitest.config.ts, vite.config.ts)
- All code patterns with examples (stores, API, components)
- Detailed mocking patterns for each Tauri API
- Full test file templates (copy-paste ready)
- Setup requirements and dependencies
- Integration test patterns
- Coverage goals and implementation plan

**Time to read:** 30-45 minutes
**Best for:** Planning test implementation, understanding patterns

---

## Testing Framework at a Glance

| Aspect | Details |
|--------|---------|
| **Framework** | Vitest 1.0+ (NOT Jest) |
| **Config** | `pipeline/vitest.config.ts` |
| **Test Pattern** | `src/**/*.{test,spec}.{js,ts}` |
| **Coverage Target** | 70%+ (lines, functions, branches, statements) |
| **Environment** | jsdom (browser-like DOM) |
| **Current Tests** | 0 (infrastructure ready) |

---

## Source Files Needing Tests

### Stores (7 files, ~200 lines)
- `pipeline/src/lib/stores/processing.ts` — Pure state (EASY)
- `pipeline/src/lib/stores/import.ts` — With I/O (MEDIUM)
- `pipeline/src/lib/stores/files.ts`
- `pipeline/src/lib/stores/search.ts`
- `pipeline/src/lib/stores/ui.ts`
- `pipeline/src/lib/stores/tags.ts`
- `pipeline/src/lib/stores/stats.ts`

### API (8 files, ~300 lines)
- `pipeline/src/lib/api/client.ts` — Core with retry/timeout (MEDIUM)
- `pipeline/src/lib/api/processing.ts` — Wrapper (EASY)
- `pipeline/src/lib/api/files.ts`
- `pipeline/src/lib/api/search.ts`
- `pipeline/src/lib/api/events.ts`
- `pipeline/src/lib/api/system.ts`
- `pipeline/src/lib/api/connection.ts`
- `pipeline/src/lib/api/stats.ts`

### Utilities (2 files, ~100 lines)
- `pipeline/src/lib/utils/validators.ts` — Pure (VERY EASY)
- `pipeline/src/lib/utils/formatters.ts` — Pure (VERY EASY)

### Components (20+ Svelte files)
- `pipeline/src/lib/components/Import/ImportProgress.svelte`
- `pipeline/src/lib/components/FileBrowser/FileBrowser.svelte`
- `pipeline/src/lib/components/Search/SearchBar.svelte`
- ... and more

---

## Test Implementation Path

### Phase 1: Utilities (15 tests, 1-2 hours)
```typescript
// pipeline/src/__tests__/unit/utils/validators.test.ts
// pipeline/src/__tests__/unit/utils/formatters.test.ts
```
- No mocking required
- Pure functions (easiest)
- High confidence quick wins

### Phase 2: Pure Stores (30+ tests, 2-3 hours)
```typescript
// pipeline/src/__tests__/unit/stores/processing.test.ts
// pipeline/src/__tests__/unit/stores/ui.test.ts
```
- No mocking required (pure state)
- Use `get(store)` + expect patterns
- Test state mutations

### Phase 3: API Client (25 tests, 2-3 hours)
```typescript
// pipeline/src/__tests__/unit/api/client.test.ts
```
- Mock `invoke()` from @tauri-apps/api/core
- Test retry logic (3 attempts)
- Test timeout (10s)
- Test exponential backoff
- Test error classification

### Phase 4: Stores with I/O (20+ tests, 2-3 hours)
```typescript
// pipeline/src/__tests__/unit/stores/import.test.ts
```
- Mock: `open()`, `listen()`, API functions
- Test file dialogs
- Test event listeners
- Test progress tracking

### Phase 5: Components (20+ tests, 2-3 hours)
```typescript
// pipeline/src/__tests__/unit/components/ImportProgress.test.ts
// Requires: @testing-library/svelte
```
- Render components
- Update stores
- Verify reactive changes

### Phase 6: Integration (10+ tests, 2-3 hours)
```typescript
// pipeline/src/__tests__/integration/import-workflow.test.ts
```
- End-to-end workflows
- File selection → import → progress → completion
- Search workflows
- Multi-step operations

**Total Estimate:** 120-150 tests, 8-10 hours development

---

## Mocking Cheat Sheet

### Tauri IPC Command
```typescript
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
const mockInvoke = invoke as ReturnType<typeof vi.fn>;

mockInvoke.mockResolvedValueOnce({ id: 123 });
mockInvoke.mockRejectedValueOnce(new Error('timeout'));
```

### Tauri Events
```typescript
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { listen } from '@tauri-apps/api/event';
const mockListen = listen as ReturnType<typeof vi.fn>;

mockListen.mockResolvedValueOnce(() => {}); // unlisten fn

// Simulate event
mockListen.mockImplementation(async (name, cb) => {
  cb({ payload: { /* data */ } });
  return () => {};
});
```

### File Dialogs
```typescript
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

import { open } from '@tauri-apps/plugin-dialog';
const mockOpen = open as ReturnType<typeof vi.fn>;

mockOpen.mockResolvedValueOnce('/path/to/file.mid');
mockOpen.mockResolvedValueOnce(null); // cancelled
mockOpen.mockRejectedValueOnce(new Error('Failed'));
```

### API Functions
```typescript
vi.mock('$lib/api/files', () => ({
  importSingleFile: vi.fn(),
  importDirectory: vi.fn(),
}));

import { importSingleFile } from '$lib/api/files';
const mockImport = importSingleFile as ReturnType<typeof vi.fn>;

mockImport.mockResolvedValueOnce(123);
```

---

## Key Patterns

### Testing Stores
```typescript
import { get } from 'svelte/store';
import { processingStore } from '$lib/stores/processing';

processingStore.start('job-123');
const state = get(processingStore);
expect(state.isProcessing).toBe(true);
expect(state.jobId).toBe('job-123');
```

### Testing API Retry
```typescript
mockInvoke
  .mockRejectedValueOnce(new Error('timeout'))
  .mockResolvedValueOnce({ id: 123 });

const result = await invokeCommand('test');
expect(mockInvoke).toHaveBeenCalledTimes(2); // 1 fail + 1 success
```

### Testing Events
```typescript
let callback;
mockListen.mockImplementation(async (name, cb) => {
  callback = cb;
  return () => {};
});

// Later
callback({ payload: { current: 50, total: 100 } });
const state = get(store);
expect(state.progress.current).toBe(50);
```

---

## File Locations (Absolute Paths)

### Configuration Files
- `/home/dojevou/projects/midi-software-center/pipeline/vitest.config.ts`
- `/home/dojevou/projects/midi-software-center/pipeline/vite.config.ts`
- `/home/dojevou/projects/midi-software-center/pipeline/package.json`

### Documentation (This Project)
- `/home/dojevou/projects/midi-software-center/FRONTEND-TESTING-QUICK-START.md`
- `/home/dojevou/projects/midi-software-center/FRONTEND-TESTING-ARCHITECTURE.md`
- `/home/dojevou/projects/midi-software-center/FRONTEND-TESTING-INDEX.md` (this file)

### Source Files (No Tests Yet)
```
/home/dojevou/projects/midi-software-center/pipeline/src/
├── lib/
│   ├── stores/
│   │   ├── processing.ts
│   │   ├── import.ts
│   │   ├── files.ts
│   │   ├── search.ts
│   │   ├── ui.ts
│   │   ├── tags.ts
│   │   └── stats.ts
│   ├── api/
│   │   ├── client.ts (KEY: retry + timeout logic)
│   │   ├── processing.ts
│   │   ├── files.ts
│   │   ├── search.ts
│   │   ├── events.ts
│   │   ├── system.ts
│   │   ├── connection.ts
│   │   └── stats.ts
│   ├── utils/
│   │   ├── validators.ts
│   │   └── formatters.ts
│   ├── components/
│   │   ├── Import/
│   │   ├── FileBrowser/
│   │   ├── Search/
│   │   └── ... (20+ Svelte files)
│   └── types/
└── routes/
```

### Where to Create Tests (Recommended)
```
/home/dojevou/projects/midi-software-center/pipeline/src/
├── __tests__/
│   ├── unit/
│   │   ├── utils/
│   │   │   ├── validators.test.ts
│   │   │   └── formatters.test.ts
│   │   ├── stores/
│   │   │   ├── processing.test.ts
│   │   │   ├── import.test.ts
│   │   │   ├── files.test.ts
│   │   │   ├── search.test.ts
│   │   │   ├── ui.test.ts
│   │   │   ├── tags.test.ts
│   │   │   └── stats.test.ts
│   │   ├── api/
│   │   │   ├── client.test.ts (KEY: 25+ tests)
│   │   │   ├── processing.test.ts
│   │   │   ├── files.test.ts
│   │   │   ├── search.test.ts
│   │   │   └── ...
│   │   └── components/
│   │       ├── ImportProgress.test.ts
│   │       ├── FileBrowser.test.ts
│   │       └── ...
│   ├── integration/
│   │   ├── import-workflow.test.ts
│   │   ├── search-workflow.test.ts
│   │   └── file-management.test.ts
│   └── fixtures/
│       └── mock-data.ts
```

---

## Running Tests

```bash
# Navigate to pipeline
cd /home/dojevou/projects/midi-software-center/pipeline

# Watch mode (development)
pnpm test

# UI dashboard
pnpm test:ui

# Coverage report (HTML output)
pnpm test:coverage

# Single file
pnpm test src/__tests__/unit/stores/processing.test.ts

# Matching pattern
pnpm test -t "should initialize"

# No watch
pnpm test --run
```

---

## Setup Checklist

- [ ] Read `FRONTEND-TESTING-QUICK-START.md` (5 min)
- [ ] Review store patterns in `pipeline/src/lib/stores/processing.ts`
- [ ] Review API client in `pipeline/src/lib/api/client.ts`
- [ ] Create `pipeline/src/__tests__/` directory structure
- [ ] Create `pipeline/src/__tests__/setup.ts` (optional, for component tests)
- [ ] Start with `pipeline/src/__tests__/unit/utils/validators.test.ts`
- [ ] Run `pnpm test` and watch first test pass
- [ ] Move to store tests
- [ ] Add API client tests (focus on retry logic)
- [ ] Add component tests (requires @testing-library/svelte)
- [ ] Run `pnpm test:coverage` and aim for 70%+

---

## Key Insights

### What's Easy to Test
1. **Utility functions** - Pure, no mocking needed
2. **Processing store** - Pure state manager
3. **Component logic** - Reactive declarations

### What Needs Mocking
1. **Tauri invoke()** - IPC commands
2. **Tauri listen()** - Event listeners
3. **File dialogs** - open() function
4. **API calls** - Imported functions

### What's Tricky
1. **Async operations** - Use fake timers wisely
2. **Event listeners** - Must simulate callbacks
3. **Store cleanup** - Always unsubscribe
4. **Memory leaks** - Clear mocks between tests

---

## Coverage Goals

**Current:** 0%
**Target:** 70%+ (lines, functions, branches, statements)

Based on ~1,050 testable lines across stores, API, utils, and components.

```
Target Breakdown:
- Utilities: 15 tests
- Pure Stores: 30+ tests
- API Client: 25+ tests
- Stores with I/O: 20+ tests
- Components: 20+ tests
- Integration: 10+ tests
= 120-150 total tests for 70%+ coverage
```

---

## Additional Resources

**Vitest Documentation:**
- https://vitest.dev/api/
- https://vitest.dev/guide/mocking.html

**Svelte Testing:**
- https://testing-library.com/docs/svelte-testing-library/intro/
- https://svelte.dev/docs#run-time-component-api

**Tauri Documentation:**
- https://tauri.app/v1/guides/features/command/
- https://tauri.app/v1/guides/features/event/

---

## Questions?

1. **"How do I test async store methods?"** — Mock the async dependencies (Tauri APIs), then call the store method and verify state changes
2. **"How do I test event listeners?"** — Mock `listen()` to capture the callback, then invoke it to simulate events
3. **"How do I test store subscriptions?"** — Use `subscribe()` to get state updates, or use `get()` to read current state
4. **"How do I test components?"** — Install @testing-library/svelte, render the component with props, update stores, verify DOM changes
5. **"How do I increase coverage?"** — Focus on error paths, edge cases, and integration scenarios

---

**Last Updated:** 2025-11-03  
**Framework:** Vitest 1.0+  
**Target Coverage:** 70%  
**Estimated Effort:** 8-10 hours (120-150 tests)
