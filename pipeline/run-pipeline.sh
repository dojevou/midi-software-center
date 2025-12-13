#!/bin/bash
# MIDI Pipeline Orchestrator
# Chains import and analysis phases for complete processing

set -e  # Exit on error

# Configuration
export DATABASE_URL="${DATABASE_URL:-postgresql://midiuser:145278963@localhost:5433/midi_library}"
MIDI_DIR="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARIES_DIR="${SCRIPT_DIR}/../target/release"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Usage
if [ -z "$MIDI_DIR" ]; then
    echo "Usage: $0 <midi-directory>"
    echo ""
    echo "Example: $0 /home/dojevou/midi-files"
    echo ""
    echo "Environment variables:"
    echo "  DATABASE_URL - PostgreSQL connection (default: localhost:5433)"
    exit 1
fi

# Validate directory exists
if [ ! -d "$MIDI_DIR" ]; then
    echo "Error: Directory not found: $MIDI_DIR"
    exit 1
fi

# Validate binaries exist
if [ ! -f "$BINARIES_DIR/import_unified" ]; then
    echo "Error: import_unified binary not found. Run 'cargo build --release' first."
    exit 1
fi

if [ ! -f "$BINARIES_DIR/analyze" ]; then
    echo "Error: analyze binary not found. Run 'cargo build --release' first."
    exit 1
fi

# Start pipeline
echo -e "${BLUE}🎵 MIDI Pipeline Orchestrator${NC}"
echo "=============================="
echo "Source: $MIDI_DIR"
echo "Database: $DATABASE_URL"
echo ""

START_TIME=$(date +%s)

# Phase 1: Import & Index
echo -e "${BLUE}📥 Phase 1/2: Import & Index${NC}"
echo "  → Scanning directories for MIDI files"
echo "  → Extracting filename metadata (BPM, key, genres)"
echo "  → Adding files to database"
echo ""

"$BINARIES_DIR/import_unified" "$MIDI_DIR"
PHASE1_EXIT=$?

if [ $PHASE1_EXIT -ne 0 ]; then
    echo -e "${YELLOW}⚠️  Phase 1 failed with exit code $PHASE1_EXIT${NC}"
    exit $PHASE1_EXIT
fi

echo -e "${GREEN}✅ Phase 1 complete${NC}"
echo ""

# Get import stats
IMPORT_END=$(date +%s)
IMPORT_DURATION=$((IMPORT_END - START_TIME))
echo "Import duration: ${IMPORT_DURATION}s"
echo ""

# Phase 2: MIDI Content Analysis
echo -e "${BLUE}🔬 Phase 2/2: MIDI Content Analysis${NC}"
echo "  → Parsing MIDI files"
echo "  → Detecting BPM from MIDI events"
echo "  → Detecting key signatures"
echo "  → Analyzing chord progressions"
echo "  → Updating database with analysis"
echo ""

"$BINARIES_DIR/analyze"
PHASE2_EXIT=$?

if [ $PHASE2_EXIT -ne 0 ]; then
    echo -e "${YELLOW}⚠️  Phase 2 failed with exit code $PHASE2_EXIT${NC}"
    exit $PHASE2_EXIT
fi

echo -e "${GREEN}✅ Phase 2 complete${NC}"
echo ""

# Summary
END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))
ANALYSIS_DURATION=$((END_TIME - IMPORT_END))

echo ""
echo -e "${GREEN}🎉 Pipeline Complete!${NC}"
echo "=============================="
echo "Phase 1 (Import):    ${IMPORT_DURATION}s"
echo "Phase 2 (Analysis):  ${ANALYSIS_DURATION}s"
echo "Total time:          ${TOTAL_DURATION}s"
echo ""
echo "Files are now indexed and analyzed in the database."
echo ""
echo "Optional: Run individual track splitting with:"
echo "  $BINARIES_DIR/split <file_id>"
