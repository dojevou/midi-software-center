#!/usr/bin/env python3
"""
Restore Original Filenames
Reverts files back to their original database names after normalization.
Uses database as source of truth.
"""

import os
import sys
import psycopg2
from pathlib import Path
import re

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def sanitize_strict(filename):
    """
    Apply same sanitization as Rust normalize_filenames.rs
    This tells us what the normalized filename would be.
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

def restore_files(limit=None, dry_run=True):
    """
    Restore files to original database names.

    Args:
        limit: Number of files to process (None = all)
        dry_run: If True, only print what would be done
    """
    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Get files where normalized name differs from database name
    query = """
        SELECT id, filename, filepath
        FROM files
        ORDER BY id
    """
    if limit:
        query += f" LIMIT {limit}"

    cur.execute(query)

    restored = 0
    errors = 0
    skipped = 0
    not_found = 0

    print(f"\n{'='*70}")
    print(f"RESTORING ORIGINAL FILENAMES FROM DATABASE")
    print(f"{'='*70}")
    print(f"Mode: {'DRY RUN' if dry_run else 'LIVE MODE'}")
    print()

    for file_id, db_filename, db_filepath in cur:
        # Calculate what the normalized filename would be
        normalized_filename = sanitize_strict(db_filename)

        # If they're the same, skip
        if normalized_filename == db_filename:
            skipped += 1
            continue

        # Get directory path
        db_path = Path(db_filepath)
        parent_dir = db_path.parent

        # What the normalized file would be called
        normalized_path = parent_dir / normalized_filename

        # Check if normalized file exists
        if not normalized_path.exists():
            # File might not have been renamed, check if original exists
            if db_path.exists():
                skipped += 1
                continue
            else:
                not_found += 1
                if not_found <= 10:  # Only print first 10
                    print(f"❌ Not found: {normalized_path}")
                continue

        # Restore to original name
        if dry_run:
            print(f"Would restore: {normalized_filename} → {db_filename}")
        else:
            try:
                # Handle case where original filename already exists
                if db_path.exists() and db_path != normalized_path:
                    print(f"⚠️  Original exists: {db_filename}, skipping")
                    skipped += 1
                    continue

                os.rename(normalized_path, db_path)
                restored += 1

                if restored % 10000 == 0:
                    print(f"Restored {restored:,} files...")
            except Exception as e:
                errors += 1
                if errors <= 10:
                    print(f"❌ Error restoring {normalized_filename}: {e}")

    cur.close()
    conn.close()

    print(f"\n{'='*70}")
    print(f"RESTORATION {'SIMULATION' if dry_run else 'COMPLETE'}")
    print(f"{'='*70}")
    print(f"Files restored:    {restored:,}")
    print(f"Files skipped:     {skipped:,}")
    print(f"Files not found:   {not_found:,}")
    print(f"Errors:            {errors:,}")
    print()

    return restored, skipped, not_found, errors

if __name__ == "__main__":
    # Test on small batch first
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        print("Testing on 100 files (dry run)...")
        restore_files(limit=100, dry_run=True)
    elif len(sys.argv) > 1 and sys.argv[1] == "--run":
        print("Running full restoration (LIVE MODE)...")
        response = input("Are you sure? This will rename ~650K files. Type 'yes' to confirm: ")
        if response.lower() == 'yes':
            restore_files(limit=None, dry_run=False)
        else:
            print("Cancelled.")
    else:
        print("Usage:")
        print("  --test    Test on 100 files (dry run)")
        print("  --run     Run full restoration (LIVE MODE)")
