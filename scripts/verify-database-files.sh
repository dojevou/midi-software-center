#!/bin/bash
# Verify that database file references still exist on disk

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "Checking sample of 100 random database file references..."
echo ""

# Get 100 random filepaths
psql "$DB_URL" -t -c "SELECT filepath FROM files ORDER BY RANDOM() LIMIT 100;" > /tmp/sample_paths.txt

total=0
exists=0
missing=0

while IFS= read -r filepath; do
  # Trim whitespace
  filepath=$(echo "$filepath" | xargs)

  if [ -n "$filepath" ]; then
    total=$((total + 1))

    if [ -f "$filepath" ]; then
      exists=$((exists + 1))
    else
      missing=$((missing + 1))
      echo "MISSING: $filepath"
    fi
  fi
done < /tmp/sample_paths.txt

echo ""
echo "=== RESULTS ==="
echo "Total checked: $total"
echo "Still exist:   $exists"
echo "Missing:       $missing"
echo ""

if [ $missing -eq 0 ]; then
  echo "✅ All sampled files still exist!"
else
  echo "⚠️  Some files are missing from disk!"
  echo ""
  echo "Next steps:"
  echo "1. Check full database: Run this script again with larger sample"
  echo "2. Clean database: Remove references to deleted files"
fi
