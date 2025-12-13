#!/bin/bash
# Wait for import to complete, then auto-start phases 2-3

echo "Monitoring import progress..."
echo ""

while ps aux | grep -q "[b]atch_import"; do
    CURRENT=$(psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -t -c "SELECT COUNT(*) FROM files;" 2>/dev/null | xargs)
    PCT=$(echo "scale=1; $CURRENT * 100 / 1715793" | bc 2>/dev/null)
    echo "$(date '+%H:%M:%S') - Import progress: $CURRENT files ($PCT%)"
    sleep 60
done

echo ""
echo "✓ Import complete!"
FINAL=$(psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -t -c "SELECT COUNT(*) FROM files;" | xargs)
echo "Total imported: $FINAL files"
echo ""
echo "Starting Phases 2-3..."
echo ""

cd /home/dojevou/projects/midi-software-center
nohup ./scripts/run-phases-1-2-3.sh > /tmp/phases_2-3_auto.log 2>&1 &

echo "Phases 2-3 started in background!"
echo "Monitor: tail -f /tmp/phases_2-3_auto.log"
