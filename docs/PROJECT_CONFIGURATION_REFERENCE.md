# MIDI Software Center - Project Configuration Reference

Generated: 2025-12-01
Purpose: Complete answers to all configuration questions for production-ready development

---

## Table of Contents

1. [Project Configuration Files](#1-project-configuration-files)
2. [Database Configuration](#2-database-configuration)
3. [Build & Deployment Scripts](#3-build--deployment-scripts)
4. [Audio/MIDI Specific Configs](#4-audiomidi-specific-configs)
5. [UI/UX Specifications](#5-uiux-specifications)
6. [Performance Requirements](#6-performance-requirements)
7. [Testing Requirements](#7-testing-requirements)
8. [Platform-Specific Requirements](#8-platform-specific-requirements)
9. [Error Reporting & Analytics](#9-error-reporting--analytics)
10. [External Integrations](#10-external-integrations)
11. [Accessibility Requirements](#11-accessibility-requirements)
12. [Security Requirements](#12-security-requirements)
13. [Documentation Requirements](#13-documentation-requirements)

---

## 1. Project Configuration Files

### 1.1 Root Workspace `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "pipeline/src-tauri",        # Import & analysis
    "daw/src-tauri",             # Playback & sequencing
    "app/src-tauri",             # Unified frontend app
    "shared/rust",               # Shared MIDI parsing
    "scripts/import-tool",       # CLI utilities
    "scripts/test-midi-files",   # MIDI testing tool
    "verification",              # Verification suite
]

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

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Configuration
config = "0.13"
parking_lot = "0.12"

# Build Profiles
[profile.dev]
opt-level = 0
debug = true
incremental = true

[profile.dev.package."*"]
opt-level = 3  # Optimized dependencies

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
strip = true
debug = false

[profile.test]
opt-level = 1
```

### 1.2 App `Cargo.toml` (`app/src-tauri/Cargo.toml`)

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

### 1.3 Pipeline `Cargo.toml` (`pipeline/src-tauri/Cargo.toml`)

```toml
[package]
name = "midi-pipeline"
version = "0.1.0"
edition = "2021"
default-run = "midi-pipeline"

[dependencies]
# Tauri framework
tauri = { workspace = true }
tauri-plugin-shell = { workspace = true }
tauri-plugin-dialog = { workspace = true }
tauri-plugin-fs = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

# Shared library
midi-library-shared = { path = "../../shared/rust" }

# MIDI processing
midly = "0.5"                    # Fast MIDI parser, zero-copy
rimd = "0.0.1"                   # MIDI manipulation
rust-music-theory = "0.3"        # Music theory

# Async runtime
tokio = { workspace = true }
tokio-util = "0.7"

# Parallel processing
rayon = "1.8"
crossbeam-channel = "0.5"
crossbeam-queue = "0.3"          # Lock-free MPMC queues
num_cpus = "1.16"

# File handling
walkdir = "2"
jwalk = "0.8"                    # Parallel directory walking
glob = "0.3"

# Archive handling - ULTRA-FAST
zip = { version = "0.6", default-features = false, features = ["deflate", "bzip2"] }
unrar = "0.5"
sevenz-rust = "0.5"
flate2 = { version = "1.0", features = ["zlib-ng"] }
async-compression = { version = "0.4", features = ["tokio", "gzip", "bzip2", "xz", "zstd"] }

# Hashing and deduplication
blake3 = "1.5"                   # Fast cryptographic hash
sha2 = "0.10"
xxhash-rust = { version = "0.8", features = ["xxh3"] }

# Database
sqlx = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
tracing-appender = "0.2"

# String utilities
regex = "1.10"
unicode-normalization = "0.1"
strsim = "0.11"

# Performance optimizations
memmap2 = "0.9"                  # Memory-mapped files
tikv-jemallocator = "0.5"        # High-performance allocator
parking_lot = "0.12"
flume = "0.11"                   # Fast MPMC channels
typed-arena = "2.0"              # Arena allocators
ahash = "0.8.12"
mimalloc = "0.1.48"
dashmap = "6.1.0"
bumpalo = "3.19.0"               # Arena allocation

# SIMD and FFT
rustfft = "6.4.1"
realfft = "3.5.0"
wide = "0.8.3"
nalgebra = "0.34.1"
ndarray = "0.17.1"

# CLI
clap = { version = "4.4", features = ["derive", "env"] }
indicatif = "0.17"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"
criterion = "0.5"
proptest = "1.4"
rand = "0.8"

# Aggressive release optimization
[profile.release]
opt-level = 3
codegen-units = 1
lto = "fat"
panic = "abort"
strip = true

# ULTRA-FAST profile
[profile.ultra-fast]
inherits = "release"
overflow-checks = false
debug-assertions = false
```

### 1.4 DAW `Cargo.toml` (`daw/src-tauri/Cargo.toml`)

```toml
[package]
name = "midi-software-center-daw"
version = "0.1.0"
edition = "2021"

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
midir = "0.8"                    # MIDI I/O
dirs = "5.0"
bincode = "1.3"

[dev-dependencies]
tempfile = "3.10"

[features]
custom-protocol = [ "tauri/custom-protocol" ]
default = [ "custom-protocol" ]
```

### 1.5 Shared Library `Cargo.toml` (`shared/rust/Cargo.toml`)

```toml
[package]
name = "midi-library-shared"
version = "0.1.0"
edition = "2021"

[lib]
name = "midi_library_shared"
path = "src/lib.rs"

[dependencies]
midly = "0.5"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid", "rust_decimal"], optional = true }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"
tokio = { version = "1.35", features = ["full"], optional = true }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
rust_decimal = { version = "1.33", features = ["serde"] }
hex = "0.4"
tracing = "0.1"

[features]
default = []
database = ["sqlx", "tokio"]
full = ["database"]
```

### 1.6 Frontend `package.json` (`app/package.json`)

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

### 1.7 `vite.config.ts`

```typescript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte()],

  resolve: {
    alias: {
      '$lib': path.resolve(__dirname, './src/lib'),
      '@': path.resolve(__dirname, './src')
    }
  },

  // CRITICAL: Base path for Tauri
  base: './',

  clearScreen: false,

  server: {
    port: 5173,
    strictPort: true,
    host: '0.0.0.0', // Allow access from Tauri webview
  },

  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
```

### 1.8 `tailwind.config.js`

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx,svelte}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        'app-bg': '#1e1e1e',
        'app-text': '#e0e0e0',
        'menu': '#2d2d2d',
        'window': '#252525',
        'window-border': '#3e3e3e',
        'window-subtle': '#1f1f1f',
        'primary': '#3498db',
        'primary-dark': '#2980b9',
        'secondary': '#95a5a6',
        'secondary-dark': '#7f8c8d',
        'success': '#27ae60',
        'error': '#e74c3c',
        'error-dark': '#c0392b',
        'hover': 'rgba(52, 152, 219, 0.1)',
        'input': '#2a2a2a',
        'gray-300': '#b0b0b0',
        'gray-400': '#808080',
        'gray-500': '#606060',
        'green-500': '#27ae60',
      },
    },
  },
  plugins: [],
}
```

### 1.9 `postcss.config.js`

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

### 1.10 `tauri.conf.json`

```json
{
  "$schema": "https://schema.tauri.app/config/2.0.0",
  "productName": "MIDI Software Center",
  "version": "1.0.0",
  "identifier": "com.midisoftwarecenter.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "MIDI Software Center",
        "width": 1920,
        "height": 1280,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "alwaysOnTop": false
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

---

## 2. Database Configuration

### 2.1 Database Connection

**Environment Variables (not in repo, create `.env` file):**

```env
# PostgreSQL Connection
DATABASE_URL=postgresql://midiuser:145278963@localhost:5433/midi_library

# Pool Configuration
DB_MAX_CONNECTIONS=10

# Meilisearch (for full-text search)
MEILI_HOST=http://localhost:7700
MEILI_MASTER_KEY=your_master_key_here

# Logging
LOG_DIR=./logs
RUST_LOG=info,midi_app=debug,midi_pipeline=debug,midi_daw=debug
```

### 2.2 Database Schema Overview

**15 Tables:**
1. `files` - Core file metadata
2. `musical_metadata` - BPM, key, duration
3. `tags` - Tag definitions
4. `file_tags` - Many-to-many relationship
5. `midi_tracks` - Track information
6. `midi_events` - Event data
7. `analysis_results` - Enhanced analysis
8. `chords` - Chord progressions
9. `drum_patterns` - Drum-specific analysis
10. `search_index` - Meilisearch integration
11. `import_batches` - Batch tracking
12. `corruption_log` - Auto-repair tracking
13. `deduplication_log` - Duplicate tracking
14. `performance_metrics` - Pipeline performance
15. `user_collections` - Custom playlists

**60+ Indexes for Performance**

### 2.3 SQLx Configuration

**Runtime:** `runtime-tokio-rustls`
**Features:** `postgres`, `chrono`, `uuid`, `bigdecimal`, `migrate`

**Connection Pattern:**
```rust
let pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(max_connections)
    .connect(database_url)
    .await?;
```

---

## 3. Build & Deployment Scripts

### 3.1 Makefile Commands

```bash
# Setup
make setup          # Install all dependencies
make docker-up      # Start PostgreSQL container

# Development
make dev-pipeline   # Run pipeline (port 5173)
make dev-daw        # Run DAW (port 5174)
make dev-both       # Run both applications
make dev-cpu        # Run with software rendering (no GPU)

# Building
make build-pipeline # Build pipeline for production
make build-daw      # Build DAW for production
make build-all      # Build both applications

# Testing
make test           # Run all tests
make test-rust      # Run Rust tests only
make test-baseline  # Run baseline library tests
make test-coverage  # Generate coverage report
make test-quick     # Quick smoke tests

# Code Quality
make format         # Format all code
make lint           # Lint all code (clippy + eslint)
make check          # Run all checks

# Database
make db-migrate     # Run migrations
make db-reset       # Reset database (destructive)
make db-backup      # Backup database

# Performance
make bench          # Run benchmarks
make pgo-build      # Profile-Guided Optimization build

# Release
make release        # Build release versions
```

### 3.2 CI/CD Pipeline (`.github/workflows/ci.yml`)

```yaml
name: CI with AI Analysis

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Build
      run: cargo build --verbose

    - name: Run Tests
      run: cargo test --verbose
```

### 3.3 Desktop Packaging

**Output Locations:**
- `pipeline/src-tauri/target/release/bundle/`
- `daw/src-tauri/target/release/bundle/`

**Supported Formats:**
- Windows: `.msi`, `.exe`
- macOS: `.dmg`, `.app`
- Linux: `.AppImage`, `.deb`

---

## 4. Audio/MIDI Specific Configs

### 4.1 Audio Settings (`daw/src-tauri/src/settings/audio.rs`)

```rust
pub enum BufferSize {
    Samples32 = 32,
    Samples64 = 64,
    Samples128 = 128,
    Samples256 = 256,
    Samples512 = 512,    // DEFAULT
    Samples1024 = 1024,
    Samples2048 = 2048,
    Samples4096 = 4096,
}

pub enum SampleRate {
    Hz44100 = 44100,
    Hz48000 = 48000,     // DEFAULT
    Hz88200 = 88200,
    Hz96000 = 96000,
    Hz176400 = 176400,
    Hz192000 = 192000,
}

pub struct AudioSettings {
    pub buffer_size: BufferSize,      // Default: 512 samples
    pub sample_rate: SampleRate,      // Default: 48000 Hz
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub latency_monitoring_enabled: bool,
}
```

**Default Latency:** ~10.67ms (512 samples @ 48kHz)

### 4.2 MIDI Settings (`daw/src-tauri/src/settings/midi.rs`)

```rust
pub enum SyncMode {
    Internal,    // DEFAULT - Internal clock
    External,    // Sync to external MIDI clock
}

pub struct MidiSettings {
    pub default_input_device: Option<String>,
    pub default_output_device: Option<String>,
    pub sync_mode: SyncMode,
    pub tempo_sync_enabled: bool,     // Default: false
    pub flush_notes_on_stop: bool,    // Default: true
}
```

### 4.3 MIDI Library

**Library:** `midir = "0.8"` (cross-platform MIDI I/O)

**Supported Backends:**
- Windows: Windows MIDI (WinMM)
- macOS: CoreMIDI
- Linux: ALSA

**Note:** No audio playback library (`cpal`) - MIDI-only output to external devices

### 4.4 MIDI File Processing

**Library:** `midly = "0.5"` (zero-copy MIDI parser)

**Supported Features:**
- MIDI Format 0, 1, 2
- All standard MIDI events
- Meta events (tempo, time signature, key signature)
- SysEx messages

---

## 5. UI/UX Specifications

### 5.1 Design System (Tailwind Config)

**Color Palette (Dark Theme):**

| Name | Hex | Usage |
|------|-----|-------|
| `app-bg` | `#1e1e1e` | Main background |
| `app-text` | `#e0e0e0` | Primary text |
| `menu` | `#2d2d2d` | Menu backgrounds |
| `window` | `#252525` | Window backgrounds |
| `window-border` | `#3e3e3e` | Borders |
| `window-subtle` | `#1f1f1f` | Subtle backgrounds |
| `primary` | `#3498db` | Primary actions |
| `primary-dark` | `#2980b9` | Primary hover |
| `secondary` | `#95a5a6` | Secondary elements |
| `success` | `#27ae60` | Success states |
| `error` | `#e74c3c` | Error states |
| `input` | `#2a2a2a` | Input backgrounds |

### 5.2 Typography

**Font:** System default (no custom fonts)
**Scale:** Tailwind default (`text-xs`, `text-sm`, `text-base`, etc.)

### 5.3 Icons

**Current:** Unicode emoji and text symbols
- Favorite: `★` / `☆`
- Delete: `🗑`
- Mute: `M`
- Solo: `S`

**No external icon library** - can add Lucide or Heroicons if needed

### 5.4 Window Dimensions

```json
{
  "width": 1920,
  "height": 1280,
  "resizable": true,
  "fullscreen": false
}
```

---

## 6. Performance Requirements

### 6.1 System Requirements

```yaml
minimum_requirements:
  ram: "8GB"
  cpu_cores: 4
  disk_space: "500MB app + 100GB library"

recommended_requirements:
  ram: "16GB"
  cpu_cores: 8
  disk_space: "500MB app + 500GB library"
```

### 6.2 Performance Targets

```yaml
import_performance:
  target: 7830 files/sec
  achieved: 7830 files/sec (LUDICROUS mode)

analysis_performance:
  target: 100 files/sec
  achieved: 181-360 files/sec

query_performance:
  simple_tag: < 10ms
  complex_multi_tag: < 100ms
  full_category_scan: < 500ms

audio_latency:
  target: < 20ms
  default: ~10.67ms (512 samples @ 48kHz)
  minimum: ~0.67ms (32 samples @ 48kHz)

ui_refresh:
  target: 60fps
  meter_update: 100ms intervals
```

### 6.3 File Size Limits

```yaml
midi_file:
  max_size: "No hard limit"
  typical_size: "50KB - 5MB"
  largest_tested: "50MB+"

database:
  current_files: 1.72M unique
  storage: "~71GB files + 3-5GB database"

cache:
  no_explicit_cache_limits: true
  uses_database_connection_pool: true
```

### 6.4 Memory Constraints

```yaml
allocators:
  pipeline: "tikv-jemallocator, mimalloc"
  arena: "bumpalo, typed-arena"

connection_pool:
  default_max: 10
  configurable: "DB_MAX_CONNECTIONS env var"
```

---

## 7. Testing Requirements

### 7.1 Test Frameworks

**Rust:**
- Built-in `#[test]`
- `criterion` for benchmarks
- `proptest` for property-based testing
- `tokio-test` for async tests

**Frontend:**
- `vitest` for unit tests
- `@playwright/test` for E2E
- `@testing-library/svelte` for components

### 7.2 Test Commands

```bash
# Rust tests
cargo test --workspace --lib -- --test-threads=1    # All library tests
cargo test --workspace                               # All tests
cargo tarpaulin --workspace --out Html              # Coverage

# Frontend tests
pnpm test                                           # Vitest
pnpm test:ui                                        # Vitest UI

# Full suite
make test           # Both Rust and frontend
make test-quick     # Quick smoke tests
make test-baseline  # Library tests only
```

### 7.3 Test Coverage

**Current:** 54.53% (1,223+ tests)
**Target:** 80% minimum, 100% goal

**Phase Status:**
- Phase 0-4: Complete (baseline tests)
- Phase 5-9: Complete (real-world validation)
- Phase 10+: Ongoing improvements

### 7.4 Test Data

**Sample MIDI Files:**
- Located in `scripts/test-midi-files/`
- 1,603 production files tested
- Various formats, sizes, complexities

---

## 8. Platform-Specific Requirements

### 8.1 Windows

```yaml
audio_backend: "Windows MIDI (WinMM via midir)"
midi_backend: "Windows MIDI"
installer: ".msi, .exe (via Tauri)"
min_version: "Windows 10"
build_target: "chrome105"
```

### 8.2 macOS

```yaml
audio_backend: "CoreMIDI (via midir)"
midi_backend: "CoreMIDI"
installer: ".dmg, .app (via Tauri)"
min_version: "macOS 10.13 (High Sierra)"
build_target: "safari13"
notarization: "Not configured"
```

### 8.3 Linux

```yaml
audio_backend: "ALSA (via midir)"
midi_backend: "ALSA"
installer: ".AppImage, .deb (via Tauri)"
dependencies:
  - "libgtk-3-dev"
  - "libwebkit2gtk-4.0-dev"
  - "libasound2-dev"
special_mode: "CPU rendering: WEBKIT_DISABLE_COMPOSITING_MODE=1"
```

### 8.4 CPU Rendering Mode (Linux)

```bash
# For systems without GPU
WEBKIT_DISABLE_COMPOSITING_MODE=1 \
WEBKIT_DISABLE_DMABUF_RENDERER=1 \
LIBGL_ALWAYS_SOFTWARE=1 \
pnpm tauri dev
```

---

## 9. Error Reporting & Analytics

### 9.1 Current Status

**Error Tracking:** NOT configured (no Sentry/Bugsnag)

**Logging:**
```rust
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-appender = "0.2"  // File logging

// Log levels
RUST_LOG=info,midi_app=debug,midi_pipeline=debug,midi_daw=debug
```

**Log Files:** `./logs/midi-app.log` (daily rotation)

### 9.2 User Analytics

**Current:** None implemented

**Privacy Considerations:**
- No telemetry
- No usage tracking
- Local-only operation

---

## 10. External Integrations

### 10.1 Search Engine

**Meilisearch:**
```yaml
version: "0.54.0 (SDK)"
host: "http://localhost:7700"
features:
  - Full-text search
  - Typo tolerance
  - Faceted filtering
```

### 10.2 Plugin Support

**VST/AU:** NOT supported (MIDI-only, no audio plugins)

### 10.3 Cloud Sync

**Current:** NOT implemented (local-only)

---

## 11. Accessibility Requirements

### 11.1 Current Status

**WCAG Compliance:** Not audited

**Implemented:**
- Dark mode by default
- Keyboard navigation (basic)
- Screen reader: Not tested

**Missing:**
- High contrast mode
- Focus indicators
- ARIA labels
- Reduced motion

### 11.2 Internationalization

**Current:** English only

**Status:**
- No i18n framework
- No RTL support
- Hardcoded strings

---

## 12. Security Requirements

### 12.1 Code Signing

**Current:** NOT configured

**Requirements for distribution:**
- Windows: Code signing certificate
- macOS: Developer ID + notarization
- Linux: No signing required

### 12.2 Data Security

```yaml
database_password: "Stored in .env (not in repo)"
encryption: "None (local database)"
secure_storage: "Not implemented"
```

### 12.3 Content Security Policy

```json
{
  "security": {
    "csp": null  // Disabled for development
  }
}
```

---

## 13. Documentation Requirements

### 13.1 Code Documentation

**Rust:**
- `rustdoc` comments (`///`)
- Module-level docs (`//!`)
- No external doc generator

**TypeScript:**
- JSDoc comments
- TypeScript interfaces

### 13.2 User Documentation

**Current:** Markdown files in `/docs`

**Key Documents:**
- `CLAUDE.md` - Project overview
- `ARCHITECTURE-REFERENCE.md` - Architecture patterns
- `PROJECT-STRUCTURE.md` - Directory structure
- `DEVELOPMENT-WORKFLOW.md` - Development process

### 13.3 API Documentation

**Backend:** Inline Rust docs
**Frontend:** TypeScript types in `types.ts`
**No generated API docs**

---

## Summary: What Can Be Assumed

Based on this configuration:

1. **Tauri 2.0** with standard features
2. **Svelte 4.2** + **Vite 5.0** + **TypeScript 5.3**
3. **Tailwind 3.4** with dark theme
4. **PostgreSQL 16** via **sqlx 0.7**
5. **MIDI via midir** (no audio playback)
6. **Target:** Desktop (Windows/macOS/Linux)
7. **No cloud features** (local-only)
8. **No analytics/telemetry**
9. **English only** (no i18n)
10. **1.72M files** scale

---

*Document provides complete configuration reference for production-ready development. Last updated: 2025-12-01*
