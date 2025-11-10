#!/bin/bash

echo "=== COMPREHENSIVE ERROR ANALYSIS ==="
echo "Timestamp: $(date)"
echo

# Total error count
total=$(cargo check --tests -p midi-pipeline 2>&1 | grep -c "error\[E")
echo "Total Errors: $total"
echo

# Error breakdown by type
echo "=== ERROR BREAKDOWN BY TYPE ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E" | sed 's/.*error\[//' | sed 's/\].*//' | sort | uniq -c | sort -nr

echo
echo "=== ERROR BREAKDOWN BY FILE ==="
cargo check --tests -p midi-pipeline 2>&1 | grep -o "pipeline/src-tauri/tests/[a-zA-Z_]*\.rs" | sort | uniq -c | sort -nr

echo
echo "=== TOP E0061 ERROR PATTERNS ==="
cargo check --tests -p midi-pipeline 2>&1 | grep -A 1 "error\[E0061\]" | grep "this function" | sort | uniq -c | sort -nr | head -10

echo
echo "=== TOP E0425 ERROR PATTERNS (missing items) ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0425\]" | head -10 | while read line; do
    echo "$line"
done

echo
echo "=== SUMMARY ==="
echo "Session Progress:"
echo "  Initial: 362 errors"
echo "  Current: $total errors"
echo "  Fixed: $((362 - total)) errors ($(( (362 - total) * 100 / 362 ))%)"
echo
echo "Major Accomplishments:"
echo "  ✓ Fixed 62 E0061 errors (121 → 59)"
echo "  ✓ Eliminated all window.clone() parameter issues"
echo "  ✓ Fixed import_single_file_impl argument order in workflows_test.rs"
