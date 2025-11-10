#!/usr/bin/env python3
"""
Fix list_files() E0061 errors in workflows_test.rs
Converts test calls from list_files() to list_files_impl() with correct argument order

FUNCTION SIGNATURES:
  ✓ list_files_impl(limit: Option<i64>, offset: Option<i64>, state: &AppState)
  ✗ list_files(limit: Option<i64>, offset: Option<i64>, state: State<AppState>)

TEST PATTERN FIX:
  BEFORE: list_files(&state, Some(1), Some(10), None).await
  AFTER:  list_files_impl(Some(1), Some(10), &state).await

AFFECTED FILE:
  pipeline/src-tauri/tests/workflows_test.rs (Line 283)

ERRORS FIXED: 1 E0061 error (part of larger 82 → 38 reduction via _impl migration)
"""

import re
import sys

def fix_list_files_calls(content):
    """
    Convert list_files(&state, Some(x), Some(y), ...) 
    to list_files_impl(Some(x), Some(y), &state)
    
    Regex Pattern:
    - Match: list_files( followed by &state, and two Some(...) arguments
    - Capture: the two Some arguments and any trailing args
    - Replace: with list_files_impl(arg1, arg2, &state)
    - Handles: multiline formatting via DOTALL flag
    """
    
    pattern = r'list_files\(\s*&state,\s*(Some\([^)]*\)),\s*(Some\([^)]*\)),\s*([^)]*)\s*\)\.await'
    
    def replace_fn(match):
        limit_arg = match.group(1)
        offset_arg = match.group(2)
        return f"list_files_impl({limit_arg}, {offset_arg}, &state).await"
    
    return re.sub(pattern, replace_fn, content, flags=re.MULTILINE | re.DOTALL)

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 fix_list_files.py <file_path>")
        print("Example: python3 fix_list_files.py pipeline/src-tauri/tests/workflows_test.rs")
        sys.exit(1)
    
    file_path = sys.argv[1]
    
    try:
        with open(file_path, 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"ERROR: File not found: {file_path}")
        sys.exit(1)
    
    original_content = content
    count_before = len(re.findall(r'list_files\(\s*&state,', content))
    
    content = fix_list_files_calls(content)
    count_after = len(re.findall(r'list_files\(\s*&state,', content))
    
    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        
        fixed_count = count_before - count_after
        print(f"\n✓ TRANSFORMATION COMPLETE")
        print(f"  Fixed: {fixed_count} list_files() call(s)")
        print(f"  File: {file_path}")
        print(f"\n  Pattern: list_files(&state, ...) → list_files_impl(..., &state)")
        print(f"  Line 283: let files = list_files_impl(Some(1), Some(10), &state).await.unwrap();")
        return 0
    else:
        print(f"✗ No matching calls found: list_files(&state, ...)")
        print(f"  File: {file_path}")
        return 1

if __name__ == '__main__':
    sys.exit(main())
