# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ⚠️ CRITICAL: Project Status

**This project is in the MIGRATION PHASE.**

**Current State:**
- ✅ Directory structure created (empty, ready for migration)
- ✅ Comprehensive planning documents completed (~250KB of markdown)
- ✅ **Original production code identified** - 222 files ready to migrate
- ✅ Deduplication analysis complete
- ✅ Component separation plan finalized
- ✅ **MCP servers configured** - 12 servers (postgres, filesystem, docker, git, bash, rust, npm, vscode, web-search, anthropic, sentry, notion)
- ✅ **Git repository initialized** - Commit be6f2ba (353 planning files)
- ✅ **Source code extracted** - `/tmp/original-project/midi-library-system/` (240 production files verified)
- ⏳ Ready to begin migration

**Source Code Status:**
- **Original code location:** `midi-library-system-refined.tar.gz` (extracted to `/tmp/original-project/`)
- **Source of truth:** `/tmp/original-project/midi-library-system/` (root directory - BEST version)
- **Production-ready files:** 123 Rust files, 80 TypeScript/Svelte files, 6 SQL files
- **Code quality:** HIGH - professionally written, well-structured
- **Total lines:** ~53,000 lines of production code

**What EXISTS in original code:**
- ✅ Complete database schema (PostgreSQL 16 + pgvector, optimized for 3M+ files)
- ✅ Full shared library with MIDI parser (921 lines), BPM/key detection, database layer
- ✅ Complete Pipeline backend (Rust + Tauri) - batch import, analysis, archive extraction
- ✅ Complete DAW backend (Rust + Tauri) - sequencer engine, MIDI I/O, real-time playback
- ✅ Complete frontends (Svelte + TypeScript) - PianoRoll (800 lines), UI components
- ✅ Optimized build system (Makefile, Cargo workspace, docker-compose)
- ✅ CLI tools and scripts

**What DOES NOT EXIST YET in new structure:**
- Any migrated code (migration hasn't started)
- The new project directory has planning docs only

See `FINAL-FILE-SEPARATION.md` for complete migration plan with source→destination mapping.

## 📚 Critical Architecture Documents (READ THESE FIRST)

**Before starting migration or development, read these three foundational documents:**

1. **[ARCHITECTURE-REFERENCE.md](./ARCHITECTURE-REFERENCE.md)** - The Building Code
   - Complete guide to the Three Archetypes Pattern (Task-O-Matic, Grown-up Script, Trusty Module)
   - Decision tree for classifying code
   - Code quality requirements (no `.unwrap()`, 80% test coverage, documentation)
   - Examples from this specific project

2. **[PROJECT-STRUCTURE.md](./PROJECT-STRUCTURE.md)** - The City Map
   - Complete directory structure for all components
   - File placement rules (where does each type of code go?)
   - Archetype-to-directory mapping
   - Examples of file classification

3. **[DEVELOPMENT-WORKFLOW.md](./DEVELOPMENT-WORKFLOW.md)** - The Construction Manual
   - 8-step feature implementation process
   - Testing strategy by archetype
   - Code review checklist
   - Common workflows (adding features, fixing bugs, refactoring)

**These documents define HOW to work with this codebase and are MANDATORY reading.**

## Project Overview

**MIDI Software Center** is migrating from an existing high-quality codebase to a new, better-organized structure. The original project was functional but had duplicate files and misplaced components. The migration will result in:

1. **Database Layer** - PostgreSQL 16 + Meilisearch for storing and searching MIDI files
2. **Pipeline** - Tauri-based desktop app for batch MIDI file processing and import
3. **DAW (Digital Audio Workstation)** - Tauri-based desktop app for real-time MIDI sequencing and playback
4. **Shared Library** - Reusable MIDI parsing, analysis, and database code
5. **Scripts** - Automation, CLI tools, and operational scripts

The system is designed to handle large-scale MIDI libraries (3M+ files) with semantic search, metadata extraction, and real-time audio processing capabilities.

## Technology Stack (Production-Ready Code)

**Backend:**
- Rust 1.70+ (primary backend - performance-critical audio/MIDI processing)
- Tauri 2.7 (desktop application framework)
- tokio 1.35 (async runtime with parking_lot)
- sqlx 0.7 (type-safe SQL with compile-time checking)
- MIDI libraries: midly 0.5, midir, rimd

**Frontend:**
- Svelte 4.2 (reactive UI framework)
- TypeScript 5.3 (type safety)
- Vite 5.0 (build tool and dev server)
- Tone.js 14.7 (Web Audio API wrapper for DAW)

**Database:**
- PostgreSQL 16 (primary database with pgvector extension)
- Meilisearch 1.5 (full-text search engine)

**Build & Operations:**
- Makefile (40+ targets for common tasks)
- Docker Compose (containerization)
- pnpm 8.11 (package manager)
- Bash scripts (operations and deployment)

## Original Code Architecture

The original production code uses a **Rust workspace** structure with shared code:

```
midi-library-system/ (SOURCE OF TRUTH)
├── database/           # PostgreSQL + Meilisearch (docker-compose)
│   ├── migrations/    # 4 SQL migrations (001-006)
│   ├── queries/       # Common queries
│   └── scripts/       # Sample data
├── shared/rust/        # ✨ COMPLETE shared library (24 modules!)
│   └── src/
│       ├── core/
│       │   ├── midi/       # Parser (921 lines), types, error handling
│       │   └── analysis/   # BPM detector, key detector, auto-tagger
│       └── db/
│           ├── models/     # All database models (7 modules)
│           └── repositories/  # Data access layer (4 repositories)
├── pipeline/           # Batch processor Tauri app
│   ├── src/           # Svelte frontend (~30 files)
│   └── src-tauri/     # Rust backend (~45 files)
│       ├── commands/  # Tauri IPC commands (file import, analysis, search, etc.)
│       ├── database/  # Batch insert optimization
│       └── io/        # Archive extraction (ZIP, RAR, 7z)
├── daw/               # Real-time DAW Tauri app
│   ├── src/           # Svelte frontend (~50 files)
│   │   └── lib/components/
│   │       ├── PianoRoll.svelte   # 800+ lines
│   │       ├── Sequencer.svelte   # 600+ lines
│   │       └── FavoritesList.svelte
│   └── src-tauri/     # Rust backend (~53 files)
│       ├── midi/      # MIDI manager (450 lines)
│       ├── sequencer/ # Engine (800+ lines), track, scheduler
│       └── core/      # Timing, loader, writer, validator
├── scripts/           # Launch scripts
│   ├── launch-all.sh, launch-daw.sh, launch-pipeline.sh
│   ├── status.sh, stop-all.sh
│   └── import-tool/   # CLI import utility (Rust)
├── Cargo.toml         # Root workspace config (172 lines, optimized profiles)
├── Makefile           # 40+ targets (setup, dev, build, test, etc.)
└── docker-compose.yml # PostgreSQL + Meilisearch setup
```

**Key architectural features:**
- Workspace structure allows code sharing between pipeline and DAW
- Separation of concerns: database layer, batch processing, real-time audio
- Desktop apps use Tauri for native performance with web UI flexibility
- Rust handles performance-critical MIDI/audio processing
- Svelte provides reactive, efficient UIs

## Migration Plan

See `FINAL-FILE-SEPARATION.md` for comprehensive source→destination mapping.

**Migration Phases:**

### Phase 1 - Foundation (Day 1)
1. Copy database/ directory (migrations, docker-compose)
2. Copy shared/rust/ library (24 modules)
3. Copy root configs (Cargo.toml, Makefile, docker-compose.yml)
4. Test: `docker-compose up` and `cargo build`

### Phase 2 - Backend (Days 2-3)
1. Copy pipeline/src-tauri/ (Rust backend)
2. Copy daw/src-tauri/ (Rust backend)
3. Copy scripts/import-tool/ (CLI tool)
4. Test: `cargo build --all`

### Phase 3 - Frontend (Day 3-4)
1. Copy pipeline/src/ (Svelte frontend)
2. Copy daw/src/ (Svelte frontend)
3. Copy package.json files
4. Test: `pnpm install` and `pnpm dev`

### Phase 4 - Scripts & Verification (Day 4)
1. Copy launch scripts to scripts/launch/
2. Adapt setup scripts to scripts/setup/
3. Copy verification scripts to scripts/verify/
4. Test: `make dev-both`

### Phase 5 - Final Verification (Day 5)
1. Full integration test
2. Verify all components compile
3. Test database connectivity
4. Test dev servers
5. Update documentation

**Estimated Time:** 5 days (full-time) or 2 weeks (part-time)

## Component Separation (Critical Rules)

**See [ARCHITECTURE-REFERENCE.md](./ARCHITECTURE-REFERENCE.md) and [PROJECT-STRUCTURE.md](./PROJECT-STRUCTURE.md) for complete rules.**

**Quick Reference:**

### Shared Library ONLY Contains:
- ✅ MIDI parsing (for analysis)
- ✅ Musical analysis (BPM, key detection, auto-tagging)
- ✅ Database models and repositories
- ❌ NO UI code
- ❌ NO application logic
- ❌ NO Tauri commands

### Pipeline ONLY Contains:
- ✅ Batch import commands
- ✅ File analysis
- ✅ Archive extraction (ZIP, RAR, 7z)
- ✅ Database batch insert operations
- ✅ Pipeline-specific UI
- ❌ NO real-time playback
- ❌ NO MIDI hardware I/O

### DAW ONLY Contains:
- ✅ Real-time sequencer engine
- ✅ MIDI hardware manager (midir)
- ✅ Playback engine with timing
- ✅ MIDI file loader (for playback)
- ✅ DAW-specific UI (PianoRoll, Sequencer, Transport)
- ❌ NO batch file import
- ❌ NO archive extraction

### Database ONLY Contains:
- ✅ SQL migrations (numbered 001-006)
- ✅ docker-compose.yml (PostgreSQL 16 + Meilisearch)
- ✅ Sample data and utility queries
- ❌ NO application code

### Scripts ONLY Contains:
- ✅ Launch scripts (start/stop services)
- ✅ CLI import tool (Rust binary)
- ✅ Setup automation
- ✅ Verification scripts
- ❌ NO emergency/fix scripts (those are archived)

## Development Workflow (Post-Migration)

**See [DEVELOPMENT-WORKFLOW.md](./DEVELOPMENT-WORKFLOW.md) for complete 8-step feature implementation process.**

Once migrated, these commands will work:

```bash
# Setup (first time)
make setup                    # Install all dependencies
make docker-up               # Start PostgreSQL + Meilisearch

# Development (daily use)
make dev-pipeline            # Launch pipeline in dev mode (port 5173)
make dev-daw                 # Launch DAW in dev mode (port 5174)
make dev-both                # Launch both applications

# Code Quality
make format                  # Format all code (Rust + TypeScript)
make lint                    # Run all linters
make check                   # Format + lint + test

# Testing
make test                    # Run all tests
make test-rust              # Rust tests only
make test-frontend          # Frontend tests only

# Building
make build-pipeline          # Build pipeline for production
make build-daw               # Build DAW for production
make build-all               # Build both applications
make release                 # Build optimized release bundles

# Database
make db-migrate              # Run database migrations
make db-backup               # Backup database
make db-reset                # ⚠️ DESTRUCTIVE: Reset database
make docker-logs             # View database logs

# Cleanup
make clean                   # Clean build artifacts
make clean-all               # Clean everything (build + cache)
```

## Important Implementation Notes

### Rust Workspace
- The project uses a Cargo workspace defined in root `Cargo.toml`
- Members: `pipeline/src-tauri`, `daw/src-tauri`, `shared/rust`, `scripts/import-tool`
- Shared dependencies defined at workspace level
- Run `cargo build` from root to build all members

### Tauri Development
- Each Tauri app has two parts: frontend (`src/`) and backend (`src-tauri/`)
- Frontend communicates with backend via `#[tauri::command]` functions
- Backend has direct access to filesystem, database, and native APIs
- Use `invoke()` from `@tauri-apps/api/core` to call Rust commands from TypeScript

### Database Schema
- Migrations in `database/migrations/` (numbered 001-006)
- Always create new migrations rather than editing existing ones
- Schema optimized for 3M+ files with proper indexes
- Uses pgvector extension for semantic similarity search
- Meilisearch for full-text search

### Frontend Architecture
- Svelte components use reactive declarations (`$:`)
- Stores in `src/lib/stores/` for shared state
- TypeScript strict mode enabled
- Vite handles builds and HMR

### Performance Optimizations
- Cargo.toml has optimized build profiles:
  - Dev: O0 for your code, O3 for dependencies (fast compile, fast run)
  - Release: O3 + thin LTO + strip (maximum optimization)
- Parallel processing with rayon
- Batch database inserts (up to 500 files/batch)
- Memory-mapped file I/O with memmap2

### Scripts Organization
- `scripts/launch/` - Service startup scripts
- `scripts/import/` - MIDI import automation
- `scripts/verify/` - Integration and quick check scripts
- `scripts/setup/` - Installation and configuration
- `scripts/import-tool/` - CLI Rust binary for bulk import

## Critical Warnings

### Destructive Operations
```bash
⚠️  make db-reset          # DELETES ALL DATABASE DATA
⚠️  make clean-all         # REMOVES ALL BUILD ARTIFACTS
⚠️  docker-compose down -v # REMOVES DOCKER VOLUMES
```

**Always backup before destructive operations:**
```bash
make db-backup              # Create backup first
```

### Dependencies Required
- **Docker** + Docker Compose 3.8+ (for database)
- **Rust 1.70+** (for backend compilation)
- **Node.js 18+** + pnpm 8+ (for frontend)
- **PostgreSQL client tools** (for direct DB access)
- **Platform-specific Tauri deps:**
  - Linux: webkit2gtk-4.0, libayatana-appindicator3, librsvg2
  - See VSCODE_SETUP_GUIDE.md for complete list

### Build Times (After Migration)
- First build: 10-15 minutes (Rust dependencies)
- Incremental builds: 30s-2min
- Dev builds (`make build`): faster, unoptimized
- Release builds (`make release`): 2-3 min, maximum optimization

## Project Context

This project is currently in the **migration preparation phase**. The original codebase has been thoroughly analyzed:

**Deduplication Results:**
- Original archive had ~500 files
- Identified source of truth: `midi-library-system/` (root directory)
- Removed ~278 duplicate/outdated files
- Final migration set: 222 production-ready files

**Code Quality Assessment:**
- ⭐⭐⭐⭐⭐ Database schema
- ⭐⭐⭐⭐⭐ MIDI parser
- ⭐⭐⭐⭐⭐ Build system
- ⭐⭐⭐⭐ Sequencer engine
- ⭐⭐⭐⭐ Frontend components
- ⭐⭐⭐ Scripts (need modernization)

**Migration Status:**
- ✅ Planning complete
- ✅ Deduplication complete
- ✅ Component separation defined
- ✅ Source of truth identified
- ⏳ Ready to begin file migration
- ❌ Migration not yet started

## Useful Documentation

**Architecture Documents (MANDATORY - Read First):**
- `ARCHITECTURE-REFERENCE.md` - Three Archetypes Pattern guide (Trusty Modules, Grown-up Scripts, Task-O-Matics)
- `PROJECT-STRUCTURE.md` - Complete directory map and file placement rules
- `DEVELOPMENT-WORKFLOW.md` - 8-step feature implementation process and code review checklist
- `RESTRUCTURE-TOPICS-SUMMARY.md` - **NEW:** Comprehensive summary of all topics from restructure.txt conversation
- `PRE-MIGRATION-ALIGNMENT-RECOMMENDATION.md` - Analysis of alignment gaps and recommendations
- `CRITICAL-REQUIREMENTS-ADDENDUM.md` - Mandatory code quality requirements (80% coverage, no .unwrap(), docs)

**Migration Documents (Essential for Migration):**
- `FINAL-FILE-SEPARATION.md` - Complete source→destination mapping, 222 files categorized
- `MIGRATION-DECISIONS.md` - Keep/adapt/recode decisions for all files
- `RESTRUCTURE-ALIGNMENT-CHECK.md` - Verification that migration plan aligns with original requirements
- `ORIGINAL-PROJECT-ANALYSIS.md` - Comprehensive code analysis (if exists)

**Planning Documents (Background):**
- `ANALYSIS_SUMMARY.md` - Project assessment and overview
- `QUICK_REFERENCE.md` - Planned operations and commands guide
- `RECOMMENDED_PROJECT_STRUCTURE.md` - Target directory structure
- `RESTRUCTURING_GAMEPLAN.md` - Strategic 5-phase implementation roadmap

**Setup Scripts:**
- `setup-project-structure.sh` - Creates empty directory structure
- `create-structure.sh` - Lightweight folder creation

## Working with This Codebase

**Important:** This project is in migration phase. When migrating the code:

### Migration Workflow
1. **Extract source archive** to `/tmp/original-project/`
2. **Use ONLY** `midi-library-system/` directory (not `projects/midi-library-system/`)
3. **Follow** `FINAL-FILE-SEPARATION.md` for exact source→destination paths
4. **Verify** each component after copying (compilation, tests)
5. **DO NOT** copy from `docs-recovered/` or `projects/` directories

### Component Integrity Rules
1. Shared library is complete - do not modify structure
2. DAW has its own MIDI modules (for real-time) - do not merge with shared
3. Pipeline and DAW must remain independent
4. Database is standalone - no code dependencies

### Post-Migration Development
1. Use the Makefile for all common tasks
2. Run `make format` before commits
3. Run `make check` before pull requests
4. Test both apps: `make dev-both`
5. Document any architectural changes

### Code Style
- **Rust**: Follow rustfmt (configured in Cargo.toml)
- **TypeScript/JavaScript**: Use TypeScript strict mode
- **Svelte**: Component file naming: PascalCase (e.g., `PianoRoll.svelte`)
- **Shell scripts**: Use error handling (`set -e`), follow existing patterns

## Getting Help

**For Migration Guidance:**
1. Read `FINAL-FILE-SEPARATION.md` for file-by-file mapping
2. Check `MIGRATION-DECISIONS.md` for rationale on each decision
3. Verify source of truth: `/tmp/original-project/midi-library-system/`
4. Follow the 5-phase migration plan (database → shared → backend → frontend → scripts)

**For Technical Architecture:**
1. Review original Cargo.toml for workspace structure
2. Check shared library structure in `shared/rust/src/`
3. Examine existing Makefile for build targets
4. Review docker-compose.yml for database setup

**For Code Understanding:**
1. MIDI parsing: `shared/rust/src/core/midi/parser.rs` (921 lines, well-documented)
2. Sequencer: `daw/src-tauri/src/sequencer/engine.rs` (800+ lines)
3. Batch import: `pipeline/src-tauri/src/commands/file_import.rs`
4. PianoRoll UI: `daw/src/lib/components/PianoRoll.svelte` (800+ lines)

**Migration Priority:**
1. Phase 1: Database + Shared + Root configs (foundation)
2. Phase 2: Backend (both apps + CLI tool)
3. Phase 3: Frontend (both apps)
4. Phase 4: Scripts (launch, verify, setup)
5. Phase 5: Final verification and documentation

## MCP Server Configuration

**This project has a comprehensive MCP (Model Context Protocol) setup configured for optimal development workflow.**

### Configured MCP Servers (12 Total)

#### Core Development Infrastructure (stdio transport)
- **postgres** - Database operations (schema inspection, migrations, queries)
  - Connection: `postgresql://midiuser:145278963@localhost:5433/midi_library`
  - Use: Query database, verify migrations, inspect schema

- **filesystem** - File operations (migration, copying, verification)
  - Paths: `/home/dojevou/projects/midi-software-center`, `/tmp/original-project`
  - Use: Copy 240 source files systematically during migration

- **docker** - Container management (monitoring, logs, health checks)
  - Use: Monitor PostgreSQL/Meilisearch containers, view logs, restart services

- **git** - Version control (commits, branches, history)
  - Use: Track migration progress, create commits, manage branches

- **bash** - Shell execution (automation, scripts, builds)
  - Use: Run build scripts, execute tests, automation tasks

#### Language & Framework Tooling (stdio transport)
- **rust** - Rust development (cargo, analysis, builds)
  - **CRITICAL** for this project (113 Rust files to migrate)
  - Use: cargo build, cargo test, cargo clippy, workspace management

- **npm** - Package management (dependencies, scripts)
  - Use: pnpm install, run dev/build scripts for Svelte frontends

- **vscode** - Editor integration (code navigation, file management)
  - Use: Open files, navigate to definitions, workspace management

#### Advanced Development (stdio transport)
- **web-search** - Web search (documentation, research, solutions)
  - Use: Look up Rust/Tauri/MIDI docs, find solutions to issues

- **anthropic** - AI assistance (Claude API access)
  - Use: Advanced AI-powered development tasks

#### Cloud Services (HTTP transport)
- **sentry** - Error monitoring (production error tracking)
  - URL: https://sentry.io
  - Use: Monitor runtime errors post-migration (requires OAuth setup)

- **notion** - Project management (documentation, planning, tracking)
  - URL: https://api.notion.com
  - Use: Track migration progress, document decisions (requires OAuth setup)

### MCP Benefits for Migration

**Why this MCP setup is valuable:**

1. **Database Access** - Direct PostgreSQL queries for verifying migrations and data
2. **File Operations** - Structured copying of 240 files with verification
3. **Container Management** - Monitor database health during migration
4. **Version Control** - Track every migration step with proper git commits
5. **Rust Tooling** - Essential for building/testing 113 Rust files
6. **Package Management** - Handle frontend dependencies (Svelte/TypeScript)
7. **Research** - Quick access to documentation for Rust/Tauri/MIDI
8. **Project Tracking** - Document migration decisions and progress

### Using MCP Servers

MCP servers activate automatically when needed. Examples:

```bash
# Database queries (postgres MCP)
"Show me all tables in the database"
"What's the schema for the files table?"

# File operations (filesystem MCP)
"Copy all files from shared/rust to the new structure"
"Verify that all source files were copied correctly"

# Container management (docker MCP)
"Show PostgreSQL container logs"
"Check if Meilisearch is healthy"

# Git operations (git MCP)
"Create a commit for Phase 1 migration"
"Show recent commit history"

# Rust development (rust MCP)
"Run cargo build for the workspace"
"Check for compilation errors in shared library"
```

### MCP Configuration Location

MCP servers are configured in: `~/.claude.json` (local scope for this project)

To view all configured servers:
```bash
claude mcp list
```

## Next Steps

**To Begin Migration:**
1. Verify source archive is extracted: `/tmp/original-project/`
2. Review `FINAL-FILE-SEPARATION.md` migration plan
3. Start with Phase 1 (Foundation):
   ```bash
   cp -r /tmp/original-project/midi-library-system/database/* database/
   cp -r /tmp/original-project/midi-library-system/shared/rust/* shared/rust/
   cp /tmp/original-project/midi-library-system/Cargo.toml .
   cp /tmp/original-project/midi-library-system/Makefile .
   cp /tmp/original-project/midi-library-system/docker-compose.yml .
   ```
4. Test foundation:
   ```bash
   docker-compose up -d
   cargo build
   ```
5. Proceed to Phase 2 (Backend migration)

**Success Criteria:**
- ✅ All components compile: `cargo build --all`
- ✅ Database starts: `docker-compose ps` shows healthy containers
- ✅ Both dev servers start: `make dev-both`
- ✅ No broken imports or missing modules
- ✅ Core features work (MIDI import, playback, search)

The migration is well-planned and low-risk. The original code is high quality and production-ready. This is primarily an organizational restructuring rather than a rewrite.
