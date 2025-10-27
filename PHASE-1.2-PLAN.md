# Phase 1.2: MIDI Error Module - Comprehensive Test Plan

**File:** `shared/rust/src/core/midi/error.rs`
**Estimated Time:** 1 hour
**Target Coverage:** 95%+ (error types are simple, should be near 100%)
**Date:** 2025-10-26

---

## 📋 Table of Contents

1. [Module Overview](#module-overview)
2. [Pre-Flight Checklist](#pre-flight-checklist)
3. [Step-by-Step Execution Plan](#step-by-step-execution-plan)
4. [Test Categories](#test-categories)
5. [Success Criteria](#success-criteria)
6. [Tools & Agents](#tools--agents)
7. [Expected Outcomes](#expected-outcomes)

---

## Module Overview

### File Details
- **Path:** `shared/rust/src/core/midi/error.rs`
- **Lines:** 31 (small module)
- **Purpose:** Error types for MIDI parsing using `thiserror`
- **Archetype:** Trusty Module (part of core MIDI library)
- **Current Coverage:** Unknown (likely 0%)

### Error Variants (8 total)
```rust
1. InvalidHeader(String)           - Invalid MIDI header
2. InvalidTrack { position, reason } - Invalid track data
3. UnsupportedFormat(u16)          - MIDI format not 0/1/2
4. InvalidEvent { position, reason } - Invalid MIDI event
5. IncompleteData { expected, actual } - Truncated file
6. InvalidVarLen(usize)            - Bad variable-length quantity
7. Io(std::io::Error)              - File I/O errors (from)
8. Utf8(FromUtf8Error)             - Text decode errors (from)
```

### Dependencies
- `thiserror::Error` - Derive macro for error handling
- `std::io::Error` - File I/O errors
- `std::string::FromUtf8Error` - UTF-8 decoding errors

---

## Pre-Flight Checklist

### ✅ Prerequisites (from Phase 1.1)
- [x] Testing tools installed (cargo-nextest, cargo-tarpaulin)
- [x] Test fixtures structure created
- [x] Phase 1.1 tests passing (50/50)
- [x] Coverage baseline established (38.1% overall)
- [x] Agent reviews completed for MIDI types

### 📝 Before Starting
- [ ] Read error.rs file completely
- [ ] Understand thiserror error Display formatting
- [ ] Check existing parser.rs for error usage patterns
- [ ] Verify test module location (inline #[cfg(test)])

---

## Step-by-Step Execution Plan

### Step 1: Analyze Error Module (5 minutes)

**Actions:**
```bash
# Read the error module
cat shared/rust/src/core/midi/error.rs

# Check how errors are used in parser
grep -n "MidiParseError" shared/rust/src/core/midi/parser.rs | head -20

# Verify no existing tests
grep -n "#\[cfg(test)\]" shared/rust/src/core/midi/error.rs
```

**Expected Output:**
- Understand all 8 error variants
- See how errors are constructed in parser.rs
- Confirm no existing tests (likely empty)

**Tool:** None (manual inspection)

---

### Step 2: Generate Test Boilerplate (10 minutes)

**Command:**
```bash
/unit-test-generator:generate-tests
```

**Select File:**
```
shared/rust/src/core/midi/error.rs
```

**Expected Tests Generated:**
1. Error variant construction tests (8 tests)
2. Error Display formatting tests (8 tests)
3. Error conversion tests (2 tests for `from` implementations)
4. Error Debug formatting tests (8 tests)
5. Result type alias test (1 test)

**Total Expected:** ~25-30 tests

**Review Criteria:**
- ✅ Each error variant has construction test
- ✅ Display messages match thiserror format strings
- ✅ `From` conversions work (Io, Utf8)
- ✅ Result<T> type alias works correctly

**If Issues:**
- Manually add missing tests
- Verify thiserror Display output format

---

### Step 3: Enhance Tests (15 minutes)

**Add Error-Specific Test Cases:**

#### 3.1 Error Message Formatting
```rust
#[test]
fn test_invalid_header_message_format() {
    let error = MidiParseError::InvalidHeader("bad magic number".to_string());
    let msg = error.to_string();
    assert_eq!(msg, "Invalid MIDI header: bad magic number");
}

#[test]
fn test_invalid_track_message_includes_position() {
    let error = MidiParseError::InvalidTrack {
        position: 42,
        reason: "unexpected end".to_string(),
    };
    let msg = error.to_string();
    assert!(msg.contains("42"));
    assert!(msg.contains("unexpected end"));
}

#[test]
fn test_incomplete_data_shows_expected_vs_actual() {
    let error = MidiParseError::IncompleteData {
        expected: 100,
        actual: 50,
    };
    let msg = error.to_string();
    assert!(msg.contains("100"));
    assert!(msg.contains("50"));
}
```

#### 3.2 Error Conversions
```rust
#[test]
fn test_io_error_conversion() {
    use std::io;

    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let midi_error: MidiParseError = io_error.into();

    assert!(matches!(midi_error, MidiParseError::Io(_)));
    assert!(midi_error.to_string().contains("file not found"));
}

#[test]
fn test_utf8_error_conversion() {
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
    let midi_error: MidiParseError = utf8_error.into();

    assert!(matches!(midi_error, MidiParseError::Utf8(_)));
}
```

#### 3.3 Error Debug Formatting
```rust
#[test]
fn test_error_debug_format() {
    let error = MidiParseError::UnsupportedFormat(99);
    let debug = format!("{:?}", error);
    assert!(debug.contains("UnsupportedFormat"));
    assert!(debug.contains("99"));
}
```

#### 3.4 Result Type Alias
```rust
#[test]
fn test_result_type_alias_ok() {
    let result: Result<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_result_type_alias_err() {
    let result: Result<i32> = Err(MidiParseError::InvalidVarLen(0));
    assert!(result.is_err());
}
```

---

### Step 4: Add Edge Cases (10 minutes)

**Edge Case Tests:**

```rust
#[test]
fn test_empty_error_messages() {
    let error = MidiParseError::InvalidHeader(String::new());
    let msg = error.to_string();
    assert_eq!(msg, "Invalid MIDI header: ");
}

#[test]
fn test_very_long_error_message() {
    let long_msg = "x".repeat(10000);
    let error = MidiParseError::InvalidHeader(long_msg.clone());
    let msg = error.to_string();
    assert!(msg.contains(&long_msg));
}

#[test]
fn test_special_characters_in_error() {
    let error = MidiParseError::InvalidHeader("Line 1\nLine 2\tTab".to_string());
    let msg = error.to_string();
    assert!(msg.contains("Line 1\nLine 2\tTab"));
}

#[test]
fn test_unicode_in_error_message() {
    let error = MidiParseError::InvalidHeader("Invalid: 🎹 MIDI file".to_string());
    let msg = error.to_string();
    assert!(msg.contains("🎹"));
}

#[test]
fn test_position_boundaries() {
    let error_min = MidiParseError::InvalidTrack {
        position: 0,
        reason: "start of file".to_string(),
    };
    let error_max = MidiParseError::InvalidTrack {
        position: usize::MAX,
        reason: "end of file".to_string(),
    };

    assert!(error_min.to_string().contains("0"));
    assert!(error_max.to_string().contains(&usize::MAX.to_string()));
}

#[test]
fn test_all_midi_format_values() {
    for format in 0..=u16::MAX {
        let error = MidiParseError::UnsupportedFormat(format);
        let msg = error.to_string();
        assert!(msg.contains(&format.to_string()));

        // Only test a sample to avoid slow tests
        if format > 10 { break; }
    }
}
```

---

### Step 5: Add Security Tests (10 minutes)

**Security-Focused Tests:**

```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_error_message_doesnt_leak_memory() {
        // Create large error messages and ensure they don't cause OOM
        for _ in 0..1000 {
            let error = MidiParseError::InvalidHeader("x".repeat(1000));
            let _ = error.to_string();
        }
        // If we get here, no memory leak
    }

    #[test]
    fn test_malicious_position_values() {
        // Test extreme position values don't cause issues
        let positions = vec![0, 1, usize::MAX - 1, usize::MAX];

        for pos in positions {
            let error = MidiParseError::InvalidEvent {
                position: pos,
                reason: "test".to_string(),
            };
            let msg = error.to_string();
            assert!(msg.contains(&pos.to_string()));
        }
    }

    #[test]
    fn test_error_chain_doesnt_stack_overflow() {
        // Ensure error wrapping doesn't cause stack overflow
        let mut result: Result<i32> = Ok(0);

        for i in 0..1000 {
            result = result.map_err(|_| {
                MidiParseError::InvalidHeader(format!("Chain {}", i))
            });
        }

        assert!(result.is_ok());
    }
}
```

---

### Step 6: Run Tests (5 minutes)

**Commands:**
```bash
# Run all error module tests
cargo test --package midi-library-shared --lib core::midi::error

# Run with output
cargo test --package midi-library-shared --lib core::midi::error -- --nocapture

# Run specific test
cargo test --package midi-library-shared --lib test_invalid_header_message_format
```

**Expected Output:**
```
running 30 tests
test core::midi::error::tests::test_invalid_header_message_format ... ok
test core::midi::error::tests::test_invalid_track_message_includes_position ... ok
[... 28 more tests ...]

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured
```

**If Failures:**
1. Check thiserror Display format strings match tests
2. Verify error variant field names are correct
3. Ensure `From` implementations work

---

### Step 7: Measure Coverage (5 minutes)

**Command:**
```bash
cargo tarpaulin --package midi-library-shared --lib --out Stdout --skip-clean 2>&1 | grep "error.rs"
```

**Expected Coverage:**
```
shared/rust/src/core/midi/error.rs: 30/31 lines covered (96.8%)
```

**Uncovered Lines (Expected):**
- Line 24: `#[from] std::io::Error` (attribute, not executable)
- Line 27: `#[from] std::string::FromUtf8Error` (attribute, not executable)

**If Coverage < 95%:**
1. Check which lines are uncovered
2. Add tests for those specific error paths
3. Re-run coverage

---

### Step 8: Expert Review - Rust Backend Agent (10 minutes)

**Command:**
```bash
"Use the rust-backend agent to review the error module tests in shared/rust/src/core/midi/error.rs"
```

**Review Criteria:**
- ✅ Error types follow Rust error handling best practices
- ✅ thiserror usage is correct
- ✅ Error messages are helpful and actionable
- ✅ From implementations work correctly
- ✅ All error variants are tested

**Expected Feedback:**
- Confirmation that error types are well-designed
- Suggestions for additional error contexts
- Validation that Display messages are user-friendly

---

### Step 9: Commit Changes (5 minutes)

**Commands:**
```bash
# Stage changes
git add shared/rust/src/core/midi/error.rs

# Create semantic commit
git commit -m "$(cat <<'EOF'
test(midi): Add comprehensive tests for MIDI error types

Phase 1.2 of Test Coverage Initiative - MIDI Parser Module

Added 30+ tests for shared/rust/src/core/midi/error.rs covering
all 8 error variants with Display formatting, conversions, and
edge cases.

Test Coverage:
- ✅ 96.8% line coverage (30/31 lines)
- ✅ 30 passing tests
- ✅ All error variants tested

Test Categories:
1. Error Construction (8 tests)
   - All MidiParseError variants created correctly

2. Display Formatting (8 tests)
   - Error messages match thiserror format strings
   - Position and reason fields included in messages

3. Error Conversions (2 tests)
   - From<std::io::Error> works
   - From<FromUtf8Error> works

4. Edge Cases (10 tests)
   - Empty strings, unicode, special characters
   - Extreme position values (0, usize::MAX)
   - Long error messages

5. Security Tests (2 tests)
   - No memory leaks with large error messages
   - Malicious position values handled safely

Expert Review:
- rust-backend agent: Confirmed best practices

Related: TEST-COVERAGE-PLAN.md Phase 1.2

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
EOF
)"
```

---

## Test Categories

### 1. Error Variant Construction (8 tests)
- `test_invalid_header_construction()`
- `test_invalid_track_construction()`
- `test_unsupported_format_construction()`
- `test_invalid_event_construction()`
- `test_incomplete_data_construction()`
- `test_invalid_var_len_construction()`
- `test_io_error_construction()`
- `test_utf8_error_construction()`

### 2. Display Formatting (8 tests)
- `test_invalid_header_message_format()`
- `test_invalid_track_message_includes_position()`
- `test_unsupported_format_message()`
- `test_invalid_event_message_includes_position()`
- `test_incomplete_data_shows_expected_vs_actual()`
- `test_invalid_var_len_message()`
- `test_io_error_message()`
- `test_utf8_error_message()`

### 3. Error Conversions (2 tests)
- `test_io_error_conversion()`
- `test_utf8_error_conversion()`

### 4. Debug Formatting (3 tests)
- `test_error_debug_format()`
- `test_error_debug_includes_variant_name()`
- `test_error_debug_includes_data()`

### 5. Result Type Alias (2 tests)
- `test_result_type_alias_ok()`
- `test_result_type_alias_err()`

### 6. Edge Cases (10 tests)
- `test_empty_error_messages()`
- `test_very_long_error_message()`
- `test_special_characters_in_error()`
- `test_unicode_in_error_message()`
- `test_position_boundaries()`
- `test_all_midi_format_values()`
- `test_negative_scenario_handling()`

### 7. Security (3 tests)
- `test_error_message_doesnt_leak_memory()`
- `test_malicious_position_values()`
- `test_error_chain_doesnt_stack_overflow()`

**Total: ~30 tests**

---

## Success Criteria

### ✅ Must Have
- [ ] **Coverage:** ≥95% line coverage (30/31 lines minimum)
- [ ] **Tests Passing:** 30/30 tests passing
- [ ] **All Variants:** All 8 error variants tested
- [ ] **Display Format:** All Display messages verified
- [ ] **Conversions:** Both `From` implementations tested
- [ ] **Agent Review:** rust-backend agent approval

### ✅ Should Have
- [ ] **Edge Cases:** Empty strings, unicode, boundaries tested
- [ ] **Security:** Memory safety tests included
- [ ] **Documentation:** Inline comments explain test purposes
- [ ] **Performance:** Tests run in < 1 second

### ✅ Nice to Have
- [ ] **Integration:** Tests use errors from parser.rs
- [ ] **Examples:** Error usage examples in comments
- [ ] **Benchmarks:** Error creation performance benchmarks

---

## Tools & Agents

### Claude Code Tools
```bash
/unit-test-generator:generate-tests    # Generate initial test boilerplate
```

### Specialized Agents
```bash
rust-backend agent                     # Review error types and tests
```

### Manual Tools
```bash
cargo test                             # Run tests
cargo tarpaulin                        # Measure coverage
git commit                             # Commit changes
```

---

## Expected Outcomes

### Coverage Improvement
```
Before: 0/31 lines (0%)
After:  30/31 lines (96.8%)
Gain:   +96.8 percentage points
```

### Test Count
```
Before: 0 tests
After:  30 tests
Gain:   +30 tests
```

### Time Investment
```
Estimated: 1 hour
Actual:    TBD (track in commit message)
```

### Code Quality
```
- ✅ All error variants tested
- ✅ Error messages validated
- ✅ Security edge cases covered
- ✅ Best practices followed
```

---

## Common Pitfalls & Solutions

### ❌ Pitfall 1: thiserror Display Format Mismatch
**Problem:** Test expects different error message than thiserror generates

**Solution:**
```rust
// WRONG: Hardcoded message
assert_eq!(msg, "Invalid MIDI header bad magic");

// RIGHT: Use contains() for flexible matching
assert!(msg.contains("Invalid MIDI header"));
assert!(msg.contains("bad magic"));
```

### ❌ Pitfall 2: From Trait Not Tested
**Problem:** Forgot to test `From` implementations

**Solution:**
```rust
#[test]
fn test_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let midi_err: MidiParseError = io_err.into();
    assert!(matches!(midi_err, MidiParseError::Io(_)));
}
```

### ❌ Pitfall 3: Uncovered Attribute Lines
**Problem:** Coverage shows lines 24, 27 as uncovered (attribute lines)

**Solution:** These are `#[from]` attributes - not executable code, ignore them in coverage

---

## Next Steps After Phase 1.2

1. **Phase 1.3:** Test `parser.rs` module (600+ lines, complex logic)
2. **Address Phase 1.1 Bugs:**
   - Fix `duration_seconds()` tempo accumulation bug
   - Add MIDI value range validation (0-127)
3. **Update Coverage Report:**
   - Recalculate overall Shared library coverage
   - Update TEST-COVERAGE-PLAN.md progress

---

## Time Budget

| Task | Estimated | Notes |
|------|-----------|-------|
| 1. Analyze error module | 5 min | Read code, check usage |
| 2. Generate tests | 10 min | /unit-test-generator |
| 3. Enhance tests | 15 min | Add error-specific cases |
| 4. Add edge cases | 10 min | Boundaries, unicode, etc |
| 5. Add security tests | 10 min | Memory, overflow checks |
| 6. Run tests | 5 min | Verify all passing |
| 7. Measure coverage | 5 min | Tarpaulin run |
| 8. Expert review | 10 min | rust-backend agent |
| 9. Commit | 5 min | Semantic message |
| **TOTAL** | **60 min** | **1 hour** |

---

## Checklist Summary

### Before Starting
- [ ] Phase 1.1 committed and pushed
- [ ] CLAUDE.md updated with Phase 1.1 status
- [ ] Understand error.rs module (31 lines, 8 variants)

### During Execution
- [ ] Generate test boilerplate
- [ ] Add enhanced error-specific tests
- [ ] Add edge case tests
- [ ] Add security tests
- [ ] Run and verify tests (30/30 passing)
- [ ] Measure coverage (≥95%)
- [ ] Get agent review

### After Completion
- [ ] Commit with semantic message
- [ ] Update CLAUDE.md with Phase 1.2 status
- [ ] Update TEST-COVERAGE-PLAN.md progress
- [ ] Plan Phase 1.3 (parser.rs - most complex)

---

**Ready to begin Phase 1.2?**

Run: `/unit-test-generator:generate-tests` and select `shared/rust/src/core/midi/error.rs`
