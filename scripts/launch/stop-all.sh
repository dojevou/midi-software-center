#!/bin/bash
# MIDI Library System - Stop All Services Script

set -e  # Exit on error

# Configuration
PROJECT_ROOT="$HOME/projects/midi-library-system"
DATABASE_DIR="$PROJECT_ROOT/database"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Function to print colored messages
print_status() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to stop backend
stop_backend() {
    print_status "🛑 Stopping backend..."

    local stopped=0

    # Try to stop using PID file
    if [ -f /tmp/midi-backend.pid ]; then
        local backend_pid=$(cat /tmp/midi-backend.pid)
        if ps -p "$backend_pid" &>/dev/null; then
            kill "$backend_pid" 2>/dev/null || kill -9 "$backend_pid" 2>/dev/null
            # Wait for process to stop
            local count=0
            while ps -p "$backend_pid" &>/dev/null && [ $count -lt 10 ]; do
                sleep 0.5
                count=$((count + 1))
            done
            if ! ps -p "$backend_pid" &>/dev/null; then
                print_success "Backend stopped"
                stopped=1
            fi
        fi
        rm /tmp/midi-backend.pid
    fi

    # Try to find and kill by process name if not already stopped
    if [ $stopped -eq 0 ]; then
        local pids=$(pgrep -f "cargo.*tauri" || true)
        if [ -n "$pids" ]; then
            echo "$pids" | xargs kill 2>/dev/null || echo "$pids" | xargs kill -9 2>/dev/null
            sleep 1
            if ! pgrep -f "cargo.*tauri" &>/dev/null; then
                print_success "Backend stopped"
                stopped=1
            fi
        fi
    fi

    if [ $stopped -eq 0 ]; then
        print_warning "Backend was not running"
    fi
}

# Function to stop frontend (if separate)
stop_frontend() {
    print_status "🛑 Stopping frontend..."

    local stopped=0

    # Try to stop using PID file
    if [ -f /tmp/midi-frontend.pid ]; then
        local frontend_pid=$(cat /tmp/midi-frontend.pid)
        if ps -p "$frontend_pid" &>/dev/null; then
            kill "$frontend_pid" 2>/dev/null || kill -9 "$frontend_pid" 2>/dev/null
            # Wait for process to stop
            local count=0
            while ps -p "$frontend_pid" &>/dev/null && [ $count -lt 10 ]; do
                sleep 0.5
                count=$((count + 1))
            done
            if ! ps -p "$frontend_pid" &>/dev/null; then
                print_success "Frontend stopped"
                stopped=1
            fi
        fi
        rm /tmp/midi-frontend.pid
    fi

    # Try to find and kill by process name if not already stopped
    if [ $stopped -eq 0 ]; then
        local pids=$(pgrep -f "npm.*dev" || true)
        if [ -n "$pids" ]; then
            echo "$pids" | xargs kill 2>/dev/null || echo "$pids" | xargs kill -9 2>/dev/null
            sleep 1
            if ! pgrep -f "npm.*dev" &>/dev/null; then
                print_success "Frontend stopped"
                stopped=1
            fi
        fi
    fi

    if [ $stopped -eq 0 ]; then
        print_warning "Frontend was not running"
    fi
}

# Function to stop database
stop_database() {
    print_status "🛑 Stopping database..."

    cd "$DATABASE_DIR"

    # Check if docker-compose is available
    if ! command -v docker-compose &>/dev/null; then
        print_warning "docker-compose not found, trying docker compose..."
        if docker compose ps &>/dev/null 2>&1; then
            docker compose down
            print_success "Database stopped"
            return 0
        else
            print_warning "Could not stop database (docker compose not available)"
            return 1
        fi
    fi

    # Check if containers are running
    if docker-compose ps | grep -q "Up"; then
        docker-compose down
        print_success "Database stopped"
    else
        print_warning "Database was not running"
    fi
}

# Main execution
main() {
    echo ""
    echo -e "${MAGENTA}🛑 ==================================${NC}"
    echo -e "${MAGENTA}🛑  Stopping MIDI Library System${NC}"
    echo -e "${MAGENTA}🛑 ==================================${NC}"
    echo ""

    # Stop services in reverse order (backend -> frontend -> database)
    stop_backend
    echo ""

    # Uncomment if you have a separate frontend
    # stop_frontend
    # echo ""

    stop_database
    echo ""

    # Clean up any remaining PID files
    rm -f /tmp/midi-backend.pid /tmp/midi-frontend.pid

    echo -e "${MAGENTA}🛑 ==================================${NC}"
    print_success "All services stopped"
    echo -e "${MAGENTA}🛑 ==================================${NC}"
    echo ""
}

# Run main function
main
