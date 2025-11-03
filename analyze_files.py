#!/usr/bin/env python3
"""
Simple MIDI Analysis Script
Analyzes all unanalyzed MIDI files and populates musical_metadata table
"""

import asyncio
import asyncpg
import mido
import time
from pathlib import Path
from typing import Optional, Tuple, Dict
import statistics

DATABASE_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def analyze_midi_file(filepath: str) -> Dict:
    """Analyze a MIDI file and extract musical metadata"""
    try:
        midi = mido.MidiFile(filepath)

        # Calculate basic stats
        total_notes = 0
        pitches = []
        velocities = []
        tempo_events = []
        has_pitch_bend = False

        # Get tempo from first tempo event
        bpm = None
        for track in midi.tracks:
            for msg in track:
                if msg.type == 'set_tempo':
                    bpm = mido.tempo2bpm(msg.tempo)
                    tempo_events.append(bpm)
                    break
            if bpm:
                break

        # Analyze notes
        for track in midi.tracks:
            for msg in track:
                if msg.type == 'note_on' and msg.velocity > 0:
                    total_notes += 1
                    pitches.append(msg.note)
                    velocities.append(msg.velocity)
                elif msg.type == 'pitchwheel':
                    has_pitch_bend = True

        # Calculate metadata
        result = {
            'total_notes': total_notes,
            'bpm': bpm if bpm and 20 <= bpm <= 300 else None,
            'bpm_confidence': 0.8 if bpm else None,
            'has_tempo_changes': len(tempo_events) > 1,
            'pitch_range_min': min(pitches) if pitches else None,
            'pitch_range_max': max(pitches) if pitches else None,
            'avg_velocity': statistics.mean(velocities) if velocities else None,
            'polyphony_max': None,  # Would need to track concurrent notes
            'is_percussive': any(p >= 35 and p <= 81 for p in pitches) if pitches else False,
            'has_chords': total_notes > 10,  # Simple heuristic
            'has_melody': len(set(pitches)) > 5 if pitches else False,
        }

        return result
    except Exception as e:
        print(f"Error analyzing {filepath}: {e}")
        return None

async def analyze_all_files():
    """Analyze all unanalyzed files"""
    print("🎵 MIDI Analysis Tool")
    print("=" * 60)

    # Connect to database
    print("\n📡 Connecting to database...")
    conn = await asyncpg.connect(DATABASE_URL)
    print("✅ Connected to database\n")

    # Get unanalyzed files (real files, not test placeholders)
    query = """
        SELECT id, filepath
        FROM files
        WHERE analyzed_at IS NULL
          AND filepath NOT LIKE '/test/%'
        ORDER BY id
    """
    files = await conn.fetch(query)
    total = len(files)

    print(f"🔍 Found {total} unanalyzed files\n")

    if total == 0:
        print("✅ All files are already analyzed!")
        await conn.close()
        return

    start_time = time.time()
    analyzed = 0
    skipped = 0
    errors = []

    print("🚀 Starting analysis...\n")

    # Process files
    for i, record in enumerate(files, 1):
        file_id = record['id']
        filepath = record['filepath']

        # Progress update every 100 files
        if i % 100 == 0 or i == total:
            elapsed = time.time() - start_time
            rate = i / elapsed if elapsed > 0 else 0
            eta = (total - i) / rate if rate > 0 else 0
            print(f"Progress: {i}/{total} ({i/total*100:.1f}%) - {rate:.1f} files/sec - ETA: {eta:.0f}s")

        # Check if file exists
        if not Path(filepath).exists():
            skipped += 1
            if len(errors) < 10:
                errors.append(f"File not found: {filepath}")
            continue

        # Analyze file
        metadata = analyze_midi_file(filepath)

        if metadata is None:
            skipped += 1
            continue

        # Insert into database
        try:
            await conn.execute("""
                INSERT INTO musical_metadata (
                    file_id,
                    bpm,
                    bpm_confidence,
                    has_tempo_changes,
                    total_notes,
                    pitch_range_min,
                    pitch_range_max,
                    avg_velocity,
                    polyphony_max,
                    is_percussive,
                    has_chords,
                    has_melody
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (file_id) DO UPDATE SET
                    bpm = EXCLUDED.bpm,
                    bpm_confidence = EXCLUDED.bpm_confidence,
                    has_tempo_changes = EXCLUDED.has_tempo_changes,
                    total_notes = EXCLUDED.total_notes,
                    pitch_range_min = EXCLUDED.pitch_range_min,
                    pitch_range_max = EXCLUDED.pitch_range_max,
                    avg_velocity = EXCLUDED.avg_velocity,
                    polyphony_max = EXCLUDED.polyphony_max,
                    is_percussive = EXCLUDED.is_percussive,
                    has_chords = EXCLUDED.has_chords,
                    has_melody = EXCLUDED.has_melody
            """,
                file_id,
                metadata['bpm'],
                metadata['bpm_confidence'],
                metadata['has_tempo_changes'],
                metadata['total_notes'],
                metadata['pitch_range_min'],
                metadata['pitch_range_max'],
                metadata['avg_velocity'],
                metadata['polyphony_max'],
                metadata['is_percussive'],
                metadata['has_chords'],
                metadata['has_melody']
            )

            # Mark as analyzed
            await conn.execute(
                "UPDATE files SET analyzed_at = NOW() WHERE id = $1",
                file_id
            )

            analyzed += 1
        except Exception as e:
            skipped += 1
            if len(errors) < 10:
                errors.append(f"DB error for {filepath}: {e}")

    # Final statistics
    duration = time.time() - start_time
    rate = analyzed / duration if duration > 0 else 0

    print("\n✅ Analysis complete!")
    print("=" * 60)
    print(f"  Total files:    {total}")
    print(f"  Analyzed:       {analyzed}")
    print(f"  Skipped:        {skipped}")
    print(f"  Duration:       {duration:.1f}s")
    print(f"  Average rate:   {rate:.1f} files/sec")

    if errors:
        print(f"\n⚠️  Errors encountered ({len(errors)}):")
        for error in errors[:10]:
            print(f"  - {error}")
        if len(errors) > 10:
            print(f"  ... and {len(errors) - 10} more errors")

    await conn.close()

if __name__ == "__main__":
    asyncio.run(analyze_all_files())
