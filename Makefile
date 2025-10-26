# MIDI Library System - Makefile
# Common development commands

.PHONY: help setup dev build test clean docker-up docker-down format lint

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
	@echo ""
	@echo "Building:"
	@echo "  make build-pipeline - Build pipeline for production"
	@echo "  make build-daw      - Build DAW for production"
	@echo "  make build-all      - Build both applications"
	@echo ""
	@echo "Testing:"
	@echo "  make test           - Run all tests"
	@echo "  make test-rust      - Run Rust tests only"
	@echo "  make test-frontend  - Run frontend tests only"
	@echo ""
	@echo "Code Quality:"
	@echo "  make format         - Format all code"
	@echo "  make lint           - Lint all code"
	@echo "  make check          - Run all checks"
	@echo ""
	@echo "Database:"
	@echo "  make db-migrate     - Run database migrations"
	@echo "  make db-reset       - Reset database"
	@echo "  make db-backup      - Backup database"
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
	cd pipeline && pnpm lint
	cd daw && pnpm lint

check: format lint test
	@echo "All checks passed!"

#=============================================================================
# DATABASE
#=============================================================================

db-migrate:
	@echo "Running database migrations..."
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
# BENCHMARKS
#=============================================================================

bench:
	cd pipeline/src-tauri && cargo bench
	cd daw/src-tauri && cargo bench

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
