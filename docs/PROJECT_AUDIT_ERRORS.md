# Project Audit - Error Report

**Generated:** 2025-12-17
**Status:** Build passes with warnings; Frontend has 24 TypeScript errors

---

## Summary

| Category | Count | Severity |
|----------|-------|----------|
| Rust Compilation Errors | 0 | - |
| Rust Warnings | 9 | Low |
| TypeScript/Svelte Errors | 24 | High |
| Accessibility Warnings | 203 | Medium |
| Security Vulnerabilities | 0 | - |

---

## TypeScript/Svelte Errors (24)

### Type Mismatch Errors

| File | Line | Error | Fix |
|------|------|-------|-----|
| `src/lib/api/vip3BrowserApi.ts` | 100 | `'has_more'` does not exist on `Vip3SearchResponse` | Add `has_more` to type or calculate differently |
| `src/lib/api/vip3BrowserApi.ts` | 346 | `CreateSavedSearchRequest` not assignable to `InvokeArgs` | Add index signature to interface |
| `src/lib/api/vip3BrowserApi.ts` | 418 | `CreateCollectionRequest` not assignable to `InvokeArgs` | Add index signature to interface |
| `src/lib/windows/ExportWindow.svelte` | 193 | `'projectMidi'` does not exist, did you mean `'projectAsMidi'`? | Change `projectMidi` to `projectAsMidi` |
| `src/lib/components/VIP3/VIP3Browser.svelte` | 42 | `'setFilters'` does not exist, did you mean `'setFilter'`? | Change `setFilters` to `setFilter` |

### Missing Properties on Type (VIP3BpmColumn.svelte)

| File | Line | Error |
|------|------|-------|
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 13 | `'bpm_counts'` does not exist on type `FilterCounts` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 14 | `'bpm_min'` does not exist on type `Vip3Filters` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 14 | `'bpm_max'` does not exist on type `Vip3Filters` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 17 | Argument `"bpm_min"` not assignable to `keyof Vip3Filters` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 18 | Argument `"bpm_max"` not assignable to `keyof Vip3Filters` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 22 | Argument `"bpm_min"` not assignable to `keyof Vip3Filters` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 23 | Argument `"bpm_max"` not assignable to `keyof Vip3Filters` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 27 | `'bpm_min'` does not exist on type `Vip3Filters` |
| `src/lib/components/VIP3/VIP3BpmColumn.svelte` | 27 | `'bpm_max'` does not exist on type `Vip3Filters` |

**Fix Required:** Add `bpm_min`, `bpm_max` to `Vip3Filters` type and `bpm_counts` to `FilterCounts` type in `src/lib/types/vip3.ts`

### Module Resolution Errors (Test Files)

| File | Line | Error |
|------|------|-------|
| `src/lib/components/__tests__/templates/AutomationLane.test.ts` | 7 | Cannot find module `'../DAW/AutomationLane.svelte'` |
| `src/lib/components/__tests__/templates/MixerWindow.test.ts` | 7 | Cannot find module `'../DAW/MixerWindow.svelte'` |
| `src/lib/components/__tests__/templates/MixerWindow.test.ts` | 8 | Cannot find module `'$lib/stores/mixerStore'` |
| `src/lib/components/__tests__/templates/VIP3Browser.test.ts` | 7 | Cannot find module `'../VIP3/VIP3Browser.svelte'` |

**Fix Required:** Update test file imports to correct paths or move test files to proper location

### Missing Store Methods

| File | Line | Error |
|------|------|-------|
| `src/lib/components/__tests__/templates/AutomationLane.test.ts` | 17 | `'reset'` does not exist on store type |
| `src/lib/components/__tests__/templates/VIP3Browser.test.ts` | 18 | `'reset'` does not exist on `Writable<VIP3State>` |

### Syntax Errors

| File | Line | Error |
|------|------|-------|
| `src/lib/components/__tests__/templates/AutomationLane.test.ts` | 164 | `'=>'` expected |
| `src/lib/components/__tests__/templates/AutomationLane.test.ts` | 169 | `'=>'` expected |

### Null Safety Errors

| File | Line | Error |
|------|------|-------|
| `src/lib/components/VIP3/Collections/CollectionViewer.svelte` | 201 | `'files'` is possibly `null` |
| `src/lib/components/VIP3/Favorites/FavoritesList.svelte` | 115 | `'favorites'` is possibly `null` |

---

## Rust Warnings (9)

### Unused Imports

| File | Line | Import |
|------|------|--------|
| `src/commands/daw/project_v2.rs` | 9 | `CreateProjectParams` |
| `src/commands/daw/project_v2.rs` | 10 | `UpdateProjectParams` |
| `src/commands/daw/project_v2.rs` | 14 | `error` (from tracing) |
| `src/commands/daw/sequencer.rs` | 215 | `super::*` |
| `src/commands/pipeline/meilisearch.rs` | 7 | `MeilisearchClient` |
| `src/commands/pipeline/meilisearch.rs` | 10 | `Deserialize`, `Serialize` |
| `src/commands/pipeline/meilisearch.rs` | 12 | `error` (from tracing) |

### Dead Code

| File | Line | Field |
|------|------|-------|
| `src/daw/effects.rs` | 326 | `comb_filters: [f32; 8]` |
| `src/daw/effects.rs` | 327 | `allpass_filters: [f32; 4]` |

### Unused Variables

| File | Line | Variable |
|------|------|----------|
| `src/bin/populate_key_id.rs` | 101 | `current_batch` |

---

## Accessibility Warnings (203)

Most common issues:
- Non-interactive elements with click handlers (use `<button>` instead of `<div on:click>`)
- Missing keyboard event handlers for click events
- Elements needing ARIA labels

**Files with A11y issues:**
- `Toast.svelte`
- `KeyboardShortcuts.svelte`
- `WindowBase.svelte`
- `ErrorDialog.svelte`
- Various VIP3 components

---

## Recommended Fix Priority

### High Priority (Blocking)
1. Fix `Vip3Filters` type - add `bpm_min`, `bpm_max` properties
2. Fix `FilterCounts` type - add `bpm_counts` property
3. Fix `ExportWindow.svelte:193` - change `projectMidi` to `projectAsMidi`
4. Fix `VIP3Browser.svelte:42` - change `setFilters` to `setFilter`

### Medium Priority (Tests)
5. Fix test file module imports
6. Fix test file syntax errors
7. Add null checks in `CollectionViewer.svelte` and `FavoritesList.svelte`

### Low Priority (Warnings)
8. Remove unused Rust imports
9. Address dead code in `effects.rs`
10. Improve accessibility in Svelte components

---

## Commands to Fix

```bash
# Fix Rust warnings automatically
cargo fix --lib -p midi-software-center

# Run type check after fixes
cd app && npx svelte-check

# Run full test suite
cargo test --workspace --lib
```
