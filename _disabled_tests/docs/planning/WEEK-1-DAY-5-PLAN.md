# Week 1 Day 5 Plan - Final Integration Push

**Date:** Friday, November 7, 2025
**Time Budget:** 6 hours
**Team:** Integration Test Engineer + Unit Test Engineer
**Status:** DECISION POINT - Choose Option A or B based on Day 3-4 progress

---

## Executive Summary

Day 5 represents the culmination of Week 1's testing initiative. We have two viable paths forward, each targeting 100% test compilation success but with different technical approaches. The team must make a strategic decision Friday morning based on Day 3-4 outcomes.

**Current Status:**
- 388/388 baseline tests passing (100%)
- 49 compilation errors remaining (down from 63)
- 53 integration tests blocked by infrastructure issues
- 1,223+ total tests written across 8 phases

**Goal:** Achieve full test suite compilation and establish foundation for Week 2 CI/CD integration

---

## Decision Tree

```
Friday Morning Assessment
│
├─ Day 3-4 Outcome: Infrastructure patterns working?
│  │
│  ├─ YES: Generic/wrapper approach validated
│  │  └─> Choose OPTION A (Integration Infrastructure Fix)
│  │      - 53 integration tests unblocked
│  │      - Scalable solution for future tests
│  │      - 6 hours sufficient for completion
│  │
│  └─ NO: Generic approach has issues
│     └─> Choose OPTION B (Extended Error Fixes)
│         - 14 remaining errors tackled individually
│         - Proven patterns from Days 1-2
│         - Guaranteed completion in 6 hours
│
└─ Output: Single unified test suite (1,223+ tests)
```

---

## Option A: Integration Infrastructure Fix (RECOMMENDED)

**When to Choose:**
- Day 3-4 successfully validated generic emitter pattern
- Team has bandwidth for architectural improvement
- Want scalable solution for future integration tests

**Time Budget:** 6 hours
- Phase 1: Emitter wrapper creation (1.5 hrs)
- Phase 2: Sequencer generics refactor (2 hrs)
- Phase 3: Test file updates (1.5 hrs)
- Phase 4: Validation and testing (1 hr)

### Phase 1: Create Emitter Wrapper (1.5 hours)

**Objective:** Build generic trait-based wrapper for test event emission

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/sequencer/emitter_wrapper.rs`

**Implementation:**

```rust
// Step 1.1: Core trait definition (20 min)
/// Generic trait for event emission - works with Tauri and test emitters
pub trait EventEmitter: Send + Sync {
    fn emit_event(&self, event_name: &str, payload: serde_json::Value);
}

// Step 1.2: Tauri implementation (20 min)
pub struct TauriEmitter<R: Runtime> {
    app_handle: AppHandle<R>,
}

impl<R: Runtime> EventEmitter for TauriEmitter<R> {
    fn emit_event(&self, event_name: &str, payload: serde_json::Value) {
        let _ = self.app_handle.emit(event_name, payload);
    }
}

// Step 1.3: Test mock implementation (30 min)
pub struct TestEmitter {
    events: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

impl EventEmitter for TestEmitter {
    fn emit_event(&self, event_name: &str, payload: serde_json::Value) {
        if let Ok(mut events) = self.events.lock() {
            events.push((event_name.to_string(), payload));
        }
    }
}

impl TestEmitter {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_events(&self) -> Vec<(String, serde_json::Value)> {
        self.events.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

// Step 1.4: Convenience constructor (20 min)
impl<R: Runtime> TauriEmitter<R> {
    pub fn new(app_handle: AppHandle<R>) -> Arc<Self> {
        Arc::new(Self { app_handle })
    }
}
```

**Validation Checklist:**
- [ ] Trait compiles without errors
- [ ] TauriEmitter implements EventEmitter
- [ ] TestEmitter provides event inspection
- [ ] Both implementations are Send + Sync
- [ ] Arc wrapping works correctly

### Phase 2: Sequencer Generics Refactor (2 hours)

**Objective:** Update sequencer functions to accept generic emitters

**Files to Modify:**
1. `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/sequencer/engine.rs`
2. `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/sequencer/mod.rs`

**Implementation Steps:**

**Step 2.1: Update SequencerEngine struct (30 min)**

```rust
// BEFORE:
pub struct SequencerEngine<R: Runtime> {
    app_handle: AppHandle<R>,
    // ... other fields
}

// AFTER:
pub struct SequencerEngine<E: EventEmitter> {
    emitter: Arc<E>,
    // ... other fields (no more R: Runtime generic)
}
```

**Step 2.2: Update method signatures (45 min)**

```rust
// Example: load_file function
// BEFORE:
impl<R: Runtime> SequencerEngine<R> {
    pub async fn load_file(&mut self, path: PathBuf) -> Result<(), String> {
        // ...
        self.app_handle.emit("playback_progress", progress)?;
    }
}

// AFTER:
impl<E: EventEmitter> SequencerEngine<E> {
    pub async fn load_file(&mut self, path: PathBuf) -> Result<(), String> {
        // ...
        self.emitter.emit_event("playback_progress",
                                serde_json::json!(progress));
    }
}
```

**Step 2.3: Update all emitter call sites (45 min)**

Affected functions (15 locations):
- `load_file()` - playback_progress event
- `start_playback()` - playback_started event
- `stop_playback()` - playback_stopped event
- `pause_playback()` - playback_paused event
- `seek()` - playback_seeked event
- `set_tempo()` - tempo_changed event
- `set_loop()` - loop_changed event
- `internal_tick()` - tick_event emission
- `emit_progress()` - progress updates
- `emit_error()` - error notifications

**Pattern for each:**
```rust
// OLD: self.app_handle.emit("event_name", data)?;
// NEW: self.emitter.emit_event("event_name", serde_json::json!(data));
```

**Validation Checklist:**
- [ ] All app_handle.emit() calls replaced
- [ ] Sequencer compiles with generic E
- [ ] No Runtime trait bounds remaining
- [ ] Existing Tauri commands still work
- [ ] cargo build succeeds

### Phase 3: Test File Updates (1.5 hours)

**Objective:** Update 8 integration test files to use new infrastructure

**Files:**
1. `/home/dojevou/projects/midi-software-center/daw/src-tauri/tests/integration/workflows_test.rs` (18 tests)
2. `/home/dojevou/projects/midi-software-center/daw/src-tauri/tests/integration/workflows_extended_test.rs` (20 tests)
3. `/home/dojevou/projects/midi-software-center/daw/src-tauri/tests/integration/performance_test.rs` (15 tests)
4. `/home/dojevou/projects/midi-software-center/daw/src-tauri/tests/integration/stress_test.rs` (12 tests)
5. `/home/dojevou/projects/midi-software-center/daw/src-tauri/tests/integration/journey_test.rs` (8 tests)

**Step 3.1: Update test imports (15 min per file = 75 min)**

```rust
// Add to each file:
use crate::sequencer::emitter_wrapper::{EventEmitter, TestEmitter};

// Remove:
use tauri::{AppHandle, Runtime};
```

**Step 3.2: Update test setup (45 min total)**

```rust
// Pattern for each test:

// OLD:
let app_handle = create_mock_app_handle(); // Doesn't exist
let mut engine = SequencerEngine::new(app_handle);

// NEW:
let emitter = TestEmitter::new();
let mut engine = SequencerEngine::new(Arc::clone(&emitter));
```

**Step 3.3: Update event assertions (30 min total)**

```rust
// Pattern for verifying events:

// OLD: Can't verify events from AppHandle
assert!(playback_started); // Hope it worked?

// NEW: Inspect TestEmitter
let events = emitter.get_events();
assert!(events.iter().any(|(name, _)| name == "playback_started"));
```

**Validation Checklist:**
- [ ] All 53 integration tests compile
- [ ] Event assertions are stronger than before
- [ ] No lingering AppHandle references
- [ ] Test isolation maintained (TestEmitter::clear())

### Phase 4: Validation & Testing (1 hour)

**Step 4.1: Compilation check (15 min)**
```bash
cd /home/dojevou/projects/midi-software-center
cargo build --workspace
cargo test --workspace --no-run  # Compile tests without running
```

**Step 4.2: Run test suite (30 min)**
```bash
# Baseline tests (should still pass)
cargo test --workspace --lib -- --test-threads=1

# Integration tests (newly unblocked)
cargo test --workspace --test workflows_test -- --test-threads=1
cargo test --workspace --test workflows_extended_test -- --test-threads=1
cargo test --workspace --test performance_test -- --test-threads=1
cargo test --workspace --test stress_test -- --test-threads=1
cargo test --workspace --test journey_test -- --test-threads=1

# Full suite
cargo test --workspace -- --test-threads=1
```

**Step 4.3: Verify metrics (15 min)**
- Total tests: 1,223+ (388 baseline + 452 generated + 53 integration + 330 others)
- Passing rate: Target 95%+ (some tests may need data adjustment)
- Compilation errors: 0
- Integration tests unblocked: 53

**Success Criteria:**
- [ ] Zero compilation errors across workspace
- [ ] 388 baseline tests still passing (100%)
- [ ] 53 integration tests compile successfully
- [ ] Production code (daw/pipeline binaries) still builds
- [ ] Event emission works in both production and test contexts

---

## Option B: Extended Error Fixes (FALLBACK)

**When to Choose:**
- Day 3-4 generic approach encountered issues
- Team prefers proven incremental fix patterns
- Need guaranteed completion in 6 hours

**Time Budget:** 6 hours
- Phase 1: Advanced error patterns (3 hrs)
- Phase 2: Infrastructure fixes (2 hrs)
- Phase 3: Final validation (1 hr)

### Phase 1: Advanced Error Patterns (3 hours)

**8 Advanced Patterns Remaining:**

#### Pattern 1: Async Mock Services (45 min)
**Error Type:** "cannot find function `create_mock_service`"
**Files Affected:** 3 test files
**Fix:** Implement async service mocker

```rust
// Create: tests/helpers/mock_service.rs
pub struct MockService {
    responses: Arc<Mutex<VecDeque<Response>>>,
}

impl MockService {
    pub fn with_responses(responses: Vec<Response>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses.into())),
        }
    }

    pub async fn handle_request(&self, req: Request) -> Response {
        self.responses.lock().unwrap()
            .pop_front()
            .unwrap_or(Response::default())
    }
}
```

#### Pattern 2: State Machine Helpers (45 min)
**Error Type:** "no method named `advance_state`"
**Files Affected:** 4 test files
**Fix:** Add state transition helpers

```rust
// Add to SequencerEngine impl:
#[cfg(test)]
pub fn advance_state(&mut self, target: PlaybackState) -> Result<(), String> {
    match (self.state, target) {
        (PlaybackState::Stopped, PlaybackState::Playing) => self.start_playback(),
        (PlaybackState::Playing, PlaybackState::Paused) => self.pause_playback(),
        // ... other transitions
        _ => Err(format!("Invalid transition: {:?} -> {:?}", self.state, target)),
    }
}
```

#### Pattern 3: Concurrent Test Fixtures (45 min)
**Error Type:** "temporary value dropped while borrowed"
**Files Affected:** 6 test files
**Fix:** Proper lifetime management

```rust
// Pattern:
let db_pool = setup_test_pool().await;  // Arc<Pool>
let repo = FileRepository::new(Arc::clone(&db_pool));  // Clone, don't borrow

// Instead of:
let repo = FileRepository::new(&setup_test_pool().await);  // Temporary!
```

#### Pattern 4: Progress Channel Validation (30 min)
**Error Type:** "mismatched types expected `Receiver<Event>` found `()`"
**Files Affected:** 2 test files
**Fix:** Create channel test helpers

```rust
pub fn create_progress_channel() -> (Sender<Event>, Receiver<Event>) {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    (tx, rx)
}

pub async fn collect_events(rx: &mut Receiver<Event>, timeout_ms: u64)
    -> Vec<Event>
{
    let mut events = Vec::new();
    let deadline = tokio::time::sleep(Duration::from_millis(timeout_ms));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            Some(event) = rx.recv() => events.push(event),
            _ = &mut deadline => break,
        }
    }
    events
}
```

#### Pattern 5: Trait Method Visibility (15 min)
**Error Type:** "method `internal_method` is private"
**Files Affected:** 3 test files
**Fix:** Add pub(crate) or test-only methods

```rust
// In production code:
impl SequencerEngine {
    // OLD: fn internal_tick(&mut self) { }
    // NEW:
    #[cfg_attr(test, visibility::make(pub))]
    pub(crate) fn internal_tick(&mut self) {
        // ... implementation
    }
}

// Or simpler:
#[cfg(test)]
pub fn internal_tick_test(&mut self) {
    self.internal_tick()
}
```

#### Pattern 6: Metadata Extraction Validation (30 min)
**Error Type:** "no field `extract_metadata` on type `Analyzer`"
**Files Affected:** 2 test files
**Fix:** Implement missing analysis method

```rust
// In shared/rust/src/core/analysis/analyzer.rs:
impl Analyzer {
    pub fn extract_metadata(&self, midi_file: &MidiFile)
        -> Result<Metadata, AnalysisError>
    {
        Ok(Metadata {
            duration: self.calculate_duration(midi_file)?,
            bpm: self.detect_bpm(midi_file)?,
            key: self.detect_key(midi_file)?,
            time_signature: self.detect_time_signature(midi_file)?,
            // ... other fields
        })
    }
}
```

#### Pattern 7: Error Propagation in Tests (30 min)
**Error Type:** "the `?` operator can only be used in a function that returns `Result`"
**Files Affected:** 8 test files
**Fix:** Add Result return types

```rust
// OLD:
#[tokio::test]
async fn test_something() {
    let result = operation().await?;  // ERROR
    assert_eq!(result, expected);
}

// NEW:
#[tokio::test]
async fn test_something() -> Result<(), Box<dyn std::error::Error>> {
    let result = operation().await?;
    assert_eq!(result, expected);
    Ok(())
}
```

#### Pattern 8: Mock Time Sources (30 min)
**Error Type:** "cannot find `MockClock` in this scope"
**Files Affected:** 3 test files
**Fix:** Implement test clock

```rust
pub struct MockClock {
    time: Arc<Mutex<Instant>>,
}

impl MockClock {
    pub fn new() -> Self {
        Self {
            time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn advance(&self, duration: Duration) {
        *self.time.lock().unwrap() += duration;
    }

    pub fn now(&self) -> Instant {
        *self.time.lock().unwrap()
    }
}
```

**Phase 1 Validation:**
- [ ] All 8 patterns implemented
- [ ] Test files using patterns compile
- [ ] Helpers are reusable across tests
- [ ] No new compilation errors introduced

### Phase 2: Infrastructure Fixes (2 hours)

**6 Infrastructure Issues:**

#### Issue 1: Test Module Structure (30 min)
**Problem:** Tests can't access internal modules
**Fix:** Add proper module declarations

```rust
// In src/lib.rs or main.rs:
#[cfg(test)]
pub mod test_helpers {
    pub mod fixtures;
    pub mod mock_service;
    pub mod assertions;
}
```

#### Issue 2: Feature Flags (20 min)
**Problem:** Test-only dependencies not available
**Fix:** Update Cargo.toml

```toml
[dev-dependencies]
mockall = "0.12"
proptest = "1.4"
criterion = "0.5"

[features]
test-helpers = []

[[test]]
name = "integration"
required-features = ["test-helpers"]
```

#### Issue 3: Database Test Isolation (30 min)
**Problem:** Tests interfere with each other
**Fix:** Transaction-based isolation

```rust
pub async fn with_test_transaction<F, T>(pool: &PgPool, test: F) -> T
where
    F: FnOnce(&mut Transaction<Postgres>) -> BoxFuture<T>,
{
    let mut tx = pool.begin().await.unwrap();
    let result = test(&mut tx).await;
    tx.rollback().await.unwrap();  // Always rollback
    result
}
```

#### Issue 4: Parallel Test Execution (20 min)
**Problem:** Database tests fail with --test-threads > 1
**Fix:** Use test database per thread

```rust
pub async fn setup_test_pool() -> Arc<PgPool> {
    let thread_id = std::thread::current().id();
    let db_name = format!("test_db_{:?}", thread_id);

    // Create dedicated test database
    create_database(&db_name).await;
    let pool = PgPool::connect(&format!("postgres://localhost/{}", db_name))
        .await
        .unwrap();
    Arc::new(pool)
}
```

#### Issue 5: Async Runtime Configuration (30 min)
**Problem:** Tests hang or timeout
**Fix:** Configure tokio runtime properly

```rust
// Add to each test file:
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_operations() {
    // Test code
}

// For blocking operations:
#[tokio::test]
async fn test_with_blocking() {
    let result = tokio::task::spawn_blocking(|| {
        // Blocking operation
    }).await.unwrap();
}
```

#### Issue 6: Cleanup Hooks (10 min)
**Problem:** Test artifacts left behind
**Fix:** Add Drop implementations

```rust
pub struct TestContext {
    pool: Arc<PgPool>,
    temp_dir: TempDir,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup is automatic for TempDir
        // Pool cleanup handled by Arc
    }
}
```

**Phase 2 Validation:**
- [ ] All infrastructure issues resolved
- [ ] Tests can run in parallel (where appropriate)
- [ ] No resource leaks detected
- [ ] Cleanup happens reliably

### Phase 3: Final Validation (1 hour)

**Same as Option A Phase 4** - Run full test suite and verify metrics

---

## Daily Schedule

### Morning (9:00 AM - 12:00 PM)

**9:00-9:30: Team Standup & Decision**
- Review Day 3-4 outcomes
- Assess generic emitter approach status
- Make Option A vs Option B decision
- Assign tasks to both engineers

**9:30-12:00: Primary Implementation**
- Option A: Phase 1 + Phase 2 (emitter wrapper + sequencer refactor)
- Option B: Phase 1 (advanced error patterns 1-4)

**12:00: Checkpoint**
- Verify morning work compiles
- Adjust afternoon schedule if needed

### Afternoon (1:00 PM - 5:00 PM)

**1:00-3:30: Complete Implementation**
- Option A: Phase 3 (test file updates)
- Option B: Phase 1 patterns 5-8 + Phase 2 (infrastructure)

**3:30-4:30: Validation & Testing**
- Run full test suite
- Fix any issues discovered
- Document any remaining blockers

**4:30-5:00: Week 2 Handoff Meeting**
- Present final metrics
- Review what worked / what didn't
- Create Week 2 priorities document
- Celebrate wins

---

## Success Criteria

### Option A Success:
- [ ] `emitter_wrapper.rs` created and working
- [ ] Sequencer functions accept generic emitters
- [ ] 53 integration tests compile
- [ ] Production code still builds and runs
- [ ] Event emission works in tests and production

### Option B Success:
- [ ] All 14 remaining errors fixed
- [ ] Total: 63/63 errors resolved (100%)
- [ ] All 452 generated tests compile
- [ ] Full test suite passes

### Shared Success Criteria:
- [ ] Zero compilation errors in workspace
- [ ] 388 baseline tests still passing
- [ ] Documentation updated (this file + summary)
- [ ] Week 2 handoff complete
- [ ] Team understands next priorities

---

## Risk Mitigation

### Risk 1: Generic approach too complex
**Mitigation:** Option B is ready as fallback
**Detection:** If Phase 1 takes >2 hours, switch to Option B
**Owner:** Integration Test Engineer

### Risk 2: Sequencer refactor breaks production
**Mitigation:** Keep Runtime-based constructor as alternative
**Detection:** `make build-daw` fails
**Owner:** Lead developer

### Risk 3: Time overrun
**Mitigation:** Prioritize compilation over test passing
**Detection:** 4:00 PM checkpoint shows <70% complete
**Owner:** Both engineers

### Risk 4: Database tests fail
**Mitigation:** Transaction isolation already proven in Phase 4
**Detection:** `cargo test --test-threads=1` fails
**Owner:** Database Test Engineer

---

## Deliverables

### Code Deliverables:
1. **Option A:**
   - `daw/src-tauri/src/sequencer/emitter_wrapper.rs` (new)
   - `daw/src-tauri/src/sequencer/engine.rs` (modified)
   - 5 integration test files (modified)

2. **Option B:**
   - `tests/helpers/mock_service.rs` (new)
   - `tests/helpers/state_machine.rs` (new)
   - `tests/helpers/mock_clock.rs` (new)
   - 14 test files (modified)

### Documentation Deliverables:
1. **WEEK-1-DAY-5-SUMMARY.md** - Final results and metrics
2. **INTEGRATION-TEST-GUIDE.md** - How to use emitter wrapper (Option A)
3. **TEST-PATTERNS-REFERENCE.md** - Advanced patterns catalog (Option B)
4. **WEEK-2-PRIORITIES.md** - Handoff to CI/CD team

### Metrics Deliverables:
1. Final compilation error count: 0 (target)
2. Test counts: 1,223+ total
3. Passing rate: 95%+ (target)
4. Coverage report: cargo tarpaulin output

---

## Week 2 Transition

### Handoff Topics:
1. **Test Infrastructure Status** - What's working, what needs refinement
2. **Known Issues** - Any tests that compile but fail
3. **CI/CD Integration Plan** - See PHASE-9-CI-CD-INTEGRATION.md
4. **Performance Baselines** - Metrics from performance_test.rs
5. **Documentation Gaps** - What still needs writing

### Next Week Priorities:
1. **CI/CD Pipeline** - GitHub Actions integration
2. **Coverage Analysis** - Run tarpaulin, identify gaps
3. **Performance Tuning** - Address any slow tests
4. **Flaky Test Fixes** - Stabilize intermittent failures
5. **Production Deploy** - Release candidate preparation

---

## Quick Reference

### Key Files:
- **Option A Core:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/sequencer/emitter_wrapper.rs`
- **Option A Tests:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/tests/integration/`
- **Option B Helpers:** `/home/dojevou/projects/midi-software-center/tests/helpers/`
- **This Plan:** `/home/dojevou/projects/midi-software-center/WEEK-1-DAY-5-PLAN.md`

### Commands:
```bash
# Compilation check
cargo build --workspace
cargo test --workspace --no-run

# Run baseline tests
cargo test --workspace --lib -- --test-threads=1

# Run integration tests
cargo test --workspace --test workflows_test -- --test-threads=1

# Full suite
cargo test --workspace -- --test-threads=1

# Coverage report
cargo tarpaulin --workspace --out Html
```

### Decision Matrix:
| Criteria | Option A | Option B |
|----------|----------|----------|
| Day 3-4 success | Required | Not required |
| Time risk | Medium | Low |
| Future scalability | High | Medium |
| Immediate success | 53 tests | 14 errors |
| Complexity | Higher | Lower |
| Recommendation | If Day 3-4 works | Otherwise |

---

## Notes

- This plan assumes Day 3-4 work is complete and assessed
- Both options lead to same outcome: full compilation success
- Option A is architecturally superior but requires validated foundation
- Option B is proven approach with guaranteed completion
- Team has full autonomy to choose based on Friday morning assessment
- Week 2 team will take over CI/CD integration regardless of option chosen

**Last Updated:** 2025-11-02
**Version:** 1.0
**Status:** Ready for Day 5 execution
