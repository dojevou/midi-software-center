# Auto Tagger Test Analysis

## Module Overview

**File:** `pipeline/src-tauri/src/core/analysis/auto_tagger.rs`
**Size:** 510 lines
**Purpose:** Intelligent tag extraction from MIDI filenames, paths, and content
**Current Tests:** 5 tests (1 failing)
**Current Coverage:** ~20%
**Target Coverage:** 85%+

## Function Inventory

### Public Functions (2)
1. `AutoTagger::new() -> Result<Self, regex::Error>`
   - Creates tagger with keyword dictionaries
   - Compiles regex pattern
   - **Edge Cases:** Regex compilation failure (should never happen)

2. `AutoTagger::extract_tags(file_path, file_name, midi_instruments, bpm, key_signature) -> Vec<Tag>`
   - Main entry point - orchestrates all extraction
   - **Edge Cases:** Empty inputs, None optionals, malformed paths

### Private Functions (7)
3. `AutoTagger::extract_from_filename(filename: &str) -> Vec<Tag>`
   - Splits filename on separators
   - Matches against dictionaries
   - **Edge Cases:** Empty string, no extension, special chars, unicode

4. `AutoTagger::extract_from_path(path: &str) -> Vec<Tag>`
   - Splits path on `/`
   - Matches folder names
   - **Edge Cases:** Empty path, Windows paths, relative paths, trailing slashes

5. `AutoTagger::extract_from_instruments(instruments: &[String]) -> Vec<Tag>`
   - Maps MIDI GM instrument names
   - **Edge Cases:** Empty array, unrecognized instruments, case variations

6. `AutoTagger::fuzzy_match(input: &str, dictionary: &HashSet<String>) -> Option<String>`
   - Levenshtein distance ≤ 2
   - Exact match first
   - **Edge Cases:** Empty input, single char, very long input, boundary distances

7. `AutoTagger::load_genre_keywords() -> HashSet<String>` (28 genres)
8. `AutoTagger::load_instrument_keywords() -> HashSet<String>` (54 instruments)
9. `AutoTagger::load_manufacturer_keywords() -> HashSet<String>` (24 brands)
10. `AutoTagger::load_style_keywords() -> HashSet<String>` (27 styles)
11. `AutoTagger::load_common_words() -> HashSet<String>` (16 words)

### Tag Struct Functions (3)
12. `Tag::new(name, category) -> Self`
13. `Tag::full_name(&self) -> String`

## Pattern Analysis

### 1. Dictionary Matching Patterns
- **4 dictionaries:** genres (28), instruments (54), manufacturers (24), styles (27)
- **Exact match:** Direct HashSet lookup
- **Fuzzy match:** Levenshtein distance ≤ 2 (threshold)
- **Min length for fuzzy:** 4 characters
- **Case handling:** All lowercase comparisons

### 2. Filename Splitting Patterns
- **Regex:** `[_\-\s.]+` (underscores, hyphens, spaces, dots)
- **Extension removal:** .mid, .MID, .midi, .MIDI
- **Min word length:** 2 characters
- **Common word filter:** 16 stopwords
- **Generic tag threshold:** >3 chars, alphanumeric only

### 3. Path Parsing Patterns
- **Delimiter:** Forward slash `/`
- **Filtering:** Empty strings removed
- **Category mapping:** instruments → "category", others by dictionary

### 4. Fuzzy Matching Algorithm
- **Distance metric:** Levenshtein (edit distance)
- **Threshold:** 2 edits
- **Min input length:** 4 characters
- **Selection:** Minimum distance (closest match)
- **Fallback:** None if no match within threshold

## Edge Case Catalog

### Input Edge Cases
1. **Empty inputs:**
   - Empty filename: `""`
   - Empty path: `""`
   - Empty instruments array: `&[]`
   - None BPM/key

2. **Special characters:**
   - Unicode: `"Café_Böhm_Kick.mid"`
   - Symbols: `"Kick@#$%_128.mid"`
   - Numbers only: `"128.mid"`
   - Mixed case: `"DeEpHoUsE.mid"`

3. **Very long strings:**
   - 256+ char filenames
   - Deep paths (20+ levels)
   - 1000+ char words

4. **Boundary cases:**
   - Single char: `"a.mid"`
   - Two chars: `"ab.mid"`
   - Three chars: `"abc.mid"` (threshold)
   - Four chars: `"abcd.mid"` (fuzzy match min)

5. **Path variations:**
   - Absolute: `/home/user/samples/kick.mid`
   - Relative: `samples/kick.mid`
   - Windows: `C:\Samples\Kick.mid`
   - Trailing slash: `/samples/`
   - No slashes: `kick.mid`

6. **Extension variations:**
   - No extension: `kick`
   - Wrong extension: `kick.wav`
   - Double extension: `kick.mid.bak`
   - Mixed case: `kick.MiDi`

### Dictionary Edge Cases
7. **Genre matching:**
   - Exact: `"house"` → genre:house
   - Fuzzy: `"houze"` → genre:house (distance 1)
   - Compound: `"deephouse"` → genre:deephouse
   - Hyphenated: `"deep-house"` → splits to "deep" + "house"

8. **Instrument matching:**
   - Short words: `"hat"` vs `"hihat"`
   - Plurals: `"drums"` vs `"drum"`
   - Variations: `"bass_drum"` vs `"bassdrum"`

9. **Manufacturer matching:**
   - Typos: `"vengance"` → vengeance
   - Abbreviations: `"NI"` vs `"native_instruments"`

10. **Style matching:**
    - No category prefix (style tags are untagged)
    - Overlap with other dicts: `"deep"` (style) vs `"deephouse"` (genre)

### Fuzzy Match Edge Cases
11. **Distance boundaries:**
    - Distance 0: Exact match
    - Distance 1: `"techno"` ← `"teckno"`
    - Distance 2: `"techno"` ← `"tecnno"` (threshold)
    - Distance 3: No match
    - Tied distances: Multiple matches at same distance

12. **Length requirements:**
    - Length 1-3: No fuzzy match, only exact
    - Length 4+: Fuzzy match enabled
    - Very long: Performance considerations

### BPM/Key Edge Cases
13. **BPM values:**
    - Fractional: `128.7` → `"bpm:129"`
    - Very low: `40.0` → `"bpm:40"`
    - Very high: `200.0` → `"bpm:200"`
    - None: No tag added

14. **Key signatures:**
    - Lowercase: `"C"` → `"key:c"`
    - "unknown": Filtered out
    - None: No tag added
    - Complex: `"C# minor"` → `"key:c# minor"`

### Deduplication Edge Cases
15. **Duplicate tags:**
    - Same tag from filename + path
    - Multiple matches in filename
    - HashSet deduplication behavior

## Test Categories & Scenarios

### Category 1: Tag Struct Tests (6 tests)
**Purpose:** Verify Tag construction and formatting

1. **test_tag_new_with_category**
   - Input: `Tag::new("house", Some("genre"))`
   - Expected: `name: "house", category: Some("genre")`

2. **test_tag_new_without_category**
   - Input: `Tag::new("deep", None::<String>)`
   - Expected: `name: "deep", category: None`

3. **test_tag_full_name_with_category**
   - Input: `Tag::new("house", Some("genre")).full_name()`
   - Expected: `"genre:house"`

4. **test_tag_full_name_without_category**
   - Input: `Tag::new("deep", None::<String>).full_name()`
   - Expected: `"deep"`

5. **test_tag_equality**
   - Input: Two identical tags
   - Expected: PartialEq works correctly

6. **test_tag_hashing**
   - Input: Tag used in HashSet
   - Expected: Hash + Eq work correctly

### Category 2: AutoTagger Construction (3 tests)
**Purpose:** Verify tagger initialization

7. **test_autotagger_new_success**
   - Input: `AutoTagger::new()`
   - Expected: Ok(tagger) with populated dictionaries

8. **test_autotagger_dictionaries_loaded**
   - Input: Check dictionary sizes
   - Expected: 28 genres, 54 instruments, 24 brands, 27 styles, 16 common

9. **test_regex_pattern_compiled**
   - Input: Check split_pattern exists
   - Expected: Pattern compiles successfully

### Category 3: Filename Extraction Tests (20 tests)
**Purpose:** Verify filename parsing and splitting

10. **test_filename_underscore_separator**
    - Input: `"VEC_Deep_Kick_128.mid"`
    - Expected: ["genre:house", "deep", "instrument:kick"]

11. **test_filename_hyphen_separator**
    - Input: `"Deep-House-Kick.mid"`
    - Expected: ["deep", "genre:house", "instrument:kick"]

12. **test_filename_space_separator**
    - Input: `"Deep House Kick.mid"`
    - Expected: ["deep", "genre:house", "instrument:kick"]

13. **test_filename_dot_separator**
    - Input: `"Deep.House.Kick.mid"`
    - Expected: ["deep", "genre:house", "instrument:kick"]

14. **test_filename_mixed_separators**
    - Input: `"Deep_House-Kick.128.mid"`
    - Expected: ["deep", "genre:house", "instrument:kick"]

15. **test_filename_camelcase** (KNOWN LIMITATION)
    - Input: `"TechnoLeadSynth.mid"`
    - Expected: ["tecknoleadsynth"] (single word - camelCase not split)
    - Note: Regex doesn't support lookahead/lookbehind

16. **test_filename_extension_removal_mid**
    - Input: `"kick.mid"`
    - Expected: Extension removed

17. **test_filename_extension_removal_MID**
    - Input: `"kick.MID"`
    - Expected: Extension removed

18. **test_filename_extension_removal_midi**
    - Input: `"kick.midi"`
    - Expected: Extension removed

19. **test_filename_extension_removal_MIDI**
    - Input: `"kick.MIDI"`
    - Expected: Extension removed

20. **test_filename_no_extension**
    - Input: `"kick"`
    - Expected: Processes normally

21. **test_filename_common_words_filtered**
    - Input: `"The_New_Kick_For_Mix.mid"`
    - Expected: Only "kick" remains (the, new, for, mix filtered)

22. **test_filename_short_words_filtered**
    - Input: `"A_B_Kick_C_D.mid"`
    - Expected: Only "kick" (single letters filtered)

23. **test_filename_generic_tags**
    - Input: `"MyCustom_Sound.mid"`
    - Expected: ["mycustom", "sound"] (>3 chars, alphanumeric)

24. **test_filename_numbers_only**
    - Input: `"128.mid"`
    - Expected: [] (numbers alone not tagged)

25. **test_filename_special_chars**
    - Input: `"Kick@#$%_128.mid"`
    - Expected: [] (non-alphanumeric filtered from generic tags)

26. **test_filename_unicode**
    - Input: `"Café_Böhm_Kick.mid"`
    - Expected: ["instrument:kick"] (unicode chars handled)

27. **test_filename_very_long**
    - Input: 256+ char filename
    - Expected: Processes without panic

28. **test_filename_empty**
    - Input: `""`
    - Expected: [] (no tags)

29. **test_filename_multiple_same_dictionary**
    - Input: `"House_Techno_Trance.mid"`
    - Expected: ["genre:house", "genre:techno", "genre:trance"]

### Category 4: Path Extraction Tests (12 tests)
**Purpose:** Verify path parsing and folder matching

30. **test_path_basic_structure**
    - Input: `"/Vengeance/DeepHouse/Drums/Kicks/file.mid"`
    - Expected: ["brand:vengeance", "genre:deephouse", "category:drums"]

31. **test_path_relative**
    - Input: `"Samples/House/Kicks/file.mid"`
    - Expected: ["genre:house", "category:kicks"]

32. **test_path_no_slashes**
    - Input: `"file.mid"`
    - Expected: [] (no path components)

33. **test_path_trailing_slash**
    - Input: `"/Samples/House/"`
    - Expected: ["genre:house"]

34. **test_path_leading_slash**
    - Input: `"/House/Kicks"`
    - Expected: ["genre:house", "category:kicks"]

35. **test_path_windows_style** (edge case)
    - Input: `"C:\\Samples\\House\\Kicks"`
    - Expected: May not split correctly (uses `/` delimiter)

36. **test_path_empty**
    - Input: `""`
    - Expected: []

37. **test_path_very_deep**
    - Input: 20+ nested folders
    - Expected: Processes without panic

38. **test_path_duplicate_folders**
    - Input: `"/House/House/House/file.mid"`
    - Expected: Single "genre:house" (HashSet dedup)

39. **test_path_category_vs_instrument**
    - Input: `"/Drums/file.mid"`
    - Expected: "category:drums" (not "instrument:drums")

40. **test_path_mixed_case**
    - Input: `"/VeNgEaNcE/DeEpHoUsE/file.mid"`
    - Expected: Lowercase matching works

41. **test_path_unicode**
    - Input: `"/Café/Böhm/file.mid"`
    - Expected: Handles unicode paths

### Category 5: Instrument Extraction Tests (8 tests)
**Purpose:** Verify MIDI instrument name matching

42. **test_instruments_basic**
    - Input: `["Acoustic Bass Drum"]`
    - Expected: ["instrument:bass_drum" or "instrument:bassdrum"]

43. **test_instruments_multiple**
    - Input: `["Acoustic Bass Drum", "Synth Lead", "Piano"]`
    - Expected: ["instrument:bass_drum", "instrument:lead", "instrument:piano"]

44. **test_instruments_empty**
    - Input: `[]`
    - Expected: []

45. **test_instruments_unrecognized**
    - Input: `["Unknown Instrument XYZ"]`
    - Expected: [] (no match)

46. **test_instruments_partial_match**
    - Input: `["Lead Synth"]`
    - Expected: ["instrument:lead"] (partial word match)

47. **test_instruments_case_insensitive**
    - Input: `["BASS DRUM", "bass drum"]`
    - Expected: ["instrument:bass_drum"] (dedup)

48. **test_instruments_fuzzy_match**
    - Input: `["Base Drum"]` (typo)
    - Expected: ["instrument:bass_drum"] or ["instrument:bass"] (distance 1)

49. **test_instruments_duplicates**
    - Input: `["Bass Drum", "Bass Drum", "Bass"]`
    - Expected: Deduplicated tags

### Category 6: Fuzzy Matching Tests (18 tests)
**Purpose:** Verify Levenshtein distance matching

50. **test_fuzzy_exact_match**
    - Input: `"techno"` in genre_keywords
    - Expected: Some("techno")

51. **test_fuzzy_distance_1**
    - Input: `"teckno"` (1 swap)
    - Expected: Some("techno")

52. **test_fuzzy_distance_2**
    - Input: `"tecnno"` (1 substitution + 1 deletion)
    - Expected: Some("techno")

53. **test_fuzzy_distance_3_fails**
    - Input: `"tekkko"` (3 edits)
    - Expected: None (exceeds threshold)

54. **test_fuzzy_min_length_3**
    - Input: `"hou"` (length 3)
    - Expected: None (< 4 chars)

55. **test_fuzzy_min_length_4**
    - Input: `"houe"` (length 4)
    - Expected: Some("house") (fuzzy enabled)

56. **test_fuzzy_empty_input**
    - Input: `""`
    - Expected: None

57. **test_fuzzy_single_char**
    - Input: `"h"`
    - Expected: None (too short)

58. **test_fuzzy_tied_distances**
    - Input: Input with multiple matches at distance 2
    - Expected: Returns one (min_by_key is deterministic)

59. **test_fuzzy_typo_vengance**
    - Input: `"vengance"` in manufacturer_keywords
    - Expected: Some("vengeance")

60. **test_fuzzy_typo_splice**
    - Input: `"splace"` (distance 2)
    - Expected: Some("splice")

61. **test_fuzzy_no_match_empty_dict**
    - Input: Any string in empty HashSet
    - Expected: None

62. **test_fuzzy_very_long_input**
    - Input: 1000+ char string
    - Expected: None or correct match (performance check)

63. **test_fuzzy_all_genres**
    - Input: Test fuzzy match for all 28 genres with typos
    - Expected: All match within threshold

64. **test_fuzzy_all_instruments**
    - Input: Test fuzzy match for all 54 instruments with typos
    - Expected: All match within threshold

65. **test_fuzzy_case_sensitivity**
    - Input: `"TECHNO"` (uppercase)
    - Expected: Some("techno") (after lowercase)

66. **test_fuzzy_unicode**
    - Input: `"café"` in dictionary with `"cafe"`
    - Expected: Distance calculation handles unicode

67. **test_fuzzy_closest_match_selection**
    - Input: `"hoz"` could match "house" (dist 3) or exact "hoz" if exists
    - Expected: Closest match selected

### Category 7: BPM/Key Tag Tests (8 tests)
**Purpose:** Verify optional metadata tagging

68. **test_bpm_integer**
    - Input: `Some(128.0)`
    - Expected: "bpm:128"

69. **test_bpm_fractional_round_down**
    - Input: `Some(128.4)`
    - Expected: "bpm:128"

70. **test_bpm_fractional_round_up**
    - Input: `Some(128.7)`
    - Expected: "bpm:129"

71. **test_bpm_very_low**
    - Input: `Some(40.0)`
    - Expected: "bpm:40"

72. **test_bpm_very_high**
    - Input: `Some(200.0)`
    - Expected: "bpm:200"

73. **test_bpm_none**
    - Input: `None`
    - Expected: No bpm tag

74. **test_key_basic**
    - Input: `Some("C")`
    - Expected: "key:c"

75. **test_key_sharp**
    - Input: `Some("C#")`
    - Expected: "key:c#"

76. **test_key_minor**
    - Input: `Some("A minor")`
    - Expected: "key:a minor"

77. **test_key_unknown_filtered**
    - Input: `Some("unknown")`
    - Expected: No key tag

78. **test_key_unknown_mixed_case**
    - Input: `Some("Unknown")`
    - Expected: No key tag

79. **test_key_none**
    - Input: `None`
    - Expected: No key tag

### Category 8: Integration Tests (12 tests)
**Purpose:** End-to-end tag extraction

80. **test_full_extraction_vengeance**
    - Input: Full Vengeance sample path
    - Expected: brand + genre + instrument + bpm + key

81. **test_full_extraction_splice**
    - Input: Splice-style naming
    - Expected: brand:splice + tags

82. **test_full_extraction_loopmasters**
    - Input: Loopmasters pack structure
    - Expected: brand:loopmasters + genre + instrument

83. **test_full_extraction_minimal**
    - Input: Simple filename, no path, no MIDI instruments
    - Expected: Only filename tags

84. **test_full_extraction_all_sources**
    - Input: Tags from filename + path + instruments + bpm + key
    - Expected: All sources contribute unique tags

85. **test_full_extraction_deduplication**
    - Input: "house" in filename AND path
    - Expected: Single "genre:house" tag

86. **test_full_extraction_empty_everything**
    - Input: Empty strings, empty arrays, None values
    - Expected: [] (no tags)

87. **test_full_extraction_genre_priority**
    - Input: Multiple genres in different locations
    - Expected: All genres extracted

88. **test_full_extraction_instrument_priority**
    - Input: Instrument in filename vs MIDI content
    - Expected: Both extracted if different

89. **test_full_extraction_complex_path**
    - Input: `/Samples/Vengeance/Vol2/DeepHouse/Drums/Kicks/VEC_Deep_Kick_128_C.mid`
    - Expected: Comprehensive tag set

90. **test_full_extraction_unicode_path**
    - Input: Path with unicode characters
    - Expected: Handles gracefully

91. **test_full_extraction_real_world_sample**
    - Input: Actual sample pack structure from real library
    - Expected: Realistic tag extraction

### Category 9: Dictionary Tests (10 tests)
**Purpose:** Verify keyword dictionaries

92. **test_genre_keywords_count**
    - Expected: 28 genres

93. **test_genre_keywords_content**
    - Expected: Contains "house", "techno", "trance", etc.

94. **test_instrument_keywords_count**
    - Expected: 54 instruments

95. **test_instrument_keywords_drums**
    - Expected: Contains "kick", "snare", "hihat", etc.

96. **test_instrument_keywords_synths**
    - Expected: Contains "lead", "pad", "pluck", etc.

97. **test_manufacturer_keywords_count**
    - Expected: 24 brands

98. **test_manufacturer_keywords_major_brands**
    - Expected: Contains "vengeance", "splice", "loopmasters"

99. **test_style_keywords_count**
    - Expected: 27 styles

100. **test_style_keywords_no_prefix**
     - Verify style tags have no category in extract_from_filename
     - Expected: "deep" not "style:deep"

101. **test_common_words_count**
     - Expected: 16 common words

102. **test_common_words_filtering**
     - Expected: "the", "and", "for" filtered out

### Category 10: Edge Case & Stress Tests (8 tests)
**Purpose:** Boundary conditions and error handling

103. **test_very_long_filename_performance**
     - Input: 1024+ char filename
     - Expected: Completes within reasonable time

104. **test_very_deep_path_performance**
     - Input: 50+ nested folders
     - Expected: Completes within reasonable time

105. **test_many_instruments_performance**
     - Input: 100+ instrument strings
     - Expected: Completes within reasonable time

106. **test_all_special_chars_filename**
     - Input: `"!@#$%^&*()_+-=[]{}|;':\",./<>?.mid"`
     - Expected: Handles gracefully, no panic

107. **test_null_bytes_in_strings** (security)
     - Input: Strings with `\0` bytes
     - Expected: Handles gracefully

108. **test_extremely_long_word**
     - Input: Single 10000-char word in filename
     - Expected: No panic, processes correctly

109. **test_tag_deduplication_stress**
     - Input: 100 duplicate tags from different sources
     - Expected: HashSet deduplicates correctly

110. **test_concurrent_access** (if applicable)
     - Input: Multiple threads using same AutoTagger
     - Expected: Thread-safe (all &self methods)

### Category 11: Regression Tests (3 tests)
**Purpose:** Prevent known bugs from reoccurring

111. **test_deep_word_categorization_bug**
     - Input: `"Deep_House_Kick.mid"`
     - Expected: "deep" as style tag (no category), NOT filtered
     - Note: Current test_extract_from_filename FAILS on this

112. **test_camelcase_limitation**
     - Input: `"TechnoLeadSynth.mid"`
     - Expected: Single word (known limitation documented)

113. **test_path_category_instrument_distinction**
     - Input: Path with "drums" folder
     - Expected: "category:drums" not "instrument:drums"

## Test Count Estimation

### By Category:
1. Tag Struct: 6 tests
2. AutoTagger Construction: 3 tests
3. Filename Extraction: 20 tests
4. Path Extraction: 12 tests
5. Instrument Extraction: 8 tests
6. Fuzzy Matching: 18 tests
7. BPM/Key Tags: 12 tests (8 + 4 additional)
8. Integration: 12 tests
9. Dictionary: 10 tests
10. Edge Cases: 8 tests
11. Regression: 3 tests

**TOTAL: 112 tests**

### Coverage Projection:

**Current State:**
- 5 tests (1 failing)
- ~20% coverage estimate

**After Adding All Tests:**
- 112 comprehensive tests
- Expected coverage: **85-90%**

**Uncovered Areas (expected ~10-15%):**
- Regex internal implementation (external crate)
- Some HashSet internal operations
- Extremely rare edge cases (null bytes, etc.)
- Some trivial getters/setters

## Priority Test Implementation Order

### Phase 1: Fix Failing Test (1 test)
- Fix `test_extract_from_filename` - "deep" word issue

### Phase 2: Core Functionality (30 tests)
1. Tag struct tests (6)
2. AutoTagger construction (3)
3. Fuzzy matching basics (10)
4. Filename extraction basics (11)

### Phase 3: Path & Instruments (20 tests)
5. Path extraction (12)
6. Instrument extraction (8)

### Phase 4: Integration & Metadata (24 tests)
7. BPM/Key tags (12)
8. Integration tests (12)

### Phase 5: Dictionaries (10 tests)
9. Dictionary tests (10)

### Phase 6: Advanced Fuzzy Matching (8 tests)
10. Advanced fuzzy tests (8 remaining)

### Phase 7: Edge Cases & Stress (18 tests)
11. Filename edge cases (9 remaining)
12. Edge case & stress tests (8)
13. Regression tests (3)

## Key Insights

### 1. The "deep" Bug
Current test expects "deep" to be extracted from `"Deep_House_Kick.mid"` but it's being filtered. This is likely because:
- "deep" matches style_keywords (line 391)
- Style tags are added without category prefix (line 151)
- But "deep" is also < 4 chars, so fuzzy matching doesn't apply
- Need exact match in dictionary

**Fix needed:** Verify "deep" is in style_keywords and exact match logic works

### 2. CamelCase Limitation
The regex `[_\-\s.]+` cannot split camelCase (requires lookahead/lookbehind).
- This is a **documented limitation** (line 64)
- Test should verify this behavior, not try to fix it

### 3. Category Assignment Logic
Different categories for same word depending on source:
- From filename: instruments → "instrument:X"
- From path: instruments → "category:X"
- This is intentional design

### 4. Fuzzy Matching Performance
Levenshtein distance calculation is O(m*n) for string lengths m, n.
- With 4 dictionaries (133 total keywords)
- Each filename word checked against all dictionaries
- For 10-word filename: ~1,330 distance calculations
- Need performance tests for real-world samples

### 5. Unicode Handling
Rust's `char` type handles unicode correctly, but:
- Levenshtein distance counts unicode chars correctly
- File path parsing may behave differently on Windows vs Linux
- Need explicit unicode tests

### 6. Thread Safety
All methods use `&self`, dictionaries are immutable after construction.
- AutoTagger is thread-safe for reading
- Can be shared across threads safely
- Good design for Tauri concurrent operations

## Recommended Test Fixtures

Create test helper module with:

```rust
// Test data generators
fn sample_vengeance_path() -> &'static str
fn sample_splice_path() -> &'static str
fn sample_loopmasters_path() -> &'static str

// MIDI instrument samples
fn sample_gm_instruments() -> Vec<String>

// Edge case generators
fn generate_long_string(len: usize) -> String
fn generate_deep_path(depth: usize) -> String
fn generate_unicode_filename() -> String
```

## Test Implementation Notes

1. **Use test helpers:** Create fixture functions for common test data
2. **Property-based testing:** Consider using `proptest` for fuzzy matching edge cases
3. **Benchmark tests:** Add criterion benchmarks for performance-critical paths
4. **Documentation:** Each test should have clear docstring explaining what it tests
5. **Organize by module:** Group tests in submodules matching categories
6. **Test data files:** Consider loading real-world sample pack structures from JSON

## Coverage Gaps (Expected)

Even with 112 tests, some code may remain uncovered:
- Regex crate internals (external dependency)
- HashSet implementation details (std library)
- Unreachable error paths (regex compile failure)
- Some iterator chain optimizations

**These are acceptable gaps for production code.**

## Success Criteria

To achieve 85%+ coverage:
1. All 11 functions tested independently
2. All 4 dictionaries verified
3. All splitting patterns tested (underscore, hyphen, space, dot)
4. Fuzzy matching: distances 0, 1, 2, 3 tested
5. All optional parameters tested (None, Some)
6. Deduplication verified
7. Edge cases handled (empty, unicode, long strings)
8. Integration tests cover real-world scenarios

**Estimated time to implement: 4-6 hours** (with tool-enhanced workflow)
