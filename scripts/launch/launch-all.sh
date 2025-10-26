#!/bin/bash
# MIDI Library System - Main Launch Script
# Starts all services: Database, Backend, and optionally Frontend

set -e  # Exit on error

# Configuration
PROJECT_ROOT="$HOME/projects/midi-software-center"
DATABASE_DIR="$PROJECT_ROOT/database"
BACKEND_DIR="$PROJECT_ROOT/pipeline/src-tauri"
LOGS_DIR="$PROJECT_ROOT/logs"
DB_HOST="localhost"
DB_PORT="5433"
DB_USER="midiuser"
DB_PASSWORD="145278963"
DB_NAME="midi_library"

# Colors for output (optional but nice)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Create logs directory
mkdir -p "$LOGS_DIR"

# Function to print colored messages
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

# Function to launch database
launch_database() {
    print_status "🗄️  Starting PostgreSQL database..."

    cd "$DATABASE_DIR"

    # Check if already running
    if docker-compose ps | grep -q "Up"; then
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

# Function to launch backend/Tauri app
launch_backend() {
    print_status "⚙️  Starting backend/Tauri app..."

    cd "$BACKEND_DIR"

    # Check if already running
    if [ -f /tmp/midi-backend.pid ]; then
        local old_pid=$(cat /tmp/midi-backend.pid)
        if ps -p "$old_pid" &>/dev/null; then
            print_warning "Backend already running (PID: $old_pid)"
            return 0
        else
            # Remove stale PID file
            rm /tmp/midi-backend.pid
        fi
    elif pgrep -f "tauri dev" &>/dev/null; then
        print_warning "Backend already running (unknown PID)"
        return 0
    fi

    # Check if pnpm is available
    if ! command -v pnpm &>/dev/null; then
        print_error "pnpm not found. Please install pnpm."
        return 1
    fi

    # Start in background with logging
    print_status "Starting Tauri dev server (this may take a moment)..."
    pnpm tauri dev > "$LOGS_DIR/backend.log" 2>&1 &
    local backend_pid=$!
    echo "$backend_pid" > /tmp/midi-backend.pid

    # Wait a bit and check if process is still running
    sleep 5
    if ps -p "$backend_pid" &>/dev/null; then
        print_success "Backend started (PID: $backend_pid)"
        return 0
    else
        print_error "Backend failed to start"
        echo "Check logs at: $LOGS_DIR/backend.log"
        rm /tmp/midi-backend.pid
        return 1
    fi
}

# Function to launch frontend (if separate from Tauri)
# Uncomment if you have a separate frontend
launch_frontend() {
    print_status "🌐 Starting frontend..."

    local frontend_dir="$PROJECT_ROOT/frontend"  # Adjust path as needed

    cd "$frontend_dir"

    # Check if already running
    if [ -f /tmp/midi-frontend.pid ]; then
        local old_pid=$(cat /tmp/midi-frontend.pid)
        if ps -p "$old_pid" &>/dev/null; then
            print_warning "Frontend already running (PID: $old_pid)"
            return 0
        else
            rm /tmp/midi-frontend.pid
        fi
    elif pgrep -f "npm.*dev" &>/dev/null; then
        print_warning "Frontend already running (unknown PID)"
        return 0
    fi

    # Check if npm is available
    if ! command -v npm &>/dev/null; then
        print_error "npm not found. Please install Node.js."
        return 1
    fi

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing frontend dependencies..."
        npm install
    fi

    # Start frontend
    npm run dev > "$LOGS_DIR/frontend.log" 2>&1 &
    local frontend_pid=$!
    echo "$frontend_pid" > /tmp/midi-frontend.pid

    sleep 2
    if ps -p "$frontend_pid" &>/dev/null; then
        print_success "Frontend started (PID: $frontend_pid)"
        return 0
    else
        print_error "Frontend failed to start"
        echo "Check logs at: $LOGS_DIR/frontend.log"
        rm /tmp/midi-frontend.pid
        return 1
    fi
}

# Function to show service URLs
show_urls() {
    echo ""
    print_status "🌐 Service URLs:"
    echo "   Database:  postgresql://$DB_HOST:$DB_PORT/$DB_NAME"
    echo "   Backend:   Check Tauri window or logs"
    # echo "   Frontend:  http://localhost:3000"  # Uncomment if separate frontend
}

# Main execution
main() {
    # Clear screen and show banner
    clear
    echo ""
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo -e "${MAGENTA}🎵  MIDI Library System Launcher${NC}"
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo ""

    # Launch services in order
    if ! launch_database; then
        print_error "Cannot continue without database"
        exit 1
    fi

    echo ""

    if ! launch_backend; then
        print_error "Backend failed to start"
        echo ""
        print_status "Database is still running. Use ./scripts/stop-all.sh to stop all services."
        exit 1
    fi

    echo ""

    # Uncomment if you have a separate frontend
    # if ! launch_frontend; then
    #     print_error "Frontend failed to start"
    #     echo ""
    #     print_status "Database and backend are still running. Use ./scripts/stop-all.sh to stop all services."
    #     exit 1
    # fi

    # Summary
    echo ""
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo -e "${MAGENTA}🎵  Startup Summary${NC}"
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    print_success "Database: Running"
    print_success "Backend: Running"
    # print_success "Frontend: Running"  # Uncomment if separate frontend

    show_urls

    echo ""
    echo -e "${BLUE}📝 Logs: $LOGS_DIR/${NC}"
    echo -e "${BLUE}🛑 To stop: ./scripts/stop-all.sh${NC}"
    echo -e "${BLUE}📊 Status check: ./scripts/status.sh${NC}"
    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop tailing logs (services will keep running)${NC}"
    echo -e "${YELLOW}Or close this window to let services run in background${NC}"
    echo ""

    # Keep terminal open and tail logs
    print_status "📋 Tailing backend logs..."
    echo ""
    tail -f "$LOGS_DIR/backend.log"
}

# Trap Ctrl+C to provide exit message
trap 'echo ""; echo ""; print_status "Logs stopped. Services are still running."; echo "Use ./scripts/stop-all.sh to stop all services."; exit 0' INT

# Run main function
main
