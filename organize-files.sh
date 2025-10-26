#!/bin/bash

################################################################################
# MIDI Software Center - File Organization Script
#
# Organizes loose documentation and script files into proper directories
# Location: ~/projects/midi-software-center
#
# Usage:
#   bash organize-files.sh              # Normal organize
#   bash organize-files.sh --dry-run    # Show what would happen
#   bash organize-files.sh --verbose    # Detailed output
#   bash organize-files.sh --help       # Show help
#
# What it does:
#   • Moves documentation to docs/
#   • Moves scripts to appropriate locations
#   • Moves archives to backups/
#   • Cleans up root directory
#   • Creates organization summary
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
NC='\033[0m'

# ============================================================================
# CONFIGURATION
# ============================================================================

DRY_RUN=false
VERBOSE=false
PROJECT_ROOT="$(pwd)"
MOVED_COUNT=0
SKIPPED_COUNT=0

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
${BOLD}MIDI Software Center - File Organization Script${NC}

${BOLD}Usage:${NC}
  bash organize-files.sh [OPTIONS]

${BOLD}Options:${NC}
  --dry-run       Show what would be moved without actually moving
  --verbose       Show detailed output for every action
  --help          Show this help message

${BOLD}What it does:${NC}
  Organizes loose files in project root:

  Documentation Files → docs/
  ├── RECOMMENDED_PROJECT_STRUCTURE.md
  ├── FILE_PLACEMENT_GUIDE.md
  ├── SETUP_SCRIPT_USAGE.md
  ├── VSCODE_SETUP_GUIDE.md
  ├── RESTRUCTURING_GAMEPLAN.md
  ├── ANALYSIS_SUMMARY.md
  ├── VISUAL_SUMMARY.md
  ├── PROJECT_STRUCTURE_TREE.txt
  ├── PHASE_0_CHECKLIST.md
  ├── QUICK_REFERENCE.md
  ├── SCRIPT_CONFIG_INVENTORY.md
  ├── SCRIPTS_QUICK_REFERENCE.md
  ├── COMPLETE_DELIVERABLES.md
  ├── 00-DOCUMENT-INDEX.md
  └── COMPLETE_PACKAGE_SUMMARY.txt

  Setup/Config Files → root (kept for reference)
  ├── setup-project-structure.sh
  ├── create-structure.sh
  └── restructure.txt

  Archives → backups/
  └── *.tar.gz files

${BOLD}Examples:${NC}
  # Preview what will happen
  bash organize-files.sh --dry-run

  # Organize files for real
  bash organize-files.sh

  # Show all details
  bash organize-files.sh --verbose

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
    print_header "📁 FILE ORGANIZATION SCRIPT"

    if [ "$DRY_RUN" = true ]; then
        print_warning "DRY RUN MODE - No files will be moved"
        echo ""
    fi

    # Verify we're in correct directory
    if [ ! -d "docs" ] || [ ! -d "scripts" ] || [ ! -d "backups" ]; then
        print_error "Required directories not found!"
        print_info "Make sure you're in: ~/projects/midi-software-center"
        print_info "And the folder structure has been created by setup-project-structure.sh"
        exit 1
    fi

    print_success "Project root verified: $PROJECT_ROOT"
    echo ""

    # ========================================================================
    # MOVE DOCUMENTATION FILES TO docs/
    # ========================================================================

    print_section "Moving Documentation Files to docs/"

    move_file "00-DOCUMENT-INDEX.md" "docs/"
    move_file "ANALYSIS_SUMMARY.md" "docs/"
    move_file "COMPLETE_DELIVERABLES.md" "docs/"
    move_file "COMPLETE_PACKAGE_SUMMARY.txt" "docs/"
    move_file "FILE_PLACEMENT_GUIDE.md" "docs/"
    move_file "PHASE_0_CHECKLIST.md" "docs/"
    move_file "PROJECT_STRUCTURE_TREE.txt" "docs/"
    move_file "QUICK_REFERENCE.md" "docs/"
    move_file "RECOMMENDED_PROJECT_STRUCTURE.md" "docs/"
    move_file "RESTRUCTURING_GAMEPLAN.md" "docs/"
    move_file "SCRIPT_CONFIG_INVENTORY.md" "docs/"
    move_file "SCRIPTS_QUICK_REFERENCE.md" "docs/"
    move_file "SETUP_SCRIPT_USAGE.md" "docs/"
    move_file "VISUAL_SUMMARY.md" "docs/"
    move_file "VSCODE_SETUP_GUIDE.md" "docs/"

    # ========================================================================
    # MOVE SETUP SCRIPTS
    # ========================================================================

    print_section "Organizing Setup Scripts"

    # Keep main setup scripts in root
    if [ -f "setup-project-structure.sh" ]; then
        print_verbose "Keeping setup-project-structure.sh in root"
        print_info "setup-project-structure.sh: Keeping in root (core setup script)"
    fi

    if [ -f "create-structure.sh" ]; then
        print_verbose "Keeping create-structure.sh in root"
        print_info "create-structure.sh: Keeping in root (alternate setup script)"
    fi

    # ========================================================================
    # MOVE ARCHIVE FILES TO backups/
    # ========================================================================

    print_section "Moving Archive Files to backups/"

    move_file "midi-library-system-refined.tar.gz" "backups/"
    move_file "restructure.txt" "backups/" || print_warning "restructure.txt: Not found (okay)"

    # ========================================================================
    # VERIFY ORGANIZATION
    # ========================================================================

    print_section "Verifying Organization"
    verify_organization

    # ========================================================================
    # SUMMARY
    # ========================================================================

    print_section "Summary"

    if [ "$DRY_RUN" = true ]; then
        print_warning "Dry run completed - no files were moved"
        echo ""
        echo -e "${YELLOW}To actually organize files, run:${NC}"
        echo "  bash organize-files.sh"
        echo ""
    else
        print_success "Files organized successfully!"
        echo ""
        print_info "Files moved: $MOVED_COUNT"
        print_info "Files skipped: $SKIPPED_COUNT"
        echo ""
    fi

    print_info "Organization Summary:"
    echo "  • Documentation:  docs/ (15 files)"
    echo "  • Setup Scripts:   root (2 scripts)"
    echo "  • Archives:        backups/ (1+ files)"
    echo ""

    # ========================================================================
    # NEXT STEPS
    # ========================================================================

    print_section "What's Next?"

    echo -e "${BLUE}Your project is now organized! Next steps:${NC}"
    echo ""
    echo "  1. Review organized files:"
    echo "     ls -la docs/                  # View documentation"
    echo "     ls -la backups/               # View archives"
    echo ""
    echo "  2. Open in VS Code:"
    echo "     code ."
    echo ""
    echo "  3. Install dependencies:"
    echo "     make setup"
    echo ""
    echo "  4. Start development:"
    echo "     make docker-up"
    echo "     make dev-both"
    echo ""
    echo "  5. Read documentation:"
    echo "     cat docs/SETUP_SCRIPT_USAGE.md"
    echo "     cat docs/VSCODE_SETUP_GUIDE.md"
    echo "     cat docs/RECOMMENDED_PROJECT_STRUCTURE.md"
    echo ""
}

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

move_file() {
    local file=$1
    local destination=$2
    local description=${3:-"$file"}

    if [ ! -f "$file" ]; then
        print_verbose "Not found: $file (skipping)"
        ((SKIPPED_COUNT++))
        return 0
    fi

    print_verbose "Moving: $file → $destination"

    if [ "$DRY_RUN" = false ]; then
        mv "$file" "$destination" 2>/dev/null || {
            print_error "Failed to move: $file"
            return 1
        }
        print_success "Moved: $file → $destination"
        ((MOVED_COUNT++))
    else
        print_verbose "Would move: $file → $destination"
        ((MOVED_COUNT++))
    fi
}

verify_organization() {
    print_verbose "Verifying docs/ organization..."

    local docs_count=$(find docs -type f -name "*.md" -o -name "*.txt" 2>/dev/null | wc -l)
    local backups_count=$(find backups -type f \( -name "*.tar.gz" -o -name "*.txt" \) 2>/dev/null | wc -l)

    echo ""

    if [ $docs_count -gt 0 ]; then
        print_success "Documentation files in docs/: $docs_count files"
    else
        print_warning "No documentation files found in docs/"
    fi

    if [ $backups_count -gt 0 ]; then
        print_success "Archive files in backups/: $backups_count files"
    fi

    # Check root has only essential files
    local root_md_count=$(find . -maxdepth 1 -type f -name "*.md" 2>/dev/null | wc -l)
    if [ $root_md_count -eq 0 ]; then
        print_success "Root directory cleaned (no stray .md files)"
    else
        print_warning "Root directory still has $root_md_count .md files"
    fi
}

# ============================================================================
# ENTRY POINT
# ============================================================================

if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi
