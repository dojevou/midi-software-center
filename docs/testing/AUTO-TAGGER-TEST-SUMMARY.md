# Auto Tagger Test Analysis - Executive Summary

## Overview
Comprehensive analysis of `pipeline/src-tauri/src/core/analysis/auto_tagger.rs` (510 lines) for test coverage improvement from 20% → 85%+.

## Current State
- **Existing tests:** 5 (1 failing ❌)
- **Coverage:** ~20%
- **Issue:** Style keyword "deep" not extracted from filename

## Recommended Test Suite

### Total: 112 Tests Across 11 Categories

| # | Category | Tests | Priority | Coverage Impact |
|---|----------|-------|----------|-----------------|
| 1 | Tag Struct | 6 | P2 | 5% |
| 2 | Construction | 3 | P2 | 3% |
| 3 | Filename Extraction | 20 | P1 | 20% |
| 4 | Path Extraction | 12 | P2 | 12% |
| 5 | Instrument Extraction | 8 | P2 | 8% |
| 6 | Fuzzy Matching | 18 | P1 | 18% |
| 7 | BPM/Key Tags | 12 | P2 | 10% |
| 8 | Integration | 12 | P1 | 12% |
| 9 | Dictionary Tests | 10 | P3 | 5% |
| 10 | Edge Cases | 8 | P3 | 5% |
| 11 | Regression | 3 | P1 | 2% |

**Expected final coverage: 85-90%** (100% functional coverage)

## Critical Findings

### 1. Failing Test Root Cause ❌
**Test:** `test_extract_from_filename`
```rust
// Input: "VEC_Deep_House_Kick_128_C.mid"
// Expected: "deep" tag extracted
// Actual: "deep" NOT extracted
```

**Issue:** Style keyword "deep" needs exact match verification:
- "deep" is in `style_keywords` dictionary (line 391)
- Style tags have no category prefix (line 151: `Tag::new(matched_style, None::<String>)`)
- Length is 4 chars (meets fuzzy match minimum)
- Need to verify exact match logic at line 208

**Fix:** Check if "deep" is being filtered by common words or length threshold

### 2. Known Limitations (Document, Don't Fix)

#### CamelCase Splitting
```rust
// Input: "TechnoLeadSynth.mid"
// Expected by test: ["techno", "lead", "synth"]
// Actual: ["technoleadsynth"] (single word)
```
- **Reason:** Rust regex doesn't support lookahead/lookbehind (line 64 comment)
- **Status:** Documented limitation, not a bug
- **Test strategy:** Verify consistent behavior

#### Windows Path Handling
```rust
// Pattern: path.split('/') (line 166)
// Issue: Windows paths like "C:\Samples\" won't split
// Impact: Cross-platform compatibility
```
- **Test needed:** Edge case for Windows-style paths
- **Consider:** Add Windows path normalization

### 3. Performance Bottleneck

**Fuzzy Matching Complexity:**
- Algorithm: Levenshtein O(m×n) per comparison
- Dictionaries: 133 keywords (28 genres + 54 instruments + 24 brands + 27 styles)
- Per filename word: ~133 distance calculations
- Per 10-word filename: ~1,330 calculations

**Optimization opportunities:**
1. Memoization cache for fuzzy matches
2. Trie data structure for prefix matching
3. Early exit on exact match (already implemented ✅)
4. Parallelize dictionary matching (rayon)

**Tests needed:**
- Benchmark with realistic filenames (10-15 words)
- Stress test with 100+ word filename
- Performance regression test

## Function-by-Function Coverage Plan

### Public Functions (100% coverage required)
1. **`AutoTagger::new()`** - 3 tests
   - Success case ✅
   - Dictionary verification
   - Regex compilation

2. **`AutoTagger::extract_tags()`** - 12 integration tests
   - All sources combined
   - Deduplication across sources
   - Empty inputs
   - Real-world sample pack structures

### Private Functions (indirect coverage via integration tests)
3. **`extract_from_filename()`** - 20 tests
   - All separators: `_`, `-`, ` `, `.`
   - All extensions: .mid, .MID, .midi, .MIDI
   - Common word filtering (16 stopwords)
   - Generic tags (>3 chars, alphanumeric)
   - Edge cases: empty, unicode, special chars

4. **`extract_from_path()`** - 12 tests
   - Basic: `/Brand/Genre/Category/file.mid`
   - Relative: `Samples/Genre/file.mid`
   - Edge cases: trailing slash, empty, very deep

5. **`extract_from_instruments()`** - 8 tests
   - MIDI GM instrument names
   - Fuzzy matching for typos
   - Empty array, unrecognized instruments

6. **`fuzzy_match()`** - 18 tests ⭐ MOST COMPLEX
   - Distance boundaries: 0, 1, 2, 3
   - Length thresholds: 1, 2, 3, 4+ chars
   - Tied distances (multiple matches)
   - Empty input, very long input
   - All genres/instruments with typos

### Dictionary Functions (10 tests)
7-11. **`load_*_keywords()`** - Verify content and counts

## Top 10 Critical Test Scenarios

### Priority 1: Must Have
1. **Fix failing test** - "deep" word extraction
2. **Fuzzy match boundaries** - Distances 0, 1, 2, 3
3. **All filename separators** - `_`, `-`, ` `, `.`
4. **All file extensions** - .mid, .MID, .midi, .MIDI
5. **Common word filtering** - 16 stopwords verified
6. **Path parsing** - Brand/Genre/Category structure
7. **BPM rounding** - 128.7 → 129
8. **Key signature filtering** - "unknown" excluded
9. **Full integration** - All sources → deduplicated tags
10. **Performance benchmark** - Realistic filename processing time

### Priority 2: Should Have
11. Empty inputs (filename, path, instruments, BPM, key)
12. Unicode handling (`"Café_Böhm_Kick.mid"`)
13. Special characters (`"Kick@#$%_128.mid"`)
14. Very long strings (256+ chars)
15. MIDI instrument fuzzy matching
16. Dictionary content verification (133 keywords)
17. Tag struct operations (new, full_name, equality)
18. Category assignment logic (instrument vs category)

### Priority 3: Nice to Have
19. Stress test: 1000-char filename
20. Stress test: 50+ nested folders
21. Concurrent access (thread safety)
22. Windows path edge case

## Edge Cases Requiring Tests

### Input Boundaries
- **Empty:** `""`, `[]`, `None` - 6 tests
- **Single char:** `"a.mid"` - filtered
- **Two chars:** `"ab.mid"` - filtered
- **Three chars:** `"abc.mid"` - no fuzzy match
- **Four chars:** `"abcd.mid"` - fuzzy enabled

### Fuzzy Match Boundaries
```rust
// Distance 0 (exact)
"techno" → "techno" ✅

// Distance 1
"teckno" → "techno" ✅ (swap)
"techno" → "techno" ✅ (deletion)

// Distance 2 (threshold)
"tecnno" → "techno" ✅ (2 changes)
"tekhno" → "techno" ✅ (2 substitutions)

// Distance 3 (fails)
"tekkko" → None ❌ (exceeds threshold)
```

### Unicode & Special Characters
```rust
// Unicode: "Café_Böhm_Kick.mid"
// Expected: ["instrument:kick"] + handle unicode gracefully

// Special chars: "Kick@#$%_128.mid"
// Expected: [] (non-alphanumeric filtered from generic tags)

// Numbers only: "128.mid"
// Expected: [] (numbers alone not tagged)
```

## Dictionary Statistics

### Keyword Sets (133 total)
- **Genres (28):** house, techno, trance, dubstep, dnb, electro, progressive, minimal, acid, ambient, trap, hiphop, lofi, jungle, etc.
- **Instruments (54):** kick, snare, hihat, bass, lead, pad, pluck, chord, stab, arp, piano, strings, brass, vocal, fx, loop, etc.
- **Manufacturers (24):** vengeance, splice, loopmasters, samplemagic, blackoctopus, cymatics, roland, korg, moog, arturia, native, serum, massive, ableton, etc.
- **Styles (27):** dark, melodic, aggressive, soft, hard, heavy, groovy, punchy, warm, analog, vintage, modern, classic, dirty, clean, atmospheric, uplifting, deep, driving, energetic, chill, etc.
- **Common Words (16):** the, and, for, with, track, midi, file, new, ver, vol, v, pt, part, demo, edit, mix

### Category Assignment Rules
| Source | Dictionary | Category Prefix |
|--------|-----------|-----------------|
| Filename | Genre | `genre:` |
| Filename | Instrument | `instrument:` |
| Filename | Manufacturer | `brand:` |
| Filename | Style | (none) |
| Path | Genre | `genre:` |
| Path | Instrument | `category:` ⚠️ |
| Path | Manufacturer | `brand:` |
| MIDI | Instrument | `instrument:` |
| Metadata | BPM | `bpm:` |
| Metadata | Key | `key:` |

**Note:** Path instruments get `category:` prefix (line 177), not `instrument:`

## Test Data Fixtures Needed

### Sample Pack Structures
```rust
// Vengeance style
const VENGEANCE_SAMPLE: &str =
    "/Samples/Vengeance/Vol2/DeepHouse/Drums/Kicks/VEC_Deep_Kick_128_C.mid";

// Splice style
const SPLICE_SAMPLE: &str =
    "/Splice/Electronic/House/Drums/splice_house_kick_128bpm.mid";

// Loopmasters style
const LOOPMASTERS_SAMPLE: &str =
    "/Loopmasters/DeepHouse/Drums/LM_DH_Kick_01.mid";
```

### MIDI Instrument Samples
```rust
fn sample_gm_instruments() -> Vec<String> {
    vec![
        "Acoustic Bass Drum".to_string(),
        "Synth Lead".to_string(),
        "Electric Piano".to_string(),
    ]
}
```

### Edge Case Generators
```rust
fn generate_long_string(len: usize) -> String {
    "a".repeat(len)
}

fn generate_deep_path(depth: usize) -> String {
    "/folder".repeat(depth) + "/file.mid"
}

fn generate_unicode_filename() -> String {
    "Café_Böhm_Schön_Kick.mid".to_string()
}
```

## Implementation Roadmap

### Phase 1: Fix Failing Test (30 min)
- [ ] Debug "deep" word extraction issue
- [ ] Verify exact match in `style_keywords`
- [ ] Fix or update test expectations
- [ ] Commit: "test(auto-tagger): fix deep word extraction test"

### Phase 2: Core Functionality (90 min)
- [ ] Tag struct tests (6)
- [ ] AutoTagger construction (3)
- [ ] Fuzzy matching core (10)
- [ ] Filename extraction basics (11)
- [ ] **Coverage milestone: 40-50%**

### Phase 3: Path & Instruments (60 min)
- [ ] Path extraction tests (12)
- [ ] Instrument extraction tests (8)
- [ ] **Coverage milestone: 60-65%**

### Phase 4: Integration & Metadata (90 min)
- [ ] BPM/Key tagging (12)
- [ ] Full integration tests (12)
- [ ] **Coverage milestone: 75-80%**

### Phase 5: Dictionaries & Edge Cases (60 min)
- [ ] Dictionary verification (10)
- [ ] Advanced fuzzy matching (8)
- [ ] **Coverage milestone: 85%+** ✅

### Phase 6: Stress & Regression (30 min)
- [ ] Performance benchmarks (3)
- [ ] Edge case stress tests (5)
- [ ] Regression tests (3)
- [ ] **Final coverage: 85-90%** ✅

**Total estimated time: 6 hours** (or 3.5-4 hours with tool-enhanced workflow)

## Quality Assurance Checklist

### Before Starting
- [ ] Read existing code (510 lines)
- [ ] Run existing tests, note failures
- [ ] Review CLAUDE.md requirements (no unwrap/expect/panic, 80%+ coverage)
- [ ] Set up coverage tooling (tarpaulin)

### During Implementation
- [ ] Write tests in order (fix failing → core → advanced)
- [ ] Run coverage after each phase
- [ ] Document any new bugs found
- [ ] Use test helpers to reduce duplication

### After Implementation
- [ ] Verify 85%+ coverage achieved
- [ ] Run all tests: `cargo test auto_tagger`
- [ ] Run benchmarks (if created)
- [ ] Security review: security-sentinel agent
- [ ] Code review: rust-backend agent
- [ ] Update CLAUDE.md with completion

## Success Criteria

### Functional Coverage (Must Achieve)
- ✅ All 2 public functions tested
- ✅ All 11 private functions tested (indirectly)
- ✅ All 4 dictionaries verified
- ✅ All splitting patterns tested
- ✅ Fuzzy match: distances 0, 1, 2, 3
- ✅ All optional parameters (None/Some)
- ✅ Deduplication verified
- ✅ Edge cases handled (empty, unicode, long)

### Code Coverage (Quantitative)
- ✅ **Line coverage: 85-90%**
- ✅ **Function coverage: 100%**
- ✅ Acceptable gaps: regex internals, HashSet std lib

### Code Quality (No Violations)
- ✅ No `unwrap()` / `expect()` / `panic!()` in production code
- ✅ All functions documented
- ✅ Tests organized by category
- ✅ Test names descriptive
- ✅ No test duplication (use helpers)

### Performance (Benchmarks)
- ✅ 10-word filename: < 1ms
- ✅ 100-word filename: < 10ms
- ✅ 50-folder path: < 5ms
- ✅ No performance regressions

## Key Takeaways

1. **112 tests needed** for 85-90% coverage (from 5 current)
2. **1 failing test** must be fixed first ("deep" word bug)
3. **Fuzzy matching is most complex** - 18 tests, performance critical
4. **4 dictionaries with 133 keywords** - all must be verified
5. **Thread-safe design** - good for Tauri concurrent operations
6. **CamelCase limitation** - documented, not a bug
7. **Performance testing essential** - Levenshtein is O(m×n)
8. **3.5-4 hours** estimated with tool-enhanced workflow

## Next Steps

1. **Run failing test** with `RUST_BACKTRACE=1` to debug "deep" issue
2. **Create test fixtures** module with sample data
3. **Implement Phase 1** - Fix failing test (30 min)
4. **Implement Phase 2** - Core functionality (90 min)
5. **Run coverage report** - Verify 40-50% after Phase 2
6. **Continue through Phase 6** - Achieve 85%+ coverage
7. **Run agent reviews** - security-sentinel, rust-backend
8. **Document completion** - Update CLAUDE.md, commit

## Related Documents

- **Full Analysis:** `AUTO-TAGGER-TEST-ANALYSIS.md` (112 detailed test scenarios)
- **Quick Reference:** `AUTO-TAGGER-TEST-QUICK-REFERENCE.md` (implementation guide)
- **Source Code:** `pipeline/src-tauri/src/core/analysis/auto_tagger.rs` (510 lines)
- **Coverage Plan:** `TEST-COVERAGE-PLAN.md` (8-phase roadmap)
