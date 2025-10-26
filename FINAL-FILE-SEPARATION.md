# FINAL FILE SEPARATION & DEDUPLICATION PLAN

**Date:** 2025-10-24
**Project:** MIDI Software Center Migration
**Source Archive:** midi-library-system-refined.tar.gz

---

## 🎯 SOURCE OF TRUTH

After comprehensive analysis of duplicate directories:

**USE THIS:** `/tmp/original-project/midi-library-system/` (ROOT)
- ✅ Most recent (timestamp 1760795368)
- ✅ Most complete (172-line Cargo.toml vs 157)
- ✅ More Rust files (122 vs 111)
- ✅ **Has complete shared library** (24 Rust modules)
- ✅ Has optimized build profiles

**IGNORE/DELETE:**
- ❌ `/tmp/original-project/projects/midi-library-system/` - Older, incomplete
- ❌ `/tmp/original-project/docs-recovered/` - Scattered duplicates, reference only
- ❌ All other scattered files at root level

---

## 📦 COMPONENT SEPARATION MAP

### Component 1: DATABASE

**Destination:** `~/projects/midi-software-center/database/`

#### Files to Migrate (100% from `midi-library-system/database/`)

```
SOURCE → DESTINATION

midi-library-system/database/
├── docker-compose.yml → database/docker-compose.yml
├── migrations/
│   ├── 001_initial_schema.sql → database/migrations/001_initial_schema.sql
│   ├── 002_add_parent_folder.sql → database/migrations/002_add_parent_folder.sql
│   ├── 003_favorites.sql → database/migrations/003_favorites.sql
│   └── 006_track_splits.sql → database/migrations/006_track_splits.sql
├── queries/
│   └── common_queries.sql → database/queries/common_queries.sql
└── scripts/
    └── insert_sample_data.sql → database/scripts/insert_sample_data.sql
```

**DELETE (Duplicates):**
- `midi-library-system/schema.sql` (superseded by migrations)
- `midi-library-system/fix_*.sql` (one-time fixes, not needed)
- `midi-library-system/add_file_categories.sql` (already in migration)
- All SQL files in docs-recovered/

**Result:** Clean database with 4 migrations + queries + sample data

---

### Component 2: SHARED LIBRARY

**Destination:** `~/projects/midi-software-center/shared/rust/`

#### Files to Migrate (100% from `midi-library-system/shared/rust/`)

```
SOURCE → DESTINATION

midi-library-system/shared/rust/
├── Cargo.toml → shared/rust/Cargo.toml
└── src/
    ├── lib.rs → shared/rust/src/lib.rs
    ├── core/
    │   ├── mod.rs → shared/rust/src/core/mod.rs
    │   ├── midi/
    │   │   ├── mod.rs → shared/rust/src/core/midi/mod.rs
    │   │   ├── parser.rs → shared/rust/src/core/midi/parser.rs (921 lines!)
    │   │   ├── types.rs → shared/rust/src/core/midi/types.rs
    │   │   └── error.rs → shared/rust/src/core/midi/error.rs
    │   └── analysis/
    │       ├── mod.rs → shared/rust/src/core/analysis/mod.rs
    │       ├── bpm_detector.rs → shared/rust/src/core/analysis/bpm_detector.rs
    │       ├── key_detector.rs → shared/rust/src/core/analysis/key_detector.rs
    │       ├── key_profiles.rs → shared/rust/src/core/analysis/key_profiles.rs
    │       └── auto_tagger.rs → shared/rust/src/core/analysis/auto_tagger.rs
    └── db/
        ├── mod.rs → shared/rust/src/db/mod.rs
        ├── models/
        │   ├── mod.rs → shared/rust/src/db/models/mod.rs
        │   ├── midi_file.rs → shared/rust/src/db/models/midi_file.rs
        │   ├── midi.rs → shared/rust/src/db/models/midi.rs
        │   ├── analysis.rs → shared/rust/src/db/models/analysis.rs
        │   ├── search.rs → shared/rust/src/db/models/search.rs
        │   ├── sequencer.rs → shared/rust/src/db/models/sequencer.rs
        │   └── error.rs → shared/rust/src/db/models/error.rs
        └── repositories/
            ├── mod.rs → shared/rust/src/db/repositories/mod.rs
            ├── file_repository.rs → shared/rust/src/db/repositories/file_repository.rs
            ├── metadata_repository.rs → shared/rust/src/db/repositories/metadata_repository.rs
            ├── search_repository.rs → shared/rust/src/db/repositories/search_repository.rs
            └── tag_repository.rs → shared/rust/src/db/repositories/tag_repository.rs
```

**Total:** 24 Rust modules - This is production-ready!

**DO NOT migrate from `projects/midi-library-system/shared/` - it only has a placeholder!**

---

### Component 3: PIPELINE

**Destination:** `~/projects/midi-software-center/pipeline/`

#### Backend (from `midi-library-system/pipeline/src-tauri/`)

```
SOURCE → DESTINATION

midi-library-system/pipeline/src-tauri/
├── Cargo.toml → pipeline/src-tauri/Cargo.toml
├── tauri.conf.json → pipeline/src-tauri/tauri.conf.json
├── build.rs → pipeline/src-tauri/build.rs
└── src/
    ├── main.rs → pipeline/src-tauri/src/main.rs
    ├── error.rs → pipeline/src-tauri/src/error.rs
    ├── commands/
    │   ├── mod.rs → pipeline/src-tauri/src/commands/mod.rs
    │   ├── file_import.rs → pipeline/src-tauri/src/commands/file_import.rs
    │   ├── archive_import.rs → pipeline/src-tauri/src/commands/archive_import.rs
    │   ├── analyze.rs → pipeline/src-tauri/src/commands/analyze.rs
    │   ├── search.rs → pipeline/src-tauri/src/commands/search.rs
    │   ├── split_file.rs → pipeline/src-tauri/src/commands/split_file.rs
    │   ├── stats.rs → pipeline/src-tauri/src/commands/stats.rs
    │   ├── tags.rs → pipeline/src-tauri/src/commands/tags.rs
    │   ├── files.rs → pipeline/src-tauri/src/commands/files.rs
    │   ├── system.rs → pipeline/src-tauri/src/commands/system.rs
    │   └── progress.rs → pipeline/src-tauri/src/commands/progress.rs
    ├── database/
    │   ├── mod.rs → pipeline/src-tauri/src/database/mod.rs
    │   └── batch_insert.rs → pipeline/src-tauri/src/database/batch_insert.rs
    ├── io/
    │   └── decompressor/
    │       ├── extractor.rs → pipeline/src-tauri/src/io/decompressor/extractor.rs
    │       └── temp_manager.rs → pipeline/src-tauri/src/io/decompressor/temp_manager.rs
    ├── core/
    │   └── [various modules]
    ├── utils/
    │   └── [utility modules]
    └── bin/
        ├── import_unified.rs → pipeline/src-tauri/src/bin/import_unified.rs
        ├── import.rs → pipeline/src-tauri/src/bin/import.rs
        ├── analyze.rs → pipeline/src-tauri/src/bin/analyze.rs
        └── split.rs → pipeline/src-tauri/src/bin/split.rs
```

#### Frontend (from `midi-library-system/pipeline/`)

```
SOURCE → DESTINATION

midi-library-system/pipeline/
├── package.json → pipeline/package.json
├── vite.config.ts → pipeline/vite.config.ts
├── svelte.config.js → pipeline/svelte.config.js
├── tsconfig.json → pipeline/tsconfig.json
└── src/
    ├── main.ts → pipeline/src/main.ts
    ├── App.svelte → pipeline/src/App.svelte
    ├── lib/
    │   ├── api.ts → pipeline/src/lib/api.ts
    │   ├── types.ts → pipeline/src/lib/types.ts
    │   ├── stores/ → pipeline/src/lib/stores/
    │   ├── components/ → pipeline/src/lib/components/
    │   └── utils/ → pipeline/src/lib/utils/
    └── [additional frontend files]
```

**DO NOT migrate any files from `docs-recovered/` - they're outdated examples!**

---

### Component 4: DAW

**Destination:** `~/projects/midi-software-center/daw/`

#### Backend (from `midi-library-system/daw/src-tauri/`)

```
SOURCE → DESTINATION

midi-library-system/daw/src-tauri/
├── Cargo.toml → daw/src-tauri/Cargo.toml
├── tauri.conf.json → daw/src-tauri/tauri.conf.json
├── build.rs → daw/src-tauri/build.rs
└── src/
    ├── main.rs → daw/src-tauri/src/main.rs
    ├── models/ → daw/src-tauri/src/models/
    ├── commands/
    │   ├── mod.rs → daw/src-tauri/src/commands/mod.rs
    │   ├── midi.rs → daw/src-tauri/src/commands/midi.rs
    │   ├── sequencer.rs → daw/src-tauri/src/commands/sequencer.rs
    │   ├── analysis.rs → daw/src-tauri/src/commands/analysis.rs
    │   ├── search.rs → daw/src-tauri/src/commands/search.rs
    │   ├── export.rs → daw/src-tauri/src/commands/export.rs
    │   └── project.rs → daw/src-tauri/src/commands/project.rs
    ├── midi/
    │   ├── mod.rs → daw/src-tauri/src/midi/mod.rs
    │   └── manager.rs → daw/src-tauri/src/midi/manager.rs (450 lines!)
    ├── sequencer/
    │   ├── mod.rs → daw/src-tauri/src/sequencer/mod.rs
    │   ├── engine.rs → daw/src-tauri/src/sequencer/engine.rs (800+ lines!)
    │   ├── track.rs → daw/src-tauri/src/sequencer/track.rs
    │   └── scheduler.rs → daw/src-tauri/src/sequencer/scheduler.rs
    └── core/
        ├── sequencer/
        │   ├── mod.rs → daw/src-tauri/src/core/sequencer/mod.rs
        │   └── timing.rs → daw/src-tauri/src/core/sequencer/timing.rs
        └── midi/
            ├── loader.rs → daw/src-tauri/src/core/midi/loader.rs
            ├── writer.rs → daw/src-tauri/src/core/midi/writer.rs
            ├── validator.rs → daw/src-tauri/src/core/midi/validator.rs
            └── parser.rs → daw/src-tauri/src/core/midi/parser.rs
```

**Note on DAW MIDI modules:** The DAW has its own MIDI parser/writer/validator in `core/midi/` which is separate from the shared library. This is intentional for real-time playback requirements. DO NOT merge with shared library.

#### Frontend (from `midi-library-system/daw/`)

```
SOURCE → DESTINATION

midi-library-system/daw/
├── package.json → daw/package.json
├── vite.config.ts → daw/vite.config.ts
├── svelte.config.js → daw/svelte.config.js
├── tsconfig.json → daw/tsconfig.json
└── src/
    ├── main.ts → daw/src/main.ts
    ├── App.svelte → daw/src/App.svelte
    ├── lib/
    │   ├── api.ts → daw/src/lib/api.ts
    │   ├── types.ts → daw/src/lib/types.ts
    │   ├── stores/ → daw/src/lib/stores/
    │   ├── components/
    │   │   ├── PianoRoll.svelte → daw/src/lib/components/PianoRoll.svelte (800+ lines!)
    │   │   ├── Sequencer.svelte → daw/src/lib/components/Sequencer.svelte (600+ lines!)
    │   │   └── FavoritesList.svelte → daw/src/lib/components/FavoritesList.svelte
    │   ├── types/ → daw/src/lib/types/
    │   ├── trusty/ → daw/src/lib/trusty/
    │   └── utils/ → daw/src/lib/utils/
    └── [additional frontend files]
```

---

### Component 5: SCRIPTS

**Destination:** `~/projects/midi-software-center/scripts/`

#### CLI Import Tool (from `midi-library-system/scripts/import-tool/`)

```
SOURCE → DESTINATION

midi-library-system/scripts/import-tool/
├── Cargo.toml → scripts/import-tool/Cargo.toml
└── src/
    └── main.rs → scripts/import-tool/src/main.rs
```

#### Launch Scripts (from `midi-library-system/scripts/`)

```
SOURCE → DESTINATION

midi-library-system/scripts/
├── launch-all.sh → scripts/launch/launch-all.sh
├── launch-daw.sh → scripts/launch/launch-daw.sh
├── launch-pipeline.sh → scripts/launch/launch-pipeline.sh
├── stop-all.sh → scripts/launch/stop-all.sh
└── status.sh → scripts/launch/status.sh
```

#### Setup Scripts (Adapt these)

```
SOURCE → ADAPT & MOVE

midi-library-system/
├── complete_setup.sh → scripts/setup/complete_setup.sh (adapt: add error handling)
├── db_helper.sh → scripts/modules/database.sh (refactor into module)
└── import-full-collection.sh → scripts/import/import-collection.sh (adapt)
```

#### Verification Scripts

```
SOURCE → DESTINATION

midi-library-system/pipeline/
├── verify_integration.sh → scripts/verify/integration_test.sh (generalize)
└── verify_quick.sh → scripts/verify/quick_check.sh
```

---

### Component 6: BUILD CONFIGURATION

**Destination:** Root of new project

```
SOURCE → DESTINATION

midi-library-system/
├── Cargo.toml → ~/projects/midi-software-center/Cargo.toml
├── Makefile → ~/projects/midi-software-center/Makefile
├── docker-compose.yml → ~/projects/midi-software-center/docker-compose.yml
└── package.json → ~/projects/midi-software-center/package.json (if exists)
```

---

### Component 7: CONFIGURATION

**Destination:** `~/projects/midi-software-center/config/`

```
SOURCE → ADAPT

midi-library-system/api/
├── pipeline-tauri.conf.json → Review and incorporate into pipeline/src-tauri/tauri.conf.json
└── daw-tauri.conf.json → Review and incorporate into daw/src-tauri/tauri.conf.json
```

**Note:** These may be backup/reference configs. The actual configs are in the respective src-tauri/ directories.

---

## 🗑️ FILES TO DELETE/IGNORE

### Duplicates (Entire Directories)

```
DELETE:
❌ /tmp/original-project/projects/ (entire directory - older copy)
❌ /tmp/original-project/docs-recovered/ (entire directory - scattered docs)
❌ /tmp/original-project/${workspaceFolder}/ (if exists)
```

### Root-Level Junk

```
DELETE from root of archive:
❌ detect_cargo_projects.sh
❌ emergency_fix.sh
❌ install-ubuntu.sh
❌ kilo-setup.sh
❌ rust_build_optimizer.sh
❌ FILE_OVERVIEW.md
❌ INSTALLATION_FLOW.md
❌ KILO_SETUP_GUIDE.md
❌ mcp-servers.json
❌ modes.json
❌ README_KILO.md
❌ README.md (old)
❌ RUST_OPTIMIZATION_GUIDE.md
❌ SETUP_CHECKLIST.md
❌ TAURI_OPTIMIZATION.md
❌ VSCODE-CONFIG-SUMMARY.md
❌ .vscode-* (all files - outdated)
❌ VSCODE-SETUP-GUIDE.md
❌ workspace-*.json
❌ settings.json
❌ midi-library-system-refined.tar.gz (the archive itself)
```

### Duplicate SQL Files

```
DELETE from midi-library-system/:
❌ schema.sql (superseded by migrations)
❌ fix_column_names.sql (one-time fix)
❌ fix_num_tracks.sql (one-time fix)
❌ add_file_categories.sql (already in migration)
```

### Emergency/Fix Scripts

```
DELETE from midi-library-system/:
❌ emergency_fix.sh
❌ fix-all-errors.sh
❌ fix_schema.sh
❌ extract-error-files.sh
❌ SIMPLE-IMPORT-NOW.sh
❌ phase0-preparation.sh
❌ restore_backups.sh (superseded by scripts/grown-up/restore-database.sh)

DELETE from midi-library-system/daw/:
❌ emergency_fix.sh
❌ export-dead-code.sh
❌ rust_build_optimizer.sh (superseded by Cargo.toml profiles)

DELETE from midi-library-system/pipeline/:
❌ src-tauri/fix_repository.sh
❌ src-tauri/models.sh
❌ src-tauri/search.sh
❌ import_directory.sh (superseded by CLI tool)
```

### Duplicate Database Scripts

```
DELETE from midi-library-system/:
❌ setup_database.sh (root level - duplicate)
❌ database/fix-database.sh (one-time fix)
❌ database/scripts/setup_database.sh (if duplicate)
```

---

## ✅ FINAL COMPONENT INVENTORY

### What Each Component Gets

| Component | Rust Files | TS/Svelte Files | SQL Files | Scripts | Configs |
|-----------|-----------|----------------|-----------|---------|---------|
| **Database** | 0 | 0 | 6 | 0 | 1 docker-compose |
| **Shared Library** | 24 | 0 | 0 | 0 | 1 Cargo.toml |
| **Pipeline** | ~45 | ~30 | 0 | 0 | 3 configs |
| **DAW** | ~53 | ~50 | 0 | 0 | 3 configs |
| **Scripts** | 1 (import-tool) | 0 | 0 | ~12 | 0 |
| **Root** | 0 | 0 | 0 | 0 | 3 build configs |
| **TOTAL** | **~123** | **~80** | **6** | **~12** | **~11** |

---

## 📊 DEDUPLICATION STATISTICS

### Files Analyzed

- **Total files in archive:** ~500
- **Unique production files:** 222
- **Duplicates identified:** 278 (55%)
- **Files to migrate:** 222
- **Files to delete/ignore:** 278

### Duplicate Breakdown

| Category | Duplicates Found | Action |
|----------|-----------------|--------|
| Entire `projects/` directory | 111 files | DELETE (older copy) |
| Entire `docs-recovered/` directory | 120 files | DELETE (scattered docs) |
| Root-level configs | 25 files | DELETE (superseded) |
| Emergency scripts | 12 files | DELETE (reactive fixes) |
| Duplicate SQL | 10 files | DELETE (use migrations) |

---

## 🎯 MIGRATION COMMAND SEQUENCE

Once we're ready to migrate, here's the exact sequence:

### Step 1: Copy Source Directory

```bash
# Use ONLY the best version
cd /tmp/original-project
cp -r midi-library-system ~/projects/midi-software-center/original-backup

# We'll migrate FROM this clean source
```

### Step 2: Migrate Database (Phase 1)

```bash
cd ~/projects/midi-software-center
cp -r original-backup/database/* database/
# Verify: ls database/migrations/ should show 4 files
```

### Step 3: Migrate Shared Library (Phase 1)

```bash
cp -r original-backup/shared/rust/* shared/rust/
# Verify: find shared/rust/src -name "*.rs" | wc -l  # Should be 24
```

### Step 4: Migrate Root Configs (Phase 1)

```bash
cp original-backup/Cargo.toml .
cp original-backup/Makefile .
cp original-backup/docker-compose.yml .
```

### Step 5: Migrate Pipeline (Phase 2)

```bash
cp -r original-backup/pipeline/* pipeline/
# Verify backend: ls pipeline/src-tauri/src/commands/
# Verify frontend: ls pipeline/src/lib/
```

### Step 6: Migrate DAW (Phase 3)

```bash
cp -r original-backup/daw/* daw/
# Verify backend: ls daw/src-tauri/src/sequencer/
# Verify frontend: ls daw/src/lib/components/
```

### Step 7: Migrate Scripts (Phase 4)

```bash
mkdir -p scripts/launch scripts/import scripts/verify
cp original-backup/scripts/launch-*.sh scripts/launch/
cp original-backup/scripts/stop-all.sh scripts/launch/
cp original-backup/scripts/status.sh scripts/launch/
cp original-backup/pipeline/verify_*.sh scripts/verify/
cp -r original-backup/scripts/import-tool scripts/
```

### Step 8: Verify Structure

```bash
# Count Rust files
find . -name "*.rs" | grep -v target | wc -l  # Should be ~123

# Count TypeScript/Svelte files
find . -name "*.ts" -o -name "*.svelte" | wc -l  # Should be ~80

# Check migrations
ls database/migrations/*.sql  # Should list 4 files

# Test compilation
cargo build --all
```

---

## ⚠️ CRITICAL SEPARATION RULES

### DO NOT Mix Components

1. **Shared library ONLY contains:**
   - MIDI parsing (for analysis)
   - Musical analysis (BPM, key detection)
   - Database models and repositories
   - NO UI code
   - NO application logic

2. **Pipeline ONLY contains:**
   - Batch import commands
   - File analysis
   - Archive extraction
   - Database insert operations
   - Pipeline-specific UI

3. **DAW ONLY contains:**
   - Real-time sequencer
   - MIDI hardware I/O
   - Playback engine
   - MIDI file loader (for playback)
   - DAW-specific UI (Piano Roll, Sequencer)

4. **Database ONLY contains:**
   - SQL migrations
   - docker-compose
   - Sample data
   - Utility queries

5. **Scripts ONLY contains:**
   - Launch scripts
   - Setup automation
   - CLI import tool
   - Verification scripts

### Why DAW Has Its Own MIDI Modules

The DAW has `core/midi/parser.rs`, `loader.rs`, `writer.rs`, `validator.rs` which seem to duplicate the shared library. **This is intentional:**

- **Shared library MIDI parser:** Used for analysis and metadata extraction (used by Pipeline)
- **DAW MIDI modules:** Optimized for real-time playback and hardware I/O

**Action:** Keep both. They serve different purposes and have different performance requirements.

---

## 🚀 READY FOR CLEAN MIGRATION

**Source Directory:** `/tmp/original-project/midi-library-system/` (ONLY THIS ONE)
**Destination:** `~/projects/midi-software-center/`
**Files to Migrate:** 222 production-ready files
**Duplicates to Ignore:** 278 files
**Estimated Time:** 2-3 hours for copying + verification

**Next Action:** Begin Step 1 (Copy source directory for clean migration)
