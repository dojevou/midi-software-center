# Codebase Q&A - MIDI Software Center

**Generated:** December 2025
**Purpose:** Answers to common questions about architectural decisions and implementation details

## 1. Lua Scripting Integration

**Question:** Is Lua intended for user macros, instrument control, or automation? How should agents extend Lua bindings?

**Answer:**

**Current Status:** Lua scripting is **not yet implemented** but dependencies are ready.

**Configuration:**
```toml
# From app/src-tauri/Cargo.toml
mlua = { version = "0.9", features = ["lua54", "async", "serialize", "vendored"] }
```

**Intended Use Cases (Future):**
1. **User Macros** - Record/playback MIDI sequences with transformations
2. **Instrument Control** - CC automation, note pattern generation
3. **Batch Transformations** - Auto-transpose based on detected key, velocity scaling

**Example Future Usage:**
```lua
-- User script: transpose all files by detected key to C major
for file in midi_library:search("piano") do
    local detected_key = midi.analyze_key(file)
    local transpose_amount = key_to_c_major(detected_key)
    midi.transpose(file, transpose_amount)
end
```

**Implementation Plan (Not Yet Started):**
- Runtime: `app/src-tauri/src/scripting/lua_runtime.rs` (planned, doesn't exist yet)
- Bindings: Expose core analysis functions (`analyze_bpm`, `analyze_key`, `transpose`)
- Safety: Sandbox file system access to `./midi-library/` only

**For Agents:** No Lua code exists yet. When implementing:
- Use `mlua::Lua::new()` with restricted environment (no `io`, `os` modules)
- Bind async functions with `mlua::async` feature
- Expose MIDI types via `UserData` trait

---

## 2. Hardware MIDI I/O Backend Selection

**Question:** How is the backend selected? What are performance tradeoffs? Should agents know about buffer sizes, sample rates, or callback threading?

**Answer:**

**Auto-Selection at Runtime** (from `app/src-tauri/src/hardware/midi_backend.rs`):

```rust
pub enum MidiBackendType {
    Jack,       // ~3ms latency, lock-free ringbuffers
    AlsaRaw,    // ~5ms latency, direct kernel access
    AlsaSeq,    // ~8-15ms latency, general purpose
    CoreMidi,   // ~5ms latency, macOS native
    Midir,      // ~10-15ms latency, cross-platform fallback
    Auto,       // Default: platform-detection
}
```

**Selection Algorithm:**
1. **macOS:** CoreMIDI (always, native framework)
2. **Linux:** Try JACK → ALSA Raw → ALSA Seq → midir (fallback)
3. **Windows:** midir only (cross-platform)

**Performance Tradeoffs:**

| Backend | Latency | Pros | Cons |
|---------|---------|------|------|
| JACK | ~3ms | Lowest latency, pro audio standard | Requires JACK daemon running |
| ALSA Raw | ~5ms | Direct kernel access, no daemon | Linux only, exclusive device access |
| CoreMIDI | ~5ms | Native macOS, zero setup | macOS only |
| midir | ~10-15ms | Works everywhere, no setup | Higher latency |

**Buffer Sizes:**
- **JACK:** Fixed-size frames (set by JACK server, typically 256 samples @ 48kHz = 5.3ms)
- **midir:** Variable-size messages (no fixed buffer)
- **ALSA:** Configurable buffer (default 1024 bytes)

**Threading Model:**
- All backends use **callback threads** (not main thread)
- Events delivered via `crossbeam_channel` to decouple from audio thread
- Zero-copy parsing with `wmidi::MidiMessage` (borrow from event buffer)

**For Agents Adding MIDI Features:**
- **Test both backends:** Low-latency (JACK) and fallback (midir)
- **Mock hardware in tests:** Use `#[cfg(test)]` to avoid real device dependencies
  ```rust
  #[cfg(test)]
  fn create_mock_backend() -> MidiBackend { /* ... */ }
  ```
- **Handle variable timing:** Timestamp events with microsecond precision (`timestamp_us`)
- **No sample rate assumptions:** MIDI is event-based, not sample-based

---

## 3. Search & Indexing (Meilisearch Integration)

**Question:** When to use Meilisearch vs. PostgreSQL queries? What's the sync strategy? Pagination limits, ranking, typo tolerance?

**Answer:**

**Current Status:** Meilisearch is **configured but NOT actively used**.

**Actual Implementation (from `app/src-tauri/src/commands/pipeline/search.rs`):**

```rust
// Current search uses pure PostgreSQL ILIKE queries
pub async fn search_files_impl(query: String, filters: SearchFilters, ...) {
    let items = sqlx::query_as::<_, SearchResultItem>(
        r#"
        SELECT f.id, f.filename, f.filepath, mm.bpm, mm.key_signature, ...
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE
            ($1::text = '' OR f.filename ILIKE '%' || $1 || '%' OR f.filepath ILIKE '%' || $1 || '%')
            AND ($2::text IS NULL OR fc.primary_category::text = $2)
            AND ($3::float8 IS NULL OR mm.bpm >= $3)
            AND ($4::float8 IS NULL OR mm.bpm <= $4)
        "#
    ).fetch_all(pool).await?;
}
```

**Why Not Meilisearch Yet?**
- PostgreSQL `ILIKE` queries are **fast enough** for current scale (<10ms for most searches)
- 60+ database indexes optimize common patterns (BPM range, key, instrument)
- Vector search (`pgvector`) planned for similarity queries

**When to Use Each:**

| Use Case | Solution | Why |
|----------|----------|-----|
| Exact filename match | PostgreSQL `ILIKE` | Already indexed, <10ms |
| BPM/key range filters | PostgreSQL indexed queries | Native numeric comparison |
| **Full-text search** | **Meilisearch (future)** | Typo tolerance, relevance ranking |
| **Faceted search UI** | **Meilisearch (future)** | Pre-aggregated facets (genre, artist) |
| **Similarity search** | **pgvector (future)** | Find similar chord progressions, melodies |

**Future Meilisearch Integration Plan:**
1. **Sync Strategy:** Eventual consistency via database triggers
   - PostgreSQL trigger on `files` table → enqueue update
   - Background worker syncs to Meilisearch every 30 seconds
2. **Indexed Fields:** `filename`, `filepath`, `tags`, `analysis_results` (JSON)
3. **Ranking:** BPM match > key match > tag match > filename similarity
4. **Typo Tolerance:** Enable for filename (e.g., "beethoven" → "bethovn")

**Pagination:**
- PostgreSQL: `LIMIT/OFFSET` (100 items max per page)
- Meilisearch (future): 1000 items max, cursor-based pagination recommended

**For Agents:** Use PostgreSQL for all searches currently. When adding Meilisearch:
- Keep PostgreSQL as source of truth
- Meilisearch is **read-only search cache**
- Re-sync on search failures (eventual consistency fallback)

---

## 4. Workspace Setup (Hidden Gotchas)

**Question:** PostgreSQL version enforcement? Environment variables? Port conflicts? MIDI test data? Platform-specific setup?

**Answer:**

### Required Versions

**PostgreSQL 16+ REQUIRED:**
```sql
-- Needs pgvector extension (not in PG <16)
CREATE EXTENSION IF NOT EXISTS vector;
```

Older PostgreSQL versions will fail migrations. Docker Compose enforces this:
```yaml
postgres:
  image: postgres:16-bookworm  # Locked version
```

### Environment Variables

**Auto-configured in `docker-compose.yml`:**
```bash
DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library
MEILISEARCH_URL=http://localhost:7700
RUST_LOG=info,midi_app=debug
RUST_BACKTRACE=1
```

**Override for tests:**
```bash
export DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library_test
cargo test --workspace --lib -- --test-threads=1
```

No `.env` file needed – Docker Compose sets everything.

### Port Conflicts (Common Issue)

**NON-STANDARD PORTS to avoid conflicts:**

| Service | Host Port | Container Port | Why Non-Standard? |
|---------|-----------|----------------|-------------------|
| PostgreSQL | **5433** | 5432 | Avoids conflict with system PostgreSQL |
| Meilisearch | 7700 | 7700 | Standard port |
| Redis | 6379 | 6379 | Standard port (optional, `--profile full`) |
| Svelte Dev | 5173 | - | Standard Vite port |

**Gotcha:** Tests connect to `:5433` by default. If you change the port:
```rust
// Update in tests:
let database_url = "postgresql://midiuser:145278963@localhost:YOUR_PORT/midi_library";
```

### MIDI Test Data

**Expected Location:** `./midi-library/` (create manually)

**Setup Test Files:**
```bash
mkdir -p midi-library/test-files
# Add sample MIDI files (10-100 files recommended)
cp ~/Downloads/*.mid midi-library/test-files/
```

**For Tests:**
```bash
# Import test files
./scripts/run-pipeline-ultra-fast.sh midi-library/test-files

# Verify import
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -c "SELECT COUNT(*) FROM files;"
```

### Platform-Specific Setup

**Linux:**
```bash
# Install ALSA development libraries (optional, for low-latency MIDI)
sudo apt install libasound2-dev libjack-jackd2-dev

# Build with JACK/ALSA support
cargo build --features jack,alsa
```

**macOS:**
```bash
# No additional setup (CoreMIDI built-in)
cargo build  # CoreMIDI auto-enabled on macOS
```

**Windows:**
```bash
# Uses midir (cross-platform, no setup needed)
cargo build
```

### Common Setup Failures

1. **"Database connection failed"**
   - Check: `docker ps` – is `midi-postgres` running?
   - Fix: `make docker-up`

2. **"Migrations failed: syntax error"**
   - Check: PostgreSQL version (must be 16+)
   - Fix: `docker-compose down -v && make docker-up` (nukes data!)

3. **"cargo: command not found"**
   - Fix: Install Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

4. **"sqlx-cli not found"**
   - Fix: `make setup` installs all tools

5. **"Frontend build failed"**
   - Check: Node.js version (18+ recommended)
   - Fix: `cd app && npm install`

6. **"Tests hang indefinitely"**
   - Cause: Missing `--test-threads=1` (parallel tests deadlock on database)
   - Fix: Always use `cargo test --workspace --lib -- --test-threads=1`

---

## 5. DAW vs Pipeline Commands (Clear Examples)

**Question:** What's the semantic difference? Different error handling? Behavioral examples?

**Answer:**

### Pipeline Commands: Batch, Durable, Partial Failure Tolerance

**Location:** `app/src-tauri/src/commands/pipeline/`

**Semantic Pattern:**
- **Goal:** Process large datasets (1K-1M files)
- **Durability:** Results persist in database
- **Partial Failures:** Expected and tracked
- **Progress:** Emit events during execution
- **Retries:** Safe to retry partial failures

**Example Command:**
```rust
// Import 1M files: some will fail, return stats
#[tauri::command]
pub async fn import_files(
    paths: Vec<String>,
    state: State<'_, AppState>
) -> Result<ImportStats, String> {
    let mut stats = ImportStats::default();

    for path in paths {
        match import_single_file(&path, &state).await {
            Ok(file_id) => {
                stats.successful += 1;
                stats.file_ids.push(file_id);
            },
            Err(e) => {
                stats.failed += 1;
                stats.errors.push(format!("{}: {}", path, e));
                // KEEP GOING - don't fail entire batch
            }
        }

        // Emit progress every 100 files
        if (stats.successful + stats.failed) % 100 == 0 {
            state.emit("import_progress", &stats)?;
        }
    }

    Ok(stats)  // Returns partial results
}
```

**Error Handling:**
- **Continue on error:** Collect errors in `Vec<String>`
- **Idempotent:** Safe to re-run (deduplication by hash)
- **Transactional per item:** Each file commits independently

**Frontend Usage:**
```typescript
const stats = await invoke<ImportStats>('import_files', { paths });
console.log(`Imported ${stats.successful}, failed ${stats.failed}`);
// User sees partial success, can retry failures
```

---

### DAW Commands: Real-Time, Stateful, All-or-Nothing

**Location:** `app/src-tauri/src/commands/daw/`

**Semantic Pattern:**
- **Goal:** Immediate state change (start playback, trigger note)
- **Real-Time:** Executes in <1ms (sub-frame latency)
- **Stateful:** Modifies sequencer/mixer state
- **All-or-Nothing:** Either succeeds or fails, no partial results
- **No Retries:** Failure requires user intervention

**Example Command:**
```rust
// Start sequencer: either playing or error
#[tauri::command]
pub async fn start_sequencer(
    engine: State<'_, Arc<SequencerEngine>>
) -> Result<(), String> {
    // Fail-fast: if already playing, return error immediately
    if engine.is_playing().await {
        return Err("Sequencer already playing".to_string());
    }

    // All-or-nothing: either starts or fails
    engine.start().await.map_err(|e| e.to_string())?;

    Ok(())  // No partial results, no stats
}
```

**Error Handling:**
- **Fail-fast:** Return error immediately, don't continue
- **Stateful:** Rollback state on error (sequencer stops if start fails)
- **Synchronous semantics:** Caller waits for immediate result

**Frontend Usage:**
```typescript
try {
    await invoke('start_sequencer');  // void return
    console.log("Playing");
} catch (err) {
    // Show error modal: "Cannot start: already playing"
    showError(err);
}
```

---

### Key Behavioral Differences

| Aspect | Pipeline Commands | DAW Commands |
|--------|-------------------|--------------|
| **Purpose** | Batch data processing | Real-time state control |
| **Failure Model** | Partial failure OK | All-or-nothing |
| **Return Type** | `Result<Stats, String>` | `Result<(), String>` |
| **Progress** | Emit events (`import_progress`) | No progress (instant) |
| **Idempotency** | Safe to retry | Not safe (state changes) |
| **Latency** | Seconds to hours | <1ms |
| **Example** | Import 1M files (200 fail) | Trigger MIDI note |

---

### Real-World Error Handling Comparison

**Pipeline (Resilient):**
```rust
for file in batch {
    match process(file) {
        Ok(result) => {
            stats.successful += 1;
            commit_to_db(result);  // Per-item commit
        },
        Err(e) => {
            stats.failed += 1;
            log::error!("Failed {}: {}", file, e);
            // KEEP PROCESSING - don't stop batch
        }
    }
}
Ok(stats)  // Return partial results
```

**DAW (Fail-Fast):**
```rust
// Validate state BEFORE any changes
if !engine.can_start() {
    return Err("Invalid state".to_string());
}

// Make atomic state change
engine.state.write().await.status = Playing;
engine.midi_clock.start().await?;  // If fails, ROLLBACK
engine.audio_thread.notify();

Ok(())  // All succeeded or all failed
```

---

## Summary

1. **Lua Scripting:** Not implemented yet; dependencies ready for future user macros/automation
2. **MIDI Backend:** Auto-selects JACK → ALSA → CoreMIDI → midir based on latency; test both low-latency and fallback
3. **Search:** Pure PostgreSQL now; Meilisearch configured but unused (future full-text search)
4. **Environment:** PostgreSQL 16+ required, port 5433 (non-standard), no `.env` needed
5. **Commands:** Pipeline = batch/resilient/stats; DAW = real-time/fail-fast/stateful

**For AI Agents:** This document provides implementation details beyond the quick-reference copilot instructions. Refer here when making architectural decisions that require understanding tradeoffs.
