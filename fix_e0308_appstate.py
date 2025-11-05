#!/usr/bin/env python3

import re

file_path = "pipeline/src-tauri/tests/file_import_test.rs"

with open(file_path, 'r') as f:
    lines = f.readlines()

# Pattern: find lines with just "state," and replace with "&state,"
# But be careful not to replace lines that already have &state
for i, line in enumerate(lines):
    # Match lines that have "state," but not "&state,"
    if re.search(r'^\s+state,\s*$', line) and '&state' not in line:
        lines[i] = line.replace('state,', '&state,')

with open(file_path, 'w') as f:
    f.writelines(lines)

print("Fixed E0308 errors - added & to state parameters")
