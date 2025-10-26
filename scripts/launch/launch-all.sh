#!/bin/bash
# MIDI Library System - Main Launch Script
# Archetype: Task-O-Matic (Complete standalone task)
# Starts all services: Database, Pipeline, and/or DAW
#
# Usage: ./scripts/launch/launch-all.sh
# Example: ./scripts/launch/launch-all.sh

set -e  # Exit on error
set -u  # Error on undefined variables
set -o pipefail  # Catch errors in pipes

# ============================================================================
# AUTO-DETECT PROJECT ROOT
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# ============================================================================
# LOAD ENVIRONMENT CONFIGURATION
# ============================================================================

if [ -f "$PROJECT_ROOT/.env" ]; then
    # Export all variables from .env
    set -a
    source "$PROJECT_ROOT/.env"
    set +a
else
    echo "❌ Error: .env file not found at $PROJECT_ROOT/.env"
    echo "Please copy .env.example to .env and configure it:"
    echo "  cp .env.example .env"
    exit 1
fi

# ============================================================================
# CONFIGURATION FROM .ENV
# ============================================================================

# Parse DATABASE_URL to extract components
# Format: postgresql://username:password@host:port/database
if [[ $DATABASE_URL =~ postgresql://([^:]+):([^@]+)@([^:]+):([^/]+)/(.+) ]]; then
    DB_USER="${BASH_REMATCH[1]}"
    DB_PASSWORD="${BASH_REMATCH[2]}"
    DB_HOST="${BASH_REMATCH[3]}"
    DB_PORT="${BASH_REMATCH[4]}"
    DB_NAME="${BASH_REMATCH[5]}"
else
    echo "❌ Error: Invalid DATABASE_URL format in .env"
    echo "Expected: postgresql://username:password@host:port/database"
    exit 1
fi

# Set directory paths
DATABASE_DIR="$PROJECT_ROOT/database"
PIPELINE_DIR="$PROJECT_ROOT/pipeline"
DAW_DIR="$PROJECT_ROOT/daw"
LOGS_DIR="${LOG_DIR:-$PROJECT_ROOT/logs}"

# ============================================================================
# COLORS FOR OUTPUT
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

print_status() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Create logs directory
mkdir -p "$LOGS_DIR"

# ============================================================================
# FUNCTION: LAUNCH DATABASE
# ============================================================================

launch_database() {
    print_status "🗄️  Starting PostgreSQL database..."

    if [ ! -d "$DATABASE_DIR" ]; then
        print_error "Database directory not found: $DATABASE_DIR"
        return 1
    fi

    cd "$DATABASE_DIR"

    # Check if already running
    if docker-compose ps 2>/dev/null | grep -q "Up"; then
        print_warning "Database already running"
        return 0
    fi

    # Start database
    if ! docker-compose up -d 2>&1 | tee "$LOGS_DIR/database.log"; then
        print_error "Failed to start database container"
        return 1
    fi

    # Wait for database to be ready
    print_status "⏳ Waiting for database to be ready..."
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1;" &>/dev/null; then
            print_success "Database is ready!"
            return 0
        fi
        attempt=$((attempt + 1))
        echo -n "."
        sleep 1
    done

    echo ""
    print_error "Database failed to start within ${max_attempts} seconds"
    echo "Check logs at: $LOGS_DIR/database.log"
    return 1
}

# ============================================================================
# FUNCTION: LAUNCH PIPELINE
# ============================================================================

launch_pipeline() {
    print_status "⚙️  Starting Pipeline application..."

    if [ ! -d "$PIPELINE_DIR" ]; then
        print_error "Pipeline directory not found: $PIPELINE_DIR"
        return 1
    fi

    cd "$PIPELINE_DIR"

    # Check if already running
    if [ -f /tmp/midi-pipeline.pid ]; then
        local old_pid=$(cat /tmp/midi-pipeline.pid)
        if ps -p "$old_pid" &>/dev/null; then
            print_warning "Pipeline already running (PID: $old_pid)"
            return 0
        else
            rm /tmp/midi-pipeline.pid
        fi
    fi

    # Check if pnpm is available
    if ! command -v pnpm &>/dev/null; then
        print_error "pnpm not found. Please install pnpm: https://pnpm.io/installation"
        return 1
    fi

    # Start in background with logging
    print_status "Starting Pipeline Tauri dev server..."
    pnpm tauri dev > "$LOGS_DIR/pipeline.log" 2>&1 &
    local pipeline_pid=$!
    echo "$pipeline_pid" > /tmp/midi-pipeline.pid

    # Wait and verify
    sleep 5
    if ps -p "$pipeline_pid" &>/dev/null; then
        print_success "Pipeline started (PID: $pipeline_pid)"
        return 0
    else
        print_error "Pipeline failed to start - check logs at: $LOGS_DIR/pipeline.log"
        rm -f /tmp/midi-pipeline.pid
        return 1
    fi
}

# ============================================================================
# FUNCTION: LAUNCH DAW
# ============================================================================

launch_daw() {
    print_status "🎹 Starting DAW application..."

    if [ ! -d "$DAW_DIR" ]; then
        print_error "DAW directory not found: $DAW_DIR"
        return 1
    fi

    cd "$DAW_DIR"

    # Check if already running
    if [ -f /tmp/midi-daw.pid ]; then
        local old_pid=$(cat /tmp/midi-daw.pid)
        if ps -p "$old_pid" &>/dev/null; then
            print_warning "DAW already running (PID: $old_pid)"
            return 0
        else
            rm /tmp/midi-daw.pid
        fi
    fi

    # Check if pnpm is available
    if ! command -v pnpm &>/dev/null; then
        print_error "pnpm not found. Please install pnpm"
        return 1
    fi

    # Start in background with logging
    print_status "Starting DAW Tauri dev server..."
    pnpm tauri dev > "$LOGS_DIR/daw.log" 2>&1 &
    local daw_pid=$!
    echo "$daw_pid" > /tmp/midi-daw.pid

    # Wait and verify
    sleep 5
    if ps -p "$daw_pid" &>/dev/null; then
        print_success "DAW started (PID: $daw_pid)"
        return 0
    else
        print_error "DAW failed to start - check logs at: $LOGS_DIR/daw.log"
        rm -f /tmp/midi-daw.pid
        return 1
    fi
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

main() {
    clear
    echo ""
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo -e "${MAGENTA}🎵  MIDI Library System Launcher${NC}"
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo ""
    echo -e "${BLUE}📁 Project: $PROJECT_ROOT${NC}"
    echo -e "${BLUE}📋 Logs: $LOGS_DIR${NC}"
    echo ""

    # Launch database first
    if ! launch_database; then
        print_error "Cannot continue without database"
        exit 1
    fi

    echo ""
    print_status "Which application(s) to start?"
    echo "  1) Pipeline only"
    echo "  2) DAW only"
    echo "  3) Both Pipeline and DAW"
    echo "  4) Database only (already started)"
    read -p "Choice [1-4] (default: 3): " choice
    choice=${choice:-3}

    case $choice in
        1)
            launch_pipeline || exit 1
            ;;
        2)
            launch_daw || exit 1
            ;;
        3)
            launch_pipeline || exit 1
            echo ""
            launch_daw || exit 1
            ;;
        4)
            print_success "Database running"
            ;;
        *)
            print_error "Invalid choice"
            exit 1
            ;;
    esac

    # Summary
    echo ""
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo -e "${MAGENTA}🎵  Services Running${NC}"
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    print_success "Database: postgresql://$DB_HOST:$DB_PORT/$DB_NAME"

    [ "$choice" = "1" ] || [ "$choice" = "3" ] && print_success "Pipeline: http://localhost:5173"
    [ "$choice" = "2" ] || [ "$choice" = "3" ] && print_success "DAW: http://localhost:5174"

    echo ""
    echo -e "${BLUE}🛑 Stop: ./scripts/launch/stop-all.sh${NC}"
    echo -e "${BLUE}📊 Status: ./scripts/launch/status.sh${NC}"
    echo ""

    if [ "$choice" != "4" ]; then
        echo -e "${YELLOW}Press Ctrl+C to stop tailing (services continue running)${NC}"
        echo ""
        print_status "📋 Tailing logs..."
        echo ""

        case $choice in
            1) tail -f "$LOGS_DIR/pipeline.log" ;;
            2) tail -f "$LOGS_DIR/daw.log" ;;
            3) tail -f "$LOGS_DIR/pipeline.log" "$LOGS_DIR/daw.log" ;;
        esac
    fi
}

# Cleanup handler
cleanup() {
    echo ""
    print_status "Logs stopped. Services still running."
    echo "Use ./scripts/launch/stop-all.sh to stop."
    exit 0
}

trap cleanup INT
main
