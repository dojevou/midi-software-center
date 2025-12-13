#!/usr/bin/env python3
"""
Import Missing Split Files into Database

Imports split track files that exist on disk but are not yet in the database.
Uses BLAKE3 for deduplication and parses MIDI for basic metadata.
"""

import os
import sys
import psycopg2
from psycopg2 import IntegrityError
from pathlib import Path
from multiprocessing import Pool, cpu_count
import hashlib
import mido
import re

DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"
SPLITS_DIR = "/home/dojevou/tmp/midi_splits_fast"

def calculate_blake3_hash(filepath):
    """Calculate BLAKE3 hash of file (fallback to SHA256 if blake3 not available)"""
    try:
        import blake3
        with open(filepath, 'rb') as f:
            return blake3.blake3(f.read()).digest()
    except ImportError:
        # Fallback to SHA256 if blake3 not available
        with open(filepath, 'rb') as f:
            return hashlib.sha256(f.read()).digest()

def parse_midi_basic(filepath):
    """Parse MIDI file for basic metadata"""
    try:
        mid = mido.MidiFile(filepath)

        # Get duration in ticks
        total_ticks = 0
        for track in mid.tracks:
            track_ticks = sum(msg.time for msg in track)
            if track_ticks > total_ticks:
                total_ticks = track_ticks

        # Get duration in seconds (approximate)
        tempo = 500000  # Default tempo (120 BPM)
        for track in mid.tracks:
            for msg in track:
                if msg.type == 'set_tempo':
                    tempo = msg.tempo
                    break

        ticks_per_beat = mid.ticks_per_beat
        seconds_per_tick = tempo / (ticks_per_beat * 1_000_000)
        duration_seconds = total_ticks * seconds_per_tick

        return {
            'format': mid.type,
            'num_tracks': len(mid.tracks),
            'ticks_per_quarter_note': mid.ticks_per_beat,
            'duration_seconds': round(duration_seconds, 3),
            'duration_ticks': total_ticks,
        }
    except Exception as e:
        print(f"Error parsing {filepath}: {e}")
        return None

def extract_parent_info_from_filename(filename):
    """Extract parent file ID and track info from split filename
    Format: {parent_id}_{original_name}_{track_num}_{instrument}.mid
    """
    # Example: 6614730_mikeshiver-feelings_bren-f_03_Bass pad.mid
    match = re.match(r'^(\d+)_', filename)
    if match:
        parent_id = int(match.group(1))

        # Try to extract track number (look for _XX_ pattern)
        track_match = re.search(r'_(\d{2})_', filename)
        track_number = int(track_match.group(1)) if track_match else 1

        return parent_id, track_number
    return None, None

def process_file(filepath):
    """Process a single split file and return data for insertion"""
    try:
        path = Path(filepath)
        filename = path.name

        # Get parent file ID from filename
        parent_id, track_number = extract_parent_info_from_filename(filename)

        # Calculate hash
        content_hash = calculate_blake3_hash(filepath)

        # Get file size
        file_size = path.stat().st_size

        # Parse MIDI
        midi_data = parse_midi_basic(filepath)
        if not midi_data:
            return None

        return {
            'filename': filename,
            'filepath': str(path),
            'content_hash': content_hash,
            'file_size_bytes': file_size,
            'format': midi_data['format'],
            'num_tracks': midi_data['num_tracks'],
            'ticks_per_quarter_note': midi_data['ticks_per_quarter_note'],
            'duration_seconds': midi_data['duration_seconds'],
            'duration_ticks': midi_data['duration_ticks'],
            'parent_file_id': parent_id,
            'track_number': track_number,
        }
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return None

def get_missing_files(limit=None):
    """Get list of files that exist on disk but not in database"""
    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Get all files currently in database from splits directory
    cur.execute("""
        SELECT filepath
        FROM files
        WHERE filepath LIKE '/home/dojevou/tmp/midi_splits_fast/%'
    """)

    db_files = set(row[0] for row in cur.fetchall())
    cur.close()
    conn.close()

    print(f"Found {len(db_files):,} split files in database")

    # Get all files on disk
    disk_files = []
    for file in Path(SPLITS_DIR).glob("*.mid"):
        if str(file) not in db_files:
            disk_files.append(str(file))
            if limit and len(disk_files) >= limit:
                break

    print(f"Found {len(disk_files):,} files on disk not in database")
    return disk_files

def import_files(files_to_import, batch_size=1000, workers=8):
    """Import missing files into database"""
    conn = psycopg2.connect(DB_URL)

    total = len(files_to_import)
    imported = 0
    errors = 0
    duplicates_skipped = 0

    print(f"\n{'='*70}")
    print(f"IMPORTING {total:,} SPLIT FILES")
    print(f"{'='*70}")
    print(f"Workers: {workers}")
    print(f"Batch size: {batch_size}\n")

    # Process files in parallel
    with Pool(workers) as pool:
        for i in range(0, total, batch_size):
            batch = files_to_import[i:i+batch_size]
            results = pool.map(process_file, batch)

            # Filter out None results (errors)
            valid_results = [r for r in results if r is not None]
            errors += len(results) - len(valid_results)

            if not valid_results:
                continue

            # Insert batch into database
            cur = conn.cursor()

            for data in valid_results:
                try:
                    # Check if hash already exists (deduplication)
                    cur.execute("""
                        SELECT id FROM files WHERE content_hash = %s
                    """, (data['content_hash'],))

                    if cur.fetchone():
                        duplicates_skipped += 1
                        continue

                    # Insert into files table
                    cur.execute("""
                        INSERT INTO files (
                            filename, filepath, original_filename, content_hash,
                            file_size_bytes, format, num_tracks, ticks_per_quarter_note,
                            duration_seconds, duration_ticks, parent_file_id
                        ) VALUES (
                            %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s
                        ) RETURNING id
                    """, (
                        data['filename'], data['filepath'], data['filename'],
                        data['content_hash'], data['file_size_bytes'],
                        data['format'], data['num_tracks'],
                        data['ticks_per_quarter_note'], data['duration_seconds'],
                        data['duration_ticks'], data['parent_file_id']
                    ))

                    file_id = cur.fetchone()[0]

                    # Create track_splits relationship if parent exists
                    if data['parent_file_id']:
                        cur.execute("""
                            INSERT INTO track_splits (
                                parent_file_id, split_file_id, track_number
                            ) VALUES (%s, %s, %s)
                            ON CONFLICT DO NOTHING
                        """, (data['parent_file_id'], file_id, data['track_number']))

                    imported += 1

                except IntegrityError:
                    conn.rollback()
                    duplicates_skipped += 1
                except Exception as e:
                    conn.rollback()
                    errors += 1
                    if errors <= 10:
                        print(f"Error importing {data['filename']}: {e}")

            conn.commit()
            cur.close()

            # Progress update
            progress = min(i + batch_size, total)
            pct = (progress / total * 100)
            print(f"Progress: {progress:,}/{total:,} ({pct:.1f}%) | "
                  f"Imported: {imported:,} | Skipped: {duplicates_skipped:,} | "
                  f"Errors: {errors:,}")

    conn.close()

    print(f"\n{'='*70}")
    print(f"IMPORT COMPLETE")
    print(f"{'='*70}")
    print(f"Files imported:      {imported:,}")
    print(f"Duplicates skipped:  {duplicates_skipped:,}")
    print(f"Errors:              {errors:,}")
    print(f"Total processed:     {imported + duplicates_skipped + errors:,}")
    print()

if __name__ == "__main__":
    if "--test" in sys.argv:
        print("Testing on 100 files...")
        missing = get_missing_files(limit=100)
        if missing:
            import_files(missing, batch_size=50, workers=4)
    elif "--run" in sys.argv:
        auto_yes = "--yes" in sys.argv
        if not auto_yes:
            response = input("Import ALL missing split files? Type 'yes': ")
            confirmed = response.lower() == 'yes'
        else:
            confirmed = True

        if confirmed:
            missing = get_missing_files()
            if missing:
                import_files(missing, batch_size=1000, workers=8)
        else:
            print("Cancelled.")
    else:
        print("Usage:")
        print("  --test    Test import on 100 files")
        print("  --run     Import all missing files")
        print("  --run --yes    Import all missing files without confirmation")
