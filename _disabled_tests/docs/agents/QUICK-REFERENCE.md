# Claude Code Quick Reference

## 🎯 Which Agent For What?

### Quick Decision Matrix

```
┌─────────────────────────────────────────────────────────────┐
│                    WHAT ARE YOU DOING?                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Writing pure functions (no I/O, no async)                  │
│  └─> Rust Backend Architect                                 │
│                                                              │
│  Creating Tauri commands or async operations                │
│  └─> Tauri Command Specialist                               │
│                                                              │
│  Building UI components or managing frontend state          │
│  └─> Svelte Frontend Specialist                             │
│                                                              │
│  Working with database (SQL, migrations, queries)           │
│  └─> Database Architect                                     │
│                                                              │
│  Writing any kind of tests or checking coverage             │
│  └─> Testing Specialist                                     │
│                                                              │
│  Reviewing code or checking architecture compliance         │
│  └─> Code Reviewer                                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 🗂️ File Location → Agent Mapping

```
src-tauri/src/core/                → Rust Backend Architect
src-tauri/src/commands/            → Tauri Command Specialist
src-tauri/src/db/repositories/     → Database Architect
src/lib/components/                → Svelte Frontend Specialist
src/lib/stores/                    → Svelte Frontend Specialist
src/lib/utils/                     → Rust Backend or Svelte Frontend
database/migrations/               → Database Architect
tests/                             → Testing Specialist
```

## 🚨 Red Flags (Auto-Reject)

```
❌ .unwrap() in production code
❌ .expect() in production code  
❌ async or I/O in core/ directories
❌ Business logic in Svelte components
❌ Tauri commands without Entry + Implementation pattern
❌ Migrations without DOWN
❌ Foreign keys without indexes
❌ Trusty Modules with <80% test coverage
```

## ✅ Green Flags (Good Patterns)

```
✅ Result<T, E> with ? operator
✅ Entry + Implementation for Tauri commands
✅ Pure functions in core/
✅ State management in Svelte stores
✅ Repository pattern for database
✅ 80%+ test coverage for core/
✅ Error conversion at boundaries
```

## 📋 Common Commands

### Create Agent
```
> /agents
> Create new agent
> Choose Personal or Project
> Paste agent config
```

### Delegate to Agent
```
> @rust-backend Create BPM detector
> @tauri-commands Add file import command
> @svelte-frontend Build file browser component
> @database Create midi_analysis table
> @testing Add tests for bpm_detector
> @reviewer Review this PR
```

### Check Coverage
```bash
# Rust
cargo tarpaulin --out Html

# TypeScript
npm run test:coverage
```

## 🎨 Code Patterns Cheat Sheet

### Rust Backend Architect Pattern
```rust
// TRUSTY MODULE - Pure logic, no I/O
pub fn detect_bpm(midi: &MidiFile) -> Result<f64, BpmError> {
    // Pure algorithm
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_detect_bpm() {
        // 80%+ coverage required
    }
}
```

### Tauri Command Specialist Pattern
```rust
// ENTRY - Thin wrapper
#[tauri::command]
pub async fn analyze_file(
    id: String,
    state: State<'_, AppState>
) -> Result<Analysis, String> {
    analyze_file_impl(&state.db_pool, &id)
        .await
        .map_err(|e| e.to_string())
}

// IMPLEMENTATION - Testable
pub async fn analyze_file_impl(
    pool: &PgPool,
    id: &str
) -> Result<Analysis, DbError> {
    // Business logic here
}
```

### Database Architect Pattern
```sql
-- MIGRATION with UP and DOWN
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_files_created_at ON files(created_at DESC);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();
```

### Svelte Frontend Specialist Pattern
```svelte
<script lang="ts">
  import { store, actions } from '$lib/stores/store';
  
  export let id: string;
  
  $: data = $store.data;
  $: loading = $store.loading;
  
  async function handleAction() {
    await actions.doSomething(id);
  }
</script>

{#if loading}
  <Loading />
{:else}
  <Content {data} on:action={handleAction} />
{/if}
```

## 📊 Coverage Requirements

```
Trusty Modules (core/)       : 80%+ MANDATORY
Grown-up Scripts             : 60%+ RECOMMENDED
Task-O-Matics (UI)           : AS NEEDED
```

## 🔄 Typical Workflow

```
1. Plan
   └─> Identify archetype for each piece

2. Core Logic
   └─> @rust-backend: Create pure functions with tests

3. Database
   └─> @database: Create tables and repositories

4. Integration
   └─> @tauri-commands: Connect everything with commands

5. Frontend
   └─> @svelte-frontend: Build UI and stores

6. Test
   └─> @testing: Add integration tests

7. Review
   └─> @reviewer: Check everything before merge
```

## 🎯 Three Archetypes Quick Reference

```
TASK-O-MATIC
├─ What: Complete applications, entry points, UI
├─ Where: main.rs, bin/, *.svelte, routes/
└─ Test: Integration tests as needed

GROWN-UP SCRIPT
├─ What: I/O, async, side effects, state management
├─ Where: commands/, services/, repositories/, stores/
└─ Test: 60%+ recommended, mock I/O

TRUSTY MODULE
├─ What: Pure functions, no I/O, no side effects
├─ Where: core/, utils/, types/
└─ Test: 80%+ MANDATORY
```

## 💡 Pro Tips

1. **Always delegate** - Don't ask general Claude for specialized tasks
2. **Test while coding** - Not after
3. **Document while coding** - Not after
4. **Review before merge** - Always run Code Reviewer
5. **Coverage is mandatory** - 80%+ for core/

## 🆘 Common Issues

### "My agent isn't working"
- Check agent has correct context files
- Verify agent model (Sonnet for most, Opus for review)
- Ensure project files are accessible

### "Code Reviewer rejected my PR"
- Check for .unwrap()/.expect()
- Verify test coverage ≥80% for core/
- Ensure correct archetype classification
- Review error handling patterns

### "Tests are failing"
- Run `cargo test` locally first
- Check database is running for SQLx tests
- Verify test fixtures are set up
- Ensure pure functions have no side effects

## 📞 Quick Help

```
Architecture questions     → Review ARCHITECTURE-REFERENCE.md
Rust patterns             → backend-architecture.md
Frontend patterns         → frontend-architecture.md
Database patterns         → database-architecture.md
Testing patterns          → Run @testing for examples
```

## 🎓 Learning Resources

```
docs/architecture/layered/     → Layer-specific patterns
docs/architecture/examples/    → Concrete code examples
docs/architecture/workflows/   → Step-by-step processes
.cursor/rules/                → Language-specific rules
```

---

**Remember:** The Three Archetypes pattern keeps your code clean, testable, and maintainable!

```
Pure Logic (Trusty) → Tested at 80%+
Integration (Grown-up) → Handles I/O properly  
Complete Apps (Task-O-Matic) → Ties everything together
```
