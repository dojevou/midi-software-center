#!/bin/bash
# MIDI Library Pipeline Launcher with Service Management

PROJECT_ROOT="/home/dojevou/projects/midi-software-center"
LOG_DIR="$PROJECT_ROOT/logs"
mkdir -p "$LOG_DIR"

echo "🎵 MIDI Library Pipeline Launcher"
echo "=================================="

# Step 1: Stop conflicting services
echo "🛑 Stopping any existing services..."

# Kill any running vite/tauri processes on port 5173
pkill -f "vite.*5173" 2>/dev/null
pkill -f "tauri.*pipeline" 2>/dev/null
fuser -k 5173/tcp 2>/dev/null

# Stop Docker services (will restart them)
cd "$PROJECT_ROOT/database"
docker compose down 2>/dev/null

sleep 2

# Step 2: Start required services
echo "🗄️  Starting PostgreSQL and Meilisearch..."
docker compose up -d

# Wait for database
echo "⏳ Waiting for database..."
for i in {1..30}; do
    if pg_isready -h localhost -p 5433 -U midiuser >/dev/null 2>&1; then
        echo "✅ Database ready!"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo "❌ Database failed to start"
        exit 1
    fi
done

# Step 3: Launch the application
echo "🚀 Launching Pipeline App..."
cd "$PROJECT_ROOT/pipeline"

# Set environment (note: database is on port 5433, not default 5432)
export DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Launch in terminal so you can see output
pnpm tauri dev 2>&1 | tee "$LOG_DIR/pipeline-$(date +%Y%m%d-%H%M%S).log"
