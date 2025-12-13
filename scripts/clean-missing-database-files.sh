#!/bin/bash
# Clean database by removing references to files that no longer exist

set -e

DB_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "════════════════════════════════════════════════════════════"
echo "  Database Cleanup - Remove Missing File References"
echo "════════════════════════════════════════════════════════════"
echo ""

# Count total files in database
total=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
echo "Files in database: $total"
echo ""

# Create temporary table for checking
echo "Creating temporary file list..."
psql "$DB_URL" -c "CREATE TEMP TABLE file_check (filepath TEXT, exists BOOLEAN);"

# Export all filepaths
echo "Exporting database filepaths..."
psql "$DB_URL" -t -c "SELECT filepath FROM files;" > /tmp/all_db_paths.txt

# Check which files still exist
echo "Checking which files still exist on disk..."
echo "(This may take a while for 1.7M files...)"

checked=0
exists=0
missing=0

while IFS= read -r filepath; do
  filepath=$(echo "$filepath" | xargs)

  if [ -n "$filepath" ]; then
    checked=$((checked + 1))

    if [ -f "$filepath" ]; then
      exists=$((exists + 1))
    else
      missing=$((missing + 1))
    fi

    # Progress every 10,000 files
    if [ $((checked % 10000)) -eq 0 ]; then
      echo "Checked: $checked files ($exists exist, $missing missing)"
    fi
  fi
done < /tmp/all_db_paths.txt

echo ""
echo "═══ CHECK COMPLETE ═══"
echo "Total checked: $checked"
echo "Still exist:   $exists ($((exists * 100 / checked))%)"
echo "Missing:       $missing ($((missing * 100 / checked))%)"
echo ""

if [ $missing -eq 0 ]; then
  echo "✅ All database files still exist! No cleanup needed."
  exit 0
fi

echo "⚠️  Found $missing missing file references"
echo ""
echo "Do you want to DELETE these $missing database records? (y/n)"
read -r response

if [[ ! "$response" =~ ^[Yy]$ ]]; then
  echo "Cancelled. No changes made."
  exit 0
fi

echo ""
echo "Creating list of missing files..."

# Create deletion list
rm -f /tmp/files_to_delete.txt
while IFS= read -r filepath; do
  filepath=$(echo "$filepath" | xargs)

  if [ -n "$filepath" ] && [ ! -f "$filepath" ]; then
    echo "$filepath" >> /tmp/files_to_delete.txt
  fi
done < /tmp/all_db_paths.txt

echo "Deleting $missing database records..."

# Use a more efficient approach - create temp table and delete
psql "$DB_URL" <<EOF
-- Create temporary table
CREATE TEMP TABLE files_to_delete (filepath TEXT);

-- Load missing filepaths
\COPY files_to_delete FROM '/tmp/files_to_delete.txt'

-- Delete from database
DELETE FROM files WHERE filepath IN (SELECT filepath FROM files_to_delete);

-- Show results
SELECT COUNT(*) as deleted FROM files_to_delete;
EOF

echo ""
echo "✅ Database cleanup complete!"
echo ""

# Verify new count
new_total=$(psql "$DB_URL" -t -c "SELECT COUNT(*) FROM files;" | xargs)
echo "Files remaining in database: $new_total"
echo "Files deleted: $((total - new_total))"
