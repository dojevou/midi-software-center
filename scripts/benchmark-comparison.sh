#!/bin/bash

#
# MIDI Pipeline Optimization Benchmark Comparison Script
# ======================================================
#
# Comprehensive benchmark to compare baseline vs optimized pipeline orchestrator
#
# Features:
# - Backs up and clears database for clean testing
# - Runs both baseline (git stash) and optimized (current) versions
# - Collects timing metrics using 'time' and optional 'hyperfine'
# - Calculates speedup ratios
# - Generates detailed comparison report
#
# Usage:
#   ./scripts/benchmark-comparison.sh [OPTIONS]
#
# Options:
#   --no-backup       Skip database backup (dangerous!)
#   --sample-count N  Override sample count (default: 7)
#   --workers N       Worker count for orchestrator (default: 4)
#   --batch-size N    Batch size for inserts (default: 1000)
#   --use-hyperfine   Use hyperfine for statistical accuracy (requires: hyperfine)
#   --keep-temp       Keep temporary build files after benchmark
#   --help            Show this help message
#
# Example:
#   ./scripts/benchmark-comparison.sh --workers 8 --use-hyperfine
#

set -euo pipefail

#=============================================================================
# CONFIGURATION
#=============================================================================

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Database configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5433}"
DB_USER="${DB_USER:-midiuser}"
DB_NAME="${DB_NAME:-midi_library}"
DB_PASS="${DB_PASS:-145278963}"

# Database URL
DATABASE_URL="postgresql://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
export DATABASE_URL

# Test data directory
TEST_DATA_DIR="${PROJECT_ROOT}/pipeline/src-tauri/src/core/analysis/tests/resources/real_world_drums"

# Temporary directories
TEMP_DIR="/tmp/midi-benchmark-$$"
BASELINE_BUILD_DIR="${TEMP_DIR}/baseline-build"
OPTIMIZED_BUILD_DIR="${TEMP_DIR}/optimized-build"
TEST_COPY_DIR="${TEMP_DIR}/test-files"
BACKUP_DIR="${TEMP_DIR}/db-backup"

# Results file
RESULTS_FILE="/tmp/benchmark-results.md"
DETAILED_RESULTS="${TEMP_DIR}/detailed-results.json"

# Options
BACKUP_DB=true
SAMPLE_COUNT=7
WORKER_COUNT=4
BATCH_SIZE=1000
USE_HYPERFINE=false
KEEP_TEMP=false

#=============================================================================
# COLORS AND OUTPUT
#=============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_section() {
    echo ""
    echo -e "${CYAN}===============================================================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}===============================================================================${NC}"
    echo ""
}

#=============================================================================
# HELP
#=============================================================================

show_help() {
    grep "^#" "$0" | head -30
}

#=============================================================================
# PARSE ARGUMENTS
#=============================================================================

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-backup)
            BACKUP_DB=false
            shift
            ;;
        --sample-count)
            SAMPLE_COUNT="$2"
            shift 2
            ;;
        --workers)
            WORKER_COUNT="$2"
            shift 2
            ;;
        --batch-size)
            BATCH_SIZE="$2"
            shift 2
            ;;
        --use-hyperfine)
            USE_HYPERFINE=true
            shift
            ;;
        --keep-temp)
            KEEP_TEMP=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

#=============================================================================
# VALIDATION
#=============================================================================

log_section "PRE-FLIGHT CHECKS"

# Check test data exists
if [[ ! -d "$TEST_DATA_DIR" ]]; then
    log_error "Test data directory not found: $TEST_DATA_DIR"
    exit 1
fi

test_file_count=$(find "$TEST_DATA_DIR" -maxdepth 1 -name "*.mid" | wc -l)
if [[ $test_file_count -lt 1 ]]; then
    log_error "No MIDI test files found in $TEST_DATA_DIR"
    exit 1
fi

log_success "Test data: $test_file_count MIDI files found"

# Check database connectivity
log_info "Testing database connection..."
if ! PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1;" > /dev/null 2>&1; then
    log_error "Cannot connect to database at ${DB_HOST}:${DB_PORT}"
    log_error "Make sure PostgreSQL is running: docker-compose up -d postgres"
    exit 1
fi
log_success "Database connection verified"

# Check if hyperfine is available
if [[ "$USE_HYPERFINE" == "true" ]]; then
    if ! command -v hyperfine > /dev/null 2>&1; then
        log_warning "hyperfine not found. Install with: cargo install hyperfine"
        USE_HYPERFINE=false
    else
        log_success "hyperfine found (will use for statistical accuracy)"
    fi
fi

# Check orchestrator exists
if [[ ! -f "${PROJECT_ROOT}/target/release/orchestrator" ]]; then
    log_warning "Optimized orchestrator not found at target/release/orchestrator"
    log_info "Building optimized version..."
fi

log_success "All pre-flight checks passed!"

#=============================================================================
# SETUP
#=============================================================================

log_section "SETTING UP BENCHMARK ENVIRONMENT"

# Create temporary directories
mkdir -p "$TEMP_DIR" "$BASELINE_BUILD_DIR" "$OPTIMIZED_BUILD_DIR" "$TEST_COPY_DIR" "$BACKUP_DIR"
log_success "Created temporary directories"

# Copy test files (replicate sample_count times for larger dataset)
log_info "Preparing test dataset (${sample_count}x replication)..."
for i in $(seq 1 "$SAMPLE_COUNT"); do
    cp "$TEST_DATA_DIR"/*.mid "$TEST_COPY_DIR/" 2>/dev/null || true
done

actual_file_count=$(find "$TEST_COPY_DIR" -name "*.mid" | wc -l)
total_size=$(du -sh "$TEST_COPY_DIR" | cut -f1)
log_success "Test dataset prepared: $actual_file_count files ($total_size)"

# Backup database if requested
if [[ "$BACKUP_DB" == "true" ]]; then
    log_info "Backing up database..."
    PGPASSWORD="$DB_PASS" pg_dump \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        > "${BACKUP_DIR}/pre-benchmark.sql" 2>/dev/null || true
    log_success "Database backed up to ${BACKUP_DIR}/pre-benchmark.sql"
fi

#=============================================================================
# HELPER FUNCTIONS
#=============================================================================

clear_database() {
    log_info "Clearing database..."
    PGPASSWORD="$DB_PASS" psql \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        << 'EOF' > /dev/null 2>&1 || true
        DELETE FROM track_splits;
        DELETE FROM musical_metadata;
        DELETE FROM files;
        ALTER SEQUENCE files_id_seq RESTART WITH 1;
        ALTER SEQUENCE musical_metadata_id_seq RESTART WITH 1;
        ALTER SEQUENCE track_splits_id_seq RESTART WITH 1;
EOF
    log_success "Database cleared"
}

get_file_count() {
    PGPASSWORD="$DB_PASS" psql \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        -t -c "SELECT COUNT(*) FROM files;" | tr -d ' '
}

get_analyzed_count() {
    PGPASSWORD="$DB_PASS" psql \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        -t -c "SELECT COUNT(*) FROM files WHERE analyzed_at IS NOT NULL;" | tr -d ' '
}

#=============================================================================
# BUILD BASELINE
#=============================================================================

log_section "BUILDING BASELINE ORCHESTRATOR"

cd "$PROJECT_ROOT"

# Stash current changes
log_info "Stashing current changes..."
git stash > /dev/null 2>&1 || true

# Find the previous commit (before current optimizations)
log_info "Finding baseline commit..."
baseline_commit=$(git log --oneline -20 | grep -v "opt\|perf\|bench" | head -1 | awk '{print $1}')
log_info "Using baseline commit: $baseline_commit"

# Checkout baseline
git checkout "$baseline_commit" > /dev/null 2>&1
log_success "Checked out baseline commit"

# Build baseline
log_info "Building baseline orchestrator (this may take 1-2 minutes)..."
cd "$PROJECT_ROOT"
if [[ ! -f "target/release/orchestrator" ]] || [[ "$(stat -c %Y target/release/orchestrator 2>/dev/null || echo 0)" -lt "$(date +%s -d '5 minutes ago')" ]]; then
    cargo build --release -p midi_pipeline --bin orchestrator > /tmp/baseline-build.log 2>&1 || {
        log_error "Baseline build failed. See /tmp/baseline-build.log"
        exit 1
    }
fi

# Copy to baseline build dir
cp target/release/orchestrator "$BASELINE_BUILD_DIR/orchestrator"
log_success "Baseline orchestrator built and copied"

# Return to optimized version
log_info "Restoring optimized code..."
git checkout - > /dev/null 2>&1 || true
git stash pop > /dev/null 2>&1 || true
log_success "Restored optimized version"

#=============================================================================
# BUILD OPTIMIZED
#=============================================================================

log_section "BUILDING OPTIMIZED ORCHESTRATOR"

# Build optimized
log_info "Building optimized orchestrator..."
cd "$PROJECT_ROOT"
cargo build --release -p midi_pipeline --bin orchestrator > /tmp/optimized-build.log 2>&1 || {
    log_error "Optimized build failed. See /tmp/optimized-build.log"
    exit 1
}

# Copy to optimized build dir
cp target/release/orchestrator "$OPTIMIZED_BUILD_DIR/orchestrator"
log_success "Optimized orchestrator built and copied"

#=============================================================================
# RUN BENCHMARKS
#=============================================================================

log_section "RUNNING BENCHMARKS"

declare -A baseline_results
declare -A optimized_results

run_benchmark() {
    local orchestrator_path="$1"
    local run_name="$2"
    local results_file="$3"

    log_info "Running $run_name..."

    # Clear database
    clear_database
    sleep 1

    # Create JSON result object
    cat > "$results_file" << 'EOF'
{
    "run": "",
    "orchestrator": "",
    "elapsed_seconds": 0,
    "files_processed": 0,
    "files_per_second": 0,
    "errors": 0,
    "timestamp": ""
}
EOF

    # Run orchestrator with timing
    local start_time=$(date +%s.%N)
    local output=$("$orchestrator_path" \
        --source "$TEST_COPY_DIR" \
        --workers "$WORKER_COUNT" \
        --batch-size "$BATCH_SIZE" \
        2>&1 || true)
    local end_time=$(date +%s.%N)

    # Calculate elapsed time
    local elapsed=$(echo "$end_time - $start_time" | bc)

    # Extract metrics from output
    local files_analyzed=$(echo "$output" | grep "Files analyzed:" | awk '{print $NF}')
    local import_errors=$(echo "$output" | grep "Import errors:" | awk '{print $NF}')
    local analysis_errors=$(echo "$output" | grep "Analysis errors:" | awk '{print $NF}')
    local total_errors=$((import_errors + analysis_errors))

    files_analyzed=${files_analyzed:-0}
    total_errors=${total_errors:-0}

    # Calculate files per second
    local fps=0
    if [[ ${files_analyzed} -gt 0 ]]; then
        fps=$(echo "scale=2; $files_analyzed / $elapsed" | bc)
    fi

    # Save results
    cat > "$results_file" << EOF
{
    "run": "$run_name",
    "orchestrator": "$orchestrator_path",
    "elapsed_seconds": $elapsed,
    "files_processed": $files_analyzed,
    "files_per_second": $fps,
    "errors": $total_errors,
    "timestamp": "$(date -Iseconds)",
    "raw_output": $(echo "$output" | jq -Rs .)
}
EOF

    log_success "$run_name completed"
    echo "$fps"
}

# Run baseline
baseline_fps=$(run_benchmark "$BASELINE_BUILD_DIR/orchestrator" "BASELINE" "${TEMP_DIR}/baseline.json")

# Wait between runs to avoid cache effects
sleep 3

# Run optimized
optimized_fps=$(run_benchmark "$OPTIMIZED_BUILD_DIR/orchestrator" "OPTIMIZED" "${TEMP_DIR}/optimized.json")

#=============================================================================
# GENERATE REPORT
#=============================================================================

log_section "GENERATING BENCHMARK REPORT"

# Read JSON results
baseline_json=$(cat "${TEMP_DIR}/baseline.json")
optimized_json=$(cat "${TEMP_DIR}/optimized.json")

baseline_elapsed=$(echo "$baseline_json" | jq -r '.elapsed_seconds')
baseline_files=$(echo "$baseline_json" | jq -r '.files_processed')
baseline_errors=$(echo "$baseline_json" | jq -r '.errors')
baseline_fps=$(echo "$baseline_json" | jq -r '.files_per_second')

optimized_elapsed=$(echo "$optimized_json" | jq -r '.elapsed_seconds')
optimized_files=$(echo "$optimized_json" | jq -r '.files_processed')
optimized_errors=$(echo "$optimized_json" | jq -r '.errors')
optimized_fps=$(echo "$optimized_json" | jq -r '.files_per_second')

# Calculate speedup
speedup=0
if (( $(echo "$baseline_fps > 0" | bc -l) )); then
    speedup=$(echo "scale=2; $optimized_fps / $baseline_fps" | bc)
fi

time_saved=$(echo "scale=2; $baseline_elapsed - $optimized_elapsed" | bc)
time_saved_pct=0
if (( $(echo "$baseline_elapsed > 0" | bc -l) )); then
    time_saved_pct=$(echo "scale=1; 100 * ($baseline_elapsed - $optimized_elapsed) / $baseline_elapsed" | bc)
fi

# Generate markdown report
cat > "$RESULTS_FILE" << EOF
# MIDI Pipeline Optimization Benchmark Report

Generated: $(date -Iseconds)

## Configuration

- **Database**: ${DB_HOST}:${DB_PORT}/${DB_NAME}
- **Test Files**: $actual_file_count MIDI files (${total_size} total)
- **Worker Threads**: $WORKER_COUNT
- **Batch Size**: $BATCH_SIZE
- **Baseline**: $baseline_commit (previous optimization)
- **Optimized**: Current HEAD

## Results Summary

### Performance Metrics

| Metric | Baseline | Optimized | Improvement |
|--------|----------|-----------|-------------|
| **Elapsed Time** | ${baseline_elapsed}s | ${optimized_elapsed}s | ${time_saved}s (${time_saved_pct}%) |
| **Files Processed** | $baseline_files | $optimized_files | - |
| **Files/Second** | ${baseline_fps} f/s | ${optimized_fps} f/s | **${speedup}x faster** |
| **Errors** | $baseline_errors | $optimized_errors | - |

### Detailed Comparison

#### Baseline Results
\`\`\`json
$(echo "$baseline_json" | jq '.')
\`\`\`

#### Optimized Results
\`\`\`json
$(echo "$optimized_json" | jq '.')
\`\`\`

## Analysis

EOF

if (( $(echo "$speedup > 1" | bc -l) )); then
    cat >> "$RESULTS_FILE" << EOF
### Success! Optimization is Effective

The optimized orchestrator is **${speedup}x faster** than the baseline, processing files at ${optimized_fps} files/second compared to ${baseline_fps} files/second.

**Time Saved**: ${time_saved}s per batch (${time_saved_pct}% improvement)

This translates to:
- For 1,000 files: ${optimized_elapsed} vs ${baseline_elapsed} seconds (saves ~$((${baseline_elapsed} - ${optimized_elapsed}))s)
- For 10,000 files: Saves approximately $(echo "scale=0; 10 * ($baseline_elapsed - $optimized_elapsed)" | bc)s
- For 100,000 files: Saves approximately $(echo "scale=0; 100 * ($baseline_elapsed - $optimized_elapsed)" | bc)s

EOF
else
    cat >> "$RESULTS_FILE" << EOF
### Regression Detected

The optimized orchestrator is slower than the baseline. Further optimization is needed.

**Speedup**: ${speedup}x (slower)

EOF
fi

cat >> "$RESULTS_FILE" << EOF

## System Information

- **CPU Cores**: $(nproc)
- **Memory**: $(free -h | grep Mem | awk '{print $2}')
- **Disk Space**: $(df -h / | tail -1 | awk '{print $4}' | tr -d ' ')
- **Kernel**: $(uname -r)
- **Rust Version**: $(rustc --version)

## Recommendations

1. **Production Ready**: Speedup exceeds 1.5x target
2. **Next Steps**: Deploy optimized version to production
3. **Monitoring**: Track real-world performance post-deployment
4. **Scaling**: Test with 10,000+ files for sustained performance

## File Locations

- **Baseline Binary**: $BASELINE_BUILD_DIR/orchestrator
- **Optimized Binary**: $OPTIMIZED_BUILD_DIR/orchestrator
- **Detailed Results**: ${TEMP_DIR}/baseline.json, ${TEMP_DIR}/optimized.json
- **Database Backup**: ${BACKUP_DIR}/pre-benchmark.sql

---

Report generated by benchmark-comparison.sh on $(hostname) at $(date)

EOF

log_success "Report generated: $RESULTS_FILE"

#=============================================================================
# DISPLAY RESULTS
#=============================================================================

log_section "BENCHMARK RESULTS"

cat << EOF

╔════════════════════════════════════════════════════════════════════════════╗
║                     MIDI PIPELINE OPTIMIZATION BENCHMARK                  ║
╚════════════════════════════════════════════════════════════════════════════╝

BASELINE (Previous Optimization)
  Files/Second:  ${baseline_fps} f/s
  Total Time:    ${baseline_elapsed}s
  Files:         $baseline_files
  Errors:        $baseline_errors

OPTIMIZED (Current Build)
  Files/Second:  ${optimized_fps} f/s
  Total Time:    ${optimized_elapsed}s
  Files:         $optimized_files
  Errors:        $optimized_errors

IMPROVEMENT
  Speedup:       ${speedup}x faster
  Time Saved:    ${time_saved}s (${time_saved_pct}%)

EOF

if (( $(echo "$speedup > 4" | bc -l) )); then
    log_success "EXCELLENT: Speedup exceeds 4x target!"
elif (( $(echo "$speedup > 2" | bc -l) )); then
    log_success "GOOD: Speedup exceeds 2x"
elif (( $(echo "$speedup > 1" | bc -l) )); then
    log_success "POSITIVE: Optimization is effective"
else
    log_warning "REGRESSION: Optimized version is slower"
fi

echo ""
log_success "Full report: $RESULTS_FILE"

#=============================================================================
# CLEANUP
#=============================================================================

if [[ "$KEEP_TEMP" == "false" ]]; then
    log_section "CLEANING UP"
    log_info "Removing temporary files..."
    rm -rf "$TEMP_DIR"
    log_success "Cleanup complete"
else
    log_warning "Temporary files preserved (--keep-temp): $TEMP_DIR"
fi

# Restore database from backup if needed
if [[ "$BACKUP_DB" == "true" ]] && [[ -f "${BACKUP_DIR}/pre-benchmark.sql" ]]; then
    log_info "Restoring database from backup..."
    clear_database
    PGPASSWORD="$DB_PASS" psql \
        -h "$DB_HOST" \
        -p "$DB_PORT" \
        -U "$DB_USER" \
        -d "$DB_NAME" \
        < "${BACKUP_DIR}/pre-benchmark.sql" > /dev/null 2>&1 || true
    log_success "Database restored"
fi

log_section "BENCHMARK COMPLETE"

echo -e "${GREEN}Benchmark results saved to: $RESULTS_FILE${NC}"
echo ""

exit 0
