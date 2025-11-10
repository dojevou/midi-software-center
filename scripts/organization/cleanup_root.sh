#!/bin/bash
set -e

# Create scripts directory structure
mkdir -p scripts/{analysis,fixes,import,grok,organization}

# Move analysis scripts
mv *_analysis.py *_analyzer.py *_parser.py 2>/dev/null || true
mv analyze*.sh analyze*.py 2>/dev/null || true

# Move fix scripts
mv fix*.sh fix*.py *_fixer.py *_fix*.py master_fixer.sh 2>/dev/null || true

# Move import/test scripts
mv test*.sh 2>/dev/null || true

# Move grok scripts
mv *grok*.py grok*.sh 2>/dev/null || true

# Move organization scripts
mv organize*.sh organize_docs.sh organize_remaining.sh 2>/dev/null || true

# Move JSON files
mkdir -p data/analysis
mv *.json 2>/dev/null || true

# Move text files (except .env, .gitignore)
mkdir -p docs/raw
mv *.txt 2>/dev/null || true

# Move large setup/reset scripts to scripts/setup
mkdir -p scripts/setup
mv 00_*.sh 01_*.sh 02_*.sh 03_*.sh 2>/dev/null || true

# Move derived files
mv derive*.py 2>/dev/null || true

echo "✅ Root folder cleaned!"
