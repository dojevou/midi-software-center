#!/bin/bash
# Interactive Pipeline Phase Runner
# Allows selective execution of pipeline phases

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎵 MIDI Pipeline - Phase Selector${NC}"
echo "=================================="
echo ""

# Database connection
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Check current status
echo -e "${BLUE}📊 Current Status:${NC}"
TOTAL=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files" 2>/dev/null || echo "0")
ANALYZED=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE analyzed_at IS NOT NULL" 2>/dev/null || echo "0")
WITH_METADATA=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM musical_metadata" 2>/dev/null || echo "0")

echo "  Files imported: $TOTAL"
echo "  Files analyzed: $ANALYZED"
echo "  With metadata: $WITH_METADATA"
echo ""

# Phase selection
echo -e "${YELLOW}Select phases to run (y/n):${NC}"
echo ""

# Phase 1: Import
read -p "1. Import (extract archives + insert to DB) [y/N]: " RUN_IMPORT
RUN_IMPORT=${RUN_IMPORT:-n}

# Phase 2: Sanitization
read -p "2. Sanitize filenames (clean special chars) [y/N]: " RUN_SANITIZE
RUN_SANITIZE=${RUN_SANITIZE:-n}

# Phase 3: Track Splitting
read -p "3. Split multi-track files [y/N]: " RUN_SPLIT
RUN_SPLIT=${RUN_SPLIT:-n}

# Phase 4: Analysis
read -p "4. Analyze (BPM/key/chords/drums) [Y/n]: " RUN_ANALYZE
RUN_ANALYZE=${RUN_ANALYZE:-y}

# Phase 5: Renaming
read -p "5. Production rename (metadata-based) [y/N]: " RUN_RENAME
RUN_RENAME=${RUN_RENAME:-n}

echo ""
echo -e "${GREEN}Starting selected phases...${NC}"
echo ""

# Change to pipeline directory
cd "$(dirname "$0")/../pipeline/src-tauri"

# Phase 1: Import
if [[ "$RUN_IMPORT" =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Phase 1: Import${NC}"
    read -p "Source directory/archive: " SOURCE_PATH

    if [ -z "$SOURCE_PATH" ]; then
        echo -e "${RED}❌ Source path required for import${NC}"
    else
        echo "Running import from: $SOURCE_PATH"
        cargo run --release --bin batch_import -- "$SOURCE_PATH"
    fi
    echo ""
fi

# Phase 2: Sanitization
if [[ "$RUN_SANITIZE" =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Phase 2: Sanitize Filenames${NC}"
    echo "Running filename sanitization..."
    cargo run --release --bin normalize_filenames
    echo ""
fi

# Phase 3: Track Splitting
if [[ "$RUN_SPLIT" =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Phase 3: Track Splitting${NC}"
    echo "Running batch split..."
    cargo run --release --bin batch_split
    echo ""
fi

# Phase 4: Analysis
if [[ "$RUN_ANALYZE" =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Phase 4: Musical Analysis${NC}"
    echo "Running BPM/key/chord/drum analysis..."
    cargo run --release --bin analyze
    echo ""
fi

# Phase 5: Renaming
if [[ "$RUN_RENAME" =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Phase 5: Production Renaming${NC}"
    echo "Renaming files with metadata..."
    # TODO: Add rename binary when ready
    echo -e "${YELLOW}⚠️  Rename phase not yet implemented${NC}"
    echo ""
fi

# Final status
echo -e "${GREEN}✅ Selected phases complete!${NC}"
echo ""
echo -e "${BLUE}📊 Final Status:${NC}"
TOTAL_FINAL=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files")
ANALYZED_FINAL=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files WHERE analyzed_at IS NOT NULL")
WITH_METADATA_FINAL=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM musical_metadata")

echo "  Files imported: $TOTAL_FINAL"
echo "  Files analyzed: $ANALYZED_FINAL"
echo "  With metadata: $WITH_METADATA_FINAL"
echo ""
