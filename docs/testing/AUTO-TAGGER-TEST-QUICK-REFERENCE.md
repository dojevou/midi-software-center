# Auto Tagger Test Quick Reference

## Module Stats
- **File:** `pipeline/src-tauri/src/core/analysis/auto_tagger.rs`
- **Size:** 510 lines
- **Functions:** 13 total (2 public, 11 private/helpers)
- **Current Tests:** 5 (1 failing)
- **Target Tests:** 112
- **Current Coverage:** ~20%
- **Target Coverage:** 85-90%

## Quick Test Breakdown

### 11 Categories | 112 Tests Total

| Category | Tests | Focus |
|----------|-------|-------|
| 1. Tag Struct | 6 | Construction, formatting, equality, hashing |
| 2. Construction | 3 | AutoTagger initialization, dictionaries |
| 3. Filename | 20 | Splitting patterns, extensions, filtering |
| 4. Path | 12 | Path parsing, folder matching, categories |
| 5. Instruments | 8 | MIDI GM instrument name matching |
| 6. Fuzzy Match | 18 | Levenshtein distance, thresholds, boundaries |
| 7. BPM/Key | 12 | Optional metadata tagging |
| 8. Integration | 12 | End-to-end tag extraction |
| 9. Dictionaries | 10 | Keyword sets (4 dicts + common words) |
| 10. Edge Cases | 8 | Performance, special chars, stress |
| 11. Regression | 3 | Known bugs, limitations |

## Critical Test Scenarios

### Priority 1: Fix Failing Test
- **Test:** `test_extract_from_filename`
- **Issue:** "deep" word not extracted from `"Deep_House_Kick.mid"`
- **Root Cause:** Style keyword "deep" needs exact match logic verification

### Priority 2: Core Functionality (30 tests)
1. Tag struct operations (6)
2. Fuzzy matching boundaries (10)
3. Filename splitting patterns (14)

### Priority 3: Path & Instruments (20 tests)
4. Path parsing (12)
5. MIDI instrument mapping (8)

## Key Edge Cases

### Input Boundaries
- Empty: `""`, `[]`, `None`
- Short: Single char, 2 chars, 3 chars (threshold)
- Long: 256+ chars filename, 50+ folder depth
- Unicode: `"Café_Böhm_Kick.mid"`
- Special: `"Kick@#$%_128.mid"`

### Fuzzy Match Thresholds
- Distance 0: Exact match
- Distance 1: `"teckno"` → `"techno"`
- Distance 2: `"tecnno"` → `"techno"` (max threshold)
- Distance 3: No match
- Min length: 4 chars (no fuzzy for shorter)

### Dictionary Stats
- **Genres:** 28 keywords
- **Instruments:** 54 keywords
- **Manufacturers:** 24 keywords
- **Styles:** 27 keywords
- **Common Words:** 16 stopwords

## Pattern Cheat Sheet

### Splitting Pattern
```regex
[_\-\s.]+  # Underscores, hyphens, spaces, dots
```

### Extension Removal
- `.mid`, `.MID`, `.midi`, `.MIDI`

### Filtering Rules
- Min length: 2 chars
- Generic tag: >3 chars + alphanumeric only
- Common words: 16 stopwords filtered

### Category Assignment
- Filename instruments: `"instrument:X"`
- Path instruments: `"category:X"`
- Genres: `"genre:X"`
- Manufacturers: `"brand:X"`
- Styles: No prefix (just `"deep"`)
- BPM: `"bpm:128"`
- Key: `"key:c"`

## Test Data Fixtures Needed

### Sample Paths
```rust
// Vengeance style
"/Samples/Vengeance/Vol2/DeepHouse/Drums/Kicks/VEC_Deep_Kick_128_C.mid"

// Splice style
"/Splice/Electronic/House/Drums/splice_house_kick_128bpm.mid"

// Loopmasters style
"/Loopmasters/DeepHouse/Drums/LM_DH_Kick_01.mid"
```

### Sample MIDI Instruments
```rust
["Acoustic Bass Drum", "Synth Lead", "Electric Piano"]
```

### Edge Case Generators
```rust
fn generate_long_string(len: usize) -> String
fn generate_deep_path(depth: usize) -> String
fn generate_unicode_samples() -> Vec<String>
```

## Known Issues & Limitations

### 1. CamelCase Splitting
- **Issue:** Regex cannot split `"TechnoLeadSynth"`
- **Reason:** Rust regex doesn't support lookahead/lookbehind
- **Status:** Documented limitation (line 64)
- **Test:** Verify this behavior is consistent

### 2. "deep" Word Bug
- **Issue:** Style keyword "deep" not extracted in current test
- **Fix:** Verify exact match in style_keywords works
- **Impact:** Core functionality issue

### 3. Windows Path Handling
- **Issue:** Uses `/` delimiter only
- **Impact:** Windows paths like `C:\Samples\` won't split correctly
- **Test:** Add Windows path edge case test

## Performance Considerations

### Fuzzy Match Complexity
- **Algorithm:** Levenshtein O(m*n) per comparison
- **Dictionaries:** 133 total keywords across 4 sets
- **Per Word:** ~133 distance calculations
- **Per File:** 10 words × 133 = ~1,330 calculations
- **Test:** Benchmark with real-world filenames

### Optimization Opportunities
1. Cache fuzzy matches (memoization)
2. Use trie structure for prefix matching
3. Parallelize dictionary matching
4. Skip fuzzy if exact match found early

## Thread Safety

### Current Status: ✅ Thread-Safe
- All methods use `&self` (immutable borrows)
- Dictionaries built at construction, never modified
- Can be safely shared across threads
- Good for Tauri concurrent IPC commands

### Testing Considerations
- Add `#[test]` with multiple threads
- Verify no data races with miri
- Test with `Arc<AutoTagger>` sharing

## Test Implementation Checklist

### Phase 1: Fix & Foundation (4 tests)
- [ ] Fix failing `test_extract_from_filename`
- [ ] Tag struct: new, full_name, equality
- [ ] AutoTagger::new() success

### Phase 2: Core Functions (30 tests)
- [ ] Fuzzy match: distances 0, 1, 2, 3
- [ ] Fuzzy match: length thresholds 1, 2, 3, 4
- [ ] Filename: all separators (_, -, space, .)
- [ ] Filename: all extensions (.mid, .MID, .midi, .MIDI)
- [ ] Filename: common word filtering
- [ ] Filename: generic tags

### Phase 3: Path & Instruments (20 tests)
- [ ] Path: basic, relative, empty, trailing slash
- [ ] Path: category vs instrument distinction
- [ ] Instruments: basic, multiple, empty, unrecognized
- [ ] Instruments: fuzzy matching

### Phase 4: Integration (24 tests)
- [ ] BPM: integer, fractional, None, boundaries
- [ ] Key: basic, sharp, minor, unknown, None
- [ ] Full extraction: all sample pack styles
- [ ] Deduplication across sources

### Phase 5: Dictionaries (10 tests)
- [ ] Genre keywords: count, content
- [ ] Instrument keywords: count, drums, synths
- [ ] Manufacturer keywords: count, major brands
- [ ] Style keywords: count, no prefix behavior
- [ ] Common words: count, filtering

### Phase 6: Advanced (18 tests)
- [ ] Fuzzy: tied distances, unicode, very long
- [ ] Fuzzy: all genres with typos
- [ ] Fuzzy: all instruments with typos
- [ ] Edge cases: performance, special chars
- [ ] Regression: known bugs

## Success Metrics

### Coverage Goals
- **Target:** 85-90% line coverage
- **Lines:** ~400 functional lines (excluding tests)
- **Tested:** ~340-360 lines needed
- **Gap:** ~40-60 lines acceptable (external deps, std lib)

### Quality Goals
- ✅ All public functions tested
- ✅ All private functions tested indirectly
- ✅ All dictionaries verified
- ✅ All edge cases handled
- ✅ No panics in production code
- ✅ Performance benchmarks pass
- ✅ Thread safety verified

## Estimated Effort

### Time Breakdown
- Phase 1 (Fix): 30 minutes
- Phase 2 (Core): 90 minutes
- Phase 3 (Path/Instruments): 60 minutes
- Phase 4 (Integration): 90 minutes
- Phase 5 (Dictionaries): 30 minutes
- Phase 6 (Advanced): 60 minutes
- **Total:** 6 hours

### With Tool-Enhanced Workflow
- Test generation: 2 hours
- Review & refinement: 1 hour
- Documentation: 30 minutes
- **Total:** 3.5-4 hours

## Quick Command Reference

### Run Tests
```bash
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri
cargo test auto_tagger -- --nocapture
```

### Run Single Test
```bash
cargo test auto_tagger::tests::test_fuzzy_matching -- --nocapture
```

### Coverage Report
```bash
cargo tarpaulin --out Html --output-dir coverage -- --test-threads=1
```

### Benchmarks
```bash
cargo bench --bench auto_tagger_bench
```

## Next Steps

1. **Fix failing test** - Investigate "deep" word extraction
2. **Create test fixtures** - Sample paths, instruments, generators
3. **Implement Phase 1** - Fix + foundation (4 tests)
4. **Implement Phase 2** - Core functionality (30 tests)
5. **Run coverage** - Verify 30-40% coverage after Phase 2
6. **Continue phases** - Until 85%+ coverage achieved
7. **Security review** - Run security-sentinel agent
8. **Performance review** - Run benchmarks with real data
9. **Production approval** - Get rust-backend agent sign-off

## Test File Structure

```rust
// pipeline/src-tauri/src/core/analysis/auto_tagger.rs

#[cfg(test)]
mod tests {
    use super::*;

    // Module 1: Tag Struct Tests
    mod tag_tests {
        // 6 tests
    }

    // Module 2: Construction Tests
    mod construction_tests {
        // 3 tests
    }

    // Module 3: Filename Extraction Tests
    mod filename_tests {
        // 20 tests
    }

    // Module 4: Path Extraction Tests
    mod path_tests {
        // 12 tests
    }

    // Module 5: Instrument Extraction Tests
    mod instrument_tests {
        // 8 tests
    }

    // Module 6: Fuzzy Matching Tests
    mod fuzzy_tests {
        // 18 tests
    }

    // Module 7: BPM/Key Tests
    mod metadata_tests {
        // 12 tests
    }

    // Module 8: Integration Tests
    mod integration_tests {
        // 12 tests
    }

    // Module 9: Dictionary Tests
    mod dictionary_tests {
        // 10 tests
    }

    // Module 10: Edge Case Tests
    mod edge_case_tests {
        // 8 tests
    }

    // Module 11: Regression Tests
    mod regression_tests {
        // 3 tests
    }

    // Test Helpers
    mod test_helpers {
        pub fn sample_vengeance_path() -> &'static str { ... }
        pub fn sample_gm_instruments() -> Vec<String> { ... }
        pub fn generate_long_string(len: usize) -> String { ... }
    }
}
```

## Key Takeaways

1. **112 tests needed** for 85-90% coverage
2. **1 failing test** must be fixed first ("deep" word)
3. **Fuzzy matching** is the most complex area (18 tests)
4. **4 dictionaries** with 133 total keywords
5. **Thread-safe** design, good for concurrent use
6. **CamelCase limitation** is documented, not a bug
7. **Performance testing** needed for large-scale use
8. **3.5-4 hours** estimated with tool-enhanced workflow
