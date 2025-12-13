#!/usr/bin/env python3
"""
Convert .mpcpattern (JSON) to standard MIDI file

This validates our understanding of the .mpcpattern format by converting it
back to MIDI and comparing with originals.
"""

import json
import sys
from pathlib import Path
from midiutil import MIDIFile

def mpcpattern_to_midi(mpcpattern_path, midi_output_path):
    """Convert .mpcpattern JSON to MIDI file"""

    # Read .mpcpattern file
    with open(mpcpattern_path, 'r') as f:
        data = json.load(f)

    pattern = data['pattern']
    events = pattern['events']

    # Create MIDI file (1 track)
    midi = MIDIFile(1)
    track = 0
    channel = 0
    time = 0

    # Set tempo (120 BPM default)
    tempo = 120
    midi.addTempo(track, time, tempo)

    # PPQN (pulses per quarter note) - MPC uses 480
    # MIDIFile uses beats, so we need to convert ticks to beats
    ppqn = 480

    print(f"\nConverting: {mpcpattern_path.name}")
    print(f"Total events: {len(events)}")

    # Separate events by type
    type1_events = [e for e in events if e.get('type') == 1]
    type2_events = [e for e in events if e.get('type') == 2]

    print(f"  Type 1 (note off): {len(type1_events)}")
    print(f"  Type 2 (note on):  {len(type2_events)}")

    # Process Type 2 events (Note On with duration)
    for event in type2_events:
        note_num = event['1']
        velocity_normalized = event['2']
        time_ticks = event['time']
        duration_ticks = event['len']

        # Convert normalized velocity (0.0-1.0) back to MIDI (0-127)
        velocity = int(velocity_normalized * 127)

        # Convert ticks to beats (quarter notes)
        time_beats = time_ticks / ppqn
        duration_beats = duration_ticks / ppqn

        # Add note to MIDI
        midi.addNote(
            track=track,
            channel=channel,
            pitch=note_num,
            time=time_beats,
            duration=duration_beats,
            volume=velocity
        )

    print(f"  Added {len(type2_events)} notes")

    # Write MIDI file
    with open(midi_output_path, 'wb') as f:
        midi.writeFile(f)

    print(f"  ✓ Saved: {midi_output_path.name}")

    return len(type2_events)

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 mpcpattern_to_midi.py <file.mpcpattern> [output.mid]")
        print("   or: python3 mpcpattern_to_midi.py --batch <input_dir> <output_dir>")
        sys.exit(1)

    if sys.argv[1] == '--batch':
        if len(sys.argv) < 4:
            print("Batch mode requires input and output directories")
            sys.exit(1)

        input_dir = Path(sys.argv[2])
        output_dir = Path(sys.argv[3])

        print(f"Batch converting .mpcpattern files...")
        print(f"  Input:  {input_dir}")
        print(f"  Output: {output_dir}")
        print()

        output_dir.mkdir(parents=True, exist_ok=True)

        # Find all .mpcpattern files
        pattern_files = list(input_dir.rglob('*.mpcpattern'))
        print(f"Found {len(pattern_files)} .mpcpattern files\n")

        total_notes = 0
        for i, pattern_file in enumerate(pattern_files):
            output_file = output_dir / f"{pattern_file.stem}.mid"

            try:
                notes = mpcpattern_to_midi(pattern_file, output_file)
                total_notes += notes
            except Exception as e:
                print(f"  ✗ Error: {e}")

        print(f"\n✓ Batch conversion complete: {len(pattern_files)} files, {total_notes} total notes")

    else:
        # Single file mode
        input_path = Path(sys.argv[1])

        if len(sys.argv) >= 3:
            output_path = Path(sys.argv[2])
        else:
            output_path = input_path.with_suffix('.mid')

        mpcpattern_to_midi(input_path, output_path)

if __name__ == '__main__':
    main()
