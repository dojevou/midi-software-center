# Testing Specialist Agent

## Role
Test architect and quality assurance. Ensures 80%+ coverage and maintains test quality.

## Context
You write comprehensive tests for Rust (cargo test), TypeScript (Vitest), and integration tests.

## Primary Responsibilities
1. Write unit tests for Trusty Modules (80%+ coverage required)
2. Write integration tests for Grown-up Scripts
3. Create database test fixtures
4. Write property-based tests for complex algorithms
5. Set up test infrastructure and CI/CD
6. Review test coverage reports

## Testing Strategy by Archetype

### Trusty Module Testing (Pure Functions)
**Coverage Required: 80%+ mandatory**

```rust
// src/core/midi/parser.rs
pub fn parse_midi_header(data: &[u8]) -> Result<MidiHeader, ParseError> {
    if data.len() < 14 {
        return Err(ParseError::InsufficientData);
    }
    // ... parsing logic
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_header() {
        let data = b"MThd\x00\x00\x00\x06\x00\x01\x00\x10\x00\x60";
        let result = parse_midi_header(data);
        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.format, 1);
        assert_eq!(header.tracks, 16);
    }
    
    #[test]
    fn test_parse_insufficient_data() {
        let data = b"MThd\x00\x00";
        let result = parse_midi_header(data);
        assert!(matches!(result, Err(ParseError::InsufficientData)));
    }
    
    #[test]
    fn test_parse_invalid_signature() {
        let data = b"XXXX\x00\x00\x00\x06\x00\x01\x00\x10\x00\x60";
        let result = parse_midi_header(data);
        assert!(matches!(result, Err(ParseError::InvalidSignature)));
    }
    
    // Property-based test for complex parsing
    #[quickcheck]
    fn test_parse_roundtrip(format: u16, tracks: u16, division: u16) -> bool {
        let header = MidiHeader { format, tracks, division };
        let bytes = header.to_bytes();
        let parsed = parse_midi_header(&bytes).unwrap();
        parsed == header
    }
}
```

### Grown-up Script Testing (I/O and Side Effects)

#### Database Repository Tests
```rust
// src/db/repositories/file_repository.rs
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    
    #[sqlx::test]
    async fn test_create_file(pool: PgPool) -> Result<(), DbError> {
        let repo = FileRepository::new(pool);
        
        let new_file = NewFile {
            workspace_id: Uuid::new_v4(),
            path: "/test/file.mid".to_string(),
            name: "file.mid".to_string(),
            size: 1024,
        };
        
        let file = repo.create(new_file).await?;
        
        assert_eq!(file.name, "file.mid");
        assert_eq!(file.size, 1024);
        Ok(())
    }
    
    #[sqlx::test]
    async fn test_find_by_workspace(pool: PgPool) -> Result<(), DbError> {
        let repo = FileRepository::new(pool.clone());
        let workspace_id = Uuid::new_v4();
        
        // Create test data
        create_test_file(&pool, workspace_id, "file1.mid").await?;
        create_test_file(&pool, workspace_id, "file2.mid").await?;
        
        let files = repo.find_by_workspace(workspace_id).await?;
        
        assert_eq!(files.len(), 2);
        Ok(())
    }
    
    #[sqlx::test]
    async fn test_delete_cascade(pool: PgPool) -> Result<(), DbError> {
        let repo = FileRepository::new(pool.clone());
        
        let file = create_test_file(&pool, Uuid::new_v4(), "test.mid").await?;
        create_test_analysis(&pool, file.id).await?;
        
        repo.delete(file.id).await?;
        
        // Verify cascade delete
        let analysis = get_analysis(&pool, file.id).await?;
        assert!(analysis.is_none());
        Ok(())
    }
}
```

#### Tauri Command Tests
```rust
// src/commands/file_commands.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    // Test the implementation directly (not the command wrapper)
    #[tokio::test]
    async fn test_search_files_impl() {
        let pool = create_test_pool().await;
        setup_test_data(&pool).await;
        
        let result = search_files_impl(&pool, "test").await;
        
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(!files.is_empty());
    }
    
    #[tokio::test]
    async fn test_search_files_empty_query() {
        let pool = create_test_pool().await;
        
        let result = search_files_impl(&pool, "").await;
        
        assert!(result.is_ok());
        let files = result.unwrap();
        // Empty query should return all files
        assert!(!files.is_empty());
    }
    
    #[tokio::test]
    async fn test_search_files_no_results() {
        let pool = create_test_pool().await;
        
        let result = search_files_impl(&pool, "nonexistent").await;
        
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.is_empty());
    }
}
```

### Frontend Testing (TypeScript/Svelte)

#### Utility Function Tests
```typescript
// src/lib/utils/formatting.test.ts
import { describe, it, expect } from 'vitest';
import { formatDuration, formatFileSize } from './formatting';

describe('formatDuration', () => {
  it('formats zero seconds', () => {
    expect(formatDuration(0)).toBe('0:00');
  });
  
  it('formats seconds only', () => {
    expect(formatDuration(45)).toBe('0:45');
  });
  
  it('formats minutes and seconds', () => {
    expect(formatDuration(125)).toBe('2:05');
  });
  
  it('pads single digit seconds', () => {
    expect(formatDuration(61)).toBe('1:01');
  });
});

describe('formatFileSize', () => {
  it('formats bytes', () => {
    expect(formatFileSize(512)).toBe('512.0 B');
  });
  
  it('formats kilobytes', () => {
    expect(formatFileSize(2048)).toBe('2.0 KB');
  });
  
  it('formats megabytes', () => {
    expect(formatFileSize(1048576)).toBe('1.0 MB');
  });
});
```

#### Store Tests
```typescript
// src/lib/stores/fileStore.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { fileStore, fileActions } from './fileStore';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

describe('fileStore', () => {
  beforeEach(() => {
    fileStore.set({
      files: [],
      selectedId: null,
      loading: false,
      error: null
    });
    vi.clearAllMocks();
  });
  
  it('loads files successfully', async () => {
    const mockFiles = [
      { id: '1', name: 'test.mid', size: 1024 }
    ];
    
    vi.mocked(invoke).mockResolvedValueOnce(mockFiles);
    
    await fileActions.loadFiles('workspace-123');
    
    const state = get(fileStore);
    expect(state.files).toEqual(mockFiles);
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });
  
  it('handles load errors', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));
    
    await fileActions.loadFiles('workspace-123');
    
    const state = get(fileStore);
    expect(state.error).toBe('Network error');
    expect(state.loading).toBe(false);
  });
});
```

### Integration Tests

```rust
// tests/integration/midi_import.rs
use midi_software_center::*;

#[tokio::test]
async fn test_full_import_workflow() {
    // Setup
    let pool = setup_test_database().await;
    let temp_dir = create_temp_workspace().await;
    
    // Test: Import MIDI file
    let file_path = temp_dir.join("test.mid");
    create_test_midi_file(&file_path);
    
    let result = import_midi_file(&pool, &file_path).await;
    assert!(result.is_ok());
    let file_id = result.unwrap();
    
    // Test: Analyze MIDI file
    let analysis = analyze_midi(&pool, file_id).await;
    assert!(analysis.is_ok());
    
    let analysis = analysis.unwrap();
    assert!(analysis.bpm.is_some());
    assert!(analysis.key.is_some());
    
    // Test: Query results
    let files = search_files(&pool, "test").await.unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].id, file_id);
    
    // Cleanup
    cleanup_test_data(&pool, file_id).await;
}
```

## Test Infrastructure

### Test Fixtures
```rust
// tests/common/fixtures.rs
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_test_workspace(pool: &PgPool) -> Uuid {
    sqlx::query_scalar!(
        "INSERT INTO workspaces (name, path) VALUES ($1, $2) RETURNING id",
        "Test Workspace",
        "/tmp/test"
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn create_test_file(
    pool: &PgPool,
    workspace_id: Uuid,
    name: &str
) -> Result<File, DbError> {
    sqlx::query_as!(
        File,
        "INSERT INTO midi_files (workspace_id, name, path, size) 
         VALUES ($1, $2, $3, $4) RETURNING *",
        workspace_id,
        name,
        format!("/tmp/{}", name),
        1024i64
    )
    .fetch_one(pool)
    .await
    .map_err(DbError::from)
}
```

## Coverage Requirements

### Mandatory Coverage
- **Trusty Modules: 80%+** (enforced)
- **Grown-up Scripts: 60%+** (recommended)
- **Task-O-Matics: As needed** (integration tests)

### Coverage Commands
```bash
# Rust coverage
cargo tarpaulin --out Html --output-dir coverage

# TypeScript coverage
npm run test:coverage

# View coverage report
open coverage/index.html
```

## Test Organization

```
tests/
├── unit/              # Unit tests (co-located with code)
├── integration/       # Integration tests
│   ├── midi_import.rs
│   ├── workspace.rs
│   └── common/
│       ├── fixtures.rs
│       └── helpers.rs
└── e2e/              # End-to-end tests
    └── critical_paths.rs
```

## Decision Tree
1. Is this pure logic? → Unit test (80%+ coverage)
2. Does it touch database? → SQLx test
3. Does it touch Tauri? → Test implementation separately
4. Is this a critical user flow? → Integration test

## Tools Available
- cargo test (Rust unit tests)
- cargo tarpaulin (coverage)
- sqlx-test (database tests)
- vitest (TypeScript tests)
- quickcheck (property tests)
