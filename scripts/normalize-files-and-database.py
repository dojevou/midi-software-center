#!/usr/bin/env python3
"""
Normalize Files AND Database Together
Keeps files and database in perfect sync during normalization.

Normalizes:
- Extensions: .MID, .MIDI, .midi → .mid
- Spaces: replaced with underscores
- Special chars: removed (keep only a-z, A-Z, 0-9, _, -, .)
"""

import os
import sys
import psycopg2
from pathlib import Path
import re
from collections import defaultdict

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def sanitize_strict(filename):
    """
    Apply same sanitization as Rust normalize_filenames.rs
    """
    result = filename

    # 1. Replace spaces with underscores
    result = result.replace(' ', '_')

    # 2. Remove special characters (keep only a-z, A-Z, 0-9, _, -, .)
    result = re.sub(r'[^a-zA-Z0-9_\-.]+', '', result)

    # 3. Normalize extension to lowercase .mid
    result = re.sub(r'\.(mid|MID|MIDI|midi|Mid|MiD)$', '.mid', result, flags=re.IGNORECASE)

    # 4. Remove consecutive underscores/dashes
    result = re.sub(r'_+', '_', result)
    result = re.sub(r'-+', '-', result)

    # 5. Remove leading/trailing underscores/dashes
    result = re.sub(r'^[_-]+', '', result)
    result = re.sub(r'[_-]+$', '', result)

    return result

def normalize_files_and_database(limit=None, dry_run=True, commit_every=1000):
    """
    Normalize files on disk AND database records together.

    Args:
        limit: Number of files to process (None = all)
        dry_run: If True, only simulate
        commit_every: Commit database transaction every N files
    """
    conn = psycopg2.connect(DB_URL)

    # Track duplicates per directory
    dir_filenames = defaultdict(set)

    stats = {
        'processed': 0,
        'normalized': 0,
        'skipped': 0,
        'errors': 0,
        'db_updated': 0,
        'disk_renamed': 0,
    }

    print(f"\n{'='*70}")
    print(f"NORMALIZE FILES AND DATABASE TOGETHER")
    print(f"{'='*70}")
    print(f"Mode: {'DRY RUN' if dry_run else 'LIVE MODE'}")
    print(f"Commit every: {commit_every} files")
    print()

    try:
        cur = conn.cursor()

        # Get all files (or limited batch)
        query = "SELECT id, filename, filepath FROM files ORDER BY id"
        if limit:
            query += f" LIMIT {limit}"

        cur.execute(query)
        all_files = cur.fetchall()
        total = len(all_files)

        print(f"Found {total:,} files to process")
        print()

        for idx, (file_id, db_filename, db_filepath) in enumerate(all_files, 1):
            try:
                # Calculate normalized filename
                normalized_filename = sanitize_strict(db_filename)

                # Skip if no change needed
                if normalized_filename == db_filename:
                    stats['skipped'] += 1
                    stats['processed'] += 1
                    continue

                # Get paths
                db_path = Path(db_filepath)
                parent_dir = db_path.parent
                new_path = parent_dir / normalized_filename

                # Handle duplicates by appending counter
                final_filename = normalized_filename
                final_path = new_path
                counter = 1

                # Check both disk and our tracking dict for duplicates
                while (final_path.exists() and final_path != db_path) or \
                      final_filename in dir_filenames[str(parent_dir)]:
                    stem = Path(normalized_filename).stem
                    ext = Path(normalized_filename).suffix
                    final_filename = f"{stem}_{counter}{ext}"
                    final_path = parent_dir / final_filename
                    counter += 1

                # Mark this filename as used in this directory
                dir_filenames[str(parent_dir)].add(final_filename)

                if dry_run:
                    if idx <= 20:  # Only print first 20 in dry run
                        print(f"Would normalize:")
                        print(f"  {db_filename} → {final_filename}")
                        if counter > 1:
                            print(f"  (duplicate handled with _{counter-1})")
                    stats['normalized'] += 1
                else:
                    # STEP 1: Rename file on disk
                    if not db_path.exists():
                        print(f"⚠️  File not found on disk: {db_path}")
                        stats['errors'] += 1
                        stats['processed'] += 1
                        continue

                    os.rename(db_path, final_path)
                    stats['disk_renamed'] += 1

                    # STEP 2: Update database
                    new_filepath = str(final_path)

                    update_query = """
                        UPDATE files
                        SET
                            filename = %s,
                            filepath = %s,
                            updated_at = NOW()
                        WHERE id = %s
                    """
                    cur.execute(update_query, (final_filename, new_filepath, file_id))
                    stats['db_updated'] += 1
                    stats['normalized'] += 1

                    # Commit periodically
                    if stats['normalized'] % commit_every == 0:
                        conn.commit()
                        print(f"Normalized {stats['normalized']:,} / {total:,} files ({stats['normalized']/total*100:.1f}%)")

                stats['processed'] += 1

            except Exception as e:
                print(f"❌ Error processing file {file_id}: {e}")
                stats['errors'] += 1
                if not dry_run:
                    conn.rollback()
                stats['processed'] += 1

        # Final commit
        if not dry_run:
            conn.commit()
            print(f"Normalized {stats['normalized']:,} / {total:,} files (100%)")

        cur.close()

    finally:
        conn.close()

    print(f"\n{'='*70}")
    print(f"NORMALIZATION {'SIMULATION' if dry_run else 'COMPLETE'}")
    print(f"{'='*70}")
    print(f"Files processed:     {stats['processed']:,}")
    print(f"Files normalized:    {stats['normalized']:,}")
    print(f"Files skipped:       {stats['skipped']:,}")
    print(f"Disk renames:        {stats['disk_renamed']:,}")
    print(f"Database updates:    {stats['db_updated']:,}")
    print(f"Errors:              {stats['errors']:,}")
    print()

    return stats

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        print("Testing on 100 files (dry run)...")
        normalize_files_and_database(limit=100, dry_run=True)
    elif len(sys.argv) > 1 and sys.argv[1] == "--test-live":
        print("Testing on 100 files (LIVE MODE)...")
        response = input("This will rename 100 files. Type 'yes' to confirm: ")
        if response.lower() == 'yes':
            normalize_files_and_database(limit=100, dry_run=False)
        else:
            print("Cancelled.")
    elif len(sys.argv) > 1 and sys.argv[1] == "--run":
        print("Running full normalization (LIVE MODE)...")
        response = input("This will rename ~650K files AND update database. Type 'yes' to confirm: ")
        if response.lower() == 'yes':
            normalize_files_and_database(limit=None, dry_run=False, commit_every=5000)
        else:
            print("Cancelled.")
    else:
        print("Usage:")
        print("  --test        Test on 100 files (dry run)")
        print("  --test-live   Test on 100 files (LIVE MODE)")
        print("  --run         Run full normalization (LIVE MODE)")
