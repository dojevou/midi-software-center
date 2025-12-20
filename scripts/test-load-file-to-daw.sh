#!/bin/bash
# Manual test script for load_file_to_daw command
#
# This script helps test the drag & drop backend integration
# by providing test data and verification steps.

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Load File to DAW - Manual Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check database connection
if [ -z "$DATABASE_URL" ]; then
    DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
fi

echo -e "${GREEN}✓ Using database:${NC} $DATABASE_URL"
echo ""

# Get sample files from database
echo -e "${YELLOW}Step 1: Getting sample file IDs from database...${NC}"
psql "$DATABASE_URL" -c "
SELECT
    id,
    filepath,
    filename,
    size_bytes
FROM files
WHERE deleted_at IS NULL
LIMIT 10;
" 2>/dev/null || {
    echo "Error: Could not connect to database"
    exit 1
}

echo ""
echo -e "${YELLOW}Step 2: Manual Testing Instructions${NC}"
echo ""
echo "The load_file_to_daw command is already implemented in:"
echo "  app/src-tauri/src/commands/daw/sequencer.rs:144-152"
echo ""
echo "To test via the application:"
echo ""
echo "1. Start the application:"
echo "   cd app && npm run tauri dev"
echo ""
echo "2. Open the browser console (F12)"
echo ""
echo "3. Call the command from console:"
echo "   const { invoke } = window.__TAURI__.tauri;"
echo "   invoke('load_file_to_daw', { fileId: <FILE_ID> })"
echo "     .then(trackId => console.log('Track ID:', trackId))"
echo "     .catch(err => console.error('Error:', err));"
echo ""
echo "4. Verify the track was loaded:"
echo "   invoke('get_tracks')"
echo "     .then(tracks => console.log('Tracks:', tracks))"
echo ""
echo -e "${YELLOW}Step 3: Expected Behavior${NC}"
echo ""
echo "✓ load_file_to_daw returns a positive track ID (i32)"
echo "✓ get_tracks shows the newly added track"
echo "✓ Track has file_id matching input"
echo "✓ Track has channel = 0 (default)"
echo "✓ Track has events loaded from MIDI file"
echo ""
echo -e "${YELLOW}Step 4: Error Cases to Test${NC}"
echo ""
echo "Test with invalid file ID:"
echo "  invoke('load_file_to_daw', { fileId: -1 })"
echo "  Expected: Error 'File not found: -1'"
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Test script complete!${NC}"
echo -e "${GREEN}========================================${NC}"
