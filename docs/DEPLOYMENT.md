# MIDI Software Center - Deployment Guide

## Prerequisites

- **System:** Linux (Ubuntu 22.04+), macOS, or Windows with WSL2
- **Rust:** 1.70+ with cargo
- **Node.js:** 18+ with pnpm
- **PostgreSQL:** 16+ (running on port 5433)
- **Storage:** 100GB+ for MIDI library

## Production Binaries

```bash
# Build all release binaries
cargo build --release

# Verify binaries
ls -la target/release/midi-*
```

**Available binaries:**
- `midi-software-center` - Main application (16MB)
- `midi-pipeline` - Import/analysis pipeline (14MB)
- `midi-software-center-daw` - DAW component (11MB)

## Database Setup

```bash
# Start PostgreSQL (Docker)
make docker-up

# Or manual PostgreSQL on port 5433
psql "postgresql://midiuser:145278963@localhost:5433/midi_library"

# Verify tables
\dt
# Should show: files, musical_metadata, tags, file_tags (15 tables total)
```

## Environment Variables

```bash
export DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
export MIDI_LIBRARY_PATH="/path/to/midi-library/files"
export RUST_LOG="info"
```

## Deployment Steps

### 1. Deploy Binaries

```bash
# Copy release binaries to /usr/local/bin
sudo cp target/release/midi-{software-center,pipeline,software-center-daw} /usr/local/bin/

# Set permissions
sudo chmod +x /usr/local/bin/midi-*
```

### 2. Database Migration

```bash
# Run migrations
cd database/migrations
for f in *.sql; do psql $DATABASE_URL -f "$f"; done
```

### 3. Start Services

```bash
# Development mode
make dev-both

# Production mode (manual)
midi-software-center &
midi-software-center-daw &
```

### 4. Import MIDI Files

```bash
# Ultra-fast import
./scripts/run-pipeline-ultra-fast.sh

# Or LUDICROUS SPEED (unsafe, import-only)
./scripts/LUDICROUS-SPEED-import.sh
```

## Performance Tuning

### PostgreSQL Settings (import mode)

```sql
ALTER SYSTEM SET synchronous_commit = 'off';
ALTER SYSTEM SET fsync = 'off';  -- DANGER: import only!
ALTER SYSTEM SET maintenance_work_mem = '2GB';
ALTER SYSTEM SET work_mem = '256MB';
SELECT pg_reload_conf();
```

### Rust Build Optimization

```bash
# Profile-guided optimization
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Health Checks

```bash
# Database connectivity
psql $DATABASE_URL -c "SELECT COUNT(*) FROM files"

# Binary versions
midi-pipeline --version
midi-software-center --version

# Test suite
cargo test --workspace --lib
```

## Rollback Procedure

```bash
# Database backup before major changes
pg_dump $DATABASE_URL > backup_$(date +%Y%m%d).sql

# Restore from backup
psql $DATABASE_URL < backup_YYYYMMDD.sql
```

## Monitoring

```bash
# Database stats
psql $DATABASE_URL -c "SELECT relname, n_live_tup FROM pg_stat_user_tables ORDER BY n_live_tup DESC"

# Import progress
psql $DATABASE_URL -c "SELECT COUNT(*) FROM files"

# Analysis coverage
psql $DATABASE_URL -c "SELECT COUNT(*) FROM musical_metadata"
```

## Docker Deployment

### Development Environment

```bash
# Start all services (PostgreSQL + Meilisearch)
docker-compose up -d

# Check service health
docker-compose ps

# View logs
docker-compose logs -f postgres
```

### CI Testing Environment

```bash
# Start test containers (isolated ports: 5434, 7701)
docker-compose -f docker-compose.test.yml up -d

# Run tests
cargo test --workspace --lib -- --test-threads=1

# Cleanup
docker-compose -f docker-compose.test.yml down -v
```

### Available Docker Profiles

| Profile | Services | Use Case |
|---------|----------|----------|
| default | postgres, meilisearch | Development |
| full | + redis, pipeline | Full stack |
| dev | + pipeline-dev | Hot-reload development |
| ci | test containers | CI/CD testing |

## CI/CD Pipeline

### GitHub Actions Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | push/PR to main | Tests + build verification |
| `lint.yml` | push/PR to main | Code formatting + clippy |
| `release.yml` | tag v* | Cross-platform installers |
| `test.yml` | push/PR | Additional test coverage |

### Creating a Release

```bash
# Tag a new version
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# GitHub Actions will automatically:
# 1. Create a draft release
# 2. Build Linux (.AppImage, .deb)
# 3. Build macOS (.dmg)
# 4. Build Windows (.msi, .exe)
# 5. Publish release with all artifacts
```

### Installer Formats

| Platform | Format | Size (approx) |
|----------|--------|---------------|
| Linux | .AppImage, .deb, .rpm | 20-25 MB |
| macOS | .dmg, .app | 25-30 MB |
| Windows | .msi, NSIS .exe | 25-30 MB |

### Bundle Configuration

Tauri bundle settings in `app/src-tauri/tauri.conf.json`:

- **Category:** Music
- **Copyright:** Copyright © 2025 MIDI Software Center
- **Minimum macOS:** 10.15 (Catalina)
- **Windows installer:** NSIS (per-user install)
- **Linux dependencies:** libasound2, libgtk-3-0, libwebkit2gtk-4.1-0

## Observability

### Sentry Integration

```rust
// Initialize in main.rs
let _guard = sentry::init((
    "YOUR_SENTRY_DSN",
    sentry::ClientOptions {
        release: sentry::release_name!(),
        traces_sample_rate: 0.1,
        ..Default::default()
    }
));
```

### Tracing

```bash
# Enable debug logging
export RUST_LOG="debug,midi_pipeline=trace"

# Production logging
export RUST_LOG="info,midi_pipeline=warn"
```

---
Generated: 2025-12-10 | MIDI Software Center v1.0.0
