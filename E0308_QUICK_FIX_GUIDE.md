# E0308 Type Mismatch Errors - Quick Implementation Guide

## Copy-Paste Implementation Plan

### Step 1: Create Backup
```bash
cd /home/dojevou/projects/midi-software-center
cp -r pipeline/src-tauri/tests pipeline/src-tauri/tests.backup.$(date +%s)
echo "Backup created"
```

### Step 2: Fix Pattern 1 (52 errors - 5 minutes)
```bash
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests

# Show what will change
echo "=== PREVIEW OF CHANGES ==="
grep -n "^\s*state,$" file_import_test.rs | head -10

# Apply fix
sed -i 's/^\(\s*\)state,\s*$/\1\&state,/g' file_import_test.rs

# Verify
echo "=== VERIFICATION ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
echo "Expected: 40 (from 92)"
```

### Step 3: Fix Pattern 2 (6 errors - 2 minutes)
```bash
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests

# Preview
echo "=== PREVIEW OF CHANGES ==="
grep -n "cleanup_test_files(state\.database\.pool()" workflows_test.rs

# Apply fix
sed -i 's/cleanup_test_files(state\.database\.pool()/cleanup_test_files(\&state.database.pool()/g' workflows_test.rs

# Verify
echo "=== VERIFICATION ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
echo "Expected: 34 (from 40)"
```

### Step 4: Fix Pattern 3 (8 errors - 10 minutes)
```bash
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests

# Preview
echo "=== PREVIEW OF CHANGES ==="
grep -n '\.min_bpm(Some("' search_repository_test.rs | head -5
grep -n '\.max_bpm(Some("' search_repository_test.rs | head -5

# Apply fix
sed -i 's/\.min_bpm(Some("\([0-9.-]*\)"\.to_string()))/.min_bpm(Some(\1))/g' search_repository_test.rs
sed -i 's/\.max_bpm(Some("\([0-9.-]*\)"\.to_string()))/.max_bpm(Some(\1))/g' search_repository_test.rs

# Verify
echo "=== VERIFICATION ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
echo "Expected: 26 (from 34)"
```

### Step 5: Fix Pattern 4 (6 errors - 5 minutes)
```bash
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests

# Preview
echo "=== PREVIEW OF CHANGES ==="
grep -n "FileRepository::list(&pool, Some" file_repository_test.rs

# Apply generic fix
sed -i 's/FileRepository::list(&pool, Some(\([^,]*\)), Some(\([^)]*\)))/FileRepository::list(\&pool, \1, \2)/g' file_repository_test.rs

# Verify
echo "=== VERIFICATION ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
echo "Expected: 20 (from 26)"
```

### Step 6: Fix Pattern 5 (3 errors - 15 minutes MANUAL)
```bash
# Edit workflows_test.rs manually around these lines:
# Line 186: Change assertion from tags.contains to tags.iter().any
# Line 339: Change assertion from tags.contains to tags.iter().any

nano +186 /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/workflows_test.rs

# Search for: tags.contains(&"
# Replace with: tags.iter().any(|t| t.name ==

# Save and verify
echo "=== VERIFICATION ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
echo "Expected: 17 (from 20)"
```

### Step 7: Final Check
```bash
cd /home/dojevou/projects/midi-software-center

# Full compilation
echo "=== RUNNING FULL CHECK ==="
cargo check --tests -p midi-pipeline 2>&1 | tail -20

# Count remaining E0308 errors
echo "=== E0308 ERROR COUNT ==="
cargo check --tests -p midi-pipeline 2>&1 | grep -c "error\[E0308\]"
echo "Expected: 17 or less"
```

---

## Manual Fix for Pattern 5 (Tag Assertions)

### Option A: Using sed (risky, test carefully)
```bash
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests

# Backup first
cp workflows_test.rs workflows_test.rs.backup

# Replace pattern
sed -i 's/tags\.contains(&"\([^"]*\)"\.to_string())/tags.iter().any(|t| t.name == "\1")/g' workflows_test.rs

# Verify
grep -n "tags.iter().any" workflows_test.rs
```

### Option B: Manual edits (safest)

#### File: `workflows_test.rs`

**Line 186 - BEFORE:**
```rust
assert!(tags.contains(&"template".to_string()));
```

**Line 186 - AFTER:**
```rust
assert!(tags.iter().any(|t| t.name == "template"));
```

**Line 339 - BEFORE:**
```rust
assert!(tags_original.contains(&"original".to_string()));
```

**Line 339 - AFTER:**
```rust
assert!(tags_original.iter().any(|t| t.name == "original"));
```

---

## Verification at Each Step

```bash
# Step 1: Initial state
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
# Result: 92

# After Pattern 1 fix
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
# Result: 40

# After Pattern 2 fix
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
# Result: 34

# After Pattern 3 fix
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
# Result: 26

# After Pattern 4 fix
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
# Result: 20

# After Pattern 5 fix
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | wc -l
# Result: 17 (or less)
```

---

## Troubleshooting

### Sed command didn't work
```bash
# Check the actual line content
grep -n "state,$" file_import_test.rs | head -3

# Use more conservative sed (test on small range first)
sed -i '737,750s/^\(\s*\)state,\s*$/\1\&state,/g' file_import_test.rs

# Verify with cargo check
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0308\]" | head -5
```

### Backup needed
```bash
# Restore from backup if something went wrong
cd /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests
cp *.backup . 2>/dev/null || echo "No backups found"

# Or restore from git
git checkout file_import_test.rs workflows_test.rs file_repository_test.rs search_repository_test.rs
```

### Verify specific line
```bash
# Check specific lines to see actual content
sed -n '737p' /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/file_import_test.rs

# Check context
sed -n '735,740p' /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/file_import_test.rs
```

---

## Full Validation Script

```bash
#!/bin/bash
cd /home/dojevou/projects/midi-software-center

echo "=========================================="
echo "E0308 Type Mismatch Error Fix Validation"
echo "=========================================="

echo ""
echo "1. Checking E0308 error count..."
ERROR_COUNT=$(cargo check --tests -p midi-pipeline 2>&1 | grep -c "error\[E0308\]")
echo "   Result: $ERROR_COUNT E0308 errors found"

echo ""
echo "2. Running compilation check..."
if cargo check --tests -p midi-pipeline 2>&1 | grep -q "error:.*E0"; then
    echo "   WARNING: Still has errors"
    cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0" | head -10
else
    echo "   SUCCESS: No E0xxx errors found!"
fi

echo ""
echo "3. Building test binary..."
if cargo build --tests -p midi-pipeline 2>&1 | tail -5 | grep -q "Finished"; then
    echo "   SUCCESS: Test build successful"
else
    echo "   WARNING: Test build may have issues"
fi

echo ""
echo "=========================================="
echo "Status: $ERROR_COUNT E0308 errors remaining"
echo "=========================================="
```

---

## Time Estimate Summary

```
Pattern 1 (52 errors):  5 min  -> 40 remaining
Pattern 2 (6 errors):   2 min  -> 34 remaining
Pattern 3 (8 errors):   10 min -> 26 remaining
Pattern 4 (6 errors):   5 min  -> 20 remaining
Pattern 5 (3 errors):   15 min -> 17 remaining
Other (17 errors):      30 min -> 0 remaining
Validation:             10 min
───────────────────────────────────
TOTAL:                  77 min (1h 17m)
```

---

## Success Indicators

- [ ] `cargo check --tests -p midi-pipeline` shows 0 E0308 errors
- [ ] `cargo build --tests -p midi-pipeline` completes successfully
- [ ] `cargo test --workspace --lib` passes all tests
- [ ] No new E0425, E0061, or E0599 errors introduced

---

**Ready to implement. Follow steps 1-7 in order.**
