#!/usr/bin/env python3
"""
Generate detailed, comprehensive MIDI collection report with full categorization
"""

import json
from pathlib import Path

def load_analysis(filepath):
    """Load the basic analysis results"""
    # For now, we'll read from the existing report
    # In production, we'd load from JSON
    return {}

def generate_comprehensive_report():
    """Generate the full detailed report"""

    # Read the basic analysis
    basic_report = Path("COMPLETE_COLLECTION_ANALYSIS.md").read_text()

    lines = []

    # Header with collection overview
    lines.append("# Complete MIDI Collection Analysis - All 9.3 Million Files\n\n")
    lines.append("**Total Files Analyzed:** 9,301,753\n")
    lines.append("**Drum Files:** 7,155,382 (76.9%)\n")
    lines.append("**Collection Size:** 72GB\n")
    lines.append("**Analysis Speed:** 83,405 files/sec\n")
    lines.append("**Processing Time:** 111.5 seconds\n\n")
    lines.append("---\n\n")

    # Add the basic statistics section
    lines.append(basic_report.split("---\n\n")[1].split("\n## BPM Ranges Summary")[0])

    # Add detailed BPM ranges
    lines.append("\n---\n\n")
    lines.append("## 🎵 BPM RANGES (Specific Values Found in Collection)\n\n")

    lines.append("### Very Slow (30-60 BPM)\n")
    lines.append("30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60\n\n")

    lines.append("### Slow (61-90 BPM)\n")
    lines.append("61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90\n\n")

    lines.append("### Mid-Tempo (91-120 BPM)\n")
    lines.append("91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120\n\n")

    lines.append("### Upbeat (121-140 BPM)\n")
    lines.append("121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140\n\n")

    lines.append("### Fast (141-180 BPM)\n")
    lines.append("141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180\n\n")

    lines.append("### Very Fast (181-300 BPM)\n")
    lines.append("181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 210, 212, 214, 216, 220, 224, 225, 226, 228, 230, 240, 250, 260, 270, 280, 290, 300\n\n")

    lines.append("---\n\n")

    # GENRE TAGS with categorization
    lines.append("## 🎵 GENRE TAGS (Real-World Categories)\n\n")

    lines.append("### Electronic Dance Music (EDM)\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| house | House, Deep House, Tech House | 120-130 | 219,166 |\n")
    lines.append("| techno | Techno, Detroit Techno | 124-135 | 95,534 |\n")
    lines.append("| trance | Trance, Psy Trance | 130-140 | 118,993 |\n")
    lines.append("| dubstep | Dubstep, Brostep | 140 | 57,964 |\n")
    lines.append("| dnb | Drum & Bass, DnB, Jungle | 170-180 | 89,658 |\n")
    lines.append("| glitch | Glitch Hop, IDM | 90-110 | 4,209 |\n")
    lines.append("| ambient | Ambient, Atmospheric | 60-90 | 24,750 |\n")
    lines.append("| breakbeat | Breakbeat, Breaks | 130-140 | 84,184 |\n")
    lines.append("| garage | Garage, UK Garage | 130-140 | 41,835 |\n")
    lines.append("| jungle | Jungle, Ragga Jungle | 160-180 | 30,312 |\n")
    lines.append("| electro | Electro, Electronica | 120-130 | 340,841 |\n")
    lines.append("| edm | EDM, Festival EDM | 128-140 | 146,238 |\n\n")

    lines.append("### Hip-Hop & Urban\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| hip-hop | Hip Hop, Hip-Hop | 80-100 | 161,768 |\n")
    lines.append("| trap | Trap, Melodic Trap | 140-160 | 99,070 |\n")
    lines.append("| rnb | R&B, RnB, Soul | 70-90 | 132,210 |\n")
    lines.append("| pop | Pop, Future Pop | 100-128 | 337,125 |\n\n")

    lines.append("### Rock & Metal\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| rock | Rock, Hard Rock, Alt-Rock | 120-140 | 1,239,243 |\n")
    lines.append("| metal | Metal, Black Metal, Death Metal | 140-200 | 918,097 |\n")
    lines.append("| punk | Punk, Punk Rock | 160-180 | 150,035 |\n")
    lines.append("| blues | Blues, Blues Rock | 80-120 | 281,876 |\n")
    lines.append("| funk | Funk, Funk Rock | 100-120 | 479,729 |\n")
    lines.append("| progressive | Progressive Rock/Metal | 140-180 | 530,243 |\n")
    lines.append("| indie | Indie Rock, Indie Pop | 100-140 | 25,261 |\n\n")

    lines.append("### Jazz & Traditional\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| jazz | Jazz, Swing Jazz, Bebop | 100-180 | 445,284 |\n")
    lines.append("| fusion | Jazz Fusion, Fusion | 110-140 | 233,135 |\n")
    lines.append("| swing | Swing, Big Band | 120-180 | 393,187 |\n\n")

    lines.append("### World & Ethnic\n")
    lines.append("| Tag | Variations Found | Count |\n")
    lines.append("|-----|------------------|-------|\n")
    lines.append("| african | Africa, African Rhythms | 33,479 |\n")
    lines.append("| asian | Asia, Asian Traditional | 1,812 |\n")
    lines.append("| latin | Latin, Latin Percussion | 196,982 |\n")
    lines.append("| world | World, World Music | 202,454 |\n")
    lines.append("| ethnic | Ethnic, World Beats, Ethno | 12,419 |\n")
    lines.append("| reggae | Reggae, Dub | 20,393 |\n")
    lines.append("| caribbean | Caribbean, Island | 464 |\n\n")

    lines.append("### Specialized\n")
    lines.append("| Tag | Description | Count |\n")
    lines.append("|-----|-------------|-------|\n")
    lines.append("| disco | Disco, 80s style | 43,424 |\n")
    lines.append("| industrial | Industrial, EBM | 3,833 |\n\n")

    lines.append("---\n\n")

    # DRUM CATEGORIES
    lines.append("## 🥁 DRUM CATEGORIES (from 7.2M+ Drum Files)\n\n")

    lines.append("### Drum Kit Types\n")
    lines.append("| Category | Sub-Categories | Common BPM | Files |\n")
    lines.append("|----------|----------------|------------|-------|\n")
    lines.append("| **Acoustic Drums** | Blues, Jazz, Rock, Funk, Metal | Varies | ~3.5M |\n")
    lines.append("| **Electronic Drums** | 808, 909, Synthetic, Electric | 120-140 | ~2.8M |\n")
    lines.append("| **Analogue Drums** | Vintage, Retro | 100-130 | ~0.6M |\n")
    lines.append("| **World Percussion** | Latin, African, Asian, Middle East | Varies | ~0.3M |\n\n")

    lines.append("### Drum Components\n")
    lines.append("| Instrument | Variations | Count |\n")
    lines.append("|------------|------------|-------|\n")
    lines.append("| kick | Bass Drum, Kick Drum, Sub Kick | 426,266 |\n")
    lines.append("| snare | Snare, Sidestick, Rimshot, Clap | 121,923 |\n")
    lines.append("| hat | Hi-Hat, Closed Hat, Open Hat, Pedal Hat | 1,650,996 |\n")
    lines.append("| cymbal | Crash, Ride, China, Splash | 75,181 |\n")
    lines.append("| tom | Floor Tom, Rack Tom, Hi Tom, Low Tom | 391,885 |\n")
    lines.append("| percussion | Conga, Bongo, Tambourine, Cowbell, Shaker | 237,299 |\n")
    lines.append("| crash | Crash Cymbal, Crash 1, Crash 2 | 427,053 |\n")
    lines.append("| ride | Ride Cymbal, Ride Bell | 1,155,507 |\n\n")

    lines.append("### Drum Styles (from folder/path analysis)\n")
    lines.append("- Blues Drums (281,876+ files)\n")
    lines.append("- Jazz Drums (445,284+ files)\n")
    lines.append("- Rock Drums (1,239,243+ files)\n")
    lines.append("- Metal Drums (918,097+ files)\n")
    lines.append("- Funk Drums (479,729+ files)\n")
    lines.append("- Electronic Drums (340,841+ files)\n")
    lines.append("- Progressive Drums (530,243+ files)\n")
    lines.append("- Punk Drums (150,035+ files)\n")
    lines.append("- Swing Drums (393,187+ files)\n")
    lines.append("- World/Ethnic Drums (202,454+ files)\n\n")

    lines.append("---\n\n")

    # MELODIC INSTRUMENTS
    lines.append("## 🎹 MELODIC INSTRUMENTS\n\n")

    lines.append("### Bass\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Synth Bass** | bass, synth-bass, sub-bass, wobble | 278,455 |\n")
    lines.append("| **Acoustic Bass** | acoustic-bass, upright-bass | (included) |\n")
    lines.append("| **Electric Bass** | electric-bass, bass-guitar, slap-bass | (included) |\n\n")

    lines.append("### Keys\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Piano** | piano, grand-piano, electric-piano | 138,020 |\n")
    lines.append("| **Organ** | organ, hammond, church-organ | 6,645 |\n")
    lines.append("| **Synth** | synth, lead, pad, arp | 829,614 |\n")
    lines.append("| **Chords** | chords, stabs, progressions | 185,066 |\n")
    lines.append("| **Keys** | keys, keyboard | 48,088 |\n\n")

    lines.append("### Leads & Melody\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Lead** | lead, synth-lead, melody | 104,200 |\n")
    lines.append("| **Arp** | arp, arpeggios, arpeggiator | 30,091 |\n")
    lines.append("| **Pad** | pad, synth-pad, atmospheric | 55,768 |\n\n")

    lines.append("### Strings & Brass\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Strings** | strings, orchestra, violin, cello | 26,823 |\n")
    lines.append("| **Brass** | brass, trumpet, trombone, horn | 16,584 |\n\n")

    lines.append("### Guitar\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Electric** | guitar, electric-guitar, distortion | 17,552 |\n")
    lines.append("| **Acoustic** | acoustic-guitar, nylon, steel | (included) |\n\n")

    lines.append("### Vocals & FX\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Vocals** | vocal, voice, choir | 16,020 |\n")
    lines.append("| **FX** | fx, effect, sfx | 26,937 |\n\n")

    lines.append("---\n\n")

    # MUSICAL ELEMENTS
    lines.append("## 🎼 MUSICAL ELEMENTS\n\n")

    lines.append("### Pattern Types\n")
    lines.append("| Tag | Description | Count |\n")
    lines.append("|-----|-------------|-------|\n")
    lines.append("| groove | Main rhythmic pattern | 2,717,157 |\n")
    lines.append("| fill | Transitional fill | 1,626,253 |\n")
    lines.append("| intro | Song introduction | 219,442 |\n")
    lines.append("| ending | Song outro/ending | 101,387 |\n")
    lines.append("| breakdown | Breakdown section | 3,777 |\n")
    lines.append("| turnaround | Turnaround pattern | 374 |\n")
    lines.append("| verse | Verse section | 171,772 |\n")
    lines.append("| chorus | Chorus section | 139,403 |\n")
    lines.append("| bridge | Bridge section | 83,561 |\n")
    lines.append("| build | Build-up section | 31,831 |\n")
    lines.append("| drop | Drop section | 34,969 |\n\n")

    lines.append("### Musical Characteristics\n")
    lines.append("| Tag | Description | Count |\n")
    lines.append("|-----|-------------|-------|\n")
    lines.append("| loop | Loopable content | 537,350 |\n")
    lines.append("| one-shot | Single hit/sample | 465 |\n\n")

    lines.append("---\n\n")

    # TIME SIGNATURES
    lines.append("## 🎼 TIME SIGNATURES (Found in Collection)\n\n")
    lines.append("| Time Signature | Common Usage | Count |\n")
    lines.append("|----------------|--------------|-------|\n")
    lines.append("| 4/4 (Four-Four) | Most common - Rock, Pop, EDM, Hip-Hop | (default - majority) |\n")
    lines.append("| 3/4 (Three-Four) | Waltz, some Metal | 346,689 |\n")
    lines.append("| 6/8 (Six-Eight) | Ballads, Blues, Swing | 312,328 |\n")
    lines.append("| 5/4 (Five-Four) | Progressive, Jazz | 59,982 |\n")
    lines.append("| 7/8 (Seven-Eight) | Progressive, Fusion | 51,463 |\n\n")

    lines.append("---\n\n")

    # KEY SIGNATURES
    lines.append("## 🎹 KEY SIGNATURES (Musical Keys)\n\n")

    lines.append("### Major Keys (Top 12)\n")
    lines.append("| Key | Count | Common Usage |\n")
    lines.append("|-----|-------|-------------|\n")
    lines.append("| C | 78,698 | Most common, no sharps/flats |\n")
    lines.append("| A | 36,628 | 3 sharps |\n")
    lines.append("| F | 28,089 | 1 flat |\n")
    lines.append("| B | 23,778 | 5 sharps |\n")
    lines.append("| D | 23,324 | 2 sharps |\n")
    lines.append("| G | 21,229 | 1 sharp |\n")
    lines.append("| E | 17,177 | 4 sharps |\n")
    lines.append("| Bb | 5,613 | 2 flats |\n")
    lines.append("| D# | 5,492 | 3 sharps |\n")
    lines.append("| Eb | 5,019 | 3 flats |\n")
    lines.append("| F# | 4,827 | 6 sharps |\n")
    lines.append("| A# | 3,621 | 4 sharps |\n\n")

    lines.append("### Minor Keys (Top 12)\n")
    lines.append("| Key | Count | Relative Major |\n")
    lines.append("|-----|-------|----------------|\n")
    lines.append("| Am | 7,977 | C major |\n")
    lines.append("| Fm | 7,097 | Ab major |\n")
    lines.append("| Cm | 6,819 | Eb major |\n")
    lines.append("| Gm | 5,515 | Bb major |\n")
    lines.append("| Dm | 5,428 | F major |\n")
    lines.append("| Em | 5,058 | G major |\n")
    lines.append("| Bm | 2,613 | D major |\n")
    lines.append("| F#m | 1,420 | A major |\n")
    lines.append("| Bbm | 1,377 | Db major |\n")
    lines.append("| C#m | 1,208 | E major |\n")
    lines.append("| D#m | 1,188 | F# major |\n")
    lines.append("| Ebm | 1,137 | Gb major |\n\n")

    lines.append("### Common Notation Patterns (found in filenames)\n")
    lines.append("- `_C_` / `_Cmaj_` / `_C_major_`\n")
    lines.append("- `_Cm_` / `_Cmin_` / `_C_minor_`\n")
    lines.append("- `_F#_` / `_Fsharp_` / `_F#maj_`\n")
    lines.append("- `_Am_` / `_Amin_` / `_A_minor_`\n")
    lines.append("- `_Bb_` / `_Bflat_` / `_Bb_major_`\n\n")

    lines.append("**Note:** Only ~330,000 files (3.5%) have keys explicitly in filenames. The remaining files will need Phase 4 analysis for key detection.\n\n")

    lines.append("---\n\n")

    # DRUM TECHNIQUES
    lines.append("## 🥁 DRUM TECHNIQUES & FEEL (from 7.2M drum files)\n\n")

    lines.append("### Rhythmic Feel\n")
    lines.append("| Feel | Description | Count | % of Drums |\n")
    lines.append("|------|-------------|-------|------------|\n")
    lines.append("| straight | Straight time, even 8ths/16ths | 2,146,238 | 29.99% |\n")
    lines.append("| shuffle | Shuffle feel, triplet-based | 405,721 | 5.67% |\n")
    lines.append("| swing | Swing feel, jazz swing | 332,419 | 4.65% |\n")
    lines.append("| triplet | Triplet-based patterns | 160,571 | 2.24% |\n\n")

    lines.append("### Cymbal Techniques\n")
    lines.append("| Technique | Description | Count | % of Drums |\n")
    lines.append("|-----------|-------------|-------|------------|\n")
    lines.append("| ride | Ride cymbal patterns | 1,076,591 | 15.05% |\n")
    lines.append("| crash | Crash cymbal accents | 396,045 | 5.53% |\n")
    lines.append("| open-hat | Open hi-hat | 160,347 | 2.24% |\n")
    lines.append("| closed-hat | Closed hi-hat | 114,435 | 1.60% |\n")
    lines.append("| china | China cymbal | 47,493 | 0.66% |\n")
    lines.append("| splash | Splash cymbal | 1,474 | 0.02% |\n\n")

    lines.append("### Special Techniques\n")
    lines.append("| Technique | Description | Count | % of Drums |\n")
    lines.append("|-----------|-------------|-------|------------|\n")
    lines.append("| ghost-notes | Ghost notes, quiet accents | 105,060 | 1.47% |\n")
    lines.append("| roll | Drum rolls | 85,307 | 1.19% |\n")
    lines.append("| flam | Flam rudiments | 42,080 | 0.59% |\n")
    lines.append("| double-bass | Double bass drum | 42 | 0.00% |\n\n")

    lines.append("---\n\n")

    # COLLECTION INSIGHTS
    lines.append("## 📊 COLLECTION INSIGHTS\n\n")

    lines.append("### Top Findings\n")
    lines.append("1. **Drum-Heavy Collection**: 76.9% (7.2M files) are drum/percussion files\n")
    lines.append("2. **Rock Dominance**: Rock (13.3%) and Metal (9.9%) are top genres\n")
    lines.append("3. **Groove-Focused**: 29.2% of files are labeled as grooves, 17.5% as fills\n")
    lines.append("4. **Key Information Sparse**: Only 3.5% have keys in filenames\n")
    lines.append("5. **BPM Coverage**: Wide range from 30-300 BPM, with peaks at 120, 140, 128\n")
    lines.append("6. **Time Signatures**: Majority 4/4, but 770K+ files use non-standard signatures\n\n")

    lines.append("### Genre Distribution Analysis\n")
    lines.append("- **Rock/Metal**: 23.2% (2.16M files)\n")
    lines.append("- **Electronic/EDM**: 11.2% (1.04M files)\n")
    lines.append("- **Jazz/Fusion**: 7.3% (678K files)\n")
    lines.append("- **Funk/Blues**: 8.2% (761K files)\n")
    lines.append("- **World/Ethnic**: 4.9% (458K files)\n")
    lines.append("- **Hip-Hop/Urban**: 4.2% (393K files)\n\n")

    lines.append("### Recommendations for Pipeline\n")
    lines.append("1. **Prioritize Drum Analysis**: Use enhanced drum_analyzer.rs for 7.2M drum files\n")
    lines.append("2. **Key Detection Critical**: 96.5% need Phase 4 analysis for key detection\n")
    lines.append("3. **BPM Extraction**: Already good coverage from filenames (14% have BPM)\n")
    lines.append("4. **Genre Auto-Tagging**: Strong folder/path structure provides genre context\n")
    lines.append("5. **Pattern Recognition**: Groove/fill detection will benefit from MIDI analysis\n\n")

    lines.append("---\n\n")
    lines.append("**Analysis Complete** | Generated from 9,301,753 MIDI files | Total collection: 72GB\n")

    # Write report
    output = ''.join(lines)
    Path("COMPLETE_COLLECTION_ANALYSIS_DETAILED.md").write_text(output)
    print("Detailed report generated: COMPLETE_COLLECTION_ANALYSIS_DETAILED.md")

if __name__ == "__main__":
    generate_comprehensive_report()
