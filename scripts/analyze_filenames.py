#!/usr/bin/env python3
"""
Analyze MIDI filenames to extract instruments, genres, keys, BPM, and descriptions
"""

import re
import sys
import json
from pathlib import Path
from collections import Counter, defaultdict
from concurrent.futures import ProcessPoolExecutor
import multiprocessing as mp

class FilenameAnalyzer:
    def __init__(self):
        # Counters for all metadata
        self.instruments = Counter()
        self.genres = Counter()
        self.keys = Counter()
        self.bpms = Counter()
        self.patterns = Counter()
        self.time_signatures = Counter()
        self.drum_elements = Counter()

        self.total_files = 0
        self.drum_files = 0

    def merge(self, other):
        """Merge results from another analyzer"""
        self.instruments.update(other.instruments)
        self.genres.update(other.genres)
        self.keys.update(other.keys)
        self.bpms.update(other.bpms)
        self.patterns.update(other.patterns)
        self.time_signatures.update(other.time_signatures)
        self.drum_elements.update(other.drum_elements)
        self.total_files += other.total_files
        self.drum_files += other.drum_files

def extract_bpm(filename):
    """Extract BPM from filename"""
    # Pattern 1: _120bpm_ or _120_bpm_
    match = re.search(r'[_\-](\d{2,3})[-_]?bpm[_\-]', filename, re.IGNORECASE)
    if match:
        bpm = int(match.group(1))
        if 30 <= bpm <= 300:
            return bpm

    # Pattern 2: _120_ (number between underscores)
    match = re.search(r'[_\-](\d{2,3})[_\-]', filename)
    if match:
        bpm = int(match.group(1))
        if 30 <= bpm <= 300:
            return bpm

    # Pattern 3: starts with number
    match = re.search(r'^(\d{2,3})[-_\.]', filename)
    if match:
        bpm = int(match.group(1))
        if 30 <= bpm <= 300:
            return bpm

    return None

def extract_key(filename):
    """Extract musical key from filename"""
    # Major keys: C, C#, Db, etc.
    match = re.search(r'[_\-]([A-G][#b]?)(?:maj|major)?[_\-]', filename, re.IGNORECASE)
    if match:
        return match.group(1).upper()

    # Minor keys: Cm, C#m, Dbm, Amin, etc.
    match = re.search(r'[_\-]([A-G][#b]?)m(?:in|inor)?[_\-]', filename, re.IGNORECASE)
    if match:
        return match.group(1).upper() + 'm'

    return None

def extract_time_signature(filename):
    """Extract time signature from filename"""
    lower = filename.lower()

    if 'threefour' in lower or '3-4' in lower or '3_4' in lower:
        return '3/4'
    if 'sixeight' in lower or '6-8' in lower or '6_8' in lower:
        return '6/8'
    if 'fivefour' in lower or '5-4' in lower or '5_4' in lower:
        return '5/4'
    if 'seveneight' in lower or '7-8' in lower or '7_8' in lower:
        return '7/8'

    return None

def extract_instruments(filename, path):
    """Extract instrument names from filename and path"""
    text = (filename + ' ' + path).lower()
    instruments = []

    # Drums
    if 'drum' in text and 'syndrome' not in text:
        instruments.append('drums')
    if 'percussion' in text:
        instruments.append('percussion')
    if 'snare' in text:
        instruments.append('snare')
    if 'kick' in text:
        instruments.append('kick')
    if 'hat' in text or 'hihat' in text or 'hi-hat' in text:
        instruments.append('hat')
    if 'cymbal' in text:
        instruments.append('cymbal')
    if 'tom' in text and 'atom' not in text and 'custom' not in text and 'bottom' not in text:
        instruments.append('tom')
    if 'ride' in text:
        instruments.append('ride')
    if 'crash' in text:
        instruments.append('crash')

    # Bass
    if 'bass' in text and 'bass drum' not in text and 'double bass' not in text:
        instruments.append('bass')

    # Synth
    if 'synth' in text:
        instruments.append('synth')
    if 'pad' in text and 'ipad' not in text:
        instruments.append('pad')
    if 'lead' in text:
        instruments.append('lead')
    if 'arp' in text and 'sharp' not in text and 'harp' not in text:
        instruments.append('arp')

    # Keys
    if 'piano' in text:
        instruments.append('piano')
    if 'organ' in text and 'organic' not in text and 'organize' not in text:
        instruments.append('organ')
    if 'chord' in text:
        instruments.append('chords')
    if 'keys' in text or 'keyboard' in text:
        instruments.append('keys')

    # Strings & Brass
    if 'string' in text:
        instruments.append('strings')
    if 'brass' in text:
        instruments.append('brass')
    if 'violin' in text or 'viola' in text or 'cello' in text:
        instruments.append('strings')
    if 'trumpet' in text or 'trombone' in text or 'horn' in text:
        instruments.append('brass')

    # Guitar
    if 'guitar' in text:
        instruments.append('guitar')

    # Vocals
    if 'vocal' in text or 'voice' in text or 'choir' in text:
        instruments.append('vocals')

    # FX
    if 'fx' in text or 'effect' in text or 'sfx' in text:
        instruments.append('fx')

    return instruments

def extract_genres(filename, path):
    """Extract genres from filename and path"""
    text = (filename + ' ' + path).lower()
    genres = []

    # Electronic
    if 'house' in text:
        genres.append('house')
    if 'techno' in text:
        genres.append('techno')
    if 'trance' in text:
        genres.append('trance')
    if 'dubstep' in text:
        genres.append('dubstep')
    if 'dnb' in text or ('drum' in text and 'bass' in text and 'drum bass' not in text):
        genres.append('dnb')
    if 'jungle' in text:
        genres.append('jungle')
    if 'breakbeat' in text or 'breaks' in text:
        genres.append('breakbeat')
    if 'garage' in text:
        genres.append('garage')
    if 'glitch' in text:
        genres.append('glitch')
    if 'ambient' in text:
        genres.append('ambient')
    if 'edm' in text:
        genres.append('edm')
    if 'electro' in text:
        genres.append('electro')

    # Hip-Hop & Urban
    if 'hiphop' in text or 'hip-hop' in text or 'hip hop' in text:
        genres.append('hip-hop')
    if 'trap' in text:
        genres.append('trap')
    if 'rnb' in text or 'r&b' in text or 'r and b' in text:
        genres.append('rnb')

    # Rock & Metal
    if 'rock' in text:
        genres.append('rock')
    if 'metal' in text:
        genres.append('metal')
    if 'punk' in text:
        genres.append('punk')
    if 'blues' in text:
        genres.append('blues')
    if 'funk' in text and 'funky' not in text:
        genres.append('funk')

    # Jazz
    if 'jazz' in text:
        genres.append('jazz')
    if 'fusion' in text:
        genres.append('fusion')
    if 'swing' in text:
        genres.append('swing')

    # World
    if 'latin' in text:
        genres.append('latin')
    if 'africa' in text:
        genres.append('african')
    if 'asia' in text:
        genres.append('asian')
    if 'world' in text:
        genres.append('world')
    if 'ethnic' in text or 'ethno' in text:
        genres.append('ethnic')
    if 'reggae' in text:
        genres.append('reggae')
    if 'caribbean' in text:
        genres.append('caribbean')

    # Other
    if 'pop' in text:
        genres.append('pop')
    if 'disco' in text:
        genres.append('disco')
    if 'progressive' in text:
        genres.append('progressive')
    if 'industrial' in text:
        genres.append('industrial')
    if 'indie' in text:
        genres.append('indie')

    return genres

def extract_patterns(filename, path):
    """Extract pattern types from filename and path"""
    text = (filename + ' ' + path).lower()
    patterns = []

    if 'fill' in text:
        patterns.append('fill')
    if 'groove' in text:
        patterns.append('groove')
    if 'intro' in text:
        patterns.append('intro')
    if 'outro' in text or 'ending' in text:
        patterns.append('ending')
    if 'breakdown' in text:
        patterns.append('breakdown')
    if 'turnaround' in text:
        patterns.append('turnaround')
    if 'verse' in text:
        patterns.append('verse')
    if 'chorus' in text:
        patterns.append('chorus')
    if 'bridge' in text:
        patterns.append('bridge')
    if 'loop' in text:
        patterns.append('loop')
    if 'one-shot' in text or 'oneshot' in text or 'one shot' in text:
        patterns.append('one-shot')
    if 'build' in text and 'building' not in text:
        patterns.append('build')
    if 'drop' in text:
        patterns.append('drop')

    return patterns

def extract_drum_elements(filename, path):
    """Extract drum-specific elements"""
    text = (filename + ' ' + path).lower()
    elements = []

    # Cymbals
    if 'crash' in text:
        elements.append('crash')
    if 'ride' in text:
        elements.append('ride')
    if 'china' in text:
        elements.append('china')
    if 'splash' in text:
        elements.append('splash')

    # Hi-hats
    if 'closed' in text and ('hat' in text or 'hihat' in text):
        elements.append('closed-hat')
    if 'open' in text and ('hat' in text or 'hihat' in text):
        elements.append('open-hat')
    if 'pedal' in text and ('hat' in text or 'hihat' in text):
        elements.append('pedal-hat')

    # Techniques
    if 'ghost' in text:
        elements.append('ghost-notes')
    if 'double' in text and 'bass' in text:
        elements.append('double-bass')
    if 'flam' in text:
        elements.append('flam')
    if 'roll' in text:
        elements.append('roll')

    # Feel
    if 'swing' in text:
        elements.append('swing')
    if 'shuffle' in text:
        elements.append('shuffle')
    if 'triplet' in text:
        elements.append('triplet')
    if 'straight' in text:
        elements.append('straight')

    return elements

def analyze_file(filepath):
    """Analyze a single file"""
    analyzer = FilenameAnalyzer()
    analyzer.total_files = 1

    path = filepath
    filename = Path(filepath).name.lower()

    # Extract BPM
    bpm = extract_bpm(filename)
    if bpm:
        analyzer.bpms[bpm] += 1

    # Extract key
    key = extract_key(filename)
    if key:
        analyzer.keys[key] += 1

    # Extract time signature
    ts = extract_time_signature(filename)
    if ts:
        analyzer.time_signatures[ts] += 1

    # Extract instruments
    instruments = extract_instruments(filename, path)
    for inst in instruments:
        analyzer.instruments[inst] += 1

    # Extract genres
    genres = extract_genres(filename, path)
    for genre in genres:
        analyzer.genres[genre] += 1

    # Extract patterns
    patterns = extract_patterns(filename, path)
    for pattern in patterns:
        analyzer.patterns[pattern] += 1

    # Check if drum file
    is_drum = any(inst in ['drums', 'percussion', 'snare', 'kick', 'hat'] for inst in instruments)
    if is_drum:
        analyzer.drum_files = 1
        elements = extract_drum_elements(filename, path)
        for elem in elements:
            analyzer.drum_elements[elem] += 1

    return analyzer

def process_batch(filepaths):
    """Process a batch of files"""
    batch_analyzer = FilenameAnalyzer()
    for filepath in filepaths:
        result = analyze_file(filepath)
        batch_analyzer.merge(result)
    return batch_analyzer

def generate_report(analyzer, output_path):
    """Generate markdown report"""
    total = analyzer.total_files

    lines = []
    lines.append("# Complete MIDI Collection Analysis\n")
    lines.append(f"**Total Files Analyzed:** {total:,}\n")
    lines.append(f"**Drum Files:** {analyzer.drum_files:,} ({analyzer.drum_files/total*100:.1f}%)\n")
    lines.append("\n---\n\n")

    # ALL Instruments (no limit)
    lines.append("## ALL Instruments Found\n\n")
    lines.append("| Instrument | Count | Percentage |\n")
    lines.append("|------------|-------|------------|\n")
    for inst, count in analyzer.instruments.most_common():
        lines.append(f"| {inst} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL Genres (no limit)
    lines.append("## ALL Genres Found\n\n")
    lines.append("| Genre | Count | Percentage |\n")
    lines.append("|-------|-------|------------|\n")
    for genre, count in analyzer.genres.most_common():
        lines.append(f"| {genre} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL Patterns (no limit)
    lines.append("## ALL Pattern Types\n\n")
    lines.append("| Pattern | Count | Percentage |\n")
    lines.append("|---------|-------|------------|\n")
    for pattern, count in analyzer.patterns.most_common():
        lines.append(f"| {pattern} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL Musical Keys (no limit)
    lines.append("## ALL Musical Keys Found\n\n")
    lines.append("| Key | Count | Percentage |\n")
    lines.append("|-----|-------|------------|\n")
    for key, count in analyzer.keys.most_common():
        lines.append(f"| {key} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL BPM Values (no limit)
    lines.append("## ALL BPM Values Found\n\n")
    lines.append("| BPM | Count | Percentage |\n")
    lines.append("|-----|-------|------------|\n")
    for bpm, count in analyzer.bpms.most_common():
        lines.append(f"| {bpm} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # BPM Ranges
    lines.append("## BPM Ranges Summary\n\n")
    bpm_ranges = defaultdict(int)
    for bpm, count in analyzer.bpms.items():
        if 30 <= bpm <= 60:
            bpm_ranges['Very Slow (30-60)'] += count
        elif 61 <= bpm <= 90:
            bpm_ranges['Slow (61-90)'] += count
        elif 91 <= bpm <= 120:
            bpm_ranges['Mid-Tempo (91-120)'] += count
        elif 121 <= bpm <= 140:
            bpm_ranges['Upbeat (121-140)'] += count
        elif 141 <= bpm <= 180:
            bpm_ranges['Fast (141-180)'] += count
        elif 181 <= bpm <= 300:
            bpm_ranges['Very Fast (181-300)'] += count

    lines.append("| BPM Range | Count | Percentage |\n")
    lines.append("|-----------|-------|------------|\n")
    for range_name, count in sorted(bpm_ranges.items(), key=lambda x: -x[1]):
        lines.append(f"| {range_name} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # Time Signatures
    if analyzer.time_signatures:
        lines.append("## Time Signatures Found\n\n")
        lines.append("| Time Signature | Count | Percentage |\n")
        lines.append("|----------------|-------|------------|\n")
        for ts, count in analyzer.time_signatures.most_common():
            lines.append(f"| {ts} | {count:,} | {count/total*100:.2f}% |\n")
        lines.append("\n")

    # ALL Drum Elements (no limit)
    if analyzer.drum_elements:
        lines.append("## ALL Drum Elements & Techniques\n\n")
        lines.append("| Element | Count | Percentage (of drum files) |\n")
        lines.append("|---------|-------|---------------------------|\n")
        drum_total = analyzer.drum_files
        for elem, count in analyzer.drum_elements.most_common():
            lines.append(f"| {elem} | {count:,} | {count/drum_total*100:.2f}% |\n")
        lines.append("\n")

    # Write to file
    with open(output_path, 'w') as f:
        f.writelines(lines)

def main():
    import time

    print("MIDI Filename Analysis")
    print("=" * 50)

    # Read file list from stdin or find files
    if not sys.stdin.isatty():
        print("Reading filenames from stdin...")
        filepaths = [line.strip() for line in sys.stdin if line.strip()]
    else:
        print("Error: Please pipe file list to this script")
        print("Usage: find /path -name '*.mid' | python3 analyze_filenames.py")
        sys.exit(1)

    total_files = len(filepaths)
    print(f"Total files to analyze: {total_files:,}")

    # Determine number of workers
    num_workers = mp.cpu_count()
    print(f"Using {num_workers} parallel workers\n")

    # Split into batches
    batch_size = max(1, total_files // (num_workers * 10))
    batches = [filepaths[i:i+batch_size] for i in range(0, total_files, batch_size)]
    print(f"Processing in {len(batches)} batches of ~{batch_size} files each\n")

    # Process in parallel
    start_time = time.time()
    main_analyzer = FilenameAnalyzer()

    with ProcessPoolExecutor(max_workers=num_workers) as executor:
        futures = [executor.submit(process_batch, batch) for batch in batches]

        completed = 0
        for future in futures:
            result = future.result()
            main_analyzer.merge(result)
            completed += 1

            if completed % 10 == 0 or completed == len(futures):
                elapsed = time.time() - start_time
                rate = main_analyzer.total_files / elapsed
                pct = completed / len(futures) * 100
                print(f"Progress: {pct:.1f}% ({main_analyzer.total_files:,}/{total_files:,} files) - {rate:.0f} files/sec")

    elapsed = time.time() - start_time
    rate = total_files / elapsed

    print(f"\nAnalysis complete!")
    print(f"Total files: {total_files:,}")
    print(f"Time: {elapsed:.1f}s")
    print(f"Rate: {rate:.0f} files/sec")
    print()

    # Generate report
    output_path = "COMPLETE_COLLECTION_ANALYSIS.md"
    print(f"Generating report to {output_path}...")
    generate_report(main_analyzer, output_path)
    print(f"Done! Report saved to {output_path}")

    # Save complete JSON data for comprehensive reporting
    json_path = "COMPLETE_COLLECTION_ANALYSIS.json"
    print(f"\nSaving complete JSON data to {json_path}...")
    json_data = {
        'total_files': main_analyzer.total_files,
        'drum_files': main_analyzer.drum_files,
        'instruments': dict(main_analyzer.instruments.most_common()),
        'genres': dict(main_analyzer.genres.most_common()),
        'patterns': dict(main_analyzer.patterns.most_common()),
        'keys': dict(main_analyzer.keys.most_common()),
        'bpms': dict(sorted(main_analyzer.bpms.items())),
        'time_signatures': dict(main_analyzer.time_signatures.most_common()),
        'drum_elements': dict(main_analyzer.drum_elements.most_common()),
    }

    with open(json_path, 'w') as f:
        json.dump(json_data, f, indent=2)

    print(f"Complete JSON data saved! ({len(json_data['instruments'])} instruments, {len(json_data['genres'])} genres, {len(json_data['bpms'])} BPM values)")

if __name__ == "__main__":
    main()
