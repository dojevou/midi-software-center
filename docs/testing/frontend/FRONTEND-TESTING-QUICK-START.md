# Pipeline Frontend Testing - Quick Reference

## What Testing Framework is Used?

**Vitest 1.0+** (not Jest)
- Configuration: `/home/dojevou/projects/midi-software-center/pipeline/vitest.config.ts`
- Test Pattern: `src/**/*.{test,spec}.{js,ts}`
- Coverage Target: 70% (lines, functions, branches, statements)
- Environment: jsdom (browser-like DOM)

## Existing Tests Status

**NONE** - Test infrastructure is configured but no tests exist yet.

- ✗ No `.test.ts` files in `pipeline/src/`
- ✗ No `.spec.ts` files
- ✗ No `__tests__` directories
- ✓ Vitest infrastructure ready to use

## NPM Test Scripts

```bash
pnpm test              # Watch mode
pnpm test:ui           # Vitest UI dashboard
pnpm test:coverage     # Coverage report (HTML + JSON)
```

## Key Code Patterns for Testing

### 1. Svelte Stores (Pure State)
- **Location:** `pipeline/src/lib/stores/*.ts`
- **Example:** `processingStore` - pure state manager, no I/O
- **Test Pattern:** Use `get(store)` to read state, call store methods, verify changes
- **Difficulty:** Easy (no mocking required)

### 2. Svelte Stores (With I/O)
- **Location:** `pipeline/src/lib/stores/import.ts`
- **Does:** File dialogs, event listeners, API calls
- **Test Pattern:** Mock Tauri APIs + event listeners
- **Difficulty:** Medium (requires mocking)

### 3. API Client
- **Location:** `pipeline/src/lib/api/client.ts`
- **Does:** Wraps Tauri IPC with retry logic, timeouts, error handling
- **Test Pattern:** Mock `invoke()` from `@tauri-apps/api/core`
- **Key Tests:** Retry logic, timeouts (10s), exponential backoff, error classification
- **Difficulty:** Medium

### 4. Svelte Components
- **Location:** `pipeline/src/lib/components/`
- **Does:** Display logic, reactive state, callbacks
- **Test Pattern:** Render component, update store, verify output
- **Difficulty:** Easy-Medium (requires `@testing-library/svelte`)

### 5. Utility Functions
- **Location:** `pipeline/src/lib/utils/*.ts`
- **Does:** Formatters, validators (pure functions)
- **Test Pattern:** Call function, assert output
- **Difficulty:** Very Easy (no mocking)

## Mocking Strategy for Tauri IPC

### Mock `invoke()` - Tauri Commands

```typescript
import { vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
const mockInvoke = invoke as ReturnType<typeof vi.fn>;

// Success
mockInvoke.mockResolvedValueOnce({ id: 123 });

// Error
mockInvoke.mockRejectedValueOnce(new Error('timeout'));
```

### Mock `listen()` - Events

```typescript
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { listen } from '@tauri-apps/api/event';
const mockListen = listen as ReturnType<typeof vi.fn>;

// Register listener (returns unlisten function)
mockListen.mockResolvedValueOnce(() => {});

// Simulate event
mockListen.mockImplementation(async (eventName, callback) => {
  callback({ payload: { /* event data */ } });
  return () => {}; // unlisten
});
```

### Mock `open()` - File Dialogs

```typescript
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

import { open } from '@tauri-apps/plugin-dialog';
const mockOpen = open as ReturnType<typeof vi.fn>;

// File selected
mockOpen.mockResolvedValueOnce('/path/to/file.mid');

// Dialog cancelled
mockOpen.mockResolvedValueOnce(null);

// Error
mockOpen.mockRejectedValueOnce(new Error('Failed'));
```

## Test File Structure (Recommended)

```
pipeline/src/
├── __tests__/
│   ├── unit/
│   │   ├── stores/
│   │   │   ├── processing.test.ts
│   │   │   └── import.test.ts
│   │   ├── api/
│   │   │   └── client.test.ts
│   │   └── utils/
│   │       ├── validators.test.ts
│   │       └── formatters.test.ts
│   ├── integration/
│   │   └── import-workflow.test.ts
│   └── fixtures/
│       └── mock-data.ts
├── lib/
├── routes/
└── App.svelte
```

**Alternative:** Put `.test.ts` files next to source files

## Key Files to Know

**Configuration:**
- `pipeline/vitest.config.ts` - Test setup, coverage thresholds (70%)
- `pipeline/vite.config.ts` - Includes test config (lines 65-80)
- `pipeline/package.json` - Test scripts

**Source (No Tests Yet):**
- `pipeline/src/lib/stores/processing.ts` - Pure state manager (EASY to test)
- `pipeline/src/lib/stores/import.ts` - With I/O (MEDIUM, needs mocking)
- `pipeline/src/lib/api/client.ts` - Retry logic, timeouts (MEDIUM, needs mocking)
- `pipeline/src/lib/api/processing.ts` - Simple wrapper (EASY, mock invokeCommand)
- `pipeline/src/lib/utils/*.ts` - Pure functions (VERY EASY)
- `pipeline/src/lib/components/` - Svelte UI (EASY, needs @testing-library/svelte)

## What You Need to Add

### 1. Component Testing Dependency (Optional)

```bash
pnpm add -D @testing-library/svelte @testing-library/dom
```

### 2. Test Setup File

Create: `pipeline/src/__tests__/setup.ts`

### 3. Create Tests

Priority order:
1. Utility functions (validators, formatters) - EASIEST
2. Processing store (pure state) - EASY
3. API client (retry logic) - MEDIUM
4. Import store (with I/O) - MEDIUM
5. Components (with @testing-library) - EASY-MEDIUM

## Coverage Status

**Current:** 0% (no tests)
**Target:** 70%+ (lines, functions, branches, statements)
**Estimated Testable Code:** ~1,050 lines

## Test Count Estimate

- Utilities: 15 tests
- Stores: 50+ tests
- API Client: 25 tests
- Components: 20+ tests
- Integration: 10+ tests
- **Total:** 120-150 tests (~8-10 hours development)

## Key Testing Patterns

### Store Testing
```typescript
import { get } from 'svelte/store';
import { processingStore } from '$lib/stores/processing';

processingStore.start('job-123');
const state = get(processingStore);
expect(state.isProcessing).toBe(true);
```

### API Testing
```typescript
const mockInvoke = invoke as ReturnType<typeof vi.fn>;
mockInvoke.mockResolvedValueOnce({ id: 123 });

const result = await invokeCommand('test_command');
expect(result).toEqual({ id: 123 });
```

### Event Testing
```typescript
let callback;
mockListen.mockImplementation(async (name, cb) => {
  callback = cb;
  return () => {}; // unlisten
});

// Later, simulate event
callback({ payload: { current: 50, total: 100 } });
```

## Critical Details

1. **Test Timeout:** 10 seconds default (increase if needed with `testTimeout: 15000`)
2. **Globals Enabled:** `describe`, `it`, `expect` work without import
3. **Fake Timers:** Use `vi.useFakeTimers()` for testing delays/retries
4. **Store Cleanup:** Call `unsubscribe()` after tests to prevent memory leaks
5. **Mock Cleanup:** Call `mockFn.mockClear()` in `beforeEach()`

## Running Tests

```bash
# Watch mode (development)
pnpm test

# UI dashboard (visual test runner)
pnpm test:ui

# Coverage report
pnpm test:coverage

# Single file
pnpm test processing.test.ts

# Specific test
pnpm test -t "should initialize with idle state"

# No watch mode
pnpm test --run
```

## Next Steps

1. Review `FRONTEND-TESTING-ARCHITECTURE.md` for comprehensive guide
2. Install @testing-library/svelte (optional, for component tests)
3. Create `pipeline/src/__tests__/` directory
4. Start with utility function tests (easiest)
5. Move to store tests
6. Add integration tests
7. Aim for 70%+ coverage

---

**See:** `/home/dojevou/projects/midi-software-center/FRONTEND-TESTING-ARCHITECTURE.md` for detailed patterns and examples.
