#!/bin/bash
# Audit Core Directories for I/O Violations
# Archetype: Task-O-Matic (Complete standalone task)
# Verifies that core/ directories contain ONLY Trusty Modules (no I/O)

set -e
set -u

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
GREEN='\033[0;32m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}🔍 Auditing core/ directories for I/O violations...${NC}"
echo ""

# Output file
OUTPUT="core-io-audit-$(date +%Y%m%d-%H%M%S).md"

# I/O patterns to search for
IO_PATTERNS=(
    "std::fs::"          # File system operations
    "File::"             # File operations
    "OpenOptions"        # File opening
    "read_to_string"     # File reading
    "write!"             # File writing
    "tokio::fs::"        # Async file operations
    "sqlx::"             # Database operations
    "reqwest::"          # HTTP requests
    "hyper::"            # HTTP operations
    "println!"           # Console output (side effect)
    "eprintln!"          # Error output (side effect)
    "dbg!"               # Debug output (side effect)
)

# Start report
cat > "$OUTPUT" << 'EOF'
# Core Directory I/O Audit Report
**Generated:** $(date)
**Purpose:** Verify Trusty Module compliance (no I/O in core/ directories)

---

## 🎯 Rule: Everything in core/ MUST be pure (no I/O)

**Trusty Module Requirements:**
- ✅ Pure functions only (same input = same output)
- ✅ No file system access
- ✅ No database operations
- ✅ No network requests
- ✅ No console output (println, eprintln, dbg)
- ✅ All logic testable without I/O

---

EOF

# Core directories to audit
CORE_DIRS=(
    "./shared/src"
    "./database/src/models"
    "./pipeline/src-tauri/src/core"
    "./daw/src-tauri/src/core"
)

VIOLATIONS_FOUND=0

for dir in "${CORE_DIRS[@]}"; do
    if [ ! -d "$dir" ]; then
        continue
    fi
    
    echo -e "${MAGENTA}Checking: $dir${NC}"
    echo "## Checking: \`$dir\`" >> "$OUTPUT"
    echo "" >> "$OUTPUT"
    
    DIR_VIOLATIONS=0
    
    for pattern in "${IO_PATTERNS[@]}"; do
        # Search for pattern
        MATCHES=$(grep -rn "$pattern" --include="*.rs" "$dir" 2>/dev/null | grep -v "/target/" | grep -v "#\[cfg(test)\]" || true)
        
        if [ -n "$MATCHES" ]; then
            DIR_VIOLATIONS=$((DIR_VIOLATIONS + 1))
            VIOLATIONS_FOUND=$((VIOLATIONS_FOUND + 1))
            
            echo -e "${RED}  ❌ Found: $pattern${NC}"
            
            echo "### ❌ VIOLATION: \`$pattern\`" >> "$OUTPUT"
            echo "" >> "$OUTPUT"
            echo "\`\`\`" >> "$OUTPUT"
            echo "$MATCHES" >> "$OUTPUT"
            echo "\`\`\`" >> "$OUTPUT"
            echo "" >> "$OUTPUT"
        fi
    done
    
    if [ $DIR_VIOLATIONS -eq 0 ]; then
        echo -e "${GREEN}  ✅ No I/O violations found${NC}"
        echo "✅ **No I/O violations found**" >> "$OUTPUT"
        echo "" >> "$OUTPUT"
    fi
    
    echo "" >> "$OUTPUT"
done

# Summary
cat >> "$OUTPUT" << EOF

---

## Summary

**Total Violations Found:** $VIOLATIONS_FOUND

EOF

if [ $VIOLATIONS_FOUND -eq 0 ]; then
    cat >> "$OUTPUT" << 'EOF'
✅ **All core/ directories are clean - no I/O violations detected!**

Your Trusty Modules are properly isolated from side effects.

EOF
else
    cat >> "$OUTPUT" << 'EOF'
❌ **I/O violations detected in core/ directories**

These violations MUST be fixed by:

1. **Moving I/O to Grown-up Scripts**: Create orchestration functions in `commands/` or `repositories/`
2. **Passing data instead of reading**: The CALLER should do I/O and pass data to Trusty Modules
3. **Removing side effects**: Replace println!/dbg! with returning data for the caller to log

### Example Fix:

```rust
// ❌ BAD - I/O in core/
pub fn process_midi(path: &Path) -> Result<MidiData> {
    let bytes = std::fs::read(path)?;  // ❌ File I/O
    analyze_midi(&bytes)
}

// ✅ GOOD - I/O separated
// In core/analysis.rs (Trusty Module)
pub fn analyze_midi(bytes: &[u8]) -> Result<MidiData> {
    // Pure analysis, no I/O
    let parsed = midly::parse(bytes)?;
    Ok(MidiData { /* ... */ })
}

// In commands/process.rs (Grown-up Script)
pub async fn process_midi_file(path: &Path) -> Result<MidiData> {
    let bytes = std::fs::read(path)?;  // ✅ I/O here
    analyze_midi(&bytes)                // ✅ Pure logic called
}
```

EOF
fi

cat >> "$OUTPUT" << 'EOF'

---

## Next Steps

1. Review each violation in this report
2. Move I/O operations to Grown-up Scripts (commands/, repositories/)
3. Refactor Trusty Modules to accept data instead of reading it
4. Re-run this audit after fixes: `./audit-core-io.sh`

EOF

echo ""
echo -e "${GREEN}✅ Audit complete: $OUTPUT${NC}"
echo ""

if [ $VIOLATIONS_FOUND -eq 0 ]; then
    echo -e "${GREEN}🎉 No I/O violations found - core/ directories are clean!${NC}"
else
    echo -e "${RED}⚠️  Found $VIOLATIONS_FOUND I/O violations in core/ directories${NC}"
    echo -e "${YELLOW}These MUST be fixed to maintain Trusty Module pattern${NC}"
fi

echo ""
echo -e "${BLUE}View report:${NC} cat $OUTPUT"
