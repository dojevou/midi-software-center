# Frontend Testing - Complete File Reference

## Documentation Files (Project Root)

**Read These First:**

1. **FRONTEND-TESTING-QUICK-START.md** (7.3 KB)
   - Fast reference guide (5-10 min read)
   - Mocking cheat sheet
   - NPM commands
   - Best for: Quick answers

2. **FRONTEND-TESTING-ARCHITECTURE.md** (35 KB)
   - Comprehensive guide (30-45 min read)
   - All code patterns with full examples
   - Copy-paste ready test templates
   - Best for: Deep understanding and implementation

3. **FRONTEND-TESTING-INDEX.md** (13 KB)
   - Overview and navigation
   - File structure recommendations
   - Setup checklist
   - Best for: Planning and organization

4. **FRONTEND-TESTING-FILE-REFERENCE.md** (This file)
   - Absolute paths for all files
   - Quick lookup

---

## Configuration Files (Ready to Use)

**Location:** `/home/dojevou/projects/midi-software-center/pipeline/`

### vitest.config.ts (59 lines)
Path: `/home/dojevou/projects/midi-software-center/pipeline/vitest.config.ts`

Key settings:
- Provider: v8
- Environment: jsdom
- Coverage thresholds: 70% (lines, functions, branches, statements)
- Test timeout: 10 seconds
- Include pattern: `src/**/*.{test,spec}.{js,ts}`

### vite.config.ts (82 lines)
Path: `/home/dojevou/projects/midi-software-center/pipeline/vite.config.ts`

Test configuration (lines 65-80):
- Globals enabled
- Environment: jsdom
- Test pattern: `src/**/*.{test,spec}.{js,ts}`

### package.json (61 lines)
Path: `/home/dojevou/projects/midi-software-center/pipeline/package.json`

Test scripts:
- `"test": "vitest"` — Watch mode
- `"test:ui": "vitest --ui"` — UI dashboard
- `"test:coverage": "vitest run --coverage"` — Coverage report

---

## Source Files Needing Tests

### STORES (7 files)

**Path:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/stores/`

| File | Lines | Type | Difficulty | Notes |
|------|-------|------|------------|-------|
| `processing.ts` | 297 | Pure State | Easy | No mocking needed |
| `import.ts` | 230 | With I/O | Medium | Needs Tauri mocks |
| `files.ts` | ? | Unknown | ? | |
| `search.ts` | ? | Unknown | ? | |
| `ui.ts` | ? | Unknown | ? | |
| `tags.ts` | ? | Unknown | ? | |
| `stats.ts` | ? | Unknown | ? | |

### API CLIENT (8 files)

**Path:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/api/`

| File | Lines | Type | Difficulty | Notes |
|------|-------|------|------------|-------|
| `client.ts` | 311 | Core IPC | Medium | Retry logic, timeouts |
| `processing.ts` | 86 | Wrapper | Easy | Mock invokeCommand |
| `files.ts` | ? | Wrapper | Easy | Mock invokeCommand |
| `search.ts` | ? | Wrapper | Easy | Mock invokeCommand |
| `events.ts` | ? | Wrapper | Easy | Mock invokeCommand |
| `system.ts` | ? | Wrapper | Easy | Mock invokeCommand |
| `connection.ts` | ? | Wrapper | Easy | Mock invokeCommand |
| `stats.ts` | ? | Wrapper | Easy | Mock invokeCommand |

### UTILITIES (2 files)

**Path:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/utils/`

| File | Type | Difficulty |
|------|------|------------|
| `validators.ts` | Pure functions | Very Easy |
| `formatters.ts` | Pure functions | Very Easy |

### COMPONENTS (20+ files)

**Path:** `/home/dojevou/projects/midi-software-center/pipeline/src/lib/components/`

Key files:
- `Import/ImportProgress.svelte`
- `FileBrowser/FileBrowser.svelte`
- `FileBrowser/FileCard.svelte`
- `Search/SearchBar.svelte`
- `Search/FilterPanel.svelte`
- `Tags/TagCloud.svelte`
- `Tags/TagEditor.svelte`
- `FileDetails/DetailsPanel.svelte`
- `FileDetails/MetadataView.svelte`
- ... and more

---

## Test File Locations (Create These)

**Recommended Structure:**

```
/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/
├── unit/
│   ├── utils/
│   │   ├── validators.test.ts
│   │   └── formatters.test.ts
│   ├── stores/
│   │   ├── processing.test.ts
│   │   ├── import.test.ts
│   │   ├── files.test.ts
│   │   ├── search.test.ts
│   │   ├── ui.test.ts
│   │   ├── tags.test.ts
│   │   └── stats.test.ts
│   ├── api/
│   │   ├── client.test.ts
│   │   ├── processing.test.ts
│   │   ├── files.test.ts
│   │   ├── search.test.ts
│   │   └── ...
│   └── components/
│       ├── ImportProgress.test.ts
│       ├── FileBrowser.test.ts
│       └── ...
├── integration/
│   ├── import-workflow.test.ts
│   ├── search-workflow.test.ts
│   └── file-management.test.ts
├── fixtures/
│   └── mock-data.ts
└── setup.ts (optional)
```

### Unit Tests

**Utils:**
- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/unit/utils/validators.test.ts`
- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/unit/utils/formatters.test.ts`

**Stores:**
- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/unit/stores/processing.test.ts`
- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/unit/stores/import.test.ts`

**API:**
- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/unit/api/client.test.ts`

**Components:**
- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/unit/components/ImportProgress.test.ts`

### Integration Tests

- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/integration/import-workflow.test.ts`

### Fixtures & Setup

- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/fixtures/mock-data.ts`
- `/home/dojevou/projects/midi-software-center/pipeline/src/__tests__/setup.ts` (optional)

---

## Tauri IPC Modules (To Mock)

All paths are npm package imports:

### Tauri Core (`invoke`)
```typescript
// Mock at this path:
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Import in tests:
import { invoke } from '@tauri-apps/api/core';
```

**Used by:** `pipeline/src/lib/api/client.ts` → `invokeCommand<T>()`

**Tests needed:** 25+ tests for retry logic, timeouts, error handling

### Tauri Events (`listen`)
```typescript
// Mock at this path:
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Import in tests:
import { listen } from '@tauri-apps/api/event';
```

**Used by:** `pipeline/src/lib/stores/import.ts` → progress events

**Tests needed:** 10+ tests for event listener registration and callbacks

### Tauri Dialog (`open`)
```typescript
// Mock at this path:
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

// Import in tests:
import { open } from '@tauri-apps/plugin-dialog';
```

**Used by:** `pipeline/src/lib/stores/import.ts` → file/directory dialogs

**Tests needed:** 10+ tests for file selection, cancellation, errors

---

## Running Tests (Commands)

From `/home/dojevou/projects/midi-software-center/pipeline/`:

```bash
# Watch mode (development)
pnpm test

# UI dashboard (visual runner)
pnpm test:ui

# Coverage report (HTML)
pnpm test:coverage

# Single file
pnpm test src/__tests__/unit/utils/validators.test.ts

# Pattern matching
pnpm test -t "should initialize"

# No watch (CI mode)
pnpm test --run
```

---

## Dependencies to Install (Optional)

For component testing with @testing-library/svelte:

```bash
cd /home/dojevou/projects/midi-software-center/pipeline
pnpm add -D @testing-library/svelte @testing-library/dom
```

---

## Test Coverage Target

**Current:** 0% (no tests)
**Goal:** 70%+ (lines, functions, branches, statements)

**Estimated Testable Code:** ~1,050 lines
**Estimated Tests Needed:** 120-150
**Estimated Time:** 8-10 hours

### Coverage Breakdown

```
Utilities:          15 tests  (100% coverage possible)
Pure Stores:        30 tests  (95%+ coverage)
API Client:         25 tests  (90%+ coverage)  ★ MOST CRITICAL
Stores with I/O:    20 tests  (85%+ coverage)
Components:         20 tests  (80%+ coverage)
Integration:        10 tests  (90%+ coverage)
─────────────────────────────
Total:             ~130 tests → 70%+ overall
```

---

## Key Source Code Locations

### Processing Store (Pure State - EASIEST)
Path: `/home/dojevou/projects/midi-software-center/pipeline/src/lib/stores/processing.ts`

Key methods to test:
- `start(jobId: string)`
- `complete()`
- `fail(error: string)`
- `getDuration(): number`
- `subscribe` (subscriptions)

Test count: 15+ tests

### Import Store (With I/O - MEDIUM)
Path: `/home/dojevou/projects/midi-software-center/pipeline/src/lib/stores/import.ts`

Key methods to test (require mocking):
- `pickFile()` → mocks `open()` dialog
- `pickDirectory()` → mocks `open()` dialog
- `importFile()` → mocks `listen()` events + API
- `importDirectory()` → mocks `listen()` events + API
- `pickAndImport()` → integration

Mocks needed:
- `@tauri-apps/plugin-dialog` → `open()`
- `@tauri-apps/api/event` → `listen()`
- `$lib/api/files` → `importSingleFile()`, `importDirectory()`

Test count: 20+ tests

### API Client (Retry Logic - MOST COMPLEX)
Path: `/home/dojevou/projects/midi-software-center/pipeline/src/lib/api/client.ts`

Key functions to test:
- `invokeCommand<T>()` → Tauri IPC with retry
- `testConnection()` → connection monitoring
- `startConnectionMonitoring()` → periodic checks
- Error classification helpers
- Connection status store

Key test scenarios:
- Successful call on first attempt
- Transient error → retry → success
- Non-retryable error → fail immediately
- Timeout handling (10s)
- Exponential backoff (1s, 2s, 4s)
- Connection status updates
- Error message transformation

Mocks needed:
- `@tauri-apps/api/core` → `invoke()`
- Fake timers for delays/retries

Test count: 25+ tests

---

## TypeScript Configuration

Path: `/home/dojevou/projects/midi-software-center/pipeline/tsconfig.json`

Important settings for testing:
- `"strict": true` — Strict mode enabled
- `"noUnusedLocals": true` — Catch unused variables
- `"noUnusedParameters": true` — Catch unused params
- `"noImplicitReturns": true` — Require explicit returns

---

## Project Structure Summary

```
/home/dojevou/projects/midi-software-center/
├── pipeline/                          # Pipeline frontend (Svelte/TypeScript)
│   ├── src/
│   │   ├── lib/
│   │   │   ├── stores/               # Svelte stores (7 files) → NEEDS TESTS
│   │   │   ├── api/                  # API client (8 files) → NEEDS TESTS
│   │   │   ├── utils/                # Utilities (2 files) → NEEDS TESTS
│   │   │   ├── components/           # Svelte components (20+ files) → NEEDS TESTS
│   │   │   └── types/                # TypeScript types
│   │   ├── routes/
│   │   └── App.svelte
│   ├── src-tauri/                    # Tauri backend (Rust)
│   ├── vitest.config.ts              # ✓ Already configured
│   ├── vite.config.ts                # ✓ Already configured
│   ├── package.json                  # ✓ Test scripts ready
│   └── tsconfig.json
│
├── daw/                               # DAW frontend (also needs tests)
├── database/                          # PostgreSQL + migrations
├── shared/                            # Shared Rust library
├── scripts/                           # CLI tools
│
├── FRONTEND-TESTING-QUICK-START.md    # ← Read first (5 min)
├── FRONTEND-TESTING-ARCHITECTURE.md   # ← Detailed guide (45 min)
├── FRONTEND-TESTING-INDEX.md          # ← Overview
└── FRONTEND-TESTING-FILE-REFERENCE.md # ← This file
```

---

## Quick Implementation Checklist

**Preparation (30 min):**
- [ ] Read `FRONTEND-TESTING-QUICK-START.md`
- [ ] Review `pipeline/src/lib/stores/processing.ts` (simple store)
- [ ] Review `pipeline/src/lib/api/client.ts` (retry logic)
- [ ] Create `pipeline/src/__tests__/` directory structure

**Phase 1: Utils (1-2 hours):**
- [ ] Create `unit/utils/validators.test.ts` (5 tests)
- [ ] Create `unit/utils/formatters.test.ts` (5 tests)
- [ ] Run `pnpm test` and verify passing

**Phase 2: Pure Stores (2-3 hours):**
- [ ] Create `unit/stores/processing.test.ts` (15 tests)
- [ ] Create `unit/stores/ui.test.ts` (10 tests)
- [ ] Add more store tests as needed

**Phase 3: API Client (2-3 hours):**
- [ ] Create `unit/api/client.test.ts` (25 tests)
  - Retry logic (3 attempts)
  - Timeout handling (10s)
  - Exponential backoff (1s, 2s, 4s)
  - Error classification
- [ ] Create other API wrapper tests

**Phase 4: Stores with I/O (2-3 hours):**
- [ ] Create `unit/stores/import.test.ts` (20 tests)
  - Mock `open()` dialog
  - Mock `listen()` events
  - Mock API functions
  - Test state updates

**Phase 5: Components (2-3 hours):**
- [ ] Install `@testing-library/svelte` (if needed)
- [ ] Create `unit/components/*.test.ts` (20 tests)
- [ ] Test reactive state and callbacks

**Phase 6: Integration (2-3 hours):**
- [ ] Create `integration/import-workflow.test.ts`
- [ ] Create other workflow tests

**Final (30 min):**
- [ ] Run `pnpm test:coverage`
- [ ] Verify 70%+ coverage
- [ ] Address any coverage gaps

---

## Total Effort Estimate

**Reading documentation:** 1 hour (quick start + architecture)
**Setup:** 30 minutes
**Test implementation:** 8-10 hours (120-150 tests)
**Coverage optimization:** 1-2 hours
**Total:** 10-14 hours to reach 70%+ coverage

---

**Start with:** `/home/dojevou/projects/midi-software-center/FRONTEND-TESTING-QUICK-START.md`
