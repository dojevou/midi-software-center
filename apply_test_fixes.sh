#!/bin/bash

################################################################################
# MIDI Software Center - Test Fixes Implementation Script
# Applies all compilation fixes systematically
# Usage: bash apply_test_fixes.sh [--dry-run] [--verify-only]
################################################################################

set -e  # Exit on error

PROJECT_ROOT="${PROJECT_ROOT:-.}"
TESTS_DIR="$PROJECT_ROOT/pipeline/src-tauri/tests"
DISABLED_DIR="$PROJECT_ROOT/_disabled_tests"
LOG_FILE="/tmp/test_fixes_$(date +%s).log"
DRY_RUN=false
VERIFY_ONLY=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

################################################################################
# UTILITY FUNCTIONS
################################################################################

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[✗]${NC} $*" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $*" | tee -a "$LOG_FILE"
}

apply_fix() {
    local file=$1
    local old=$2
    local new=$3
    local count
    
    if [ ! -f "$file" ]; then
        log_warning "File not found: $file"
        return 1
    fi
    
    if $DRY_RUN; then
        count=$(grep -c "$old" "$file" || true)
        if [ "$count" -gt 0 ]; then
            log_info "[DRY-RUN] Would replace $count occurrences in $(basename $file)"
            grep -n "$old" "$file" | head -3 >> "$LOG_FILE"
        fi
    else
        sed -i.bak "s|$old|$new|g" "$file"
        count=$(grep -c "$new" "$file" || true)
        log_success "Applied fix to $(basename $file) ($count occurrences)"
    fi
    
    return 0
}

backup_file() {
    local file=$1
    if [ -f "$file" ]; then
        cp "$file" "${file}.pre-fix-backup"
        log_success "Backed up: $(basename $file)"
    fi
}

################################################################################
# PHASE 1: BRACE ERRORS (3 files)
################################################################################

fix_brace_errors() {
    log_info "PHASE 1: Fixing brace errors..."
    
    local files=(
        "$TESTS_DIR/file_repository_test.rs:2443"
        "$TESTS_DIR/metadata_repository_test.rs:1812"
        "$TESTS_DIR/tag_repository_test.rs:1720"
    )
    
    for entry in "${files[@]}"; do
        IFS=':' read -r file line <<< "$entry"
        if [ -f "$file" ]; then
            backup_file "$file"
            
            if $DRY_RUN; then
                log_info "[DRY-RUN] Would remove extra brace at line $line in $(basename $file)"
            else
                # Get total lines
                total=$(wc -l < "$file")
                # Check if last line is extra brace
                last_line=$(tail -1 "$file")
                if [ "$last_line" = "}" ]; then
                    # Remove last line
                    head -n -1 "$file" > "$file.tmp" && mv "$file.tmp" "$file"
                    log_success "Removed extra brace from $(basename $file)"
                else
                    log_warning "Last line is not a brace in $(basename $file): '$last_line'"
                fi
            fi
        else
            log_error "File not found: $file"
        fi
    done
}

################################################################################
# PHASE 2: FIELD NAME CHANGES
################################################################################

fix_field_names() {
    log_info "PHASE 2: Fixing field name changes..."
    
    local test_files=(
        "$TESTS_DIR/journey_test.rs"
        "$TESTS_DIR/file_import_test.rs"
        "$TESTS_DIR/workflows_test.rs"
        "$TESTS_DIR/workflows_extended_test.rs"
        "$DISABLED_DIR/journey_test.rs"
        "$DISABLED_DIR/file_import_test.rs"
        "$DISABLED_DIR/workflows_test.rs"
        "$DISABLED_DIR/workflows_extended_test.rs"
    )
    
    # FileMetadata.file_id -> FileMetadata.id
    log_info "Fixing FileMetadata.file_id -> .id..."
    for file in "${test_files[@]}"; do
        if [ -f "$file" ]; then
            backup_file "$file"
            
            # Multiple patterns to catch
            apply_fix "$file" 'import_result\.file_id' 'import_result.id'
            apply_fix "$file" 'arrangement_result\.file_id' 'arrangement_result.id'
            apply_fix "$file" 'file\.file_id' 'file.id'
            apply_fix "$file" 'metadata\.file_id' 'metadata.id'
        fi
    done
    
    # file_path -> filepath
    log_info "Fixing file_path -> filepath..."
    for file in "${test_files[@]}"; do
        if [ -f "$file" ]; then
            apply_fix "$file" '\.file_path' '.filepath'
        fi
    done
    
    # file_size_bytes (usually already correct, but check)
    log_info "Checking file_size_bytes usage..."
    for file in "${test_files[@]}"; do
        if [ -f "$file" ]; then
            grep -n "\.file_size" "$file" >> "$LOG_FILE" 2>&1 || true
        fi
    done
}

################################################################################
# PHASE 3: IMPORT/USE STATEMENT FIXES
################################################################################

fix_imports() {
    log_info "PHASE 3: Fixing import statements..."
    
    local test_files=(
        "$TESTS_DIR/file_import_test.rs"
        "$TESTS_DIR/journey_test.rs"
        "$DISABLED_DIR/file_import_test.rs"
        "$DISABLED_DIR/journey_test.rs"
    )
    
    for file in "${test_files[@]}"; do
        if [ -f "$file" ]; then
            # Add generic to Emitter impl
            if grep -q "impl Emitter for" "$file"; then
                backup_file "$file"
                if $DRY_RUN; then
                    log_info "[DRY-RUN] Would add generic to Emitter impl in $(basename $file)"
                else
                    sed -i 's/impl Emitter for/impl<R: tauri::Runtime> Emitter<R> for/g' "$file"
                    log_success "Fixed Emitter trait in $(basename $file)"
                fi
            fi
        fi
    done
}

################################################################################
# PHASE 4: TAURI STATE ISSUES
################################################################################

create_test_helpers() {
    log_info "PHASE 4: Creating test helper utilities..."
    
    local helpers_file="$TESTS_DIR/test_helpers.rs"
    
    if $DRY_RUN; then
        log_info "[DRY-RUN] Would create test_helpers.rs"
        return 0
    fi
    
    if [ ! -f "$helpers_file" ]; then
        cat > "$helpers_file" << 'EOF'
//! Test helper utilities for MIDI software tests

use crate::AppState;
use sqlx::PgPool;
use std::sync::Arc;

/// Create an AppState for testing
pub async fn create_test_app_state(pool: PgPool) -> AppState {
    AppState {
        db: pool,
        // Initialize other fields as needed
    }
}

/// Setup test database connection pool
pub async fn setup_test_pool() -> Result<PgPool, sqlx::Error> {
    // Use test database URL from env
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/midi_test".to_string());
    
    PgPool::connect(&database_url).await
}

/// Cleanup test database after tests
pub async fn cleanup_test_pool(pool: &PgPool) {
    // Optionally clean up
    let _ = pool.execute("TRUNCATE TABLE files CASCADE").await;
}

/// Helper to avoid constructing tauri::State directly
/// Instead, use this to pass AppState to functions that accept &AppState
pub fn create_app_state_ref(pool: PgPool) -> Arc<AppState> {
    Arc::new(AppState {
        db: pool,
    })
}
EOF
        log_success "Created test helpers file: $helpers_file"
    else
        log_warning "Test helpers file already exists: $helpers_file"
    fi
}

################################################################################
# PHASE 5: FUNCTION SIGNATURE UPDATES
################################################################################

fix_function_signatures() {
    log_info "PHASE 5: Analyzing function signatures..."
    
    log_info "Checking get_file_count calls..."
    grep -rn "get_file_count(tauri::State" "$TESTS_DIR" 2>/dev/null | head -5 >> "$LOG_FILE" || true
    
    log_info "Checking get_file_details calls..."
    grep -rn "get_file_details(tauri::State" "$TESTS_DIR" 2>/dev/null | head -5 >> "$LOG_FILE" || true
    
    log_info "Note: Function signature fixes require manual review per function"
    log_warning "See source files in src/commands/ for actual signatures"
}

################################################################################
# PHASE 6: COMPILATION VERIFICATION
################################################################################

verify_compilation() {
    log_info "PHASE 6: Verifying compilation..."
    
    if [ -z "$(command -v cargo)" ]; then
        log_warning "Cargo not found - skipping compilation check"
        return 1
    fi
    
    if $DRY_RUN; then
        log_info "[DRY-RUN] Would run: cargo build --tests 2>&1 | grep error"
        return 0
    fi
    
    log_info "Running compilation check..."
    cd "$PROJECT_ROOT" || exit 1
    
    if cargo build --tests 2>&1 | grep -q "^error"; then
        log_error "Compilation errors still present"
        cargo build --tests 2>&1 | grep "^error" | head -10 >> "$LOG_FILE"
        return 1
    else
        log_success "Compilation successful!"
        return 0
    fi
}

################################################################################
# PHASE 7: TEST EXECUTION
################################################################################

run_tests() {
    log_info "PHASE 7: Running test suite..."
    
    if [ -z "$(command -v cargo)" ]; then
        log_warning "Cargo not found - skipping tests"
        return 1
    fi
    
    if $DRY_RUN; then
        log_info "[DRY-RUN] Would run: cargo test --lib --tests"
        return 0
    fi
    
    cd "$PROJECT_ROOT" || exit 1
    
    if cargo test --lib --tests 2>&1 | tee -a "$LOG_FILE" | grep -q "test result: ok"; then
        log_success "All tests passed!"
        return 0
    else
        log_error "Some tests failed"
        return 1
    fi
}

################################################################################
# HELPER: Show backup files
################################################################################

show_backups() {
    log_info "Backup files created during fixes:"
    find "$TESTS_DIR" -name "*.pre-fix-backup" -exec ls -lh {} \; | tee -a "$LOG_FILE"
}

################################################################################
# HELPER: Restore from backups
################################################################################

restore_from_backups() {
    log_warning "Restoring from backup files..."
    
    find "$TESTS_DIR" -name "*.pre-fix-backup" | while read -r backup; do
        original="${backup%.pre-fix-backup}"
        if [ -f "$backup" ]; then
            cp "$backup" "$original"
            log_success "Restored: $(basename $original)"
        fi
    done
}

################################################################################
# MAIN EXECUTION
################################################################################

main() {
    log_info "════════════════════════════════════════════════════════════"
    log_info "MIDI Software Center - Test Fixes Implementation"
    log_info "════════════════════════════════════════════════════════════"
    log_info "Project Root: $PROJECT_ROOT"
    log_info "Tests Directory: $TESTS_DIR"
    log_info "Log File: $LOG_FILE"
    if $DRY_RUN; then
        log_warning "DRY RUN MODE - No changes will be made"
    fi
    log_info "════════════════════════════════════════════════════════════"
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --verify-only)
                VERIFY_ONLY=true
                shift
                ;;
            --restore)
                restore_from_backups
                exit 0
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --dry-run      Show what changes would be made"
                echo "  --verify-only  Only verify, don't apply fixes"
                echo "  --restore      Restore from backup files"
                echo "  --help         Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    if [ ! -d "$TESTS_DIR" ]; then
        log_error "Tests directory not found: $TESTS_DIR"
        exit 1
    fi
    
    # Execute phases
    if ! $VERIFY_ONLY; then
        fix_brace_errors
        fix_field_names
        fix_imports
        create_test_helpers
    fi
    
    # Always run verification/analysis
    fix_function_signatures
    
    # Optional: compilation and tests
    if ! $DRY_RUN && ! $VERIFY_ONLY; then
        if verify_compilation; then
            run_tests
        else
            log_error "Compilation failed - not running tests"
        fi
    fi
    
    log_info "════════════════════════════════════════════════════════════"
    log_success "Script execution completed"
    log_info "Log file: $LOG_FILE"
    log_info "════════════════════════════════════════════════════════════"
    
    show_backups
}

# Run main
main "$@"
