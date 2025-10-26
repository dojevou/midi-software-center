#!/bin/bash

# MIDI Library System - Database Setup Script
# This script sets up PostgreSQL with pgvector in Docker

set -e  # Exit on any error

echo "=========================================="
echo "MIDI Library System - Database Setup"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}Error: Docker is not running!${NC}"
    echo "Please start Docker and try again."
    exit 1
fi

echo -e "${GREEN}✓ Docker is running${NC}"
echo ""

# Navigate to database directory
cd "$(dirname "$0")/.."

echo "Setting up database..."
echo ""

# Stop and remove existing container if it exists
if docker ps -a | grep -q midi_library_db; then
    echo -e "${YELLOW}Stopping existing container...${NC}"
    docker stop midi_library_db > /dev/null 2>&1 || true
    docker rm midi_library_db > /dev/null 2>&1 || true
    echo -e "${GREEN}✓ Removed existing container${NC}"
fi

# Start PostgreSQL container
echo "Starting PostgreSQL container..."
docker-compose up -d

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
for i in {1..30}; do
    if docker exec midi_library_db pg_isready -U midiuser -d midi_library > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PostgreSQL is ready!${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}Error: PostgreSQL failed to start within 30 seconds${NC}"
        exit 1
    fi
    echo -n "."
    sleep 1
done
echo ""

# Verify pgvector extension
echo "Verifying pgvector extension..."
docker exec midi_library_db psql -U midiuser -d midi_library -c "SELECT extversion FROM pg_extension WHERE extname = 'vector';" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ pgvector extension is installed${NC}"
else
    echo -e "${RED}Error: pgvector extension not found${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}=========================================="
echo "Database setup complete!"
echo "==========================================${NC}"
echo ""
echo "Connection details:"
echo "  Host: localhost"
echo "  Port: 5432"
echo "  Database: midi_library"
echo "  User: midiuser"
echo "  Password: midipass_change_in_production"
echo ""
echo "Connection string:"
echo "  postgresql://midiuser:midipass_change_in_production@localhost:5432/midi_library"
echo ""
echo "To connect with psql:"
echo "  docker exec -it midi_library_db psql -U midiuser -d midi_library"
echo ""
echo "To stop the database:"
echo "  docker-compose down"
echo ""
echo "To stop and remove all data:"
echo "  docker-compose down -v"
echo ""
