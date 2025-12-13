#!/bin/bash
# Quick Verification Script for Tailwind v3 Fix
# Usage: ./verify-fix.sh

set -e

echo "========================================="
echo "Tailwind v3 Fix - Verification Script"
echo "========================================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0

# Check function - executes command directly without eval for security
check() {
    local test_name="$1"
    shift
    # Execute remaining arguments as command directly (no eval needed)
    if "$@" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC} - $test_name"
        ((PASS++))
        return 0
    else
        echo -e "${RED}❌ FAIL${NC} - $test_name"
        ((FAIL++))
        return 1
    fi
}

echo "🔍 Running verification checks..."
echo ""

# Check 1: Tailwind v3 installed
check "Tailwind v3 installed" bash -c 'pnpm list tailwindcss 2>/dev/null | grep -q "3.4.17"'

# Check 2: Config file exists
check "tailwind.config.js exists" test -f tailwind.config.js

# Check 3: PostCSS config correct
check "PostCSS config has tailwindcss plugin" grep -q 'tailwindcss' postcss.config.js

# Check 4: CSS has v3 directives
check "app.css has @tailwind directives" grep -q '@tailwind base' src/app.css

# Check 5: index.html has dark class
check "index.html has dark mode class" grep -q 'class="dark"' index.html

# Check 6: Custom colors in config
check "Custom colors defined in config" grep -q 'app-bg' tailwind.config.js

# Check 7: Backups created
check "Backup files created" bash -c 'test -f postcss.config.js.backup && test -f src/app.css.backup'

# Check 8: No v4 imports in CSS
if grep -q '@import "tailwindcss"' src/app.css 2>/dev/null; then
    echo -e "${RED}❌ FAIL${NC} - app.css still has v4 @import directive"
    ((FAIL++))
else
    echo -e "${GREEN}✅ PASS${NC} - app.css has correct v3 syntax"
    ((PASS++))
fi

echo ""
echo "========================================="
echo "Verification Results"
echo "========================================="
echo -e "${GREEN}Passed: $PASS${NC}"
echo -e "${RED}Failed: $FAIL${NC}"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run: pnpm dev"
    echo "  2. Open: http://localhost:5173/"
    echo "  3. Verify: Dark background and styled interface"
    echo ""
    exit 0
else
    echo -e "${RED}❌ Some checks failed!${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Review failed checks above"
    echo "  2. See: TAILWIND-V4-FIX-GUIDE.md"
    echo "  3. Run: ./fix-tailwind.sh (to re-run fix)"
    echo ""
    exit 1
fi
