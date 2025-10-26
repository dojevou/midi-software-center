\#!/bin/bash
cd ~/projects/midi-library-system/database

# Stop and clean up
docker-compose down -v

# Ensure we're using the correct password in docker-compose
sed -i 's/midipass_change_in_production/midipass_dev_only/g' docker-compose.yml

# Start services
docker-compose up -d

# Wait for database to be ready
echo "Waiting for database to be ready..."
sleep 20

# Test connection
echo "Testing database connection..."
PGPASSWORD=midipass_dev_only psql -h localhost -p 5433 -U midiuser -d midi_library -c "SELECT 1;" && echo "✅ Database connection successful!" || echo "❌ Database connection failed"

# Set up the pipeline
cd ../pipeline/src-tauri
echo "DATABASE_URL=postgresql://midiuser:admin@localhost:5433/midi_library" > .env
export DATABASE_URL="postgresql://midiuser:admin@localhost:5433/midi_library"

# Build
echo "Building the application..."
cargo build

# If build fails due to SQLx verification, use offline mode as fallback
if [ $? -ne 0 ]; then
    echo "Build failed, trying with offline mode..."
    cargo build --features sqlx/offline
fi

# Launch
cd ..
npm run tauri:dev
