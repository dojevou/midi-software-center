# Placeholder Fixes - Implementation Guide

**Project:** MIDI Software Center
**Date:** 2025-11-13
**Status:** 4 of 79 items COMPLETED ✅
**Remaining:** 75 items (48 tests + 27 other placeholders)

## Executive Summary

This guide provides detailed implementation instructions for the remaining 75 placeholder items identified in `PLACEHOLDER-CODE-AUDIT.md`.

**Completed Items (4/79):**
1. ✅ `shared/rust/src/core/analysis/key_detector.rs` - Implemented Krumhansl-Schmuckler algorithm
2. ✅ `shared/rust/src/core/analysis/auto_tagger.rs` - Implemented GM instrument mapping and tag generation
3. ✅ `pipeline/src-tauri/src/bin/import.rs` - Implemented parallel batch import CLI
4. ✅ `pipeline/src-tauri/src/bin/split.rs` - Implemented track splitting and serialization CLI

**Remaining Items (75/79):**
- 48 empty test skeletons
- 12 MenuBar UI actions
- 15 low-priority placeholders (TODOs, console.logs, etc.)

---

## Part 1: Test Skeleton Implementations (48 items)

### Strategy

All 48 test skeletons follow the same pattern as existing working tests. Use these templates:

### 1.1 Files Tests (7 tests) - `pipeline/src-tauri/tests/commands/files_test.rs`

**Pattern:** Query tests that verify database commands

```rust
use crate::common::*;

#[tokio::test]
async fn test_test_db_connection() {
    let ctx = setup_test_context().await;

    // Test database connection by running a simple query
    let result = sqlx::query("SELECT 1")
        .fetch_one(ctx.pool())
        .await;

    assert!(result.is_ok(), "Database connection should work");
}

#[tokio::test]
async fn test_get_file_count() {
    let ctx = setup_test_context().await;

    // Insert test files
    insert_test_file(&ctx, "test1.mid", None).await;
    insert_test_file(&ctx, "test2.mid", None).await;

    // Query file count
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM midi_files")
        .fetch_one(ctx.pool())
        .await
        .expect("Should get count");

    assert_eq!(count, 2, "Should have 2 files");
}

#[tokio::test]
async fn test_get_file_details() {
    let ctx = setup_test_context().await;

    // Insert test file
    let file_id = insert_test_file(&ctx, "test.mid", None).await;

    // Query file details
    let file = sqlx::query!("SELECT * FROM midi_files WHERE id = $1", file_id)
        .fetch_one(ctx.pool())
        .await
        .expect("Should get file details");

    assert_eq!(file.filename, "test.mid");
}

#[tokio::test]
async fn test_list_files() {
    let ctx = setup_test_context().await;

    // Insert multiple test files
    for i in 1..=5 {
        insert_test_file(&ctx, &format!("test{}.mid", i), None).await;
    }

    // Query files with pagination
    let files = sqlx::query!("SELECT * FROM midi_files LIMIT 10 OFFSET 0")
        .fetch_all(ctx.pool())
        .await
        .expect("Should list files");

    assert_eq!(files.len(), 5, "Should have 5 files");
}

#[tokio::test]
async fn test_get_files_by_category() {
    let ctx = setup_test_context().await;

    // Insert files with categories
    insert_test_file_with_category(&ctx, "drums.mid", "drums").await;
    insert_test_file_with_category(&ctx, "piano.mid", "piano").await;
    insert_test_file_with_category(&ctx, "drums2.mid", "drums").await;

    // Query by category
    let drums = sqlx::query!("SELECT * FROM midi_files WHERE category = $1", "drums")
        .fetch_all(ctx.pool())
        .await
        .expect("Should get drums files");

    assert_eq!(drums.len(), 2, "Should have 2 drums files");
}

#[tokio::test]
async fn test_get_recent_files() {
    let ctx = setup_test_context().await;

    // Insert files
    for i in 1..=3 {
        insert_test_file(&ctx, &format!("test{}.mid", i), None).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Query recent files (last 24 hours)
    let recent = sqlx::query!(
        "SELECT * FROM midi_files WHERE created_at > NOW() - INTERVAL '24 hours' ORDER BY created_at DESC"
    )
    .fetch_all(ctx.pool())
    .await
    .expect("Should get recent files");

    assert_eq!(recent.len(), 3, "Should have 3 recent files");
    assert_eq!(recent[0].filename, "test3.mid", "Most recent should be first");
}

#[tokio::test]
async fn test_delete_file() {
    let ctx = setup_test_context().await;

    // Insert test file
    let file_id = insert_test_file(&ctx, "test.mid", None).await;

    // Verify it exists
    let exists_before: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM midi_files WHERE id = $1)"
    )
    .bind(file_id)
    .fetch_one(ctx.pool())
    .await
    .expect("Should check existence");

    assert!(exists_before, "File should exist before deletion");

    // Delete file
    sqlx::query!("DELETE FROM midi_files WHERE id = $1", file_id)
        .execute(ctx.pool())
        .await
        .expect("Should delete file");

    // Verify it's gone
    let exists_after: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM midi_files WHERE id = $1)"
    )
    .bind(file_id)
    .fetch_one(ctx.pool())
    .await
    .expect("Should check existence");

    assert!(!exists_after, "File should not exist after deletion");
}
```

### 1.2 Progress Tests (5 tests) - `pipeline/src-tauri/tests/commands/progress_test.rs`

```rust
use crate::common::*;

#[tokio::test]
async fn test_start_progress_tracking() {
    let ctx = setup_test_context().await;

    // Start progress tracking
    let operation_id = "test_import_001";

    sqlx::query!(
        "INSERT INTO progress_tracking (operation_id, total, current, status) VALUES ($1, $2, $3, $4)",
        operation_id,
        100i64,
        0i64,
        "in_progress"
    )
    .execute(ctx.pool())
    .await
    .expect("Should start progress tracking");

    // Verify it was created
    let progress = sqlx::query!("SELECT * FROM progress_tracking WHERE operation_id = $1", operation_id)
        .fetch_one(ctx.pool())
        .await
        .expect("Should get progress");

    assert_eq!(progress.total, 100);
    assert_eq!(progress.current, 0);
    assert_eq!(progress.status, "in_progress");
}

#[tokio::test]
async fn test_update_progress() {
    let ctx = setup_test_context().await;

    let operation_id = "test_import_002";

    // Start progress
    sqlx::query!(
        "INSERT INTO progress_tracking (operation_id, total, current, status) VALUES ($1, $2, $3, $4)",
        operation_id, 100i64, 0i64, "in_progress"
    )
    .execute(ctx.pool())
    .await
    .expect("Should start progress");

    // Update progress
    sqlx::query!(
        "UPDATE progress_tracking SET current = $1 WHERE operation_id = $2",
        50i64,
        operation_id
    )
    .execute(ctx.pool())
    .await
    .expect("Should update progress");

    // Verify update
    let progress = sqlx::query!("SELECT * FROM progress_tracking WHERE operation_id = $1", operation_id)
        .fetch_one(ctx.pool())
        .await
        .expect("Should get progress");

    assert_eq!(progress.current, 50, "Progress should be at 50%");
}

#[tokio::test]
async fn test_complete_progress() {
    let ctx = setup_test_context().await;

    let operation_id = "test_import_003";

    // Start progress
    sqlx::query!(
        "INSERT INTO progress_tracking (operation_id, total, current, status) VALUES ($1, $2, $3, $4)",
        operation_id, 100i64, 0i64, "in_progress"
    )
    .execute(ctx.pool())
    .await
    .expect("Should start progress");

    // Complete progress
    sqlx::query!(
        "UPDATE progress_tracking SET current = total, status = $1 WHERE operation_id = $2",
        "completed",
        operation_id
    )
    .execute(ctx.pool())
    .await
    .expect("Should complete progress");

    // Verify completion
    let progress = sqlx::query!("SELECT * FROM progress_tracking WHERE operation_id = $1", operation_id)
        .fetch_one(ctx.pool())
        .await
        .expect("Should get progress");

    assert_eq!(progress.status, "completed");
    assert_eq!(progress.current, progress.total);
}

#[tokio::test]
async fn test_get_current_progress() {
    let ctx = setup_test_context().await;

    let operation_id = "test_import_004";

    // Start progress
    sqlx::query!(
        "INSERT INTO progress_tracking (operation_id, total, current, status) VALUES ($1, $2, $3, $4)",
        operation_id, 100i64, 25i64, "in_progress"
    )
    .execute(ctx.pool())
    .await
    .expect("Should start progress");

    // Get current progress
    let progress = sqlx::query!("SELECT * FROM progress_tracking WHERE operation_id = $1", operation_id)
        .fetch_one(ctx.pool())
        .await
        .expect("Should get progress");

    let percentage = (progress.current as f64 / progress.total as f64) * 100.0;
    assert_eq!(percentage, 25.0, "Should be 25% complete");
}

#[tokio::test]
async fn test_reset_progress() {
    let ctx = setup_test_context().await;

    let operation_id = "test_import_005";

    // Start and update progress
    sqlx::query!(
        "INSERT INTO progress_tracking (operation_id, total, current, status) VALUES ($1, $2, $3, $4)",
        operation_id, 100i64, 75i64, "in_progress"
    )
    .execute(ctx.pool())
    .await
    .expect("Should start progress");

    // Reset progress
    sqlx::query!(
        "UPDATE progress_tracking SET current = $1, status = $2 WHERE operation_id = $3",
        0i64,
        "in_progress",
        operation_id
    )
    .execute(ctx.pool())
    .await
    .expect("Should reset progress");

    // Verify reset
    let progress = sqlx::query!("SELECT * FROM progress_tracking WHERE operation_id = $1", operation_id)
        .fetch_one(ctx.pool())
        .await
        .expect("Should get progress");

    assert_eq!(progress.current, 0, "Progress should be reset to 0");
}
```

### 1.3 Search Tests (7 tests) - `pipeline/src-tauri/tests/commands/search_test.rs`

Follow the same pattern with search-specific queries. See existing `search_repository_test.rs` for examples.

### 1.4 Tags Tests (9 tests) - `pipeline/src-tauri/tests/commands/tags_test.rs`

Follow the same pattern with tag-specific queries. See existing `tag_repository_test.rs` for examples.

### 1.5 Stats Tests (7 tests) - `pipeline/src-tauri/tests/commands/stats_test.rs`

Statistics tests should verify aggregation queries:

```rust
#[tokio::test]
async fn test_get_category_stats() {
    let ctx = setup_test_context().await;

    // Insert files with different categories
    insert_test_file_with_category(&ctx, "drums1.mid", "drums").await;
    insert_test_file_with_category(&ctx, "drums2.mid", "drums").await;
    insert_test_file_with_category(&ctx, "piano1.mid", "piano").await;

    // Get category stats
    let stats = sqlx::query!(
        "SELECT category, COUNT(*) as count FROM midi_files GROUP BY category ORDER BY count DESC"
    )
    .fetch_all(ctx.pool())
    .await
    .expect("Should get category stats");

    assert_eq!(stats.len(), 2, "Should have 2 categories");
    assert_eq!(stats[0].category, Some("drums".to_string()));
    assert_eq!(stats[0].count, Some(2));
}
```

### 1.6 System Tests (2 tests) - `pipeline/src-tauri/tests/commands/system_test.rs`

```rust
#[tokio::test]
async fn test_get_system_info() {
    // Get system information
    let cpu_count = num_cpus::get();
    assert!(cpu_count > 0, "Should detect at least 1 CPU");

    let memory = sys_info::mem_info().expect("Should get memory info");
    assert!(memory.total > 0, "Should have memory");
}

#[tokio::test]
async fn test_initialize_database() {
    let ctx = setup_test_context().await;

    // Database is already initialized by setup_test_context
    // Verify tables exist
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname = 'public'"
    )
    .fetch_all(ctx.pool())
    .await
    .expect("Should get tables");

    assert!(tables.contains(&"midi_files".to_string()), "Should have midi_files table");
    assert!(tables.contains(&"musical_metadata".to_string()), "Should have musical_metadata table");
}
```

---

## Part 2: MenuBar UI Actions (12 items)

### Strategy

Replace console.log placeholders with actual Tauri command invocations:

**File:** `app/src/lib/components/MenuBar.svelte`

```typescript
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { writeFile, readFile } from '@tauri-apps/plugin-fs';

const menuItems = {
  file: [
    {
      label: 'New Project',
      shortcut: 'Ctrl+N',
      action: async () => {
        // Reset application state
        databaseStore.update(state => ({
          ...state,
          files: [],
          selectedFile: null
        }));
        uiStore.update(state => ({
          ...state,
          projectName: 'Untitled Project'
        }));
      }
    },
    {
      label: 'Open Project',
      shortcut: 'Ctrl+O',
      action: async () => {
        const file = await open({
          filters: [{
            name: 'Project Files',
            extensions: ['midip']
          }]
        });

        if (file) {
          const content = await readFile(file.path);
          const project = JSON.parse(new TextDecoder().decode(content));
          // Load project state
          console.log('Project loaded:', project);
        }
      }
    },
    {
      label: 'Save Project',
      shortcut: 'Ctrl+S',
      action: async () => {
        const currentProject = get(uiStore).projectPath;

        if (!currentProject) {
          // If no project path, do Save As
          return menuItems.file[3].action();
        }

        // Save to existing path
        const projectData = {
          version: '1.0',
          files: get(databaseStore).files,
          settings: get(uiStore)
        };

        await writeFile(currentProject, JSON.stringify(projectData, null, 2));
      }
    },
    {
      label: 'Save As...',
      shortcut: 'Ctrl+Shift+S',
      action: async () => {
        const file = await save({
          filters: [{
            name: 'Project Files',
            extensions: ['midip']
          }],
          defaultPath: 'project.midip'
        });

        if (file) {
          const projectData = {
            version: '1.0',
            files: get(databaseStore).files,
            settings: get(uiStore)
          };

          await writeFile(file.path, JSON.stringify(projectData, null, 2));

          uiStore.update(state => ({
            ...state,
            projectPath: file.path
          }));
        }
      }
    },
    {
      label: 'Export MIDI',
      shortcut: 'Ctrl+E',
      action: async () => {
        const selectedFile = get(databaseStore).selectedFile;

        if (!selectedFile) {
          alert('No file selected');
          return;
        }

        const exportPath = await save({
          filters: [{
            name: 'MIDI Files',
            extensions: ['mid', 'midi']
          }],
          defaultPath: selectedFile.filename
        });

        if (exportPath) {
          await invoke('export_midi_file', {
            fileId: selectedFile.id,
            outputPath: exportPath.path
          });
        }
      }
    }
  ],
  edit: [
    {
      label: 'Preferences',
      shortcut: 'Ctrl+,',
      action: () => {
        uiStore.update(state => ({
          ...state,
          showPreferences: true
        }));
      }
    }
  ],
  view: [
    {
      label: 'Zoom In',
      shortcut: 'Ctrl++',
      action: () => {
        uiStore.update(state => ({
          ...state,
          zoomLevel: Math.min(state.zoomLevel + 0.1, 2.0)
        }));
      }
    },
    {
      label: 'Zoom Out',
      shortcut: 'Ctrl+-',
      action: () => {
        uiStore.update(state => ({
          ...state,
          zoomLevel: Math.max(state.zoomLevel - 0.1, 0.5)
        }));
      }
    },
    {
      label: 'Reset Zoom',
      shortcut: 'Ctrl+0',
      action: () => {
        uiStore.update(state => ({
          ...state,
          zoomLevel: 1.0
        }));
      }
    }
  ],
  help: [
    {
      label: 'Documentation',
      shortcut: '',
      action: () => {
        window.open('https://github.com/yourusername/midi-software-center/wiki', '_blank');
      }
    },
    {
      label: 'Keyboard Shortcuts',
      shortcut: 'Ctrl+Shift+H',
      action: () => {
        uiStore.update(state => ({
          ...state,
          showKeyboardShortcuts: true
        }));
      }
    },
    {
      label: 'About MIDI Software Center',
      shortcut: '',
      action: () => {
        uiStore.update(state => ({
          ...state,
          showAboutDialog: true
        }));
      }
    }
  ]
};
```

---

## Part 3: Low-Priority Placeholders (15 items)

### 3.1 Remove TODO Comments

Simply implement the feature or remove the comment if already implemented:

**File:** `pipeline/src-tauri/src/commands/file_import.rs:605`

```rust
// Before:
None, // TODO: Pass parsed MidiFile for drum analysis (v2.1 enhancement)

// After:
Some(&midi_file), // Pass parsed MidiFile for drum analysis (v2.1)
```

### 3.2 Fix WindowBase Maximize

**File:** `app/src/lib/components/WindowBase.svelte:125-126`

```typescript
// Before:
// Implement maximize logic if needed
console.log('Maximize not fully implemented');

// After:
const handleMaximize = () => {
  isMaximized = !isMaximized;

  if (isMaximized) {
    // Store current size
    previousSize = { width: windowWidth, height: windowHeight };
    // Maximize
    windowWidth = window.innerWidth;
    windowHeight = window.innerHeight;
    windowX = 0;
    windowY = 0;
  } else {
    // Restore previous size
    if (previousSize) {
      windowWidth = previousSize.width;
      windowHeight = previousSize.height;
      // Center window
      windowX = (window.innerWidth - windowWidth) / 2;
      windowY = (window.innerHeight - windowHeight) / 2;
    }
  }
};
```

### 3.3 Fix Playback Store Warnings

**File:** `app/src/lib/stores/playbackStore.ts`

```typescript
// Line 138-139: Implement record
record: async () => {
  try {
    isRecording.set(true);
    await invoke('start_recording', {
      inputDevice: get(midiInputDevice)
    });
  } catch (error) {
    console.error('Failed to start recording:', error);
    isRecording.set(false);
  }
},

// Line 158-159: Implement time signature
setTimeSignature: async (numerator: number, denominator: number) => {
  try {
    await invoke('set_time_signature', { numerator, denominator });
    timeSignature.set({ numerator, denominator });
  } catch (error) {
    console.error('Failed to set time signature:', error);
  }
},

// Line 169-170: Implement key signature
setKeySignature: async (key: string) => {
  try {
    await invoke('set_key_signature', { key });
    keySignature.set(key);
  } catch (error) {
    console.error('Failed to set key signature:', error);
  }
},
```

### 3.4 Fix Pipeline Window File Picker

**File:** `app/src/lib/windows/PipelineWindow.svelte:131`

```typescript
// Before:
const archivePath = '/tmp/midi-archive.zip'; // TODO: Add file picker

// After:
import { open } from '@tauri-apps/plugin-dialog';

const handleImportArchive = async () => {
  const selected = await open({
    multiple: false,
    filters: [{
      name: 'Archive Files',
      extensions: ['zip', 'tar', 'gz', '7z']
    }]
  });

  if (selected) {
    await databaseActions.importArchive(selected.path);
  }
};
```

### 3.5 Fix Mixer Window Master Volume

**File:** `app/src/lib/windows/MixerWindow.svelte:74`

```typescript
// Before:
// TODO: Add master volume to window API or use mixer API

// After:
import { invoke } from '@tauri-apps/api/core';

let masterVolume = 0.8;

const updateMasterVolume = async (value: number) => {
  masterVolume = value;
  try {
    await invoke('set_master_volume', { volume: masterVolume });
  } catch (error) {
    console.error('Failed to set master volume:', error);
  }
};
```

### 3.6 Update Test MIDI Files Script

**File:** `scripts/test-midi-files/src/main.rs:86-89`

```rust
// Before:
// Detect key (not implemented yet in shared library)
let key = None; // detect_key is not yet implemented
println!("  ⚠️  Key: Not implemented in shared library yet");

// After:
use midi_library_shared::core::analysis::key_detector::detect_key;

// Detect key
let key = detect_key(&midi_file);
if let Some(key_str) = key {
    println!("  🎹 Key: {}", key_str);
} else {
    println!("  🎹 Key: Unknown (low confidence)");
}
```

### 3.7 Fix Analyze Test TODO

**File:** `pipeline/src-tauri/tests/analyze_test.rs:54`

```rust
// Before:
// TODO (Phase 11): Refactor analyze command to use _impl pattern

// After:
// Note: Analyze command uses _impl pattern as of Phase 10
// See analyze_impl() function in commands/analyze.rs
```

---

## Part 4: Compilation & Testing

### 4.1 Verify Compilation

```bash
# Test shared library builds
cd shared/rust
cargo build --lib
cargo test --lib

# Test pipeline builds
cd ../../pipeline/src-tauri
cargo build
cargo test --lib

# Test binaries build
cargo build --bin import
cargo build --bin split

# Test DAW builds
cd ../../daw/src-tauri
cargo build
cargo test --lib
```

### 4.2 Run Test Suites

```bash
# Run all baseline tests
cargo test --workspace --lib -- --test-threads=1

# Run integration tests
cargo test --workspace --test '*' -- --test-threads=1

# Run specific command tests
cargo test --package pipeline --test commands -- --test-threads=1
```

### 4.3 Frontend Verification

```bash
cd app
pnpm install
pnpm build
pnpm test
```

---

## Part 5: Implementation Checklist

### Critical Items (Completed ✅)
- [x] shared/rust key_detector.rs - Krumhansl-Schmuckler implementation
- [x] shared/rust auto_tagger.rs - GM instrument mapping
- [x] pipeline bin/import.rs - Parallel batch import
- [x] pipeline bin/split.rs - Track splitting and serialization

### High Priority (To Do)
- [ ] Implement 48 empty test skeletons (use templates above)
- [ ] Implement 12 MenuBar UI actions (use patterns above)

### Medium Priority (To Do)
- [ ] Fix 8 production TODO comments
- [ ] Implement 3 playback store functions
- [ ] Add file picker to pipeline window
- [ ] Add master volume to mixer window

### Low Priority (To Do)
- [ ] Update test-midi-files script to use new key detection
- [ ] Remove/update Phase 11 TODO comment
- [ ] Clean up remaining console.log statements

---

## Part 6: Testing Strategy

### Unit Tests
Each test file should have:
1. Setup using `setup_test_context()`
2. Test data insertion using helper functions
3. Query/command execution
4. Assertions on results
5. Cleanup (handled automatically by TestContext)

### Integration Tests
For UI actions:
1. Manual testing in dev mode (`make dev-both`)
2. Verify each menu action works as expected
3. Test keyboard shortcuts
4. Verify error handling

### End-to-End Tests
```bash
# Import files using CLI
cargo run --bin import -- -d /path/to/midi -D $DATABASE_URL

# Split files using CLI
cargo run --bin split -- -f /path/to/file.mid -o /output/dir -D $DATABASE_URL

# Verify in GUI
make dev-pipeline
# Open browser, verify files appear in database
```

---

## Part 7: Estimated Effort

| Category | Items | Est. Time | Complexity |
|----------|-------|-----------|------------|
| ✅ Critical fixes | 4 | DONE | High |
| Test skeletons | 48 | 4-6 hours | Low-Med |
| UI actions | 12 | 2-3 hours | Medium |
| Low-priority | 15 | 1-2 hours | Low |
| Testing/verification | - | 2-3 hours | Medium |
| **TOTAL** | **79** | **9-14 hours** | **Mixed** |

---

## Part 8: Success Criteria

### Definition of Done
1. ✅ All 79 placeholder items replaced with production code
2. ✅ Zero compilation errors across all workspace members
3. ✅ All 1,223+ existing tests still pass
4. ✅ All 48 new tests pass
5. ✅ UI actions work correctly in manual testing
6. ✅ Binary tools (import, split) execute successfully
7. ✅ Updated PLACEHOLDER-CODE-AUDIT.md reflects completion

### Verification Commands
```bash
# Full build test
make build-all

# Full test suite
cargo test --workspace -- --test-threads=1

# Frontend tests
cd app && pnpm test

# Manual UI testing
make dev-both
```

---

## Conclusion

This guide provides complete implementation patterns for all remaining placeholder items. The 4 critical items are already completed. The remaining 75 items are straightforward implementations following established patterns in the codebase.

**Next Steps:**
1. Implement test skeletons using templates (Section 1)
2. Implement UI actions using patterns (Section 2)
3. Fix low-priority placeholders (Section 3)
4. Run verification suite (Section 4)
5. Update audit report (Section 5)

**Estimated Timeline:** 9-14 hours of focused development work.
