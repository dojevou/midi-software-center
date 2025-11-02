# Phase 4: Repository Layer Testing - Next Steps (Tool-Stacked Workflow)

**Date:** 2025-11-01
**Current Status:** Phase 4.4 Complete (search_repository ✅)
**Next:** Phase 4.5+ (Remaining Repositories)

---

## 📊 Current Progress

### ✅ Completed (Pipeline Repositories)
- **Phase 4.1:** file_repository (109 tests, ~95% coverage)
- **Phase 4.2:** tag_repository (100 tests, 95%+ coverage)
- **Phase 4.3:** metadata_repository (79 tests, 90%+ coverage)
- **Phase 4.4:** search_repository (82 tests, 95%+ coverage)

**Total:** 370 tests, 4 repositories ✅

### 🔄 Remaining Work

**Shared Library Repositories (Not yet tested):**
1. `shared/rust/src/db/repositories/file_repository.rs`
2. `shared/rust/src/db/repositories/tag_repository.rs`
3. `shared/rust/src/db/repositories/metadata_repository.rs`
4. `shared/rust/src/db/repositories/search_repository.rs`
5. `shared/rust/src/db/repositories/mod.rs`

**Estimated:** 200-250 tests, 8-12 hours

---

## 🎯 Phase 4.5: Shared Library Repositories

### Overview
Test all 5 shared repository files with comprehensive coverage, using maximum tool stacking for efficiency.

**Goal:** 90%+ coverage for all shared repositories
**Estimated Time:** 8-12 hours
**Tools Used:** 15+ tools stacked together

---

## 🛠️ COMPREHENSIVE TOOL-STACKED WORKFLOW

### Step 1: Analysis & Planning (30 minutes)

#### 🔍 Analyze Current State

**Tool Stack:**
- MCP Server: `filesystem` (file operations)
- MCP Server: `rust` (cargo analysis)
- Agent: `rust-backend` (code understanding)
- Agent: `database` (schema understanding)

**Commands:**
```bash
# 1. List all repository files
find shared/rust/src/db/repositories -name "*.rs" -type f

# 2. Analyze code complexity
wc -l shared/rust/src/db/repositories/*.rs

# 3. Check existing tests
find shared/rust/tests -name "*repository*.rs" -type f
```

**Agent Invocation:**
```
"Use the rust-backend agent to analyze the shared repository files and
estimate test coverage requirements. Focus on:
1. Public method count per repository
2. Error handling patterns
3. Database query complexity
4. Transaction boundaries"
```

**Expected Output:** Comprehensive analysis report with test estimates

---

### Step 2: Database Schema Analysis (30 minutes)

#### 🗄️ Deep Dive into Schema

**Tool Stack:**
- MCP Server: `postgres` (query database directly)
- Agent: `database` (PostgreSQL expert)
- Agent: `data-integrity-guardian` (constraint analysis)
- Skill: `query-performance-analyzer:query-performance-analyzer`

**MCP Commands (postgres):**
```sql
-- Via postgres MCP: Get all table schemas
SELECT table_name, column_name, data_type, is_nullable
FROM information_schema.columns
WHERE table_schema = 'public'
ORDER BY table_name, ordinal_position;

-- Get all constraints
SELECT tc.table_name, tc.constraint_name, tc.constraint_type,
       kcu.column_name, ccu.table_name AS foreign_table_name,
       ccu.column_name AS foreign_column_name
FROM information_schema.table_constraints AS tc
JOIN information_schema.key_column_usage AS kcu
  ON tc.constraint_name = kcu.constraint_name
LEFT JOIN information_schema.constraint_column_usage AS ccu
  ON ccu.constraint_name = tc.constraint_name
WHERE tc.table_schema = 'public';

-- Get all indexes
SELECT tablename, indexname, indexdef
FROM pg_indexes
WHERE schemaname = 'public'
ORDER BY tablename, indexname;
```

**Agent Invocation:**
```
"Use the database agent to analyze the PostgreSQL schema and identify:
1. Foreign key constraints that need testing
2. UNIQUE constraints that need validation
3. CHECK constraints that need edge case tests
4. Trigger functions that need verification
5. Index usage patterns for performance tests

Use the data-integrity-guardian agent to validate all referential integrity scenarios."
```

**Expected Output:** Schema analysis with test scenarios

---

### Step 3: Test Infrastructure Setup (45 minutes)

#### 🏗️ Build Comprehensive Test Framework

**Tool Stack:**
- Slash Command: `/database-test-manager:db-test`
- Agent: `rust-backend` (test infrastructure)
- Agent: `database` (fixtures and setup)
- MCP Server: `bash` (file operations)
- MCP Server: `rust` (cargo commands)

**Slash Command:**
```bash
/database-test-manager:db-test
```

**Follow-up Instructions:**
```
"After setting up database test infrastructure, use the rust-backend agent to:
1. Create comprehensive test fixtures for all 18 database tables
2. Build helper functions for common test operations
3. Implement transaction rollback for test isolation
4. Create mock data generators with realistic values

Fixtures should include:
- Files: MIDI files with varying sizes, formats, manufacturers
- Tags: Genre, instrument, mood tags with many-to-many relationships
- Metadata: Musical metadata (BPM, key, time signature)
- Associations: File-tag associations, duplicates, favorites

Use BigDecimal for NUMERIC types, proper ENUM casting for PostgreSQL types."
```

**Files to Create:**
```
shared/rust/tests/
├── fixtures/
│   ├── mod.rs (fixture exports)
│   ├── file_fixtures.rs (NewFileBuilder, Fixtures::drum_loop, etc.)
│   ├── tag_fixtures.rs (TagBuilder, genre/instrument/mood presets)
│   ├── metadata_fixtures.rs (MetadataBuilder, musical presets)
│   └── association_fixtures.rs (file-tag, duplicate fixtures)
├── helpers/
│   ├── mod.rs
│   ├── db.rs (cleanup_database, setup_test_pool)
│   └── assertions.rs (assert_file_eq, assert_metadata_close)
└── common/
    ├── mod.rs
    └── test_utils.rs (generate_hash, random_file, etc.)
```

**Expected Output:** Complete test infrastructure ready

---

### Step 4: Generate Repository Tests (2-3 hours per repository)

#### 📝 Comprehensive Test Generation (Stacked Workflow)

**For Each Repository (file, tag, metadata, search, mod):**

##### 🎯 4A. Test Coverage Analysis

**Tool Stack:**
- Slash Command: `/test-coverage-analyzer:analyze-coverage`
- Agent: `pattern-recognition-specialist` (identify test patterns)
- MCP Server: `rust` (code analysis)

**Slash Command:**
```bash
/test-coverage-analyzer:analyze-coverage

# Focus on: shared/rust/src/db/repositories/file_repository.rs
```

**Agent Follow-up:**
```
"Use the pattern-recognition-specialist agent to:
1. Analyze the repository code for all public methods
2. Identify error handling patterns (Result types)
3. Detect database query patterns (INSERT, UPDATE, SELECT, DELETE)
4. Find edge cases (NULL values, boundary conditions, large data)
5. Estimate test count needed for 90%+ coverage

Generate a test plan with specific test categories."
```

**Expected Output:** Test coverage gap analysis with recommendations

---

##### 🎯 4B. Generate Initial Test Suite

**Tool Stack:**
- Slash Command: `/unit-test-generator:generate-tests`
- Agent: `rust-backend` (Rust best practices)
- Agent: `database` (SQL expertise)

**Slash Command:**
```bash
/unit-test-generator:generate-tests

# Target: shared/rust/src/db/repositories/file_repository.rs
# Generate: 80-100 tests covering all methods
```

**Agent Enhancement:**
```
"Use the rust-backend agent to enhance the generated tests with:
1. Proper error handling (no unwrap/expect in production paths)
2. Comprehensive edge cases (empty strings, very long strings, special characters)
3. Database constraints testing (UNIQUE violations, FK violations, CHECK constraints)
4. Transaction testing (commit, rollback, isolation)
5. Realistic test data using fixtures

Use the database agent to ensure:
1. All SQL queries are properly parameterized (SQL injection safe)
2. BigDecimal used for NUMERIC types
3. Proper PostgreSQL ENUM casting (key_signature::text::musical_key)
4. LEFT JOIN correctness for optional relationships
5. CASCADE delete behavior tested"
```

**Expected Output:** 80-100 comprehensive tests

---

##### 🎯 4C. Add Database-Specific Tests

**Tool Stack:**
- Slash Command: `/database-test-manager:db-test`
- Agent: `data-integrity-guardian` (integrity checks)
- MCP Server: `postgres` (direct SQL testing)

**Agent Invocation:**
```
"Use the data-integrity-guardian agent to create additional tests for:

1. **Foreign Key Constraints:**
   - INSERT with invalid FK (should fail)
   - DELETE with CASCADE (should delete children)
   - UPDATE FK reference (should update/fail based on constraints)

2. **UNIQUE Constraints:**
   - Duplicate filepath (should fail)
   - Duplicate content_hash with different path (allowed)
   - Case sensitivity in UNIQUE columns

3. **CHECK Constraints:**
   - Invalid MIDI format (not 0, 1, or 2)
   - Invalid multi_track/parent_file_id combination
   - Negative file_size_bytes

4. **Transaction Integrity:**
   - Rollback on error (no partial commits)
   - Isolation between concurrent transactions
   - Deadlock prevention

5. **Trigger Functions:**
   - search_vector auto-population
   - updated_at auto-update
   - Custom triggers if any

Generate 20-30 additional tests for these scenarios."
```

**MCP Testing (postgres):**
```sql
-- Test FK constraint directly via MCP
BEGIN;
INSERT INTO musical_metadata (file_id, bpm, key_signature)
VALUES (999999, 120, 'C'::text::musical_key);
-- Should fail: FK violation
ROLLBACK;
```

**Expected Output:** 20-30 database constraint tests

---

##### 🎯 4D. Performance & Query Optimization Tests

**Tool Stack:**
- Skill: `query-performance-analyzer:query-performance-analyzer`
- Agent: `performance-oracle` (performance analysis)
- MCP Server: `postgres` (EXPLAIN plans)

**Skill Invocation:**
```
Use the query-performance-analyzer skill to:
1. Analyze all SELECT queries in the repository
2. Check index usage (should use idx_*, not seq scans)
3. Identify N+1 query patterns
4. Test pagination performance (OFFSET vs cursor-based)
5. Benchmark batch operations (bulk inserts)

Generate performance tests that fail if queries are too slow:
- Single row operations: <10ms
- Batch operations (100 rows): <100ms
- Full table scans with LIMIT: <50ms
- Search queries with indexes: <20ms
```

**Agent Invocation:**
```
"Use the performance-oracle agent to:
1. Identify slow queries (>50ms)
2. Suggest index improvements
3. Recommend query rewrites (CTEs, subqueries vs JOINs)
4. Test with realistic data volumes (1000+ files)

Create 10-15 performance tests with assertions on execution time."
```

**MCP Testing (postgres):**
```sql
-- Get EXPLAIN plans via MCP
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM files WHERE manufacturer = 'Roland' LIMIT 10;

-- Check index usage
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM files WHERE search_vector @@ plainto_tsquery('english', 'piano');
```

**Expected Output:** 10-15 performance tests

---

##### 🎯 4E. Code Quality Review

**Tool Stack:**
- Agent: `kieran-rust-reviewer` (strict code review)
- Agent: `rust-backend` (Rust best practices)
- Agent: `code-simplicity-reviewer` (simplification)

**Agent Invocation (Sequential):**
```
"Use the kieran-rust-reviewer agent to review all generated tests for:
1. Zero unwrap/expect/panic in production code paths
2. Proper error handling (Result propagation)
3. Clear test names (test_insert_duplicate_filepath_fails)
4. Comprehensive assertions (not just 'it runs')
5. No code duplication (use fixtures and helpers)

After Kieran's review, use the code-simplicity-reviewer agent to:
1. Identify over-complicated test logic
2. Suggest fixture consolidation
3. Recommend helper function extraction
4. Simplify assertion patterns

Finally, use the rust-backend agent to ensure:
1. Tests follow Rust conventions (snake_case, #[tokio::test])
2. Async/await used correctly
3. Pool management is efficient
4. No resource leaks (connections, transactions)"
```

**Expected Output:** High-quality, production-ready tests

---

##### 🎯 4F. Run Tests & Fix Issues

**Tool Stack:**
- MCP Server: `rust` (cargo test)
- MCP Server: `bash` (test execution)
- Agent: `rust-backend` (compilation fixes)

**Bash Commands:**
```bash
# Compile tests first
cargo test --test file_repository_test --no-run

# Run with single thread (shared database)
cargo test --test file_repository_test -- --test-threads=1

# Check coverage (if tarpaulin available)
cargo tarpaulin --test file_repository_test --out Html
```

**Agent for Fixes:**
```
"If tests fail, use the rust-backend agent to:
1. Fix compilation errors (type mismatches, imports)
2. Fix runtime errors (database constraints, SQL errors)
3. Add missing cleanup (database state between tests)
4. Fix flaky tests (timing issues, ordering dependencies)

Iterate until all tests pass (100% pass rate required)."
```

**Expected Output:** All tests passing, 90%+ coverage

---

### Step 5: Security & Vulnerability Testing (1 hour)

#### 🔒 Security-Focused Testing

**Tool Stack:**
- Agent: `security-sentinel` (vulnerability scanner)
- Agent: `input-validation-scanner` (input validation)
- Skill: `input-validation-scanner:scan-input`

**Agent Invocation:**
```
"Use the security-sentinel agent to perform security audit on all repository code:

1. **SQL Injection:**
   - Test all dynamic SQL queries
   - Verify prepared statements used everywhere
   - Test special characters in inputs (' OR '1'='1, etc.)

2. **Path Traversal:**
   - Test filepath inputs (../../etc/passwd)
   - Verify path sanitization

3. **Integer Overflow:**
   - Test file_size_bytes boundaries (i64::MAX)
   - Test BPM/tempo boundaries (NUMERIC precision)

4. **Hash Collisions:**
   - Test content_hash uniqueness
   - Verify BYTEA handling

5. **Denial of Service:**
   - Test very large batch operations (10000+ rows)
   - Test query timeouts
   - Test connection pool exhaustion

Generate 15-20 security tests."
```

**Skill Invocation:**
```bash
/input-validation-scanner:scan-input

# Target: All repository public methods
# Focus: User-provided inputs (filepath, filename, manufacturer, etc.)
```

**Expected Output:** 15-20 security tests, vulnerability report

---

### Step 6: Integration Testing (1 hour)

#### 🔗 Cross-Repository Integration

**Tool Stack:**
- Slash Command: `/integration-test-runner:run-integration`
- Agent: `architecture-strategist` (integration patterns)
- MCP Server: `postgres` (multi-table operations)

**Slash Command:**
```bash
/integration-test-runner:run-integration

# Test: File + Metadata + Tags together
# Scenario: Import MIDI file with full metadata extraction
```

**Agent Invocation:**
```
"Use the architecture-strategist agent to design integration tests that:

1. **Full File Import Flow:**
   - Insert file → file_repository
   - Extract metadata → metadata_repository
   - Auto-tag file → tag_repository
   - Index for search → search_repository
   - Verify all relationships

2. **Duplicate Detection:**
   - Insert file with same content_hash
   - Verify duplicate_groups created
   - Test CASCADE delete behavior

3. **Batch Operations:**
   - Import 100 files in transaction
   - Verify atomicity (all or nothing)
   - Test rollback on error

4. **Search with Filters:**
   - Search by text + BPM + tags
   - Verify JOIN correctness
   - Test pagination across repositories

Generate 10-15 integration tests."
```

**Expected Output:** 10-15 integration tests

---

### Step 7: Documentation & Commit (30 minutes)

#### 📄 Document Phase Completion

**Tool Stack:**
- Agent: `git-history-analyzer` (context for commit)
- MCP Server: `git` (commit operations)
- Agent: `best-practices-researcher` (commit message format)

**Agent Invocation:**
```
"Use the best-practices-researcher agent to research conventional commit best practices for test additions.

Then create comprehensive documentation:
1. PHASE-4.5-COMPLETION.md (similar to Phase 4.4)
2. Test coverage report
3. Performance benchmarks
4. Security audit results

Format: Professional, metrics-driven, production-ready"
```

**Git Commit:**
```bash
# Stage files
git add shared/rust/tests/
git add docs/testing/PHASE-4.5-COMPLETION.md

# Commit with comprehensive message
git commit -m "test(shared-repositories): complete Phase 4.5 with XXX tests

- Add comprehensive tests for 5 shared repository files
- Achieve 90%+ coverage on all repositories
- Include security, performance, and integration tests
- Zero unwrap/expect/panic in production code

Details:
- file_repository: XX tests
- tag_repository: XX tests
- metadata_repository: XX tests
- search_repository: XX tests
- mod.rs: XX tests

Total: XXX tests, YY% coverage

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

**Expected Output:** Professional commit, complete documentation

---

## 📊 Tool Usage Summary (Per Repository)

### Tools Used Per Repository File
| Tool Type | Tool Name | Usage Count | Purpose |
|-----------|-----------|-------------|---------|
| **Slash Commands** | /test-coverage-analyzer:analyze-coverage | 1x | Coverage gaps |
| | /unit-test-generator:generate-tests | 1x | Initial tests |
| | /database-test-manager:db-test | 1x | DB test setup |
| | /integration-test-runner:run-integration | 1x | Integration tests |
| | /input-validation-scanner:scan-input | 1x | Security scan |
| **Agents** | rust-backend | 3-4x | Code quality, fixes |
| | database | 2-3x | SQL expertise |
| | data-integrity-guardian | 2x | Constraints, integrity |
| | security-sentinel | 1x | Vulnerability scan |
| | performance-oracle | 1x | Performance analysis |
| | kieran-rust-reviewer | 1x | Code review |
| | code-simplicity-reviewer | 1x | Simplification |
| | pattern-recognition-specialist | 1x | Pattern analysis |
| | architecture-strategist | 1x | Integration design |
| **Skills** | query-performance-analyzer | 1x | Query optimization |
| | input-validation-scanner | 1x | Input validation |
| **MCP Servers** | postgres | 10-15x | Direct SQL testing |
| | rust | 5-10x | Cargo operations |
| | bash | 5-10x | File operations |
| | filesystem | 3-5x | File management |
| | git | 1-2x | Version control |

**Total Tools per Repository:** 15-20 different tools
**Total Tool Invocations:** 40-50 per repository

---

## 🚀 Parallel Execution Strategy

### Option 1: Sequential (Safer)
Process one repository at a time with full tool stack.

**Timeline:** 2.5h per repository × 5 = 12.5 hours

**Advantages:**
- Can learn from previous repository
- Reuse fixtures and patterns
- Less context switching

### Option 2: Parallel (Faster)
Process multiple repositories simultaneously.

**Tool Stack for Parallel:**
- Use multiple Claude Code sessions (if available)
- Task tool with multiple agents in parallel
- Separate database schemas for isolation

**Timeline:** 8-10 hours (30% faster)

**Command (if using parallel):**
```
"Launch 5 parallel tasks using the Task tool:
1. file_repository tests (rust-backend + database agents)
2. tag_repository tests (rust-backend + database agents)
3. metadata_repository tests (rust-backend + database agents)
4. search_repository tests (rust-backend + database agents)
5. mod.rs tests (rust-backend agent)

Each task should generate tests independently using the full tool stack."
```

---

## 🎯 Success Criteria (Per Repository)

### Code Coverage
- ✅ 90%+ line coverage
- ✅ 100% function coverage (all public methods)
- ✅ All error paths tested
- ✅ All edge cases covered

### Code Quality
- ✅ Zero unwrap/expect/panic in production code
- ✅ Kieran's review score: 9.0+ / 10
- ✅ All tests passing (100% pass rate)
- ✅ No flaky tests (consistent results)

### Security
- ✅ Security-sentinel audit: 9.0+ / 10
- ✅ All SQL injection vectors tested
- ✅ Input validation comprehensive
- ✅ No critical vulnerabilities

### Performance
- ✅ Single operations: <10ms
- ✅ Batch operations: <100ms
- ✅ Search queries: <20ms
- ✅ All indexes used correctly

### Documentation
- ✅ Completion report written
- ✅ Coverage metrics documented
- ✅ Performance benchmarks included
- ✅ Security audit results documented

---

## 📋 Quick Start Checklist

### Before Starting Phase 4.5

- [ ] Read this document completely (20 min)
- [ ] Verify all MCP servers are configured and working
- [ ] Verify all agents are available
- [ ] Test database connection (postgres MCP)
- [ ] Create branch: `git checkout -b phase-4.5-shared-repositories`
- [ ] Review TEST-COVERAGE-PLAN.md for context
- [ ] Review Phase 4.4 completion for patterns

### During Phase 4.5 (Per Repository)

- [ ] **Step 1:** Analyze code with rust-backend + database agents (30 min)
- [ ] **Step 2:** Analyze schema with postgres MCP + data-integrity-guardian (30 min)
- [ ] **Step 3:** Setup test infrastructure with /database-test-manager (45 min)
- [ ] **Step 4A:** Coverage analysis with /test-coverage-analyzer (15 min)
- [ ] **Step 4B:** Generate tests with /unit-test-generator (30 min)
- [ ] **Step 4C:** Database tests with data-integrity-guardian (30 min)
- [ ] **Step 4D:** Performance tests with query-performance-analyzer (30 min)
- [ ] **Step 4E:** Code review with kieran-rust-reviewer (20 min)
- [ ] **Step 4F:** Run tests and fix with rust-backend (30 min)
- [ ] **Step 5:** Security testing with security-sentinel (1 hour)
- [ ] **Step 6:** Integration testing with /integration-test-runner (1 hour)
- [ ] **Step 7:** Document and commit (30 min)

**Total per repository:** 2.5-3 hours
**Total for 5 repositories:** 12-15 hours

### After Phase 4.5

- [ ] All tests passing (100% pass rate)
- [ ] Coverage measured (90%+ achieved)
- [ ] Documentation complete
- [ ] Commit created and pushed
- [ ] Update TEST-COVERAGE-PLAN.md progress
- [ ] Celebrate! 🎉

---

## 🔥 Pro Tips for Maximum Efficiency

### 1. Stack Tools Aggressively
Don't use one tool at a time. Example:
```
"Use the rust-backend agent to generate tests, then immediately pipe the
output to the kieran-rust-reviewer agent for quality review, then to the
code-simplicity-reviewer for simplification, then compile with rust MCP."
```

### 2. Reuse Fixtures
After creating fixtures for file_repository, reuse them for all other repositories:
```rust
// In tag_repository_test.rs
use crate::fixtures::file_fixtures::Fixtures;

let file_id = Fixtures::drum_loop().insert(&pool).await?;
```

### 3. Parallel Agent Invocation
Use Task tool to run multiple agents simultaneously:
```
"Launch 3 agents in parallel using the Task tool:
1. rust-backend: analyze file_repository.rs
2. database: analyze schema relationships
3. pattern-recognition-specialist: identify test patterns

Synthesize their outputs into a comprehensive test plan."
```

### 4. MCP for Fast Verification
Use postgres MCP to quickly verify database behavior:
```sql
-- Fast check if test data exists
SELECT COUNT(*) FROM files WHERE filename LIKE 'test_%';

-- Fast check of constraints
SELECT conname, contype FROM pg_constraint
WHERE conrelid = 'files'::regclass;
```

### 5. Continuous Quality Gates
After each test generation step, immediately run quality checks:
```bash
# Generate tests
/unit-test-generator:generate-tests

# Immediately review (don't wait)
"Use kieran-rust-reviewer to review the generated tests NOW"

# Immediately run (catch errors early)
cargo test --test file_repository_test -- --test-threads=1
```

---

## 🎓 Learning from Phase 4.4

### What Worked Well
1. ✅ Fixing compilation errors immediately (don't accumulate)
2. ✅ Database trigger enhancements during testing
3. ✅ Comprehensive documentation
4. ✅ Tool stacking (agents + MCP + slash commands)

### What to Improve
1. ⚡ Use postgres MCP more for fast SQL verification
2. ⚡ Run security-sentinel earlier (not at the end)
3. ⚡ Generate fixtures first (before any test generation)
4. ⚡ Use parallel agent invocation more aggressively

### Apply These Improvements
- Generate ALL fixtures FIRST (Step 3)
- Use postgres MCP throughout (not just at end)
- Run security-sentinel after Step 4B (not Step 5)
- Use Task tool for parallel agent invocation

---

## 📞 Quick Reference Commands

### Most Used Commands

```bash
# Start analysis
/test-coverage-analyzer:analyze-coverage

# Generate tests
/unit-test-generator:generate-tests

# Database setup
/database-test-manager:db-test

# Security scan
/input-validation-scanner:scan-input

# Integration tests
/integration-test-runner:run-integration

# Run tests
cargo test --test TESTNAME -- --test-threads=1

# Check coverage
cargo tarpaulin --test TESTNAME --out Html

# Commit
git add . && git commit -m "test(shared-repositories): ..."
```

### Most Used Agents

```
rust-backend - Rust code expertise
database - PostgreSQL/SQL expertise
data-integrity-guardian - Constraints and integrity
security-sentinel - Security vulnerabilities
performance-oracle - Performance optimization
kieran-rust-reviewer - Strict code review
```

### Most Used MCP Servers

```
postgres - Direct SQL queries
rust - Cargo operations
bash - Shell commands
filesystem - File operations
git - Version control
```

---

## ✅ Ready to Start?

**Next Command:**
```
"I'm ready to start Phase 4.5: Shared Library Repositories.

Let's begin with Step 1: Analysis & Planning.

Use the rust-backend agent and database agent in parallel to analyze:
1. shared/rust/src/db/repositories/file_repository.rs
2. shared/rust/src/db/repositories/tag_repository.rs
3. shared/rust/src/db/repositories/metadata_repository.rs
4. shared/rust/src/db/repositories/search_repository.rs
5. shared/rust/src/db/repositories/mod.rs

Provide comprehensive analysis with test estimates for each file."
```

---

**Total Tools Available:** 15+ tools
**Estimated Stacking Opportunities:** 40-50 per repository
**Expected Efficiency Gain:** 3-5x faster than manual testing

**Let's build comprehensive test coverage with maximum tool leverage!** 🚀
