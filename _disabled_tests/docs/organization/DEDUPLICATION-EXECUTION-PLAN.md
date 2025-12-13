# MIDI Library Deduplication Execution Plan

**Date:** November 21, 2025
**Status:** READY FOR EXECUTION

---

## Summary

**Problem:** Massive file duplication (83.4%) wasting 12.68 GB of storage

**Current State:**
- Database: 1,172,161 unique files (all with blake3 hashes)
- Filesystem: 6,746,098 MIDI files (71 GB total)
- Duplicates: 7,760,687 files (83.4% of scanned files)
- Unique files: 1,541,576 (16.6%)
- Wasted space: 12.68 GB

**Goal:** Remove 7.7M duplicate files, keep 1.5M unique files, import all to database

---

## Phase 1: Safety Checks (30 minutes)

### 1.1 Verify Delete List

```bash
# Check first 10 files in delete list
head -10 DUPLICATE_REPORT.delete.txt

# Check last 10 files
tail -10 DUPLICATE_REPORT.delete.txt

# Verify files exist
head -100 DUPLICATE_REPORT.delete.txt | while read f; do
    if [ ! -f "$f" ]; then
        echo "Missing: $f"
    fi
done
```

### 1.2 Create Backup Manifest

```bash
# Save current state
find /home/dojevou/projects/midi-software-center/midi-library -type f -name "*.mid" -o -name "*.MID" -o -name "*.midi" -o -name "*.MIDI" > /tmp/midi_files_before_dedup.txt

wc -l /tmp/midi_files_before_dedup.txt
# Expected: 6,746,098 files
```

### 1.3 Database Backup

```bash
# Backup database before major changes
pg_dump -h localhost -p 5433 -U midiuser -d midi_library -F c -f /tmp/midi_library_backup_$(date +%Y%m%d_%H%M%S).dump
```

---

## Phase 2: Deduplication Execution (2-3 hours)

### 2.1 Dry Run Test (first 1000 files)

```bash
# Test with first 1000 files
head -1000 DUPLICATE_REPORT.delete.txt > /tmp/test_delete.txt

# Count files
wc -l /tmp/test_delete.txt

# Dry run: check what would be deleted
cat /tmp/test_delete.txt | while read f; do
    if [ -f "$f" ]; then
        echo "Would delete: $f"
    else
        echo "Missing: $f"
    fi
done | head -20
```

### 2.2 Test Deletion (first 1000)

```bash
# Actually delete test batch
cat /tmp/test_delete.txt | while read f; do
    if [ -f "$f" ]; then
        rm "$f" && echo "Deleted: $(basename "$f")"
    fi
done

# Verify test deletions
find /home/dojevou/projects/midi-software-center/midi-library -type f -name "*.mid" -o -name "*.MID" | wc -l
# Expected: 6,746,098 - 1,000 = 6,745,098
```

### 2.3 Full Deletion (7.7M files) - CAREFUL!

**⚠️ DANGER ZONE - NO UNDO AFTER THIS**

```bash
# Option A: Direct deletion (faster, ~30-60 minutes)
cat DUPLICATE_REPORT.delete.txt | while read f; do
    [ -f "$f" ] && rm "$f"
done

# Option B: Parallel deletion (fastest, ~15-30 minutes)
cat DUPLICATE_REPORT.delete.txt | parallel -j 8 'rm {} 2>/dev/null'

# Option C: With progress monitoring (slower, ~60-90 minutes)
total=$(wc -l < DUPLICATE_REPORT.delete.txt)
count=0
cat DUPLICATE_REPORT.delete.txt | while read f; do
    [ -f "$f" ] && rm "$f"
    ((count++))
    if [ $((count % 10000)) -eq 0 ]; then
        echo "Progress: $count / $total ($(echo "scale=2; $count * 100 / $total" | bc)%)"
    fi
done
```

### 2.4 Clean Empty Directories

```bash
# Find and remove empty directories after deletion
find /home/dojevou/projects/midi-software-center/midi-library -type d -empty -delete

# Count removed
echo "Empty directories cleaned"
```

---

## Phase 3: Verification (15 minutes)

### 3.1 Count Remaining Files

```bash
# Count MIDI files after deduplication
find /home/dojevou/projects/midi-software-center/midi-library -type f \( -name "*.mid" -o -name "*.MID" -o -name "*.midi" -o -name "*.MIDI" \) | wc -l

# Expected: ~1,541,576 unique files (or ~985,411 if some were already in DB)
```

### 3.2 Check Disk Space Freed

```bash
# Check new size
du -sh /home/dojevou/projects/midi-software-center/midi-library/

# Expected: ~58 GB (71 GB - 12.68 GB)
```

### 3.3 Verify No Duplicates Remain

```bash
# Re-scan for duplicates (optional, slow)
cd pipeline/src-tauri
cargo run --bin find_duplicates -- /home/dojevou/projects/midi-software-center/midi-library

# Expected: 0 duplicates
```

---

## Phase 4: Database Import (2-4 hours)

### 4.1 Import Remaining Files

```bash
# Import all unique files not yet in database
cd pipeline/src-tauri
cargo run --release --bin batch_import -- \
  --source /home/dojevou/projects/midi-software-center/midi-library \
  --workers 16 \
  --batch-size 1000

# Expected: Import ~369,415 new files (1,541,576 - 1,172,161)
```

### 4.2 Verify Database Count

```bash
# Check final database count
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c \
  "SELECT COUNT(*) as total, COUNT(DISTINCT content_hash) as unique FROM files;"

# Expected: ~1,541,576 total, all unique hashes
```

---

## Phase 5: Reorganization (1-2 hours)

### 5.1 Consolidate Files (Optional)

After deduplication, files are scattered across archives/, extracted/, splits/ directories.

**Option A: Keep As-Is** (Recommended for now)
- Files stay where they are
- Database tracks all locations
- Easier to roll back if issues

**Option B: Consolidate to files/** (Future work)
- Move all unique files to `midi-library/files/`
- Organize alphabetically (a/, b/, c/, ...)
- Update database filepaths

### 5.2 Update Documentation

```bash
# Create completion report
cat > DEDUPLICATION-COMPLETE.md <<EOF
# Deduplication Complete

Date: $(date)

## Results
- Files before: 6,746,098
- Files after: [COUNT HERE]
- Duplicates removed: [COUNT HERE]
- Space freed: [SIZE HERE]
- Database files: [COUNT HERE]

## Status
✅ Deduplication complete
✅ Database updated
✅ No duplicates remaining
EOF
```

---

## Rollback Plan (If Needed)

### If something goes wrong during deletion:

1. **Stop immediately** (Ctrl+C)

2. **Restore from backup:**
```bash
# List available backups
ls -lh /tmp/midi_library_backup_*.dump

# Restore database
pg_restore -h localhost -p 5433 -U midiuser -d midi_library -c /tmp/midi_library_backup_TIMESTAMP.dump
```

3. **File recovery:** No file recovery possible after deletion (files are permanently removed)
   - This is why we do test deletions first
   - Ensure DUPLICATE_REPORT.delete.txt is correct

---

## Safety Checklist

Before executing Phase 2 (deletion):

- [ ] Database backup created
- [ ] File manifest saved (/tmp/midi_files_before_dedup.txt)
- [ ] Dry run test completed successfully
- [ ] Test deletion (1000 files) verified
- [ ] Confirmed DUPLICATE_REPORT.delete.txt contains correct files
- [ ] Disk space available for logs
- [ ] Ready to proceed (no undo after this!)

---

## Estimated Timeline

- Phase 1 (Safety): 30 minutes
- Phase 2 (Deletion): 30-60 minutes (parallel) or 60-90 minutes (with progress)
- Phase 3 (Verification): 15 minutes
- Phase 4 (Import): 2-4 hours
- Phase 5 (Reorganization): 1-2 hours (optional)

**Total: 4-6 hours**

---

## Next Steps After Completion

1. Run Phase 4 Analysis on all new files
2. Generate tags for all imported files
3. Update CLAUDE.md with new stats
4. Consider consolidating to database-centric organization (Option B)

---

**Status:** READY FOR EXECUTION
**Last Updated:** November 21, 2025
