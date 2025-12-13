#!/bin/bash
# Monitor deduplication progress

echo "=== MIDI Library Deduplication Monitor ==="
echo ""
echo "Process PID: 1051791"
echo "Files to delete: 5,030,558"
echo ""

while true; do
    # Check if process is still running
    if ! ps -p 1051791 > /dev/null 2>&1; then
        echo ""
        echo "✓ Deletion process completed!"
        break
    fi

    # Count remaining files (sample from splits directory for speed)
    splits_count=$(find /home/dojevou/projects/midi-software-center/midi-library/splits -type f -name "*.mid" 2>/dev/null | wc -l)

    # Get process stats
    cpu=$(ps -p 1051791 -o %cpu --no-headers 2>/dev/null)
    runtime=$(ps -p 1051791 -o etime --no-headers 2>/dev/null)

    echo "[$(date +%H:%M:%S)] Runtime: $runtime | CPU: ${cpu}% | Splits dir: $splits_count files"

    sleep 30
done

echo ""
echo "=== Final Verification ==="
echo "Counting all remaining MIDI files (this may take a few minutes)..."
total=$(find /home/dojevou/projects/midi-software-center/midi-library -type f \( -name "*.mid" -o -name "*.MID" \) 2>/dev/null | wc -l)
echo "Total MIDI files remaining: $total"
echo ""
du -sh /home/dojevou/projects/midi-software-center/midi-library
