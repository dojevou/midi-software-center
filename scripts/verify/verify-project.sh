#!/bin/bash
#
# MIDI Software Center - Comprehensive Project Verification
#
# Usage:
#   ./scripts/verify/verify-project.sh [OPTIONS]
#
# Options:
#   --all         Run all verification checks (default)
#   --quick       Run quick checks only (no database)
#   --security    Run security checks only
#   --performance Run performance checks only
#   --database    Run database checks only
#   --ci          CI mode - fail fast on any error
#   --verbose     Enable verbose output
#   --help        Show this help message

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
RUN_ALL=true
QUICK_MODE=false
SECURITY_ONLY=false
PERFORMANCE_ONLY=false
DATABASE_ONLY=false
CI_MODE=false
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            RUN_ALL=true
            shift
            ;;
        --quick)
            QUICK_MODE=true
            RUN_ALL=false
            shift
            ;;
        --security)
            SECURITY_ONLY=true
            RUN_ALL=false
            shift
            ;;
        --performance)
            PERFORMANCE_ONLY=true
            RUN_ALL=false
            shift
            ;;
        --database)
            DATABASE_ONLY=true
            RUN_ALL=false
            shift
            ;;
        --ci)
            CI_MODE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            head -30 "$0" | tail -25
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$PROJECT_ROOT"

# Database connection
export DATABASE_URL="${DATABASE_URL:-postgresql://midiuser:145278963@localhost:5433/midi_library}"

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     MIDI Software Center - Project Verification              ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Track results
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNINGS=0

check_result() {
    local name="$1"
    local result="$2"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [ "$result" -eq 0 ]; then
        echo -e "  ${GREEN}✓${NC} $name"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        echo -e "  ${RED}✗${NC} $name"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        if [ "$CI_MODE" = true ]; then
            echo -e "${RED}CI Mode: Failing fast on error${NC}"
            exit 1
        fi
    fi
}

warn_result() {
    local name="$1"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    echo -e "  ${YELLOW}⚠${NC} $name"
    WARNINGS=$((WARNINGS + 1))
}

# ============================================================================
# 1. Basic Structure Checks
# ============================================================================

if [ "$RUN_ALL" = true ] || [ "$QUICK_MODE" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 1. Basic Structure Checks                                   │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    # Check required directories
    for dir in app pipeline daw shared database verification; do
        if [ -d "$dir" ]; then
            check_result "Directory: $dir" 0
        else
            check_result "Directory: $dir" 1
        fi
    done

    # Check required files
    for file in Cargo.toml CLAUDE.md app/package.json; do
        if [ -f "$file" ]; then
            check_result "File: $file" 0
        else
            check_result "File: $file" 1
        fi
    done

    echo ""
fi

# ============================================================================
# 2. Rust Compilation Check
# ============================================================================

if [ "$RUN_ALL" = true ] || [ "$QUICK_MODE" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 2. Rust Compilation Check                                   │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    if cargo check --workspace 2>/dev/null; then
        check_result "Cargo check (workspace)" 0
    else
        check_result "Cargo check (workspace)" 1
    fi

    # Check clippy warnings
    if cargo clippy --workspace -- -D warnings 2>/dev/null; then
        check_result "Clippy (no warnings)" 0
    else
        warn_result "Clippy has warnings"
    fi

    echo ""
fi

# ============================================================================
# 3. Frontend Check
# ============================================================================

if [ "$RUN_ALL" = true ] || [ "$QUICK_MODE" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 3. Frontend Check                                           │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    cd "$PROJECT_ROOT/app"

    # Check dependencies installed
    if [ -d "node_modules" ]; then
        check_result "Node modules installed" 0
    else
        warn_result "Node modules not installed (run: pnpm install)"
    fi

    # Check TypeScript
    if pnpm exec tsc --noEmit 2>/dev/null; then
        check_result "TypeScript compilation" 0
    else
        check_result "TypeScript compilation" 1
    fi

    cd "$PROJECT_ROOT"
    echo ""
fi

# ============================================================================
# 4. Security Checks
# ============================================================================

if [ "$RUN_ALL" = true ] || [ "$SECURITY_ONLY" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 4. Security Checks                                          │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    # Check for hardcoded secrets
    SECRETS_FOUND=0
    if grep -rE "(password|api_key|secret).*=.*['\"][^'\"]+['\"]" \
        --include="*.rs" --include="*.ts" --include="*.js" \
        --exclude-dir=target --exclude-dir=node_modules \
        pipeline/ daw/ shared/ app/src/ 2>/dev/null | grep -v "env::var\|std::env\|//" > /dev/null; then
        SECRETS_FOUND=1
    fi

    if [ "$SECRETS_FOUND" -eq 0 ]; then
        check_result "No hardcoded secrets" 0
    else
        check_result "No hardcoded secrets" 1
    fi

    # Check for cargo-audit (if installed)
    if command -v cargo-audit &> /dev/null; then
        if cargo audit 2>/dev/null; then
            check_result "Cargo audit (no vulnerabilities)" 0
        else
            check_result "Cargo audit (no vulnerabilities)" 1
        fi
    else
        warn_result "cargo-audit not installed (run: cargo install cargo-audit)"
    fi

    # Check for .unwrap() in command handlers
    UNWRAP_COUNT=$(grep -r "\.unwrap()" --include="*.rs" \
        pipeline/src-tauri/src/commands/ \
        daw/src-tauri/src/commands/ 2>/dev/null | wc -l || echo "0")

    if [ "$UNWRAP_COUNT" -lt 5 ]; then
        check_result "Minimal .unwrap() in commands ($UNWRAP_COUNT)" 0
    else
        warn_result ".unwrap() in commands: $UNWRAP_COUNT occurrences"
    fi

    echo ""
fi

# ============================================================================
# 5. Performance Checks
# ============================================================================

if [ "$RUN_ALL" = true ] || [ "$PERFORMANCE_ONLY" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 5. Performance Checks                                       │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    # Check for blocking operations in async code
    BLOCKING_OPS=$(grep -rE "std::thread::sleep|std::fs::(read|write)" \
        --include="*.rs" \
        pipeline/src-tauri/src/commands/ \
        daw/src-tauri/src/commands/ 2>/dev/null | wc -l || echo "0")

    if [ "$BLOCKING_OPS" -eq 0 ]; then
        check_result "No blocking ops in async code" 0
    else
        warn_result "Blocking operations in async code: $BLOCKING_OPS"
    fi

    # Check for release build
    if [ -f "target/release/midi-verification" ]; then
        check_result "Release build exists" 0
    else
        warn_result "Release build not found (run: cargo build --release)"
    fi

    echo ""
fi

# ============================================================================
# 6. Database Checks
# ============================================================================

if [ "$RUN_ALL" = true ] || [ "$DATABASE_ONLY" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 6. Database Checks                                          │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    # Check database connection
    if psql "$DATABASE_URL" -c "SELECT 1" &>/dev/null; then
        check_result "Database connection" 0

        # Check required tables
        for table in files musical_metadata tags file_tags; do
            if psql "$DATABASE_URL" -c "SELECT 1 FROM $table LIMIT 1" &>/dev/null; then
                check_result "Table: $table" 0
            else
                check_result "Table: $table" 1
            fi
        done

        # Check row counts
        FILE_COUNT=$(psql "$DATABASE_URL" -t -c "SELECT COUNT(*) FROM files" 2>/dev/null | tr -d ' ')
        if [ -n "$FILE_COUNT" ] && [ "$FILE_COUNT" -gt 0 ]; then
            check_result "Files in database: $FILE_COUNT" 0
        else
            warn_result "No files in database"
        fi

        # Check for orphan records
        ORPHANS=$(psql "$DATABASE_URL" -t -c "
            SELECT COUNT(*) FROM file_tags ft
            WHERE NOT EXISTS (SELECT 1 FROM files f WHERE f.id = ft.file_id)
        " 2>/dev/null | tr -d ' ')

        if [ -n "$ORPHANS" ] && [ "$ORPHANS" -eq 0 ]; then
            check_result "No orphan file_tags" 0
        else
            warn_result "Orphan file_tags: $ORPHANS"
        fi

    else
        check_result "Database connection" 1
        warn_result "Skipping database table checks"
    fi

    echo ""
fi

# ============================================================================
# 7. Run Verification Binary (if available)
# ============================================================================

if [ "$RUN_ALL" = true ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ 7. Full Verification Suite                                  │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────────┘${NC}"

    if [ -f "target/release/midi-verification" ]; then
        echo "  Running midi-verification..."
        if ./target/release/midi-verification --all 2>&1 | tail -20; then
            check_result "Verification suite" 0
        else
            check_result "Verification suite" 1
        fi
    else
        # Build and run
        echo "  Building verification tool..."
        if cargo build --release -p verification 2>/dev/null; then
            echo "  Running midi-verification..."
            if ./target/release/midi-verification --all 2>&1 | tail -20; then
                check_result "Verification suite" 0
            else
                check_result "Verification suite" 1
            fi
        else
            check_result "Build verification tool" 1
        fi
    fi

    echo ""
fi

# ============================================================================
# Summary
# ============================================================================

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                       SUMMARY                                ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Checks:  $TOTAL_CHECKS"
echo -e "  ${GREEN}Passed:${NC}        $PASSED_CHECKS"
echo -e "  ${RED}Failed:${NC}        $FAILED_CHECKS"
echo -e "  ${YELLOW}Warnings:${NC}      $WARNINGS"
echo ""

if [ "$FAILED_CHECKS" -eq 0 ]; then
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║               ALL CHECKS PASSED!                             ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║               $FAILED_CHECKS CHECK(S) FAILED                              ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
