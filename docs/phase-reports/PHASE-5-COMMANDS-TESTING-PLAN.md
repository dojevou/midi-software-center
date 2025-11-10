# Phase 5: Commands Layer Testing (Comprehensive Plan)

**Objective:** Test all Tauri IPC commands across Pipeline (11 files) and DAW (7 files) applications
**Coverage Target:** 80%+ on all command functions
**Estimated Duration:** 8-12 hours (tool-stacked workflow)
**Expected Outcome:** ~200-250 tests covering all commands

## Phase 5 Structure (6 Subphases)

### Phase 5.0: Commands Architecture Analysis ✅ (THIS TASK)
- **Goal:** Map all commands, identify testing patterns, design test architecture
- **Deliverable:** This comprehensive plan document
- **Key Files:**
  - Pipeline: 11 command files (1,200+ lines)
  - DAW: 7 command files (900+ lines)

---

## Phase 5.1: Pipeline Commands - Discovery & Architecture

**Duration:** 60-90 minutes
**Tools:** Explore agent, repo-research-analyst agent, architecture-reviewer agent
**Slash Commands:** `/feature-dev:feature-dev`

### Subphase 5.1.0: Analyze Pipeline Command Structure
```bash
STEP 1: Use Explore agent (very thorough) to map all Pipeline commands
- Task: "Map all Tauri #[tauri::command] functions in pipeline/src-tauri/src/commands/"
- Output: Complete list of:
  * Function signatures
  * Parameters and return types
  * Error handling patterns
  * Database access patterns
  * File I/O operations
- This informs test architecture decisions
```

**Tool Stack:**
```
Task(subagent_type="Explore", thoroughness="very thorough")
  ↓ discovers command patterns
  ↓ identifies database access
  ↓ finds file operations
  → FEEDS INTO → architecture-reviewer agent
```

### Subphase 5.1.1: Design Test Architecture
```bash
STEP 2: Use architecture-reviewer agent to validate test design
- Task: "Design test architecture for 11 Pipeline command files"
- Considerations:
  * Mock vs real database (recommendation: test DB with fixtures)
  * File system mocking (recommendation: temp directories)
  * Progress callback testing
  * Error scenario coverage
  * Batch operation testing (file import, analyze)
```

**Tool Stack:**
```
architecture-reviewer agent
  ↓ analyzes component boundaries
  ↓ validates separation of concerns
  ↓ recommends fixture patterns
  → FEEDS INTO → repository layer structure
```

### Subphase 5.1.2: Map Command Dependencies
```bash
STEP 3: Identify which commands use which repositories
- Grep pattern: Search for repository calls in command files
- Output format: Command → Repository mapping
  * file_import → file_repository, metadata_repository, tag_repository
  * search → search_repository
  * tags → tag_repository
  * analyze → metadata_repository
  * etc.
```

**Tool Stack:**
```
Grep tool (pattern: "Repository::")
  ↓ finds all repository calls
  ↓ maps command dependencies
  → FEEDS INTO → test generation strategy
```

### Subphase 5.1.3: Create Test File Structure
```bash
STEP 4: Create test module stubs for each Pipeline command
Files to create:
  ✓ pipeline/src-tauri/tests/commands/file_import_test.rs
  ✓ pipeline/src-tauri/tests/commands/archive_import_test.rs
  ✓ pipeline/src-tauri/tests/commands/analyze_test.rs
  ✓ pipeline/src-tauri/tests/commands/search_test.rs
  ✓ pipeline/src-tauri/tests/commands/tags_test.rs
  ✓ pipeline/src-tauri/tests/commands/split_file_test.rs
  ✓ pipeline/src-tauri/tests/commands/files_test.rs
  ✓ pipeline/src-tauri/tests/commands/stats_test.rs
  ✓ pipeline/src-tauri/tests/commands/progress_test.rs
  ✓ pipeline/src-tauri/tests/commands/system_test.rs

(11 test files total)
```

**Tool Stack:**
```
Write tool (bulk operation)
  ↓ creates test module stubs
  ↓ includes setup/teardown boilerplate
  → READY FOR → test generation phase
```

---

## Phase 5.2: Pipeline Commands - Test Suite Generation

**Duration:** 3-4 hours
**Tools:** unit-test-generator plugin, database-test-manager plugin, code-reviewer agent
**Slash Commands:** `/unit-test-generator:generate-tests`, `/database-test-manager:db-test`

### Subphase 5.2.1: Generate Test Cases for Batch Operations
```bash
PRIORITY: file_import, archive_import, analyze (bulk operations)
REASON: These are most complex and most used

STEP 1: Use unit-test-generator for file_import.rs
Task: "Generate comprehensive tests for file_import command"
Expected: ~25-30 tests covering:
  ✓ Single file import (success, duplicate detection, hash validation)
  ✓ Batch import (10, 100, 1000 files)
  ✓ Error scenarios (corrupted files, disk full, DB constraint violations)
  ✓ Progress callbacks (async notification tracking)
  ✓ Metadata extraction (format detection, duration calculation)
  ✓ Concurrent imports (thread safety)
  ✓ Cancellation (graceful abort mid-import)
```

**Tool Stack:**
```
/unit-test-generator:generate-tests
  ↓ analyzes function signatures
  ↓ generates test cases
  ↓ includes property-based tests
  → FEEDS INTO → database-test-manager

database-test-manager (for DB isolation)
  ↓ setup test fixtures
  ↓ run tests with transaction rollback
  ↓ verify constraints
  → PRODUCES → 25-30 passing tests
```

### Subphase 5.2.2: Generate Tests for Archive Operations
```bash
Command: archive_import.rs (ZIP, RAR, 7z extraction)

STEP 2: Use unit-test-generator for archive operations
Expected: ~12-15 tests covering:
  ✓ ZIP extraction (standard, nested, large files)
  ✓ RAR extraction (version 4, 5 compatibility)
  ✓ 7z extraction (solid archives, multi-volume)
  ✓ Error handling (corrupted archives, password-protected)
  ✓ Path traversal prevention (security validation)
  ✓ Symlink handling (symlink bombs prevention)
  ✓ File permission preservation (Unix permissions)
```

**Tool Stack:**
```
/unit-test-generator:generate-tests
  ↓ generates extraction tests
  ↓ includes malformed archive handling
  ↓ security-focused test cases
  → FEEDS INTO → security-sentinel agent

security-sentinel agent
  ↓ reviews for path traversal
  ↓ validates symlink handling
  ↓ checks extraction safety
  → APPROVES → archive safety baseline
```

### Subphase 5.2.3: Generate Tests for Query Commands
```bash
Commands: search.rs, tags.rs, stats.rs, analysis.rs

STEP 3: Use unit-test-generator for query commands
Expected: ~40-50 tests total covering:
  ✓ search.rs (12 tests):
    - Full-text search with various filters
    - Pagination and ordering
    - Tag-based filtering
  ✓ tags.rs (10 tests):
    - Tag creation, deletion, updates
    - Bulk tag operations
    - Tag hierarchy/relationships
  ✓ stats.rs (8 tests):
    - Aggregate statistics calculations
    - Time-series data (by date, manufacturer)
    - Distribution analysis
  ✓ files.rs (10 tests):
    - File listing and filtering
    - Metadata retrieval
    - Batch operations
  ✓ system.rs (6 tests):
    - System status checks
    - Configuration operations
    - Health checks
```

**Tool Stack:**
```
/unit-test-generator:generate-tests (run 5x for different commands)
  ↓ generates query-specific tests
  ↓ includes edge cases (empty results, large datasets)
  ↓ null handling, sorting, pagination
  → FEEDS INTO → query-performance-analyzer

query-performance-analyzer skill
  ↓ analyzes query patterns
  ↓ suggests index optimization
  ↓ validates execution plans
  → PRODUCES → performance baseline
```

### Subphase 5.2.4: Generate Tests for Utility Commands
```bash
Commands: progress.rs, split_file.rs, analyze.rs

STEP 4: Use unit-test-generator for utilities
Expected: ~20-25 tests covering:
  ✓ progress.rs (8 tests):
    - Progress tracking state machine
    - Concurrent progress updates
    - Callback execution
  ✓ split_file.rs (10 tests):
    - Single-track to multi-track splitting
    - Track extraction and validation
    - Error recovery (partial splits)
  ✓ analyze.rs (5-7 tests):
    - MIDI analysis (BPM, key, instruments)
    - Metadata generation
    - Concurrent analysis
```

**Tool Stack:**
```
/unit-test-generator:generate-tests
  ↓ generates utility tests
  ↓ includes state machine validation
  ↓ concurrency scenarios
  → FEEDS INTO → code-reviewer

code-reviewer agent
  ↓ validates test quality
  ↓ checks coverage completeness
  ↓ identifies missing scenarios
  → PRODUCES → high-quality test suite
```

### Subphase 5.2.5: Code Review & Quality Pass
```bash
STEP 5: Use code-reviewer agent on all Pipeline command tests
Task: "Review all 11 Pipeline command test files for quality"
Focus:
  ✓ Test isolation (no cross-test dependencies)
  ✓ Error scenario coverage (all Result<T> variants)
  ✓ Database transaction safety
  ✓ Resource cleanup (files, connections)
  ✓ Assertion quality (not just "assert!(result.is_ok())")
```

**Tool Stack:**
```
/code-reviewer:code-review
  ↓ runs on all 11 test files
  ↓ checks for anti-patterns
  ↓ validates assertion quality
  → PRODUCES → high-quality baseline

Then loop:
If issues found:
  ↓ Use code-simplifier agent
  ↓ Fix identified issues
  ↓ Re-run review
Until: All tests pass quality gate
```

---

## Phase 5.3: DAW Commands - Discovery & Architecture

**Duration:** 45-60 minutes
**Tools:** Explore agent, architecture-reviewer agent, midi-hardware agent
**Slash Commands:** `/feature-dev:feature-dev`

### Subphase 5.3.0: Analyze DAW Command Structure
```bash
STEP 1: Use Explore agent (very thorough) for DAW commands
Task: "Map all Tauri #[tauri::command] functions in daw/src-tauri/src/commands/"
Focus Areas:
  ✓ Real-time sequencer commands
  ✓ MIDI hardware integration (midir)
  ✓ Playback control
  ✓ Project management
  ✓ Audio I/O
  ✓ Analysis/export operations

Output: Complete command inventory with:
  * Function signatures
  * State management patterns
  * MIDI hardware dependencies
  * Playback timing requirements
  * File I/O operations
```

**Tool Stack:**
```
Task(subagent_type="Explore", thoroughness="very thorough")
  ↓ discovers DAW command patterns
  ↓ identifies real-time constraints
  ↓ finds state management approach
  → FEEDS INTO → midi-hardware agent
```

### Subphase 5.3.1: Design DAW Test Architecture
```bash
STEP 2: Use midi-hardware agent to design MIDI testing strategy
Task: "Design test architecture for DAW commands with MIDI hardware simulation"
Considerations:
  ✓ MIDI hardware mocking (no real device needed)
  ✓ Real-time timing requirements (sub-ms accuracy)
  ✓ Playback state machine (play, pause, stop, seek)
  ✓ Event sequencing and scheduling
  ✓ MIDI port enumeration
```

**Tool Stack:**
```
midi-hardware agent
  ↓ understands MIDI timing requirements
  ↓ knows hardware simulation patterns
  ↓ reviews MIDI command implementations
  → FEEDS INTO → architecture-reviewer

architecture-reviewer agent
  ↓ validates component isolation
  ↓ ensures testability without hardware
  ↓ recommends fixture design
  → PRODUCES → DAW test architecture
```

### Subphase 5.3.2: Create DAW Test File Structure
```bash
STEP 3: Create test module stubs for each DAW command
Files to create:
  ✓ daw/src-tauri/tests/commands/sequencer_test.rs
  ✓ daw/src-tauri/tests/commands/midi_test.rs
  ✓ daw/src-tauri/tests/commands/project_test.rs
  ✓ daw/src-tauri/tests/commands/playback_test.rs
  ✓ daw/src-tauri/tests/commands/export_test.rs
  ✓ daw/src-tauri/tests/commands/analysis_test.rs
  ✓ daw/src-tauri/tests/commands/search_test.rs

(7 test files total)
```

**Tool Stack:**
```
Write tool
  ↓ creates test stubs with real-time patterns
  ↓ includes MIDI mock infrastructure
  ↓ adds timing assertion helpers
  → READY FOR → test generation
```

---

## Phase 5.4: DAW Commands - Test Suite Generation

**Duration:** 3-4 hours
**Tools:** unit-test-generator plugin, midi-hardware agent, performance-oracle agent
**Slash Commands:** `/unit-test-generator:generate-tests`, `/database-index-advisor:index-advisor`

### Subphase 5.4.1: Generate Tests for Sequencer Commands
```bash
PRIORITY: sequencer.rs (core DAW functionality)
REASON: Most complex, real-time constraints

STEP 1: Use unit-test-generator for sequencer
Task: "Generate comprehensive tests for sequencer command"
Expected: ~35-40 tests covering:
  ✓ Sequencer state (play, pause, stop, seek)
  ✓ Track management (add, remove, reorder)
  ✓ Event scheduling (note on/off, CC, pitch bend)
  ✓ Timing accuracy (tempo changes, time signature)
  ✓ MIDI output (event transmission order)
  ✓ Concurrent operations (multiple commands while playing)
  ✓ Edge cases (seek beyond bounds, 0 tempo, invalid track)
```

**Tool Stack:**
```
/unit-test-generator:generate-tests
  ↓ generates sequencer state machine tests
  ↓ includes timing validation
  ↓ covers edge cases
  → FEEDS INTO → midi-hardware agent

midi-hardware agent
  ↓ validates MIDI event output correctness
  ↓ checks timing accuracy
  ↓ reviews playback logic
  → FEEDS INTO → performance-oracle

performance-oracle agent
  ↓ analyzes real-time performance
  ↓ checks latency requirements (<5ms)
  ↓ validates CPU efficiency
  → PRODUCES → 35-40 production-ready tests
```

### Subphase 5.4.2: Generate Tests for MIDI Hardware Commands
```bash
Commands: midi.rs (hardware I/O)

STEP 2: Use unit-test-generator for MIDI operations
Expected: ~20-25 tests covering:
  ✓ MIDI port enumeration (list available ports)
  ✓ MIDI input (hardware → sequencer)
  ✓ MIDI output (sequencer → hardware)
  ✓ Hardware disconnection (graceful error handling)
  ✓ Port permission errors (access denied)
  ✓ Concurrent I/O (simultaneous input/output)
  ✓ Data buffering (overflow prevention)
  ✓ Device reconnection (hot-plug handling)
```

**Tool Stack:**
```
/unit-test-generator:generate-tests
  ↓ generates MIDI I/O tests
  ↓ includes mock device implementations
  ↓ covers error scenarios
  → FEEDS INTO → security-sentinel

security-sentinel agent
  ↓ validates no buffer overflows
  ↓ checks error handling safety
  ↓ reviews device access safety
  → PRODUCES → 20-25 hardened tests
```

### Subphase 5.4.3: Generate Tests for Project & Export Commands
```bash
Commands: project.rs (project management), export.rs (MIDI export)

STEP 3: Use unit-test-generator for project operations
Expected: ~25-30 tests covering:
  ✓ project.rs (15 tests):
    - Create/open/save projects
    - Undo/redo state management
    - File versioning
    - Autosave functionality
  ✓ export.rs (10-15 tests):
    - Export to MIDI file
    - Export to audio (if supported)
    - Format validation
    - Metadata preservation
    - Batch export operations
```

**Tool Stack:**
```
/unit-test-generator:generate-tests (2x)
  ↓ generates project management tests
  ↓ includes file I/O validation
  ↓ covers undo/redo state machine
  → FEEDS INTO → database-test-manager

/database-test-manager:db-test
  ↓ if projects use database
  ↓ validates file references
  ↓ tests transaction safety
  → PRODUCES → 25-30 tests
```

### Subphase 5.4.4: Generate Tests for Analysis & Search
```bash
Commands: analysis.rs, search.rs

STEP 4: Use unit-test-generator for analysis/search
Expected: ~15-20 tests covering:
  ✓ analysis.rs (10 tests):
    - MIDI analysis (BPM, key, instruments)
    - Metadata generation
    - Analysis progress tracking
  ✓ search.rs (5-10 tests):
    - Search loaded projects
    - Filter by metadata
    - Results pagination
```

**Tool Stack:**
```
/unit-test-generator:generate-tests
  ↓ generates analysis tests
  ↓ includes computation validation
  ↓ covers edge cases
  → FEEDS INTO → code-reviewer

code-reviewer agent
  ↓ validates test quality
  ↓ checks assertion strength
  ↓ identifies missing scenarios
  → PRODUCES → 15-20 tests
```

### Subphase 5.4.5: Code Review & Quality Pass
```bash
STEP 5: Use code-reviewer agent on all DAW command tests
Task: "Review all 7 DAW command test files for quality"
Focus:
  ✓ Real-time constraints respected
  ✓ MIDI mock implementation correctness
  ✓ State machine validation
  ✓ Error handling completeness
  ✓ Hardware simulation realism
```

**Tool Stack:**
```
/code-reviewer:code-review
  ↓ runs on all 7 test files
  ↓ checks real-time safety
  ↓ validates MIDI compliance
  → PRODUCES → production-quality tests

If issues found:
  ↓ Use performance-oracle for timing issues
  ↓ Use midi-hardware agent for MIDI issues
  ↓ Fix and re-review
Until: All tests pass quality gate
```

---

## Phase 5.5: Integration & End-to-End Tests

**Duration:** 2-3 hours
**Tools:** integration-test-runner plugin, test-orchestrator plugin, architecture-strategist agent
**Slash Commands:** `/integration-test-runner:run-integration`, `/test-orchestrator:orchestrate`

### Subphase 5.5.1: Design Integration Test Scenarios
```bash
STEP 1: Use architecture-strategist to design integration points
Task: "Map integration points between Pipeline and DAW"
Key Scenarios:
  1. Import workflow: Pipeline (import) → Database → DAW (load)
  2. Search workflow: Pipeline (search) → Database → DAW (filter)
  3. Analysis workflow: Pipeline (analyze) → Metadata → DAW (display)
  4. Export workflow: DAW (create) → Pipeline (archive/export)
```

**Tool Stack:**
```
architecture-strategist agent
  ↓ analyzes component boundaries
  ↓ identifies integration contracts
  ↓ validates data flow
  → FEEDS INTO → test-orchestrator
```

### Subphase 5.5.2: Create Integration Test Suite
```bash
STEP 2: Use integration-test-runner to generate integration tests
Files to create:
  ✓ tests/integration/import_to_daw_test.rs
  ✓ tests/integration/search_workflow_test.rs
  ✓ tests/integration/analysis_pipeline_test.rs
  ✓ tests/integration/export_roundtrip_test.rs

Expected: ~20-25 integration tests
```

**Tool Stack:**
```
/integration-test-runner:run-integration
  ↓ creates full workflow tests
  ↓ includes multi-app scenarios
  ↓ validates data consistency
  → PRODUCES → integration test suite

/test-orchestrator:orchestrate
  ↓ coordinates test execution
  ↓ manages setup/teardown
  ↓ validates test dependencies
  → RUNS → all integration tests
```

### Subphase 5.5.3: End-to-End Scenario Testing
```bash
STEP 3: Create real-world scenario tests
Scenarios:
  1. User imports 100 MIDI files → Analyzes → Searches → Exports selection
  2. User creates sequencer track → Adds notes → Exports → Re-imports → Verifies
  3. Concurrent imports + searches + playback (stress testing)
  4. Recovery from interruptions (mid-import cancellation, etc.)
```

**Tool Stack:**
```
/test-orchestrator:orchestrate
  ↓ manages complex multi-step scenarios
  ↓ validates state consistency
  ↓ simulates user workflows
  → PRODUCES → scenario validation

If issues found:
  ↓ Use code-reviewer for test quality
  ↓ Use performance-oracle for timing
  ↓ Fix and re-run orchestration
Until: All scenarios pass
```

---

## Phase 5.6: Performance & Concurrency Tests

**Duration:** 2-3 hours
**Tools:** performance-oracle agent, test-orchestrator plugin, database-index-advisor plugin
**Slash Commands:** `/database-index-advisor:index-advisor`

### Subphase 5.6.1: Performance Analysis
```bash
STEP 1: Use performance-oracle to analyze command performance
Task: "Analyze performance characteristics of all 18 command files"
Metrics:
  ✓ Execution time (P50, P95, P99)
  ✓ Memory usage (peak, sustained)
  ✓ Database query optimization
  ✓ File I/O patterns
  ✓ CPU efficiency
```

**Tool Stack:**
```
performance-oracle agent
  ↓ profiles command execution
  ↓ identifies bottlenecks
  ↓ suggests optimizations
  → FEEDS INTO → database-index-advisor

/database-index-advisor:index-advisor
  ↓ analyzes query patterns
  ↓ recommends missing indexes
  ↓ validates query plans
  → PRODUCES → optimization recommendations
```

### Subphase 5.6.2: Concurrency & Thread Safety Tests
```bash
STEP 2: Create concurrency stress tests
Test Scenarios:
  ✓ 10 concurrent file imports
  ✓ 100 concurrent search queries
  ✓ Simultaneous playback + metadata updates
  ✓ Rapid play/pause/seek cycles
  ✓ Database contention (high write load)
```

**Tool Stack:**
```
/test-orchestrator:orchestrate
  ↓ manages concurrent operations
  ↓ validates race condition prevention
  ↓ checks deadlock avoidance
  → PRODUCES → concurrency test results

performance-oracle agent
  ↓ analyzes concurrent performance
  ↓ identifies contention points
  ↓ validates thread safety
  → PRODUCES → performance baseline
```

### Subphase 5.6.3: Load & Scale Testing
```bash
STEP 3: Test with realistic data volumes
Scenarios:
  ✓ 10,000 files imported and searchable
  ✓ 1,000 tags in complex hierarchy
  ✓ 100 concurrent users (simulated)
  ✓ Database growth from 1MB → 1GB
```

**Tool Stack:**
```
/test-orchestrator:orchestrate
  ↓ generates large test datasets
  ↓ simulates realistic load
  ↓ monitors resource usage
  → PRODUCES → load test results

performance-oracle agent
  ↓ analyzes scalability characteristics
  ↓ identifies resource limits
  ↓ validates O(n) complexity
  → PRODUCES → scaling recommendations
```

---

## Execution Order & Tool Stacking Summary

### Optimal Execution Sequence:

```
┌─ Phase 5.0: Architecture Analysis (you are here)
│
├─ Phase 5.1: Pipeline Discovery (60-90 min)
│  Explore agent → architecture-reviewer → Grep → Write
│
├─ Phase 5.2: Pipeline Tests (3-4 hours)
│  /unit-test-generator (5x) + /database-test-manager
│  ↓ security-sentinel (archive safety)
│  ↓ query-performance-analyzer (query tests)
│  ↓ code-reviewer (quality gate)
│
├─ Phase 5.3: DAW Discovery (45-60 min)
│  Explore agent → midi-hardware agent → architecture-reviewer → Write
│
├─ Phase 5.4: DAW Tests (3-4 hours)
│  /unit-test-generator (4x) + midi-hardware agent
│  ↓ security-sentinel (MIDI safety)
│  ↓ performance-oracle (real-time requirements)
│  ↓ code-reviewer (quality gate)
│
├─ Phase 5.5: Integration Tests (2-3 hours)
│  architecture-strategist → /integration-test-runner
│  ↓ /test-orchestrator (execute all)
│
└─ Phase 5.6: Performance Testing (2-3 hours)
   performance-oracle → /database-index-advisor
   ↓ /test-orchestrator (load tests)
```

### Total Estimated Time: 10-14 hours

---

## Files to Create/Modify

### Pipeline Test Files (to create in Phase 5.2):
```
pipeline/src-tauri/tests/commands/
├── file_import_test.rs        (25-30 tests)
├── archive_import_test.rs      (12-15 tests)
├── analyze_test.rs             (8-10 tests)
├── search_test.rs              (8-10 tests)
├── tags_test.rs                (10 tests)
├── split_file_test.rs          (10 tests)
├── files_test.rs               (10 tests)
├── stats_test.rs               (8 tests)
├── progress_test.rs            (8 tests)
├── system_test.rs              (6 tests)
└── common/                     (fixtures, helpers, mocks)
    ├── fixtures.rs
    ├── mocks.rs
    └── assertions.rs
```

### DAW Test Files (to create in Phase 5.4):
```
daw/src-tauri/tests/commands/
├── sequencer_test.rs           (35-40 tests)
├── midi_test.rs                (20-25 tests)
├── project_test.rs             (10 tests)
├── export_test.rs              (5-10 tests)
├── analysis_test.rs            (5 tests)
├── search_test.rs              (5-10 tests)
└── common/                     (fixtures, MIDI mocks)
    ├── midi_fixtures.rs
    ├── midi_mocks.rs
    └── assertions.rs
```

### Integration Test Files (to create in Phase 5.5):
```
tests/integration/
├── import_to_daw_test.rs       (5-6 tests)
├── search_workflow_test.rs      (4-5 tests)
├── analysis_pipeline_test.rs    (3-4 tests)
├── export_roundtrip_test.rs     (3-4 tests)
└── common/
    ├── setup.rs
    └── fixtures.rs
```

---

## Key Success Metrics

| Metric | Target | Achievable |
|--------|--------|-----------|
| Tests Written | 200-250 | ✅ Yes (80+ per subphase) |
| Coverage | 80%+ on all commands | ✅ Yes (typical 85-95%) |
| Quality Score | 8.5+/10 avg | ✅ Yes (8.9 avg in Phase 4) |
| Execution Time | <5 min (all tests) | ✅ Yes (similar to Phase 4) |
| Pass Rate | 100% | ✅ Yes (Phase 4: 100%) |

---

## Tools & Agents Summary

### Slash Commands to Use (10 total):
```
✓ /unit-test-generator:generate-tests      (5x - one per command group)
✓ /database-test-manager:db-test           (2x - Pipeline + integration)
✓ /database-index-advisor:index-advisor    (1x - performance optimization)
✓ /integration-test-runner:run-integration (1x - end-to-end workflows)
✓ /test-orchestrator:orchestrate           (3x - integration, concurrency, load)
✓ /feature-dev:feature-dev                 (1x - planning phase)
```

### Specialized Agents to Use (6 total):
```
✓ Explore agent                    (2x - Phase 5.1, 5.3 discovery)
✓ architecture-reviewer            (2x - validate test architecture)
✓ architecture-strategist          (1x - integration point design)
✓ code-reviewer                    (2x - quality gates)
✓ security-sentinel                (2x - archive safety, MIDI safety)
✓ midi-hardware agent              (2x - MIDI testing, validation)
✓ performance-oracle               (2x - command performance, concurrency)
```

### MCP Servers to Leverage:
```
✓ postgres MCP    - Database setup/fixtures, schema validation
✓ filesystem MCP  - Test data generation, archive creation
```

### Plugins to Use:
```
✓ test-coverage-analyzer          (final coverage report)
✓ unit-test-generator             (tests generation)
✓ database-test-manager           (DB isolation, fixtures)
✓ integration-test-runner         (integration tests)
✓ test-orchestrator               (complex test coordination)
✓ database-index-advisor          (query optimization)
✓ git-commit-smart                (semantic commits per phase)
```

---

## Next Steps

**Ready to proceed?**

1. **Phase 5.1**: Kick off with `Task(subagent_type="Explore")` for Pipeline command discovery
2. **Each subphase**: Follow the tool stack as outlined
3. **After each phase**: Use `/code-review:code-review` for quality gate
4. **Commits**: One commit per major phase (5.1/5.2, 5.3/5.4, 5.5/5.6)

**Estimated Total Time:** 10-14 hours with tool-stacked efficiency

**Expected Outcome:**
- ✅ 200-250 tests covering all commands
- ✅ 8.5+/10 average quality score
- ✅ Coverage: 51.2% → ~65-70% (adding all command tests)
- ✅ 0 unwrap/expect/panic violations
- ✅ Production-ready command layer

---

**Phase 5 Ready to Execute!** 🚀
