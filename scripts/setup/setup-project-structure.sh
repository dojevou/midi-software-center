#!/bin/bash

################################################################################
# MIDI Software Center - Project Structure Setup Script
# 
# Creates complete folder hierarchy for midi-software-center project
# Location: ~/projects/midi-software-center
#
# Usage:
#   bash setup-project-structure.sh              # Normal setup
#   bash setup-project-structure.sh --dry-run    # Show what would be created
#   bash setup-project-structure.sh --verbose    # Show detailed output
#   bash setup-project-structure.sh --help       # Show help
#
# Features:
#   ✓ Creates 20+ organized directories
#   ✓ Creates .gitignore files
#   ✓ Creates README.md files in each section
#   ✓ Error handling and validation
#   ✓ Progress tracking
#   ✓ Dry-run mode
#   ✓ Verbose mode
#   ✓ Color-coded output
#
################################################################################

set -e

# ============================================================================
# COLORS & FORMATTING
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# ============================================================================
# CONFIGURATION
# ============================================================================

DRY_RUN=false
VERBOSE=false
PROJECT_ROOT="${HOME}/projects/midi-software-center"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ============================================================================
# FUNCTIONS
# ============================================================================

print_header() {
    echo -e "\n${BOLD}${BLUE}=================================================================================${NC}"
    echo -e "${BOLD}${BLUE}$1${NC}"
    echo -e "${BOLD}${BLUE}=================================================================================${NC}\n"
}

print_section() {
    echo -e "\n${BOLD}${CYAN}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_verbose() {
    if [ "$VERBOSE" = true ]; then
        echo -e "  ${CYAN}→${NC} $1"
    fi
}

show_help() {
    cat << EOF
${BOLD}MIDI Software Center - Project Structure Setup${NC}

${BOLD}Usage:${NC}
  bash setup-project-structure.sh [OPTIONS]

${BOLD}Options:${NC}
  --dry-run       Show what would be created without actually creating
  --verbose       Show detailed output for every action
  --help          Show this help message

${BOLD}Examples:${NC}
  # Normal setup (creates all folders)
  bash setup-project-structure.sh

  # Preview what will be created
  bash setup-project-structure.sh --dry-run

  # Detailed output
  bash setup-project-structure.sh --verbose

${BOLD}What it does:${NC}
  • Creates 20+ organized directories
  • Creates .gitignore files where needed
  • Creates README.md files for documentation
  • Validates project structure
  • Takes ~5-10 seconds

${BOLD}Project Structure:${NC}
  • Root configuration (Makefile, .env, etc.)
  • config/          - Centralized configuration
  • docs/            - Consolidated documentation
  • database/        - Database layer
  • scripts/         - Automation hub
  • shared/          - Reusable code
  • pipeline/        - Batch processor app
  • daw/             - Audio workstation app
  • infrastructure/  - DevOps
  • tests/           - Testing
  • backups/         - Backup storage

${BOLD}Location:${NC}
  Project root: ${PROJECT_ROOT}

EOF
    exit 0
}

# ============================================================================
# MAIN LOGIC
# ============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --dry-run)
                DRY_RUN=true
                VERBOSE=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help)
                show_help
                ;;
            *)
                print_error "Unknown option: $1"
                show_help
                ;;
        esac
    done

    # Print header
    print_header "🏗️  MIDI SOFTWARE CENTER - PROJECT STRUCTURE SETUP"

    if [ "$DRY_RUN" = true ]; then
        print_warning "DRY RUN MODE - No changes will be made"
        echo ""
    fi

    # Check if project root exists
    if [ ! -d "$PROJECT_ROOT" ]; then
        print_error "Project root not found: $PROJECT_ROOT"
        echo ""
        print_info "Creating project root directory..."
        if [ "$DRY_RUN" = false ]; then
            mkdir -p "$PROJECT_ROOT"
            print_success "Project root created"
        else
            print_verbose "Would create: $PROJECT_ROOT"
        fi
    else
        print_success "Project root exists: $PROJECT_ROOT"
    fi

    cd "$PROJECT_ROOT"
    print_info "Working directory: $(pwd)"

    echo ""

    # ========================================================================
    # CREATE FOLDER STRUCTURE
    # ========================================================================

    print_section "Creating Configuration Directory"
    create_directory "config" "Centralized project configuration"
    create_directory "config/.keep" "Placeholder"

    print_section "Creating Documentation Directory"
    create_directory "docs" "Consolidated documentation"
    create_directory "docs/api" "API documentation"
    create_directory "docs/architecture" "Architecture documentation"
    create_directory "docs/database" "Database documentation"
    create_directory "docs/guides" "How-to guides"
    create_directory "docs/workflows" "Common workflows"

    print_section "Creating Database Directory"
    create_directory "database" "Database layer"
    create_directory "database/migrations" "Schema migrations"
    create_directory "database/queries" "Utility queries"
    create_directory "database/seeds" "Sample data"
    create_directory "database/scripts" "Database helper scripts"
    create_directory "database/config" "Database configuration"

    print_section "Creating Scripts Directory"
    create_directory "scripts" "Automation and operations hub"
    create_directory "scripts/modules" "Reusable script modules"
    create_directory "scripts/tasks" "Task-o-matic tasks"
    create_directory "scripts/tasks/db" "Database tasks"
    create_directory "scripts/tasks/build" "Build tasks"
    create_directory "scripts/tasks/deploy" "Deployment tasks"
    create_directory "scripts/tasks/dev" "Development tasks"
    create_directory "scripts/tasks/test" "Testing tasks"
    create_directory "scripts/launch" "Launch scripts"
    create_directory "scripts/grown-up" "Production-grade scripts"
    create_directory "scripts/maintenance" "Maintenance scripts"
    create_directory "scripts/legacy" "Archived scripts"

    print_section "Creating Shared Code Directory"
    create_directory "shared" "Shared code and utilities"
    create_directory "shared/rust" "Shared Rust code"
    create_directory "shared/ui" "Shared UI components"
    create_directory "shared/types" "Shared TypeScript types"

    print_section "Creating Application Directories"
    create_directory "pipeline" "Batch processor application"
    create_directory "pipeline/tests" "Pipeline tests"
    create_directory "pipeline/docs" "Pipeline documentation"

    create_directory "daw" "Digital audio workstation"
    create_directory "daw/tests" "DAW tests"
    create_directory "daw/docs" "DAW documentation"

    print_section "Creating Infrastructure Directory"
    create_directory "infrastructure" "DevOps and deployment"
    create_directory "infrastructure/docker" "Docker configurations"
    create_directory "infrastructure/kubernetes" "Kubernetes configurations"
    create_directory "infrastructure/github" "GitHub workflows"
    create_directory "infrastructure/github/workflows" "GitHub CI/CD workflows"
    create_directory "infrastructure/nginx" "Nginx reverse proxy"

    print_section "Creating Testing Directory"
    create_directory "tests" "Testing infrastructure"
    create_directory "tests/integration" "Integration tests"
    create_directory "tests/e2e" "End-to-end tests"
    create_directory "tests/fixtures" "Test data and fixtures"
    create_directory "tests/fixtures/midi-files" "Sample MIDI files"

    print_section "Creating Backup Directory"
    create_directory "backups" "Backup storage"

    print_section "Creating VS Code Configuration"
    create_directory ".vscode" "VS Code settings"

    # ========================================================================
    # CREATE .gitignore FILES
    # ========================================================================

    print_section "Creating .gitignore Files"
    create_gitignore "backups" "Ignore all backup files"
    create_gitignore ".env.local" "Ignore local environment overrides"
    create_gitignore "config/.local" "Ignore local config overrides"

    # ========================================================================
    # CREATE README.md FILES
    # ========================================================================

    print_section "Creating README.md Files"
    
    create_readme "config/README.md" "Configuration" \
        "Central location for all project configuration files.

## Files
- defaults.conf - Default settings for all environments
- development.conf - Development environment overrides
- production.conf - Production environment overrides
- testing.conf - Testing environment overrides
- load-config.sh - Configuration loader script

## Usage
Source the configuration loader in scripts:
\`\`\`bash
source config/load-config.sh
\`\`\`"

    create_readme "docs/README.md" "Documentation" \
        "Master documentation index for the entire project.

## Structure
- INDEX.md - Master documentation index
- SETUP.md - Getting started guide
- ARCHITECTURE.md - System architecture
- api/ - API documentation
- architecture/ - Architecture docs
- database/ - Database documentation
- guides/ - How-to guides
- workflows/ - Common workflows

## Contributing
Keep docs organized by category and up-to-date."

    create_readme "database/README.md" "Database" \
        "Database layer configuration and scripts.

## Structure
- docker-compose.yml - Main database setup
- migrations/ - Schema migrations
- queries/ - Utility SQL queries
- seeds/ - Sample data
- scripts/ - Database helper scripts
- config/ - Database configuration

## Getting Started
\`\`\`bash
make docker-up          # Start database
make db-migrate         # Run migrations
make db-seed            # Seed sample data
\`\`\`"

    create_readme "scripts/README.md" "Scripts" \
        "Automation and operations hub.

## Structure
- modules/ - Reusable script functions (Trusty Modules)
- tasks/ - Task dispatcher commands (Task-O-Matic)
- launch/ - Application launch scripts
- grown-up/ - Production-grade scripts
- maintenance/ - Maintenance utilities
- legacy/ - Archived scripts

## Usage
\`\`\`bash
./task-o-matic.sh db:backup
./task-o-matic.sh build:all
./task-o-matic.sh deploy:prod
\`\`\`"

    create_readme "shared/README.md" "Shared Code" \
        "Shared code and utilities used by multiple applications.

## Structure
- rust/ - Shared Rust code (MIDI parser, database client)
- ui/ - Shared UI components (Svelte components)
- types/ - Shared TypeScript type definitions

## Guidelines
- Keep shared code DRY (Don't Repeat Yourself)
- Document public APIs
- Test thoroughly before using in multiple places"

    create_readme "pipeline/README.md" "Pipeline" \
        "Batch processor application.

## Stack
- Frontend: Svelte + TypeScript
- Backend: Rust + Tauri
- Build: Vite
- Database: PostgreSQL

## Getting Started
\`\`\`bash
cd pipeline
pnpm install
make dev-pipeline
\`\`\`

Visit: http://localhost:5173"

    create_readme "daw/README.md" "DAW" \
        "Digital Audio Workstation.

## Stack
- Frontend: Svelte + TypeScript
- Backend: Rust + Tauri
- MIDI I/O: Rust backend
- Sequencer: Rust backend
- Build: Vite
- Database: PostgreSQL

## Getting Started
\`\`\`bash
cd daw
pnpm install
make dev-daw
\`\`\`

Visit: http://localhost:5174"

    create_readme "infrastructure/README.md" "Infrastructure" \
        "DevOps and deployment configuration.

## Structure
- docker/ - Docker configurations
- kubernetes/ - Kubernetes deployments
- github/ - GitHub CI/CD workflows
- nginx/ - Reverse proxy configuration

## Deployment
\`\`\`bash
make deploy-prod
\`\`\`"

    create_readme "tests/README.md" "Testing" \
        "Testing infrastructure and test suites.

## Structure
- integration/ - Integration tests
- e2e/ - End-to-end tests
- fixtures/ - Test data and sample files

## Running Tests
\`\`\`bash
make test              # Run all tests
make test-rust         # Rust tests only
make test-frontend     # Frontend tests only
\`\`\`"

    create_readme "backups/README.md" "Backups" \
        "Backup storage for database and project files.

## Guidelines
- DO NOT commit backup files to git
- Use backup scripts for automated backups
- Keep versioned backups
- Test restore procedures regularly

## Commands
\`\`\`bash
make db-backup         # Create database backup
make db-restore        # Restore from backup
\`\`\`"

    # ========================================================================
    # VALIDATION
    # ========================================================================

    print_section "Validating Structure"
    validate_structure

    # ========================================================================
    # SUMMARY
    # ========================================================================

    print_section "Setup Complete!"
    
    if [ "$DRY_RUN" = true ]; then
        print_warning "Dry run completed - no changes were made"
    else
        print_success "Project structure created successfully"
    fi

    echo ""
    print_info "Project Root: $PROJECT_ROOT"
    echo ""
    print_info "Next Steps:"
    echo "  1. cd $PROJECT_ROOT"
    echo "  2. code .                    (Open in VS Code)"
    echo "  3. make setup                (Install dependencies)"
    echo "  4. make docker-up            (Start database)"
    echo "  5. make dev-both             (Start applications)"
    echo ""
    print_info "Documentation:"
    echo "  • Read: FILE_PLACEMENT_GUIDE.md"
    echo "  • Read: RECOMMENDED_PROJECT_STRUCTURE.md"
    echo "  • Read: QUICK_REFERENCE.md"
    echo ""

    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}To actually create the structure, run without --dry-run:${NC}"
        echo "  bash setup-project-structure.sh"
        echo ""
    fi
}

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

create_directory() {
    local dir=$1
    local description=$2

    print_verbose "Creating directory: $dir"

    if [ "$DRY_RUN" = false ]; then
        mkdir -p "$dir" 2>/dev/null || true
        if [ -d "$dir" ]; then
            print_success "$description: $dir"
        else
            print_error "Failed to create: $dir"
        fi
    else
        print_verbose "Would create: $dir ($description)"
    fi
}

create_gitignore() {
    local file=$1
    local description=$2

    print_verbose "Creating .gitignore for: $file"

    if [ "$DRY_RUN" = false ]; then
        case "$file" in
            "backups")
                cat > "$file/.gitignore" << 'EOF'
# Ignore all backup files
*
!.gitignore
!README.md
!README
EOF
                print_success "Created .gitignore: $file/"
                ;;
            ".env.local")
                cat > ".env.local" 2>/dev/null || true
                echo ".env.local" >> .gitignore 2>/dev/null || true
                print_success "Added to .gitignore: $file"
                ;;
            "config/.local")
                mkdir -p "config/.local" 2>/dev/null || true
                echo "*" > "config/.local/.gitignore" 2>/dev/null || true
                print_success "Created .gitignore: $file/"
                ;;
        esac
    else
        print_verbose "Would create .gitignore for: $file"
    fi
}

create_readme() {
    local filepath=$1
    local title=$2
    local content=$3

    print_verbose "Creating README: $filepath"

    if [ "$DRY_RUN" = false ]; then
        mkdir -p "$(dirname "$filepath")" 2>/dev/null || true
        cat > "$filepath" << EOF
# 📚 $title

$content
EOF
        print_success "Created README: $filepath"
    else
        print_verbose "Would create README: $filepath"
    fi
}

validate_structure() {
    local expected_dirs=(
        "config"
        "docs"
        "database"
        "scripts"
        "shared"
        "pipeline"
        "daw"
        "infrastructure"
        "tests"
        "backups"
    )

    local created=0
    local missing=0

    for dir in "${expected_dirs[@]}"; do
        if [ -d "$dir" ]; then
            ((created++))
        else
            ((missing++))
            print_warning "Missing: $dir"
        fi
    done

    echo ""
    print_info "Validation Results:"
    echo "  Created: $created/${#expected_dirs[@]} main directories"
    
    if [ $missing -eq 0 ]; then
        print_success "All directories created successfully!"
    else
        print_warning "$missing directories are missing"
    fi
}

# ============================================================================
# ENTRY POINT
# ============================================================================

if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi
