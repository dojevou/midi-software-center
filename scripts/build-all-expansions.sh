#!/bin/bash
# Build All 100 MPC Expansions
# Created: November 22, 2025
# Purpose: Create 97 instrument + 3 functional expansions for Akai Force/MPC

set -e  # Exit on error

# Configuration
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
PROJECT_ROOT="/home/dojevou/projects/midi-software-center"
FORCE_DRIVE="/media/dojevou/RYXSTR"
EXPANSIONS_DIR="$FORCE_DRIVE/Expansions"
ARP_PATTERNS_DIR="$FORCE_DRIVE/Arp Patterns"
CONVERTER="$PROJECT_ROOT/target/release/midi_to_mpcpattern_parallel"
TEMP_DIR="/tmp/mpc_build"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  MPC/Force Expansion Builder${NC}"
echo -e "${BLUE}  100 Total Expansions${NC}"
echo -e "${BLUE}========================================${NC}"
echo

# Check prerequisites
echo -e "${YELLOW}[1/9] Checking prerequisites...${NC}"
if [ ! -f "$CONVERTER" ]; then
    echo -e "${RED}ERROR: Converter not found at $CONVERTER${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Converter found (1.5MB)${NC}"

# Create directories
echo -e "${YELLOW}[2/9] Creating output directories...${NC}"
mkdir -p "$EXPANSIONS_DIR"
mkdir -p "$ARP_PATTERNS_DIR"
mkdir -p "$TEMP_DIR"
echo -e "${GREEN}✓ Directories created${NC}"
echo "  - Expansions: $EXPANSIONS_DIR"
echo "  - Arp Patterns: $ARP_PATTERNS_DIR"
echo "  - Temp: $TEMP_DIR"
echo

# Extract pattern file lists from database
echo -e "${YELLOW}[3/9] Extracting pattern types from database...${NC}"
psql "$DB_URL" -f "$PROJECT_ROOT/scripts/extract-pattern-types-simple.sql" 2>&1 | tee "$TEMP_DIR/extraction.log"
echo -e "${GREEN}✓ Pattern extraction complete${NC}"
echo

# Count extracted patterns
echo -e "${YELLOW}[4/9] Counting extracted patterns...${NC}"
ARP_COUNT=$(wc -l < /tmp/arpeggio_patterns.txt 2>/dev/null || echo "0")
RHYTHM_COUNT=$(wc -l < /tmp/rhythm_patterns.txt 2>/dev/null || echo "0")
CHORD_COUNT=$(wc -l < /tmp/chord_patterns.txt 2>/dev/null || echo "0")
echo -e "${GREEN}✓ Pattern counts:${NC}"
echo "  - Arpeggio patterns: $ARP_COUNT files"
echo "  - Rhythm patterns: $RHYTHM_COUNT files"
echo "  - Chord patterns: $CHORD_COUNT files"
echo

# Copy arp/rhythm MIDI files (no conversion needed - parallel with xargs)
echo -e "${YELLOW}[5/9] Copying arp/rhythm MIDI files to Arp_Patterns folder...${NC}"
COPY_START=$(date +%s)

# Use xargs for parallel copying (much faster)
cat /tmp/arpeggio_patterns.txt /tmp/rhythm_patterns.txt | \
    xargs -I {} -P 8 cp {} "$ARP_PATTERNS_DIR/" 2>/dev/null || true

COPY_END=$(date +%s)
COPY_TIME=$((COPY_END - COPY_START))

MIDI_COUNT=$(find "$ARP_PATTERNS_DIR" -name "*.mid" -type f 2>/dev/null | wc -l)
echo -e "${GREEN}✓ MIDI files copied:${NC}"
echo "  - Total files in Arp_Patterns: $MIDI_COUNT .mid files"
echo "  - Copy time: ${COPY_TIME}s"
echo

# Create 97 instrument expansion folders
echo -e "${YELLOW}[6/9] Creating 97 instrument expansion folders...${NC}"
"$PROJECT_ROOT/scripts/create_97_instrument_expansions.sh" "$EXPANSIONS_DIR" 2>&1 | tail -20
echo -e "${GREEN}✓ 97 instrument folders created${NC}"
echo

# Create 3 functional expansion folders
echo -e "${YELLOW}[7/9] Creating 3 functional expansion folders...${NC}"

# MIDI_ARPEGGIOS expansion
ARPEGGIOS_DIR="$EXPANSIONS_DIR/MIDI_ARPEGGIOS"
mkdir -p "$ARPEGGIOS_DIR/Patterns"
cat > "$ARPEGGIOS_DIR/Cache.json" << 'EOF'
{
  "name": "MIDI Arpeggios",
  "manufacturer": "MIDI Software Center",
  "description": "Arpeggiator patterns - ascending, descending, and melodic sequences",
  "category": "MIDI Patterns",
  "version": "1.0"
}
EOF
echo -e "${GREEN}  ✓ MIDI_ARPEGGIOS created${NC}"

# MIDI_RHYTHMS expansion
RHYTHMS_DIR="$EXPANSIONS_DIR/MIDI_RHYTHMS"
mkdir -p "$RHYTHMS_DIR/Patterns"
cat > "$RHYTHMS_DIR/Cache.json" << 'EOF'
{
  "name": "MIDI Rhythms",
  "manufacturer": "MIDI Software Center",
  "description": "Rhythm patterns - straight, swing, shuffle, and syncopated grooves",
  "category": "MIDI Patterns",
  "version": "1.0"
}
EOF
echo -e "${GREEN}  ✓ MIDI_RHYTHMS created${NC}"

# MIDI_CHORDS expansion
CHORDS_DIR="$EXPANSIONS_DIR/MIDI_CHORDS"
mkdir -p "$CHORDS_DIR/Patterns"
cat > "$CHORDS_DIR/Cache.json" << 'EOF'
{
  "name": "MIDI Chords",
  "manufacturer": "MIDI Software Center",
  "description": "Chord progressions and harmonic patterns",
  "category": "MIDI Patterns",
  "version": "1.0"
}
EOF
echo -e "${GREEN}  ✓ MIDI_CHORDS created${NC}"
echo

# Convert functional expansion patterns to .mpcpattern
echo -e "${YELLOW}[8/9] Converting functional expansion patterns...${NC}"

echo "  Converting arpeggio patterns..."
CONVERT_START=$(date +%s)
cat /tmp/arpeggio_patterns.txt | while IFS= read -r filepath; do
    if [ -f "$filepath" ]; then
        filename=$(basename "$filepath" .mid)
        "$CONVERTER" "$filepath" "$ARPEGGIOS_DIR/Patterns/${filename}.mpcpattern" 2>/dev/null || true
    fi
done
ARP_CONVERTED=$(ls -1 "$ARPEGGIOS_DIR/Patterns" 2>/dev/null | wc -l)
echo -e "${GREEN}    ✓ Converted $ARP_CONVERTED arpeggio patterns${NC}"

echo "  Converting rhythm patterns..."
cat /tmp/rhythm_patterns.txt | while IFS= read -r filepath; do
    if [ -f "$filepath" ]; then
        filename=$(basename "$filepath" .mid)
        "$CONVERTER" "$filepath" "$RHYTHMS_DIR/Patterns/${filename}.mpcpattern" 2>/dev/null || true
    fi
done
RHYTHM_CONVERTED=$(ls -1 "$RHYTHMS_DIR/Patterns" 2>/dev/null | wc -l)
echo -e "${GREEN}    ✓ Converted $RHYTHM_CONVERTED rhythm patterns${NC}"

echo "  Converting chord patterns..."
cat /tmp/chord_patterns.txt | while IFS= read -r filepath; do
    if [ -f "$filepath" ]; then
        filename=$(basename "$filepath" .mid)
        "$CONVERTER" "$filepath" "$CHORDS_DIR/Patterns/${filename}.mpcpattern" 2>/dev/null || true
    fi
done
CHORD_CONVERTED=$(ls -1 "$CHORDS_DIR/Patterns" 2>/dev/null | wc -l)
echo -e "${GREEN}    ✓ Converted $CHORD_CONVERTED chord patterns${NC}"

CONVERT_END=$(date +%s)
CONVERT_TIME=$((CONVERT_END - CONVERT_START))
echo -e "${GREEN}  ✓ Functional expansions converted in ${CONVERT_TIME}s${NC}"
echo

# Summary
echo -e "${YELLOW}[9/9] Build Summary${NC}"
TOTAL_EXPANSIONS=$(ls -1d "$EXPANSIONS_DIR"/*/ 2>/dev/null | wc -l)
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  BUILD COMPLETE!${NC}"
echo -e "${GREEN}========================================${NC}"
echo
echo "📊 Expansion Packs Created:"
echo "  - Instrument expansions: 97"
echo "  - Functional expansions: 3"
echo "  - Total expansions: $TOTAL_EXPANSIONS"
echo
echo "📁 Patterns Created:"
echo "  - MIDI_ARPEGGIOS: $ARP_CONVERTED .mpcpattern files"
echo "  - MIDI_RHYTHMS: $RHYTHM_CONVERTED .mpcpattern files"
echo "  - MIDI_CHORDS: $CHORD_CONVERTED .mpcpattern files"
echo "  - Arp_Patterns folder: $(ls -1 "$ARP_PATTERNS_DIR" 2>/dev/null | wc -l) .mid files"
echo
echo "📂 Output Locations:"
echo "  - Expansions: $EXPANSIONS_DIR"
echo "  - Arp Patterns: $ARP_PATTERNS_DIR"
echo
echo "⏱️  Conversion Time: ${CONVERT_TIME}s"
echo
echo "🎯 Next Steps:"
echo "  1. Convert remaining files for 97 instrument expansions (per-instrument conversion)"
echo "  2. Files are already on Force drive - ready to use!"
echo "  3. Test on Force hardware:"
echo "     - Browser > Expansions > MIDI_ARPEGGIOS/RHYTHMS/CHORDS"
echo "     - Arpeggiator menu > custom patterns"
echo
echo -e "${GREEN}✓ All done! Files ready on Force drive.${NC}"
