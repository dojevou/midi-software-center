#!/bin/bash
# Create curated tag lists from keyword analysis
# Filters out low-frequency and noise keywords

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Creating Curated Tag Lists"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Configuration
MIN_FREQUENCY=50  # Minimum file count for a keyword to be included
OUTPUT_DIR="/tmp"

# Check if input files exist
if [ ! -f "/tmp/grandparent_folders.txt" ] || \
   [ ! -f "/tmp/parent_folders.txt" ] || \
   [ ! -f "/tmp/filenames.txt" ]; then
    echo "❌ ERROR: Input files not found in /tmp/"
    echo "   Run keyword extraction first!"
    exit 1
fi

echo "📋 Step 1: Extract Grandparent Folder Keywords"
echo "   Filtering: frequency >= $MIN_FREQUENCY, exclude generic names"
echo ""

cat /tmp/grandparent_folders.txt | \
  grep -v -E "(Variation-[0-9]+|Theme-[0-9]+|cd-[0-9]+|tmp|splits|archives)" | \
  awk -v min=$MIN_FREQUENCY '$1 >= min {for(i=2;i<=NF;i++) printf "%s ", $i; print ""}' | \
  sed 's/ $//' | \
  tr '[:upper:]' '[:lower:]' | \
  sort -u > $OUTPUT_DIR/curated_grandparent_tags.txt

GRANDPARENT_COUNT=$(wc -l < $OUTPUT_DIR/curated_grandparent_tags.txt)
echo "   ✅ Extracted $GRANDPARENT_COUNT unique grandparent keywords"
echo ""

echo "📋 Step 2: Extract Parent Folder Keywords"
echo "   Filtering: frequency >= $MIN_FREQUENCY, exclude generic names"
echo ""

cat /tmp/parent_folders.txt | \
  grep -v -E "(Variation-[0-9]+|Theme-[0-9]+|Preview|Bonus|tmp|splits)" | \
  awk -v min=$MIN_FREQUENCY '$1 >= min {for(i=2;i<=NF;i++) printf "%s ", $i; print ""}' | \
  sed 's/ $//' | \
  tr '[:upper:]' '[:lower:]' | \
  sort -u > $OUTPUT_DIR/curated_parent_tags.txt

PARENT_COUNT=$(wc -l < $OUTPUT_DIR/curated_parent_tags.txt)
echo "   ✅ Extracted $PARENT_COUNT unique parent keywords"
echo ""

echo "📋 Step 3: Extract Filename Keywords"
echo "   Filtering: frequency >= $MIN_FREQUENCY, exclude track numbers and variations"
echo ""

cat /tmp/filenames.txt | \
  grep -v -E "(track_[0-9]+|Variation_[0-9]+|Beat-[0-9]+-[0-9]+|^[0-9]+$)" | \
  awk -v min=$MIN_FREQUENCY '$1 >= min {for(i=2;i<=NF;i++) printf "%s ", $i; print ""}' | \
  sed 's/ $//' | \
  tr '[:upper:]' '[:lower:]' | \
  grep -v "^\." | \
  sort -u > $OUTPUT_DIR/curated_filename_tags.txt

FILENAME_COUNT=$(wc -l < $OUTPUT_DIR/curated_filename_tags.txt)
echo "   ✅ Extracted $FILENAME_COUNT unique filename keywords"
echo ""

echo "📋 Step 4: Normalize and Split Multi-Word Keywords"
echo ""

# Function to normalize keywords (split on _, -, and spaces)
normalize_keywords() {
  local input_file=$1
  local output_file=$2

  cat $input_file | \
    sed 's/_/ /g' | \
    sed 's/-/ /g' | \
    tr -s ' ' '\n' | \
    grep -v '^$' | \
    awk 'length($0) >= 3' | \
    sort -u > $output_file
}

echo "   Normalizing grandparent keywords..."
normalize_keywords $OUTPUT_DIR/curated_grandparent_tags.txt $OUTPUT_DIR/normalized_grandparent_tags.txt

echo "   Normalizing parent keywords..."
normalize_keywords $OUTPUT_DIR/curated_parent_tags.txt $OUTPUT_DIR/normalized_parent_tags.txt

echo "   Normalizing filename keywords..."
normalize_keywords $OUTPUT_DIR/curated_filename_tags.txt $OUTPUT_DIR/normalized_filename_tags.txt

NORM_GP=$(wc -l < $OUTPUT_DIR/normalized_grandparent_tags.txt)
NORM_P=$(wc -l < $OUTPUT_DIR/normalized_parent_tags.txt)
NORM_F=$(wc -l < $OUTPUT_DIR/normalized_filename_tags.txt)

echo "   ✅ Normalized: $NORM_GP grandparent, $NORM_P parent, $NORM_F filename"
echo ""

echo "📋 Step 5: Create Master Tag List (Combined + Deduplicated)"
echo ""

cat $OUTPUT_DIR/normalized_grandparent_tags.txt \
    $OUTPUT_DIR/normalized_parent_tags.txt \
    $OUTPUT_DIR/normalized_filename_tags.txt | \
  sort -u | \
  grep -E '^[a-z0-9]+$' | \
  awk 'length($0) >= 3 && length($0) <= 50' > $OUTPUT_DIR/master_tag_list.txt

MASTER_COUNT=$(wc -l < $OUTPUT_DIR/master_tag_list.txt)
echo "   ✅ Created master tag list: $MASTER_COUNT unique tags"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Curated Tag Lists Created"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Summary:"
echo "   • Grandparent tags (curated):   $GRANDPARENT_COUNT"
echo "   • Parent tags (curated):        $PARENT_COUNT"
echo "   • Filename tags (curated):      $FILENAME_COUNT"
echo "   • Grandparent tags (normalized):$NORM_GP"
echo "   • Parent tags (normalized):     $NORM_P"
echo "   • Filename tags (normalized):   $NORM_F"
echo "   • Master tag list (deduplicated): $MASTER_COUNT"
echo ""
echo "📁 Files created in $OUTPUT_DIR:"
echo "   • curated_grandparent_tags.txt (original multi-word)"
echo "   • curated_parent_tags.txt (original multi-word)"
echo "   • curated_filename_tags.txt (original multi-word)"
echo "   • normalized_grandparent_tags.txt (single words)"
echo "   • normalized_parent_tags.txt (single words)"
echo "   • normalized_filename_tags.txt (single words)"
echo "   • master_tag_list.txt (all unique single-word tags)"
echo ""
echo "✅ Ready for tag dictionary creation!"
echo ""

# Show top 20 most common tags from master list
echo "🔝 Preview of master tag list (first 30):"
head -30 $OUTPUT_DIR/master_tag_list.txt | nl
echo ""
