#!/bin/bash

file="pipeline/src-tauri/tests/workflows_test.rs"

# This is complex because we need to reorder multi-line function calls
# We'll use a Python script to handle this properly

python3 << 'PYTHON'
import re

with open("pipeline/src-tauri/tests/workflows_test.rs", "r") as f:
    content = f.read()

# Pattern: import_single_file_impl with 2-3 line arguments
# Current: import_single_file_impl(\n        &state,\n        file_path,\n    )
# Target: import_single_file_impl(\n        file_path,\n        None,\n        &state,\n    )

# Find all occurrences of import_single_file_impl with &state as first arg
pattern = r'import_single_file_impl\(\s*&state,\s*([^,\)]+(?:\.[^,\)]*)?),\s*\)'

def replace_func(match):
    file_path = match.group(1).strip()
    return f'import_single_file_impl(\n        {file_path},\n        None,\n        &state,\n    )'

content = re.sub(pattern, replace_func, content, flags=re.MULTILINE | re.DOTALL)

with open("pipeline/src-tauri/tests/workflows_test.rs", "w") as f:
    f.write(content)

print("Fixed import_single_file_impl argument order")
PYTHON
