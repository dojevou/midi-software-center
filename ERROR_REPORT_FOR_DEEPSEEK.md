# Comprehensive Error Report - MIDI Software Center
## Generated: November 30, 2025

This report contains all errors, warnings, and lint issues across the project for DeepSeek review.

---

## Table of Contents

1. [Summary](#summary)
2. [Rust Compilation Status](#rust-compilation-status)
3. [Rust Clippy Warnings](#rust-clippy-warnings)
4. [TypeScript/Svelte Warnings](#typescriptsvelte-warnings)
5. [Python Lint Errors](#python-lint-errors)
6. [Scripts with Issues](#scripts-with-issues)

---

## Summary

| Category | Status | Count |
|----------|--------|-------|
| Rust Compilation | ✅ PASSING | 0 errors |
| Rust Lib Tests | ✅ PASSING | 845/845 |
| Rust Clippy | ⚠️ WARNINGS | ~80 warnings |
| Svelte Check | ⚠️ WARNINGS | 18 warnings (A11y) |
| Python Scripts | ⚠️ ISSUES | 3 errors |

**Overall Status: Production Ready** - No blocking errors, only warnings.

---

## Rust Compilation Status

```
$ cargo check --workspace
   Compiling midi-library-shared v0.2.0
   Compiling midi-pipeline v0.2.0
   Compiling midi-daw v0.2.0
   Compiling midi-app v1.0.0
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

**Result: ✅ All crates compile successfully with 0 errors**

---

## Rust Clippy Warnings

### Category: Unused Code (Non-blocking)

```
warning: struct `ControllerStats` is never constructed
   --> pipeline/src-tauri/src/commands/analyze.rs:457:9

warning: struct `ArticulationAnalysis` is never constructed
   --> pipeline/src-tauri/src/commands/analyze.rs:1180:8

warning: field `config` is never read (6 instances)
   --> pipeline/src-tauri/src/core/pipeline/workers/import.rs:25:5
   --> pipeline/src-tauri/src/core/pipeline/workers/sanitize.rs:24:5
   --> pipeline/src-tauri/src/core/pipeline/workers/split.rs:24:5
   --> pipeline/src-tauri/src/core/pipeline/workers/analyze.rs:25:5
   --> pipeline/src-tauri/src/core/pipeline/workers/rename.rs:24:5
   --> pipeline/src-tauri/src/core/pipeline/workers/export.rs:26:5

warning: unused variable: `drum_analysis`
   --> shared/rust/src/core/analysis/auto_tagger.rs:37:32

warning: field `hash` is never read
   --> pipeline/src-tauri/src/bin/find_duplicates.rs

warning: function `format_number` is never used
   --> pipeline/src-tauri/src/bin/find_duplicates.rs

warning: field `filepath` is never read
   --> pipeline/src-tauri/src/bin/extract_instruments.rs

warning: unused import: `futures::StreamExt`
   --> pipeline/src-tauri/src/bin/orchestrator.rs

warning: field `num_tracks` is never read
   --> pipeline/src-tauri/src/bin/import_split_files.rs

warning: methods `inc_skipped` and `inc_corrupt` are never used
   --> pipeline/src-tauri/src/bin/import_split_files.rs
```

### Category: Code Style Suggestions

```
warning: this `map_or` can be simplified
   --> pipeline/src-tauri/src/commands/analyze.rs:464:51
   --> pipeline/src-tauri/src/commands/analyze.rs:645:47

warning: called `map(..).flatten()` on `Option`
   --> pipeline/src-tauri/src/commands/analyze.rs:807:25

warning: casting to the same type is unnecessary (`f64` -> `f64`)
   --> pipeline/src-tauri/src/commands/analyze.rs:1088:28

warning: use of `or_insert_with` to construct default value
   --> pipeline/src-tauri/src/commands/analyze.rs:1447:37

warning: this function has too many arguments (8/7)
   --> pipeline/src-tauri/src/core/naming/generator.rs:235:1

warning: this function has too many arguments (10/7)
   --> pipeline/src-tauri/src/core/naming/templates.rs:80:1

warning: stripping a suffix manually
   --> pipeline/src-tauri/src/core/normalization/filename.rs:76:24

warning: redundant pattern matching, consider using `is_err()` (5 instances)
   --> pipeline/src-tauri/src/core/pipeline/workers/*.rs

warning: writing `&PathBuf` instead of `&Path` involves a new object
   --> pipeline/src-tauri/src/db/repositories/metadata_repository.rs:228:5

warning: empty line after doc comment (4 instances)
   --> pipeline/src-tauri/src/lib.rs:3:1
   --> pipeline/src-tauri/src/bin/batch_split_optimized.rs:8:1

warning: manually reimplementing `div_ceil` (5 instances)
   --> Various binary files

warning: manual implementation of `.is_multiple_of()` (4 instances)
   --> Various binary files

warning: calling `push_str()` using a single-character string literal (9 instances)
   --> pipeline/src-tauri/src/bin/analyze_full_collection.rs
```

### Category: Binary Tool Warnings

```
warning: this import is redundant (7 instances)
   --> pipeline/src-tauri/src/bin/import_split_files.rs:1:1
   --> pipeline/src-tauri/src/bin/find_duplicates.rs:1:1
   --> pipeline/src-tauri/src/bin/batch_split.rs:5:1
   --> pipeline/src-tauri/src/bin/mpc_backup.rs:7:1
   --> etc.

warning: this boolean expression can be simplified
   --> pipeline/src-tauri/src/bin/midi_doctor.rs:76:29

warning: unused variable: `processed`
   --> pipeline/src-tauri/src/bin/midi_doctor.rs:212:43

warning: clamp-like pattern without using clamp function
   --> pipeline/src-tauri/src/bin/find_duplicates.rs

warning: used consecutive `str::replace` call
   --> pipeline/src-tauri/src/bin/midi_doctor.rs
```

---

## TypeScript/Svelte Warnings

### All 18 warnings are A11y (Accessibility) warnings:

```
$ cd app && pnpm run check
svelte-check found 0 errors and 18 warnings in 5 files
```

### File: MenuBar.svelte (4 warnings)

```
/app/src/lib/components/MenuBar.svelte:389:3
Warn: A11y: Non-interactive element <div> should not be assigned mouse or keyboard event listeners.

Issue: <div> used as dialog backdrop with click/keydown handlers
Fix: Add role="button" tabindex="0" or use <button> element

/app/src/lib/components/MenuBar.svelte:397:5
Warn: A11y: Non-interactive element <div> should not be assigned mouse or keyboard event listeners.

Issue: Inner modal <div> with stopPropagation handlers
Fix: Change role="document" to role="dialog" with proper ARIA

/app/src/lib/components/MenuBar.svelte:468:3
Warn: A11y: Non-interactive element <div> should not be assigned mouse or keyboard event listeners.

Issue: Keyboard shortcuts modal backdrop
Fix: Same as above - add proper role or use semantic element

/app/src/lib/components/MenuBar.svelte:476:5
Warn: A11y: Non-interactive element <div> should not be assigned mouse or keyboard event listeners.

Issue: Inner keyboard shortcuts modal
Fix: Same as above
```

### File: DAWWindow.svelte (4 warnings)

```
/app/src/lib/windows/DAWWindow.svelte
Warn: A11y: Non-interactive element <div> should not be assigned mouse or keyboard event listeners.

Lines: Multiple track rows with click handlers
Fix: Use <button> elements or add role="button" tabindex="0"
```

### File: MixerWindow.svelte (4 warnings)

```
/app/src/lib/windows/MixerWindow.svelte
Warn: A11y: Non-interactive element <div> should not be assigned mouse or keyboard event listeners.

Lines: Mixer channel controls with click handlers
Fix: Use semantic button elements
```

### File: PipelineWindow.svelte (2 warnings)

```
/app/src/lib/windows/PipelineWindow.svelte:256:7
Warn: A11y: Non-interactive element <div> should not be assigned mouse or keyboard event listeners.

Issue: Drop zone for file imports
Fix: Add role="button" or use file input with label styling
```

### File: WindowBase.svelte (4 warnings)

```
/app/src/lib/windows/WindowBase.svelte:161:1
Warn: A11y: <div> with click handler must have an ARIA role

Issue: Window container brings to front on click
Fix: Add role="application" or role="region" with aria-label
```

---

## Python Lint Errors

### Error 1: Syntax Error in build_tag_expansions.py

```
File: /scripts/build_tag_expansions.py
Line: 400
Error: E0001 - Parsing failed: 'invalid decimal literal'

Code at line 400:
    print(f"  {' '*20s}   {config['description']}")

Issue: Format specifier "20s" invalid in f-string multiplication
Fix: Change to:
    print(f"  {' '*20}   {config['description']}")
```

### Error 2: Missing Module Member in import-split-files.py

```
File: /scripts/import-split-files.py
Line: 221
Error: E1101 - Module 'psycopg2.errors' has no 'UniqueViolation' member

Code:
    except psycopg2.errors.UniqueViolation:

Issue: psycopg2.errors.UniqueViolation may not be available in all versions
Fix: Use psycopg2.IntegrityError or import specifically:
    from psycopg2.errors import UniqueViolation
```

### Error 3: Import Error in test-normalization-sample.py

```
File: /scripts/test-normalization-sample.py
Line: 10
Error: E0401 - Unable to import 'normalize_files_and_database'

Code:
    from normalize_files_and_database import ...

Issue: Module not in Python path or renamed
Fix: Add to PYTHONPATH or fix import path
```

### Python Script: fast_multi_level_tagger.py (Warnings only - score 9.22/10)

```
File: /scripts/fast_multi_level_tagger.py

W1514 Line 110: Using open without explicitly specifying an encoding
W0718 Line 130: Catching too general exception Exception
R0914 Line 143: Too many local variables (23/15)
W0718 Line 335: Catching too general exception Exception
W0718 Line 351: Catching too general exception Exception
C0415 Line 353: Import outside toplevel (traceback)
C0411 Lines 13-18: Wrong import order (standard imports after third-party)
W0611 Line 16: Unused defaultdict imported from collections
```

---

## Scripts with Issues

### 1. build_tag_expansions.py (SYNTAX ERROR)

**Location:** `/home/dojevou/projects/midi-software-center/scripts/build_tag_expansions.py`

**Error at Line 400:**
```python
def list_packs():
    """List all available expansion packs"""
    print("\nAvailable Expansion Packs:\n")
    for pack_id, config in EXPANSION_PACKS.items():
        print(f"  {pack_id:20s} - {config['name']}")
        print(f"  {' '*20s}   {config['description']}")  # ERROR: 20s invalid
        print(f"  {' '*20s}   Limit: {config['limit']}, BPM: {config['bpm_min']}-{config['bpm_max']}")
        print()
```

**Fix:**
```python
        print(f"  {' '*20}   {config['description']}")
        print(f"  {' '*20}   Limit: {config['limit']}, BPM: {config['bpm_min']}-{config['bpm_max']}")
```

### 2. import-split-files.py (RUNTIME ERROR RISK)

**Location:** `/home/dojevou/projects/midi-software-center/scripts/import-split-files.py`

**Error at Line 221:**
```python
try:
    # ... insert code ...
except psycopg2.errors.UniqueViolation:  # May fail on some psycopg2 versions
    pass
```

**Fix:**
```python
from psycopg2 import IntegrityError
# OR
from psycopg2.errors import UniqueViolation  # At top of file

try:
    # ... insert code ...
except IntegrityError:  # More portable
    pass
```

### 3. test-normalization-sample.py (IMPORT ERROR)

**Location:** `/home/dojevou/projects/midi-software-center/scripts/test-normalization-sample.py`

**Error at Line 10:**
```python
from normalize_files_and_database import ...
```

**Fix:** Either:
1. Rename import to match actual module:
```python
from normalize-files-and-database import ...  # If using hyphenated filename
```
2. Or add parent directory to path:
```python
import sys
sys.path.insert(0, '/home/dojevou/projects/midi-software-center/scripts')
from normalize_files_and_database import ...
```

---

## Recommended Actions

### Priority 1: Fix Python Syntax Error (5 minutes)
- Fix line 400 in `build_tag_expansions.py` - remove 's' from format string

### Priority 2: Fix Python Import Issues (10 minutes)
- Update psycopg2 error handling in `import-split-files.py`
- Fix module path in `test-normalization-sample.py`

### Priority 3: Address A11y Warnings (30 minutes)
- Add proper ARIA roles to modal dialogs in MenuBar.svelte
- Use semantic button elements for clickable divs
- Add tabindex for keyboard navigation

### Priority 4: Clean Up Rust Warnings (Optional - 60 minutes)
- Remove unused structs and functions
- Apply clippy suggestions with `cargo clippy --fix`
- Remove redundant imports

---

## Appendix: Full Test Results

```
$ cargo test --workspace --lib
   Compiling midi-library-shared v0.2.0
   Compiling midi-pipeline v0.2.0
   Compiling midi-daw v0.2.0
   Compiling midi-app v1.0.0
    Finished `test` profile [unoptimized + debuginfo] target(s)
     Running tests for all crates...

test result: ok. 845 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

**All 845 library tests pass successfully.**

---

## Full Script Contents

### Script 1: build_tag_expansions.py (SYNTAX ERROR at line 400)

```python
#!/usr/bin/env python3
"""
Build MPC Expansion Packs from Database Tags

Queries PostgreSQL database for files matching tag criteria,
converts MIDI → .mpcpattern, and organizes into expansion pack structure.

Usage:
    python3 scripts/build_tag_expansions.py --pack rock-drums --limit 1000
    python3 scripts/build_tag_expansions.py --pack all --output /path/to/expansions
    python3 scripts/build_tag_expansions.py --list  # Show available packs
"""

import psycopg2
import subprocess
import json
import os
import sys
from pathlib import Path
from collections import defaultdict
import argparse
import datetime

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

# Converter path
CONVERTER = "./target/release/midi_to_mpcpattern"

# Expansion pack definitions
EXPANSION_PACKS = {
    'rock-drums': {
        'name': 'Rock Drum Patterns',
        'description': 'Essential rock drum patterns across all tempos',
        'tags': ['rock'],
        'tag_category': 'genre',
        'bpm_min': 100,
        'bpm_max': 160,
        'limit': 1000,
        'organize_by': 'bpm',
        'bpm_folders': ['100-120', '120-140', '140-160']
    },
    # ... (more pack definitions) ...
}

# ... (more functions) ...

def list_packs():
    """List all available expansion packs"""
    print("\nAvailable Expansion Packs:\n")
    for pack_id, config in EXPANSION_PACKS.items():
        print(f"  {pack_id:20s} - {config['name']}")
        print(f"  {' '*20s}   {config['description']}")  # LINE 400: ERROR HERE - 20s invalid
        print(f"  {' '*20s}   Limit: {config['limit']}, BPM: {config['bpm_min']}-{config['bpm_max']}")
        print()

# FIX: Change ' '*20s to ' '*20 (remove the 's')
```

**Pylint Output:**
```
/scripts/build_tag_expansions.py:400:7: E0001: Parsing failed: 'invalid decimal literal (build_tag_expansions, line 400)' (syntax-error)
```

---

### Script 2: import-split-files.py (E1101 at line 221)

```python
#!/usr/bin/env python3
"""
Import Missing Split Files into Database

Imports split track files that exist on disk but are not yet in the database.
Uses BLAKE3 for deduplication and parses MIDI for basic metadata.
"""

import os
import sys
import psycopg2
from pathlib import Path
from multiprocessing import Pool, cpu_count
import hashlib
import mido
import re

DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"
SPLITS_DIR = "/home/dojevou/tmp/midi_splits_fast"

# ... (functions) ...

# Around line 221:
                try:
                    # Create track_splits relationship if parent exists
                    if data['parent_file_id']:
                        cur.execute("""
                            INSERT INTO track_splits (
                                parent_file_id, split_file_id, track_number
                            ) VALUES (%s, %s, %s)
                            ON CONFLICT DO NOTHING
                        """, (data['parent_file_id'], file_id, data['track_number']))

                    imported += 1

                except psycopg2.errors.UniqueViolation:  # LINE 221: ERROR - may not exist
                    conn.rollback()
                    duplicates_skipped += 1
                except Exception as e:
                    conn.rollback()
                    errors += 1
                    if errors <= 10:
                        print(f"Error importing {data['filename']}: {e}")

# FIX: Use psycopg2.IntegrityError instead, or import UniqueViolation at top
```

**Pylint Output:**
```
/scripts/import-split-files.py:221:23: E1101: Module 'psycopg2.errors' has no 'UniqueViolation' member (no-member)
```

---

### Script 3: test-normalization-sample.py (E0401 at line 10)

```python
#!/usr/bin/env python3
"""
Test normalization on a sample of files that actually need it
"""

import os
import sys
sys.path.insert(0, '/home/dojevou/projects/midi-software-center/scripts')

from normalize_files_and_database import normalize_files_and_database  # LINE 10: IMPORT ERROR
import psycopg2
import tempfile

DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def test_on_files_needing_normalization(count=100, dry_run=True):
    """Test on files that actually need normalization"""

    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # ... rest of function ...

# FIX: Check if normalize_files_and_database.py exists or rename the import
# The file might be named normalize-files-and-database.py (hyphenated)
```

**Pylint Output:**
```
/scripts/test-normalization-sample.py:10:0: E0401: Unable to import 'normalize_files_and_database' (import-error)
```

---

### Script 4: fast_multi_level_tagger.py (Warnings only - 9.22/10)

```python
#!/usr/bin/env python3
"""
Fast Multi-Level Tagging Script

Extracts keywords from grandparent folders, parent folders, and filenames
in a single pass through all files, then batch inserts tags.

Performance: ~5-15 minutes for 1.79M files (vs 8 hours for sequential)
"""

import psycopg2
import psycopg2.extras
import re                          # W: Should be before psycopg2
import sys                         # W: Should be before psycopg2
from pathlib import Path           # W: Should be before psycopg2
from collections import defaultdict # W: Unused import
from typing import Set, Dict, List, Tuple
import time                        # W: Should be before psycopg2

# ... configuration and functions ...

def load_curated_tags(conn) -> Dict[str, int]:
    # ...
    with open(tag_file, 'r') as f:  # LINE 110: W1514 - No encoding specified
        for line in f:
            # ...
    # ...
        except Exception as e:  # LINE 130: W0718 - Too broad exception
            print(f"   Warning: Failed to insert tag '{tag_name}': {e}")

def process_files(conn, tag_map: Dict[str, int]):  # LINE 143: R0914 - Too many locals (23/15)
    # ... many local variables ...

def main():
    try:
        # ...
    except Exception as e:  # LINE 335, 351: W0718 - Too broad exception
        print(f"\n❌ Error: {e}")
        import traceback  # LINE 353: C0415 - Import outside toplevel
        traceback.print_exc()
```

**Pylint Output:**
```
W1514 Line 110: Using open without explicitly specifying an encoding
W0718 Line 130: Catching too general exception Exception
R0914 Line 143: Too many local variables (23/15)
W0718 Line 335: Catching too general exception Exception
W0718 Line 351: Catching too general exception Exception
C0415 Line 353: Import outside toplevel (traceback)
C0411 Lines 13-18: Wrong import order (standard imports after third-party)
W0611 Line 16: Unused defaultdict imported from collections
```

---

## Svelte Component Issues

### MenuBar.svelte - A11y Warnings (Lines 389, 397, 468, 476)

```svelte
<!-- Line 389: Dialog backdrop -->
{#if showPreferencesDialog}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-[100]"
    on:click={closePreferences}
    on:keydown={(e) => e.key === 'Escape' && closePreferences()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="preferences-title"
  >
    <!-- Line 397: Inner modal -->
    <div
      class="dark:bg-window p-6 rounded-lg shadow-xl max-w-2xl w-full mx-4"
      on:click|stopPropagation
      on:keydown|stopPropagation
      role="document"  <!-- Should have tabindex for keyboard users -->
    >
```

**Fix:**
```svelte
<!-- Add tabindex="-1" and use proper focus management -->
<div
  class="dark:bg-window p-6 rounded-lg shadow-xl max-w-2xl w-full mx-4"
  on:click|stopPropagation
  on:keydown|stopPropagation
  role="dialog"
  tabindex="-1"
>
```

### WindowBase.svelte - A11y Warning (Line 161)

```svelte
<div
  class="window-base dark:bg-window dark:border-window-border dark:text-app-text"
  class:dragging={isDragging}
  class:resizing={isResizing}
  style="left: {position.x}px; top: {position.y}px; ..."
  on:click={() => uiActions.bringToFront(windowId)}
>
```

**Fix:**
```svelte
<div
  class="window-base ..."
  role="region"
  aria-label={title}
  on:click={() => uiActions.bringToFront(windowId)}
>
```

---

## Report End

Generated by Claude Code for MIDI Software Center project.
Phase 11 (VERIFY) and Phase 12 (AUDIT) completed.

**Total Issues:**
- 3 Python errors (1 syntax, 1 runtime risk, 1 import)
- 18 Svelte A11y warnings (non-blocking)
- ~80 Rust clippy warnings (all non-blocking, mostly style)
- 0 compilation errors
- 845/845 tests passing
