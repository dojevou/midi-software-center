#!/bin/bash
# Complete Pipeline for ~/projects/midi-software-center/midi-library/archives
# Processes 1.5M+ MIDI files through all 5 phases
# Phases: Import → Sanitize → Split → Analyze → Rename

set -e

# Configuration
SOURCE_DIR="$HOME/projects/midi-software-center/midi-library/archives"
PROJECT_DIR="$HOME/projects/midi-software-center"
DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     MIDI Pipeline - Complete 5-Phase Processing (1.5M files)      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Source: $SOURCE_DIR"
echo "Database: midi_library (port 5433)"
echo ""

# Function to show progress
show_stats() {
    echo -e "${BLUE}📊 Current Database Stats:${NC}"
    psql "$DB_URL" -t <<EOF
SELECT '  Total files:      ' || COUNT(*)::text FROM files;
SELECT '  Analyzed files:   ' || COUNT(*)::text FROM files WHERE analyzed_at IS NOT NULL;
SELECT '  With metadata:    ' || COUNT(*)::text FROM musical_metadata;
SELECT '  Unique tags:      ' || COUNT(DISTINCT tag_name)::text FROM tags;
SELECT '  Database size:    ' || pg_size_pretty(pg_database_size('midi_library'));
EOF
    echo ""
}

# Ensure database is running
echo -e "${YELLOW}[Pre-check] Verifying database...${NC}"
if ! docker ps | grep -q midi-library-postgres; then
    echo "Starting database..."
    cd "$PROJECT_DIR"
    make docker-up
    sleep 5
fi
echo -e "${GREEN}✓ Database running${NC}"
echo ""

# Show initial stats
show_stats

# Build optimized binaries if needed
echo -e "${YELLOW}[Pre-check] Building optimized binaries...${NC}"
cd "$PROJECT_DIR/pipeline/src-tauri"
if [ ! -f "../../target/release/import" ]; then
    echo "First-time build (10-15 minutes)..."
    RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
        cargo build --release --bins
else
    echo "Binaries already built (use 'cargo clean' to rebuild)"
fi
echo -e "${GREEN}✓ Binaries ready${NC}"
echo ""

PIPELINE_START=$(date +%s)

# ============================================================================
# PHASE 1: IMPORT (7,830 files/sec = ~3 minutes for 1.5M files)
# ============================================================================
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}PHASE 1: IMPORT${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo "Expected: ~3 minutes for 1.5M files (7,830 files/sec)"
echo "Log: /tmp/import_log.txt"
echo ""

PHASE1_START=$(date +%s)

DATABASE_URL="$DB_URL" \
    cargo run --release --bin import -- \
    --input "$SOURCE_DIR" \
    --recursive \
    --threads 16 \
    --batch-size 1000 \
    2>&1 | tee /tmp/import_log.txt

PHASE1_END=$(date +%s)
PHASE1_DURATION=$((PHASE1_END - PHASE1_START))
echo ""
echo -e "${GREEN}✓ Phase 1 complete: $PHASE1_DURATION seconds${NC}"
show_stats

# ============================================================================
# PHASE 2: SANITIZE FILENAMES (fast - 30 seconds estimate)
# ============================================================================
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}PHASE 2: SANITIZE FILENAMES${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo "Cleaning filenames: spaces→underscores, special chars removed"
echo "Log: /tmp/sanitize_log.txt"
echo ""

PHASE2_START=$(date +%s)

DATABASE_URL="$DB_URL" \
    cargo run --release --bin normalize_filenames \
    2>&1 | tee /tmp/sanitize_log.txt

PHASE2_END=$(date +%s)
PHASE2_DURATION=$((PHASE2_END - PHASE2_START))
echo ""
echo -e "${GREEN}✓ Phase 2 complete: $PHASE2_DURATION seconds${NC}"
echo ""

# ============================================================================
# PHASE 3: TRACK SPLITTING (2,000-3,650 files/min = ~7-12 hours)
# ============================================================================
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}PHASE 3: TRACK SPLITTING (OPTIONAL - Long-running)${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo "This phase can take 7-12 hours for 1.5M files"
echo ""
read -p "Run track splitting now? [y/N]: " RUN_SPLIT
RUN_SPLIT=${RUN_SPLIT:-n}

if [[ "$RUN_SPLIT" =~ ^[Yy]$ ]]; then
    PHASE3_START=$(date +%s)

    DATABASE_URL="$DB_URL" \
        cargo run --release --bin batch_split_optimized \
        2>&1 | tee /tmp/split_log.txt

    PHASE3_END=$(date +%s)
    PHASE3_DURATION=$((PHASE3_END - PHASE3_START))
    echo ""
    echo -e "${GREEN}✓ Phase 3 complete: $PHASE3_DURATION seconds${NC}"
else
    echo -e "${YELLOW}⊘ Phase 3 skipped (run manually later with: cargo run --release --bin batch_split_optimized)${NC}"
    PHASE3_DURATION=0
fi
echo ""

# ============================================================================
# PHASE 4: ANALYSIS (181 files/sec = ~2.3 hours for 1.5M files)
# ============================================================================
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}PHASE 4: MUSICAL ANALYSIS${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo "Expected: ~2.3 hours for 1.5M files (181 files/sec)"
echo "Analysis: BPM, key, chords, drums, tags"
echo "Log: /tmp/analyze_log.txt"
echo ""

PHASE4_START=$(date +%s)

DATABASE_URL="$DB_URL" \
    RAYON_NUM_THREADS=16 \
    cargo run --release --bin analyze -- \
    --batch-size 200 \
    --threads 16 \
    --parallel \
    2>&1 | tee /tmp/analyze_log.txt

PHASE4_END=$(date +%s)
PHASE4_DURATION=$((PHASE4_END - PHASE4_START))
echo ""
echo -e "${GREEN}✓ Phase 4 complete: $PHASE4_DURATION seconds${NC}"
show_stats

# ============================================================================
# PHASE 5: PRODUCTION RENAMING (Not yet implemented)
# ============================================================================
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}PHASE 5: PRODUCTION RENAMING${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}⚠️  Phase 5 not yet implemented as standalone binary${NC}"
echo "Files retain sanitized names from Phase 2"
echo ""

# ============================================================================
# FINAL SUMMARY
# ============================================================================
PIPELINE_END=$(date +%s)
TOTAL_DURATION=$((PIPELINE_END - PIPELINE_START))

echo -e "${GREEN}════════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}PIPELINE COMPLETE ✅${NC}"
echo -e "${GREEN}════════════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Timing Summary:"
echo "  Phase 1 (Import):     $(printf '%5d' $PHASE1_DURATION) sec (~$(echo "scale=1; $PHASE1_DURATION/60" | bc) min)"
echo "  Phase 2 (Sanitize):   $(printf '%5d' $PHASE2_DURATION) sec"
echo "  Phase 3 (Split):      $(printf '%5d' $PHASE3_DURATION) sec (skipped: use 0)"
echo "  Phase 4 (Analysis):   $(printf '%5d' $PHASE4_DURATION) sec (~$(echo "scale=1; $PHASE4_DURATION/3600" | bc) hours)"
echo "  Phase 5 (Rename):     not implemented"
echo "  ─────────────────────────────"
echo "  Total:                $(printf '%5d' $TOTAL_DURATION) sec (~$(echo "scale=1; $TOTAL_DURATION/3600" | bc) hours)"
echo ""

# Final statistics
echo -e "${BLUE}📊 Final Database Statistics:${NC}"
psql "$DB_URL" <<EOF
SELECT '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━';
SELECT 'Total Files:          ' || COUNT(*)::text FROM files;
SELECT 'Analyzed Files:       ' || COUNT(*)::text FROM files WHERE analyzed_at IS NOT NULL;
SELECT 'Musical Metadata:     ' || COUNT(*)::text FROM musical_metadata;
SELECT 'Unique Tags:          ' || COUNT(DISTINCT tag_name)::text FROM tags;
SELECT 'Tag Associations:     ' || COUNT(*)::text FROM file_tags;
SELECT 'Database Size:        ' || pg_size_pretty(pg_database_size('midi_library'));
SELECT '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━';
EOF

echo ""
echo -e "${GREEN}Next Steps:${NC}"
echo "  • Launch GUI:        make dev-pipeline"
echo "  • View logs:         tail -f /tmp/analyze_log.txt"
echo "  • Run Phase 3 later: cd pipeline/src-tauri && cargo run --release --bin batch_split_optimized"
echo "  • Monitor progress:  ./scripts/monitor-pipeline.sh"
echo ""
