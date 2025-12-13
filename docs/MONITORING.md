# MIDI Software Center - Monitoring & Observability

This document describes the monitoring, logging, and observability setup for MIDI Software Center.

## Overview

The system provides monitoring at multiple levels:
- **Application Metrics** - Performance, throughput, error rates
- **Database Metrics** - Query performance, connection pool, table sizes
- **Pipeline Metrics** - Import speed, analysis throughput, queue depths
- **System Metrics** - CPU, memory, disk I/O

---

## Application Monitoring

### Health Endpoints

```bash
# Database health check
psql "$DATABASE_URL" -c "SELECT 1"

# Application binary check
midi-software-center --version
midi-pipeline --version

# Quick stats
psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM files"
```

### Key Metrics

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Import Speed | 7,830 files/sec | < 5,000/sec |
| Analysis Speed | 181-360 files/sec | < 100/sec |
| Query Latency | < 10ms | > 100ms |
| Error Rate | 0% | > 1% |
| Database Connections | < 34 | > 30 |

---

## Database Monitoring

### Connection Pool Status

```sql
-- Check active connections
SELECT count(*) as connections,
       state,
       wait_event_type
FROM pg_stat_activity
WHERE datname = 'midi_library'
GROUP BY state, wait_event_type;

-- Connection pool usage
SELECT numbackends as active_connections,
       xact_commit as commits,
       xact_rollback as rollbacks,
       blks_read,
       blks_hit,
       ROUND(100.0 * blks_hit / NULLIF(blks_hit + blks_read, 0), 2) as cache_hit_ratio
FROM pg_stat_database
WHERE datname = 'midi_library';
```

### Table Statistics

```sql
-- Table sizes and row counts
SELECT relname as table_name,
       n_live_tup as row_count,
       pg_size_pretty(pg_total_relation_size(relid)) as total_size
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC;

-- Index usage
SELECT indexrelname as index_name,
       idx_scan as scans,
       idx_tup_read as tuples_read,
       idx_tup_fetch as tuples_fetched
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC
LIMIT 20;
```

### Query Performance

```sql
-- Slow queries (requires pg_stat_statements)
SELECT query,
       calls,
       ROUND(total_exec_time::numeric, 2) as total_ms,
       ROUND(mean_exec_time::numeric, 2) as mean_ms,
       rows
FROM pg_stat_statements
WHERE dbid = (SELECT oid FROM pg_database WHERE datname = 'midi_library')
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Current running queries
SELECT pid,
       now() - pg_stat_activity.query_start AS duration,
       query,
       state
FROM pg_stat_activity
WHERE datname = 'midi_library'
  AND state != 'idle'
ORDER BY duration DESC;
```

---

## Pipeline Monitoring

### Import Progress

```bash
# Real-time import monitoring
watch -n 1 'psql "$DATABASE_URL" -c "SELECT COUNT(*) as files FROM files"'

# Import rate calculation
psql "$DATABASE_URL" -c "
SELECT
  COUNT(*) as total_files,
  COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '1 minute') as last_minute,
  COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '5 minutes') / 5.0 as avg_per_minute
FROM files;
"
```

### Analysis Coverage

```sql
-- Analysis completion status
SELECT
  (SELECT COUNT(*) FROM files) as total_files,
  (SELECT COUNT(*) FROM musical_metadata) as analyzed_files,
  ROUND(100.0 * (SELECT COUNT(*) FROM musical_metadata) /
        NULLIF((SELECT COUNT(*) FROM files), 0), 2) as percent_complete;

-- Analysis by category
SELECT
  CASE
    WHEN bpm IS NOT NULL THEN 'has_bpm'
    ELSE 'no_bpm'
  END as bpm_status,
  COUNT(*) as file_count
FROM musical_metadata
GROUP BY 1;
```

### Queue Monitoring (Pipeline Mode)

```bash
# Monitor pipeline log
tail -f /tmp/import_log.txt

# Check queue depths (if using lock-free queues)
grep "queue_depth" /tmp/pipeline_*.log | tail -20
```

---

## Log Management

### Log Locations

| Component | Log Path | Contents |
|-----------|----------|----------|
| Import | `/tmp/import_log.txt` | File import progress |
| Analysis | `/tmp/analysis_log.txt` | BPM/key detection |
| Pipeline | `/tmp/pipeline_*.log` | Queue operations |
| PostgreSQL | `/var/log/postgresql/` | Database logs |

### Log Rotation

```bash
# Configure logrotate for pipeline logs
cat > /etc/logrotate.d/midi-software-center << 'EOF'
/tmp/import_log.txt /tmp/analysis_log.txt /tmp/pipeline_*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 644 $USER $USER
}
EOF
```

### Log Analysis

```bash
# Count errors in import log
grep -c "ERROR" /tmp/import_log.txt

# Find slow operations
grep "duration" /tmp/analysis_log.txt | awk -F'=' '{print $2}' | sort -n | tail -10

# Track import rate over time
grep "files/sec" /tmp/import_log.txt | tail -20
```

---

## Alerting Rules

### Critical Alerts

```yaml
# Prometheus-style alerting rules (conceptual)
groups:
  - name: midi-software-center
    rules:
      - alert: DatabaseDown
        expr: pg_up == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "PostgreSQL database is down"

      - alert: HighErrorRate
        expr: rate(import_errors_total[5m]) > 0.01
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Import error rate above 1%"

      - alert: SlowQueries
        expr: pg_stat_activity_max_tx_duration > 30
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Query running longer than 30 seconds"

      - alert: ConnectionPoolExhausted
        expr: pg_stat_activity_count > 30
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Database connection pool near exhaustion"
```

### Alert Notification Script

```bash
#!/bin/bash
# scripts/alert.sh - Simple alerting script

DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

# Check database connectivity
if ! psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
    echo "CRITICAL: Database connection failed" | mail -s "MIDI Alert" admin@example.com
    exit 1
fi

# Check for high error rate in logs
ERROR_COUNT=$(grep -c "ERROR" /tmp/import_log.txt 2>/dev/null || echo 0)
if [ "$ERROR_COUNT" -gt 100 ]; then
    echo "WARNING: High error count: $ERROR_COUNT" | mail -s "MIDI Alert" admin@example.com
fi

# Check disk space
DISK_USAGE=$(df -h /home | tail -1 | awk '{print $5}' | tr -d '%')
if [ "$DISK_USAGE" -gt 90 ]; then
    echo "WARNING: Disk usage at ${DISK_USAGE}%" | mail -s "MIDI Alert" admin@example.com
fi
```

---

## Performance Baselines

### Import Performance

| Mode | Speed | CPU | Memory |
|------|-------|-----|--------|
| Normal | 3,000 files/sec | 40% | 2 GB |
| Ultra-Fast | 7,830 files/sec | 80% | 4 GB |
| LUDICROUS | 10,000+ files/sec | 95% | 8 GB |

### Analysis Performance

| Analysis Type | Speed | Notes |
|---------------|-------|-------|
| BPM Detection | 360 files/sec | MIDI tempo events |
| Key Detection | 200 files/sec | Krumhansl-Schmuckler |
| Full Analysis | 181 files/sec | All features |

### Database Performance

| Query Type | Target | Actual |
|------------|--------|--------|
| Simple lookup | < 5ms | 2-3ms |
| Tag search | < 10ms | 5-8ms |
| Complex join | < 50ms | 20-40ms |
| Full-text search | < 100ms | 50-80ms |

---

## Monitoring Dashboard (SQL)

Run this comprehensive status check:

```sql
-- MIDI Software Center Status Dashboard
WITH stats AS (
    SELECT
        (SELECT COUNT(*) FROM files) as total_files,
        (SELECT COUNT(*) FROM musical_metadata) as analyzed_files,
        (SELECT COUNT(*) FROM tags) as total_tags,
        (SELECT COUNT(*) FROM file_tags) as tag_assignments,
        (SELECT COUNT(DISTINCT file_id) FROM file_tags) as tagged_files,
        (SELECT pg_size_pretty(pg_database_size('midi_library'))) as db_size
)
SELECT
    'Total Files' as metric, total_files::text as value FROM stats
UNION ALL
SELECT 'Analyzed Files', analyzed_files::text FROM stats
UNION ALL
SELECT 'Analysis Coverage', ROUND(100.0 * analyzed_files / NULLIF(total_files, 0), 1)::text || '%' FROM stats
UNION ALL
SELECT 'Total Tags', total_tags::text FROM stats
UNION ALL
SELECT 'Tag Assignments', tag_assignments::text FROM stats
UNION ALL
SELECT 'Tagged Files', tagged_files::text FROM stats
UNION ALL
SELECT 'Database Size', db_size FROM stats;
```

---

## Troubleshooting

### Common Issues

**Slow Import Speed**
```bash
# Check for lock contention
psql "$DATABASE_URL" -c "SELECT * FROM pg_locks WHERE NOT granted;"

# Verify LUDICROUS mode settings
psql "$DATABASE_URL" -c "SHOW synchronous_commit; SHOW fsync;"
```

**High Memory Usage**
```bash
# Check PostgreSQL memory
psql "$DATABASE_URL" -c "SHOW shared_buffers; SHOW work_mem; SHOW maintenance_work_mem;"

# Check Rust process memory
ps aux | grep midi | awk '{print $6/1024 " MB", $11}'
```

**Connection Exhaustion**
```bash
# Kill idle connections
psql "$DATABASE_URL" -c "
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'midi_library'
  AND state = 'idle'
  AND query_start < NOW() - INTERVAL '10 minutes';
"
```

---

## Backup Verification

```bash
# Verify backup integrity
pg_restore --list backup_YYYYMMDD.sql | head -20

# Test restore to temp database
createdb midi_library_test
pg_restore -d midi_library_test backup_YYYYMMDD.sql
psql midi_library_test -c "SELECT COUNT(*) FROM files"
dropdb midi_library_test
```

---

Generated: 2025-12-10 | MIDI Software Center v1.0.0
