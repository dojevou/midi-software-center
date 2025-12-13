# MIDI Software Center - Maintenance Procedures

This document describes routine maintenance tasks, upgrade procedures, and operational best practices.

## Overview

Regular maintenance ensures optimal system performance:
- **Daily** - Log review, health checks
- **Weekly** - Database optimization, backups
- **Monthly** - Security updates, performance review
- **Quarterly** - Full system audit, dependency updates

---

## Daily Maintenance

### Health Checks

```bash
#!/bin/bash
# scripts/daily-health-check.sh

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "=== MIDI Software Center Daily Health Check ==="
echo "Date: $(date)"
echo ""

# 1. Database connectivity
echo "1. Database Connection:"
if psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
    echo "   [OK] Database connected"
else
    echo "   [FAIL] Database connection failed!"
    exit 1
fi

# 2. File count
echo ""
echo "2. File Statistics:"
psql "$DATABASE_URL" -t -c "
SELECT
    'Total files: ' || COUNT(*) FROM files;
"
psql "$DATABASE_URL" -t -c "
SELECT
    'Analyzed: ' || COUNT(*) || ' (' || ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM files), 1) || '%)'
FROM musical_metadata;
"

# 3. Error log check
echo ""
echo "3. Recent Errors:"
if [ -f /tmp/import_log.txt ]; then
    ERROR_COUNT=$(grep -c "ERROR" /tmp/import_log.txt 2>/dev/null || echo 0)
    echo "   Import errors (last 24h): $ERROR_COUNT"
fi

# 4. Disk usage
echo ""
echo "4. Disk Usage:"
df -h /home | tail -1 | awk '{print "   Used: " $3 "/" $2 " (" $5 ")"}'

# 5. Database size
echo ""
echo "5. Database Size:"
psql "$DATABASE_URL" -t -c "
SELECT '   ' || pg_size_pretty(pg_database_size('midi_library'));
"

echo ""
echo "=== Health Check Complete ==="
```

### Log Review

```bash
# Check for errors in recent logs
grep -i "error\|fail\|panic" /tmp/import_log.txt | tail -20

# Check database logs (if available)
sudo tail -100 /var/log/postgresql/postgresql-16-main.log | grep -i error
```

---

## Weekly Maintenance

### Database Optimization

```bash
#!/bin/bash
# scripts/weekly-maintenance.sh

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "=== Weekly Database Maintenance ==="
echo "Started: $(date)"

# 1. VACUUM ANALYZE (reclaim space, update statistics)
echo "Running VACUUM ANALYZE..."
psql "$DATABASE_URL" -c "VACUUM ANALYZE;"

# 2. Reindex if needed (based on bloat)
echo "Checking index bloat..."
psql "$DATABASE_URL" -c "
SELECT
    schemaname || '.' || relname as table,
    indexrelname as index,
    pg_size_pretty(pg_relation_size(indexrelid)) as size,
    idx_scan as scans
FROM pg_stat_user_indexes
WHERE idx_scan < 50
ORDER BY pg_relation_size(indexrelid) DESC
LIMIT 10;
"

# 3. Check for unused indexes
echo "Checking unused indexes..."
psql "$DATABASE_URL" -c "
SELECT
    schemaname || '.' || relname as table,
    indexrelname as index,
    idx_scan as scans,
    pg_size_pretty(pg_relation_size(indexrelid)) as size
FROM pg_stat_user_indexes
WHERE idx_scan = 0
  AND indexrelname NOT LIKE '%_pkey'
ORDER BY pg_relation_size(indexrelid) DESC;
"

# 4. Check table bloat
echo "Checking table bloat..."
psql "$DATABASE_URL" -c "
SELECT
    relname as table,
    n_live_tup as live_rows,
    n_dead_tup as dead_rows,
    CASE WHEN n_live_tup > 0
         THEN ROUND(100.0 * n_dead_tup / n_live_tup, 1)
         ELSE 0 END as dead_pct
FROM pg_stat_user_tables
WHERE n_dead_tup > 10000
ORDER BY n_dead_tup DESC
LIMIT 10;
"

echo "Completed: $(date)"
```

### Backup Procedure

```bash
#!/bin/bash
# scripts/backup.sh

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
BACKUP_DIR="/home/dojevou/backups"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

echo "Creating backup: backup_$DATE.sql.gz"

# Full database backup with compression
pg_dump "$DATABASE_URL" | gzip > "$BACKUP_DIR/backup_$DATE.sql.gz"

# Keep only last 7 daily backups
find "$BACKUP_DIR" -name "backup_*.sql.gz" -mtime +7 -delete

# Verify backup
if gzip -t "$BACKUP_DIR/backup_$DATE.sql.gz"; then
    echo "Backup verified successfully"
    ls -lh "$BACKUP_DIR/backup_$DATE.sql.gz"
else
    echo "ERROR: Backup verification failed!"
    exit 1
fi
```

---

## Monthly Maintenance

### Security Updates

```bash
#!/bin/bash
# scripts/monthly-security.sh

echo "=== Monthly Security Maintenance ==="

# 1. Update Rust toolchain
echo "1. Updating Rust..."
rustup update stable

# 2. Check for security advisories
echo ""
echo "2. Checking cargo-audit..."
if ! command -v cargo-audit &> /dev/null; then
    cargo install cargo-audit
fi
cargo audit

# 3. Update dependencies (review Cargo.lock changes)
echo ""
echo "3. Checking outdated dependencies..."
cargo outdated --workspace

# 4. Check PostgreSQL version
echo ""
echo "4. PostgreSQL version:"
psql "$DATABASE_URL" -c "SELECT version();"
```

### Performance Review

```bash
#!/bin/bash
# scripts/monthly-performance.sh

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "=== Monthly Performance Review ==="

# 1. Database statistics
echo "1. Table Statistics:"
psql "$DATABASE_URL" -c "
SELECT
    relname as table,
    n_live_tup as rows,
    pg_size_pretty(pg_total_relation_size(relid)) as total_size,
    COALESCE(seq_scan, 0) as seq_scans,
    COALESCE(idx_scan, 0) as idx_scans
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(relid) DESC
LIMIT 15;
"

# 2. Cache hit ratio
echo ""
echo "2. Cache Hit Ratio:"
psql "$DATABASE_URL" -c "
SELECT
    ROUND(100.0 * blks_hit / NULLIF(blks_hit + blks_read, 0), 2) as cache_hit_ratio_pct
FROM pg_stat_database
WHERE datname = 'midi_library';
"

# 3. Slow queries (if pg_stat_statements is enabled)
echo ""
echo "3. Slowest Queries (avg execution time):"
psql "$DATABASE_URL" -c "
SELECT
    ROUND(mean_exec_time::numeric, 2) as avg_ms,
    calls,
    LEFT(query, 80) as query_preview
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = 'midi_library')
ORDER BY mean_exec_time DESC
LIMIT 10;
" 2>/dev/null || echo "   pg_stat_statements not enabled"

# 4. Index usage
echo ""
echo "4. Most Used Indexes:"
psql "$DATABASE_URL" -c "
SELECT
    indexrelname as index,
    idx_scan as scans,
    idx_tup_read as tuples_read
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC
LIMIT 10;
"
```

---

## Quarterly Maintenance

### Full System Audit

```bash
#!/bin/bash
# scripts/quarterly-audit.sh

echo "=== Quarterly System Audit ==="
echo "Date: $(date)"
echo ""

# 1. Code quality
echo "1. Code Quality Checks:"
make check 2>&1 | tail -20

# 2. Test suite
echo ""
echo "2. Test Suite:"
cargo test --workspace --lib 2>&1 | grep "test result" | tail -5

# 3. Build verification
echo ""
echo "3. Release Build:"
cargo build --release 2>&1 | tail -5

# 4. Documentation check
echo ""
echo "4. Documentation Files:"
ls -la docs/*.md | wc -l
echo "   documentation files present"

# 5. Dependency audit
echo ""
echo "5. Dependency Security:"
cargo audit 2>&1 | tail -10

# 6. Database integrity
echo ""
echo "6. Database Integrity:"
psql "$DATABASE_URL" -c "
SELECT
    'files' as table,
    COUNT(*) as rows,
    COUNT(*) - COUNT(DISTINCT id) as duplicates
FROM files
UNION ALL
SELECT
    'musical_metadata',
    COUNT(*),
    COUNT(*) - COUNT(DISTINCT file_id)
FROM musical_metadata
UNION ALL
SELECT
    'file_tags',
    COUNT(*),
    0
FROM file_tags;
"
```

### Dependency Updates

```bash
#!/bin/bash
# scripts/update-dependencies.sh

echo "=== Dependency Update Procedure ==="

# 1. Create backup branch
git checkout -b deps-update-$(date +%Y%m%d)

# 2. Update Cargo dependencies
echo "Updating Cargo dependencies..."
cargo update

# 3. Update npm dependencies
echo "Updating npm dependencies..."
cd app && pnpm update && cd ..

# 4. Run tests
echo "Running tests..."
cargo test --workspace --lib

# 5. Check for breaking changes
echo "Checking clippy..."
cargo clippy --workspace 2>&1 | grep -E "^error" | head -20

echo ""
echo "Review changes in Cargo.lock and package.json"
echo "If tests pass, merge to main"
```

---

## Database Recovery

### Restore from Backup

```bash
#!/bin/bash
# scripts/restore-database.sh

BACKUP_FILE=$1
DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file.sql.gz>"
    exit 1
fi

echo "WARNING: This will DESTROY all current data!"
read -p "Continue? (type 'yes' to confirm): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

# 1. Drop and recreate database
echo "Dropping database..."
psql "postgresql://midiuser:145278963@localhost:5433/postgres" -c "
DROP DATABASE IF EXISTS midi_library;
CREATE DATABASE midi_library;
"

# 2. Restore from backup
echo "Restoring from $BACKUP_FILE..."
gunzip -c "$BACKUP_FILE" | psql "$DATABASE_URL"

# 3. Verify restore
echo "Verifying restore..."
psql "$DATABASE_URL" -c "SELECT COUNT(*) as files FROM files;"

echo "Restore complete."
```

### Point-in-Time Recovery (PITR)

For critical deployments, configure PostgreSQL WAL archiving:

```sql
-- postgresql.conf settings for PITR
-- archive_mode = on
-- archive_command = 'cp %p /var/lib/postgresql/wal_archive/%f'
-- wal_level = replica

-- To restore to specific point in time:
-- recovery_target_time = '2025-12-10 14:00:00'
```

---

## Index Maintenance

### Rebuild Indexes

```bash
#!/bin/bash
# scripts/rebuild-indexes.sh

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

echo "=== Index Rebuild ==="
echo "This may take several minutes for large tables"

# Reindex all tables
psql "$DATABASE_URL" -c "REINDEX DATABASE midi_library;"

# Alternative: Concurrent reindex (no locks, PostgreSQL 12+)
# psql "$DATABASE_URL" -c "REINDEX DATABASE CONCURRENTLY midi_library;"

echo "Index rebuild complete"
```

### Index Analysis

```sql
-- Find missing indexes (tables with high sequential scans)
SELECT
    relname as table,
    seq_scan as seq_scans,
    seq_tup_read as seq_rows_read,
    idx_scan as idx_scans,
    CASE WHEN idx_scan > 0
         THEN ROUND(100.0 * seq_scan / (seq_scan + idx_scan), 1)
         ELSE 100 END as seq_scan_pct
FROM pg_stat_user_tables
WHERE seq_scan > 1000
ORDER BY seq_scan DESC
LIMIT 10;

-- Index bloat estimation
SELECT
    schemaname || '.' || relname as table,
    indexrelname as index,
    pg_size_pretty(pg_relation_size(indexrelid)) as size,
    idx_scan as scans
FROM pg_stat_user_indexes
ORDER BY pg_relation_size(indexrelid) DESC
LIMIT 20;
```

---

## Upgrade Procedures

### Application Upgrade

```bash
#!/bin/bash
# scripts/upgrade-application.sh

echo "=== Application Upgrade ==="

# 1. Create backup
echo "Creating backup..."
./scripts/backup.sh

# 2. Pull latest code
echo "Pulling latest code..."
git pull origin main

# 3. Update dependencies
echo "Updating dependencies..."
cargo update
cd app && pnpm install && cd ..

# 4. Run migrations
echo "Running migrations..."
make db-migrate

# 5. Build release
echo "Building release..."
cargo build --release

# 6. Run tests
echo "Running tests..."
cargo test --workspace --lib

# 7. Deploy binaries
echo "Deploying binaries..."
sudo cp target/release/midi-* /usr/local/bin/

echo "Upgrade complete. Restart services to apply."
```

### Database Migration

```bash
#!/bin/bash
# scripts/migrate-database.sh

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"
MIGRATIONS_DIR="database/migrations"

echo "=== Database Migration ==="

# 1. Backup before migration
./scripts/backup.sh

# 2. Apply new migrations
for migration in $(ls -1 "$MIGRATIONS_DIR"/*.sql | sort); do
    echo "Applying: $migration"
    psql "$DATABASE_URL" -f "$migration"
    if [ $? -ne 0 ]; then
        echo "ERROR: Migration failed!"
        exit 1
    fi
done

echo "Migrations complete."
```

---

## Troubleshooting Procedures

### Database Connection Issues

```bash
# 1. Check PostgreSQL status
sudo systemctl status postgresql

# 2. Check port binding
ss -tlnp | grep 5433

# 3. Check pg_hba.conf for access rules
sudo cat /etc/postgresql/16/main/pg_hba.conf | grep -v "^#" | grep -v "^$"

# 4. Test connection manually
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "SELECT 1"
```

### High Disk Usage

```bash
# 1. Find large tables
psql "$DATABASE_URL" -c "
SELECT
    relname as table,
    pg_size_pretty(pg_total_relation_size(relid)) as size
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(relid) DESC
LIMIT 10;
"

# 2. Clean up dead tuples
psql "$DATABASE_URL" -c "VACUUM FULL;"

# 3. Archive old logs
find /tmp -name "*.log" -mtime +7 -exec gzip {} \;
```

### Performance Degradation

```bash
# 1. Check for long-running queries
psql "$DATABASE_URL" -c "
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE state != 'idle'
ORDER BY duration DESC
LIMIT 5;
"

# 2. Kill stuck queries (if necessary)
# psql "$DATABASE_URL" -c "SELECT pg_terminate_backend(<pid>);"

# 3. Analyze statistics
psql "$DATABASE_URL" -c "ANALYZE;"

# 4. Check lock contention
psql "$DATABASE_URL" -c "SELECT * FROM pg_locks WHERE NOT granted;"
```

---

## Maintenance Schedule

| Task | Frequency | Command |
|------|-----------|---------|
| Health check | Daily | `./scripts/daily-health-check.sh` |
| Log review | Daily | `grep -i error /tmp/*.log` |
| Database backup | Weekly | `./scripts/backup.sh` |
| VACUUM ANALYZE | Weekly | `./scripts/weekly-maintenance.sh` |
| Security updates | Monthly | `./scripts/monthly-security.sh` |
| Performance review | Monthly | `./scripts/monthly-performance.sh` |
| Full audit | Quarterly | `./scripts/quarterly-audit.sh` |
| Dependency updates | Quarterly | `./scripts/update-dependencies.sh` |

---

## Emergency Procedures

### System Down

1. Check database: `sudo systemctl status postgresql`
2. Check disk space: `df -h`
3. Check logs: `tail -100 /var/log/postgresql/*.log`
4. Restart services: `sudo systemctl restart postgresql`
5. Verify: `psql "$DATABASE_URL" -c "SELECT 1"`

### Data Corruption

1. Stop all writes immediately
2. Create backup of current state
3. Restore from last known good backup
4. Re-run pipeline from last checkpoint
5. Verify data integrity

### Performance Emergency

1. Kill long-running queries
2. Run `VACUUM ANALYZE`
3. Check index usage
4. Increase connection pool if needed
5. Consider scaling resources

---

Generated: 2025-12-10 | MIDI Software Center v1.0.0
