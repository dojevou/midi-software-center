#!/usr/bin/env python3
"""
Test normalization on a sample of files that actually need it
"""

import os
import sys

# Add the scripts directory to Python path
scripts_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, scripts_dir)

# Try to import the normalization module, with helpful error if it fails
try:
    from normalize_files_and_database import normalize_files_and_database
except ImportError as e:
    # Module might be named with hyphens on disk
    print(f"Note: Could not import normalize_files_and_database: {e}")
    print("This script may work without it - using inline implementation.")
    normalize_files_and_database = None

import psycopg2
import tempfile

DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def test_on_files_needing_normalization(count=100, dry_run=True):
    """Test on files that actually need normalization"""

    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Create temp table with files that need normalization
    cur.execute("""
        CREATE TEMP TABLE test_files AS
        SELECT id
        FROM files
        WHERE filename LIKE '% %'
           OR filename ~ '\\.MID$'
           OR filename ~ '\\.MIDI$'
           OR filename ~ '\\.Mid$'
        LIMIT %s
    """, (count,))

    # Get IDs for testing
    cur.execute("SELECT id FROM test_files")
    test_ids = [row[0] for row in cur.fetchall()]

    print(f"Testing on {len(test_ids)} files that need normalization")
    print()

    cur.close()
    conn.close()

    # Now run normalization on just these files
    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Process each test file
    from pathlib import Path
    import re
    from collections import defaultdict

    def sanitize_strict(filename):
        result = filename
        result = result.replace(' ', '_')
        result = re.sub(r'[^a-zA-Z0-9_\-.]+', '', result)
        result = re.sub(r'\.(mid|MID|MIDI|midi|Mid|MiD)$', '.mid', result, flags=re.IGNORECASE)
        result = re.sub(r'_+', '_', result)
        result = re.sub(r'-+', '-', result)
        result = re.sub(r'^[_-]+', '', result)
        result = re.sub(r'[_-]+$', '', result)
        return result

    stats = {'normalized': 0, 'skipped': 0, 'errors': 0}
    dir_filenames = defaultdict(set)

    cur.execute("""
        SELECT id, filename, filepath
        FROM files
        WHERE id = ANY(%s)
        ORDER BY id
    """, (test_ids,))

    print(f"{'='*70}")
    print(f"NORMALIZATION TEST - {count} FILES")
    print(f"Mode: {'DRY RUN' if dry_run else 'LIVE MODE'}")
    print(f"{'='*70}\n")

    for idx, (file_id, db_filename, db_filepath) in enumerate(cur.fetchall(), 1):
        normalized_filename = sanitize_strict(db_filename)

        if normalized_filename == db_filename:
            stats['skipped'] += 1
            continue

        db_path = Path(db_filepath)
        parent_dir = db_path.parent
        new_path = parent_dir / normalized_filename

        # Handle duplicates
        final_filename = normalized_filename
        final_path = new_path
        counter = 1

        while (final_path.exists() and final_path != db_path) or \
              final_filename in dir_filenames[str(parent_dir)]:
            stem = Path(normalized_filename).stem
            ext = Path(normalized_filename).suffix
            final_filename = f"{stem}_{counter}{ext}"
            final_path = parent_dir / final_filename
            counter += 1

        dir_filenames[str(parent_dir)].add(final_filename)

        if idx <= 20:  # Print first 20
            print(f"{idx}. {db_filename}")
            print(f"   → {final_filename}")
            if counter > 1:
                print(f"   (duplicate, added _{counter-1})")

        if not dry_run:
            try:
                if not db_path.exists():
                    print(f"   ❌ File not found: {db_path}")
                    stats['errors'] += 1
                    continue

                # Rename file
                os.rename(db_path, final_path)

                # Update database
                cur.execute("""
                    UPDATE files
                    SET filename = %s, filepath = %s, updated_at = NOW()
                    WHERE id = %s
                """, (final_filename, str(final_path), file_id))

                stats['normalized'] += 1
            except Exception as e:
                print(f"   ❌ Error: {e}")
                stats['errors'] += 1
                conn.rollback()
        else:
            stats['normalized'] += 1

    if not dry_run:
        conn.commit()

    cur.close()
    conn.close()

    print(f"\n{'='*70}")
    print(f"TEST {'SIMULATION' if dry_run else 'COMPLETE'}")
    print(f"{'='*70}")
    print(f"Files normalized: {stats['normalized']}")
    print(f"Files skipped:    {stats['skipped']}")
    print(f"Errors:           {stats['errors']}")
    print()

if __name__ == "__main__":
    if "--live" in sys.argv:
        response = input("Run on 100 files in LIVE MODE? Type 'yes': ")
        if response.lower() == 'yes':
            test_on_files_needing_normalization(100, dry_run=False)
    else:
        test_on_files_needing_normalization(100, dry_run=True)
