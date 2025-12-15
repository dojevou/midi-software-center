#!/usr/bin/env python3
"""
VIP3 Category Re-tagger
=======================
Re-processes all MIDI files to extract the 8 VIP3 categories:
1. Instrument - drums, bass, keys, guitar, brass, synth, strings, woodwind, vocal, orchestral
2. Genre - rock, jazz, house, techno, hip-hop, etc.
3. BPM - from musical_metadata (already extracted)
4. Key - from musical_metadata (already extracted)
5. Type - loop, fill, groove, progression, melody, chord, one-shot, riff
6. Mood - happy, dark, bright, energetic, chill, etc.
7. Time Signature - from musical_metadata (already extracted)
8. Duration - from files table (already extracted)

This script focuses on improving extraction for:
- Instrument (38% -> 90%)
- Genre (17% -> 70%)
- Type/Pattern (11% -> 80%)
- Mood (0% -> 50%)

Features:
- Spelling normalization (kicks -> kick, hihats -> hihat)
- Fuzzy matching with Levenshtein distance
- Comprehensive keyword dictionaries

Usage:
    python scripts/retag-vip3-categories.py [--batch-size 1000] [--dry-run]
"""

import argparse
import os
import re
import sys
from collections import defaultdict
from multiprocessing import Pool, cpu_count
from pathlib import Path
from typing import Optional, Set, Dict, List, Tuple

import psycopg2
from psycopg2.extras import execute_batch

# Database connection
DB_URL = os.environ.get(
    "DATABASE_URL",
    "postgresql://midiuser:145278963@localhost:5433/midi_library"
)

# =============================================================================
# SPELLING NORMALIZATION MAPS
# =============================================================================
# Maps spelling variants to canonical names

INSTRUMENT_NORMALIZATIONS = {
    # Drums variations
    "kicks": "kick", "kik": "kick", "kck": "kick", "bd": "kick", "bassdrum": "kick",
    "snares": "snare", "sn": "snare", "sd": "snare", "snr": "snare",
    "hihats": "hihat", "hh": "hihat", "hi-hats": "hihat", "hats": "hihat",
    "hats": "hihat", "hihatclosed": "hihat", "hihatopen": "hihat",
    "claps": "clap", "clp": "clap", "handclap": "clap",
    "cymbals": "cymbal", "cym": "cymbal", "cyms": "cymbal",
    "crashes": "crash", "crsh": "crash",
    "rides": "ride", "rd": "ride",
    "toms": "tom", "tomtom": "tom", "tom-tom": "tom", "flrtom": "tom", "floortom": "tom",
    "percs": "percussion", "prcs": "percussion",
    "drm": "drums", "drms": "drums", "drumkit": "drums", "kit": "drums",
    "tambourines": "tambourine", "tamb": "tambourine",
    "shakers": "shaker", "shkr": "shaker",
    "congas": "conga", "cng": "conga",
    "bongos": "bongo", "bng": "bongo",
    "cowbells": "cowbell", "cbell": "cowbell",
    "rimshot": "rim", "rimshots": "rim",

    # Bass variations
    "basses": "bass", "bs": "bass", "bassline": "bass", "basslines": "bass",
    "subbass": "sub", "sub-bass": "sub", "subs": "sub",
    "808s": "808", "tr808": "808", "tr-808": "808",
    "909s": "909", "tr909": "909", "tr-909": "909",

    # Keys variations
    "pianos": "piano", "pno": "piano", "pian": "piano", "grand": "piano",
    "organs": "organ", "org": "organ", "b3": "organ", "hammond": "organ",
    "ep": "electric-piano", "epiano": "electric-piano", "e-piano": "electric-piano",
    "rhodes": "electric-piano", "wurly": "wurlitzer", "wurli": "wurlitzer",
    "clavs": "clav", "clavinet": "clav", "clavi": "clav",
    "harps": "harpsichord", "harpsi": "harpsichord",
    "keyboards": "keyboard", "keyb": "keyboard", "kbd": "keyboard",

    # Guitar variations
    "guitars": "guitar", "gtr": "guitar", "guit": "guitar", "gt": "guitar",
    "acousticguitar": "acoustic-guitar", "acgtr": "acoustic-guitar", "ac-guitar": "acoustic-guitar",
    "electricguitar": "electric-guitar", "elgtr": "electric-guitar", "el-guitar": "electric-guitar",
    "strums": "strum", "strumming": "strum",
    "fingerpicking": "fingerpick", "fingerpicked": "fingerpick",

    # Brass variations
    "trumpets": "trumpet", "tpt": "trumpet", "trp": "trumpet", "trpt": "trumpet",
    "trombones": "trombone", "trb": "trombone", "tbn": "trombone",
    "horns": "horn", "frenchhorn": "horn", "french-horn": "horn",
    "saxophones": "saxophone", "saxes": "saxophone", "sxp": "saxophone",
    "altosax": "saxophone", "alto-sax": "saxophone", "tenorsax": "saxophone",
    "barisax": "saxophone", "sopranosax": "saxophone",

    # Synth variations
    "synths": "synth", "syn": "synth", "synthesizer": "synth", "synthesizers": "synth",
    "leads": "lead", "ld": "lead", "leadsynth": "lead", "lead-synth": "lead",
    "pads": "pad", "pd": "pad", "padsynth": "pad", "pad-synth": "pad",
    "arps": "arp", "arpeggio": "arp", "arpeggios": "arp", "arpeggiated": "arp",
    "plucks": "pluck", "plk": "pluck", "plucked": "pluck",
    "stabs": "stab", "stb": "stab", "stabsynth": "stab",
    "sequences": "sequence", "seq": "sequence", "seqs": "sequence", "sequencer": "sequence",

    # Strings variations
    "string": "strings", "str": "strings", "strgs": "strings",
    "violins": "violin", "vln": "violin", "viol": "violin",
    "violas": "viola", "vla": "viola",
    "cellos": "cello", "vlc": "cello", "vc": "cello",
    "ensembles": "ensemble", "ens": "ensemble",

    # Woodwind variations
    "flutes": "flute", "flt": "flute", "fl": "flute",
    "clarinets": "clarinet", "clar": "clarinet", "cl": "clarinet", "clr": "clarinet",
    "oboes": "oboe", "ob": "oboe",
    "bassoons": "bassoon", "bsn": "bassoon", "fag": "bassoon",
    "woodwinds": "woodwind", "ww": "woodwind",

    # Vocal variations
    "vocals": "vocal", "voc": "vocal", "vocs": "vocal", "vx": "vocal",
    "voices": "voice", "voic": "voice",
    "choirs": "choir", "chr": "choir", "chorale": "choir",
    "chants": "chant", "chanting": "chant",
    "singing": "sing", "singer": "sing", "sung": "sing",

    # Orchestral variations
    "orch": "orchestral", "orchestras": "orchestral", "orchestrated": "orchestral",
    "cinematic": "orchestral", "film": "orchestral", "filmscore": "orchestral",
    "scores": "orchestral", "score": "orchestral", "soundtrack": "orchestral",
}

GENRE_NORMALIZATIONS = {
    # Rock variants
    "rocks": "rock", "rck": "rock", "rocker": "rock",
    "metals": "metal", "mtl": "metal", "hvy": "metal", "heavymetal": "metal",
    "punks": "punk", "pnk": "punk",

    # Jazz variants
    "jazzy": "jazz", "jz": "jazz", "jzz": "jazz",
    "swinging": "swing", "swg": "swing",
    "fusions": "fusion", "jazzfusion": "fusion", "jazz-fusion": "fusion",

    # Electronic variants
    "houses": "house", "hse": "house", "deephse": "house",
    "tech-house": "techhouse", "techhouse": "house",
    "technos": "techno", "tch": "techno", "tekno": "techno",
    "minimalist": "minimal", "min": "minimal",

    # Hip-hop variants
    "hiphop": "hip-hop", "hip_hop": "hip-hop", "hh": "hip-hop",
    "raps": "rap", "rapping": "rap", "rapper": "rap",
    "traps": "trap", "trp": "trap",
    "boombap": "boom-bap", "boom_bap": "boom-bap",

    # DnB variants
    "drum-and-bass": "dnb", "drumnbass": "dnb", "drumandbase": "dnb",
    "drumandbass": "dnb", "d&b": "dnb", "d-n-b": "dnb",
    "jungles": "jungle", "jngl": "jungle",
    "liquids": "liquid", "liq": "liquid",

    # Trance variants
    "trances": "trance", "trnc": "trance", "trn": "trance",
    "psytrances": "psytrance", "psy": "psytrance", "goa-trance": "goa",

    # Dubstep variants
    "dubsteps": "dubstep", "dub-step": "dubstep", "dbs": "dubstep",
    "brosteps": "brostep", "riddims": "riddim",

    # Other EDM variants
    "electros": "electro", "elc": "electro", "electronica": "electro",
    "electronics": "electronic", "elec": "electronic",
    "dances": "dance", "dnc": "dance",

    # Ambient variants
    "ambients": "ambient", "amb": "ambient", "ambience": "ambient",
    "atmospherics": "atmospheric", "atmo": "atmospheric", "atmos": "atmospheric",
    "soundscapes": "soundscape", "scape": "soundscape",
    "drones": "drone", "drn": "drone",

    # Other genre variants
    "funks": "funk", "fnk": "funk", "funkier": "funk",
    "souls": "soul", "soulful": "soul",
    "reggaes": "reggae", "rga": "reggae", "regge": "reggae",
    "dubs": "dub", "dubby": "dub",
    "latins": "latin", "lat": "latin",
    "lofis": "lofi", "lo-fi": "lofi", "lofy": "lofi",
    "chillhops": "chillhop", "chill-hop": "chillhop",
    "pops": "pop", "popper": "pop",
}

# =============================================================================
# VIP3 CATEGORY DEFINITIONS (EXPANDED)
# =============================================================================

# Category 1: INSTRUMENT (expanded with more keywords)
INSTRUMENT_KEYWORDS = {
    # Drums (Western) - comprehensive
    "drums": [
        "kick", "snare", "hihat", "hat", "hi-hat", "clap", "tom", "toms",
        "cymbal", "crash", "ride", "percussion", "perc", "drum", "drums",
        "cowbell", "tambourine", "shaker", "conga", "bongo", "rim", "splash",
        "china", "fill", "stick", "closed", "open", "ghost", "flam",
        "rimshot", "cross-stick", "sidestick", "brushes", "mallet",
        "triangle", "woodblock", "cabasa", "guiro", "agogo", "timbale",
        "drumloop", "breakbeat", "beat", "groove", "pattern",
        # Common abbreviations
        "bd", "sd", "hh", "oh", "ch", "cym", "prcs",
    ],
    # Bass - comprehensive
    "bass": [
        "bass", "sub", "subbass", "sub-bass", "reese", "808", "909",
        "bassline", "basslines", "lowend", "low-end", "wobble",
        "fingerbass", "slap", "picked", "fretless", "upright",
        "synbass", "synth-bass", "acid", "squelch",
        # Common abbreviations
        "bs", "bss",
    ],
    # Keys - comprehensive
    "keys": [
        "piano", "keys", "organ", "rhodes", "wurlitzer", "electric-piano",
        "clav", "harpsichord", "keyboard", "grand", "upright", "baby-grand",
        "honky-tonk", "tack", "prepared", "hammer", "b3", "hammond",
        "farfisa", "vox", "combo", "drawbar", "leslie", "rotary",
        "celesta", "celeste", "glockenspiel", "vibraphone", "marimba",
        "xylophone", "bells", "chimes", "tubular",
        # Common abbreviations
        "pno", "kbd", "ep", "org",
    ],
    # Guitar - comprehensive
    "guitar": [
        "guitar", "acoustic-guitar", "electric-guitar", "12-string",
        "slide", "muted", "strum", "fingerpick", "nylon", "steel",
        "classical", "flamenco", "folk", "country", "jazz-guitar",
        "clean", "distorted", "overdrive", "crunch", "fuzz",
        "wah", "chorus", "tremolo", "palm-mute", "power-chord",
        "arpeggiated", "picked", "plucked", "harmonics",
        "ukulele", "uke", "banjo", "mandolin", "bouzouki", "sitar",
        # Common abbreviations
        "gtr", "gt", "acgtr", "elgtr",
    ],
    # Brass - comprehensive
    "brass": [
        "brass", "trumpet", "trombone", "horn", "sax", "saxophone",
        "french-horn", "flugelhorn", "cornet", "euphonium", "tuba",
        "alto", "tenor", "baritone", "soprano", "muted", "harmon",
        "plunger", "cup", "straight", "section", "stab", "hit",
        "fanfare", "big-band", "mariachi", "ska-brass",
        # Common abbreviations
        "tpt", "trb", "sxp", "hrn",
    ],
    # Synth - comprehensive
    "synth": [
        "synth", "lead", "pad", "arp", "pluck", "stab", "sequence",
        "analog", "digital", "fm", "wavetable", "granular", "additive",
        "subtractive", "modular", "mono", "poly", "unison", "detune",
        "filter", "resonance", "cutoff", "envelope", "lfo",
        "moog", "prophet", "jupiter", "juno", "minimoog", "dx7",
        "vintage", "retro", "modern", "ambient-synth", "texture",
        "sweep", "riser", "fall", "swell", "drone",
        # Common abbreviations
        "syn", "ld", "pd", "sq",
    ],
    # Strings - comprehensive
    "strings": [
        "strings", "string", "violin", "viola", "cello", "ensemble",
        "orchestra", "orchestral", "contrabass", "double-bass",
        "chamber", "quartet", "section", "pizzicato", "arco",
        "tremolo", "spiccato", "legato", "staccato", "marcato",
        "sul-pont", "col-legno", "harmonics", "glissando",
        "harp", "zither",
        # Common abbreviations
        "str", "vln", "vla", "vlc", "ens",
    ],
    # Woodwind - comprehensive
    "woodwind": [
        "flute", "clarinet", "oboe", "bassoon", "woodwind",
        "piccolo", "alto-flute", "bass-flute", "recorder",
        "english-horn", "cor-anglais", "bass-clarinet", "contrabassoon",
        "pan-flute", "panpipes", "ocarina", "tin-whistle", "penny-whistle",
        "shakuhachi", "bansuri", "dizi", "xiao",
        # Common abbreviations
        "fl", "cl", "ob", "bsn", "ww",
    ],
    # Vocal - comprehensive
    "vocal": [
        "vocal", "vocals", "vox", "voice", "choir", "chant", "sing",
        "singer", "singing", "acapella", "a-capella", "harmony",
        "lead-vocal", "backing", "background", "bv", "adlib",
        "soprano", "alto", "tenor", "baritone", "bass-voice",
        "falsetto", "belting", "whisper", "spoken", "rap-vocal",
        "scream", "growl", "clean", "processed", "vocoder",
        "talk-box", "autotune", "formant", "chorale", "chorus",
        # Common abbreviations
        "voc", "vx", "chr",
    ],
    # Orchestral - comprehensive
    "orchestral": [
        "orchestra", "orchestral", "cinematic", "film", "score",
        "soundtrack", "trailer", "epic", "dramatic", "heroic",
        "action", "adventure", "fantasy", "sci-fi", "horror",
        "suspense", "tension", "romantic", "emotional",
        "symphonic", "symphony", "philharmonic", "concert",
        "tutti", "full-orchestra", "chamber-orchestra",
        # Common abbreviations
        "orch", "cine", "flm",
    ],
}

# Category 2: GENRE (expanded)
GENRE_KEYWORDS = {
    "rock": [
        "rock", "metal", "punk", "grunge", "alternative", "indie",
        "hard-rock", "soft-rock", "prog-rock", "classic-rock",
        "glam", "nu-metal", "metalcore", "deathcore", "thrash",
        "death-metal", "black-metal", "doom", "stoner", "sludge",
        "post-rock", "shoegaze", "noise-rock", "garage",
    ],
    "jazz": [
        "jazz", "jazzy", "swing", "bebop", "fusion", "smooth-jazz",
        "cool", "modal", "free-jazz", "avant-garde", "big-band",
        "dixieland", "ragtime", "stride", "boogie", "bossa",
        "latin-jazz", "afro-cuban", "gypsy-jazz", "manouche",
        # Compound variants found in filenames
        "jazzswing", "jazzfunk", "tightjazz", "jazzhop", "jazzrock",
    ],
    "blues": ["blues", "bluesy", "delta", "chicago", "texas", "soul-blues"],
    "funk": [
        "funk", "funky", "disco", "boogie", "groove", "p-funk",
        "acid-jazz", "nu-funk", "electro-funk",
        # Compound variants
        "jazzfunk", "tightjazzfunk", "acidfunk", "deepfunk", "discofunk",
    ],
    "soul": [
        "soul", "motown", "r&b", "rnb", "neo-soul", "nu-soul",
        "northern", "southern", "philadelphia", "quiet-storm",
    ],
    "house": [
        "house", "deephouse", "deep-house", "tech-house", "techhouse",
        "progressive-house", "prog-house", "future-house", "tropical",
        "chicago-house", "french-house", "funky-house", "soulful-house",
        "afro-house", "tribal-house", "nu-disco", "disco-house",
        "jackin", "garage-house", "speed-garage",
    ],
    "techno": [
        "techno", "minimal", "industrial", "dark-techno", "peak-time",
        "detroit", "berlin", "acid-techno", "hard-techno",
        "dub-techno", "ambient-techno",
    ],
    "trance": [
        "trance", "psytrance", "psy-trance", "uplifting", "goa",
        "progressive-trance", "prog-trance", "vocal-trance",
        "hard-trance", "tech-trance", "dream-trance", "eurotrance",
    ],
    "hip-hop": [
        "hiphop", "hip-hop", "hip_hop", "rap", "trap", "boom-bap",
        "old-school", "new-school", "east-coast", "west-coast",
        "southern", "dirty-south", "crunk", "g-funk", "conscious",
        "underground", "drill", "grime", "uk-rap", "phonk",
    ],
    "dnb": [
        "dnb", "drum-and-bass", "drumnbass", "jungle", "liquid",
        "neurofunk", "neuro", "jump-up", "darkstep", "techstep",
        "atmospheric", "rollers", "dancefloor", "halftime",
    ],
    "dubstep": [
        "dubstep", "brostep", "riddim", "tearout", "melodic-dubstep",
        "chillstep", "uk-dubstep", "140", "deep-dubstep",
    ],
    "edm": [
        "edm", "electro", "electronic", "dance", "electronica",
        "big-room", "festival", "mainstage", "complextro",
        "moombahton", "hardstyle", "hardcore", "gabber",
    ],
    "ambient": [
        "ambient", "atmospheric", "soundscape", "drone", "new-age",
        "meditation", "relaxation", "nature", "field-recording",
        "dark-ambient", "space-ambient", "isolationist",
    ],
    "classical": [
        "classical", "baroque", "romantic", "symphony", "concerto",
        "sonata", "chamber", "opera", "ballet", "contemporary",
        "modern-classical", "neo-classical", "minimalist",
    ],
    "pop": [
        "pop", "synth-pop", "electropop", "indie-pop", "dream-pop",
        "art-pop", "power-pop", "bubblegum", "teen-pop", "k-pop",
        "j-pop", "europop", "dance-pop",
    ],
    "reggae": [
        "reggae", "dub", "ska", "dancehall", "roots", "rocksteady",
        "lovers-rock", "ragga", "jungle", "bashment",
    ],
    "latin": [
        "latin", "salsa", "bossa", "samba", "tango", "cumbia",
        "merengue", "bachata", "reggaeton", "latin-house",
        "afrobeat", "afro-latin", "boogaloo", "mambo", "cha-cha",
    ],
    "world": [
        "world", "ethnic", "tribal", "african", "asian", "middle-east",
        "indian", "arabic", "celtic", "nordic", "balkan",
        "caribbean", "polynesian", "native", "folk",
    ],
    "lofi": [
        "lofi", "lo-fi", "chillhop", "jazzhop", "study", "beats",
        "bedroom", "tape", "vinyl", "dusty", "nostalgic",
    ],
    "country": [
        "country", "folk", "bluegrass", "americana", "western",
        "honky-tonk", "outlaw", "nashville", "alt-country",
        "country-rock", "country-pop", "bro-country",
    ],
}

# Category 5: TYPE (pattern/structure)
TYPE_KEYWORDS = {
    "loop": [
        "loop", "loops", "looped", "looping", "lp",
        "synthloop", "drumloop", "bassloop", "keyloop", "vocalloop",
        "melodyloop", "percloop", "guitarloop", "padloop",
    ],
    "groove": ["groove", "grooves", "groovy", "grv", "grooving"],
    "fill": ["fill", "fills", "drum-fill", "drumfill", "fl"],
    "progression": [
        "progression", "chord-progression", "chords", "chordprog", "prog",
        "chordloop", "chordseq",
    ],
    "melody": [
        "melody", "melodic", "melodious", "tune", "mel", "melodie",
        "topline", "top-line", "lead-melody",
    ],
    "riff": ["riff", "lick", "hook", "riffs", "licks", "hooks", "hookpart"],
    "one-shot": ["one-shot", "oneshot", "hit", "stab", "shot", "single"],
    "break": ["break", "breakdown", "breakbeat", "brk", "breaks"],
    "intro": ["intro", "introduction", "opener", "opening", "start"],
    "outro": ["outro", "ending", "closer", "closing"],
    "verse": ["verse", "verses", "vrs", "vs"],
    "chorus": ["chorus", "refrain", "chr"],
    "bridge": ["bridge", "middle-8", "middle8", "brdg"],
    "build": ["build", "buildup", "riser", "rising", "bld", "rise"],
    "drop": ["drop", "drops", "drp", "dropmelody"],
    "transition": ["transition", "trans", "fx", "effect", "sweep"],
    "arp": ["arp", "arpeggio", "arpeggios", "arpeggiated", "arpeg"],
    "sequence": ["sequence", "seq", "pattern", "seqs", "sequences"],
    "part": ["part", "section", "a-section", "b-section"],
    "kit": ["kit", "drumkit", "kit-part"],
    "main": ["main", "mainloop", "mainpart", "mainmelody"],
    "beat": ["beat", "beats", "beatloop", "beat-loop"],
    "bar": ["1-bar", "2-bar", "4-bar", "8-bar", "16-bar", "1bar", "2bar", "4bar", "8bar"],
    "lead": ["lead", "leadpart", "lead-part", "leadline"],
    "track": ["track", "trk", "fulltrack", "full-track"],
}

# Category 6: MOOD (emotional quality)
MOOD_KEYWORDS = {
    "happy": ["happy", "joyful", "upbeat", "cheerful", "positive", "bright", "joy", "uplifting"],
    "sad": ["sad", "melancholy", "melancholic", "somber", "emotional", "tearful", "mournful"],
    "dark": ["dark", "sinister", "evil", "menacing", "ominous", "scary", "creepy", "haunting"],
    "energetic": ["energetic", "high-energy", "powerful", "driving", "intense", "pumping", "banging"],
    "chill": ["chill", "relaxed", "mellow", "calm", "peaceful", "serene", "tranquil", "soothing"],
    "aggressive": ["aggressive", "angry", "hard", "heavy", "brutal", "harsh", "fierce", "violent"],
    "dreamy": ["dreamy", "ethereal", "floating", "spacey", "ambient", "hazy", "foggy", "misty"],
    "groovy": ["groovy", "funky", "bouncy", "swinging", "grooving", "tight", "pocket"],
    "epic": ["epic", "cinematic", "dramatic", "heroic", "triumphant", "majestic", "grandiose"],
    "mysterious": ["mysterious", "suspenseful", "tense", "eerie", "enigmatic", "cryptic", "strange"],
    "romantic": ["romantic", "love", "sensual", "intimate", "tender", "sweet", "lovely"],
    "nostalgic": ["nostalgic", "retro", "vintage", "classic", "old-school", "throwback", "80s", "90s"],
    "warm": ["warm", "soft", "gentle", "smooth", "cozy", "comforting", "lush"],
    "cold": ["cold", "icy", "frozen", "sterile", "clinical", "distant", "detached"],
    "playful": ["playful", "fun", "quirky", "whimsical", "bouncy", "silly", "goofy"],
}

# =============================================================================
# FUZZY MATCHING
# =============================================================================

def levenshtein_distance(s1: str, s2: str) -> int:
    """Calculate Levenshtein distance between two strings."""
    if len(s1) < len(s2):
        return levenshtein_distance(s2, s1)

    if len(s2) == 0:
        return len(s1)

    previous_row = range(len(s2) + 1)
    for i, c1 in enumerate(s1):
        current_row = [i + 1]
        for j, c2 in enumerate(s2):
            insertions = previous_row[j + 1] + 1
            deletions = current_row[j] + 1
            substitutions = previous_row[j] + (c1 != c2)
            current_row.append(min(insertions, deletions, substitutions))
        previous_row = current_row

    return previous_row[-1]

def normalize_variant(word: str, normalization_map: Dict[str, str]) -> str:
    """Normalize a word using the normalization map (direct lookup only for speed)."""
    word_lower = word.lower()
    return normalization_map.get(word_lower, word_lower)

# =============================================================================
# TAG EXTRACTION FUNCTIONS
# =============================================================================

def normalize_text(text: str) -> str:
    """Normalize text for matching: lowercase, replace separators with spaces."""
    text = text.lower()
    text = re.sub(r'[_\-./\\]+', ' ', text)
    return text

def extract_keywords_from_text(
    text: str,
    keyword_dict: Dict[str, List[str]],
    normalization_map: Optional[Dict[str, str]] = None
) -> Set[str]:
    """Extract matching keywords from text using the keyword dictionary."""
    text = normalize_text(text)
    found = set()

    # First pass: direct keyword matching with word boundaries
    for category, keywords in keyword_dict.items():
        for keyword in keywords:
            # Check for whole word match
            pattern = r'\b' + re.escape(keyword.replace('-', ' ').replace('_', ' ')) + r'\b'
            if re.search(pattern, text, re.IGNORECASE):
                found.add(category)
                break

    # Second pass: substring matching for keywords 4+ chars (to avoid false positives)
    for category, keywords in keyword_dict.items():
        if category in found:
            continue  # Already found this category
        for keyword in keywords:
            if len(keyword) >= 4 and keyword in text:
                found.add(category)
                break

    # Third pass: normalize words and check against normalization map
    if normalization_map:
        words = set(re.findall(r'\b[a-z0-9]+\b', text))
        for word in words:
            if word in normalization_map:
                canonical = normalization_map[word]
                # Find which category this canonical form belongs to
                for category, keywords in keyword_dict.items():
                    if canonical in keywords or canonical == category:
                        found.add(category)
                        break

    return found

def extract_instruments(filename: str, filepath: str) -> Set[str]:
    """Extract instrument categories from filename and path."""
    text = f"{filepath} {filename}"
    return extract_keywords_from_text(text, INSTRUMENT_KEYWORDS, INSTRUMENT_NORMALIZATIONS)

def extract_genres(filename: str, filepath: str) -> Set[str]:
    """Extract genre from filename and path."""
    text = f"{filepath} {filename}"
    return extract_keywords_from_text(text, GENRE_KEYWORDS, GENRE_NORMALIZATIONS)

def extract_types(filename: str, filepath: str) -> Set[str]:
    """Extract type/pattern from filename and path."""
    text = f"{filepath} {filename}"
    return extract_keywords_from_text(text, TYPE_KEYWORDS)

def extract_moods(filename: str, filepath: str) -> Set[str]:
    """Extract mood from filename and path."""
    text = f"{filepath} {filename}"
    return extract_keywords_from_text(text, MOOD_KEYWORDS)

def extract_all_categories(file_tuple: Tuple[int, str, str]) -> Tuple[int, Set[str], Set[str], Set[str], Set[str]]:
    """Extract all categories from a single file (for parallel processing)."""
    file_id, filename, filepath = file_tuple
    filepath = filepath or ""

    instruments = extract_instruments(filename, filepath)
    genres = extract_genres(filename, filepath)
    types = extract_types(filename, filepath)
    moods = extract_moods(filename, filepath)

    return (file_id, instruments, genres, types, moods)

# =============================================================================
# DATABASE FUNCTIONS
# =============================================================================

def get_or_create_tag(cursor, name: str, category: str) -> int:
    """Get or create a tag and return its ID."""
    cursor.execute("""
        INSERT INTO tags (name, category, usage_count, created_at)
        VALUES (%s, %s, 0, NOW())
        ON CONFLICT (name) DO UPDATE SET category = COALESCE(tags.category, EXCLUDED.category)
        RETURNING id
    """, (name, category))
    return cursor.fetchone()[0]

def add_tag_to_file(cursor, file_id: int, tag_id: int):
    """Add a tag to a file (ignore if exists)."""
    cursor.execute("""
        INSERT INTO file_tags (file_id, tag_id, added_at, added_by, source)
        VALUES (%s, %s, NOW(), 'vip3-retagger', 'filename')
        ON CONFLICT (file_id, tag_id) DO NOTHING
    """, (file_id, tag_id))

def process_batch_parallel(conn, files: list, dry_run: bool = False, num_workers: int = None) -> dict:
    """Process a batch of files using parallel extraction."""
    stats = defaultdict(int)

    if num_workers is None:
        num_workers = cpu_count()

    # Parallel extraction of categories
    with Pool(num_workers) as pool:
        results = pool.map(extract_all_categories, files)

    # Sequential database writes (can't parallelize DB connections easily)
    if not dry_run:
        with conn.cursor() as cursor:
            for file_id, instruments, genres, types, moods in results:
                for instrument in instruments:
                    stats['instrument'] += 1
                    tag_id = get_or_create_tag(cursor, instrument, 'instrument')
                    add_tag_to_file(cursor, file_id, tag_id)

                for genre in genres:
                    stats['genre'] += 1
                    tag_id = get_or_create_tag(cursor, genre, 'genre')
                    add_tag_to_file(cursor, file_id, tag_id)

                for type_tag in types:
                    stats['type'] += 1
                    tag_id = get_or_create_tag(cursor, type_tag, 'pattern')
                    add_tag_to_file(cursor, file_id, tag_id)

                for mood in moods:
                    stats['mood'] += 1
                    tag_id = get_or_create_tag(cursor, mood, 'mood')
                    add_tag_to_file(cursor, file_id, tag_id)

            conn.commit()
    else:
        # Dry run - just count stats
        for file_id, instruments, genres, types, moods in results:
            stats['instrument'] += len(instruments)
            stats['genre'] += len(genres)
            stats['type'] += len(types)
            stats['mood'] += len(moods)

    return stats

def main():
    parser = argparse.ArgumentParser(description='Re-tag MIDI files with VIP3 categories')
    parser.add_argument('--batch-size', type=int, default=10000, help='Batch size for processing')
    parser.add_argument('--dry-run', action='store_true', help='Do not write to database')
    parser.add_argument('--limit', type=int, help='Limit number of files to process')
    parser.add_argument('--workers', type=int, default=cpu_count(), help='Number of parallel workers')
    args = parser.parse_args()

    print("=" * 60)
    print("VIP3 Category Re-tagger (Parallel)")
    print("=" * 60)
    print(f"Database: {DB_URL.split('@')[1] if '@' in DB_URL else DB_URL}")
    print(f"Batch size: {args.batch_size}")
    print(f"Workers: {args.workers}")
    print(f"Dry run: {args.dry_run}")
    if args.limit:
        print(f"Limit: {args.limit} files")
    print()

    # Show normalization stats
    print(f"Instrument normalizations: {len(INSTRUMENT_NORMALIZATIONS)} variants")
    print(f"Genre normalizations: {len(GENRE_NORMALIZATIONS)} variants")
    print()

    conn = psycopg2.connect(DB_URL)

    try:
        with conn.cursor() as cursor:
            # Get total count
            cursor.execute("SELECT COUNT(*) FROM files")
            total_files = cursor.fetchone()[0]
            print(f"Total files in database: {total_files:,}")

            if args.limit:
                total_files = min(total_files, args.limit)

            # Process in batches
            offset = 0
            processed = 0
            total_stats = defaultdict(int)

            while offset < total_files:
                limit = min(args.batch_size, total_files - offset)

                cursor.execute("""
                    SELECT id, filename, filepath
                    FROM files
                    ORDER BY id
                    OFFSET %s LIMIT %s
                """, (offset, limit))

                files = cursor.fetchall()
                if not files:
                    break

                batch_stats = process_batch_parallel(conn, files, args.dry_run, args.workers)

                for key, value in batch_stats.items():
                    total_stats[key] += value

                processed += len(files)
                offset += limit

                # Progress
                pct = (processed / total_files) * 100
                print(f"\rProcessed: {processed:,}/{total_files:,} ({pct:.1f}%) - "
                      f"Instrument: {total_stats['instrument']:,}, "
                      f"Genre: {total_stats['genre']:,}, "
                      f"Type: {total_stats['type']:,}, "
                      f"Mood: {total_stats['mood']:,}", end='', flush=True)

            print("\n")
            print("=" * 60)
            print("RESULTS")
            print("=" * 60)
            print(f"Files processed: {processed:,}")
            print(f"Instrument tags added: {total_stats['instrument']:,}")
            print(f"Genre tags added: {total_stats['genre']:,}")
            print(f"Type tags added: {total_stats['type']:,}")
            print(f"Mood tags added: {total_stats['mood']:,}")
            print(f"Total tags added: {sum(total_stats.values()):,}")

            if args.dry_run:
                print("\n[DRY RUN - No changes written to database]")
            else:
                # Update usage counts
                print("\nUpdating tag usage counts...")
                cursor.execute("""
                    UPDATE tags t
                    SET usage_count = (
                        SELECT COUNT(*) FROM file_tags ft WHERE ft.tag_id = t.id
                    )
                """)
                conn.commit()
                print("Done!")

    finally:
        conn.close()

if __name__ == '__main__':
    main()
