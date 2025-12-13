#!/bin/bash
# Monitor import progress

echo "=== MIDI Library Import Monitor ==="
echo ""
echo "Database: localhost:5433/midi_library"
echo "Source: /home/dojevou/projects/midi-software-center/midi-library"
echo ""

while true; do
    # Check if process is still running
    if ! pgrep -f import_unified > /dev/null 2>&1; then
        echo ""
        echo "✓ Import process completed or not running!"
        break
    fi

    # Get database count
    db_count=$(psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -t -c "SELECT COUNT(*) FROM files;" 2>/dev/null | tr -d ' ')

    # Get unique content hash count
    unique_count=$(psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -t -c "SELECT COUNT(DISTINCT content_hash) FROM files;" 2>/dev/null | tr -d ' ')

    # Check log file
    if [ -f /tmp/import_log.txt ]; then
        last_line=$(tail -1 /tmp/import_log.txt 2>/dev/null | grep -v "warning:")
    else
        last_line="No log yet"
    fi

    echo "[$(date +%H:%M:%S)] DB files: $db_count | Unique hashes: $unique_count"

    sleep 10
done

echo ""
echo "=== Final Database Stats ==="
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "
SELECT
    COUNT(*) as total_files,
    COUNT(DISTINCT content_hash) as unique_files,
    pg_size_pretty(pg_total_relation_size('files')) as table_size
FROM files;
"
