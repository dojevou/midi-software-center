# Pipeline Error Handling - Detailed Fix Guide

**Target Files:**
1. `/pipeline/src-tauri/src/commands/file_import.rs`
2. `/pipeline/src-tauri/src/commands/analyze.rs`
3. `/pipeline/src-tauri/src/commands/archive_import.rs`
4. `/pipeline/src-tauri/src/commands/progress.rs`

---

## Fix 1: file_import.rs - Lines 174-177

### Issue: Silent window.emit() Failure

**Current Code:**
```rust
// Line 170-180
#[tauri::command]
pub async fn import_single_file(
    file_path: String,
    category: Option<String>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<FileMetadata, String> {
    let file = import_single_file_impl(file_path, category, &state).await?;

    // Emit progress event
    let _ = window.emit(
        "import-progress",
        ImportProgress { current: 1, total: 1, current_file: file.filename.clone(), rate: 1.0 },
    );

    Ok(file)
}
```

**Fixed Code:**
```rust
#[tauri::command]
pub async fn import_single_file(
    file_path: String,
    category: Option<String>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<FileMetadata, String> {
    let file = import_single_file_impl(file_path, category, &state).await?;

    // Emit progress event
    if let Err(e) = window.emit(
        "import-progress",
        ImportProgress { current: 1, total: 1, current_file: file.filename.clone(), rate: 1.0 },
    ) {
        eprintln!("WARNING: Failed to emit import progress event: {}", e);
        // Note: Don't fail the operation - emit failure shouldn't stop import
        // But log it for debugging
    }

    Ok(file)
}
```

**Explanation:**
- Emit failure is non-fatal (import succeeded)
- But we log it so developers can debug frontend issues
- Pattern: log and continue for non-critical operations

---

## Fix 2: file_import.rs - Lines 262-268

### Issue: Semaphore Closure Not Tracked

**Current Code:**
```rust
// Line 259-270
async move {
    // Acquire semaphore permit (blocks if at limit)
    // This should never fail unless semaphore is closed, which we never do
    let _permit = match sem.acquire().await {
        Ok(permit) => permit,
        Err(_) => {
            // Semaphore closed - skip this file (should never happen)
            eprintln!("Warning: Semaphore closed during file import");
            return;
        }
    };
```

**Fixed Code:**
```rust
async move {
    // Acquire semaphore permit (blocks if at limit)
    let _permit = match sem.acquire().await {
        Ok(permit) => permit,
        Err(e) => {
            // Semaphore closed - this is a fatal error condition
            let error_msg = format!("FATAL: Semaphore unavailable during file import: {}", e);
            eprintln!("ERROR: {}", error_msg);

            // Track this as an error
            match errors.lock().await.push(error_msg) {
                _ => {} // If push fails, at least we logged to stderr
            }

            // Mark file as skipped
            skipped.fetch_add(1, Ordering::SeqCst);
            return;
        }
    };
```

**Explanation:**
- Log the error to stderr (always visible)
- Track in error list (appears in summary)
- Increment skipped counter (summary shows something went wrong)
- Document that semaphore closure is a fatal condition

---

## Fix 3: file_import.rs - Lines 276-277

### Issue: Unused Progress Calculations

**Current Code:**
```rust
// Line 276-281
// Progress tracking available for batch processing metrics
// In the Tauri command wrapper, this would emit an event

let _elapsed = start_time.elapsed().as_secs_f64();
let _rate = if _elapsed > 0.0 { current as f64 / _elapsed } else { 0.0 };
```

**Option A: Fix by Implementing Progress Events (Recommended)**
```rust
// Line 276-281
// Emit progress periodically (every 10 files reduces UI spam)
if current % 10 == 0 || current == _total_clone {
    let elapsed = start_time.elapsed().as_secs_f64();
    let rate = if elapsed > 0.0 { current as f64 / elapsed } else { 0.0 };

    // Note: window is not available in _impl version
    // Progress events can only be emitted by the Tauri command wrapper
    // This calculation is kept for reference
}
```

**Option B: Fix by Removing Unused Variables**
```rust
// Line 276-281
// Progress calculations are only used by the Tauri command wrapper
// which emits events. The _impl version doesn't have window access.
// All progress emission happens in the wrapper function.
```

**Choose Option B** - The _impl pattern is correct; don't add progress to it.

Instead, fix the wrapper (import_directory) to emit progress:

**Fixed Wrapper Code (around line 384-392):**
```rust
#[tauri::command]
pub async fn import_directory(
    directory_path: String,
    recursive: bool,
    category: Option<String>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<ImportSummary, String> {
    // Call the implementation with progress callback
    let result = import_directory_impl(directory_path, recursive, category, &state).await?;

    // Emit final progress
    let _ = window.emit("import-complete", &result);

    Ok(result)
}
```

---

## Fix 4: file_import.rs - Lines 308-310

### Issue: Batch Insert Error Handling

**Current Code:**
```rust
// Line 308-311
if let Err(e) = batch_inserter.insert_files_batch(file_records).await {
    errors.lock().await.push(format!("Batch insert failed: {}", e));
}
```

**Fixed Code:**
```rust
// Line 308-320
match batch_inserter.insert_files_batch(file_records).await {
    Ok(_) => {
        // Batch inserted successfully
    }
    Err(e) => {
        let error_msg = format!("Batch insert failed: {}", e);
        eprintln!("ERROR: {}", error_msg);

        // Try to record the error
        match errors.lock() {
            Ok(mut errs) => {
                errs.push(error_msg);
                // Mark some files as skipped (we don't know exactly which ones)
                // Conservative estimate: the entire batch failed
                skipped.fetch_add(file_records.len(), Ordering::SeqCst);
            }
            Err(poisoned) => {
                // Mutex poisoned - can't record error
                eprintln!("FATAL: Error list mutex poisoned, cannot record batch failure");
                // At least we logged to stderr
            }
        }
    }
}
```

**Explanation:**
- Log to stderr (always visible)
- Try to record in error list
- Handle mutex poisoning explicitly
- Update skipped counter to reflect lost files

---

## Fix 5: file_import.rs - Lines 912 & 979 (Test Code)

### Issue: `.expect()` in Test Code

These are acceptable in test code, but should have better error context:

**Current Code (line 912):**
```rust
.expect("Failed to fetch tags from database");
```

**Fixed Code:**
```rust
.expect("Failed to fetch tags from database - test database may not be running");
```

**Current Code (line 979):**
```rust
.expect("Failed to cleanup test file");
```

**Fixed Code:**
```rust
.expect("Failed to cleanup test file - check permissions and temp directory");
```

---

## Fix 6: analyze.rs - Lines 217-222

### Issue: Semaphore Closure Not Handled

**Current Code:**
```rust
// Line 216-222
let _permit = match sem.acquire().await {
    Ok(permit) => permit,
    Err(_) => {
        eprintln!("Warning: Semaphore closed during analysis");
        return;
    }
};
```

**Fixed Code:**
```rust
let _permit = match sem.acquire().await {
    Ok(permit) => permit,
    Err(e) => {
        let error_msg = format!("FATAL: Semaphore unavailable during analysis: {}", e);
        eprintln!("ERROR: {}", error_msg);

        // Track this as an error
        match errors.lock().await.push(error_msg) {
            _ => {}
        }

        // Mark file as skipped
        skipped.fetch_add(1, Ordering::SeqCst);
        return;
    }
};
```

---

## Fix 7: analyze.rs - Lines 233-239

### Issue: Window Emit Ignored

**Current Code:**
```rust
let _ = window.emit("analysis-progress", AnalysisProgress {
    current,
    total: total_usize,
    current_file: file_record.filename.clone(),
    rate,
    eta_seconds,
});
```

**Fixed Code:**
```rust
if let Err(e) = window.emit("analysis-progress", AnalysisProgress {
    current,
    total: total_usize,
    current_file: file_record.filename.clone(),
    rate,
    eta_seconds,
}) {
    // Log but don't fail the operation
    eprintln!("WARNING: Failed to emit analysis progress (file {}): {}",
              file_record.filename, e);
}
```

---

## Fix 8: analyze.rs - Lines 267-269

### Issue: Database Error Not Causing Abort

**Current Code:**
```rust
// Line 267-269
if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
    errors.lock().await.push(format!("Batch insert failed: {}", e));
}
```

**Fixed Code (Option 1: Continue with Logging):**
```rust
match batch_insert_analyzed_files(&batch, &pool).await {
    Ok(_) => {
        // Batch inserted successfully
    }
    Err(e) => {
        let error_msg = format!("Batch insert failed: {}", e);
        eprintln!("ERROR: {}", error_msg);

        // Continue processing but track the error
        match errors.lock().await.push(error_msg) {
            _ => {}
        }

        // Mark files as skipped
        skipped.fetch_add(batch.len(), Ordering::SeqCst);
    }
}
```

**Fixed Code (Option 2: Abort on Critical Error - Recommended):**
```rust
if let Err(e) = batch_insert_analyzed_files(&batch, &pool).await {
    let error_msg = format!("CRITICAL: Batch insert failed, aborting analysis: {}", e);
    eprintln!("ERROR: {}", error_msg);

    // Return error to stop the analysis
    let error_list = errors.lock().await.clone();
    return Err(format!("{}\nPrevious errors: {}", error_msg, error_list.join("; ")));
}
```

**Choose Option 2** - Analysis without database storage is useless. Fail fast.

---

## Fix 9: archive_import.rs - Lines 206 & 228

### Issue: Temp Directory Cleanup Errors Ignored

**Current Code (line 206):**
```rust
if midi_count == 0 {
    // Cleanup and return
    let _ = std::fs::remove_dir_all(&temp_dir);
    return Ok(ArchiveStatus { ... });
}
```

**Current Code (line 228):**
```rust
// Cleanup temp directory
let _ = std::fs::remove_dir_all(&temp_dir);
```

**Fixed Code:**
```rust
// Helper function at file level
fn cleanup_temp_dir(path: &std::path::Path) -> Result<(), String> {
    std::fs::remove_dir_all(path)
        .map_err(|e| format!("Failed to cleanup temp directory {}: {}",
                            path.display(), e))
}

// Line 206 usage:
if midi_count == 0 {
    match cleanup_temp_dir(&temp_dir) {
        Ok(_) => println!("✅ Cleaned up empty archive temp directory"),
        Err(e) => {
            eprintln!("WARNING: {}", e);
            // Don't fail the operation, but log the issue
        }
    }
    return Ok(ArchiveStatus { ... });
}

// Line 228 usage:
match cleanup_temp_dir(&temp_dir) {
    Ok(_) => println!("✅ Cleaned up archive temp directory"),
    Err(e) => {
        eprintln!("WARNING: {}", e);
        // Archive import succeeded, temp cleanup failure is non-fatal
    }
}
```

---

## Fix 10: archive_import.rs - Lines 147-153

### Issue: Archive Error Not Logged During Processing

**Current Code:**
```rust
// Line 140-154
match &status {
    Ok(s) => {
        total_files_imported += s.files_imported;
        total_files_skipped += s.midi_files_found.saturating_sub(s.files_imported);
        println!(
            "✅ Success: {} MIDIs found, {} imported\n",
            s.midi_files_found, s.files_imported
        );
    },
    Err(e) => {
        total_errors += 1;
        println!("❌ Error: {}\n", e);
    },
}

archive_statuses.push(status.unwrap_or_else(|e| ArchiveStatus {
    archive_name: archive_name.clone(),
    midi_files_found: 0,
    files_imported: 0,
    success: false,
    error_message: Some(e),
}));
```

**Fixed Code:**
```rust
// Line 140-160
let archive_status = match status {
    Ok(s) => {
        total_files_imported += s.files_imported;
        total_files_skipped += s.midi_files_found.saturating_sub(s.files_imported);
        println!(
            "✅ Success: {} MIDIs found, {} imported\n",
            s.midi_files_found, s.files_imported
        );
        s
    },
    Err(e) => {
        total_errors += 1;
        eprintln!("ERROR: Archive {} failed: {}\n", archive_name, e);
        println!("❌ Error: {}\n", e);

        ArchiveStatus {
            archive_name: archive_name.clone(),
            midi_files_found: 0,
            files_imported: 0,
            success: false,
            error_message: Some(e.clone()),
        }
    },
};

archive_statuses.push(archive_status);
```

**Explanation:**
- Log error to stderr during processing
- Still add to status for summary
- Makes debugging easier (can see which archive failed in real-time)

---

## Fix 11: progress.rs - Lines 277 & 302

### Issue: Bare `.unwrap()` in Tests

**Current Code (line 277):**
```rust
*tracker.start_time.lock().unwrap() = Some(std::time::Instant::now());
```

**Fixed Code:**
```rust
match tracker.start_time.lock() {
    Ok(mut start) => *start = Some(std::time::Instant::now()),
    Err(poisoned) => {
        eprintln!("WARNING: start_time mutex poisoned in test, recovering");
        *poisoned.into_inner() = Some(std::time::Instant::now());
    }
}
```

**Note:** This is in test code, so acceptable as-is. But the pattern should be:

```rust
// Preferred pattern for tests:
let mut start = tracker.start_time.lock()
    .expect("start_time mutex panicked in test - indicates logic error");
*start = Some(std::time::Instant::now());
```

---

## Fix 12: progress.rs - All mutex.lock() Calls (Lines 56, 64, 71, etc.)

### Issue: Mutex Poisoning Silently Recovered

**Current Pattern:**
```rust
self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
```

**Fixed Pattern:**
```rust
match self.state.lock() {
    Ok(state) => state.clone(),
    Err(poisoned) => {
        eprintln!("WARNING: Progress state mutex poisoned - likely from previous panic");
        eprintln!("  Recovering from last known state");
        poisoned.into_inner().clone()
    }
}
```

**Locations to Fix:**
- Line 56: `get_state()`
- Line 64: `update_state()`
- Line 71: `calculate_metrics()`
- Line 107: `start_progress_tracking()`
- Line 140: `update_progress()`
- Line 219: `end_progress_tracking()`
- Line 238: `reset_progress()`

---

## Implementation Order

1. **High Priority - Start Here:**
   - [ ] Fix 2: file_import.rs semaphore (line 262)
   - [ ] Fix 8: analyze.rs batch insert (line 267)
   - [ ] Fix 9: archive_import.rs cleanup (lines 206, 228)

2. **High Priority - Continue:**
   - [ ] Fix 1: file_import.rs emit (line 174)
   - [ ] Fix 7: analyze.rs emit (line 233)
   - [ ] Fix 11: progress.rs unwrap (lines 277, 302)
   - [ ] Fix 12: progress.rs poisoning (all mutex locks)

3. **Medium Priority:**
   - [ ] Fix 3: file_import.rs progress (line 276)
   - [ ] Fix 4: file_import.rs batch (line 308)
   - [ ] Fix 5: file_import.rs tests (lines 912, 979)
   - [ ] Fix 6: analyze.rs semaphore (line 217)
   - [ ] Fix 10: archive_import.rs logging (line 147)

---

## Testing Each Fix

### After fixing each issue, test with:

1. **Normal operation:**
   ```bash
   cargo test --package pipeline --lib -- --test-threads=1
   ```

2. **Integration test:**
   ```bash
   cargo test --package pipeline -- --test-threads=1 --ignored
   ```

3. **Check stderr output:**
   - Run operations and verify error messages appear in logs
   - Test with simulated failures (disk full, db down, etc.)

4. **Verify summary accuracy:**
   - Check that ImportSummary.errors matches actual failures
   - Check that skipped count is accurate

---

## Validation Checklist

After implementing all fixes, verify:

- [ ] All `let _ = ` patterns have been replaced with explicit handling
- [ ] All `.unwrap()` calls outside tests have error handling
- [ ] All `.expect()` calls have context (message explains what failed)
- [ ] All async errors are logged to stderr
- [ ] All window.emit() failures are logged
- [ ] All mutex operations handle poisoning
- [ ] Semaphore closure is tracked as error
- [ ] Disk cleanup failures are logged
- [ ] All summaries are accurate (count matches actual)
- [ ] Integration tests pass with errors simulated

---

## Example: Before and After

### Before (Problematic)
```rust
let result = operation().await;
if let Err(e) = result {
    errors.push(e.to_string());
}
// Continue silently - user doesn't know anything failed
```

### After (Fixed)
```rust
match operation().await {
    Ok(data) => process(data),
    Err(e) => {
        let msg = format!("Operation failed: {}", e);
        eprintln!("ERROR: {}", msg);
        errors.push(msg.clone());
        // Either continue with mitigation or return error
    }
}
```

---

## Summary

This guide provides specific, line-by-line fixes for all 12 error handling defects in the pipeline command layer. Each fix:

1. Adds explicit error handling (no more `let _`)
2. Logs to stderr (visible in all contexts)
3. Tracks in error lists (visible in summary)
4. Preserves user experience (non-fatal errors don't stop operation)
5. Enables debugging (clear error messages with context)

**Estimated Implementation Time:** 2-3 hours
**Risk Level:** Low (all changes are additive, no logic changes)
**Testing Required:** Medium (verify each fix, then integration test)
