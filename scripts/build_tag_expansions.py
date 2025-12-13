#!/usr/bin/env python3
"""
Build MPC Expansion Packs from Database Tags

Queries PostgreSQL database for files matching tag criteria,
converts MIDI → .mpcpattern, and organizes into expansion pack structure.

Usage:
    python3 scripts/build_tag_expansions.py --pack rock-drums --limit 1000
    python3 scripts/build_tag_expansions.py --pack all --output /path/to/expansions
    python3 scripts/build_tag_expansions.py --list  # Show available packs
"""

import psycopg2
import subprocess
import json
import os
import sys
from pathlib import Path
from collections import defaultdict
import argparse
import datetime

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

# Converter path
CONVERTER = "./target/release/midi_to_mpcpattern"

# Expansion pack definitions
EXPANSION_PACKS = {
    'rock-drums': {
        'name': 'Rock Drum Patterns',
        'description': 'Essential rock drum patterns across all tempos',
        'tags': ['rock'],
        'tag_category': 'genre',
        'bpm_min': 100,
        'bpm_max': 160,
        'limit': 1000,
        'organize_by': 'bpm',  # or 'key', 'flat'
        'bpm_folders': ['100-120', '120-140', '140-160']
    },

    'ride-library': {
        'name': 'Ride Cymbal Library',
        'description': 'Comprehensive ride cymbal patterns for all styles',
        'tags': ['ride'],
        'tag_category': 'drums',
        'bpm_min': 60,
        'bpm_max': 200,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['060-100', '100-140', '140-180', '180+']
    },

    'kick-collection': {
        'name': 'Kick Drum Collection',
        'description': 'Kick drum patterns from soft to hard',
        'tags': ['kick'],
        'tag_category': 'drums',
        'bpm_min': 80,
        'bpm_max': 180,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['080-100', '100-120', '120-140', '140-180']
    },

    'house-grooves': {
        'name': 'House Grooves 120-130',
        'description': 'Classic house drum patterns in the perfect BPM range',
        'tags': ['house'],
        'tag_category': 'genre',
        'bpm_min': 120,
        'bpm_max': 130,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['120-124', '124-128', '128-130']
    },

    'fill-patterns': {
        'name': 'Fill Patterns Collection',
        'description': 'Drum fills for transitions and breaks',
        'tags': ['fill'],
        'tag_category': 'drums',
        'bpm_min': 80,
        'bpm_max': 180,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['080-110', '110-140', '140-180']
    },

    'funk-drums': {
        'name': 'Funk Drum Patterns',
        'description': 'Funky grooves and pocket patterns',
        'tags': ['funk'],
        'tag_category': 'genre',
        'bpm_min': 90,
        'bpm_max': 120,
        'limit': 400,
        'organize_by': 'bpm',
        'bpm_folders': ['090-100', '100-110', '110-120']
    },

    'tom-patterns': {
        'name': 'Tom Patterns',
        'description': 'Tom drum patterns and fills',
        'tags': ['tom'],
        'tag_category': 'drums',
        'bpm_min': 80,
        'bpm_max': 180,
        'limit': 400,
        'organize_by': 'bpm',
        'bpm_folders': ['080-120', '120-150', '150-180']
    },

    'groove-library': {
        'name': 'Groove Library',
        'description': 'Essential groove patterns for all styles',
        'tags': ['groove'],
        'tag_category': 'pattern',
        'bpm_min': 80,
        'bpm_max': 160,
        'limit': 400,
        'organize_by': 'bpm',
        'bpm_folders': ['080-100', '100-120', '120-140', '140-160']
    },

    'edm-drums': {
        'name': 'EDM Drums 120-140',
        'description': 'Electronic dance music drum patterns',
        'tags': ['edm'],
        'tag_category': 'genre',
        'bpm_min': 120,
        'bpm_max': 140,
        'limit': 300,
        'organize_by': 'bpm',
        'bpm_folders': ['120-128', '128-135', '135-140']
    },

    'hiphop-beats': {
        'name': 'Hip-Hop Beats 85-100',
        'description': 'Classic hip-hop drum patterns',
        'tags': ['beat', 'drum'],  # Multiple tags
        'tag_category': ['keyword', 'drums'],
        'bpm_min': 85,
        'bpm_max': 100,
        'limit': 300,
        'organize_by': 'bpm',
        'bpm_folders': ['085-090', '090-095', '095-100']
    }
}


def query_files_for_pack(config):
    """Query database for files matching pack criteria"""

    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    tags = config['tags']
    tag_category = config['tag_category']
    bpm_min = config['bpm_min']
    bpm_max = config['bpm_max']
    limit = config['limit']

    # Build query
    if isinstance(tag_category, list):
        # Multiple tag categories
        tag_conditions = " OR ".join([f"t.category = '{cat}'" for cat in tag_category])
    else:
        tag_conditions = f"t.category = '{tag_category}'"

    tag_names = "', '".join(tags)

    query = f"""
    SELECT DISTINCT
        f.id,
        f.filepath,
        m.bpm,
        m.key_signature,
        string_agg(DISTINCT t.name, ', ') as tags
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('{tag_names}')
      AND ({tag_conditions})
      AND m.bpm BETWEEN {bpm_min} AND {bpm_max}
      AND m.bpm IS NOT NULL
    GROUP BY f.id, f.filepath, m.bpm, m.key_signature
    ORDER BY m.bpm, f.filepath
    LIMIT {limit};
    """

    print(f"Executing query...")
    cur.execute(query)
    results = cur.fetchall()

    cur.close()
    conn.close()

    return results


def get_bpm_folder(bpm, bpm_folders):
    """Determine which BPM folder a file belongs to"""
    for folder in bpm_folders:
        if '+' in folder:
            # e.g., "180+"
            min_bpm = int(folder.replace('+', ''))
            if bpm >= min_bpm:
                return folder
        else:
            # e.g., "120-140"
            parts = folder.split('-')
            min_bpm = int(parts[0])
            max_bpm = int(parts[1])
            if min_bpm <= bpm < max_bpm:
                return folder

    # Fallback
    return 'Other'


def convert_and_organize(files, pack_id, config, output_base):
    """Convert MIDI files and organize into expansion pack structure"""

    pack_name = config['name']
    pack_dir = output_base / pack_name

    # Create expansion folder
    pack_dir.mkdir(parents=True, exist_ok=True)

    # Create [Previews] folder
    previews_dir = pack_dir / "[Previews]"
    previews_dir.mkdir(exist_ok=True)

    print(f"\nBuilding expansion: {pack_name}")
    print(f"  Output: {pack_dir}")
    print(f"  Files to convert: {len(files)}")

    # Organize files by BPM folder
    if config['organize_by'] == 'bpm':
        files_by_folder = defaultdict(list)
        for file_data in files:
            file_id, filepath, bpm, key_sig, tags = file_data
            folder_name = get_bpm_folder(bpm, config['bpm_folders'])
            files_by_folder[folder_name].append(file_data)

        # Create BPM subfolders and convert
        total_converted = 0
        total_failed = 0

        for folder_name, folder_files in files_by_folder.items():
            folder_path = pack_dir / folder_name
            folder_path.mkdir(exist_ok=True)

            print(f"\n  Converting {len(folder_files)} files to {folder_name}/")

            for i, file_data in enumerate(folder_files):
                file_id, filepath, bpm, key_sig, tags = file_data

                # Generate output filename
                file_stem = Path(filepath).stem
                output_name = f"{int(bpm):03d}bpm-{file_stem}.mpcpattern"
                output_path = folder_path / output_name

                # Convert using Rust converter
                try:
                    result = subprocess.run(
                        [CONVERTER, filepath, str(output_path)],
                        capture_output=True,
                        text=True,
                        timeout=30
                    )

                    if result.returncode == 0:
                        total_converted += 1
                        if (i + 1) % 50 == 0:
                            print(f"    Progress: {i+1}/{len(folder_files)}")
                    else:
                        total_failed += 1
                        print(f"    ✗ Error converting: {filepath}")

                except Exception as e:
                    total_failed += 1
                    print(f"    ✗ Exception: {e}")

        print(f"\n  ✓ Converted: {total_converted}")
        print(f"  ✗ Failed: {total_failed}")

    else:
        # Flat organization
        print(f"  Converting all files to root...")
        total_converted = 0
        total_failed = 0

        for i, file_data in enumerate(files):
            file_id, filepath, bpm, key_sig, tags = file_data

            file_stem = Path(filepath).stem
            output_name = f"{int(bpm):03d}bpm-{file_stem}.mpcpattern"
            output_path = pack_dir / output_name

            try:
                result = subprocess.run(
                    [CONVERTER, filepath, str(output_path)],
                    capture_output=True,
                    text=True,
                    timeout=30
                )

                if result.returncode == 0:
                    total_converted += 1
                    if (i + 1) % 50 == 0:
                        print(f"    Progress: {i+1}/{len(files)}")
                else:
                    total_failed += 1

            except Exception as e:
                total_failed += 1

        print(f"\n  ✓ Converted: {total_converted}")
        print(f"  ✗ Failed: {total_failed}")

    # Create Cache.json (minimal for now)
    cache_data = {
        "name": pack_name,
        "description": config['description'],
        "version": "1.0",
        "files": total_converted
    }

    with open(pack_dir / "Cache.json", 'w') as f:
        json.dump(cache_data, f, indent=2)

    # Create README
    readme_content = f"""# {pack_name}

{config['description']}

## Contents
- {total_converted} .mpcpattern files
- Organized by: {config['organize_by']}
- BPM range: {config['bpm_min']}-{config['bpm_max']}
- Tags: {', '.join(config['tags'])}

## Installation
1. Copy this folder to your Force/MPC drive: /Expansions/
2. The expansion will appear in your Expansion Browser

## Generated by
MIDI Software Center - Tag-Based Expansion Builder
Date: {datetime.datetime.now().strftime('%Y-%m-%d')}
"""

    with open(pack_dir / "README.txt", 'w') as f:
        f.write(readme_content)

    return total_converted, total_failed


def build_pack(pack_id, output_dir):
    """Build a single expansion pack"""

    if pack_id not in EXPANSION_PACKS:
        print(f"Error: Unknown pack '{pack_id}'")
        print(f"Available packs: {', '.join(EXPANSION_PACKS.keys())}")
        return False

    config = EXPANSION_PACKS[pack_id]

    print(f"="*70)
    print(f"Building: {config['name']}")
    print(f"="*70)

    # Query database
    files = query_files_for_pack(config)
    print(f"Found {len(files)} matching files")

    if len(files) == 0:
        print("No files found matching criteria!")
        return False

    # Convert and organize
    converted, failed = convert_and_organize(files, pack_id, config, output_dir)

    print(f"\n✓ Pack complete: {config['name']}")
    print(f"  Location: {output_dir / config['name']}")
    print(f"  Patterns: {converted}")

    return True


def list_packs():
    """List all available expansion packs"""
    print("\nAvailable Expansion Packs:\n")
    for pack_id, config in EXPANSION_PACKS.items():
        print(f"  {pack_id:20s} - {config['name']}")
        print(f"  {' '*20}   {config['description']}")
        print(f"  {' '*20}   Limit: {config['limit']}, BPM: {config['bpm_min']}-{config['bpm_max']}")
        print()


def main():
    parser = argparse.ArgumentParser(description='Build MPC expansion packs from database tags')
    parser.add_argument('--pack', help='Pack ID to build (or "all" for all packs)')
    parser.add_argument('--output', default='/media/dojevou/RYXSTR/Expansions',
                       help='Output directory for expansions')
    parser.add_argument('--list', action='store_true', help='List available packs')

    args = parser.parse_args()

    if args.list:
        list_packs()
        return

    if not args.pack:
        print("Error: --pack required (or use --list to see available packs)")
        return

    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    if args.pack == 'all':
        print(f"Building ALL expansion packs to: {output_dir}\n")
        success_count = 0
        for pack_id in EXPANSION_PACKS.keys():
            if build_pack(pack_id, output_dir):
                success_count += 1
            print("\n")

        print(f"="*70)
        print(f"Completed: {success_count}/{len(EXPANSION_PACKS)} packs")
        print(f"="*70)
    else:
        build_pack(args.pack, output_dir)


if __name__ == '__main__':
    main()
