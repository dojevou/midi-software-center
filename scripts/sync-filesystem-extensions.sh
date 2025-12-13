#!/bin/bash
set -e

# Sync Filesystem Extensions with Database
# Database is already updated, this just renames files on disk

LOG_FILE="/tmp/filesystem_sync_$(date +%Y%m%d_%H%M%S).log"

echo "🎵 Syncing Filesystem with Database Extensions"
echo "=============================================="
echo ""
echo "This will rename .MID, .midi, and .Mid files to .mid on disk"
echo "Log File: $LOG_FILE"
echo ""

# Phase 1: Find and rename .MID files
echo "📁 Phase 1: Renaming .MID → .mid"
echo "================================"
echo ""

renamed_mid=0
find /home/dojevou/projects/midi-software-center/midi-library -type f -name "*.MID" | while read -r file; do
    new_file="${file%.MID}.mid"

    if [ -f "$new_file" ]; then
        echo "⚠️  Target exists: $new_file" | tee -a "$LOG_FILE"
    else
        if mv "$file" "$new_file" 2>/dev/null; then
            renamed_mid=$((renamed_mid + 1))
            if [ $((renamed_mid % 100)) -eq 0 ]; then
                echo "  Renamed $renamed_mid .MID files..."
            fi
        else
            echo "⚠️  Failed: $file" | tee -a "$LOG_FILE"
        fi
    fi
done

echo ""
echo "✅ .MID files renamed"
echo ""

# Phase 2: Find and rename .midi files
echo "📁 Phase 2: Renaming .midi → .mid"
echo "================================="
echo ""

renamed_midi=0
find /home/dojevou/projects/midi-software-center/midi-library -type f -name "*.midi" | while read -r file; do
    new_file="${file%.midi}.mid"

    if [ -f "$new_file" ]; then
        echo "⚠️  Target exists: $new_file" | tee -a "$LOG_FILE"
    else
        if mv "$file" "$new_file" 2>/dev/null; then
            renamed_midi=$((renamed_midi + 1))
            if [ $((renamed_midi % 100)) -eq 0 ]; then
                echo "  Renamed $renamed_midi .midi files..."
            fi
        else
            echo "⚠️  Failed: $file" | tee -a "$LOG_FILE"
        fi
    fi
done

echo ""
echo "✅ .midi files renamed"
echo ""

# Phase 3: Find and rename .Mid files
echo "📁 Phase 3: Renaming .Mid → .mid"
echo "================================"
echo ""

renamed_Mid=0
find /home/dojevou/projects/midi-software-center/midi-library -type f -name "*.Mid" | while read -r file; do
    new_file="${file%.Mid}.mid"

    if [ -f "$new_file" ]; then
        echo "⚠️  Target exists: $new_file" | tee -a "$LOG_FILE"
    else
        if mv "$file" "$new_file" 2>/dev/null; then
            renamed_Mid=$((renamed_Mid + 1))
            if [ $((renamed_Mid % 100)) -eq 0 ]; then
                echo "  Renamed $renamed_Mid .Mid files..."
            fi
        else
            echo "⚠️  Failed: $file" | tee -a "$LOG_FILE"
        fi
    fi
done

echo ""
echo "✅ .Mid files renamed"
echo ""

# Final verification
echo "📊 Verification"
echo "==============="
echo ""

echo "Remaining non-.mid files:"
find /home/dojevou/projects/midi-software-center/midi-library -type f \( -name "*.MID" -o -name "*.midi" -o -name "*.Mid" \) | wc -l

echo ""
echo "🎉 Filesystem sync complete!"
echo "============================"
echo ""
echo "Log file: $LOG_FILE"
echo ""
