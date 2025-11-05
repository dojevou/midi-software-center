#!/bin/bash

file="pipeline/src-tauri/tests/file_import_test.rs"

# Replace calls to import_single_file( with import_single_file_impl(
# But keep import_single_file_impl already correct
sed -i 's/\bimport_single_file(/import_single_file_impl(/g' "$file"

# Now fix the calls that are still wrong - they need to remove the window parameter
# Pattern: import_single_file_impl(..., ..., state_clone, window)
# Should become: import_single_file_impl(..., ..., &state_clone)

python3 << 'PYTHON'
import re

file_path = "pipeline/src-tauri/tests/file_import_test.rs"

with open(file_path, 'r') as f:
    content = f.read()

# Fix calls that still have window parameter with state_clone
# Pattern: import_single_file_impl(..., state_clone, window,)
pattern = r'import_single_file_impl\(\s*([^,]+),\s*([^,]+),\s*(state_clone),\s*window,\s*\)'

def replace_func(match):
    arg1 = match.group(1).strip()
    arg2 = match.group(2).strip()
    state = match.group(3).strip()
    return f'import_single_file_impl(\n        {arg1},\n        {arg2},\n        &{state},\n    )'

content = re.sub(pattern, replace_func, content, flags=re.MULTILINE | re.DOTALL)

with open(file_path, 'w') as f:
    f.write(content)

print("Fixed E0308 type mismatch errors in file_import_test.rs")
PYTHON

echo "Fixed import function calls"
