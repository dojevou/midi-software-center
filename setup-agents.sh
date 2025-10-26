#!/bin/bash
# Claude Code Agent Setup Script for MIDI Software Center
# This script installs specialized agents for the project

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Agent files
AGENTS=(
    "rust-backend-agent.toml"
    "frontend-agent.toml"
    "architecture-reviewer-agent.toml"
    "database-agent.toml"
    "midi-hardware-agent.toml"
)

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Claude Code Agent Setup${NC}"
    echo -e "${BLUE}  MIDI Software Center Project${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check if agent files exist
    local missing=0
    for agent in "${AGENTS[@]}"; do
        if [ ! -f "$agent" ]; then
            print_error "Missing agent file: $agent"
            missing=1
        fi
    done
    
    if [ $missing -eq 1 ]; then
        print_error "Some agent files are missing. Please ensure all .toml files are present."
        exit 1
    fi
    
    print_success "All agent files found"
}

install_personal() {
    print_info "Installing agents to personal directory (~/.claude/agents/)..."
    
    local personal_dir="$HOME/.claude/agents"
    mkdir -p "$personal_dir"
    
    for agent in "${AGENTS[@]}"; do
        cp "$agent" "$personal_dir/"
        print_success "Installed $agent"
    done
    
    print_success "Personal agents installed successfully!"
    print_info "Agents are now available in all your Claude Code projects"
}

install_project() {
    print_info "Installing agents to project directory (.claude/agents/)..."
    
    local project_dir=".claude/agents"
    mkdir -p "$project_dir"
    
    for agent in "${AGENTS[@]}"; do
        cp "$agent" "$project_dir/"
        print_success "Installed $agent"
    done
    
    print_success "Project agents installed successfully!"
    print_info "Agents are now available in this project"
    print_info "You can commit these to git for team sharing"
}

show_menu() {
    echo ""
    echo -e "${YELLOW}Where would you like to install the agents?${NC}"
    echo ""
    echo "1) Personal directory (~/.claude/agents/)"
    echo "   - Available in ALL your projects"
    echo "   - Only you will have these agents"
    echo ""
    echo "2) Project directory (.claude/agents/)"
    echo "   - Available only in THIS project"
    echo "   - Can be committed to git for team sharing"
    echo ""
    echo "3) Both locations"
    echo "   - Install everywhere"
    echo ""
    echo "4) Show agent information"
    echo ""
    echo "5) Exit"
    echo ""
}

show_agent_info() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Available Agents${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    echo -e "${GREEN}1. rust-backend${NC}"
    echo "   Backend Rust development, Tauri commands, async operations"
    echo "   Use for: Tauri commands, services, core logic, MIDI I/O"
    echo ""
    
    echo -e "${GREEN}2. frontend${NC}"
    echo "   Svelte/TypeScript UI development, stores, components"
    echo "   Use for: Svelte components, stores, utilities, types"
    echo ""
    
    echo -e "${GREEN}3. architecture-reviewer${NC}"
    echo "   Code review, architectural compliance, Three Archetypes"
    echo "   Use for: Reviewing code, checking placement, enforcing rules"
    echo ""
    
    echo -e "${GREEN}4. database${NC}"
    echo "   PostgreSQL schema design, migrations, repositories"
    echo "   Use for: Migrations, repositories, queries, transactions"
    echo ""
    
    echo -e "${GREEN}5. midi-hardware${NC}"
    echo "   MIDI processing, hardware integration, ALSA"
    echo "   Use for: MIDI parsing, hardware devices, BPM detection"
    echo ""
    
    echo -e "${BLUE}========================================${NC}"
    echo ""
    print_info "See AGENT-SETUP-GUIDE.md for detailed usage instructions"
    echo ""
}

show_next_steps() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Next Steps${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    print_info "1. Read the setup guide:"
    echo "   cat AGENT-SETUP-GUIDE.md"
    echo ""
    
    print_info "2. In Claude Code, use agents with:"
    echo "   > /agents                    # List available agents"
    echo "   > /agent rust-backend        # Select an agent"
    echo "   > /agent frontend            # Switch agents"
    echo ""
    
    print_info "3. Start with architecture-reviewer to understand patterns:"
    echo "   > /agent architecture-reviewer"
    echo "   'Explain the Three Archetypes pattern'"
    echo ""
    
    print_info "4. Example workflows:"
    echo "   - New backend feature: rust-backend → architecture-reviewer"
    echo "   - Database changes: database → rust-backend → architecture-reviewer"
    echo "   - Full-stack feature: database → rust-backend → frontend → architecture-reviewer"
    echo ""
    
    if [ -d ".claude/agents" ]; then
        print_info "5. Project agents can be committed to git:"
        echo "   git add .claude/agents/"
        echo "   git commit -m 'Add Claude Code agents'"
        echo ""
    fi
    
    print_success "You're all set! Happy coding with Claude Code!"
    echo ""
}

main() {
    print_header
    check_prerequisites
    
    while true; do
        show_menu
        read -p "Enter your choice [1-5]: " choice
        
        case $choice in
            1)
                install_personal
                show_next_steps
                break
                ;;
            2)
                install_project
                show_next_steps
                break
                ;;
            3)
                install_personal
                echo ""
                install_project
                show_next_steps
                break
                ;;
            4)
                show_agent_info
                ;;
            5)
                echo ""
                print_info "Installation cancelled"
                exit 0
                ;;
            *)
                print_error "Invalid choice. Please enter 1-5."
                ;;
        esac
    done
}

# Run main function
main
