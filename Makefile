# MIDI Library System - Makefile
# Common development commands
#
# Usage: make <target>
# Run 'make help' to see all available targets

.PHONY: help setup dev build test clean docker-up docker-down format lint cc codememory pgo-build pgo-pipeline pgo-daw pgo-clean bench release docker-build docker-dev lint-frontend lint-fix format-frontend format-check validate

# Default target
help:
	@echo "MIDI Library System - Development Commands"
	@echo ""
	@echo "Setup:"
	@echo "  make setup          - Install all dependencies"
	@echo "  make docker-up      - Start database containers"
	@echo "  make docker-down    - Stop database containers"
	@echo ""
	@echo "Development:"
	@echo "  make dev-pipeline   - Run pipeline in dev mode"
	@echo "  make dev-daw        - Run DAW in dev mode"
	@echo "  make dev-cpu        - Run with CPU rendering (no GPU)"
	@echo ""
	@echo "Building:"
	@echo "  make build-pipeline - Build pipeline for production"
	@echo "  make build-daw      - Build DAW for production"
	@echo "  make build-all      - Build both applications"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build   - Build Docker image for pipeline tools"
	@echo "  make docker-dev     - Build development Docker image"
	@echo "  make docker-run     - Run pipeline in Docker (ARGS=...)"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests"
	@echo "  make test-rust      - Run Rust tests only"
	@echo "  make test-frontend  - Run frontend tests only"
	@echo "  make test-baseline  - Run baseline library tests (Phases 0-4)"
	@echo "  make test-coverage-baseline - Generate coverage report (baseline)"
	@echo "  make test-quick     - Run quick smoke tests"
	@echo ""
	@echo "Code Quality:"
	@echo "  make format         - Format all code (Rust + frontend)"
	@echo "  make format-frontend - Format frontend code only"
	@echo "  make format-check   - Check formatting without changes"
	@echo "  make lint           - Lint all code (Rust + frontend)"
	@echo "  make lint-frontend  - Lint frontend code only"
	@echo "  make lint-fix       - Auto-fix lint issues"
	@echo "  make typecheck      - TypeScript type checking"
	@echo "  make check          - Run format, lint, and test"
	@echo "  make check-all      - Full validation (no auto-format)"
	@echo "  make validate       - Full frontend validation"
	@echo ""
	@echo "Performance:"
	@echo "  make bench          - Run benchmarks"
	@echo "  make pgo-build      - Build with PGO (10-20% faster) - 15-25 min"
	@echo "  make pgo-pipeline   - PGO for pipeline only"
	@echo "  make pgo-daw        - PGO for DAW only"
	@echo "  make pgo-clean      - Clean PGO profiling data"
	@echo ""
	@echo "Database:"
	@echo "  make db-migrate     - Run database migrations"
	@echo "  make db-reset       - Reset database"
	@echo "  make db-backup      - Backup database"
	@echo "  make db-shell       - Open PostgreSQL shell"
	@echo ""
	@echo "Knowledge Management:"
	@echo "  make cc             - Launch Claude Code (unrestricted)"
	@echo "  make codememory     - Populate CodeMemory knowledge base"
	@echo ""
	@echo "Release:"
	@echo "  make release        - Build release versions"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make clean-all      - Clean everything"

#=============================================================================
# SETUP
#=============================================================================

setup:
	@echo "Installing Rust dependencies..."
	cd pipeline/src-tauri && cargo build
	cd daw/src-tauri && cargo build
	@echo "Installing Node dependencies..."
	cd pipeline && pnpm install
	cd daw && pnpm install
	@echo "Setup complete!"

#=============================================================================
# DOCKER
#=============================================================================

docker-up:
	docker-compose up -d postgres
	@echo "Waiting for database to be ready..."
	@sleep 5
	@echo "Database is ready!"

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f postgres

docker-build:
	@echo "Building Docker image for pipeline tools..."
	docker build -t midi-software-center .
	@echo "Docker image built: midi-software-center"
	@echo "Usage: docker run -v /path/to/midi:/data midi-software-center import /data"

docker-dev:
	@echo "Building development Docker image..."
	docker build --target development -t midi-software-center:dev .
	@echo "Development image built: midi-software-center:dev"

docker-run:
	docker run --rm -it \
		-v $(PWD)/midi-library:/data \
		-e DATABASE_URL="postgresql://midiuser:145278963@host.docker.internal:5433/midi_library" \
		midi-software-center $(ARGS)

#=============================================================================
# DEVELOPMENT
#=============================================================================

dev-pipeline:
	cd pipeline && pnpm tauri dev

dev-daw:
	cd daw && pnpm tauri dev

dev-both:
	@echo "Starting both applications..."
	@echo "Pipeline: http://localhost:5173"
	@echo "DAW: http://localhost:5174"
	@make -j2 dev-pipeline dev-daw

dev-cpu:
	@echo "🚀 Launching MIDI Software Center (CPU rendering mode)..."
	@echo "   Hardware acceleration: DISABLED"
	@echo "   Use this on systems without GPU"
	cd app && WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 LIBGL_ALWAYS_SOFTWARE=1 pnpm tauri dev

#=============================================================================
# BUILD
#=============================================================================

build-pipeline:
	cd pipeline && pnpm tauri build

build-daw:
	cd daw && pnpm tauri build

build-all: build-pipeline build-daw

#=============================================================================
# TESTING
#=============================================================================

test:
	@make test-rust
	@make test-frontend

test-rust:
	@echo "Running Rust tests..."
	cd pipeline/src-tauri && cargo test --all
	cd daw/src-tauri && cargo test --all

test-frontend:
	@echo "Running frontend tests..."
	cd pipeline && pnpm test
	cd daw && pnpm test

test-coverage:
	cd pipeline/src-tauri && cargo tarpaulin --out Html
	cd daw/src-tauri && cargo tarpaulin --out Html

# Phase 9: Baseline testing (library tests only - no integration tests)
test-baseline:
	@echo "Running baseline library tests (Phases 0-4)..."
	cargo test --workspace --lib -- --test-threads=1

# Phase 9: Baseline + coverage report
test-coverage-baseline:
	@echo "Generating coverage report for baseline tests..."
	cargo tarpaulin --workspace --lib --out Html --timeout 300 --exclude-files "*/migrations/*"

# Phase 9: Quick smoke tests
test-quick:
	@echo "Running quick smoke tests (library tests, excluding long tests)..."
	cargo test --workspace --lib -- --test-threads=1 --skip "integration" --skip "performance" --skip "stress"

#=============================================================================
# CODE QUALITY
#=============================================================================

format:
	@echo "Formatting Rust code..."
	cargo fmt --all
	@echo "Formatting TypeScript/Svelte code..."
	cd pipeline && pnpm format
	cd daw && pnpm format

lint:
	@echo "Linting Rust code..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "Linting TypeScript/Svelte code..."
	cd app && pnpm lint

lint-frontend:
	@echo "Linting frontend code only..."
	cd app && pnpm lint

lint-fix:
	@echo "Auto-fixing lint issues..."
	cd app && pnpm lint:fix

format-frontend:
	@echo "Formatting frontend code..."
	cd app && pnpm format

format-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check
	cd app && pnpm format:check

typecheck:
	@echo "Running TypeScript type checking..."
	cd app && pnpm typecheck

validate:
	@echo "Running full validation (lint, format, types, tests)..."
	cd app && pnpm validate

check: format lint test
	@echo "All checks passed!"

check-all: format-check lint typecheck test
	@echo "All validations passed!"

#=============================================================================
# DATABASE
#=============================================================================

db-migrate:
	@echo "Running database migrations with sqlx..."
	@cd database && sqlx migrate run --database-url postgresql://midiuser:145278963@localhost:5433/midi_library
	@echo "✅ Database migrations complete"

db-migrate-manual:
	@echo "Running database migrations manually (legacy method)..."
	docker-compose exec postgres psql -U midiuser -d midi_library -f /docker-entrypoint-initdb.d/001_schema.sql

db-reset:
	@echo "WARNING: This will delete all data!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		docker-compose down -v; \
		docker-compose up -d postgres; \
		sleep 5; \
		make db-migrate; \
	fi

db-backup:
	@echo "Backing up database..."
	docker-compose exec postgres pg_dump -U midiuser midi_library > backup_$$(date +%Y%m%d_%H%M%S).sql
	@echo "Backup complete!"

db-shell:
	docker-compose exec postgres psql -U midiuser -d midi_library

#=============================================================================
# CLEANUP
#=============================================================================

clean:
	@echo "Cleaning build artifacts..."
	rm -rf pipeline/build pipeline/.svelte-kit pipeline/node_modules
	rm -rf daw/build daw/.svelte-kit daw/node_modules
	cd pipeline/src-tauri && cargo clean
	cd daw/src-tauri && cargo clean

clean-all: clean
	@echo "Removing all generated files..."
	rm -rf target
	docker-compose down -v

#=============================================================================
# KNOWLEDGE MANAGEMENT
#=============================================================================

populate-knowledge:
	@echo "Populating CodeMemory knowledge base..."
	@claude "Read and analyze my MIDI Software Center project. Here are the key documents: \
	\
	CLAUDE.md - Complete project overview and status \
	ARCHITECTURE-REFERENCE.md - Three Archetypes Pattern \
	PROJECT-STRUCTURE.md - Directory organization \
	DEVELOPMENT-WORKFLOW.md - 8-step feature process \
	TEST-COVERAGE-PLAN.md - Testing strategy (51.2% → 100%) \
	FINAL-FILE-SEPARATION.md - 222 files migrated \
	UNWRAP-AUDIT-REPORT.md - Zero unwrap/expect/panic achievement \
	\
	Key facts: \
	- 222 files, ~53,000 lines of Rust/TypeScript \
	- PostgreSQL 16 + pgvector for 3M+ MIDI files \
	- Tauri 2.7 desktop apps (Pipeline + DAW) \
	- Test coverage: 51.2% (689 tests), targeting 100% \
	- Phase 4 in progress: Repository layer testing \
	\
	Please extract and store: \
	1. Architecture patterns (Three Archetypes) \
	2. Code quality requirements (80% coverage, zero unwraps) \
	3. Component separation rules (Shared/Pipeline/DAW) \
	4. Testing strategy and current progress \
	5. Common workflows and commands \
	6. Technology stack and dependencies \
	\
	This will serve as the foundation knowledge for future sessions."

codememory: populate-knowledge

# Launch Claude Code with updated CodeMemory knowledge base
# Note: Use 'make cc' instead of bare 'cc' to avoid conflict with C compiler
cc: codememory
	@echo "🚀 Launching Claude Code with updated knowledge (unrestricted mode)..."
	@echo "Project: ~/projects/midi-software-center"
	@cd ~/projects/midi-software-center && cc --dangerously-skip-permissions || \
		(echo "⚠️  Claude Code CLI not found. Installing..." && npm install -g @anthropic-ai/claude-code && cc --dangerously-skip-permissions)

#=============================================================================
# BENCHMARKS
#=============================================================================

bench:
	cd pipeline/src-tauri && cargo bench
	cd daw/src-tauri && cargo bench

#=============================================================================
# PROFILE-GUIDED OPTIMIZATION (PGO)
#=============================================================================

pgo-build:
	@echo "Building with Profile-Guided Optimization (10-20% speedup)..."
	@echo "This takes 15-25 minutes. See docs/PGO-GUIDE.md for details."
	@./scripts/pgo-build.sh all

pgo-pipeline:
	@echo "PGO build for pipeline component only..."
	@./scripts/pgo-build.sh pipeline

pgo-daw:
	@echo "PGO build for DAW component only..."
	@./scripts/pgo-build.sh daw

pgo-clean:
	@echo "Cleaning PGO profiling data..."
	@rm -rf pipeline/src-tauri/target/pgo-profile
	@rm -rf daw/src-tauri/target/pgo-profile
	@echo "PGO data cleaned. Standard builds will be used next."

#=============================================================================
# RELEASE
#=============================================================================

release: check
	@echo "Building release versions..."
	@make build-all
	@echo "Release builds complete!"
	@echo "Binaries are in:"
	@echo "  pipeline/src-tauri/target/release/bundle/"
	@echo "  daw/src-tauri/target/release/bundle/"
	@echo ""
	@echo "For 10-20% additional performance, try: make pgo-build"
	@echo "See docs/PGO-GUIDE.md for details."
