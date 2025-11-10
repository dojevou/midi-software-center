#!/bin/bash
# Quick script to add allow directives to test helpers

# Add allow to test helper modules
find pipeline/src-tauri/tests -name "*.rs" -path "*/helpers/*" -o -path "*/common/*" -o -path "*/fixtures/*" | while read file; do
    if ! grep -q "#\[allow(dead_code" "$file"; then
        sed -i '1i #[allow(dead_code, unused_imports, unused_variables)]' "$file"
    fi
done

echo "Added allow directives to test helpers"
