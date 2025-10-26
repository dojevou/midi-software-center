#!/bin/bash
# Import full 1M+ MIDI collection with auto-tagging

set -e

COLLECTION_PATH="/home/dojevou/floorp_downloads/_1.002.000-Midi-Collection_"

echo "╔══════════════════════════════════════════════════════╗"
echo "║    MIDI Collection Import - Full 1M+ Files           ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
echo "📂 Collection: $COLLECTION_PATH"
echo ""

# Count archives
ARCHIVE_COUNT=$(ls "$COLLECTION_PATH"/*.zip 2>/dev/null | wc -l)
echo "📦 Found $ARCHIVE_COUNT archives to process"
echo ""

# Show largest archives
echo "📊 Largest archives:"
ls -lhS "$COLLECTION_PATH"/*.zip | head -5 | awk '{print "   " $9 " (" $5 ")"}'
echo ""

# Database stats before
echo "📊 Database stats BEFORE import:"
psql postgresql://midiuser:145278963@localhost:5433/midi_library -c "
SELECT
  (SELECT COUNT(*) FROM files) as files,
  (SELECT COUNT(*) FROM tags) as tags,
  (SELECT COUNT(*) FROM file_tags) as relations,
  (SELECT pg_size_pretty(pg_database_size('midi_library'))) as db_size;
"

echo ""
echo "⏰ Estimated time: 1-2 hours at 400-800 files/sec"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "MANUAL STEP REQUIRED:"
echo ""
echo "1. Make sure the Tauri app is running:"
echo "   cd /home/dojevou/projects/midi-software-center/pipeline"
echo "   pnpm tauri dev"
echo ""
echo "2. In the browser DevTools console, run:"
echo ""
echo "   const result = await window.__TAURI__.core.invoke('import_archive_collection', {"
echo "     collectionPath: '$COLLECTION_PATH'"
echo "   });"
echo "   console.log(result);"
echo ""
echo "3. Wait for completion (monitor console for progress)"
echo ""
echo "4. Press Enter here when done..."
echo ""
read

# Database stats after
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Database stats AFTER import:"
psql postgresql://midiuser:145278963@localhost:5433/midi_library -c "
SELECT
  (SELECT COUNT(*) FROM files) as files,
  (SELECT COUNT(*) FROM tags) as tags,
  (SELECT COUNT(*) FROM file_tags) as relations,
  (SELECT pg_size_pretty(pg_database_size('midi_library'))) as db_size;
"

echo ""
echo "📋 Tag categories created:"
psql postgresql://midiuser:145278963@localhost:5433/midi_library -c "
SELECT
  category,
  COUNT(*) as tag_count,
  SUM(usage_count) as total_uses
FROM tags
GROUP BY category
ORDER BY total_uses DESC;
"

echo ""
echo "🏆 Most popular tags:"
psql postgresql://midiuser:145278963@localhost:5433/midi_library -c "
SELECT
  CASE
    WHEN category IS NOT NULL THEN category || ':' || name
    ELSE name
  END as tag,
  usage_count
FROM tags
ORDER BY usage_count DESC
LIMIT 20;
"

echo ""
echo "╔══════════════════════════════════════════════════════╗"
echo "║                IMPORT COMPLETE                       ║"
echo "╚══════════════════════════════════════════════════════╝"
