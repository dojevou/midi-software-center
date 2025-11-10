#!/bin/bash
# Grok Project Reviewer Wrapper
# Configured for: /home/dojevou/projects/midi-software-center

set -e

PROJECT_ROOT="/home/dojevou/projects/midi-software-center"
SCRIPT="$PROJECT_ROOT/grok4_project_reviewer.py"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if API key is set
check_api_key() {
    if [ -z "$GROK_API_KEY" ]; then
        echo -e "${RED}❌ Error: GROK_API_KEY not set${NC}"
        echo ""
        echo -e "${YELLOW}Setup instructions:${NC}"
        echo ""
        echo "1. Generate a new API key at: https://console.x.ai/api"
        echo ""
        echo "2. Set the environment variable:"
        echo -e "   ${CYAN}export GROK_API_KEY='xai-your-key-here'${NC}"
        echo ""
        echo "3. Optional: Add to ~/.bashrc for persistence"
        echo ""
        exit 1
    fi
}

# Ensure dependencies
check_deps() {
    if ! command -v python3 &>/dev/null; then
        echo -e "${RED}❌ Python 3 not found${NC}"
        exit 1
    fi

    if ! python3 -c "import httpx" 2>/dev/null; then
        echo -e "${YELLOW}📦 Installing httpx...${NC}"
        pip3 install httpx --break-system-packages
    fi
}

# Run analysis
run_analysis() {
    local analysis=$1

    case $analysis in
    errors | compilation)
        echo -e "${BLUE}🔍 Analyzing compilation errors...${NC}"
        python3 "$SCRIPT" errors
        ;;
    tests | test)
        echo -e "${BLUE}🧪 Reviewing test files...${NC}"
        python3 "$SCRIPT" tests
        ;;
    full | audit | all)
        echo -e "${BLUE}🚀 Running full project audit...${NC}"
        python3 "$SCRIPT" full
        ;;
    *)
        echo -e "${BLUE}Grok Project Reviewer${NC}"
        echo ""
        echo "Usage: $0 [command]"
        echo ""
        echo -e "${GREEN}Commands:${NC}"
        echo "  errors       Analyze compilation errors"
        echo "  full         Run complete audit"
        echo ""
        exit 1
        ;;
    esac
}

# Main
main() {
    cd "$PROJECT_ROOT"

    check_deps
    check_api_key

    if [ $# -eq 0 ]; then
        run_analysis "help"
    else
        run_analysis "$1"
    fi
}

main "$@"
