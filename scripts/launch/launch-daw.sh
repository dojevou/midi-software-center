#!/bin/bash
# MIDI Library System - DAW Launcher
# Archetype: Task-O-Matic (Complete standalone task)
# Starts DAW application with required services
#
# Usage: ./scripts/launch/launch-daw.sh
# Example: ./scripts/launch/launch-daw.sh

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
# CLEANUP FUNCTION
# ============================================================================

cleanup_services() {
    print_status "🛑 Stopping any existing DAW services..."

    # Kill any running vite/tauri processes for DAW
    pkill -f "vite.*5174" 2>/dev/null || true
    pkill -f "tauri.*daw" 2>/dev/null || true
    fuser -k 5174/tcp 2>/dev/null || true

    # Remove PID file if exists
    [ -f /tmp/midi-daw.pid ] && rm /tmp/midi-daw.pid

    sleep 2
}

# ============================================================================
# FUNCTION: START DATABASE
# ============================================================================

start_database() {
    print_status "🗄️  Starting PostgreSQL and Meilisearch..."

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
# FUNCTION: CHECK MIDI HARDWARE
# ============================================================================

check_midi_hardware() {
    print_status "🎹 Checking MIDI hardware..."

    # Check if MOCK_MIDI_HARDWARE is enabled in .env
    if [ "${MOCK_MIDI_HARDWARE:-false}" = "true" ]; then
        print_warning "MOCK_MIDI_HARDWARE enabled - using virtual MIDI device"
        return 0
    fi

    # Check for ALSA MIDI
    if command -v aconnect &>/dev/null; then
        echo ""
        aconnect -l
        echo ""

        # Check if any MIDI devices are connected
        if aconnect -l | grep -q "client"; then
            print_success "MIDI hardware detected"
        else
            print_warning "No MIDI devices found"
            echo "You can enable MOCK_MIDI_HARDWARE=true in .env for testing"
        fi
    else
        print_warning "ALSA not found - MIDI features may be limited"
        echo "Install ALSA: sudo apt-get install alsa-utils"
        echo "Or enable MOCK_MIDI_HARDWARE=true in .env for testing"
    fi
}

# ============================================================================
# FUNCTION: START DAW
# ============================================================================

start_daw() {
    print_status "🎹 Starting DAW application..."

    if [ ! -d "$DAW_DIR" ]; then
        print_error "DAW directory not found: $DAW_DIR"
        return 1
    fi

    cd "$DAW_DIR"

    # Check if pnpm is available
    if ! command -v pnpm &>/dev/null; then
        print_error "pnpm not found. Please install pnpm: https://pnpm.io/installation"
        return 1
    fi

    # Start DAW
    print_status "🚀 Launching DAW Tauri dev server..."
    print_status "📋 Output will be logged to: $LOGS_DIR/daw.log"
    echo ""

    # Export DATABASE_URL and other .env variables for the application
    export DATABASE_URL
    export MEILISEARCH_URL
    export MEILISEARCH_API_KEY

    # Launch with logging
    pnpm tauri dev 2>&1 | tee "$LOGS_DIR/daw.log"
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================

main() {
    clear
    echo ""
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo -e "${MAGENTA}🎵  MIDI DAW Launcher${NC}"
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo ""
    echo -e "${BLUE}📁 Project: $PROJECT_ROOT${NC}"
    echo -e "${BLUE}📋 Logs: $LOGS_DIR${NC}"
    echo ""

    # Cleanup any existing services
    cleanup_services

    # Start database
    if ! start_database; then
        print_error "Cannot continue without database"
        exit 1
    fi

    echo ""

    # Check MIDI hardware
    check_midi_hardware

    echo ""

    # Start DAW (this will block and show output)
    if ! start_daw; then
        print_error "Failed to start DAW"
        exit 1
    fi
}

# Cleanup handler for Ctrl+C
cleanup_on_exit() {
    echo ""
    print_status "DAW stopped."
    exit 0
}

trap cleanup_on_exit INT

main
