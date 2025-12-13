#!/bin/bash
# Quick pipeline progress checker

echo "=== MIDI Pipeline Progress ==="
echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Check if pipeline is running
if pgrep -f pipeline-orchestrator > /dev/null; then
    echo "✅ Pipeline Status: RUNNING"
    ps aux | grep pipeline-orchestrator | grep -v grep | awk '{printf "   CPU: %s%% | Memory: %s MB\n", $3, int($6/1024)}'
else
    echo "❌ Pipeline Status: NOT RUNNING"
fi

echo ""
echo "📊 Database Statistics:"
docker exec midi-library-postgres psql -U midiuser -d midi_library -t -c "
SELECT
    '   Total Files: ' || COUNT(*) ||
    ' | Folders: ' || COUNT(DISTINCT parent_folder) ||
    ' | Analyzed: ' || COUNT(*) FILTER (WHERE analyzed_at IS NOT NULL)
FROM files;
" | xargs echo

echo ""
echo "📝 Recent Activity (last 5 non-duplicate lines):"
tail -n 100 /tmp/pipeline_ludicrous.log 2>/dev/null | grep -v "duplicate key" | grep -v "No such file" | tail -n 5 | sed 's/^/   /'

echo ""
echo "💡 Tips:"
echo "   - Run './check-progress.sh' anytime to see progress"
echo "   - View full log: tail -f /tmp/pipeline_ludicrous.log"
echo "   - Stop pipeline: pkill -f pipeline-orchestrator"
echo ""
echo "⚠️  LUDICROUS MODE ACTIVE (fsync=off, no crash safety!)"
