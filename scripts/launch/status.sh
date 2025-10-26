#!/bin/bash
# MIDI Library System - Status Check Script

# Configuration
PROJECT_ROOT="$HOME/projects/midi-library-system"
LOGS_DIR="$PROJECT_ROOT/logs"
DB_HOST="localhost"
DB_PORT="5433"
DB_USER="midiuser"
DB_PASSWORD="145278963"
DB_NAME="midi_library"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Function to print colored messages
print_running() {
    echo -e "${GREEN}✅ Running${NC}"
}

print_stopped() {
    echo -e "${RED}❌ Stopped${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to check database status
check_database() {
    echo -n "Database:  "

    # Check if Docker container is running
    if command -v docker-compose &>/dev/null; then
        cd "$PROJECT_ROOT/database"
        if docker-compose ps 2>/dev/null | grep -q "Up"; then
            # Container is up, now check if database is accepting connections
            if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1;" &>/dev/null; then
                print_running
                echo "           postgresql://$DB_HOST:$DB_PORT/$DB_NAME"
                return 0
            else
                print_warning "Container up but database not ready"
                return 1
            fi
        else
            print_stopped
            return 1
        fi
    elif command -v docker &>/dev/null; then
        # Try docker compose without hyphen
        cd "$PROJECT_ROOT/database"
        if docker compose ps 2>/dev/null | grep -q "Up"; then
            if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1;" &>/dev/null; then
                print_running
                echo "           postgresql://$DB_HOST:$DB_PORT/$DB_NAME"
                return 0
            else
                print_warning "Container up but database not ready"
                return 1
            fi
        else
            print_stopped
            return 1
        fi
    else
        print_warning "Docker not found"
        return 1
    fi
}

# Function to check backend status
check_backend() {
    echo -n "Backend:   "

    # Check PID file first
    if [ -f /tmp/midi-backend.pid ]; then
        local backend_pid=$(cat /tmp/midi-backend.pid)
        if ps -p "$backend_pid" &>/dev/null; then
            print_running
            echo "           PID: $backend_pid"
            # Show uptime
            local start_time=$(ps -p "$backend_pid" -o lstart=)
            echo "           Started: $start_time"
            return 0
        else
            print_warning "Stale PID file (process not running)"
            return 1
        fi
    # Check by process name
    elif pgrep -f "cargo.*tauri" &>/dev/null; then
        local pids=$(pgrep -f "cargo.*tauri" | tr '\n' ' ')
        print_warning "Running (unknown PID: $pids)"
        echo "           No PID file found"
        return 0
    else
        print_stopped
        return 1
    fi
}

# Function to check frontend status (if separate)
check_frontend() {
    echo -n "Frontend:  "

    # Check PID file first
    if [ -f /tmp/midi-frontend.pid ]; then
        local frontend_pid=$(cat /tmp/midi-frontend.pid)
        if ps -p "$frontend_pid" &>/dev/null; then
            print_running
            echo "           PID: $frontend_pid"
            # Try to find the port
            if command -v lsof &>/dev/null; then
                local port=$(lsof -Pan -p "$frontend_pid" -i | grep LISTEN | awk '{print $9}' | cut -d: -f2 | head -n1)
                if [ -n "$port" ]; then
                    echo "           http://localhost:$port"
                fi
            fi
            return 0
        else
            print_warning "Stale PID file (process not running)"
            return 1
        fi
    # Check by process name
    elif pgrep -f "npm.*dev" &>/dev/null; then
        local pids=$(pgrep -f "npm.*dev" | tr '\n' ' ')
        print_warning "Running (unknown PID: $pids)"
        return 0
    else
        print_stopped
        return 1
    fi
}

# Function to check Docker status
check_docker() {
    if ! command -v docker &>/dev/null; then
        echo -e "${RED}⚠️  Docker not installed${NC}"
        return 1
    fi

    if ! docker ps &>/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Docker daemon not running${NC}"
        return 1
    fi

    return 0
}

# Function to show recent log entries
show_recent_logs() {
    echo ""
    echo -e "${BLUE}📋 Recent Log Entries:${NC}"
    echo "===================="

    if [ -f "$LOGS_DIR/backend.log" ]; then
        echo ""
        echo -e "${BLUE}Backend (last 5 lines):${NC}"
        tail -n 5 "$LOGS_DIR/backend.log" 2>/dev/null || echo "  No logs available"
    fi

    # Uncomment if you have frontend logs
    # if [ -f "$LOGS_DIR/frontend.log" ]; then
    #     echo ""
    #     echo -e "${BLUE}Frontend (last 5 lines):${NC}"
    #     tail -n 5 "$LOGS_DIR/frontend.log" 2>/dev/null || echo "  No logs available"
    # fi

    if [ -f "$LOGS_DIR/database.log" ]; then
        echo ""
        echo -e "${BLUE}Database (last 5 lines):${NC}"
        tail -n 5 "$LOGS_DIR/database.log" 2>/dev/null || echo "  No logs available"
    fi
}

# Function to show disk usage
show_disk_usage() {
    echo ""
    echo -e "${BLUE}💾 Disk Usage:${NC}"
    echo "=============="

    if [ -d "$LOGS_DIR" ]; then
        local logs_size=$(du -sh "$LOGS_DIR" 2>/dev/null | cut -f1)
        echo "Logs:      $logs_size ($LOGS_DIR)"
    fi

    # Check Docker volumes
    if command -v docker &>/dev/null; then
        local db_volume=$(docker volume ls | grep midi | awk '{print $2}' | head -n1)
        if [ -n "$db_volume" ]; then
            echo "Database:  Docker volume '$db_volume'"
        fi
    fi
}

# Main execution
main() {
    echo ""
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo -e "${MAGENTA}🎵  MIDI Library System Status${NC}"
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    echo ""

    # Check Docker first
    check_docker
    echo ""

    # Check all services
    local all_running=1

    if ! check_database; then
        all_running=0
    fi
    echo ""

    if ! check_backend; then
        all_running=0
    fi
    echo ""

    # Uncomment if you have a separate frontend
    # if ! check_frontend; then
    #     all_running=0
    # fi
    # echo ""

    # Show overall status
    echo -e "${MAGENTA}🎵 ==================================${NC}"
    if [ $all_running -eq 1 ]; then
        echo -e "${GREEN}Overall Status: All services running ✅${NC}"
    else
        echo -e "${YELLOW}Overall Status: Some services are down ⚠️${NC}"
    fi
    echo -e "${MAGENTA}🎵 ==================================${NC}"

    # Show additional info
    show_disk_usage
    show_recent_logs

    echo ""
    echo -e "${BLUE}📁 Log files: $LOGS_DIR/${NC}"
    echo -e "${BLUE}🚀 Start:     ./scripts/launch-all.sh${NC}"
    echo -e "${BLUE}🛑 Stop:      ./scripts/stop-all.sh${NC}"
    echo ""
}

# Run main function
main
