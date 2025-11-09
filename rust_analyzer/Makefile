# Makefile for Quantum Analyzer

.PHONY: all build release test clean install help

# Default target
all: build

# Build debug version
build:
	@echo "🔨 Building debug version..."
	cargo build

# Build optimized release version
release:
	@echo "🚀 Building release version..."
	cargo build --release
	@echo "✅ Binary: target/release/quantum-analyzer"

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Run clippy
clippy:
	@echo "📎 Running clippy..."
	cargo clippy -- -D warnings

# Format code
fmt:
	@echo "✨ Formatting code..."
	cargo fmt

# Check formatting
fmt-check:
	@echo "🔍 Checking formatting..."
	cargo fmt -- --check

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	cargo clean

# Install as cargo plugin
install:
	@echo "📦 Installing as cargo plugin..."
	cargo install --path .
	@echo "✅ Use with: cargo quantum"

# Install to /usr/local/bin
install-system:
	@echo "📦 Installing to /usr/local/bin..."
	cargo build --release
	sudo cp target/release/quantum-analyzer /usr/local/bin/
	@echo "✅ Installed: /usr/local/bin/quantum-analyzer"

# Run on test project
demo:
	@echo "🎬 Running demo analysis..."
	cargo run -- --project . --verbose

# Run with autofix
demo-fix:
	@echo "🔧 Running demo with auto-fix..."
	cargo run -- --project . --autofix --verbose

# Generate docs
docs:
	@echo "📚 Generating documentation..."
	cargo doc --open

# Check everything
check-all: fmt-check clippy test
	@echo "✅ All checks passed!"

# Build and run
run:
	cargo run -- $(ARGS)

# Help
help:
	@echo "Quantum Analyzer - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make build          - Build debug version"
	@echo "  make release        - Build optimized release"
	@echo "  make test           - Run tests"
	@echo "  make clippy         - Run clippy linter"
	@echo "  make fmt            - Format code"
	@echo "  make fmt-check      - Check code formatting"
	@echo "  make clean          - Remove build artifacts"
	@echo "  make install        - Install as cargo plugin"
	@echo "  make install-system - Install to /usr/local/bin"
	@echo "  make demo           - Run demo analysis"
	@echo "  make demo-fix       - Run demo with auto-fix"
	@echo "  make docs           - Generate documentation"
	@echo "  make check-all      - Run all checks"
	@echo "  make run ARGS='...' - Build and run with args"
	@echo ""
	@echo "Examples:"
	@echo "  make release"
	@echo "  make run ARGS='--project /path/to/project --autofix'"
	@echo "  make install"
