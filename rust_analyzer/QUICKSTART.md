# 🚀 Quick Start Guide - Quantum Analyzer

## Installation (2 minutes)

```bash
cd rust-analyzer

# Build release binary
chmod +x build.sh
./build.sh

# Or manually:
cargo build --release
```

## Basic Usage

### 1. Analyze Your Project

```bash
# From the rust-analyzer directory
./target/release/quantum-analyzer --project /home/dojevou/projects/midi-software-center

# Or navigate to your project first
cd /home/dojevou/projects/midi-software-center
/path/to/rust-analyzer/target/release/quantum-analyzer
```

### 2. Auto-Fix Common Issues

```bash
./target/release/quantum-analyzer --autofix

# This automatically fixes:
# - Missing derive attributes
# - Missing #[inline] attributes
# - Missing SAFETY comments
# - Some unwrap() calls
```

### 3. Generate Claude Code Tasks

```bash
./target/release/quantum-analyzer --claude-code

# Creates: CLAUDE_CODE_TASKS.md
# Then execute: claude-code --file CLAUDE_CODE_TASKS.md
```

## 🎯 What You'll See

```
🚀 QUANTUM ANALYZER
Advanced Rust Project Analysis
════════════════════════════════════════════════════════════

🔍 Analyzing project...
⠋ [####################################] 8/8 Analysis complete!

📊 ANALYSIS REPORT
════════════════════════════════════════════════════════════

📁 PROJECT STRUCTURE
   Cargo files:  5
   Rust files:   47
   Total lines:  12,450

🔨 BUILD STATUS
   ❌ 3 build errors

✨ CODE QUALITY
   Clippy warnings: 23
   Status: ⚠️  Needs attention

🛡️  SECURITY
   Unwrap calls: 156
   Unsafe blocks: 8

🎹 MIDI ANALYSIS
   Real-time issues: 15
   ⚠️  src/audio.rs:45 - Heap allocation in audio callback
```

## 📋 Common Use Cases

### Daily Development Check

```bash
# Quick health check
./target/release/quantum-analyzer

# If issues found, auto-fix
./target/release/quantum-analyzer --autofix

# Verify fixes
cargo check && cargo clippy && cargo test
```

### Pre-Commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
/path/to/quantum-analyzer --output json > analysis.json

CRITICAL=$(jq '.build_errors | length' analysis.json)
if [ "$CRITICAL" -gt "0" ]; then
    echo "❌ Critical issues found - commit blocked"
    exit 1
fi
```

### CI/CD Integration

```bash
# Run in CI
./target/release/quantum-analyzer --output json > report.json

# Check results
jq '.has_critical_issues' report.json
```

### Find Specific Issues

```bash
# Find all unwrap() calls
./target/release/quantum-analyzer --output json | \
  jq '.security.unwrap_calls[] | "\(.file):\(.line)"'

# Find MIDI violations
./target/release/quantum-analyzer --output json | \
  jq '.midi_analysis.real_time_issues[] | select(.kind == "HeapAllocation")'

# Find high-impact performance issues
./target/release/quantum-analyzer --output json | \
  jq '.performance_hints[] | select(.impact == "High")'
```

## 🤖 With AI Insights

```bash
# Set API key
export GROK_API_KEY='xai-your-key-here'

# Run with AI
./target/release/quantum-analyzer --ai

# AI will provide:
# - Architecture recommendations
# - Performance optimization strategies
# - MIDI-specific best practices
```

## 📊 Output Formats

### Text (Human-Readable)
```bash
./target/release/quantum-analyzer --output text
```

### JSON (Machine-Readable)
```bash
./target/release/quantum-analyzer --output json > report.json
```

### Markdown (Documentation)
```bash
./target/release/quantum-analyzer --output markdown > ANALYSIS.md
```

## 🔄 Watch Mode

```bash
# Continuous monitoring
./target/release/quantum-analyzer --watch

# Re-analyzes every 5 seconds
# Alerts on critical issues
# Press Ctrl+C to stop
```

## 🎓 Advanced Usage

### Combine Multiple Options

```bash
./target/release/quantum-analyzer \
  --autofix \
  --claude-code \
  --ai \
  --verbose
```

### As Cargo Plugin

```bash
# Install
cargo install --path .

# Use
cargo quantum
cargo quantum --autofix
cargo quantum --claude-code
```

### Scripted Analysis

```bash
#!/bin/bash
# analyze.sh - Comprehensive analysis script

echo "Running analysis..."
./target/release/quantum-analyzer --output json > report.json

echo "Found issues:"
jq '{
  build_errors: (.build_errors | length),
  security_issues: (.security.unwrap_calls | length),
  midi_issues: (.midi_analysis.real_time_issues | length)
}' report.json

echo "Applying fixes..."
./target/release/quantum-analyzer --autofix

echo "Generating tasks..."
./target/release/quantum-analyzer --claude-code

echo "Done! Review CLAUDE_CODE_TASKS.md"
```

## 🎯 For Your MIDI Project

### Recommended Workflow

```bash
# 1. Initial analysis
cd /home/dojevou/projects/midi-software-center
/path/to/quantum-analyzer --verbose

# 2. Focus on MIDI issues
/path/to/quantum-analyzer --output json | \
  jq '.midi_analysis.real_time_issues'

# 3. Apply safe auto-fixes
/path/to/quantum-analyzer --autofix

# 4. Generate tasks for remaining issues
/path/to/quantum-analyzer --claude-code

# 5. Execute with Claude Code
claude-code --file CLAUDE_CODE_TASKS.md

# 6. Verify everything works
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

### MIDI-Specific Checks

The analyzer looks for:

1. **Heap allocations** in audio threads
2. **Mutex locks** in real-time code
3. **Blocking I/O** operations
4. **System calls** in callbacks
5. **Unwrap calls** that could panic
6. **Missing #[inline]** on hot paths
7. **Buffer size** optimization

## 💡 Pro Tips

1. **Run before commits** - Catch issues early
2. **Use --autofix liberally** - It's conservative and safe
3. **Check MIDI issues first** - They cause audio glitches
4. **Generate Claude Code tasks** - Let AI fix complex issues
5. **Enable AI insights weekly** - Get architecture recommendations

## 📞 Getting Help

```bash
# Full help
./target/release/quantum-analyzer --help

# Version
./target/release/quantum-analyzer --version

# Verbose output for debugging
./target/release/quantum-analyzer --verbose
```

## ✅ Success Checklist

After running the analyzer, you should see:

- ✅ Zero build errors
- ✅ Minimal clippy warnings (<10)
- ✅ No security vulnerabilities
- ✅ No critical MIDI violations
- ✅ Reasonable unwrap count (<50)

## 🎉 You're Ready!

The analyzer is ready to use on your MIDI Software Center project!

```bash
# Start analyzing now:
./target/release/quantum-analyzer --project /home/dojevou/projects/midi-software-center --autofix --claude-code
```
