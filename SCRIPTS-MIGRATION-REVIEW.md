# SCRIPTS MIGRATION REVIEW

**Date:** 2025-10-24
**Purpose:** Review and adapt scripts to follow Three Archetypes pattern and code quality standards

---

## 📋 ORIGINAL SCRIPTS INVENTORY

From `/tmp/original-project/midi-library-system/scripts/`:

### Bash Scripts (Launch/Setup)
1. `launch-all.sh` (7.2KB, 258 lines)
2. `launch-pipeline.sh` (1.1KB)
3. `launch-daw.sh` (1.1KB)
4. `status.sh` (7.2KB)
5. `stop-all.sh` (4.8KB)
6. `install-launcher.sh` (9.1KB)
7. `uninstall-launcher.sh` (2.7KB)

### Rust CLI Tools
8. `import-tool/` (Rust CLI binary)
9. `analyze-tool/` (Rust CLI binary)

---

## 🎯 ARCHETYPE CLASSIFICATION

### Task-O-Matic (All Bash Scripts)

**Why:** Complete standalone tasks, not imported, have clear start/end

1. **launch-all.sh** → `scripts/launch/launch-all.sh`
   - **Archetype:** Task-O-Matic
   - **Purpose:** Start all services (database → pipeline/DAW)
   - **Issues Found:**
     - ❌ Hardcoded path: `$HOME/projects/midi-library-system`
     - ❌ Hardcoded password in script: `DB_PASSWORD="145278963"`
     - ❌ No error handling in some functions
   - **Required Changes:**
     - ✅ Use relative paths or detect project root
     - ✅ Move credentials to .env file
     - ✅ Add proper error handling throughout

2. **launch-pipeline.sh** → `scripts/launch/launch-pipeline.sh`
   - **Archetype:** Task-O-Matic
   - **Purpose:** Start Pipeline app only
   - **Issues Found:**
     - ❌ Likely has hardcoded paths (need to verify)
   - **Required Changes:**
     - ✅ Update paths to be relative
     - ✅ Check for dependencies before launching

3. **launch-daw.sh** → `scripts/launch/launch-daw.sh`
   - **Archetype:** Task-O-Matic
   - **Purpose:** Start DAW app only
   - **Issues Found:**
     - ❌ Likely has hardcoded paths (need to verify)
   - **Required Changes:**
     - ✅ Update paths to be relative
     - ✅ Check for dependencies before launching

4. **status.sh** → `scripts/launch/status.sh`
   - **Archetype:** Task-O-Matic
   - **Purpose:** Check status of all services
   - **Issues Found:** Need to review
   - **Required Changes:** Update to match new structure

5. **stop-all.sh** → `scripts/launch/stop-all.sh`
   - **Archetype:** Task-O-Matic
   - **Purpose:** Stop all running services
   - **Issues Found:** Need to review
   - **Required Changes:** Update to match new structure

6. **install-launcher.sh** → `scripts/setup/install-launcher.sh`
   - **Archetype:** Task-O-Matic
   - **Purpose:** One-time installation setup
   - **Issues Found:** Need to review
   - **Required Changes:** Update for new directory structure

7. **uninstall-launcher.sh** → `scripts/setup/uninstall-launcher.sh`
   - **Archetype:** Task-O-Matic
   - **Purpose:** Remove installation
   - **Issues Found:** Need to review
   - **Required Changes:** Update for new directory structure

### Task-O-Matic (Rust CLI Tools)

8. **import-tool/** → `scripts/import-tool/`
   - **Archetype:** Task-O-Matic (Rust binary with `main()`)
   - **Purpose:** CLI tool for importing MIDI files
   - **Issues to Check:**
     - ❌ May have `.unwrap()` calls (must remove)
     - ❌ May lack proper documentation
   - **Required Changes:**
     - ✅ Review for `.unwrap()` and replace with `?` operator
     - ✅ Add doc comments to public functions
     - ✅ Ensure proper error handling with `anyhow::Result`

9. **analyze-tool/** → `scripts/analyze-tool/`
   - **Archetype:** Task-O-Matic (Rust binary with `main()`)
   - **Purpose:** CLI tool for analyzing MIDI files
   - **Issues to Check:**
     - ❌ May have `.unwrap()` calls
     - ❌ May lack proper documentation
   - **Required Changes:**
     - ✅ Same as import-tool

---

## 🔧 MIGRATION PLAN

### Phase 1: Copy and Organize

```bash
# Launch scripts
cp launch-all.sh      scripts/launch/
cp launch-pipeline.sh scripts/launch/
cp launch-daw.sh      scripts/launch/
cp status.sh          scripts/launch/
cp stop-all.sh        scripts/launch/

# Setup scripts
cp install-launcher.sh   scripts/setup/
cp uninstall-launcher.sh scripts/setup/

# CLI tools (already in correct structure)
cp -r import-tool/  scripts/
cp -r analyze-tool/ scripts/
```

### Phase 2: Fix Critical Issues

#### A. Remove Hardcoded Paths

**Before:**
```bash
PROJECT_ROOT="$HOME/projects/midi-library-system"
```

**After:**
```bash
# Auto-detect project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
```

#### B. Move Credentials to .env

**Before:**
```bash
DB_PASSWORD="145278963"
```

**After:**
```bash
# Load from .env file
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
else
    echo "Error: .env file not found"
    exit 1
fi
```

**Create .env.example:**
```bash
# Database Configuration
DB_HOST=localhost
DB_PORT=5432
DB_USER=midi_user
DB_PASSWORD=your_password_here
DB_NAME=midi_library
```

#### C. Add Error Handling

**Add to all scripts:**
```bash
set -e  # Exit on error
set -u  # Error on undefined variables
set -o pipefail  # Catch errors in pipes
```

### Phase 3: Review Rust CLI Tools

#### Checklist for import-tool and analyze-tool:

1. **Remove `.unwrap()` and `.expect()`:**
```bash
grep -r "\.unwrap()" scripts/import-tool/src/
grep -r "\.expect(" scripts/import-tool/src/
```

2. **Add proper error handling:**
```rust
// ❌ BAD
let file = File::open(path).unwrap();

// ✅ GOOD
let file = File::open(path)
    .context(format!("Failed to open file: {}", path))?;
```

3. **Add documentation:**
```rust
/// Import MIDI files from a directory
///
/// # Arguments
/// * `path` - Directory path to scan for MIDI files
/// * `recursive` - Whether to scan subdirectories
///
/// # Returns
/// * `Ok(count)` - Number of files imported
/// * `Err(error)` - Import error
///
/// # Examples
/// ```
/// let count = import_from_directory("/path/to/midi", true)?;
/// println!("Imported {} files", count);
/// ```
pub fn import_from_directory(path: &str, recursive: bool) -> Result<usize> {
    // Implementation
}
```

4. **Verify dependencies in Cargo.toml:**
```toml
[dependencies]
anyhow = "1.0"      # For error handling
thiserror = "1.0"   # For custom errors
clap = "4.0"        # For CLI argument parsing
```

---

## ✅ CODE QUALITY STANDARDS FOR SCRIPTS

### Bash Scripts (Task-O-Matics)

**Required:**
- ✅ `set -e` for error handling
- ✅ No hardcoded passwords or secrets
- ✅ Relative paths or auto-detect project root
- ✅ Clear error messages
- ✅ Check for required commands before using them
- ✅ Proper cleanup on exit (use `trap`)

**Example Header:**
```bash
#!/bin/bash
# MIDI Library System - [Script Purpose]
# Archetype: Task-O-Matic (Complete standalone task)
#
# Usage: ./script-name.sh [options]
# Example: ./launch-all.sh

set -e  # Exit on error
set -u  # Error on undefined variables
set -o pipefail  # Catch errors in pipes

# Auto-detect project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Load environment variables
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
fi

# ... rest of script
```

### Rust CLI Tools (Task-O-Matics)

**Required:**
- ✅ NO `.unwrap()` or `.expect()` in production code
- ✅ Use `anyhow::Result` for error handling
- ✅ Use `clap` for argument parsing
- ✅ Add doc comments to all public functions
- ✅ Proper error messages with context

**Example Structure:**
```rust
use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
#[command(name = "import-tool")]
#[command(about = "Import MIDI files into the library", long_about = None)]
struct Args {
    /// Directory to import from
    #[arg(short, long)]
    path: String,

    /// Import recursively
    #[arg(short, long, default_value = "false")]
    recursive: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Importing from: {}", args.path);

    let count = import_from_directory(&args.path, args.recursive)
        .context("Failed to import files")?;

    println!("Successfully imported {} files", count);
    Ok(())
}

fn import_from_directory(path: &str, recursive: bool) -> Result<usize> {
    // Implementation with proper error handling
    Ok(0)
}
```

---

## 📊 MIGRATION STATUS

### Completed
- [x] Directory structure created (`scripts/{setup,launch,verify}`)
- [x] Scripts classified by archetype
- [x] Issues identified

### In Progress
- [ ] Fix hardcoded paths in bash scripts
- [ ] Move credentials to .env file
- [ ] Review Rust CLI tools for `.unwrap()`
- [ ] Add documentation to Rust tools

### Pending
- [ ] Create integration tests for scripts
- [ ] Add setup.sh master script
- [ ] Create verification scripts
- [ ] Test all scripts end-to-end

---

## 🎯 NEXT STEPS

1. **Update launch-all.sh** - Fix hardcoded paths and credentials
2. **Create .env file** - Move all config to environment
3. **Review import-tool** - Check for `.unwrap()` and add docs
4. **Review analyze-tool** - Same as import-tool
5. **Test scripts** - Ensure all work with new structure
6. **Create setup.sh** - Master setup script for initial project setup

---

## 📝 NOTES

- All bash scripts are **Task-O-Matics** (complete standalone tasks)
- All Rust CLI tools are **Task-O-Matics** (have `main()`, not imported)
- NO Grown-up Scripts or Trusty Modules in scripts/ directory
- Scripts follow "do one thing well" Unix philosophy
- Each script should be independently runnable
