# Project Report: project-definitions

> Generated: 2025-11-30 09:35:02
> Path: `/home/dojevou/projects/midi-software-center/temp_project_defs`

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | **Excellent** - Score ≥ 8/10 or Maintainability ≥ 65 |
| ⚠️ | **Warning** - Score 5-8/10 or Maintainability 40-65 |
| ❌ | **Needs Work** - Score < 5/10 or Maintainability < 40 |
| 🔒 | **Security** - Security-related finding or issue |
| 🐛 | **Bug** - Potential bug or error detected |
| 📁 | **File/Folder** - File system related item |
| 📊 | **Metrics** - Statistical data or analysis |
| 📝 | **Documentation** - Docstring or comment related |
| 🔍 | **Analysis** - Currently being analyzed |
| 📦 | **Package** - Dependency or import related |
| 🚀 | **Performance** - Performance or optimization related |

## Table of Contents

- [Legend](#legend)
- [Summary](#summary)
- [Project Statistics](#project-statistics)
- [Code Quality](#code-quality)
- [Dependencies](#dependencies)
- [File Structure](#file-structure)
- [TODOs and FIXMEs](#todos-and-fixmes)
- [File Details](#file-details)

## Summary

| Metric | Value |
|--------|-------|
| Total Files | 7 |
| Total Lines | 573 |
| Lines of Code | 481 |
| Functions | 0 |
| Classes | 0 |
| Avg Pylint Score | 0.00/10 |
| Docstring Coverage | 0.0% |

## Project Statistics

### Files by Extension

| Extension | Count | Lines |
|-----------|-------|-------|
| .toml | 6 | 532 |
| .json | 1 | 41 |

## Code Quality

## Dependencies

## File Structure

```
temp_project_defs/
├── app_Cargo.toml
├── app_package.json
├── daw_Cargo.toml
├── pipeline_Cargo.toml
├── root_Cargo.toml
├── rust_analyzer_Cargo.toml
└── shared_Cargo.toml
```

## TODOs and FIXMEs

*No TODOs or FIXMEs found*

## File Details

### `app_Cargo.toml` {#app-cargo-toml}

- **Lines**: 29 (code: 25, comments: 0, blank: 4)

#### Source Code

```toml
[package]
name = "midi-software-center"
version = "1.0.0"
edition = "2021"

[lib]
name = "midi_app"
path = "src/lib.rs"

[[bin]]
name = "midi-software-center"
path = "src/main.rs"

[dependencies]
tauri = { version = "2.0", features = [] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "migrate", "chrono", "bigdecimal"] }
tokio = { version = "1.35", features = ["full"] }
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-appender = "0.2"
chrono = { version = "0.4", features = ["serde"] }
midi-pipeline = { path = "../../pipeline/src-tauri" }
midi-software-center-daw = { path = "../../daw/src-tauri" }

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

```

### `app_package.json` {#app-package-json}

- **Lines**: 41 (code: 41, comments: 0, blank: 0)

#### Source Code

```json
{
  "name": "midi-software-center",
  "private": true,
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "meilisearch": "^0.54.0",
    "svelte": "^4.2.8",
    "tone": "^15.1.22"
  },
  "devDependencies": {
    "@playwright/test": "^1.56.1",
    "@sveltejs/vite-plugin-svelte": "^3.0.1",
    "@tauri-apps/cli": "^2.0.0",
    "@testing-library/svelte": "^4.0.5",
    "@tsconfig/svelte": "^5.0.0",
    "@types/node": "^20.0.0",
    "@vitest/ui": "^1.0.4",
    "autoprefixer": "^10.4.22",
    "happy-dom": "^12.10.3",
    "postcss": "^8.5.6",
    "svelte-check": "^3.6.2",
    "tailwindcss": "^3.4.17",
    "typescript": "^5.3.3",
    "vite": "^5.0.8",
    "vitest": "^1.0.4"
  }
}
```

### `daw_Cargo.toml` {#daw-cargo-toml}

- **Lines**: 47 (code: 41, comments: 0, blank: 6)

#### Source Code

```toml
# daw/src-tauri/Cargo.toml
[package]
name = "midi-software-center-daw"
version = "0.1.0"
description = "DAW Backend for MIDI Software Center"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { workspace = true }
serde = { workspace = true }
clap = { version = "4.4", features = ["derive"] }
serde_json = { workspace = true }
tokio = { workspace = true }
midly = "0.5"
sqlx = { workspace = true }
meilisearch-sdk = "0.24"
sysinfo = "0.30"
chrono = { workspace = true }
rand = "0.8"
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
tracing-appender = "0.2"
zip = "0.6"
futures-util = "0.3"
uuid = { workspace = true }
thiserror = { workspace = true }
parking_lot = { workspace = true }
midir = "0.8"
dirs = "5.0"
bincode = "1.3"

[dev-dependencies]
tempfile = "3.10"

[features]
# this feature is used for production builds or when `devPath` points to the filesystem and the built-in dev server is disabled.
# If you use cargo directly instead of tauri's cli you need to __also__ set `TAURI_DEV` to `false`
custom-protocol = [ "tauri/custom-protocol" ]
default = [ "custom-protocol" ]


```

### `pipeline_Cargo.toml` {#pipeline-cargo-toml}

- **Lines**: 199 (code: 171, comments: 0, blank: 28)

#### Source Code

```toml
# pipeline/src-tauri/Cargo.toml
[package]
name = "midi-pipeline"
version = "0.1.0"
description = "MIDI file processing pipeline"
authors = ["Your Name"]
license = ""
repository = ""
edition = "2021"
default-run = "midi-pipeline"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
# Tauri framework
tauri = { workspace = true }
tauri-plugin-shell = { workspace = true }
tauri-plugin-dialog = { workspace = true }
tauri-plugin-fs = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

# Shared library (common code)
midi-library-shared = { path = "../../shared/rust" }

# MIDI processing
midly = "0.5"                    # Fast MIDI parser, zero-copy
rimd = "0.0.1"                   # MIDI manipulation and generation
rust-music-theory = "0.3"        # Music theory (chords, scales, intervals)

# Async runtime
tokio = { workspace = true }
tokio-util = "0.7"

# Parallel processing
rayon = "1.8"                    # Data parallelism
crossbeam-channel = "0.5"        # Multi-producer multi-consumer channels
crossbeam-queue = "0.3"          # Lock-free MPMC queues for pipeline
num_cpus = "1.16"                # Detect CPU count

# File handling
walkdir = "2"                    # Recursive directory traversal
jwalk = "0.8"                    # Parallel directory walking
glob = "0.3"                     # File pattern matching

# Archive handling - ULTRA-FAST parallel extraction
zip = { version = "0.6", default-features = false, features = ["deflate", "bzip2"] }
unrar = "0.5"                    # RAR extraction
sevenz-rust = "0.5"              # 7z extraction
flate2 = { version = "1.0", features = ["zlib-ng"] }  # 2x faster zlib decompression
async-compression = { version = "0.4", features = ["tokio", "gzip", "bzip2", "xz", "zstd"] }  # Async multi-format decompression
bzip2 = "0.4"                    # Multi-threaded bzip2

# Hashing and deduplication
blake3 = "1.5"                   # Fast cryptographic hash
sha2 = "0.10"                    # SHA-256 for compatibility
xxhash-rust = { version = "0.8", features = ["xxh3"] }  # Fast non-cryptographic hash

# Database
sqlx = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging and tracing
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
tracing-appender = "0.2"

# String utilities
regex = "1.10"
unicode-normalization = "0.1"
unicode-segmentation = "1.10"
strsim = "0.11"                  # String similarity for fuzzy matching
num-traits = "0.2"               # Numeric traits for BigDecimal conversion

# Configuration
config = { workspace = true }
directories = "5.0"              # Cross-platform directories

# Performance monitoring
sysinfo = "0.30"                 # System information

# CLI tools
clap = { version = "4.4", features = ["derive", "env"] }
dotenv = "0.15"
indicatif = "0.17"
futures = "0.3"
hex = "0.4.3"

# Performance optimizations
memmap2 = "0.9"                  # Memory-mapped files (zero-copy I/O)
tikv-jemallocator = "0.5"        # High-performance allocator
once_cell = "1.19"               # Lazy static initialization
parking_lot = "0.12"             # Faster mutexes/rwlocks
flume = "0.11"                   # Fast MPMC channels (faster than crossbeam)
typed-arena = "2.0"              # Arena allocators for cache-friendly memory layouts
ahash = "0.8.12"
mimalloc = "0.1.48"
dashmap = "6.1.0"
nom = "8.0.0"
winnow = "0.7.13"
bumpalo = "3.19.0"
memchr = "2.7.6"
simdutf8 = "0.1.5"
highway = "1.3.0"
wyhash = "0.6.0"
zune-inflate = "0.2.54"
askama = "0.14.0"
zstd = "0.13.3"
lz4 = "1.28.1"
snap = "1.1.1"
ultraviolet = "0.10.0"
rustfft = "6.4.1"
realfft = "3.5.0"
monoio = "0.2.4"
simdeez = "2.0.0"
snmalloc-rs = "0.3.8"
tokio-postgres = "0.7.15"
wide = "0.8.3"
nalgebra = "0.34.1"
ndarray = "0.17.1"
aho-corasick = "1.1.4"
smartstring = "1.0.1"
compact_str = "0.9.0"
itoa = "1.0.15"
ryu = "1.0.20"
dtoa = "1.0.10"
crossbeam = "0.8.4"
lockfree = "0.5.1"
atomic-counter = "1.0.1"
libdeflater = "1.25.0"
lzma = "0.2.2"
miniz_oxide = "0.8.9"
# Note: portable_simd feature requires Rust 1.75+ stable

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
criterion = "0.5"                # Benchmarking
proptest = "1.4"                 # Property-based testing
rand = "0.8"                     # Random data generation for tests
pprof = "0.15.0"
tracy-client = "0.18.3"
dhat = "0.3.3"

# Binary targets
[[bin]]
name = "import_unified"
path = "src/bin/import_unified.rs"

[[bin]]
name = "orchestrator"
path = "src/bin/orchestrator.rs"

[[bin]]
name = "parallel_extract"
path = "src/bin/parallel_extract.rs"

[[bin]]
name = "normalize_filenames"
path = "src/bin/normalize_filenames.rs"

[[bin]]
name = "pipeline-orchestrator"
path = "src/bin/pipeline-orchestrator.rs"

[[bin]]
name = "midi_to_mpcpattern"
path = "src/bin/midi_to_mpcpattern.rs"

[[bin]]
name = "midi_to_mpcpattern_parallel"
path = "src/bin/midi_to_mpcpattern_parallel.rs"

# Aggressive optimization profile
[profile.release]
opt-level = 3                # Maximum optimizations
codegen-units = 1            # Better optimization (slower compile)
lto = "fat"                  # Full link-time optimization
panic = "abort"              # Smaller binary, faster execution
strip = true                 # Remove debug symbols

# MAXIMUM SPEED profile for batch converters
[profile.ultra-fast]
inherits = "release"
opt-level = 3
codegen-units = 1
lto = "fat"
panic = "abort"
strip = true
overflow-checks = false      # Disable integer overflow checks (unsafe but faster)
debug-assertions = false     # Disable debug assertions

```

### `root_Cargo.toml` {#root-cargo-toml}

- **Lines**: 124 (code: 101, comments: 0, blank: 23)

#### Source Code

```toml
# MIDI Library System - Workspace Cargo.toml
# Root workspace configuration for all Rust projects
#
# Architecture:
#   - pipeline/src-tauri   : MIDI import, processing, and analysis
#   - daw/src-tauri        : Real-time MIDI playback and sequencing
#   - shared/rust          : Common MIDI parsing and analysis code
#   - scripts/import-tool  : CLI utilities
#
# Build Performance:
#   - Full workspace: ~28 seconds (dev), ~2-3 minutes (release)
#   - Shared library: ~1-2 seconds
#   - Pipeline: ~9-24 seconds (uses shared)
#   - DAW: ~10-20 seconds (independent MIDI for playback)
#
# Note: Pipeline uses shared library for MIDI file analysis.
#       DAW has independent MIDI for real-time playback (not duplication).

[workspace]
resolver = "2"
members = [
    "pipeline/src-tauri",        # Import & analysis (uses shared library)
    "daw/src-tauri",             # Playback & sequencing (independent MIDI)
    "app/src-tauri",             # Unified frontend app
    # "studio/src-tauri",        # Removed - incomplete, may add later
    "shared/rust",               # Shared MIDI parsing & analysis
    "scripts/import-tool",       # CLI utilities
    "scripts/test-midi-files",   # MIDI testing tool
]

# Shared dependencies across all workspace members
[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full", "parking_lot"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "bigdecimal"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Tauri v2
tauri = { version = "2.7", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
sha2 = "0.10"

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Configuration
config = "0.13"
parking_lot = "0.12"        # Faster mutexes and RwLocks

# Platform-specific configurations
[workspace.metadata.cross]
# Configuration for cross-compilation if needed

# ============================================================================
# OPTIMIZED BUILD PROFILES (Merged and Deduplicated)
# ============================================================================

# Development profile - fast compilation for your code, optimized dependencies
[profile.dev]
opt-level = 0          # No optimization for your code (fast compile)
debug = true           # Include debug info
incremental = true     # Incremental compilation for faster rebuilds

# Optimize ALL dependencies at high level (HUGE speedup)
[profile.dev.package."*"]
opt-level = 3          # Maximum optimization for dependencies

# Extra optimization for heavy Tauri crates
[profile.dev.package.tauri]
opt-level = 3

# Database heavy crates
[profile.dev.package.sqlx]
opt-level = 3

# Async runtime
[profile.dev.package.tokio]
opt-level = 3

[profile.dev.package.tokio-util]
opt-level = 3

# Serialization
[profile.dev.package.serde]
opt-level = 3

[profile.dev.package.serde_json]
opt-level = 3

# Release profile - maximum optimization
[profile.release]
opt-level = 3          # Maximum optimization
lto = "thin"           # Link-time optimization (thin is faster than fat)
codegen-units = 1      # Better optimization, slower compile
panic = "abort"        # Smaller binary
strip = true           # Remove debug symbols
debug = false          # No debug info

# Benchmarking profile
[profile.bench]
inherits = "release"
debug = true

# Test profile - slight optimization for faster tests
[profile.test]
opt-level = 1

```

### `rust_analyzer_Cargo.toml` {#rust-analyzer-cargo-toml}

- **Lines**: 87 (code: 67, comments: 0, blank: 20)

#### Source Code

```toml
[package]
name = "quantum-analyzer"
version = "1.0.0"
edition = "2021"
authors = ["MIDI Software Center"]
description = "Advanced Rust project analyzer with AST-level analysis and auto-fix capabilities"
license = "MIT"
repository = "https://github.com/midi-software-center/quantum-analyzer"

[[bin]]
name = "quantum-analyzer"
path = "src/main.rs"

[[bin]]
name = "cargo-quantum"
path = "src/cargo_plugin.rs"

[dependencies]
# Core functionality
syn = { version = "2.0", features = ["full", "extra-traits", "visit", "visit-mut"] }
quote = "1.0"
proc-macro2 = "1.0"

# Cargo integration
cargo_metadata = "0.18"

# Async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# HTTP client for AI integration
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# CLI
clap = { version = "4.4", features = ["derive", "cargo"] }
colored = "2.1"
indicatif = "0.17"

# File handling
walkdir = "2.4"
ignore = "0.4"
globset = "0.4"

# Parallel processing
rayon = "1.8"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Text processing
regex = "1.10"
lazy_static = "1.4"

# Path handling
pathdiff = "0.2"

# Graph algorithms for dependency analysis
petgraph = "0.6"

# Output formatting
tabled = "0.15"

# Date/time
chrono = "0.4"

# Environment
dirs = "5.0"

[dev-dependencies]
criterion = "0.5"
tempfile = "3.8"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
opt-level = 0

```

### `shared_Cargo.toml` {#shared-cargo-toml}

- **Lines**: 46 (code: 35, comments: 0, blank: 11)

#### Source Code

```toml
[package]
name = "midi-library-shared"
version = "0.1.0"
edition = "2021"
authors = ["MIDI Library System"]
description = "Shared library for MIDI Library System (Pipeline + DAW)"

[lib]
name = "midi_library_shared"
path = "src/lib.rs"

[dependencies]
# MIDI parsing - for Priority 1 (MIDI Core)
midly = "0.5"

# Database - for Priority 2 (Database models & repositories)
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"], optional = true }

# Serialization - for all modules
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling - for all modules
thiserror = "1.0"
anyhow = "1.0"

# Async runtime - for Priority 2 (repositories)
tokio = { version = "1.35", features = ["full"], optional = true }

# Time - for database models
chrono = { version = "0.4", features = ["serde"] }

# UUID - for database models
uuid = { version = "1.6", features = ["v4", "serde"] }

# Logging - for all modules
tracing = "0.1"

[features]
default = []
database = ["sqlx", "tokio"]
full = ["database"]

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"

```
