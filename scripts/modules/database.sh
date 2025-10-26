#!/bin/bash

# MIDI Library Database Management Helper
# Usage: ./db_helper.sh [command]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
DB_NAME="midi_library"
DB_USER="postgres"
DB_PASSWORD="password"
DB_HOST="localhost"
DB_PORT="5432"

DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

# Functions
show_help() {
    echo -e "${BLUE}MIDI Library Database Helper${NC}"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  status    - Check database status"
    echo "  connect   - Connect to database with psql"
    echo "  reset     - Drop and recreate database (WARNING: deletes all data)"
    echo "  backup    - Backup database to file"
    echo "  restore   - Restore database from backup file"
    echo "  tables    - List all tables"
    echo "  count     - Show row counts for all tables"
    echo "  test      - Test database connection"
    echo "  help      - Show this help message"
}

check_status() {
    echo -e "${YELLOW}Checking database status...${NC}"
    
    if sudo systemctl is-active --quiet postgresql; then
        echo -e "${GREEN}✓ PostgreSQL is running${NC}"
    else
        echo -e "${RED}✗ PostgreSQL is not running${NC}"
        echo "  Start with: sudo systemctl start postgresql"
        exit 1
    fi
    
    if PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -lqt | cut -d \| -f 1 | grep -qw ${DB_NAME}; then
        echo -e "${GREEN}✓ Database '${DB_NAME}' exists${NC}"
    else
        echo -e "${RED}✗ Database '${DB_NAME}' does not exist${NC}"
        echo "  Create with: ./complete_setup.sh"
        exit 1
    fi
    
    echo -e "${GREEN}✓ All checks passed${NC}"
}

connect_db() {
    echo -e "${BLUE}Connecting to database...${NC}"
    PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME}
}

reset_db() {
    echo -e "${RED}WARNING: This will delete ALL data in the database!${NC}"
    read -p "Are you sure? Type 'yes' to continue: " confirm
    
    if [ "$confirm" != "yes" ]; then
        echo "Cancelled."
        exit 0
    fi
    
    echo -e "${YELLOW}Dropping database...${NC}"
    sudo -u postgres psql -c "DROP DATABASE IF EXISTS ${DB_NAME};"
    
    echo -e "${YELLOW}Creating database...${NC}"
    sudo -u postgres psql -c "CREATE DATABASE ${DB_NAME};"
    
    if [ -f "schema.sql" ]; then
        echo -e "${YELLOW}Creating schema...${NC}"
        PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} -f schema.sql > /dev/null
        echo -e "${GREEN}✓ Database reset complete${NC}"
    else
        echo -e "${RED}✗ schema.sql not found${NC}"
        exit 1
    fi
}

backup_db() {
    BACKUP_FILE="midi_library_backup_$(date +%Y%m%d_%H%M%S).sql"
    echo -e "${YELLOW}Backing up to ${BACKUP_FILE}...${NC}"
    PGPASSWORD="${DB_PASSWORD}" pg_dump -h ${DB_HOST} -U ${DB_USER} ${DB_NAME} > ${BACKUP_FILE}
    echo -e "${GREEN}✓ Backup saved to ${BACKUP_FILE}${NC}"
}

restore_db() {
    if [ -z "$2" ]; then
        echo -e "${RED}Error: Please provide backup file${NC}"
        echo "Usage: $0 restore <backup_file>"
        exit 1
    fi
    
    BACKUP_FILE="$2"
    if [ ! -f "$BACKUP_FILE" ]; then
        echo -e "${RED}Error: Backup file not found: ${BACKUP_FILE}${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Restoring from ${BACKUP_FILE}...${NC}"
    PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} < ${BACKUP_FILE}
    echo -e "${GREEN}✓ Database restored${NC}"
}

list_tables() {
    echo -e "${BLUE}Tables in ${DB_NAME}:${NC}"
    PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} -c "\dt"
}

count_rows() {
    echo -e "${BLUE}Row counts:${NC}"
    PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} << 'EOF'
SELECT 
    schemaname as schema,
    tablename as table,
    n_live_tup as rows
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC;
EOF
}

test_connection() {
    echo -e "${YELLOW}Testing connection...${NC}"
    if PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -d ${DB_NAME} -c "SELECT 1;" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Connection successful${NC}"
        echo "  Host: ${DB_HOST}"
        echo "  Port: ${DB_PORT}"
        echo "  Database: ${DB_NAME}"
        echo "  User: ${DB_USER}"
    else
        echo -e "${RED}✗ Connection failed${NC}"
        exit 1
    fi
}

# Main
case "${1:-help}" in
    status)
        check_status
        ;;
    connect)
        connect_db
        ;;
    reset)
        reset_db
        ;;
    backup)
        backup_db
        ;;
    restore)
        restore_db "$@"
        ;;
    tables)
        list_tables
        ;;
    count)
        count_rows
        ;;
    test)
        test_connection
        ;;
    help|*)
        show_help
        ;;
esac
