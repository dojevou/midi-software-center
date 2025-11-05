#!/usr/bin/env python3
"""
Fix add_tags_to_file and add_tags_to_file_impl function call signatures.

Correct signature: (file_id, tag_names, state)
Wrong patterns:
  - add_tags_to_file(&state, file_id, tags) → add_tags_to_file(file_id, tags, &state)
  - add_tags_to_file_impl(file_id, tags) → add_tags_to_file_impl(file_id, tags, &state)
"""

import re
import sys
from pathlib import Path

def fix_add_tags_to_file(content: str) -> tuple[str, int]:
    """Fix add_tags_to_file calls with state-first pattern."""
    fixes = 0

    # Pattern 1: add_tags_to_file(&state, file_id, vec![...])
    pattern1 = r'add_tags_to_file\(\s*&state,\s*(\w+),\s*(vec!\[.*?\]),\s*\)'

    def replacer1(match):
        nonlocal fixes
        file_id = match.group(1)
        tags = match.group(2)
        fixes += 1
        return f'add_tags_to_file({file_id}, {tags}, &state)'

    # Pattern 2: add_tags_to_file(tauri::State(&state), file_id, vec![...])
    pattern2 = r'add_tags_to_file\(\s*tauri::State\(&state\),\s*(\w+),\s*(vec!\[.*?\]),\s*\)'

    def replacer2(match):
        nonlocal fixes
        file_id = match.group(1)
        tags = match.group(2)
        fixes += 1
        return f'add_tags_to_file({file_id}, {tags}, tauri::State(&state))'

    # Apply pattern 1 (simple &state)
    new_content = re.sub(pattern1, replacer1, content, flags=re.DOTALL)

    # Apply pattern 2 (tauri::State wrapper)
    new_content = re.sub(pattern2, replacer2, new_content, flags=re.DOTALL)

    # Handle multi-line formatted calls for pattern 1
    multiline_pattern1 = r'add_tags_to_file\(\s+&state,\s+([^,]+),\s+(vec!\[[^\]]*\]),\s+\)'
    new_content = re.sub(multiline_pattern1, replacer1, new_content, flags=re.DOTALL)

    # Handle multi-line formatted calls for pattern 2
    multiline_pattern2 = r'add_tags_to_file\(\s+tauri::State\(&state\),\s+([^,]+),\s+(vec!\[[^\]]*\]),\s+\)'
    new_content = re.sub(multiline_pattern2, replacer2, new_content, flags=re.DOTALL)

    # Pattern 3: Multi-line with newlines between arguments
    # add_tags_to_file(
    #     tauri::State(&state),
    #     file_id,
    #     tags,
    # )
    multiline_pattern3 = r'add_tags_to_file\(\s+tauri::State\(&state\),\s+([^,]+),\s+([^,\)]+),\s+\)'

    def replacer3(match):
        nonlocal fixes
        file_id = match.group(1).strip()
        tags = match.group(2).strip()
        fixes += 1
        return f'add_tags_to_file({file_id}, {tags}, tauri::State(&state))'

    new_content = re.sub(multiline_pattern3, replacer3, new_content, flags=re.DOTALL)

    return new_content, fixes

def fix_add_tags_to_file_impl(content: str) -> tuple[str, int]:
    """Fix add_tags_to_file_impl calls missing state parameter."""
    fixes = 0

    # Pattern: add_tags_to_file_impl(file_id, vec![...]).await
    # Should be: add_tags_to_file_impl(file_id, vec![...], &state).await
    pattern = r'add_tags_to_file_impl\(([^,]+),\s*(vec!\[[^\]]*\])\)\.await'

    def replacer(match):
        nonlocal fixes
        file_id = match.group(1)
        tags = match.group(2)
        fixes += 1
        return f'add_tags_to_file_impl({file_id}, {tags}, &state).await'

    return re.sub(pattern, replacer, content), fixes

def process_file(filepath: Path) -> bool:
    """Process a single file and return True if changes were made."""
    try:
        content = filepath.read_text()
        original = content

        # Apply both fixes
        content, fixes1 = fix_add_tags_to_file(content)
        content, fixes2 = fix_add_tags_to_file_impl(content)

        total_fixes = fixes1 + fixes2

        if content != original:
            filepath.write_text(content)
            print(f"✓ {filepath.relative_to(Path.cwd())}: {total_fixes} fixes")
            return True
        else:
            print(f"  {filepath.relative_to(Path.cwd())}: no changes needed")
            return False
    except Exception as e:
        print(f"✗ {filepath}: ERROR - {e}", file=sys.stderr)
        return False

def main():
    """Process all test files."""
    base = Path("/home/dojevou/projects/midi-software-center")

    # Find all test files that might contain add_tags_to_file calls
    test_files = [
        base / "pipeline/src-tauri/tests/workflows_test.rs",
        base / "_disabled_tests/workflows_test.rs",
        base / "_disabled_tests/workflows_extended_test.rs",
        base / "_disabled_tests/journey_test.rs",
        base / "_disabled_tests/performance_test.rs",
        base / "_disabled_tests/stress_test.rs",
    ]

    total_files = 0
    changed_files = 0

    print("Fixing add_tags_to_file function signatures...\n")

    for filepath in test_files:
        if filepath.exists():
            total_files += 1
            if process_file(filepath):
                changed_files += 1
        else:
            print(f"⚠ {filepath}: file not found (skipping)")

    print(f"\nSummary: {changed_files}/{total_files} files modified")
    return 0 if changed_files > 0 else 1

if __name__ == "__main__":
    sys.exit(main())
