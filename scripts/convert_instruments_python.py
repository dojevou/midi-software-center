#!/usr/bin/env python3
"""
Convert 2,000 patterns per instrument for all 97 expansions
Created: November 23, 2025
"""

import subprocess
import sys
from pathlib import Path

# Configuration
DB_HOST = "localhost"
DB_PORT = "5433"
DB_USER = "midiuser"
DB_PASS = "145278963"
DB_NAME = "midi_library"

PROJECT_ROOT = Path("/home/dojevou/projects/midi-software-center")
FORCE_DRIVE = Path("/media/dojevou/RYXSTR")
EXPANSIONS_DIR = FORCE_DRIVE / "Expansions"
CONVERTER = PROJECT_ROOT / "target/release/midi_to_mpcpattern_parallel"
INSTRUMENT_LIST = PROJECT_ROOT / "INSTRUMENT_LIST.txt"
PATTERNS_PER_INSTRUMENT = 2000

def main():
    print("=" * 50)
    print("  Converting 97 Instrument Expansions")
    print(f"  {PATTERNS_PER_INSTRUMENT:,} patterns per instrument")
    print(f"  ~{97 * PATTERNS_PER_INSTRUMENT:,} total patterns")
    print("=" * 50)
    print()

    # Read instrument list
    instruments = []
    with open(INSTRUMENT_LIST) as f:
        for line in f:
            line = line.strip()
            if not line or ':' not in line:
                continue
            instrument, count = line.split(':', 1)
            instruments.append((instrument.strip(), int(count.strip())))

    print(f"Found {len(instruments)} instruments")
    print()

    total_converted = 0
    total_failed = 0

    for idx, (instrument, db_count) in enumerate(instruments, 1):
        # Convert instrument name to expansion format
        expansion_name = f"MIDI_{instrument.upper().replace('-', '_').replace('&', 'AND')}"
        expansion_dir = EXPANSIONS_DIR / expansion_name
        patterns_dir = expansion_dir / "Patterns"

        if not expansion_dir.exists():
            print(f"⚠️  [{idx}/97] Skipping {expansion_name} (folder not found)")
            continue

        print(f"[{idx}/97] Processing: {expansion_name}")
        print(f"  Available in database: {db_count:,}")

        # Query database for files
        query = f"""
            SELECT f.filepath
            FROM files f
            LEFT JOIN file_tags ft ON f.id = ft.file_id
            LEFT JOIN tags t ON ft.tag_id = t.id
            WHERE f.num_tracks = 1
              AND (
                t.name = '{instrument}'
                OR f.filename ILIKE '%{instrument}%'
              )
            GROUP BY f.filepath
            ORDER BY RANDOM()
            LIMIT {PATTERNS_PER_INSTRUMENT};
        """

        try:
            import os
            env = os.environ.copy()
            env["PGPASSWORD"] = DB_PASS

            result = subprocess.run(
                ["psql", "-h", DB_HOST, "-p", DB_PORT, "-U", DB_USER, "-d", DB_NAME, "-t", "-c", query],
                env=env,
                capture_output=True,
                text=True
            )

            if result.returncode != 0:
                print(f"  ❌ Database query failed: {result.stderr.strip()}")
                continue

            filepaths = [line.strip() for line in result.stdout.strip().split('\n') if line.strip()]

            if not filepaths:
                print(f"  ⚠️  No files found")
                print()
                continue

            print(f"  Files to convert: {len(filepaths)}")

            # Convert files
            converted = 0
            failed = 0

            for filepath in filepaths:
                filepath_obj = Path(filepath)

                if not filepath_obj.exists():
                    failed += 1
                    continue

                filename = filepath_obj.stem
                output_file = patterns_dir / f"{filename}.mpcpattern"

                # Convert
                result = subprocess.run(
                    [str(CONVERTER), str(filepath), str(output_file)],
                    capture_output=True
                )

                if result.returncode == 0:
                    converted += 1
                else:
                    failed += 1

                # Progress dots
                if converted % 100 == 0 and converted > 0:
                    print(".", end="", flush=True)

            print()
            print(f"  ✓ Converted: {converted:,} patterns")
            if failed > 0:
                print(f"  ✗ Failed: {failed:,} files")

            total_converted += converted
            total_failed += failed

        except Exception as e:
            print(f"  ❌ Error: {e}")

        print()

    # Final summary
    print("=" * 50)
    print("  CONVERSION COMPLETE!")
    print("=" * 50)
    print()
    print("📊 Summary:")
    print(f"  - Instruments processed: {idx}/97")
    print(f"  - Total patterns converted: {total_converted:,}")
    print(f"  - Failed conversions: {total_failed:,}")
    print()
    print(f"📂 Location: {EXPANSIONS_DIR}")
    print()
    print("🎯 Next Steps:")
    print("  1. Test expansions on Force hardware")
    print("  2. Browser > Expansions > MIDI_[INSTRUMENT]")
    print("  3. Enjoy your library!")
    print()
    print("✓ All done!")

if __name__ == "__main__":
    main()
