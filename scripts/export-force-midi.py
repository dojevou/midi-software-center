#!/usr/bin/env python3
"""
Export MIDI files from database to Akai Force folder structure.
Exports to:
  - Progressions/ (chord progressions by key)
  - Arp Patterns/ (arpeggiator patterns by BPM)
  - MIDI_Patterns/ (drums, bass, keys organized)
"""

import psycopg2
import psycopg2.extras
import shutil
import os
from pathlib import Path
import sys

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

# Export destination (Akai Force drive)
FORCE_ROOT = "/media/dojevou/RYXSTR"

# Export limits (adjust as needed)
LIMITS = {
    'progressions': 10000,  # Top 10,000 chord progressions
    'arp_patterns': 5000,   # Top 5,000 arp patterns
    'drum_patterns': 20000, # Top 20,000 drum patterns
    'bass_patterns': 5000,  # Top 5,000 bass patterns
    'key_patterns': 5000,   # Top 5,000 key/piano patterns
}

def connect_db():
    """Connect to PostgreSQL database"""
    return psycopg2.connect(DB_URL)

def export_progressions(conn, limit=10000, dry_run=False):
    """Export chord progressions organized by key"""
    print(f"\n{'='*70}")
    print(f"EXPORTING CHORD PROGRESSIONS (limit: {limit:,})")
    print(f"{'='*70}\n")

    query = """
    SELECT
        f.filepath,
        f.filename,
        m.key_signature,
        m.bpm,
        COUNT(ft.tag_id) as tag_count,
        string_agg(t.name, ', ') as tags
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('chord', 'progression', 'piano', 'keys', 'pad', 'synth')
      AND m.key_signature IS NOT NULL
    GROUP BY f.id, f.filepath, f.filename, m.key_signature, m.bpm
    HAVING COUNT(ft.tag_id) >= 3
    ORDER BY m.key_signature, COUNT(ft.tag_id) DESC, m.bpm
    LIMIT %s;
    """

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)
    cursor.execute(query, (limit,))

    results = cursor.fetchall()
    print(f"Found {len(results):,} chord progression files\n")

    # Map database keys to folder names
    key_map = {
        'C': 'C Major', 'Cm': 'C Minor',
        'C#': 'C# Major', 'C#m': 'C# Minor',
        'D': 'D Major', 'Dm': 'D Minor',
        'D#': 'D# Major', 'D#m': 'D# Minor',
        'E': 'E Major', 'Em': 'E Minor',
        'F': 'F Major', 'Fm': 'F Minor',
        'F#': 'F# Major', 'F#m': 'F# Minor',
        'G': 'G Major', 'Gm': 'G Minor',
        'G#': 'G# Major', 'G#m': 'G# Minor',
        'A': 'A Major', 'Am': 'A Minor',
        'A#': 'A# Major', 'A#m': 'A# Minor',
        'B': 'B Major', 'Bm': 'B Minor',
    }

    stats = {}
    copied = 0
    skipped = 0
    errors = 0

    for row in results:
        source = row['filepath']
        key_sig = row['key_signature']
        bpm = row['bpm'] or 120

        # Map key signature to folder
        folder_key = key_map.get(key_sig)
        if not folder_key:
            # Try to infer (e.g., "Cmaj" → "C Major")
            if 'maj' in key_sig.lower():
                folder_key = key_sig.replace('maj', '').strip() + ' Major'
            elif 'min' in key_sig.lower():
                folder_key = key_sig.replace('min', '').strip() + ' Minor'
            else:
                folder_key = 'Unknown'

        dest_dir = Path(FORCE_ROOT) / "Progressions" / folder_key
        dest_file = dest_dir / f"{int(bpm)}bpm_{Path(source).name}"

        stats[folder_key] = stats.get(folder_key, 0) + 1

        if not dry_run:
            try:
                if not dest_file.exists():
                    dest_dir.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source, dest_file)
                    copied += 1
                else:
                    skipped += 1
            except Exception as e:
                print(f"Error copying {source}: {e}")
                errors += 1
        else:
            copied += 1

    print("Progressions by key:")
    for key in sorted(stats.keys()):
        print(f"  {key:20s}: {stats[key]:,} files")

    print(f"\nTotal: {copied:,} copied, {skipped:,} skipped, {errors:,} errors")
    return copied

def export_arp_patterns(conn, limit=5000, dry_run=False):
    """Export arpeggiator patterns organized by BPM"""
    print(f"\n{'='*70}")
    print(f"EXPORTING ARP PATTERNS (limit: {limit:,})")
    print(f"{'='*70}\n")

    query = """
    SELECT
        f.filepath,
        f.filename,
        m.bpm,
        m.key_signature,
        COUNT(ft.tag_id) as tag_count
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('arp', 'arpeggiat', 'pattern', 'sequence')
      AND m.bpm IS NOT NULL
      AND m.bpm BETWEEN 60 AND 180
    GROUP BY f.id, f.filepath, f.filename, m.bpm, m.key_signature
    HAVING COUNT(ft.tag_id) >= 2
    ORDER BY m.bpm, COUNT(ft.tag_id) DESC
    LIMIT %s;
    """

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)
    cursor.execute(query, (limit,))

    results = cursor.fetchall()
    print(f"Found {len(results):,} arp pattern files\n")

    stats = {}
    copied = 0
    skipped = 0
    errors = 0

    for row in results:
        source = row['filepath']
        bpm = int(row['bpm'])

        # Determine BPM folder
        if bpm < 80:
            bpm_folder = '60-80_BPM'
        elif bpm < 100:
            bpm_folder = '80-100_BPM'
        elif bpm < 120:
            bpm_folder = '100-120_BPM'
        elif bpm < 140:
            bpm_folder = '120-140_BPM'
        elif bpm < 160:
            bpm_folder = '140-160_BPM'
        else:
            bpm_folder = '160-180_BPM'

        dest_dir = Path(FORCE_ROOT) / "Arp Patterns" / bpm_folder
        dest_file = dest_dir / f"{bpm}bpm_{Path(source).name}"

        stats[bpm_folder] = stats.get(bpm_folder, 0) + 1

        if not dry_run:
            try:
                if not dest_file.exists():
                    dest_dir.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source, dest_file)
                    copied += 1
                else:
                    skipped += 1
            except Exception as e:
                print(f"Error copying {source}: {e}")
                errors += 1
        else:
            copied += 1

    print("Arp patterns by BPM:")
    for folder in sorted(stats.keys()):
        print(f"  {folder:20s}: {stats[folder]:,} files")

    print(f"\nTotal: {copied:,} copied, {skipped:,} skipped, {errors:,} errors")
    return copied

def export_drum_patterns(conn, limit=20000, dry_run=False):
    """Export drum patterns organized by BPM"""
    print(f"\n{'='*70}")
    print(f"EXPORTING DRUM PATTERNS (limit: {limit:,})")
    print(f"{'='*70}\n")

    query = """
    SELECT
        f.filepath,
        f.filename,
        m.bpm,
        COUNT(ft.tag_id) as tag_count,
        string_agg(t.name, ', ') as tags
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('drums', 'drum', 'groove', 'fill', 'kick', 'snare', 'hihat', 'cymbal', 'ride', 'crash', 'tom')
      AND m.bpm IS NOT NULL
      AND m.bpm BETWEEN 80 AND 180
    GROUP BY f.id, f.filepath, f.filename, m.bpm
    HAVING COUNT(ft.tag_id) >= 2
    ORDER BY m.bpm, COUNT(ft.tag_id) DESC
    LIMIT %s;
    """

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)
    cursor.execute(query, (limit,))

    results = cursor.fetchall()
    print(f"Found {len(results):,} drum pattern files\n")

    stats = {}
    copied = 0
    skipped = 0
    errors = 0

    for row in results:
        source = row['filepath']
        bpm = int(row['bpm'])

        # Determine BPM folder
        if bpm < 100:
            bpm_folder = '80-100_BPM'
        elif bpm < 120:
            bpm_folder = '100-120_BPM'
        elif bpm < 140:
            bpm_folder = '120-140_BPM'
        elif bpm < 160:
            bpm_folder = '140-160_BPM'
        else:
            bpm_folder = '160-180_BPM'

        dest_dir = Path(FORCE_ROOT) / "MIDI_Patterns" / "Drums" / bpm_folder
        dest_file = dest_dir / f"{bpm}bpm_{Path(source).name}"

        stats[bpm_folder] = stats.get(bpm_folder, 0) + 1

        if not dry_run:
            try:
                if not dest_file.exists():
                    dest_dir.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source, dest_file)
                    copied += 1
                else:
                    skipped += 1
            except Exception as e:
                print(f"Error copying {source}: {e}")
                errors += 1
        else:
            copied += 1

    print("Drum patterns by BPM:")
    for folder in sorted(stats.keys()):
        print(f"  {folder:20s}: {stats[folder]:,} files")

    print(f"\nTotal: {copied:,} copied, {skipped:,} skipped, {errors:,} errors")
    return copied

def main():
    dry_run = '--dry-run' in sys.argv or '--test' in sys.argv

    if dry_run:
        print("\n" + "="*70)
        print("DRY RUN MODE - No files will be copied")
        print("="*70 + "\n")

    conn = connect_db()

    try:
        # Export progressions
        prog_count = export_progressions(conn, LIMITS['progressions'], dry_run)

        # Export arp patterns
        arp_count = export_arp_patterns(conn, LIMITS['arp_patterns'], dry_run)

        # Export drum patterns
        drum_count = export_drum_patterns(conn, LIMITS['drum_patterns'], dry_run)

        # Summary
        print(f"\n{'='*70}")
        print("EXPORT COMPLETE")
        print(f"{'='*70}")
        print(f"Chord Progressions: {prog_count:,} files")
        print(f"Arp Patterns:       {arp_count:,} files")
        print(f"Drum Patterns:      {drum_count:,} files")
        print(f"{'='*70}")
        print(f"Total exported:     {prog_count + arp_count + drum_count:,} files")
        print(f"{'='*70}\n")

        if dry_run:
            print("This was a DRY RUN. Run without --dry-run to actually copy files.\n")

    finally:
        conn.close()

if __name__ == "__main__":
    main()
