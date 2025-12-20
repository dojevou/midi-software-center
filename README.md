# MIDI Software Center

A high-performance MIDI library management system for organizing, analyzing, and working with large-scale MIDI collections.

## Overview

MIDI Software Center manages **2.15M+ MIDI files** with advanced musical analysis, deduplication, and organization capabilities. Built with Rust and Tauri for maximum performance.

### Key Features

- **High-Speed Import**: 7,830 files/sec with automatic deduplication
- **Musical Analysis**: BPM detection, key detection, chord analysis, drum pattern recognition
- **Smart Organization**: 97 instrument categories, 1,861 tag types, 7.9M+ tags applied
- **Auto-Repair**: Fixes corrupted MIDI files automatically (99.5% success rate)
- **Track Splitting**: Separates multi-track files into individual tracks
- **Full-Text Search**: Meilisearch integration for instant search
- **DAW Integration**: Real-time MIDI I/O, sequencer, piano roll editor

### Performance

| Operation | Speed | Industry Comparison |
|-----------|-------|---------------------|
| Import | 7,830 files/sec | 150-780x faster |
| Analysis | 181-360 files/sec | 3-7x faster |
| Hash Calculation | 88,656 files/sec | BLAKE3 |
| Query Performance | < 10ms | Indexed PostgreSQL |
| Deduplication | 73.4% | 4.74M duplicates removed |

## Architecture

```
midi-software-center/
├── app/                    # Main Tauri application (unified)
│   ├── src/               # Svelte/TypeScript frontend
│   └── src-tauri/         # Rust backend (midi-software-center)
├── database/               # PostgreSQL migrations and schemas
├── scripts/                # Automation and utility scripts
│   ├── import-tool/       # CLI import utilities
│   └── test-midi-files/   # MIDI testing tool
└── verification/           # Verification suite
```

### Unified Application

The application combines all functionality in a single codebase:

| Feature Area | Description |
|--------------|-------------|
| **Pipeline** | Batch import, archive extraction, analysis |
| **DAW** | Real-time MIDI, hardware I/O, sequencer |
| **Analysis** | BPM/key detection, chord analysis, drum patterns |
| **Database** | PostgreSQL integration, repositories, batch operations |

### Technology Stack

**Backend:**
- Rust 2021 Edition with Tokio async runtime
- Tauri 2.0 for desktop application framework
- SQLx 0.7 for type-safe database queries
- midly 0.5 for MIDI parsing

**Frontend:**
- Svelte 5 with TypeScript 5.3
- Vite 5.0 build system
- Tailwind CSS

**Database:**
- PostgreSQL 16 with 15 tables and 60+ indexes
- pgvector for similarity search
- Meilisearch 1.5 for full-text search

## Quick Start

### Prerequisites

- Rust 1.75+
- Node.js 20+ and pnpm 8+
- PostgreSQL 16
- Docker (optional, for containerized database)

### Installation

```bash
# Clone the repository
git clone https://github.com/dojevou/midi-software-center.git
cd midi-software-center

# Setup (installs dependencies, starts database)
make setup
make docker-up

# Run database migrations
make db-migrate

# Start development server
make dev    # Launches application at :5173
```

### Running the Pipeline

```bash
# Import MIDI files (ultra-fast mode)
./scripts/run-pipeline-ultra-fast.sh /path/to/midi/files

# Apply instrument organization (97 categories)
./scripts/organize-database.sh

# Monitor progress
./scripts/monitor-pipeline.sh

# Check database statistics
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -c "SELECT COUNT(*) as total_files FROM files"
```

## Database Schema

The system uses PostgreSQL with 15 tables:

| Table | Purpose |
|-------|---------|
| `files` | Core file metadata (path, hash, size) |
| `musical_metadata` | BPM, key, duration, time signature |
| `tags` | Tag definitions (1,861 types) |
| `file_tags` | File-to-tag relationships (7.9M+) |
| `midi_tracks` | Track information per file |
| `midi_events` | Individual MIDI events |
| `analysis_results` | Enhanced analysis (chords, structure) |
| `drum_patterns` | Drum-specific analysis |
| `chords` | Chord progressions |

### Example Queries

```sql
-- Find all drum files at 120 BPM
SELECT f.filename, m.bpm, m.key_signature
FROM files f
JOIN musical_metadata m ON f.id = m.file_id
JOIN file_tags ft ON f.id = ft.file_id
JOIN tags t ON ft.tag_id = t.id
WHERE t.name = 'drums' AND m.bpm BETWEEN 118 AND 122;

-- Get files by instrument (built-in function)
SELECT * FROM get_files_by_instrument('piano');

-- Get files by BPM range
SELECT * FROM get_files_by_bpm_range(118, 122);

-- Search by multiple instruments
SELECT * FROM get_files_by_instruments(ARRAY['jazz', 'piano']);
```

## Development

### Make Commands

```bash
# Development
make dev               # Start dev server

# Building
make build-all         # Build all components
make build-release     # Production build

# Testing
make test              # Run all tests
make check             # Lint and type check
make format            # Format code

# Database
make db-migrate        # Run migrations
make db-backup         # Backup database
make db-reset          # Reset database (destructive!)
```

### Running Tests

```bash
# All workspace tests
cargo test --workspace --lib

# Main application tests
cargo test --package midi-software-center --lib

# Verification suite
cargo test --package verification

# With coverage
cargo tarpaulin --workspace --out Html
```

**Current Test Status:** 1,999 tests passing

### Key Files

| Purpose | Location |
|---------|----------|
| MIDI Parser | `app/src-tauri/src/core/midi/analysis_parser.rs` |
| BPM Detector | `app/src-tauri/src/core/analysis/bpm_detector.rs` |
| Key Detector | `app/src-tauri/src/core/analysis/key_detector.rs` |
| Auto-tagger | `app/src-tauri/src/core/analysis/auto_tagger.rs` |
| Drum Analyzer | `app/src-tauri/src/core/analysis/drum_analyzer.rs` |
| File Repository | `app/src-tauri/src/db/repositories/file_repository.rs` |
| Sequencer Engine | `app/src-tauri/src/sequencer/engine.rs` |
| Hardware Manager | `app/src-tauri/src/hardware/device_manager.rs` |

## Pipeline Phases

| Phase | Purpose | Speed |
|-------|---------|-------|
| 1. Import | Ingest + hash + parse + tag + index | 7,830/sec |
| 2. Sanitize | Clean filenames | Instant |
| 3. Split | Multi-track separation + auto-repair | Batch |
| 4. Analyze | BPM, key, drums, chords, structure | 181-360/sec |
| 5. Rename | Metadata-based filenames | Batch |

**Recommended Order:** Import -> Sanitize -> Split -> Analyze -> Rename

## Configuration

### Environment Variables

```bash
DATABASE_URL=postgresql://midiuser:password@localhost:5433/midi_library
MEILISEARCH_URL=http://localhost:7700
RUST_LOG=info
```

### Docker Compose

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f postgres

# Stop services
docker-compose down
```

## API Reference

### Tauri Commands

The application exposes commands via Tauri's IPC system:

```typescript
// Import files
await invoke('import_files', { paths: ['/path/to/midi'] });

// Search files
await invoke('search_files', {
  query: 'drum loop',
  bpmMin: 118,
  bpmMax: 122
});

// Get file details
await invoke('get_file_details', { fileId: 12345 });

// Analyze file
await invoke('analyze_file', { fileId: 12345 });
```

### Repository Methods

```rust
// File operations
file_repository.insert_file(&file).await?;
file_repository.find_by_hash(&hash).await?;
file_repository.search(&query, limit, offset).await?;

// Metadata operations
metadata_repository.get_by_file_id(file_id).await?;
metadata_repository.update_bpm(file_id, bpm).await?;

// Tag operations
tag_repository.add_tags(file_id, &tags).await?;
tag_repository.get_files_by_tag("drums").await?;
```

## New Features (In Development)

The application is being enhanced with professional DAW capabilities and advanced browsing features. See `docs/PARALLEL_WORK_STREAMS.md` for implementation details.

### VIP3 Browser (Stream A, C)

Advanced MIDI file browser with real-time filtering:

- **Filter Counts**: Live counts for each filter showing available matches (<50ms response)
- **Multi-Column Filtering**: Folders, Instruments, BPM, Key, Time Signature, Track Count
- **Saved Searches**: Save and reload filter combinations
- **Collections**: Organize files into custom collections
- **Favorites**: Quick access to frequently used files
- **Drag & Drop**: Direct integration with DAW sequencer

**Status:** ~40% complete (filter counts + collections)

### DAW Mixer (Stream B, D)

Professional mixing capabilities:

- **30+ Mixer Commands**: Gain, pan, mute, solo, effects, routing
- **Real-time VU Meters**: 60 FPS peak/RMS metering with clipping detection
- **Effect Chain**: EQ, Compressor, Reverb, Delay (per track)
- **Automation System**: Full automation with Write/Read/Latch/Touch modes
- **Preset Management**: Save and recall mixer configurations
- **Bus Routing**: Flexible routing matrix with aux sends/returns

**Performance Targets:**
- Mixer operations: <10ms
- Meter updates: <1ms (60 Hz)
- 100+ tracks supported

**Status:** ~30% complete (basic mixer + sequencer)

### Project Management (Stream E)

- Save/load complete projects
- Export to MIDI (Type 0 and Type 1)
- Project metadata and version history
- Auto-save and backup

**Status:** Not started

### Integration Features (Stream F)

- Drag files from VIP3 Browser to DAW
- Multi-file batch loading
- Auto-arrange tracks

**Status:** Not started

## Testing & Quality Assurance

### Test Infrastructure (Stream G)

Comprehensive testing framework prepared for all features:

```bash
# Unit tests (Rust)
app/src-tauri/tests/templates/
├── mixer_test.rs           # 30+ mixer commands
├── automation_test.rs      # Automation engine
├── project_test.rs         # Project save/load
├── collections_test.rs     # VIP3 collections
└── filter_counts_test.rs   # Filter count queries

# Integration tests
app/src-tauri/tests/integration/
└── full_workflow_test.rs   # End-to-end workflows

# Frontend tests (TypeScript)
app/src/lib/components/__tests__/templates/
├── VIP3Browser.test.ts     # VIP3 UI
├── MixerWindow.test.ts     # Mixer UI
└── AutomationLane.test.ts  # Automation UI

# Performance benchmarks
scripts/benchmarks/
├── benchmark-filter-counts.sh        # <50ms target
├── benchmark-mixer-processing.sh     # <10ms target
└── benchmark-automation-playback.sh  # <1ms target
```

### Running Tests

```bash
# Run all unit tests
cargo test --workspace --lib

# Run specific test suite
cargo test mixer_test

# Run integration tests
cargo test --test integration

# Run benchmarks
./scripts/benchmarks/benchmark-filter-counts.sh

# Frontend tests
cd app && pnpm test
```

### Coverage Targets

- **Overall Coverage**: >80%
- **Critical Paths**: >95% (mixer, automation, filter counts)
- **Performance Tests**: All operations meet targets

### Documentation

- **User Guide**: `docs/USER_GUIDE.md` - Complete usage documentation
- **API Reference**: `docs/API_REFERENCE.md` - All Tauri commands
- **Testing Guide**: `docs/TESTING_QUICK_START.md` - Test execution
- **Implementation Plan**: `docs/PARALLEL_WORK_STREAMS.md` - Development roadmap

## License

This project is proprietary software. All rights reserved.

---

**Status:** Production Ready (Core) | In Development (DAW/VIP3) | **Tests:** 1,999 passing | **Files Managed:** 2.15M+ | **Tags:** 7.9M+
