# Quick Wins Checklist - Architecture Improvements

**MIDI Software Center - Actionable Items by Priority**

Generated: 2025-12-01

---

## 🔴 HIGH Priority (Immediate Impact)

### 1. Command Organization (2 hours)

**Current:** Commands scattered in `main.rs` or single `commands.rs` file
**Target:** Modular command structure

```bash
# Create command modules
mkdir -p app/src-tauri/src/commands
touch app/src-tauri/src/commands/{mod.rs,database.rs,pipeline.rs,daw.rs,mixer.rs,system.rs}
```

**Checklist:**
- [ ] Create `src/commands/mod.rs` with module exports
- [ ] Move database commands to `database.rs`
- [ ] Move pipeline commands to `pipeline.rs`
- [ ] Move DAW commands to `daw.rs`
- [ ] Update `main.rs` imports
- [ ] Test all commands still work

**Files:** See `/home/dojevou/projects/midi-software-center/STACK_BEST_PRACTICES_GUIDE.md` Section 1.1

---

### 2. Type-Safe Error Handling (1 hour)

**Current:** String errors from commands
**Target:** Structured error types with `thiserror`

```rust
// Add to dependencies
thiserror = "1.0"
```

**Checklist:**
- [ ] Create `src/error.rs` with `CommandError` enum
- [ ] Implement `Serialize` for errors
- [ ] Update commands to return `Result<T, CommandError>`
- [ ] Add error variants: `Database`, `FileNotFound`, `InvalidInput`, etc.

**Files:** See Section 1.4 in best practices guide

---

### 3. Tauri 2.0 Permissions (3 hours)

**Current:** Default wide-open permissions
**Target:** Granular capability-based security

**Checklist:**
- [ ] Create `capabilities/main.json` for main window
- [ ] Define custom permissions in `permissions/` directory
- [ ] Restrict filesystem access to MIDI library only
- [ ] Test commands require correct permissions
- [ ] Document permission requirements

**Files:** See Section 1.5 in best practices guide

---

## 🟠 MEDIUM Priority (Next Sprint)

### 4. Svelte Store Refactoring (4 hours)

**Current:** Multiple writable stores, no structure
**Target:** Organized, typed, derived stores

```bash
# Create store structure
mkdir -p app/src/lib/stores
touch app/src/lib/stores/{index.ts,types.ts,uiStore.ts,databaseStore.ts,pipelineStore.ts,playbackStore.ts}
```

**Checklist:**
- [ ] Create store factory functions with TypeScript types
- [ ] Add derived stores for computed values (filtered files, stats)
- [ ] Implement async store pattern for API calls
- [ ] Use immutable updates (`{...state, field: value}`)
- [ ] Export all stores from `index.ts`

**Files:** See Section 2 in best practices guide

---

### 5. Repository Pattern Enhancement (3 hours)

**Current:** Good foundation, some missing patterns
**Target:** Complete repository pattern with transactions

**Checklist:**
- [ ] Review existing repositories for completeness
- [ ] Add transaction wrapper methods
- [ ] Implement domain model aggregation (join multiple tables)
- [ ] Add comprehensive error handling
- [ ] Test transaction rollback scenarios

**Files:** See Section 3 in best practices guide

---

### 6. Workspace Dependency Consolidation (2 hours)

**Current:** Dependencies duplicated across member crates
**Target:** Centralized workspace dependencies

**Checklist:**
- [ ] Add `[workspace.dependencies]` to root `Cargo.toml`
- [ ] Move common deps to workspace level (tokio, sqlx, serde, etc.)
- [ ] Update member crates to use `workspace = true`
- [ ] Verify all crates still build
- [ ] Run `cargo test --workspace` to validate

**Files:** See Section 4 in best practices guide

---

## 🟢 LOW Priority (Nice to Have)

### 7. Derived Stores for Performance (2 hours)

**Target:** Reduce re-renders by 35%

**Checklist:**
- [ ] Create derived stores for filtered files
- [ ] Create derived stores for statistics
- [ ] Create derived stores for multi-store combinations
- [ ] Benchmark render performance before/after

**Files:** See Section 2.3 in best practices guide

---

### 8. CI/CD Multi-Platform Builds (3 hours)

**Current:** Manual builds per platform
**Target:** Automated GitHub Actions

**Checklist:**
- [ ] Create `.github/workflows/build.yml`
- [ ] Add matrix for `[macos, ubuntu, windows]`
- [ ] Configure platform-specific dependencies
- [ ] Test build artifacts for all platforms
- [ ] Add release automation with `tauri-action`

**Files:** See Section 5.5 in best practices guide

---

## Testing Checklist

After each change:

- [ ] `cargo check --workspace` - Verify all crates compile
- [ ] `cargo test --workspace` - Run all tests
- [ ] `pnpm check` - TypeScript validation
- [ ] `pnpm test` - Frontend tests
- [ ] Manual testing of affected commands

---

## Current Architecture Status

### ✅ Already Good
- SQLx with compile-time checks
- Repository pattern foundation
- Workspace structure
- Type-safe queries
- Connection pooling
- Performance optimization (mimalloc, parking_lot, ahash)

### 🔧 Needs Improvement
- Command organization (scattered)
- Error handling (string errors)
- Security permissions (too permissive)
- Store organization (no structure)
- Dependency duplication (workspace)

### 🆕 Missing
- Tauri 2.0 capabilities/permissions
- Derived stores
- Transaction wrappers
- Multi-platform CI/CD
- Context API (optional)

---

## Estimated Time Investment

**Week 1 (High Priority):**
- Day 1: Command organization (2h) + Error handling (1h) = 3h
- Day 2: Tauri permissions (3h)
- Day 3: Testing and validation (2h)

**Week 2 (Medium Priority):**
- Day 1: Store refactoring (4h)
- Day 2: Repository enhancements (3h)
- Day 3: Workspace consolidation (2h)

**Total:** ~17 hours for all HIGH + MEDIUM priority items

---

## Success Metrics

1. **Code Quality**
   - 0 string errors in commands ✓
   - 100% typed stores ✓
   - All commands in logical modules ✓

2. **Security**
   - Granular permissions for all commands ✓
   - Filesystem access restricted to MIDI library ✓
   - No exposed system APIs ✓

3. **Performance**
   - 35% reduction in re-renders (derived stores) ✓
   - No blocking UI operations (async commands) ✓
   - Fast compilation (workspace deps) ✓

4. **Maintainability**
   - Easy to find command definitions ✓
   - Clear store organization ✓
   - Consistent error handling ✓

---

**Next Step:** Start with HIGH priority items in order (Command Organization → Error Handling → Permissions)

**Reference:** `/home/dojevou/projects/midi-software-center/STACK_BEST_PRACTICES_GUIDE.md`
