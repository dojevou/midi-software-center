#!/bin/bash
# MIDI Software Center Launcher
# Cleanly stops all processes and launches fresh instance

set -e

# Parse arguments
MODE="app"
while [[ $# -gt 0 ]]; do
    case $1 in
        --pipeline)
            MODE="pipeline"
            shift
            ;;
        --daw)
            MODE="daw"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

echo "🎵 MIDI Software Center Launcher"
echo "================================"
echo "Mode: $MODE"

# Set working directory based on mode
case $MODE in
    pipeline)
        APP_DIR="/home/dojevou/projects/midi-software-center/pipeline"
        ;;
    daw)
        APP_DIR="/home/dojevou/projects/midi-software-center/daw"
        ;;
    *)
        APP_DIR="/home/dojevou/projects/midi-software-center/app"
        ;;
esac

# Change to app directory
cd "$APP_DIR"

# Kill any existing processes
echo "🛑 Stopping existing processes..."
pkill -9 -f "midi-software-center.*target/debug" 2>/dev/null || true
pkill -9 -f "vite.*5173" 2>/dev/null || true
pkill -9 -f "pnpm.*tauri" 2>/dev/null || true
sleep 2

# Clean caches
echo "🧹 Cleaning caches..."
rm -rf node_modules/.vite 2>/dev/null || true

# Ensure database is running
echo "🔌 Checking database..."
if ! docker ps | grep -q midi-library-postgres; then
    echo "⚠️  Starting database..."
    cd /home/dojevou/projects/midi-software-center
    docker-compose up -d
    sleep 5
fi

# Launch application
echo "🚀 Launching MIDI Software Center..."
cd /home/dojevou/projects/midi-software-center/app
pnpm tauri dev 2>&1 | tee ~/midi-center.log

echo "✅ Application closed"
