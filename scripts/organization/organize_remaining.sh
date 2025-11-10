#!/bin/bash
set -e

# Keep in root (essential project files)
KEEP_IN_ROOT=(
    "README.md"
    "CLAUDE.md"
)

# Move everything else to appropriate folders
mkdir -p docs/{analysis,deployment,guides,code-quality,planning,sessions,errors}

# Move analysis files
mv grok_*.json* CLIPPY*.txt *.txt 2>/dev/null || true

# Check if files should stay in root
for file in *.md; do
    if [[ " ${KEEP_IN_ROOT[@]} " =~ " ${file} " ]]; then
        continue
    fi
    
    # Determine destination based on file name
    if [[ $file == *"KILO"* ]]; then
        mv "$file" docs/kilo-code/ 2>/dev/null || true
    elif [[ $file == *"STEP"* ]] || [[ $file == *"IMPL"* ]]; then
        mv "$file" docs/guides/ 2>/dev/null || true  
    elif [[ $file == *"ERROR"* ]] || [[ $file == *"REPAIR"* ]]; then
        mv "$file" docs/errors/ 2>/dev/null || true
    elif [[ $file == *"TOOLKIT"* ]]; then
        mv "$file" docs/guides/ 2>/dev/null || true
    elif [[ $file == *"CHECKLIST"* ]]; then
        mv "$file" docs/planning/ 2>/dev/null || true
    else
        mv "$file" docs/ 2>/dev/null || true
    fi
done

echo "✅ Remaining files organized!"
