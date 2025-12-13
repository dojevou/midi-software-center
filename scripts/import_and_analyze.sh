#!/bin/bash
# Combined Import + Analysis Script
# Runs Phase 1 (import) and Phase 2 (analysis) automatically

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IMPORT_TOOL="/home/dojevou/projects/midi-software-center/target/release/import-tool"
ANALYZE_TOOL="/home/dojevou/projects/midi-software-center/target/release/analyze"
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Check if directory argument provided
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: No directory specified${NC}"
    echo "Usage: $0 <directory>"
    echo ""
    echo "Example:"
    echo "  $0 /path/to/midi/files"
    exit 1
fi

MIDI_DIR="$1"

# Validate directory exists
if [ ! -d "$MIDI_DIR" ]; then
    echo -e "${RED}Error: Directory not found: $MIDI_DIR${NC}"
    exit 1
fi

# Check tools exist
if [ ! -f "$IMPORT_TOOL" ]; then
    echo -e "${RED}Error: Import tool not found at $IMPORT_TOOL${NC}"
    echo "Run: cargo build --release --package import-tool"
    exit 1
fi

if [ ! -f "$ANALYZE_TOOL" ]; then
    echo -e "${RED}Error: Analyze tool not found at $ANALYZE_TOOL${NC}"
    echo "Run: cargo build --release --package midi-pipeline"
    exit 1
fi

echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo -e "${BLUE}   MIDI Import & Analysis Pipeline${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}📁 Directory:${NC} $MIDI_DIR"
echo -e "${YELLOW}🗄️  Database:${NC} midi_library @ localhost:5433"
echo ""

# Count MIDI files
MIDI_COUNT=$(find "$MIDI_DIR" -type f \( -iname "*.mid" -o -iname "*.midi" \) 2>/dev/null | wc -l)
echo -e "${YELLOW}📊 Found:${NC} $MIDI_COUNT MIDI files"
echo ""

# Confirmation
read -p "Continue with import + analysis? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}PHASE 1: File Import + Filename Extraction${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Run import
START_TIME=$(date +%s)
"$IMPORT_TOOL" "$MIDI_DIR"
IMPORT_EXIT=$?
IMPORT_END_TIME=$(date +%s)
IMPORT_DURATION=$((IMPORT_END_TIME - START_TIME))

if [ $IMPORT_EXIT -ne 0 ]; then
    echo -e "${RED}❌ Import failed with exit code $IMPORT_EXIT${NC}"
    exit $IMPORT_EXIT
fi

echo ""
echo -e "${GREEN}✅ Phase 1 Complete${NC} (${IMPORT_DURATION}s)"
echo ""
sleep 2

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}PHASE 2: MIDI Content Analysis${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Run analysis
ANALYSIS_START_TIME=$(date +%s)
export DATABASE_URL
"$ANALYZE_TOOL"
ANALYSIS_EXIT=$?
ANALYSIS_END_TIME=$(date +%s)
ANALYSIS_DURATION=$((ANALYSIS_END_TIME - ANALYSIS_START_TIME))

if [ $ANALYSIS_EXIT -ne 0 ]; then
    echo -e "${RED}❌ Analysis failed with exit code $ANALYSIS_EXIT${NC}"
    echo -e "${YELLOW}Note: Files were imported successfully, but analysis failed${NC}"
    exit $ANALYSIS_EXIT
fi

echo ""
echo -e "${GREEN}✅ Phase 2 Complete${NC} (${ANALYSIS_DURATION}s)"
echo ""

# Summary
TOTAL_TIME=$((IMPORT_DURATION + ANALYSIS_DURATION))
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo -e "${BLUE}   PIPELINE COMPLETE${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}✅ Phase 1 (Import):${NC}     ${IMPORT_DURATION}s"
echo -e "${GREEN}✅ Phase 2 (Analysis):${NC}   ${ANALYSIS_DURATION}s"
echo -e "${GREEN}✅ Total Time:${NC}           ${TOTAL_TIME}s"
echo ""
echo -e "${YELLOW}📊 Database now contains:${NC}"
echo "   - File records with filename metadata"
echo "   - Musical analysis (BPM, key, notes, etc.)"
echo "   - Total: 57 metadata fields per file"
echo ""
