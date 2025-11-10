#!/usr/bin/env python3

import re

# Fix pool references in test files
test_files = [
    "pipeline/src-tauri/tests/search_repository_test.rs",
    "pipeline/src-tauri/tests/tag_repository_test.rs",
    "pipeline/src-tauri/tests/file_repository_test.rs",
    "pipeline/src-tauri/tests/metadata_repository_test.rs",
]

for file_path in test_files:
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Replace patterns like:
        # some_function(pool) with some_function(&pool)
        # But skip if already has &pool
        # This is tricky, let me just fix common patterns
        
        # Pattern: Repository method calls passing pool without &
        # like: search_repository::SearchRepository::search(pool, ...)
        # should be: search_repository::SearchRepository::search(&pool, ...)
        content = re.sub(
            r'SearchRepository::search\(([^&])',
            r'SearchRepository::search(&\1',
            content
        )
        
        # Similar for other repository methods
        content = re.sub(
            r'TagRepository::new\(([^&])',
            r'TagRepository::new(&\1',
            content
        )
        
        with open(file_path, 'w') as f:
            f.write(content)
        
        print(f"Fixed {file_path}")
    except FileNotFoundError:
        pass

print("Fixed pool reference E0308 errors")
