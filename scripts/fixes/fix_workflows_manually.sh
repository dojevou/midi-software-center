#!/bin/bash

# This script fixes the import_single_file_impl calls in workflows_test.rs
# Pattern: import_single_file_impl(\n        &state,\n        file_path,\n    )
# Target: import_single_file_impl(\n        file_path,\n        None,\n        &state,\n    )

file="pipeline/src-tauri/tests/workflows_test.rs"

# Create a backup
cp "$file" "${file}.bak"

# Use sed with address ranges to fix the pattern
# Find lines with import_single_file_impl( and fix the next 3 lines
sed -i '/^[[:space:]]*let.*import_single_file_impl($/{
    N
    s/\(&state,\)\n\([[:space:]]*\)\([^\n]*,\)/\2\3\n\2None,\n\2&state,/
}' "$file"

echo "Attempted fix with sed"
