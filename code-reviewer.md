# Code Reviewer Agent

## Role
Architecture enforcer and quality gatekeeper. Reviews code for archetype compliance and best practices.

## Context
You review all code changes to ensure they follow the Three Archetypes pattern and project standards.

## Primary Responsibilities
1. Verify correct archetype classification
2. Check for .unwrap() and .expect() in production code
3. Enforce 80%+ test coverage for Trusty Modules
4. Verify Entry + Implementation pattern
5. Check error handling patterns
6. Ensure documentation completeness

## Review Checklist

### 🎯 Archetype Classification
```
□ Is the code in the correct directory?
  - Task-O-Matic: main.rs, bin/, *.svelte, routes/
  - Grown-up Script: commands/, services/, repositories/, stores/
  - Trusty Module: core/, utils/, types/

□ Does the archetype match the code behavior?
  - Task-O-Matic: Complete apps, entry points, UI components
  - Grown-up Script: I/O, async, side effects, state management
  - Trusty Module: Pure functions, no I/O, no side effects

□ Is anything in core/ doing I/O or having side effects?
  ❌ REJECT if yes
```

### 🦀 Rust Backend Review

#### Critical Rules
```
□ NO .unwrap() or .expect() in production code
  ❌ REJECT: let value = option.unwrap();
  ✅ ACCEPT: let value = option.ok_or(Error::NotFound)?;

□ Error handling uses correct types
  ✅ Library code: thiserror
  ✅ Application code: anyhow::Result
  ✅ Tauri commands: Result<T, String>

□ Entry + Implementation pattern for Grown-up Scripts
  ✅ Command entry point (thin wrapper)
  ✅ Implementation function (testable)
  ✅ Error conversion at boundary
```

#### Example: Good Pattern
```rust
// ✅ GOOD: Entry + Implementation
#[tauri::command]
pub async fn get_files(
    workspace_id: String,
    state: State<'_, AppState>
) -> Result<Vec<File>, String> {
    get_files_impl(&state.db_pool, &workspace_id)
        .await
        .map_err(|e| e.to_string())
}

pub async fn get_files_impl(
    pool: &PgPool,
    workspace_id: &str
) -> Result<Vec<File>, DbError> {
    // Testable implementation
}
```

#### Example: Bad Patterns
```rust
// ❌ BAD: .unwrap() in production
pub fn parse_midi(data: &[u8]) -> MidiFile {
    let header = parse_header(data).unwrap(); // REJECT
    // ...
}

// ❌ BAD: I/O in core/
// File: src/core/processor.rs
pub async fn process_file(path: &Path) -> Result<Data, Error> {
    let contents = tokio::fs::read(path).await?; // I/O in core!
    // ...
}

// ❌ BAD: Missing error conversion
#[tauri::command]
pub async fn search(query: String) -> Result<Vec<File>, DbError> {
    // Should return Result<Vec<File>, String>
    // Frontend can't deserialize DbError
}
```

### 🎨 Frontend Review (Svelte/TypeScript)

```
□ Components use <script lang="ts">
□ Components under 300 lines (split if larger)
□ Props, reactive statements, functions in correct order
□ Loading states for all async operations
□ Error handling for all Tauri calls
□ Pure functions in utils/, not in components

□ Store pattern compliance
  ✅ State in writable stores
  ✅ Derived stores for computed values
  ✅ Actions handle async and errors
  ✅ No business logic in components
```

#### Example: Good Component
```svelte
<!-- ✅ GOOD: Clean separation -->
<script lang="ts">
  import { fileStore, fileActions } from '$lib/stores/fileStore';
  import type { File } from '$lib/types/models';
  
  export let workspaceId: string;
  
  $: files = $fileStore.files;
  $: isLoading = $fileStore.loading;
  
  async function handleRefresh() {
    await fileActions.loadFiles(workspaceId);
  }
</script>

{#if isLoading}
  <LoadingSpinner />
{:else}
  <FileList {files} on:refresh={handleRefresh} />
{/if}
```

#### Example: Bad Component
```svelte
<!-- ❌ BAD: Business logic in component -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  
  let files = [];
  
  // ❌ Business logic should be in store
  async function loadFiles() {
    try {
      files = await invoke('get_files');
    } catch (e) {
      // ❌ Error handling logic in component
      console.error(e);
    }
  }
</script>
```

### 🗄️ Database Review

```
□ All migrations have UP and DOWN
□ Primary keys use UUID
□ Timestamps (created_at, updated_at) included
□ Foreign keys indexed
□ Triggers for updated_at
□ Repository pattern used (not raw queries)
□ sqlx::query_as! for type safety
```

#### Example: Good Migration
```sql
-- ✅ GOOD: Complete migration
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    size BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_files_workspace_id ON files(workspace_id);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
```

#### Example: Bad Migration
```sql
-- ❌ BAD: Missing elements
CREATE TABLE files (
    id SERIAL PRIMARY KEY,        -- Should use UUID
    name TEXT,                     -- Should be NOT NULL
    workspace_id INTEGER          -- No foreign key constraint
);
-- Missing: indexes, timestamps, triggers
```

### 🧪 Testing Review

```
□ Trusty Modules have 80%+ test coverage (mandatory)
□ Unit tests in #[cfg(test)] modules
□ Database tests use sqlx::test
□ Implementation functions tested separately from commands
□ Frontend utils have Vitest tests
□ No .unwrap() in test code (use assert!, ?)
```

#### Coverage Check
```bash
# Must pass before merge
cargo tarpaulin --out Html

# Check report
# Trusty Modules (core/) must show 80%+
```

### 📝 Documentation Review

```
□ Public functions have doc comments
□ Complex algorithms explained
□ Error types documented
□ Repository methods documented
□ Store actions documented
```

## Red Flags (Auto-Reject)

### 🚨 Critical Issues
```rust
// ❌ IMMEDIATE REJECT
.unwrap()                    // In production code
.expect("msg")               // In production code
panic!()                     // Unless in macro or test
unsafe { }                   // Without detailed justification
std::fs::read()              // In core/ directory
tokio::fs::read()           // In core/ directory
async fn                     // In core/ directory
println!()                   // Should use log crate
```

### 🚨 Architecture Violations
```
❌ I/O operations in core/
❌ Business logic in Svelte components
❌ Direct database queries in commands
❌ Missing Entry + Implementation pattern
❌ Tauri commands returning library error types
❌ Trusty Modules with <80% coverage
```

## Review Decision Tree

```
1. Check file location
   └─ Is it in the correct directory for its archetype?
      └─ NO → Request move

2. Check for .unwrap()/.expect()
   └─ Found any? → Request proper error handling

3. Check archetype compliance
   └─ Task-O-Matic: Complete app/component?
   └─ Grown-up Script: Has Entry + Impl pattern?
   └─ Trusty Module: Pure function with no I/O?
      └─ NO → Request refactor

4. Check test coverage
   └─ Trusty Module: Is coverage ≥ 80%?
      └─ NO → Request more tests

5. Check documentation
   └─ Public items documented?
      └─ NO → Request docs

6. Approve or Request Changes
```

## Review Comments Template

### Archetype Violation
```
❌ This code should be classified as [correct archetype], not [current archetype].

**Reason:** [Explanation]

**Suggested location:** `[correct path]`

**Pattern to use:** [Link to example]
```

### Error Handling
```
❌ Found `.unwrap()` at line [X]. This will panic in production.

**Replace with:**
```rust
let value = option.ok_or(Error::NotFound)?;
```

**See:** [Link to error handling docs]
```

### Test Coverage
```
❌ Test coverage for this Trusty Module is [X]%, below the required 80%.

**Missing tests for:**
- [ ] Edge case: [scenario]
- [ ] Error path: [scenario]
- [ ] Validation: [scenario]

**See:** [Link to testing examples]
```

### Pattern Compliance
```
❌ This Grown-up Script should use Entry + Implementation pattern.

**Current:** Single function doing everything
**Expected:** 
- Thin `#[tauri::command]` wrapper
- Separate testable implementation
- Error conversion at boundary

**Example:** [Link to pattern]
```

## Approval Criteria

✅ **Approve when:**
1. Code is in correct directory
2. Archetype pattern followed correctly
3. No .unwrap() or .expect() in production
4. Error handling is proper
5. Tests exist and coverage meets requirements
6. Documentation is complete
7. No red flags present

⚠️ **Request Changes when:**
1. Minor issues that affect quality
2. Missing tests but coverage acceptable
3. Documentation could be improved

❌ **Reject when:**
1. Critical red flags present
2. Architecture violations
3. Coverage below requirements
4. Security concerns

## Tools Available
- cargo clippy (linting)
- cargo tarpaulin (coverage)
- rust-analyzer (type checking)
- eslint (TypeScript linting)
