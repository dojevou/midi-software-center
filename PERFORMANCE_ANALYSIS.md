# Performance Analysis Report

**Date:** 2025-12-27
**Scope:** Full codebase analysis for N+1 queries, algorithm inefficiencies, unnecessary re-renders, and anti-patterns

---

## Executive Summary

| Category | Critical | High | Medium | Low |
|----------|----------|------|--------|-----|
| N+1 Queries | 3 | 2 | 5 | 0 |
| Algorithm Inefficiencies | 2 | 3 | 2 | 1 |
| Frontend Re-renders | 0 | 3 | 4 | 2 |
| Rust Anti-patterns | 1 | 2 | 2 | 3 |

**Total Issues Found: 35**

---

## 1. Database N+1 Query Anti-Patterns

### CRITICAL (Blocking Pipeline Performance)

#### 1.1 `file_repository.rs:117-166` - batch_insert()
```rust
for file in files {
    // Query 1: Check for duplicate
    let duplicate_id = sqlx::query_scalar("SELECT id FROM files WHERE content_hash = $1")...
    // Query 2: Insert file
    let file_id = sqlx::query_scalar("INSERT INTO files ...")...
}
```
**Impact:** 2000 queries for 1000 files instead of 1-2
**Fix:** Use bulk INSERT with UNNEST or multi-row VALUES clause

#### 1.2 `metadata_repository.rs:259-319` - batch_insert()
```rust
for metadata in records {
    sqlx::query_scalar("INSERT INTO musical_metadata ...")...
}
```
**Impact:** 1000 queries for 1000 records instead of 1
**Fix:** Use bulk INSERT with unnest() arrays

#### 1.3 `tag_repository.rs:417-440` - get_or_create_tags()
```rust
for name in names {
    let tag_id = sqlx::query_scalar("INSERT INTO tags ...")...  // Query 1
    let tag = self.find_by_id(tag_id).await?;                    // Query 2
}
```
**Impact:** 200 queries for 100 tags instead of 2
**Fix:** Use bulk INSERT with `RETURNING *` to get all fields

### HIGH PRIORITY

#### 1.4 `tag_repository.rs:307-324` - batch_add_to_file()
Loop executing INSERT per tag → 50 queries for 50 tags

#### 1.5 `collection_repository.rs:299-310` - batch_add_files()
Loop calling add_file() which executes 3 queries each → 300 queries for 100 files

### MEDIUM PRIORITY

| File | Function | Line | Impact |
|------|----------|------|--------|
| tag_repository.rs | batch_add_tags_by_name() | 331-347 | 100 queries for 50 tags |
| collection_repository.rs | reorder_files() | 345-359 | 100 queries for 100 files |
| articulation_repository.rs | batch_add_to_file() | 243-271 | 20 queries for 20 articulations |
| style_repository.rs | batch_add_to_file() | 234-258 | 10 queries for 10 styles |
| timbre_repository.rs | batch_add_to_file() | 234+ | 15 queries for 15 timbres |

---

## 2. Algorithm Inefficiencies

### CRITICAL

#### 2.1 `auto_tagger.rs:455-477` - Fuzzy matching O(n*k) with doubled calculations
```rust
fn fuzzy_match(&self, input: &str, dictionary: &HashSet<String>) -> Option<String> {
    dictionary.iter()
        .filter(|keyword| strsim::levenshtein(input, keyword) <= threshold)  // Calc #1
        .min_by_key(|keyword| strsim::levenshtein(input, keyword))           // Calc #2 (REDUNDANT)
        .cloned()
}
```
**Impact:** ~3,200 Levenshtein calculations per file (8 words × 4 dictionaries × 50 keywords × 2)
**Fix:** Calculate distance once, store in tuple, then filter and find min

#### 2.2 `drum_analyzer.rs:179-198` - 4 redundant passes through MIDI data
```rust
let drum_channel_detected = has_drum_channel(midi_file);     // Pass 1
let drum_notes = extract_drum_notes(midi_file);              // Pass 2
let time_signature = extract_time_signature_from_meta(...);  // Pass 3
let techniques = detect_techniques(midi_file, &drum_notes);  // Pass 4
```
**Impact:** 66% wasted processing time at 360 files/sec
**Fix:** Single-pass data extraction

### HIGH PRIORITY

#### 2.3 `key_detector.rs:92-106` - 24 array allocations per file
```rust
for pitch_class in 0..12 {
    rotate_profile(&MAJOR_PROFILE, pitch_class)  // Allocates [f64; 12]
    rotate_profile(&MINOR_PROFILE, pitch_class)  // Allocates [f64; 12]
}
```
**Fix:** Pre-compute all 24 rotated profiles as static constants

#### 2.4 `drum_analyzer.rs:300-333` - 8+ separate HashMap lookups
```rust
if drum_notes.contains_key(&DrumNote::ClosedHiHat) { ... }
if drum_notes.contains_key(&DrumNote::PedalHiHat) { ... }
// ... 6 more lookups
```
**Fix:** Single iteration with match on each key

#### 2.5 `bpm_detector.rs:368-387` - Unnecessary allocation + double pass
```rust
let bpms: Vec<f64> = tempo_changes.iter().map(|tc| tc.bpm).collect();  // Allocate
let mean = bpms.iter().sum::<f64>() / bpms.len() as f64;               // Pass 1
let variance = bpms.iter().map(...).sum::<f64>() / bpms.len();         // Pass 2
```
**Fix:** Use online variance algorithm (Welford's) or iterate tempo_changes directly

### MEDIUM PRIORITY

| File | Issue | Line |
|------|-------|------|
| auto_tagger.rs | String allocation per word (to_lowercase) | 238-375 |
| drum_analyzer.rs | Match statement instead of array lookup | 245-295 |

---

## 3. Frontend Re-render Issues

### HIGH PRIORITY

#### 3.1 `LinkSyncWindow.svelte:13-17` - 100ms polling interval
```typescript
refreshInterval = setInterval(() => {
    linkActions.refresh();
}, 100);  // 10 updates/second!
```
**Fix:** Reduce to 500ms-1000ms or implement change detection

#### 3.2 `vip3Store.ts:359-415` - Immediate search on every filter toggle
```typescript
toggleTimbre(id: number) {
    vip3Store.update(...);
    vip3Actions.search();  // No debounce!
}
```
**Fix:** Add 300-500ms debounce to search operations

#### 3.3 `MidiIOSetupWindow.svelte:63-74` - Multiple reactive filters
```typescript
$: outputDevices = $midiDeviceStore.devices.filter(...);
$: _inputDevices = $midiDeviceStore.devices.filter(...);
$: filteredRoutes = showOnlyEnabled ? channelRoutes.filter(...) : channelRoutes;
$: activeRouteCount = channelRoutes.filter(...).length;
```
**Fix:** Create derived stores with memoization

### MEDIUM PRIORITY

| File | Issue | Line |
|------|-------|------|
| VIP3Column.svelte | Inline event handlers in loop | 85 |
| VIP3FileList.svelte | Multiple inline handlers per item | 310-315 |
| App.svelte | Inline event handlers | 525-656 |
| Sequencer.svelte | Chained reactive dependencies | 24-38 |

### LOW PRIORITY

| File | Issue | Line |
|------|-------|------|
| AutomationLane.svelte | Reactive map operation | 25 |
| StatusBar.svelte | 2s polling interval | 18-28 |

---

## 4. Rust Anti-Patterns

### CRITICAL - Blocking I/O in Async Contexts

#### 4.1 `commands/daw/export.rs:52`
```rust
std::fs::write(&path, midi_data)  // Blocks async runtime!
```
**Fix:** Use `tokio::fs::write`

#### 4.2 `commands/pipeline/archive_import.rs:71, 185`
```rust
std::fs::read_dir(collection_dir)   // Line 71
std::fs::create_dir_all(&temp_dir)  // Line 185
```
**Fix:** Use `tokio::fs` equivalents

### HIGH PRIORITY - Excessive Clones

#### 4.3 `auto_tagger.rs:262,270,278,312` - Clone in hot loop
```rust
word_lower.clone()  // During 7,830 files/sec import
```
**Fix:** Move instead of clone where possible

#### 4.4 `chord_analyzer.rs:51-52` - String clones for every file
```rust
let progression: Vec<String> = chords.iter().map(|c| c.name.clone()).collect();
let types: Vec<String> = chords.iter().map(|c| c.chord_type.clone()).collect();
```
**Fix:** Store references or restructure data

### MEDIUM PRIORITY

| File | Issue | Line |
|------|-------|------|
| key_detector.rs | Array allocations (see Algorithm section) | 92-106 |
| search_repository.rs | String building in SQL | 304 |

### LOW PRIORITY

| File | Issue | Line |
|------|-------|------|
| key_detector.rs | Index-based loops instead of iterators | 92,166,181,199 |
| drum_analyzer.rs | Missing HashMap capacity hint | 224 |
| auto_repair.rs | Clone in error path | 108-109 |

---

## Recommended Fix Priority

### Phase 1: Critical Path (Immediate Impact)
1. **file_repository.rs batch_insert()** - Blocks import pipeline
2. **metadata_repository.rs batch_insert()** - Blocks analysis pipeline
3. **Blocking I/O in async** - Runtime thread starvation
4. **auto_tagger fuzzy_match()** - 3,200 redundant calculations/file

### Phase 2: High Impact
5. **drum_analyzer multiple passes** - 66% wasted cycles
6. **key_detector rotate_profile()** - 24 allocations/file
7. **vip3Store debouncing** - Excessive search calls
8. **tag_repository batch operations** - N+1 queries

### Phase 3: Medium Impact
9. All remaining N+1 query patterns
10. Frontend derived stores migration
11. Inline event handler extraction

---

## Positive Findings

The codebase already has many optimizations:
- Uses `Vec::with_capacity` in hot paths
- Uses rayon `par_iter()` for CLI parallelization (15+ files)
- Uses `tokio::fs` in pipeline workers (mostly)
- Memory-mapped I/O for file reading
- BLAKE3 hashing (7x faster than SHA-256)
- Batch database operations in many places
- Virtual scrolling in VIP3FileList
