# CI Failure Analysis & Fix Guide

**Generated:** 2025-12-14
**Repository:** midi-software-center
**Status:** Multiple workflow failures on `main` branch

---

## Summary of Failures

| Workflow | Job | Status | Root Cause |
|----------|-----|--------|------------|
| CI | Build Check | FAILED | DAW frontend `dist` directory missing |
| Lint | Clippy | FAILED | SQLx compile-time type inference errors |
| Tests & Coverage | - | IN PROGRESS | Waiting on dependencies |
| Verification | - | IN PROGRESS | Waiting on dependencies |

---

## Issue 1: DAW Frontend Missing (`frontendDist` Error)

### Error Message
```
error: proc macro panicked
  --> daw/src-tauri/src/main.rs:602:14
    |
602 |         .run(tauri::generate_context!())?;
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: message: The `frontendDist` configuration is set to `"../dist"` but this path doesn't exist
```

### Root Cause
The DAW Tauri configuration expects a built frontend at `daw/dist`, but:
1. DAW has no frontend yet (only backend/Rust code)
2. CI creates a placeholder `dist` directory, but it's created **in the wrong location**

### Current CI Workflow (`.github/workflows/ci.yml` - Build Check job)
```yaml
- name: Build frontends for Tauri
  run: |
    cd app && pnpm install && pnpm build
    cd ../pipeline && pnpm install && pnpm build
    # DAW doesn't have a frontend yet - create placeholder
    mkdir -p daw/dist  # <-- BUG: Creates at root/daw/dist, not relative to src-tauri
    echo '<!DOCTYPE html><html><body>DAW Placeholder</body></html>' > daw/dist/index.html
```

### DAW Tauri Config (`daw/src-tauri/tauri.conf.json`)
```json
{
  "build": {
    "frontendDist": "../dist"   // Expects: daw/src-tauri/../dist = daw/dist
  }
}
```

### Current Directory Structure
```
daw/
├── dist/              # EXISTS (created by CI)
│   └── index.html     # Placeholder file
└── src-tauri/
    └── tauri.conf.json
```

### Why It's Failing
The `frontendDist: "../dist"` path is relative to `daw/src-tauri/`, so:
- Expected path: `daw/src-tauri/../dist` = `daw/dist` ✅
- The placeholder **should work** based on the directory listing

**ACTUAL ISSUE:** The lint workflow (`.github/workflows/lint.yml`) is **missing** the placeholder step!

### Lint Workflow Missing Placeholder (`.github/workflows/lint.yml`)
```yaml
clippy:
  name: Clippy
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    # ... dependencies ...
    - name: Run Clippy
      run: |
        cargo clippy --workspace --all-targets -- -D warnings
    # ❌ NO FRONTEND BUILD OR PLACEHOLDER CREATION!
```

---

## Issue 2: SQLx Type Inference Errors

### Error Messages
```
error[E0282]: type annotations needed
   --> daw/src-tauri/src/commands/analysis.rs:147:26
    |
147 |                     } else if (ratio - 0.75).abs() < 0.05 {
    |                               ^^^^^^^^^^^^^^ cannot infer type

error[E0282]: type annotations needed
   --> daw/src-tauri/src/commands/analysis.rs:196:5
    |
196 | /     sqlx::query!(
197 | |         "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
...   |
201 | |     .await
    | |__________^ cannot infer type
```

### Root Cause
SQLx's `query!` macro performs compile-time SQL validation which requires either:
1. **A live database connection** during compilation (via `DATABASE_URL`)
2. **Offline mode** (`SQLX_OFFLINE=true`) with pre-generated `.sqlx/` cache files

### Current Setup Analysis

**Workspace `.sqlx/` Directory:**
```
.sqlx/
├── query-02db9d86624bc094b6badd3fe14181c0d1d8ae1f3d2407546da8d51cd28c2a77.json
├── query-03f7568c18a129aa5ef8d4fecaddd6b92e6395a255c46b68cab384edd1de2bfb.json
... (52 files total)
```

**CI Environment (`ci.yml`):**
```yaml
env:
  SQLX_OFFLINE: true  # ✅ Set correctly

jobs:
  test:
    services:
      postgres:        # ✅ Database available for tests
        image: postgres:16
```

**Lint Workflow (`lint.yml`):**
```yaml
env:
  CARGO_TERM_COLOR: always
  # ❌ SQLX_OFFLINE NOT SET!

clippy:
  # ❌ NO DATABASE SERVICE!
  # ❌ NO DATABASE_URL!
```

### Why SQLx Fails in Clippy
1. `SQLX_OFFLINE` is not set → SQLx tries to connect to database
2. No database service running → Connection fails
3. No database connection → Type inference impossible
4. Clippy compilation fails

### Affected Files
| File | Line(s) | Query Type |
|------|---------|------------|
| `daw/src-tauri/src/commands/analysis.rs` | 147, 164, 196, 216, 231, 249 | `query!`, `query_as!` |
| `daw/src-tauri/src/commands/search.rs` | 230 | `query_as!` |
| `daw/src-tauri/src/commands/sequencer.rs` | 95 | `query!` |
| `daw/src-tauri/src/profiling/query_analyzer.rs` | 844 | `query!` |

---

## Fixes Required

### Fix 1: Update Lint Workflow (`.github/workflows/lint.yml`)

```yaml
name: Lint

on:
  push:
    branches: [main, master, develop]
  pull_request:
    branches: [main, master, develop]

env:
  CARGO_TERM_COLOR: always
  SQLX_OFFLINE: true  # ADD THIS

jobs:
  rustfmt:
    name: Rust Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libgtk-3-dev \
            libwebkit2gtk-4.1-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev \
            libasound2-dev \
            libc++-dev \
            libc++abi-dev \
            pkg-config

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Setup Node.js  # ADD THIS SECTION
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Setup pnpm  # ADD THIS SECTION
        uses: pnpm/action-setup@v2
        with:
          version: 8

      - name: Build frontends for Tauri  # ADD THIS SECTION
        run: |
          cd app && pnpm install && pnpm build
          cd ../pipeline && pnpm install && pnpm build
          mkdir -p daw/dist
          echo '<!DOCTYPE html><html><body>DAW Placeholder</body></html>' > daw/dist/index.html

      - name: Override native CPU flags for CI  # ADD THIS SECTION
        run: |
          mkdir -p .cargo
          echo '[target.x86_64-unknown-linux-gnu]' > .cargo/config.toml
          echo 'rustflags = []' >> .cargo/config.toml

      - name: Run Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

  # ... rest of the jobs unchanged
```

### Fix 2: Regenerate SQLx Cache (Local)

The `.sqlx/` cache may be incomplete. Regenerate it:

```bash
# Ensure database is running
make docker-up

# Set DATABASE_URL
export DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Regenerate cache for entire workspace
cargo sqlx prepare --workspace

# Verify cache was generated
ls -la .sqlx/ | wc -l  # Should be > 52 if new queries added

# Commit the updated cache
git add .sqlx/
git commit -m "chore: update SQLx query cache for CI"
git push
```

### Fix 3: Alternative - Use `query_unchecked!` (Quick but not recommended)

For rapid fix without regenerating cache, change specific queries:

```rust
// Before
sqlx::query!(
    "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
    file_id as i64
)

// After (loses compile-time validation)
sqlx::query_unchecked!(
    "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
    file_id as i64
)
```

**NOT RECOMMENDED** - Loses compile-time SQL validation benefits.

---

## Example Failing Query Code

### `daw/src-tauri/src/commands/analysis.rs:196-201`
```rust
#[tauri::command]
pub async fn add_favorite(file_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    debug!("Adding file {} to favorites", file_id);

    // Insert into favorites table (ON CONFLICT DO NOTHING to handle duplicates)
    sqlx::query!(
        "INSERT INTO favorites (file_id) VALUES ($1) ON CONFLICT (file_id) DO NOTHING",
        file_id as i64
    )
    .execute(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
    .await
    .map_err(|e| {
        error!("Failed to add favorite: {}", e);
        format!("Failed to add favorite: {}", e)
    })?;

    debug!("Successfully added file {} to favorites", file_id);
    Ok(())
}
```

### `daw/src-tauri/src/commands/sequencer.rs:95-109`
```rust
let file_result = sqlx::query!(
    r#"
    SELECT filepath
    FROM files
    WHERE id = $1
    "#,
    file_id as i64
)
.fetch_one(
    state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database pool not initialized".to_string())?,
)
.await
.map_err(|e| {
    error!("Failed to query file {} from database: {}", file_id, e);
    format!("File not found: {}", file_id)
})?;
```

---

## Configuration Files Reference

### Root `Cargo.toml` (SQLx Configuration)
```toml
[workspace.dependencies]
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "bigdecimal"] }
```

### DAW `Cargo.toml` (`daw/src-tauri/Cargo.toml`)
```toml
[dependencies]
sqlx = { workspace = true }
```

### DAW Tauri Config (`daw/src-tauri/tauri.conf.json`)
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "MIDI Library DAW",
  "version": "0.1.0",
  "identifier": "com.midilibrary.daw",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5174",
    "frontendDist": "../dist"
  }
}
```

---

## Action Plan

1. **Immediate Fix (Lint Workflow):**
   - Add `SQLX_OFFLINE: true` to environment
   - Add frontend build steps before Clippy
   - Add `.cargo/config.toml` override for CI

2. **Regenerate SQLx Cache:**
   - Run `cargo sqlx prepare --workspace` locally
   - Commit updated `.sqlx/` directory

3. **Verify Fix:**
   - Push changes
   - Monitor CI workflows
   - All 4 workflows should pass

---

## Commands to Apply Fixes

```bash
# 1. Apply lint workflow fix (edit the file with the content above)
# Then commit:
git add .github/workflows/lint.yml
git commit -m "ci: fix Clippy by adding SQLX_OFFLINE and frontend placeholder"

# 2. Regenerate SQLx cache
export DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
cargo sqlx prepare --workspace
git add .sqlx/
git commit -m "chore: regenerate SQLx query cache"

# 3. Push and verify
git push
gh run list --limit 5  # Watch for green checkmarks
```
