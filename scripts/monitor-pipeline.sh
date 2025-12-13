#!/bin/bash
# Pipeline Progress Monitor

echo "=== MIDI Pipeline Monitor ==="
echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Check if pipeline is running
if pgrep -f pipeline-orchestrator > /dev/null; then
    PID=$(pgrep -f pipeline-orchestrator)
    echo "✅ Pipeline Status: RUNNING (PID: $PID)"

    # CPU usage
    CPU=$(ps aux | grep $PID | grep -v grep | awk '{print $3}')
    echo "   CPU Usage: ${CPU}%"

    # Memory usage
    MEM=$(ps aux | grep $PID | grep -v grep | awk '{print $4}')
    echo "   Memory Usage: ${MEM}%"
else
    echo "❌ Pipeline Status: NOT RUNNING"
fi

echo ""
echo "=== Database Statistics ==="

# Database stats
docker exec midi-library-postgres psql -U midiuser -d midi_library -t -c "
SELECT
    'Total Files: ' || COUNT(*) || E'\n' ||
    'Folders: ' || COUNT(DISTINCT parent_folder) || E'\n' ||
    'Analyzed: ' || COUNT(*) FILTER (WHERE analyzed_at IS NOT NULL) || E'\n' ||
    'Pending Analysis: ' || COUNT(*) FILTER (WHERE analyzed_at IS NULL)
FROM files
" 2>/dev/null

echo ""
echo "=== Recent Pipeline Logs ==="
tail -20 /tmp/pipeline_continue.log 2>/dev/null | grep -E "INFO|WARN|ERROR" | tail -10

echo ""
echo "=== Stage Progress Hints ==="
tail -100 /tmp/pipeline_continue.log 2>/dev/null | grep -E "Stage [0-9]:" | tail -5

echo ""
echo "Run this script again to update progress"
