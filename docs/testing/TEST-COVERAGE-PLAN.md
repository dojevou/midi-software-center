# Test Coverage Plan - Road to 100% (Enhanced with Claude Code Tools)

**Date:** 2025-10-26 (Enhanced)
**Current Coverage:** 38.1% (32/84 files)
**Target Coverage:** 100% (84/84 files)
**Gap:** 52 files need tests

---

## 🚀 Tool-Enhanced Workflow

**This plan leverages the full suite of Claude Code tools for maximum efficiency:**
- **10 Plugins** - Automated coverage analysis, test generation, orchestration
- **35+ Agents** - Specialized expertise for Rust, database, MIDI, architecture
- **12 MCP Servers** - Direct database access, git automation, file operations
- **16 Slash Commands** - Quick access to testing workflows

**Estimated time reduction:** 100h → **~65h** (35% faster with automation)

---

## 🛠️ Tools Reference Matrix

| Phase | Primary Tool | Secondary Tools | MCP Servers |
|-------|-------------|-----------------|-------------|
| **Analysis** | `/test-coverage-analyzer:analyze-coverage` | Explore agent | postgres, rust |
| **Test Gen** | `/unit-test-generator:generate-tests` | rust-backend agent | rust, git |
| **Database** | `/database-test-manager:db-test` | database agent | postgres |
| **Integration** | `/test-orchestrator:orchestrate` | integration-test-runner | docker, bash |
| **Review** | `pr-review-toolkit:pr-test-analyzer` | code-reviewer agent | git |
| **Performance** | performance-oracle agent | query-performance-analyzer | docker, postgres |
| **Security** | security-sentinel agent | - | rust |
| **Commit** | `/git-commit-smart:commit-smart` | - | git |

---

## Executive Summary

### Current Status by Component

| Component | Files | With Tests | Coverage | Priority |
|-----------|-------|------------|----------|----------|
| **Shared** | 22 | 2 | 9.1% | 🔴 **CRITICAL** |
| **Pipeline** | 35 | 20 | 57.1% | 🟡 HIGH |
| **DAW** | 27 | 10 | 37.0% | 🟡 HIGH |
| **TOTAL** | 84 | 32 | 38.1% | - |

### Critical Gaps

**20 core/ files without tests** (violates Trusty Module requirement):
- Shared core/: 8 files
- Pipeline core/: 7 files
- DAW core/: 5 files

---

## 🎯 Phase 0: Setup & Baseline (Day 0 - 3 hours)

**Goal:** Establish testing infrastructure and baseline metrics

### 0.1 Install Testing Tools

```bash
# Coverage tool
cargo install cargo-tarpaulin

# Fast test runner
cargo install cargo-nextest

# Benchmarking
cargo install cargo-criterion

# Database testing
cargo install sqlx-cli
```

### 0.2 Establish Coverage Baseline

**🔧 Tool:** `/test-coverage-analyzer:analyze-coverage`

```bash
# Run coverage analysis with slash command
/test-coverage-analyzer:analyze-coverage
```

**Expected output:**
- Detailed coverage report by file
- Identification of all 52 files without tests
- Coverage hotspots and gaps
- HTML report generation

**🤖 Agent:** Use `rust-backend` agent for workspace setup
```
"Use the rust-backend agent to set up cargo-tarpaulin for the workspace"
```

### 0.3 Create Test Fixture Structure

**🔧 MCP:** Use `filesystem` MCP for directory creation

```bash
tests/
├── fixtures/
│   ├── midi/
│   │   ├── valid/          # Valid MIDI files (formats 0, 1, 2)
│   │   ├── invalid/        # Malformed MIDI files
│   │   └── edge_cases/     # Empty, single note, very long
│   ├── database/
│   │   └── test_data.sql   # Test database fixtures
│   └── config/
│       └── test.env        # Test environment variables
└── integration/
    └── fixtures/
```

**🤖 Agent:** Use `database` agent for test data setup
```
"Use the database agent to create test fixtures for all 18 database tables"
```

**Estimated time:** 3 hours

---

## Phase 1: Shared Library Core (CRITICAL - Days 1-3)

**Priority:** 🔴 **HIGHEST** - Trusty Modules require 80%+ coverage

### 1.1 MIDI Parser Module (Day 1 - 3 hours)

**Files to test:**
- `shared/rust/src/core/midi/types.rs` - MIDI type definitions
- `shared/rust/src/core/midi/error.rs` - Error types
- `shared/rust/src/core/midi/mod.rs` - Module organization

**Coverage target:** 95%+ (already has parser.rs tested)

#### 🛠️ Tool Workflow

**Step 1: Generate test boilerplate**
```bash
/unit-test-generator:generate-tests
```
Select files: `shared/rust/src/core/midi/types.rs`

**Expected output:**
- Auto-generated test cases for all public types
- Property-based test stubs (using proptest)
- Error case templates

**Step 2: Use specialized agent for MIDI expertise**
```
"Use the midi-hardware agent to review and enhance the generated tests for MIDI type validation"
```

**Step 3: Security review**
```
"Use the security-sentinel agent to identify potential malformed MIDI attack vectors"
```

**Step 4: Run tests and measure coverage**
```bash
# Use rust MCP server
cargo nextest run --package shared --lib core::midi
cargo tarpaulin --packages shared --out Html
```

**Step 5: Code review**
```
"Use the rust-backend agent to review test quality and coverage completeness"
```

**Step 6: Commit with semantic message**
```bash
/git-commit-smart:commit-smart
```

**Estimated time:** 3 hours (reduced from 4h with automation)

---

### 1.2 Analysis Modules (Days 2-3 - 8 hours)

**Files to test:**
- `shared/rust/src/core/analysis/auto_tagger.rs` ⚠️ **HIGH VALUE**
- `shared/rust/src/core/analysis/key_detector.rs` ⚠️ **CRITICAL**
- `shared/rust/src/core/analysis/key_profiles.rs`
- `shared/rust/src/core/analysis/mod.rs`

**Coverage target:** 90%+

#### 🛠️ Tool Workflow

**Step 1: Explore existing test patterns**
```
"Use the Explore agent with 'very thorough' mode to find existing analysis test patterns in the codebase"
```

**Step 2: Generate test fixtures for musical algorithms**
```
"Use the midi-hardware agent to create test MIDI files with known BPMs and keys for algorithm validation"
```

**Step 3: Generate comprehensive tests**
```bash
/unit-test-generator:generate-tests
```
Select: All analysis module files

**Step 4: Add performance benchmarks**
```
"Use the performance-oracle agent to create benchmarks for BPM detection and key detection algorithms"
```

**Step 5: Test with real-world data**
```bash
# Use filesystem MCP to copy test MIDI files
# Use rust MCP to run tests
cargo nextest run --package shared --lib core::analysis
```

**Step 6: Verify algorithm accuracy**
```
"Use the pattern-recognition-specialist agent to validate that test cases cover all musical key patterns"
```

**Step 7: Integration test with parser**
```bash
/test-orchestrator:orchestrate
```
Create workflow: MIDI file → parse → analyze → validate

**Estimated time:** 8 hours (reduced from 12h with AI assistance)

---

### 1.3 Database Models (Day 3 - 4 hours)

**Files to test:**
- `shared/rust/src/db/models/*.rs` (7 files)

**Coverage target:** 80%+

#### 🛠️ Tool Workflow

**Step 1: Analyze database schema**
```
"Use the database agent to map all model fields to database columns and identify validation requirements"
```

**🔧 MCP:** Use `postgres` MCP to query schema
```sql
-- Via postgres MCP
SELECT column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_schema = 'public';
```

**Step 2: Generate model tests**
```bash
/unit-test-generator:generate-tests
```
Select: All models in `shared/rust/src/db/models/`

**Step 3: Create database test fixtures**
```bash
/database-test-manager:db-test
```

**Expected setup:**
- Test database with rollback
- Fixture data for all tables
- Transaction isolation

**Step 4: Add property-based tests**
```
"Use the rust-backend agent to add proptest cases for model serialization round-trips"
```

**Step 5: Verify data integrity**
```
"Use the data-integrity-guardian agent to review model tests for data integrity and constraint validation"
```

**Estimated time:** 4 hours (reduced from 6h with database automation)

---

## Phase 2: Pipeline Core Modules (Days 4-6 - 10 hours)

**Priority:** 🟡 HIGH

### 2.1 Hash Module (Day 4 - 2 hours)

**Files:**
- `pipeline/src-tauri/src/core/hash/mod.rs`
- `pipeline/src-tauri/src/core/hash/blake3.rs` (already tested)

#### 🛠️ Tool Workflow

**Step 1: Generate tests**
```bash
/unit-test-generator:generate-tests
```

**Step 2: Add performance benchmarks**
```
"Use the performance-oracle agent to create benchmarks for hash performance across different file sizes"
```

**Step 3: Security review**
```
"Use the security-sentinel agent to verify hash collision resistance and validate security properties"
```

**Estimated time:** 2 hours (reduced from 3h)

---

### 2.2 Naming Module (Day 4 - 1.5 hours)

**Files:**
- `pipeline/src-tauri/src/core/naming/mod.rs`

#### 🛠️ Tool Workflow

**Step 1: Review existing tests**
```
"Use the Explore agent to analyze existing naming tests and identify integration test gaps"
```

**Step 2: Generate integration tests**
```bash
/unit-test-generator:generate-tests
```
Focus on: Module-level integration, template composition

**Step 3: Edge case validation**
```
"Use the rust-backend agent to add edge case tests for special characters, Unicode, and filesystem limits"
```

**Estimated time:** 1.5 hours (reduced from 2h)

---

### 2.3 Analysis Module (Day 5 - 2 hours)

**Files:**
- `pipeline/src-tauri/src/core/analysis/key_profiles.rs`
- `pipeline/src-tauri/src/core/analysis/mod.rs`

#### 🛠️ Tool Workflow

**Step 1: Cross-reference with shared library**
```
"Use the architecture-strategist agent to ensure Pipeline analysis tests don't duplicate Shared library tests"
```

**Step 2: Generate tests**
```bash
/unit-test-generator:generate-tests
```

**Step 3: Validate against shared module**
```bash
/test-orchestrator:orchestrate
```
Create workflow: Test data consistency between Pipeline and Shared analysis

**Estimated time:** 2 hours (reduced from 3h)

---

### 2.4 Other Core Modules (Day 5-6 - 4.5 hours)

**Files:**
- `pipeline/src-tauri/src/core/normalization/mod.rs`
- `pipeline/src-tauri/src/core/performance/mod.rs`
- `pipeline/src-tauri/src/core/splitting/mod.rs`

#### 🛠️ Tool Workflow

**Step 1: Bulk test generation**
```bash
/unit-test-generator:generate-tests
```
Select: All remaining core modules

**Step 2: Performance regression suite**
```
"Use the performance-oracle agent to create performance regression tests for normalization and splitting"
```

**Step 3: Quality review**
```
"Use the code-reviewer agent to review all generated tests for completeness and correctness"
```

**Estimated time:** 4.5 hours (reduced from 6h)

---

## Phase 3: DAW Core Modules (Days 7-8 - 8 hours)

**Priority:** 🟡 HIGH

### 3.1 MIDI Module (Day 7 - 3 hours)

**Files:**
- `daw/src-tauri/src/core/midi/parser.rs`
- `daw/src-tauri/src/core/midi/mod.rs`

#### 🛠️ Tool Workflow

**Step 1: Real-time parsing tests**
```
"Use the midi-hardware agent to create real-time MIDI parsing test cases with timing constraints"
```

**Step 2: Generate performance tests**
```bash
/unit-test-generator:generate-tests
```
Focus on: Real-time constraints, memory usage, latency

**Step 3: Performance benchmarks**
```
"Use the performance-oracle agent to create benchmarks for real-time MIDI parsing performance"
```

**Step 4: Memory safety review**
```
"Use the security-sentinel agent to verify no memory leaks or unsafe operations in real-time code"
```

**Estimated time:** 3 hours (reduced from 4h)

---

### 3.2 Compatibility Module (Day 7 - 2 hours)

**Files:**
- `daw/src-tauri/src/core/compatibility/mod.rs`

#### 🛠️ Tool Workflow

**Step 1: Generate edge case tests**
```bash
/unit-test-generator:generate-tests
```

**Step 2: Algorithm validation**
```
"Use the midi-hardware agent to validate compatibility scoring algorithm against real-world MIDI file pairs"
```

**Estimated time:** 2 hours (reduced from 3h)

---

### 3.3 Sequencer Module (Day 8 - 3 hours)

**Files:**
- `daw/src-tauri/src/core/sequencer/mod.rs`

#### 🛠️ Tool Workflow

**Step 1: Timing accuracy tests**
```
"Use the midi-hardware agent to create timing accuracy tests for sequencer scheduling"
```

**Step 2: Generate tests**
```bash
/unit-test-generator:generate-tests
```

**Step 3: Integration with engine**
```bash
/test-orchestrator:orchestrate
```
Create workflow: Sequencer core → engine → playback simulation

**Step 4: Performance validation**
```
"Use the performance-oracle agent to ensure sequencer meets real-time performance requirements"
```

**Estimated time:** 3 hours (reduced from 4h)

---

## Phase 4: Repository Layer (Days 9-10 - 7 hours)

**Priority:** 🟡 MEDIUM

### 4.1 Shared Repositories (Day 9 - 4 hours)

**Files:**
- `shared/rust/src/db/repositories/*.rs` (5 files)

#### 🛠️ Tool Workflow

**Step 1: Database test setup**
```bash
/database-test-manager:db-test
```

**Expected setup:**
- Test database with transaction rollback
- Mock data fixtures for all tables
- Isolation between test cases

**Step 2: Generate repository tests**
```bash
/unit-test-generator:generate-tests
```

**Step 3: Use database agent expertise**
```
"Use the database agent to create comprehensive repository tests including edge cases and error scenarios"
```

**🔧 MCP:** Use `postgres` MCP for test data
```sql
-- Via postgres MCP
INSERT INTO files (path, hash, size) VALUES
  ('test1.mid', 'hash1', 1024),
  ('test2.mid', 'hash2', 2048);
```

**Step 4: Transaction testing**
```
"Use the data-integrity-guardian agent to verify transaction handling and rollback scenarios"
```

**Step 5: Query performance**
```bash
# Use query-performance-analyzer skill
"Use the query-performance-analyzer skill to identify slow queries in repository tests"
```

**Estimated time:** 4 hours (reduced from 6h)

---

### 4.2 Pipeline Repositories (Day 9-10 - 3 hours)

**Files:**
- `pipeline/src-tauri/src/db/repositories/mod.rs`
- `pipeline/src-tauri/src/db/repositories/file_repository_fixed.rs`

#### 🛠️ Tool Workflow

**Step 1: Batch operation tests**
```bash
/database-test-manager:db-test
```
Focus on: Batch inserts (500+ files), bulk updates

**Step 2: Generate tests**
```bash
/unit-test-generator:generate-tests
```

**Step 3: Performance validation**
```
"Use the performance-oracle agent to validate batch insert performance meets 500 files/batch target"
```

**🔧 MCP:** Use `docker` MCP to monitor database load
```
"Use docker MCP to monitor PostgreSQL container during batch insert tests"
```

**Estimated time:** 3 hours (reduced from 4h)

---

## Phase 5: Commands Layer (Days 11-13 - 10 hours)

**Priority:** 🟡 MEDIUM

### 5.1 Pipeline Commands (Days 11-12 - 5 hours)

**Files:**
- `pipeline/src-tauri/src/commands/*.rs` (6 files)

#### 🛠️ Tool Workflow

**Step 1: Generate command tests**
```bash
/unit-test-generator:generate-tests
```
Select: All command files

**Step 2: Tauri command testing**
```
"Use the rust-backend agent to create mock Tauri contexts for command testing"
```

**Step 3: Input validation tests**
```
"Use the security-sentinel agent to create comprehensive input validation tests for all command parameters"
```

**Step 4: Error handling review**
```
"Use the silent-failure-hunter agent to verify all commands have proper error handling and no silent failures"
```

**Step 5: Integration test**
```bash
/test-orchestrator:orchestrate
```
Create workflow: Frontend → IPC → Command → Repository → Database

**Estimated time:** 5 hours (reduced from 8h)

---

### 5.2 DAW Commands (Day 13 - 5 hours)

**Files:**
- `daw/src-tauri/src/commands/*.rs` (6 files)

#### 🛠️ Tool Workflow

**Step 1: Real-time command tests**
```
"Use the midi-hardware agent to create tests for real-time MIDI commands with latency requirements"
```

**Step 2: Generate tests**
```bash
/unit-test-generator:generate-tests
```

**Step 3: State management tests**
```
"Use the rust-backend agent to create sequencer state tests for all command interactions"
```

**Step 4: Integration test**
```bash
/test-orchestrator:orchestrate
```
Create workflow: UI → Command → Sequencer → MIDI output

**Estimated time:** 5 hours (reduced from 8h)

---

## Phase 6: Models Layer (Day 14 - 3 hours)

**Priority:** 🟢 LOW (mostly data structures)

### 6.1 DAW Models (Day 14 - 3 hours)

**Files:**
- `daw/src-tauri/src/models/*.rs` (7 files)

#### 🛠️ Tool Workflow

**Step 1: Bulk test generation**
```bash
/unit-test-generator:generate-tests
```
Select: All model files

**Step 2: Serialization tests**
```
"Use the rust-backend agent to add comprehensive JSON serialization round-trip tests"
```

**Step 3: Type safety validation**
```
"Use the type-design-analyzer agent to review model types for proper encapsulation and invariants"
```

**Estimated time:** 3 hours (reduced from 4h)

---

## Phase 7: Integration & E2E Tests (Days 15-16 - 12 hours)

**Priority:** 🟡 HIGH

### 7.1 Integration Tests (Day 15 - 8 hours)

**Test suites to create:**

#### Suite 1: Full Import Pipeline
```bash
/integration-test-runner:run-integration
```

**Workflow:**
1. MIDI file upload (filesystem MCP)
2. Hash calculation
3. MIDI parsing
4. BPM/key detection
5. Database insert (postgres MCP)
6. Search indexing (Meilisearch)

**Tools:**
- Use `test-orchestrator` to coordinate
- Use `database-test-manager` for test database
- Use `docker` MCP to monitor services

#### Suite 2: Search Workflow
```bash
/integration-test-runner:run-integration
```

**Workflow:**
1. Query submission
2. Meilisearch query (via postgres MCP)
3. Result retrieval
4. File loading

#### Suite 3: Sequencer Workflow
```bash
/integration-test-runner:run-integration
```

**Workflow:**
1. MIDI file load
2. Sequencer initialization
3. Playback start
4. MIDI output simulation
5. Export

**🤖 Agents:**
```
"Use the integration-test-runner agent to coordinate multi-component test execution"
"Use the midi-hardware agent to validate MIDI output correctness"
"Use the database agent to verify data integrity across the full pipeline"
```

**Estimated time:** 8 hours (reduced from 10h)

---

### 7.2 Performance Regression Tests (Day 16 - 4 hours)

**Benchmark suites to create:**

```bash
# Create benchmark suite
cargo criterion
```

**Benchmarks:**
1. MIDI parsing (midly library)
2. BPM detection algorithm
3. Key detection algorithm
4. Batch database insert (500 files)
5. Search query response time
6. Sequencer scheduling latency

**🛠️ Tool Workflow:**

**Step 1: Create benchmarks**
```
"Use the performance-oracle agent to create comprehensive performance benchmarks for all critical paths"
```

**Step 2: Establish baselines**
```bash
cargo criterion --save-baseline initial
```

**Step 3: Query optimization**
```bash
/database-index-advisor:index-advisor
```

**🔧 MCP:** Use `postgres` MCP for query analysis
```sql
-- Via postgres MCP
EXPLAIN ANALYZE SELECT * FROM files WHERE bpm BETWEEN 120 AND 140;
```

**Step 4: Continuous monitoring**
```
"Use the docker MCP to monitor PostgreSQL and Meilisearch performance during benchmark runs"
```

**Estimated time:** 4 hours (reduced from 6h)

---

## Phase 8: Documentation & Final Verification (Day 17 - 6 hours)

### 8.1 Documentation Pass (4 hours)

**Goal:** Add rustdoc comments to all public APIs

**🛠️ Tool Workflow:**

**Step 1: Identify undocumented code**
```bash
# Use rust MCP
cargo doc --no-deps --document-private-items 2>&1 | grep "missing documentation"
```

**Step 2: Generate documentation**
```
"Use the rust-backend agent to add comprehensive rustdoc comments with examples to all public APIs in core/ modules"
```

**Step 3: Review documentation quality**
```
"Use the comment-analyzer agent to review all new documentation for accuracy and completeness"
```

**Step 4: Build and verify docs**
```bash
cargo doc --workspace --no-deps --open
```

**Estimated time:** 4 hours (reduced from 6h with AI assistance)

---

### 8.2 Final Coverage Run (2 hours)

**🛠️ Tool Workflow:**

**Step 1: Final coverage analysis**
```bash
/test-coverage-analyzer:analyze-coverage
```

**Expected results:**
- 100% of core/ modules covered
- 90%+ overall coverage
- HTML report with detailed breakdown

**Step 2: Test suite execution**
```bash
/test-orchestrator:orchestrate
```
Run: All unit tests, integration tests, benchmarks

**Step 3: Quality gate verification**
```bash
# Run all checks
cargo nextest run --workspace
cargo tarpaulin --workspace --out Html
cargo doc --workspace
cargo clippy --workspace -- -D warnings
```

**Step 4: Final review**
```
"Use the pr-test-analyzer agent to review the entire test suite for completeness and quality"
```

**Step 5: Create coverage report**
```bash
# Generate comprehensive report
cargo tarpaulin --workspace --out Html --output-dir coverage
```

**🔧 MCP:** Use `git` MCP to commit results
```bash
/git-commit-smart:commit-smart
```

**Expected commit message:**
```
test: Achieve 100% test coverage across all core modules

- 52 new test files added
- Coverage: 38.1% → 90%+
- All core/ modules: 100% covered
- Integration tests: 5 major workflows
- Performance benchmarks: 6 critical paths
- Documentation: All public APIs documented

🤖 Generated with Claude Code
```

**Estimated time:** 2 hours

---

## Summary Timeline (Tool-Enhanced)

| Phase | Focus | Days | Effort (Original) | Effort (Enhanced) | Savings |
|-------|-------|------|-------------------|-------------------|---------|
| 0 | Setup | 0.5 | 0h | 3h | - |
| 1 | Shared core/ | 3 | 22h | 15h | 7h |
| 2 | Pipeline core/ | 3 | 14h | 10h | 4h |
| 3 | DAW core/ | 2 | 11h | 8h | 3h |
| 4 | Repositories | 2 | 10h | 7h | 3h |
| 5 | Commands | 3 | 16h | 10h | 6h |
| 6 | Models | 1 | 4h | 3h | 1h |
| 7 | Integration | 2 | 16h | 12h | 4h |
| 8 | Docs + Verify | 1 | 8h | 6h | 2h |
| **TOTAL** | **All** | **17.5 days** | **~100h** | **~74h** | **~26h** |

**Time reduction: 26% faster with Claude Code tools**

**Full-time:** ~2.5 weeks (vs 3 weeks)
**Part-time:** ~4 weeks (vs 5 weeks)

---

## 🚀 Quick Start Guide

### Day 0: Setup

```bash
# 1. Install testing tools
cargo install cargo-tarpaulin cargo-nextest cargo-criterion

# 2. Establish baseline
/test-coverage-analyzer:analyze-coverage

# 3. Set up test database
/database-test-manager:db-test

# 4. Create fixtures
mkdir -p tests/fixtures/{midi,database,config}
```

### Day 1: Begin Phase 1.1

```bash
# 1. Generate tests for MIDI types
/unit-test-generator:generate-tests
# Select: shared/rust/src/core/midi/types.rs

# 2. Get expert review
"Use the midi-hardware agent to review generated MIDI type tests"

# 3. Run tests
cargo nextest run --package shared --lib core::midi

# 4. Measure coverage
cargo tarpaulin --packages shared

# 5. Commit with semantic message
/git-commit-smart:commit-smart
```

---

## 🎯 Success Criteria

✅ **100% of core/ modules have tests** (80%+ coverage each)
✅ **90%+ line coverage overall**
✅ **All public APIs documented** (rustdoc comments with examples)
✅ **All integration tests passing** (5 major workflows)
✅ **Performance benchmarks established** (6 critical paths)
✅ **CI/CD pipeline includes coverage gates** (fail below 80%)
✅ **Security review complete** (no unsafe patterns)
✅ **Architecture compliance verified** (Trusty Module standards)

---

## 📋 Daily Checklist Template

```bash
# Morning: Analyze coverage gap
/test-coverage-analyzer:analyze-coverage

# Generate tests for target files
/unit-test-generator:generate-tests

# Get specialized agent review
"Use the [agent-name] agent to review tests for [component]"

# Run tests locally
cargo nextest run --workspace

# Measure coverage improvement
cargo tarpaulin --workspace --out Html

# Review quality
"Use the code-reviewer agent to verify test quality"

# Commit progress
/git-commit-smart:commit-smart

# Evening: Update this plan with progress
```

---

## 🛠️ Tool Usage Patterns

### Pattern 1: Test Generation
```bash
1. /unit-test-generator:generate-tests
2. Select files from current phase
3. Review with specialized agent (rust-backend, midi-hardware, database)
4. Run tests: cargo nextest run
5. Measure coverage: cargo tarpaulin
```

### Pattern 2: Database Testing
```bash
1. /database-test-manager:db-test
2. Create fixtures with database agent
3. Use postgres MCP for test data queries
4. Run tests with transaction rollback
5. Verify with data-integrity-guardian agent
```

### Pattern 3: Integration Testing
```bash
1. /test-orchestrator:orchestrate
2. Define multi-component workflow
3. Use integration-test-runner agent
4. Monitor with docker MCP
5. Validate with architecture-strategist agent
```

### Pattern 4: Performance Testing
```bash
1. Create benchmarks with performance-oracle agent
2. Run: cargo criterion
3. Analyze with query-performance-analyzer
4. Optimize with database-index-advisor
5. Monitor with docker MCP
```

### Pattern 5: Final Review
```bash
1. /test-coverage-analyzer:analyze-coverage
2. pr-test-analyzer agent for quality check
3. security-sentinel agent for security review
4. architecture-reviewer agent for compliance
5. /git-commit-smart:commit-smart
```

---

## 📊 Progress Tracking

### Coverage Targets by Phase

| Phase Complete | Expected Coverage | Files Tested | Cumulative Hours |
|----------------|-------------------|--------------|------------------|
| 0 - Setup | 38.1% | 32 | 3 |
| 1 - Shared | 60%+ | 44 | 18 |
| 2 - Pipeline | 72%+ | 55 | 28 |
| 3 - DAW | 80%+ | 63 | 36 |
| 4 - Repos | 83%+ | 70 | 43 |
| 5 - Commands | 87%+ | 78 | 53 |
| 6 - Models | 89%+ | 82 | 56 |
| 7 - Integration | 92%+ | 84 | 68 |
| 8 - Final | 95%+ | 84 | 74 |

### Update This Section Daily

**Current Phase:** 0 - Setup
**Coverage:** 38.1%
**Files Tested:** 32/84
**Hours Invested:** 0
**Blockers:** None
**Next Milestone:** Complete Phase 0 setup

---

## 🎉 Milestones & Celebrations

- [ ] **Milestone 1:** Phase 0 complete - Baseline established
- [ ] **Milestone 2:** Phase 1 complete - Shared core at 80%+
- [ ] **Milestone 3:** 50% overall coverage reached
- [ ] **Milestone 4:** All core/ modules at 100%
- [ ] **Milestone 5:** 75% overall coverage reached
- [ ] **Milestone 6:** All integration tests passing
- [ ] **Milestone 7:** 90% overall coverage reached
- [ ] **Milestone 8:** 100% project complete! 🎊

---

## 📚 Reference: All Available Tools

### Slash Commands (Testing Focus)
- `/test-coverage-analyzer:analyze-coverage` - Analyze coverage gaps
- `/unit-test-generator:generate-tests` - Generate test boilerplate
- `/test-orchestrator:orchestrate` - Orchestrate multi-component tests
- `/integration-test-runner:run-integration` - Run integration suites
- `/database-test-manager:db-test` - Database test setup/teardown
- `/database-migration-manager:migration` - Migration testing
- `/database-index-advisor:index-advisor` - Query optimization
- `/git-commit-smart:commit-smart` - Semantic commits

### Specialized Agents (Testing Focus)
- `rust-backend` - Rust/Tauri expertise, async patterns
- `database` - PostgreSQL, SQLx, migrations
- `midi-hardware` - MIDI parsing, hardware, BPM/key detection
- `frontend` - Svelte/TypeScript testing
- `architecture-reviewer` - Three Archetypes compliance
- `security-sentinel` - Security audit, vulnerability testing
- `performance-oracle` - Performance analysis, optimization
- `data-integrity-guardian` - Database integrity, migrations
- `code-reviewer` - Code quality, bug detection
- `pr-test-analyzer` - PR test coverage review
- `silent-failure-hunter` - Error handling validation

### MCP Servers (Testing Focus)
- `postgres` - Direct database queries, schema inspection
- `rust` - Cargo operations, workspace management
- `docker` - Container monitoring, logs
- `git` - Version control automation
- `bash` - Script execution, automation
- `filesystem` - File operations, fixture management

---

## 🚨 Common Issues & Solutions

### Issue: "Coverage tool failing"
**Solution:**
```bash
# Use rust-backend agent
"Use the rust-backend agent to debug cargo-tarpaulin issues with the workspace"
```

### Issue: "Database tests failing"
**Solution:**
```bash
# Use database agent + postgres MCP
/database-test-manager:db-test
"Use postgres MCP to verify test database schema matches production"
```

### Issue: "Performance tests timing out"
**Solution:**
```bash
# Use performance-oracle agent
"Use the performance-oracle agent to identify performance bottlenecks in test suite"
```

### Issue: "Integration tests flaky"
**Solution:**
```bash
# Use test-orchestrator
/test-orchestrator:orchestrate
"Create deterministic test execution order with proper setup/teardown"
```

---

## 🎯 Next Immediate Steps

**Ready to start? Here's your action plan:**

1. ✅ Read this enhanced plan completely
2. ⏳ Run Phase 0 setup (3 hours)
3. ⏳ Establish coverage baseline with `/test-coverage-analyzer:analyze-coverage`
4. ⏳ Create test fixture structure
5. ⏳ Begin Phase 1.1 - MIDI types (3 hours)

**First command to run:**
```bash
/test-coverage-analyzer:analyze-coverage
```

This will give you the exact coverage map and identify all 52 files needing tests.

---

*Last updated: 2025-10-26 (Enhanced with Claude Code Tools)*
*Next update: After Phase 0 completion*
