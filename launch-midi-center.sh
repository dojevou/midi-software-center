#!/bin/bash
# MIDI Software Center Launcher
# Starts the application in development or release mode

set -e

PROJECT_DIR="/home/dojevou/projects/midi-software-center"
APP_DIR="$PROJECT_DIR/app"
LOG_FILE="$HOME/midi-center.log"

echo "🎵 MIDI Software Center Launcher"
echo "================================"
echo "Project: $PROJECT_DIR"
echo "Log: $LOG_FILE"
echo ""

# Parse arguments
MODE="${1:-dev}"

# Function to check database
check_database() {
    echo "🔌 Checking database..."
    if ! docker ps | grep -q midi-library-postgres; then
        echo "⚠️  Starting database..."
        cd "$PROJECT_DIR"
        docker-compose up -d
        echo "⏳ Waiting for database to be ready..."
        sleep 5
    else
        echo "✅ Database already running"
    fi
}

# Function to kill existing processes
cleanup_processes() {
    echo "🛑 Stopping existing processes..."
    pkill -9 -f "midi-software-center.*target" 2>/dev/null || true
    pkill -9 -f "vite.*5173" 2>/dev/null || true
    pkill -9 -f "pnpm.*tauri" 2>/dev/null || true
    sleep 1
}

# Function to clean caches
clean_caches() {
    echo "🧹 Cleaning caches..."
    rm -rf "$APP_DIR/node_modules/.vite" 2>/dev/null || true
}

# Main execution
cleanup_processes
check_database

if [ "$MODE" = "release" ]; then
    echo "🚀 Launching MIDI Software Center (Release Mode)..."

    # Check if release build exists
    RELEASE_BIN="$APP_DIR/src-tauri/target/release/midi-software-center"
    if [ -f "$RELEASE_BIN" ]; then
        cd "$APP_DIR"
        "$RELEASE_BIN" 2>&1 | tee "$LOG_FILE"
    else
        echo "⚠️  No release build found. Building..."
        cd "$APP_DIR"
        pnpm tauri build
        "$RELEASE_BIN" 2>&1 | tee "$LOG_FILE"
    fi
else
    echo "🚀 Launching MIDI Software Center (Development Mode)..."
    clean_caches
    cd "$APP_DIR"
    pnpm tauri dev 2>&1 | tee "$LOG_FILE"
fi

echo ""
echo "✅ Application closed"
