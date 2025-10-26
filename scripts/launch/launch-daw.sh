#!/bin/bash
# MIDI DAW Launcher with Service Management

PROJECT_ROOT="/home/dojevou/projects/midi-software-center"
LOG_DIR="$PROJECT_ROOT/logs"
mkdir -p "$LOG_DIR"

echo "🎵 MIDI DAW Launcher"
echo "===================="

# Step 1: Stop conflicting services
echo "🛑 Stopping any existing services..."

# Kill any running vite/tauri processes on port 5174
pkill -f "vite.*5174" 2>/dev/null
pkill -f "tauri.*daw" 2>/dev/null
fuser -k 5174/tcp 2>/dev/null

# Ensure database is running (read-only access)
cd "$PROJECT_ROOT/database"
if ! docker compose ps | grep -q "postgres.*Up"; then
    echo "🗄️  Starting database..."
    docker compose up -d
    sleep 3
fi

# Step 2: Check MIDI hardware
echo "🎹 Checking MIDI devices..."
if command -v aconnect &> /dev/null; then
    aconnect -l
else
    echo "⚠️  ALSA not found, MIDI features may be limited"
fi

# Step 3: Launch the application
echo "🚀 Launching DAW App..."
cd "$PROJECT_ROOT/daw"

export DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

pnpm tauri dev 2>&1 | tee "$LOG_DIR/daw-$(date +%Y%m%d-%H%M%S).log"
