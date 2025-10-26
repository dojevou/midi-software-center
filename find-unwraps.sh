#!/bin/bash
# Find All .unwrap() Calls in MIDI Library System
# Archetype: Task-O-Matic (Complete standalone task)
# Generates report of all .unwrap() calls that need fixing

set -e
set -u

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
GREEN='\033[0;32m'
NC='\033[0m'

echo -e "${BLUE}🔍 Searching for .unwrap() calls in Rust code...${NC}"
echo ""

# Output file
OUTPUT="unwrap-audit-$(date +%Y%m%d-%H%M%S).md"

# Start markdown report
cat > "$OUTPUT" << 'EOF'
# .unwrap() Audit Report
**Generated:** $(date)
**Purpose:** Identify all .unwrap() calls that need proper error handling

---

## Summary

EOF

# Count total unwraps
TOTAL_UNWRAPS=$(grep -r "\.unwrap()" --include="*.rs" . | grep -v "target/" | grep -v "\.git/" | grep -v "#\[cfg(test)\]" -A 5 | wc -l)

echo "**Total .unwrap() calls found:** $TOTAL_UNWRAPS" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# By Priority (Critical = core/, High = repositories/, Medium = commands/, Low = bin/)

echo "## By Priority" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Critical: Trusty Modules (core/, shared/src/, utils/)
echo "### 🚨 CRITICAL: Trusty Modules (NO .unwrap() allowed)" >> "$OUTPUT"
echo "" >> "$OUTPUT"
echo "These MUST be fixed - Trusty Modules should never have .unwrap():" >> "$OUTPUT"
echo "" >> "$OUTPUT"
grep -rn "\.unwrap()" --include="*.rs" ./shared/src ./database/src/models ./pipeline/src-tauri/src/core ./daw/src-tauri/src/core 2>/dev/null | grep -v "target/" | grep -v "#\[cfg(test)\]" >> "$OUTPUT" || echo "✅ No .unwrap() in Trusty Modules" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# High: Grown-up Scripts (repositories, commands)
echo "### ⚠️  HIGH: Grown-up Scripts (orchestration layer)" >> "$OUTPUT"
echo "" >> "$OUTPUT"
grep -rn "\.unwrap()" --include="*.rs" ./database/src/repositories ./pipeline/src-tauri/src/commands ./daw/src-tauri/src/commands 2>/dev/null | grep -v "target/" | grep -v "#\[cfg(test)\]" >> "$OUTPUT" || echo "✅ No .unwrap() in Grown-up Scripts" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Medium: Main files and entry points
echo "### 📋 MEDIUM: Entry Points" >> "$OUTPUT"
echo "" >> "$OUTPUT"
grep -rn "\.unwrap()" --include="main.rs" --include="lib.rs" . 2>/dev/null | grep -v "target/" | grep -v "#\[cfg(test)\]" >> "$OUTPUT" || echo "✅ No .unwrap() in entry points" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# By File (detailed list)
echo "## Detailed List by File" >> "$OUTPUT"
echo "" >> "$OUTPUT"

# Get all files with unwraps
FILES_WITH_UNWRAPS=$(grep -rl "\.unwrap()" --include="*.rs" . | grep -v "target/" | grep -v "\.git/" | sort)

for file in $FILES_WITH_UNWRAPS; do
    # Skip test files
    if [[ $file == *"test"* ]] || grep -q "#\[cfg(test)\]" "$file"; then
        continue
    fi
    
    echo "### \`$file\`" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    echo "\`\`\`rust" >> "$OUTPUT"
    grep -n "\.unwrap()" "$file" | head -20 >> "$OUTPUT"
    echo "\`\`\`" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
done

# Recommendations
cat >> "$OUTPUT" << 'EOF'

---

## Fixing Strategy

### Pattern 1: Replace with ? operator
```rust
// ❌ BAD
let value = some_result.unwrap();

// ✅ GOOD
let value = some_result?;
```

### Pattern 2: Replace with proper error handling
```rust
// ❌ BAD
let value = some_result.unwrap();

// ✅ GOOD
let value = some_result.map_err(|e| {
    MyError::ParseError(format!("Failed to parse: {}", e))
})?;
```

### Pattern 3: Replace with default value
```rust
// ❌ BAD
let value = some_option.unwrap();

// ✅ GOOD
let value = some_option.unwrap_or_default();
// or
let value = some_option.unwrap_or(120.0); // sensible default
```

### Pattern 4: Early return with error
```rust
// ❌ BAD
let value = some_option.unwrap();

// ✅ GOOD
let value = some_option.ok_or(MyError::MissingValue)?;
```

---

## Priority Order

1. **CRITICAL**: Fix all .unwrap() in `core/` directories (Trusty Modules)
2. **HIGH**: Fix all .unwrap() in `repositories/` and `commands/` (Grown-up Scripts)
3. **MEDIUM**: Fix all .unwrap() in entry points (`main.rs`, `lib.rs`)
4. **LOW**: Fix remaining .unwrap() in `bin/` (Task-O-Matics)

**Note:** Test code (in `#[cfg(test)]` blocks) can keep `.unwrap()` - it's acceptable in tests.

EOF

echo -e "${GREEN}✅ Report generated: $OUTPUT${NC}"
echo ""
echo -e "${YELLOW}Summary:${NC}"
echo -e "  Total .unwrap() calls found: ${RED}$TOTAL_UNWRAPS${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Review the report: cat $OUTPUT"
echo "  2. Start with CRITICAL items (core/ directories)"
echo "  3. Use the fixing patterns provided"
echo "  4. Run cargo clippy to catch more issues"
