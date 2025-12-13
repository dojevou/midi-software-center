#!/usr/bin/env python3
"""
Strict MIDI filename sanitizer
Rules:
- Only allowed: letters, numbers, underscores, hyphens, periods
- No consecutive special characters (__, --, .., -_, _., etc.)
- No spaces
"""

import os
import re
import sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from collections import defaultdict

def sanitize_filename_strict(filename):
    """
    Apply strict sanitization rules to filename.

    Rules:
    1. Keep only: a-z, A-Z, 0-9, _, -, .
    2. Replace all other chars with underscore
    3. Collapse consecutive special chars to single char
    4. Preserve .mid extension
    """
    # Split into name and extension
    if filename.endswith('.mid'):
        name = filename[:-4]
        ext = '.mid'
    elif filename.endswith('.midi'):
        name = filename[:-5]
        ext = '.mid'
    else:
        name = filename
        ext = ''

    # Step 1: Replace all non-allowed characters with underscore
    # Allowed: a-z A-Z 0-9 _ - .
    name = re.sub(r'[^a-zA-Z0-9_.\-]', '_', name)

    # Step 2: Collapse consecutive special characters
    # Replace __, --, .., or any mix of consecutive special chars with single underscore
    name = re.sub(r'[_.\-]{2,}', '_', name)

    # Step 3: Remove leading/trailing underscores
    name = name.strip('_')

    # If name is empty after sanitization, use default
    if not name:
        name = 'unnamed'

    return name + ext


def sanitize_file(file_path, dry_run=False):
    """Sanitize a single file."""
    try:
        parent = file_path.parent
        old_name = file_path.name
        new_name = sanitize_filename_strict(old_name)

        if old_name == new_name:
            return None, None  # No change needed

        new_path = parent / new_name

        # Handle conflicts by appending counter
        if new_path.exists() and new_path != file_path:
            base = new_name[:-4] if new_name.endswith('.mid') else new_name
            ext = '.mid' if new_name.endswith('.mid') else ''
            counter = 1
            while new_path.exists():
                new_name = f"{base}_{counter}{ext}"
                new_path = parent / new_name
                counter += 1

        if dry_run:
            return old_name, new_name

        # Rename the file
        file_path.rename(new_path)
        return old_name, new_name

    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return None, None


def find_midi_files(directory):
    """Find all MIDI files recursively."""
    midi_files = []
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.lower().endswith(('.mid', '.midi')):
                midi_files.append(Path(root) / file)
    return midi_files


def main():
    if len(sys.argv) < 2:
        print("Usage: ./strict-sanitize-filenames.py <directory> [--dry-run] [workers]")
        print("Example: ./strict-sanitize-filenames.py /path/to/midi --dry-run 64")
        sys.exit(1)

    directory = Path(sys.argv[1])
    dry_run = '--dry-run' in sys.argv
    workers = 64

    # Check for worker count
    for arg in sys.argv[2:]:
        if arg.isdigit():
            workers = int(arg)

    if not directory.exists():
        print(f"Error: Directory {directory} does not exist")
        sys.exit(1)

    print("🧹 STRICT MIDI FILENAME SANITIZATION")
    print("═" * 80)
    print(f"📂 Directory: {directory}")
    print(f"⚙️  Workers: {workers}")
    print(f"🔍 Mode: {'DRY RUN (preview only)' if dry_run else 'LIVE (will rename files)'}")
    print()

    # Find all MIDI files
    print("📊 Scanning for MIDI files...")
    midi_files = find_midi_files(directory)
    print(f"✓ Found {len(midi_files):,} MIDI files")
    print()

    if dry_run:
        print("🔍 DRY RUN - Preview of changes:")
        print("-" * 80)

    # Process files in parallel
    renamed_count = 0
    errors = 0
    changes = []

    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(sanitize_file, f, dry_run): f for f in midi_files}

        for i, future in enumerate(as_completed(futures), 1):
            try:
                old_name, new_name = future.result()
                if old_name and new_name:
                    renamed_count += 1
                    changes.append((old_name, new_name))
                    if dry_run and renamed_count <= 50:  # Show first 50 in dry run
                        print(f"  {old_name}")
                        print(f"  → {new_name}")
                        print()

                # Progress indicator
                if i % 10000 == 0:
                    print(f"  Processed: {i:,} / {len(midi_files):,} ({i*100//len(midi_files)}%)")
            except Exception as e:
                errors += 1
                print(f"Error: {e}", file=sys.stderr)

    # Summary
    print()
    print("═" * 80)
    if dry_run:
        print("✅ DRY RUN COMPLETE")
        print(f"   Files that would be renamed: {renamed_count:,}")
        if renamed_count > 50:
            print(f"   (showing first 50 of {renamed_count:,} changes)")
        print()
        print("Run without --dry-run to apply changes:")
        print(f"  ./strict-sanitize-filenames.py {directory} {workers}")
    else:
        print("✅ SANITIZATION COMPLETE")
        print(f"   Files renamed: {renamed_count:,}")
        print(f"   Files unchanged: {len(midi_files) - renamed_count:,}")
        print(f"   Errors: {errors}")
    print("═" * 80)


if __name__ == "__main__":
    main()
