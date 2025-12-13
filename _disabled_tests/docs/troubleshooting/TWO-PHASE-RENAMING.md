# Two-Phase Renaming System

## Overview

The MIDI pipeline now has **two separate renaming phases** to handle different stages of file processing:

## Phase 0: Strict Sanitization (`sanitize_strict`)
**When:** Immediately after archive extraction
**Purpose:** Clean filenames for safe filesystem operations
**Location:** `pipeline/src-tauri/src/core/naming/sanitizer.rs`

### Rules:
1. Replace all spaces with underscores: `" "` → `"_"`
2. Convert `.midi` extension to `.mid`
3. Keep ONLY: letters, numbers, underscores `_`, hyphens `-`
4. Remove all other special characters: `()[]{}!@#$%^&*+=<>?/\|;:'",`
5. Remove consecutive underscores/hyphens
6. Trim leading/trailing underscores/hyphens
7. Limit to 250 characters

### Examples:
```
"My Song (2023).midi"           → "My_Song_2023.mid"
"bass & lead!.mid"              → "bass_lead.mid"
"file#1@test.mid"               → "file1test.mid"
"test___file---name.mid"        → "test_file_name.mid"
"@#$%.mid"                      → "untitled.mid"
```

### Usage:
```rust
use pipeline::core::naming::sanitize_strict;

let clean = sanitize_strict("My Song (2023).midi");
// Returns: "My_Song_2023.mid"
```

---

## Phase 1: Production Renaming (`generate_production_filename`)
**When:** After analysis (BPM, key detection complete)
**Purpose:** Generate descriptive filenames based on musical metadata
**Location:** `pipeline/src-tauri/src/core/naming/generator.rs`

### Rules:
1. Use detected BPM, key, tags from analysis
2. Follow template: `{bpm}bpm_{key}_{tags}_{description}.mid`
3. Example: `"128bpm_Cmaj_bass_loop.mid"`

### Examples:
```
Original: "My_Song_2023.mid"
After analysis: "128bpm_Cmaj_bass_loop.mid"

Original: "bass_lead.mid"
After analysis: "95bpm_Dmin_bass_lead.mid"
```

---

## Pipeline Order with Two-Phase Renaming:

```
1. Archive Extraction
   └─> Extract .zip/.rar/.7z files

2. Phase 0: Strict Sanitization (NEW)
   └─> sanitize_strict() on all extracted files
   └─> "My Song (2023).midi" → "My_Song_2023.mid"

3. Phase 1: Production Renaming (EXISTING)
   └─> generate_production_filename() (optional)
   └─> Metadata-based naming

4. Import
   └─> Hash, deduplication, database insert

5. Analysis
   └─> BPM, key, drum detection

6. Track Splitting
   └─> Multi-track file separation
```

---

## Implementation Status:

✅ **Phase 0 Function Created:** `sanitize_strict()` in `sanitizer.rs`
✅ **Tests Added:** 6 comprehensive tests covering all cases
✅ **Exported:** Available via `use pipeline::core::naming::sanitize_strict`
⏳ **Orchestrator Integration:** Pending (needs to call sanitize_strict after extraction)
✅ **Documentation Updated:** PIPELINE-STEPS.md, TWO-PHASE-RENAMING.md

---

## Benefits:

1. **Clean Filesystem Names:** No special characters cause issues
2. **Cross-Platform Compatible:** Works on Windows, macOS, Linux
3. **Preserves Metadata:** Original info available for production renaming
4. **Two-Stage Process:** Separation of concerns (safety vs. descriptiveness)

---

## Testing:

Run tests:
```bash
cd pipeline/src-tauri
cargo test sanitize_strict
```

Expected output:
```
test naming::sanitizer::tests::test_sanitize_strict_spaces ... ok
test naming::sanitizer::tests::test_sanitize_strict_midi_extension ... ok
test naming::sanitizer::tests::test_sanitize_strict_special_chars ... ok
test naming::sanitizer::tests::test_sanitize_strict_keep_valid ... ok
test naming::sanitizer::tests::test_sanitize_strict_consecutive ... ok
test naming::sanitizer::tests::test_sanitize_strict_empty ... ok
```

---

## Next Steps:

1. Integrate Phase 0 into orchestrator (after extraction, before import)
2. Add `--skip-sanitize` flag to orchestrator for optional Phase 0
3. Test with real archive collections
4. Monitor for edge cases
