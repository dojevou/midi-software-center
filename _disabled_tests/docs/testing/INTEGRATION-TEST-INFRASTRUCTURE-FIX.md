# Integration Test Infrastructure Fix Guide

**Priority:** P2 (Week 1-2)
**Estimated Effort:** 4-6 hours
**Impact:** Enable 53+ integration tests in CI/CD
**Status:** Ready for implementation

---

## 🎯 Objective

Fix Tauri mock compatibility issues to enable integration tests in CI/CD pipeline.

**Current Status:** Integration tests disabled due to type mismatch
```
MockWindow type ≠ tauri::Window type
```

**Target:** Generic test infrastructure that works with both real and mock Tauri types

---

## 📋 Problem Analysis

### Root Cause

Integration tests use `MockWindow` (test helper) but functions expect `tauri::Window` (production type).

**Error Example:**
```rust
// In file_import_test.rs:2626
let result = import_single_file(
    file_path.to_str().unwrap().to_string(),
    None,
    tauri::State::from(&state),
    MockWindow::new()  // ❌ Type mismatch: expected tauri::Window
).await;
```

### Why This Happens

The `import_single_file` function signature requires:
```rust
pub async fn import_single_file(
    path: String,
    tag_id: Option<i32>,
    state: tauri::State<AppState>,
    window: Window,  // ← This expects tauri::Window
) -> Result<FileMetadata>
```

But tests provide `MockWindow` which only implements `Emitter` trait.

---

## ✅ Solution: Generic Emitter Trait Wrapper

### Step 1: Create Generic Wrapper Trait

**File:** `pipeline/src-tauri/tests/common/emitter_wrapper.rs` (NEW)

```rust
use tauri::Emitter;

/// Generic wrapper trait for any type that implements Emitter
pub trait EmitterWrapper: Emitter + Send + Sync + Clone {
    fn as_emitter(&self) -> &dyn Emitter;
}

// Implementation for real Tauri Window
impl<'r> EmitterWrapper for tauri::Window {
    fn as_emitter(&self) -> &dyn Emitter {
        self as &dyn Emitter
    }
}

// Implementation for MockWindow
pub struct MockWindow {
    events: Arc<Mutex<Vec<MockEvent>>>,
}

#[derive(Debug, Clone)]
struct MockEvent {
    name: String,
    payload: String,
}

impl Clone for MockWindow {
    fn clone(&self) -> Self {
        Self {
            events: Arc::clone(&self.events),
        }
    }
}

impl Emitter for MockWindow {
    fn emit<S: serde::Serialize + Clone>(&self, event: &str, payload: S) -> tauri::Result<()> {
        // Implementation (from existing code)
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| tauri::Error::Emit(e.to_string()))?;

        let events_clone = Arc::clone(&self.events);
        let event_name = event.to_string();

        tokio::spawn(async move {
            events_clone.lock().await.push(MockEvent {
                name: event_name,
                payload: payload_json,
            });
        });

        Ok(())
    }

    fn emit_to<S: serde::Serialize + Clone>(
        &self,
        _target: &str,
        event: &str,
        payload: S,
    ) -> tauri::Result<()> {
        self.emit(event, payload)
    }
}

impl EmitterWrapper for MockWindow {
    fn as_emitter(&self) -> &dyn Emitter {
        self as &dyn Emitter
    }
}
```

### Step 2: Create Test Helper Module

**File:** `pipeline/src-tauri/tests/common/mod.rs` (UPDATE)

```rust
pub mod emitter_wrapper;
pub use emitter_wrapper::{EmitterWrapper, MockWindow};

pub struct TestEmitter(Box<dyn Emitter + Send + Sync>);

impl TestEmitter {
    pub fn mock() -> MockWindow {
        MockWindow::new()
    }

    pub fn new<E: Emitter + Send + Sync + 'static>(emitter: E) -> Self {
        Self(Box::new(emitter))
    }
}

impl Emitter for TestEmitter {
    fn emit<S: serde::Serialize + Clone>(&self, event: &str, payload: S) -> tauri::Result<()> {
        self.0.emit(event, payload)
    }

    fn emit_to<S: serde::Serialize + Clone>(
        &self,
        target: &str,
        event: &str,
        payload: S,
    ) -> tauri::Result<()> {
        self.0.emit_to(target, event, payload)
    }
}
```

### Step 3: Update Test Functions

**Before:** Type-specific function signature
```rust
pub async fn import_single_file(
    path: String,
    tag_id: Option<i32>,
    state: tauri::State<AppState>,
    window: Window,  // ← Hard-coded type
) -> Result<FileMetadata>
```

**After:** Generic over any Emitter
```rust
use crate::tests::common::emitter_wrapper::EmitterWrapper;

pub async fn import_single_file<W: EmitterWrapper>(
    path: String,
    tag_id: Option<i32>,
    state: tauri::State<AppState>,
    window: W,  // ← Generic type parameter
) -> Result<FileMetadata>
```

### Step 4: Update Test Calls

**Before:**
```rust
let result = import_single_file(
    file_path.to_str().unwrap().to_string(),
    None,
    tauri::State::from(&state),
    MockWindow::new()  // ❌ Type mismatch
).await;
```

**After:**
```rust
use pipeline::tests::common::MockWindow;

let result = import_single_file(
    file_path.to_str().unwrap().to_string(),
    None,
    tauri::State::from(&state),
    MockWindow::new()  // ✅ Type compatible
).await;
```

---

## 🔄 Implementation Approach

### Option A: Minimal Change (Recommended for Week 1)

Create a type alias that accepts both real and mock Emitters:

```rust
// In pipeline/src-tauri/src/lib.rs or tests/common/mod.rs

use std::sync::Arc;
use tauri::Emitter;

/// Generic window type for testing and production
pub struct WindowAdapter {
    inner: Box<dyn Emitter + Send + Sync>,
}

impl WindowAdapter {
    pub fn new<E: Emitter + Send + Sync + 'static>(emitter: E) -> Self {
        Self {
            inner: Box::new(emitter),
        }
    }

    pub fn from_mock(mock: MockWindow) -> Self {
        Self::new(mock)
    }
}

impl Emitter for WindowAdapter {
    fn emit<S: serde::Serialize + Clone>(&self, event: &str, payload: S) -> tauri::Result<()> {
        self.inner.emit(event, payload)
    }

    fn emit_to<S: serde::Serialize + Clone>(
        &self,
        target: &str,
        event: &str,
        payload: S,
    ) -> tauri::Result<()> {
        self.inner.emit_to(target, event, payload)
    }
}
```

Then update function signatures:
```rust
pub async fn import_single_file(
    path: String,
    tag_id: Option<i32>,
    state: tauri::State<AppState>,
    window: impl Emitter,  // ← Accept any Emitter
) -> Result<FileMetadata>
```

### Option B: Feature Flag Approach (For Later)

Make test functions available only in test builds:

```rust
#[cfg(test)]
pub async fn import_single_file_test(
    path: String,
    state: tauri::State<AppState>,
    window: MockWindow,
) -> Result<FileMetadata>

#[cfg(not(test))]
pub async fn import_single_file(
    path: String,
    state: tauri::State<AppState>,
    window: Window,
) -> Result<FileMetadata>
```

---

## 🛠️ Implementation Checklist

### Phase 1: Setup (2 hours)
- [ ] Create `emitter_wrapper.rs` module
- [ ] Implement `MockWindow` with `Emitter` trait
- [ ] Implement wrapper trait for generics
- [ ] Add to `tests/common/mod.rs`

### Phase 2: Update Functions (1 hour)
- [ ] Update `import_single_file` signature
- [ ] Update `import_directory` signature
- [ ] Update `analyze` command signature
- [ ] Update other functions as needed

### Phase 3: Update Tests (1 hour)
- [ ] Update `file_import_test.rs` calls
- [ ] Update `performance_test.rs` calls
- [ ] Verify all tests compile
- [ ] Run full test suite

### Phase 4: Integration (1 hour)
- [ ] Enable tests in CI/CD
- [ ] Verify tests run in CI
- [ ] Document new test infrastructure
- [ ] Create test best practices guide

---

## 📊 Files to Modify

### New Files
- `pipeline/src-tauri/tests/common/emitter_wrapper.rs`

### Modified Files
- `pipeline/src-tauri/tests/common/mod.rs` - Add emitter module
- `pipeline/src-tauri/src/commands/file_import.rs` - Make generic
- `pipeline/src-tauri/src/commands/analyze.rs` - Make generic
- `pipeline/src-tauri/tests/file_import_test.rs` - Update calls
- `pipeline/src-tauri/tests/performance_test.rs` - Update calls
- `pipeline/src-tauri/tests/integration_test.rs` - Update calls

---

## ✅ Validation Steps

### After Implementation
```bash
# 1. Check compilation
cargo test --lib --workspace -- --test-threads=1

# 2. Run specific integration tests
cargo test --test file_import_test -- --test-threads=1

# 3. Check all pipeline tests
cargo test --package midi-pipeline -- --test-threads=1

# 4. Verify no regressions
cargo test --workspace -- --test-threads=1
```

### Expected Results
- ✅ All 388 core library tests pass
- ✅ file_import_test.rs compiles and runs
- ✅ performance_test.rs compiles and runs
- ✅ integration_test.rs compiles and runs
- ✅ Zero new failures introduced

---

## 📈 Timeline

**Estimated Effort:** 4-6 hours
**Best Time:** Week 1 or 2 (after critical fixes)
**Dependencies:** None (can be done in parallel with error handling fixes)

### Week 1-2 Integration Test Roadmap
- **Days 1-3:** Critical error handling fixes (parallel)
- **Days 3-5:** Integration test infrastructure (can start on Day 3)
- **Day 5-6:** Enable tests in CI/CD and validate

---

## 🚀 Success Criteria

After implementation:
- [ ] All integration tests compile without errors
- [ ] MockWindow properly implements Emitter
- [ ] Test functions accept generic Emitter types
- [ ] No regressions in existing tests
- [ ] Integration tests included in CI/CD pipeline

---

## 📝 Documentation

After completion, document:
1. **Architecture Decision:** Why generic Emitter approach
2. **Implementation Guide:** How test infrastructure works
3. **Best Practices:** How to write new integration tests
4. **Troubleshooting:** Common issues and solutions

---

## 🎯 Next Steps

### Immediate (Week 1)
1. Create emitter_wrapper.rs with MockWindow
2. Update critical command functions
3. Verify compilation

### Short-term (Week 2)
1. Update all test files
2. Enable in CI/CD
3. Document best practices

### Follow-up
1. Consider trait objects for broader compatibility
2. Evaluate feature flag approach for long-term
3. Create testing utilities library (Phase 3)

---

**Owner:** Integration Test Infrastructure Team
**Start Date:** Week 1-2 (after critical fixes)
**Target Completion:** Week 2 end
**Priority:** P2 (enables 53+ integration tests)

*This guide provides a clear path to fix the Tauri mock compatibility issues and enable comprehensive integration testing.*
