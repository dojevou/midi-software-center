# Frontend Testing Documentation Guide

This document helps you navigate the frontend testing documentation created for the MIDI Software Center project.

## Available Documentation

### 1. Start Here: FRONTEND-TESTING-QUICK-START.md
**Time:** 5-10 minutes
**Purpose:** Quick reference and mocking cheat sheet
**Contains:**
- Testing framework overview (Vitest)
- Mocking strategies for all Tauri APIs
- NPM test commands
- Code patterns at a glance
- Best for: Getting started quickly

**When to use:** You need a quick answer or reminder

---

### 2. Deep Dive: FRONTEND-TESTING-ARCHITECTURE.md
**Time:** 30-45 minutes
**Purpose:** Comprehensive guide with copy-paste ready examples
**Contains:**
- Vitest configuration (59 lines explained)
- 7 Svelte stores analysis (7 files, 200 lines)
- API client layer analysis (8 files, 300 lines)
- Utility functions overview (2 files)
- Component patterns (20+ files)
- Complete mocking strategies with code examples
- Full test templates for:
  - Store testing (15+ examples)
  - API client testing (20+ examples)
  - Component testing (10+ examples)
  - Integration testing (5+ examples)
- Setup instructions with code
- Mock data fixtures template
- Integration test patterns
- Coverage goals and breakdown

**When to use:** Planning test implementation, learning patterns, need complete examples

---

### 3. Overview: FRONTEND-TESTING-INDEX.md
**Time:** 10-15 minutes
**Purpose:** Navigation and planning guide
**Contains:**
- Overview of testing status
- Quick reference tables
- File structure recommendations
- Test implementation path (6 phases)
- Mocking cheat sheet
- Setup checklist
- Coverage breakdown
- Key insights

**When to use:** Planning which tests to write, understanding the big picture

---

### 4. File Reference: FRONTEND-TESTING-FILE-REFERENCE.md
**Time:** Lookup reference
**Purpose:** Absolute paths and file locations
**Contains:**
- All documentation file locations
- Configuration file locations
- Source file locations (stores, API, utils, components)
- Where to create test files
- Tauri IPC modules to mock
- Running test commands
- Implementation checklist
- Total effort estimate

**When to use:** Need to find a specific file, absolute paths

---

## Quick Navigation by Task

### "I want to get started quickly"
1. Read: `FRONTEND-TESTING-QUICK-START.md` (5 min)
2. Scan: `FRONTEND-TESTING-INDEX.md` implementation path (5 min)
3. Start: Create `pipeline/src/__tests__/unit/utils/validators.test.ts`

### "I'm planning the test implementation"
1. Read: `FRONTEND-TESTING-QUICK-START.md` (5 min)
2. Review: `FRONTEND-TESTING-INDEX.md` (10 min)
   - Test implementation path (6 phases)
   - Coverage breakdown
   - Setup checklist
3. Estimate: 8-10 hours for 120-150 tests

### "I need to understand store testing"
1. Read: `FRONTEND-TESTING-QUICK-START.md` → Store Testing section
2. Jump to: `FRONTEND-TESTING-ARCHITECTURE.md` → Section 3.1 (Svelte Stores)
3. Review: Example test templates in Section 5.2 (Store Test Pattern)

### "I need to understand API client testing"
1. Read: `FRONTEND-TESTING-QUICK-START.md` → API Testing section
2. Jump to: `FRONTEND-TESTING-ARCHITECTURE.md` → Section 3.2 (API Layer)
3. Review: Example test template in Section 5.3 (API Client Test Pattern)

### "I need to understand Tauri IPC mocking"
1. Read: `FRONTEND-TESTING-QUICK-START.md` → Mocking Strategy section
2. Jump to: `FRONTEND-TESTING-ARCHITECTURE.md` → Section 4 (Mocking Strategy)
3. Reference: All three mocking patterns with examples

### "I need absolute file paths"
1. Go to: `FRONTEND-TESTING-FILE-REFERENCE.md`
2. Jump to relevant section:
   - Configuration Files
   - Source Files
   - Test File Locations
   - Tauri IPC Modules

### "I need test templates I can copy-paste"
1. Jump to: `FRONTEND-TESTING-ARCHITECTURE.md`
2. Go to: Section 5 (Test File Patterns)
   - Section 5.2: Store test pattern (copy-paste ready)
   - Section 5.3: API client test pattern
   - Section 5.4: Store with I/O test pattern
   - Section 5.5: Component test pattern

---

## Key Information Summary

### Testing Framework
- **Framework:** Vitest 1.0+
- **Config:** `/pipeline/vitest.config.ts` (59 lines, ready to use)
- **Pattern:** `src/**/*.{test,spec}.{js,ts}`
- **Coverage Target:** 70%

### Current Status
- **Tests:** 0 (green field)
- **Testable Code:** ~1,050 lines
- **Estimated Tests Needed:** 120-150
- **Estimated Time:** 8-10 hours

### Code to Test
| Category | Files | Type | Tests | Difficulty |
|----------|-------|------|-------|------------|
| Utilities | 2 | Pure functions | 15 | Very Easy |
| Pure Stores | 7 | State managers | 30 | Easy |
| API Client | 8 | With retry logic | 25 | Medium |
| Stores with I/O | 7 | Tauri integration | 20 | Medium |
| Components | 20+ | Svelte UI | 20 | Easy |
| Integration | - | Workflows | 10 | Medium |

### Mocking Needed
| API | Module | Function | Tests |
|-----|--------|----------|-------|
| Tauri Core | @tauri-apps/api/core | invoke() | 25+ |
| Tauri Events | @tauri-apps/api/event | listen() | 10+ |
| Tauri Dialog | @tauri-apps/plugin-dialog | open() | 10+ |

---

## Implementation Phases

### Phase 1: Utilities (1-2 hours)
- Create `validators.test.ts` (5 tests)
- Create `formatters.test.ts` (5 tests)
- First tests to write (EASIEST)

### Phase 2: Pure Stores (2-3 hours)
- Create `processing.test.ts` (15 tests)
- Create `ui.test.ts` (10 tests)
- No mocking required

### Phase 3: API Client (2-3 hours)
- Create `client.test.ts` (25 tests)
- Test retry logic, timeouts, error handling
- Most complex phase

### Phase 4: Stores with I/O (2-3 hours)
- Create `import.test.ts` (20 tests)
- Mock dialogs, events, API functions

### Phase 5: Components (2-3 hours)
- Create component tests (20 tests)
- Requires @testing-library/svelte

### Phase 6: Integration (2-3 hours)
- Create workflow tests (10 tests)
- End-to-end scenarios

---

## Mocking Quick Reference

### Mock Tauri invoke()
```typescript
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
const mockInvoke = invoke as ReturnType<typeof vi.fn>;

mockInvoke.mockResolvedValueOnce({ id: 123 });
```

### Mock Tauri listen()
```typescript
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { listen } from '@tauri-apps/api/event';
const mockListen = listen as ReturnType<typeof vi.fn>;

mockListen.mockResolvedValueOnce(() => {});
```

### Mock Tauri open()
```typescript
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

import { open } from '@tauri-apps/plugin-dialog';
const mockOpen = open as ReturnType<typeof vi.fn>;

mockOpen.mockResolvedValueOnce('/path/to/file.mid');
```

---

## Common Questions & Answers

**Q: What's the difference between the documents?**
A: Quick-Start (fast reference), Architecture (comprehensive guide with examples), Index (planning overview), File-Reference (path lookups)

**Q: Should I read all documents?**
A: Start with Quick-Start (5 min), then Architecture (30 min) if implementing tests

**Q: Which file should I read first?**
A: `FRONTEND-TESTING-QUICK-START.md` - it's only 5-10 minutes

**Q: Where are the test templates I can copy-paste?**
A: Section 5 of `FRONTEND-TESTING-ARCHITECTURE.md` has complete, ready-to-use examples

**Q: How many tests will I need to write?**
A: 120-150 tests to reach 70% coverage, estimated 8-10 hours

**Q: Which tests should I write first?**
A: Utilities and pure stores (easiest), then API client (most complex)

**Q: What do I need to mock?**
A: Three Tauri APIs: invoke(), listen(), open() - all mocking patterns provided

**Q: Do I need @testing-library/svelte?**
A: Only if you want to test components (it's optional but recommended)

---

## Running Tests

```bash
cd /home/dojevou/projects/midi-software-center/pipeline

# Watch mode
pnpm test

# UI dashboard
pnpm test:ui

# Coverage report
pnpm test:coverage

# Single file
pnpm test src/__tests__/unit/utils/validators.test.ts
```

---

## File Locations

**Documentation (Project Root):**
- `FRONTEND-TESTING-QUICK-START.md`
- `FRONTEND-TESTING-ARCHITECTURE.md`
- `FRONTEND-TESTING-INDEX.md`
- `FRONTEND-TESTING-FILE-REFERENCE.md`
- `TESTING-DOCUMENTATION-GUIDE.md` (this file)

**Configuration:**
- `/pipeline/vitest.config.ts`
- `/pipeline/vite.config.ts`
- `/pipeline/package.json`

**Source to Test:**
- `/pipeline/src/lib/stores/`
- `/pipeline/src/lib/api/`
- `/pipeline/src/lib/utils/`
- `/pipeline/src/lib/components/`

**Where to Create Tests:**
- `/pipeline/src/__tests__/unit/`
- `/pipeline/src/__tests__/integration/`

---

## Effort Estimate

**Total Time to 70% Coverage: 10-14 hours**

- Reading documentation: 1 hour
- Setup: 30 minutes
- Test implementation: 8-10 hours
- Coverage optimization: 1-2 hours

---

## Next Steps

1. Open: `FRONTEND-TESTING-QUICK-START.md`
2. Read it (5 minutes)
3. Create `pipeline/src/__tests__/` directory
4. Write your first test (validators)
5. Run `pnpm test`
6. Feel the joy of watching tests pass!

---

**Last Updated:** 2025-11-03
**Framework:** Vitest 1.0+
**Coverage Target:** 70%
**Estimated Effort:** 8-10 hours

Start here: `FRONTEND-TESTING-QUICK-START.md`
