#!/bin/bash
#
# Phase 7C Frontend Optimization - Verification Script
#
# Verifies that all profiling files are present and properly structured
#

set -e

echo "================================================"
echo "Phase 7C Frontend Optimization - Verification"
echo "================================================"
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PROJECT_ROOT="/home/dojevou/projects/midi-software-center"
PROFILING_DIR="$PROJECT_ROOT/pipeline/src/lib/profiling"

# Track results
PASSED=0
FAILED=0

# Function to check file exists
check_file() {
    local file=$1
    local description=$2

    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $description"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} $description (MISSING)"
        ((FAILED++))
    fi
}

# Function to check line count
check_line_count() {
    local file=$1
    local min_lines=$2
    local description=$3

    if [ -f "$file" ]; then
        local lines=$(wc -l < "$file")
        if [ "$lines" -ge "$min_lines" ]; then
            echo -e "${GREEN}✓${NC} $description ($lines lines)"
            ((PASSED++))
        else
            echo -e "${YELLOW}⚠${NC} $description ($lines lines, expected >$min_lines)"
        fi
    else
        echo -e "${RED}✗${NC} $description (FILE MISSING)"
        ((FAILED++))
    fi
}

echo "1. Core Profiling Files"
echo "------------------------"
check_file "$PROFILING_DIR/performance.ts" "performance.ts"
check_file "$PROFILING_DIR/hooks.ts" "hooks.ts"
check_file "$PROFILING_DIR/vite-plugin.ts" "vite-plugin.ts"
check_file "$PROFILING_DIR/index.ts" "index.ts"
check_file "$PROFILING_DIR/example-optimized-component.svelte" "example-optimized-component.svelte"
check_file "$PROFILING_DIR/README.md" "README.md"
echo ""

echo "2. Line Count Verification"
echo "---------------------------"
check_line_count "$PROFILING_DIR/performance.ts" 1400 "performance.ts (target: 1500)"
check_line_count "$PROFILING_DIR/hooks.ts" 180 "hooks.ts (target: 200)"
check_line_count "$PROFILING_DIR/vite-plugin.ts" 280 "vite-plugin.ts (target: 300)"
check_line_count "$PROFILING_DIR/example-optimized-component.svelte" 280 "example-optimized-component.svelte (target: 300)"
echo ""

echo "3. Documentation Files"
echo "-----------------------"
check_file "$PROJECT_ROOT/PHASE-7C-FRONTEND-OPTIMIZATION-GUIDE.md" "Optimization Guide"
check_file "$PROJECT_ROOT/PHASE-7C-IMPLEMENTATION-SUMMARY.md" "Implementation Summary"
echo ""

echo "4. TypeScript Export Verification"
echo "-----------------------------------"

# Check for key exports in index.ts
if grep -q "BundleAnalyzer" "$PROFILING_DIR/index.ts"; then
    echo -e "${GREEN}✓${NC} BundleAnalyzer export found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} BundleAnalyzer export missing"
    ((FAILED++))
fi

if grep -q "RenderProfiler" "$PROFILING_DIR/index.ts"; then
    echo -e "${GREEN}✓${NC} RenderProfiler export found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} RenderProfiler export missing"
    ((FAILED++))
fi

if grep -q "useFullProfiler" "$PROFILING_DIR/index.ts"; then
    echo -e "${GREEN}✓${NC} useFullProfiler export found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} useFullProfiler export missing"
    ((FAILED++))
fi

if grep -q "bundleAnalyzerPlugin" "$PROFILING_DIR/index.ts"; then
    echo -e "${GREEN}✓${NC} bundleAnalyzerPlugin export found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} bundleAnalyzerPlugin export missing"
    ((FAILED++))
fi

echo ""

echo "5. Key Classes Verification"
echo "----------------------------"

# Check for key classes in performance.ts
if grep -q "export class BundleAnalyzer" "$PROFILING_DIR/performance.ts"; then
    echo -e "${GREEN}✓${NC} BundleAnalyzer class found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} BundleAnalyzer class missing"
    ((FAILED++))
fi

if grep -q "export class RenderProfiler" "$PROFILING_DIR/performance.ts"; then
    echo -e "${GREEN}✓${NC} RenderProfiler class found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} RenderProfiler class missing"
    ((FAILED++))
fi

if grep -q "export class StoreProfiler" "$PROFILING_DIR/performance.ts"; then
    echo -e "${GREEN}✓${NC} StoreProfiler class found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} StoreProfiler class missing"
    ((FAILED++))
fi

if grep -q "export class NetworkProfiler" "$PROFILING_DIR/performance.ts"; then
    echo -e "${GREEN}✓${NC} NetworkProfiler class found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} NetworkProfiler class missing"
    ((FAILED++))
fi

if grep -q "export class MemoryProfiler" "$PROFILING_DIR/performance.ts"; then
    echo -e "${GREEN}✓${NC} MemoryProfiler class found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} MemoryProfiler class missing"
    ((FAILED++))
fi

if grep -q "export class VirtualScrollHelper" "$PROFILING_DIR/performance.ts"; then
    echo -e "${GREEN}✓${NC} VirtualScrollHelper class found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} VirtualScrollHelper class missing"
    ((FAILED++))
fi

echo ""

echo "6. Hook Functions Verification"
echo "--------------------------------"

# Check for hooks
if grep -q "export function useRenderProfiler" "$PROFILING_DIR/hooks.ts"; then
    echo -e "${GREEN}✓${NC} useRenderProfiler hook found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} useRenderProfiler hook missing"
    ((FAILED++))
fi

if grep -q "export function useMemoryMonitor" "$PROFILING_DIR/hooks.ts"; then
    echo -e "${GREEN}✓${NC} useMemoryMonitor hook found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} useMemoryMonitor hook missing"
    ((FAILED++))
fi

if grep -q "export function useFPSMonitor" "$PROFILING_DIR/hooks.ts"; then
    echo -e "${GREEN}✓${NC} useFPSMonitor hook found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} useFPSMonitor hook missing"
    ((FAILED++))
fi

if grep -q "export function useFullProfiler" "$PROFILING_DIR/hooks.ts"; then
    echo -e "${GREEN}✓${NC} useFullProfiler hook found"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} useFullProfiler hook missing"
    ((FAILED++))
fi

echo ""

echo "7. Total Line Count"
echo "--------------------"
TOTAL_LINES=$(find "$PROFILING_DIR" -type f \( -name "*.ts" -o -name "*.svelte" -o -name "*.md" \) -exec wc -l {} + | tail -1 | awk '{print $1}')
echo -e "Total lines in profiling module: ${GREEN}$TOTAL_LINES${NC}"

if [ "$TOTAL_LINES" -ge 2300 ]; then
    echo -e "${GREEN}✓${NC} Line count target met (>2,300 lines)"
    ((PASSED++))
else
    echo -e "${YELLOW}⚠${NC} Line count below target ($TOTAL_LINES < 2,300)"
fi

echo ""

# Summary
echo "================================================"
echo "Verification Summary"
echo "================================================"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ Phase 7C Implementation Complete!${NC}"
    echo ""
    echo "Next Steps:"
    echo "1. Enable profiling in components:"
    echo "   import { useFullProfiler } from '\$lib/profiling';"
    echo ""
    echo "2. Add Vite plugin to vite.config.ts:"
    echo "   import { bundleAnalyzerPlugin } from './src/lib/profiling/vite-plugin';"
    echo ""
    echo "3. Build and view bundle report:"
    echo "   pnpm build"
    echo "   cat .bundle-analysis/bundle-report.md"
    echo ""
    echo "4. Read comprehensive guide:"
    echo "   cat PHASE-7C-FRONTEND-OPTIMIZATION-GUIDE.md"
    exit 0
else
    echo -e "${RED}✗ Verification failed with $FAILED errors${NC}"
    exit 1
fi
