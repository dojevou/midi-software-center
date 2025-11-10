#!/bin/bash
# Create recommended folder structure for midi-software-center
# Usage: bash create-structure.sh

set -e

echo "🏗️  Creating MIDI Software Center Project Structure"
echo "=================================================="

# Root level
echo "📄 Creating root level directories..."
mkdir -p .github/workflows
mkdir -p .vscode

# Configuration
echo "⚙️  Creating config directory..."
mkdir -p config

# Documentation
echo "📚 Creating docs directory..."
mkdir -p docs/{api,architecture,database,guides,workflows}

# Database
echo "🗄️  Creating database directory..."
mkdir -p database/{migrations,queries,seeds,scripts,config}

# Scripts (the heart of automation)
echo "🔧 Creating scripts directory..."
mkdir -p scripts/{modules,tasks/{db,build,deploy,dev,test},launch,grown-up,maintenance,legacy}

# Shared code
echo "🗂️  Creating shared directory..."
mkdir -p shared/{rust,ui,types}

# Applications
echo "🚀 Creating application directories..."
# Pipeline already exists but ensure structure
mkdir -p pipeline/tests
mkdir -p pipeline/docs
# DAW already exists but ensure structure
mkdir -p daw/tests
mkdir -p daw/docs

# Infrastructure
echo "⚡ Creating infrastructure directory..."
mkdir -p infrastructure/{docker,kubernetes,github/workflows,nginx}

# Tests
echo "📊 Creating tests directory..."
mkdir -p tests/{integration,e2e,fixtures/midi-files}

# Backups
echo "🔒 Creating backups directory..."
mkdir -p backups

echo ""
echo "✅ Directory structure created!"
echo ""
echo "Next steps:"
echo "1. Review RECOMMENDED_PROJECT_STRUCTURE.md"
echo "2. Create config files: touch config/{defaults,development,production,testing}.conf"
echo "3. Move existing scripts to scripts/launch/"
echo "4. Consolidate docs to docs/"
echo "5. Start Phase 0 preparation"
echo ""
echo "Happy building! 🎉"
