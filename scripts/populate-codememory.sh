#!/bin/bash
# Populate CodeMemory Knowledge Base
# Run this from the project root to analyze and store project knowledge

cd "$(dirname "$0")/.." || exit 1

cc "Read and analyze my MIDI Software Center project. Here are the key documents:

CLAUDE.md - Complete project overview and status
ARCHITECTURE-REFERENCE.md - Three Archetypes Pattern
PROJECT-STRUCTURE.md - Directory organization
DEVELOPMENT-WORKFLOW.md - 8-step feature process
TEST-COVERAGE-PLAN.md - Testing strategy (51.2% → 100%)
FINAL-FILE-SEPARATION.md - 222 files migrated
UNWRAP-AUDIT-REPORT.md - Zero unwrap/expect/panic achievement

Key facts:
- 222 files, ~53,000 lines of Rust/TypeScript
- PostgreSQL 16 + pgvector for 3M+ MIDI files
- Tauri 2.7 desktop apps (Pipeline + DAW)
- Test coverage: 51.2% (689 tests), targeting 100%
- Phase 4 in progress: Repository layer testing

Please extract and store:
1. Architecture patterns (Three Archetypes)
2. Code quality requirements (80% coverage, zero unwraps)
3. Component separation rules (Shared/Pipeline/DAW)
4. Testing strategy and current progress
5. Common workflows and commands
6. Technology stack and dependencies

This will serve as the foundation knowledge for future sessions."
