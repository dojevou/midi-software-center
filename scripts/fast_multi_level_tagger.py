#!/usr/bin/env python3
"""
Fast Multi-Level Tagging Script

Extracts keywords from grandparent folders, parent folders, and filenames
in a single pass through all files, then batch inserts tags.

Performance: ~5-15 minutes for 1.79M files (vs 8 hours for sequential)
"""

import psycopg2
import psycopg2.extras
import re
import sys
from pathlib import Path
from collections import defaultdict
from typing import Set, Dict, List, Tuple
import time

# Configuration
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"
BATCH_SIZE = 10000  # Insert 10k tags at once
CHUNK_SIZE = 5000   # Process 5k files at a time
MIN_KEYWORD_LENGTH = 3
MAX_KEYWORD_LENGTH = 50

# Tag categories
CATEGORY_GRANDPARENT = "grandparent_folder"
CATEGORY_PARENT = "parent_folder"
CATEGORY_FILENAME = "filename"

def normalize_keyword(text: str) -> Set[str]:
    """
    Normalize a string into keywords.

    Examples:
        "120bpm_Bass_Loop" -> {"120bpm", "bass", "loop"}
        "4-4 Grooves" -> {"grooves"}
        "EDM Midi pack" -> {"edm", "midi", "pack"}
    """
    if not text:
        return set()

    # Lowercase
    text = text.lower()

    # Replace delimiters with spaces
    text = re.sub(r'[_\-\(\)\[\]@#]', ' ', text)

    # Split on spaces and filter
    keywords = set()
    for word in text.split():
        # Clean word
        word = word.strip()

        # Skip if too short or too long
        if len(word) < MIN_KEYWORD_LENGTH or len(word) > MAX_KEYWORD_LENGTH:
            continue

        # Skip numbers-only
        if word.isdigit():
            continue

        # Skip common noise words
        if word in {'the', 'and', 'for', 'with', 'from', 'midi', 'mid'}:
            continue

        keywords.add(word)

    return keywords

def extract_path_components(filepath: str) -> Tuple[str, str, str]:
    """
    Extract grandparent folder, parent folder, and filename from path.

    Example: "./archives/Linear Drums/House/track.mid"
    Returns: ("Linear Drums", "House", "track")
    """
    path = Path(filepath)

    # Filename without extension
    filename = path.stem

    # Parent folder (1 level up)
    parent = path.parent.name if path.parent != Path('.') else ""

    # Grandparent folder (2 levels up)
    grandparent = ""
    if path.parent != Path('.') and path.parent.parent != Path('.'):
        grandparent = path.parent.parent.name

    return (grandparent, parent, filename)

def load_curated_tags(conn) -> Dict[str, int]:
    """
    Load curated tags from master_tag_list.txt and insert into database.
    Returns mapping of tag_name -> tag_id
    """
    print("📋 Loading curated tags...")

    # Check if master tag list exists
    tag_file = Path("/tmp/master_tag_list.txt")
    if not tag_file.exists():
        print("❌ ERROR: /tmp/master_tag_list.txt not found!")
        print("   Run: ./scripts/create-curated-tags.sh first")
        sys.exit(1)

    # Load tags from file
    tags_to_insert = []
    with open(tag_file, 'r') as f:
        for line in f:
            tag_name = line.strip()
            if tag_name and len(tag_name) >= MIN_KEYWORD_LENGTH:
                # Determine category (just use 'keyword' for now)
                tags_to_insert.append((tag_name, 'keyword'))

    print(f"   Found {len(tags_to_insert)} tags to insert")

    # Insert tags (ignore duplicates)
    cursor = conn.cursor()
    inserted = 0
    for tag_name, category in tags_to_insert:
        try:
            cursor.execute(
                "INSERT INTO tags (name, category) VALUES (%s, %s) ON CONFLICT (name) DO NOTHING",
                (tag_name, category)
            )
            if cursor.rowcount > 0:
                inserted += 1
        except Exception as e:
            print(f"   Warning: Failed to insert tag '{tag_name}': {e}")

    conn.commit()
    print(f"   ✅ Inserted {inserted} new tags")

    # Load tag mapping
    cursor.execute("SELECT id, LOWER(name) FROM tags")
    tag_map = {name: tag_id for tag_id, name in cursor.fetchall()}

    print(f"   ✅ Loaded {len(tag_map)} total tags into memory")
    return tag_map

def process_files(conn, tag_map: Dict[str, int]):
    """
    Process all files and extract tags from paths.
    """
    print("\n📂 Processing files...")

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)

    # Get total file count
    cursor.execute("SELECT COUNT(*) FROM files")
    total_files = cursor.fetchone()[0]
    print(f"   Total files: {total_files:,}")

    # Statistics
    files_processed = 0
    tags_found = 0
    batch = []

    # Process files in chunks
    offset = 0
    start_time = time.time()

    while True:
        # Fetch chunk of files
        cursor.execute(
            """
            SELECT id, filepath, filename
            FROM files
            ORDER BY id
            LIMIT %s OFFSET %s
            """,
            (CHUNK_SIZE, offset)
        )

        rows = cursor.fetchall()
        if not rows:
            break

        # Process each file
        for row in rows:
            file_id = row['id']
            filepath = row['filepath'] or ""
            filename = row['filename'] or ""

            # Extract path components
            grandparent, parent, fname = extract_path_components(filepath)

            # Collect all keywords
            all_keywords = set()

            # Grandparent keywords
            all_keywords.update(normalize_keyword(grandparent))

            # Parent keywords
            all_keywords.update(normalize_keyword(parent))

            # Filename keywords
            all_keywords.update(normalize_keyword(fname))
            all_keywords.update(normalize_keyword(filename))

            # Match keywords against tag map
            for keyword in all_keywords:
                if keyword in tag_map:
                    tag_id = tag_map[keyword]
                    batch.append((file_id, tag_id))
                    tags_found += 1

            files_processed += 1

            # Batch insert when batch is full
            if len(batch) >= BATCH_SIZE:
                insert_batch(conn, batch)
                batch = []

            # Progress reporting
            if files_processed % 10000 == 0:
                elapsed = time.time() - start_time
                rate = files_processed / elapsed
                eta = (total_files - files_processed) / rate if rate > 0 else 0
                print(f"   Progress: {files_processed:,}/{total_files:,} files "
                      f"({files_processed/total_files*100:.1f}%) | "
                      f"{rate:.0f} files/sec | "
                      f"ETA: {eta/60:.1f} min | "
                      f"Tags: {tags_found:,}")

        offset += CHUNK_SIZE

    # Insert remaining batch
    if batch:
        insert_batch(conn, batch)

    elapsed = time.time() - start_time
    print(f"\n   ✅ Processed {files_processed:,} files in {elapsed:.1f} seconds")
    print(f"   ✅ Found {tags_found:,} tag relationships")
    print(f"   ✅ Average: {files_processed/elapsed:.0f} files/sec")

def insert_batch(conn, batch: List[Tuple[int, int]]):
    """
    Batch insert file_tags relationships.
    """
    if not batch:
        return

    cursor = conn.cursor()

    # Use execute_values for fast batch insert
    psycopg2.extras.execute_values(
        cursor,
        """
        INSERT INTO file_tags (file_id, tag_id)
        VALUES %s
        ON CONFLICT (file_id, tag_id) DO NOTHING
        """,
        batch,
        page_size=1000
    )

    conn.commit()

def verify_results(conn):
    """
    Verify tagging results and show statistics.
    """
    print("\n📊 Verification and Statistics")
    print("━" * 60)

    cursor = conn.cursor()

    # Files tagged
    cursor.execute("""
        SELECT
            COUNT(DISTINCT file_id) as files_tagged,
            COUNT(*) as total_tags,
            ROUND(AVG(tags_per_file), 2) as avg_tags_per_file
        FROM (
            SELECT file_id, COUNT(*) as tags_per_file
            FROM file_tags
            GROUP BY file_id
        ) t
    """)

    row = cursor.fetchone()
    print(f"Files tagged:        {row[0]:,}")
    print(f"Total tag assignments: {row[1]:,}")
    print(f"Average tags/file:   {row[2]}")

    # Tag distribution
    cursor.execute("""
        SELECT COUNT(*) as tag_count, COUNT(file_id) as file_count
        FROM (
            SELECT file_id, COUNT(*) as tag_count
            FROM file_tags
            GROUP BY file_id
        ) t
        GROUP BY tag_count
        ORDER BY tag_count
        LIMIT 10
    """)

    print("\nTag distribution (files by tag count):")
    for row in cursor.fetchall():
        print(f"  {row[0]} tags: {row[1]:,} files")

    # Top 20 tags
    cursor.execute("""
        SELECT t.name, COUNT(*) as file_count
        FROM file_tags ft
        JOIN tags t ON ft.tag_id = t.id
        GROUP BY t.name
        ORDER BY file_count DESC
        LIMIT 20
    """)

    print("\nTop 20 most common tags:")
    for i, row in enumerate(cursor.fetchall(), 1):
        print(f"  {i:2}. {row[0]:20} - {row[1]:,} files")

def main():
    """
    Main execution function.
    """
    print("━" * 60)
    print("  Fast Multi-Level Tagging")
    print("━" * 60)
    print()

    # Connect to database
    print("🔌 Connecting to database...")
    try:
        conn = psycopg2.connect(DB_URL)
        conn.set_session(autocommit=False)
        print("   ✅ Connected")
    except Exception as e:
        print(f"   ❌ Connection failed: {e}")
        sys.exit(1)

    try:
        # Load curated tags
        tag_map = load_curated_tags(conn)

        # Process files
        process_files(conn, tag_map)

        # Verify results
        verify_results(conn)

        print("\n✅ Tagging complete!")

    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        conn.rollback()
        sys.exit(1)
    finally:
        conn.close()

if __name__ == "__main__":
    main()
