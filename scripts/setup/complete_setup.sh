#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  MIDI Library System - Complete Setup ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Get the project root directory
PROJECT_ROOT="$HOME/projects/midi-software-center"
DAW_DIR="$PROJECT_ROOT/daw"
TAURI_DIR="$DAW_DIR/src-tauri"

# Database configuration
DB_NAME="midi_library"
DB_USER="postgres"
DB_PASSWORD="password"
DB_HOST="localhost"
DB_PORT="5432"

DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

# Step 1: Check PostgreSQL
echo -e "${YELLOW}[1/6] Checking PostgreSQL...${NC}"
if ! sudo systemctl is-active --quiet postgresql; then
    echo -e "${YELLOW}      PostgreSQL is not running. Starting...${NC}"
    sudo systemctl start postgresql
    sleep 2
fi
echo -e "${GREEN}      ✓ PostgreSQL is running${NC}"
echo ""

# Step 2: Create database
echo -e "${YELLOW}[2/6] Setting up database...${NC}"
sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname = '${DB_NAME}'" | grep -q 1 && {
    echo -e "${BLUE}      Database '${DB_NAME}' already exists${NC}"
} || {
    sudo -u postgres psql -c "CREATE DATABASE ${DB_NAME};"
    echo -e "${GREEN}      ✓ Database '${DB_NAME}' created${NC}"
}

# Set postgres password
sudo -u postgres psql -c "ALTER USER ${DB_USER} WITH PASSWORD '${DB_PASSWORD}';" > /dev/null 2>&1
echo -e "${GREEN}      ✓ Database user configured${NC}"
echo ""

# Step 3: Create schema
echo -e "${YELLOW}[3/6] Creating database schema...${NC}"
if [ -f "schema.sql" ]; then
    PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} -f schema.sql > /dev/null 2>&1
    echo -e "${GREEN}      ✓ Schema created successfully${NC}"
else
    echo -e "${RED}      ✗ schema.sql not found!${NC}"
    echo -e "${YELLOW}      Please make sure schema.sql is in the current directory${NC}"
    exit 1
fi
echo ""

# Step 4: Create .env file
echo -e "${YELLOW}[4/6] Creating .env file...${NC}"
mkdir -p "${TAURI_DIR}"
cat > "${TAURI_DIR}/.env" << EOF
DATABASE_URL=${DATABASE_URL}

# Logging
RUST_LOG=info
RUST_BACKTRACE=1
EOF
echo -e "${GREEN}      ✓ .env file created at ${TAURI_DIR}/.env${NC}"
echo ""

# Step 5: Prepare SQLx for offline mode (optional but recommended)
echo -e "${YELLOW}[5/6] Preparing SQLx...${NC}"
if command -v sqlx &> /dev/null; then
    cd "${TAURI_DIR}"
    DATABASE_URL="${DATABASE_URL}" cargo sqlx prepare > /dev/null 2>&1 && {
        echo -e "${GREEN}      ✓ SQLx metadata prepared${NC}"
    } || {
        echo -e "${YELLOW}      ! SQLx prepare skipped (run manually if needed)${NC}"
    }
    cd - > /dev/null
else
    echo -e "${YELLOW}      ! sqlx-cli not installed (optional)${NC}"
    echo -e "${YELLOW}      Install with: cargo install sqlx-cli --no-default-features --features postgres${NC}"
fi
echo ""

# Step 6: Verify setup
echo -e "${YELLOW}[6/6] Verifying setup...${NC}"
PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} -c "\dt" > /dev/null 2>&1 && {
    echo -e "${GREEN}      ✓ Database connection successful${NC}"
    echo -e "${GREEN}      ✓ Tables created:${NC}"
    PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} -c "\dt" | grep "public" | awk '{print "        - " $3}'
} || {
    echo -e "${RED}      ✗ Could not verify database${NC}"
    exit 1
}
echo ""

# Success summary
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           Setup Complete! ✓            ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Database Information:${NC}"
echo -e "  Database: ${DB_NAME}"
echo -e "  User:     ${DB_USER}"
echo -e "  Host:     ${DB_HOST}"
echo -e "  Port:     ${DB_PORT}"
echo -e "  URL:      ${DATABASE_URL}"
echo ""
echo -e "${GREEN}Next Steps:${NC}"
echo -e "  1. Navigate to the DAW directory:"
echo -e "     ${BLUE}cd ${DAW_DIR}${NC}"
echo -e ""
echo -e "  2. Run the application:"
echo -e "     ${BLUE}npm run tauri dev${NC}"
echo ""
