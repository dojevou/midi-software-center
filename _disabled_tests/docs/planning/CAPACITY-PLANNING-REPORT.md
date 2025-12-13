# MIDI Software Center - Capacity Planning Report

**Report Date:** November 29, 2025
**Current Scale:** 1.72M MIDI files, 71 GB storage
**Analysis Scope:** 2x/5x/10x Growth Scenarios
**Current Host:** 16 CPU cores, 60 GB RAM, Ubuntu 25.04

---

## Executive Summary

**Current Status:** System operating at 30-40% capacity with optimization headroom

**Key Findings:**
- ✅ **2x Growth (3.4M files):** No infrastructure changes needed (~2-3 hours added to pipeline)
- ⚠️ **5x Growth (8.6M files):** Database scaling + query optimization recommended
- 🔴 **10x Growth (17.2M files):** Full distributed architecture required

**Critical Path:** Database I/O (index scans, join operations) becomes bottleneck at 5x+ scale

**Recommended Actions:**
1. Immediate: Enable connection pooling, optimize database indexes
2. At 5x: Upgrade PostgreSQL hardware, implement read replicas
3. At 10x: Distributed architecture with sharding/federation

---

## Current Infrastructure Analysis

### Hardware Specifications

**Host System:**
```
OS:          Ubuntu 25.04
CPU:         16 cores (appears to be dual-socket or high-end single-socket)
RAM:         60 GB available (8.8 GB used, 51 GB available)
Storage:     NVMe SSD (based on throughput metrics)
Architecture: x86_64, supports AVX2, SSE4.2
```

**Utilization:**
- RAM: 15% used, 85% available
- CPU: Idle during import (~40-50% utilization during analysis)
- Storage: ~100 GB total used (85 GB project, ~15 GB database)

### Current Performance Metrics

| Operation | Speed | Throughput | Bottleneck |
|-----------|-------|-----------|-----------|
| **Import** | 7,830 files/sec | 28.2 GB/hour | CPU (hashing, parsing) |
| **Analysis** | 181-360 files/sec | 0.65-1.3 GB/hour | CPU (MIDI analysis) |
| **Dedup Hash** | 88,656 files/sec | 319 GB/hour | RAM (hash buffer) |
| **File Trimming** | 48,935 files/sec | 176 GB/hour | Disk I/O |
| **DB Insert** | 1,000 batch/sec | ~7,830 files/sec | Connection pool (34 max) |
| **Query - Simple Tag** | <10ms latency | - | Network/Connection |
| **Query - Multi-Join** | <100ms latency | - | Index efficiency |
| **Full Category Scan** | <500ms latency | 3.4M files/sec | Sequential scan |

### Database Architecture

**PostgreSQL 16 Configuration:**
- **Connection Pool:** 34 max connections (16 workers + 18 buffer)
- **Work Memory:** 256 MB (per operation)
- **Maintenance Memory:** 2 GB
- **WAL Size:** 4 GB max
- **Cache:** Shared buffers = 8-12 GB (default 25% of RAM)

**Current Schema:**
- **Tables:** 15 (files, musical_metadata, file_categories, tags, etc.)
- **Indexes:** 60+ (B-tree, GIN, partial indexes)
- **Rows:** ~1.72M files + ~5-10M file_tags + ~8-15M midi_tracks
- **Size:** 3-5 GB database, 100-200 MB indexes

**Meilisearch Integration:**
- Full-text search index (~500 MB)
- Synced after each import batch

---

## Capacity Scenarios

### Scenario 1: 2x Growth (3.4M Files)

**Target State:**
- Files: 3.4M (from 1.72M)
- Storage: 142 GB files + 20-30 GB database
- Total: ~170 GB

**Timeline Impact:**
```
Current (1.72M):
- Import:    3-4 minutes (at 7,830 files/sec)
- Analysis:  8-10 hours (at 360 files/sec worst case)
- Total:     8-10 hours

2x Growth (3.4M):
- Import:    6-8 minutes (+1-2 min)
- Analysis:  16-20 hours (+8-10 hours, linear scaling)
- Total:     16-20 hours (+100% time, linear scaling)
```

**Infrastructure Changes Required:** ✅ NONE

**Rationale:**
- CPU: 2x files = 2x processing time (still OK with 16 cores)
- RAM: 60 GB available; database grows to 20-30 GB (still 30 GB free)
- Storage: 170 GB fits easily on 1+ TB NVMe
- DB Connections: 34 connections sufficient for 2x files

**Optimization Recommendations:**
- Enable index-only scans for tag queries
- Implement query result caching (Redis/Memcached)
- Monitor DB query performance with pg_stat_statements

**Cost:** Minimal (no additional hardware)

---

### Scenario 2: 5x Growth (8.6M Files)

**Target State:**
- Files: 8.6M (from 1.72M)
- Storage: 355 GB files + 50-70 GB database
- Total: ~425 GB
- Records: ~43M file_tags, ~60-80M midi_tracks

**Timeline Impact:**
```
Current (1.72M):
- Import:    3-4 minutes
- Analysis:  8-10 hours
- Total:     8-10 hours

5x Growth (8.6M):
- Import:    15-20 minutes (+12-16 min)
  [hitting connection pool saturation at 34 connections]
- Analysis:  40-50 hours (+32-40 hours, linear scaling)
  [CPU bound, sustained high load]
- Total:     40-50 hours
```

**Infrastructure Changes Required:** ⚠️ MODERATE

**Critical Bottlenecks:**

1. **Database Connection Pool (34 max)**
   - Current: 16 workers + 18 buffer = 34
   - Needed: 64-96 connections (scale 2x workers, 3x buffer)
   - Fix: Increase max_connections in PostgreSQL
   - Cost: Minimal (just config change)
   - Impact: +2x import throughput

2. **Database Memory**
   - Current: 8-12 GB shared buffers
   - Needed: 20-30 GB shared buffers (for 50-70 GB database)
   - Fix: Upgrade to 128 GB RAM
   - Cost: $200-400
   - Impact: Faster index caching, fewer disk seeks

3. **CPU Saturation**
   - Current: 16 cores, analysis uses 40-50%
   - At 5x: 100% utilization during analysis phase
   - Fix: Upgrade to 32-64 core CPU (if bottleneck confirmed)
   - Cost: Server upgrade ($500-2000)
   - Impact: 2-4x faster analysis
   - Alternative: Offload analysis to secondary server

4. **Index Performance on Larger Dataset**
   - Current: 60+ indexes covering ~5-10M rows
   - At 5x: Same indexes covering ~25-50M rows
   - Risk: Join queries degrade from <100ms to 500ms+
   - Fix: Add covering indexes, partition large tables
   - Cost: Minimal (5-10 min setup)
   - Impact: Maintain <100ms query latency

**Recommended Hardware Upgrade:**
```
Current:               Recommended for 5x:
- CPU: 16 cores       → 32-64 cores
- RAM: 60 GB          → 128 GB
- SSD: 1-2 TB         → 2-4 TB NVMe RAID1
- Network: 1 Gbps     → 10 Gbps (if distributed)

OR use High-Memory Instance:
- AWS: r6g.4xlarge (16 vCPU, 128 GB RAM) → r7g.16xlarge (64 vCPU, 512 GB RAM)
- Azure: E16ds_v5 (16 vCPU, 128 GB RAM) → E64ds_v5 (64 vCPU, 504 GB RAM)
- GCP: n2-highmem-16 (16 vCPU, 128 GB RAM) → n2-highmem-64 (64 vCPU, 256 GB RAM)
```

**Database Optimizations at 5x Scale:**

1. **Enable Partitioning (for files table)**
   ```sql
   -- Partition by content hash (first byte: 0-255)
   -- 256 partitions × ~33K files each = faster scans
   -- Setup time: 30 min, benefit: 50% faster full scans
   ```

2. **Add Covering Indexes**
   ```sql
   -- For tag searches (no table lookup needed)
   CREATE INDEX idx_file_tags_covering ON file_tags(tag_id)
   INCLUDE (file_id, files.filename, files.bpm);
   ```

3. **Parallel Query Execution**
   ```sql
   -- Enable for large scans (already available in PG16)
   ALTER SYSTEM SET max_parallel_workers_per_gather = 16;
   -- Processes large joins in parallel across 16 cores
   ```

**Cost Analysis for 5x Growth:**
```
Hardware:           $300-800 (upgrade RAM to 128GB + SSD)
Database Tuning:    $0 (config changes)
Personnel:          20-40 hours (planning + implementation)
Downtime:           2-4 hours (for hardware upgrade + data reload)
Timeline:           2-4 weeks (plan → test → deploy)
```

**Cost:** $300-800 hardware + 20-40 hours labor

---

### Scenario 3: 10x Growth (17.2M Files)

**Target State:**
- Files: 17.2M (from 1.72M)
- Storage: 710 GB files + 100-150 GB database
- Total: ~850 GB
- Records: ~86M file_tags, ~120-160M midi_tracks

**Timeline Impact:**
```
Current (1.72M):
- Import:    3-4 minutes
- Analysis:  8-10 hours
- Total:     8-10 hours

10x Growth (17.2M):
- Import:    30-45 minutes (connection pool / DB saturation limit)
- Analysis:  80-100 hours (CPU maxed out)
- Total:     80-100+ hours (split across multiple days)
  [Batching required to avoid DB locks / timeout]
```

**Infrastructure Changes Required:** 🔴 MAJOR

**Critical Bottlenecks:**

1. **Single-Node PostgreSQL Limit**
   - Current architecture: Single PostgreSQL instance
   - At 10x: Database operations become single-threaded bottleneck
   - Index lookups: O(log N) but N is now 100M+ rows
   - Join operations: CPU bound, slow with large result sets
   - Fix: Distributed architecture required
   - Cost: Major architectural change

2. **CPU Saturation During Analysis**
   - Current: 16 cores at 40-50% utilization
   - At 10x: 64+ cores at 100% utilization (still insufficient)
   - Analysis is CPU-intensive (BPM/key detection, MIDI parsing)
   - Fix: Distributed workers + GPU acceleration
   - Cost: $2000-5000 hardware

3. **Storage I/O**
   - Current: 710 GB files fits on 1-2 TB SSD
   - Throughput: 7,830 files/sec = 28 GB/hour (well within NVMe limits)
   - At 10x: Same throughput, but 850 GB total storage
   - Fix: 4-8 TB NVMe RAID1 or SAN
   - Cost: $500-1500

4. **Network Bottleneck (if distributed)**
   - Current: Single machine (no network overhead)
   - At 10x: If using distributed workers
   - Database queries: 10,000+ qps → network saturation at 1 Gbps
   - Fix: 10 Gbps network infrastructure
   - Cost: $200-500 per server

**Recommended Architecture for 10x:**

```
┌─────────────────────────────────────────────────────────────┐
│ DISTRIBUTED ANALYSIS PIPELINE (10x scale)                   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Worker Nodes (8x):                     Main Coordinator:    │
│  ┌──────────────────┐                   ┌──────────────────┐│
│  │ Analyzer 1-8     │                   │ API Server       ││
│  │ (32 cores each)  │◄──────────────────│ (Tauri)          ││
│  │ (256 GB RAM)     │      gRPC         │                  ││
│  └──────────────────┘      queries      └──────────────────┘│
│         │                                                     │
│         ├─► Work Queue (Redis)                               │
│         │   (2,000 files batch)                              │
│         │                                                     │
│         └─► PostgreSQL Cluster                               │
│             ┌─────────────────────────┐                      │
│             │ Primary (400GB RAM)      │                      │
│             │ 256 GB Buffers           │                      │
│             │ 64 connections → 256     │                      │
│             └─────────────────────────┘                      │
│                        │                                     │
│             ┌──────────┴──────────┐                          │
│             │                     │                          │
│        ┌────▼──┐           ┌─────▼──┐                       │
│        │Replica│           │Replica │                       │
│        │(Read) │           │(Read)  │                       │
│        └───────┘           └────────┘                       │
│                                                               │
│  Meilisearch Cluster (3 nodes):                             │
│  └─────────────────────────────────────                    │
│     High-availability search index                           │
│                                                               │
│  Storage:                                                    │
│  └─────────────────────────────────────                    │
│     NVMe RAID1 (8 TB) / S3 Compatible Storage               │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Key Architectural Changes:**

1. **Distributed Workers**
   - 8 worker nodes with 32 cores each = 256 cores total
   - Each worker processes 10-20 files in parallel
   - Throughput: 360-720 files/sec per worker × 8 = 2,880-5,760 files/sec
   - Analysis time: 17.2M ÷ 3,000 files/sec = ~5,733 sec = 1.6 hours
   - Import time: ~35 minutes (limited by network bandwidth)

2. **PostgreSQL Cluster**
   - Primary: 400 GB RAM, 64 cores, 256 connections
   - 2x Read Replicas: For query load distribution
   - Replication: Synchronous (durability) with <100ms latency
   - Sharding: Optional if query latency becomes issue

3. **Message Queue (Redis)**
   - Coordinates work distribution across 8 workers
   - 2,000 file batches queued
   - Built-in monitoring and retry logic

4. **Meilisearch Cluster**
   - Replicated across 3 nodes
   - High availability for search index
   - Automatic failover

**Hardware Requirements for 10x:**

```
Main Server:
- CPU: 64 cores (AMD EPYC / Intel Xeon)
- RAM: 400-512 GB
- SSD: 8 TB NVMe RAID1
- Network: 10 Gbps
- Cost: $3,000-5,000

Worker Nodes (8x):
- CPU: 32 cores each
- RAM: 128 GB each
- SSD: 2 TB NVMe
- Network: 10 Gbps
- Cost: $1,500-2,000 per node = $12,000-16,000

Database Replicas (2x):
- CPU: 32 cores
- RAM: 256 GB
- SSD: 4 TB NVMe
- Network: 10 Gbps
- Cost: $1,500-2,500 per node = $3,000-5,000

Total Infrastructure Cost: $18,000-26,000
Networking Gear: $2,000-5,000
```

**Operational Complexity:**
- Cluster management (Kubernetes/Docker Swarm recommended)
- Database replication monitoring
- Network latency concerns (gRPC between workers)
- Data consistency across distributed nodes
- Cost: $50,000-100,000/year (DevOps salary + cloud infrastructure)

**Timeline for 10x Deployment:**
- Planning & Design: 2-4 weeks
- Infrastructure Setup: 2-4 weeks
- Code Refactoring: 4-8 weeks (distributed worker API)
- Testing & Validation: 2-4 weeks
- Production Rollout: 1-2 weeks
- **Total: 12-22 weeks**

**Cost Analysis for 10x Growth:**
```
Hardware:           $20,000-30,000
Networking:         $2,000-5,000
Software (licenses):$0 (all open source)
Personnel:          $80,000-150,000 (20-40 weeks @ 200/hr)
Total:              $102,000-185,000
Annual Ops Cost:    $50,000-100,000
```

---

## Bottleneck Analysis by Growth Stage

### Current State (1.72M files) - Optimal

| Component | Utilization | Status | Limiting Factor |
|-----------|-------------|--------|-----------------|
| **CPU** | 40-50% | ✅ Good | Parallelization maxed (16 cores) |
| **RAM** | 15% | ✅ Excellent | 51 GB available |
| **Storage** | 100 GB used | ✅ Good | NVMe throughput OK |
| **DB Connections** | 16-20 / 34 | ✅ Good | Headroom available |
| **DB Queries** | <100ms | ✅ Excellent | Index coverage good |

**Current Headroom:** 2-3x files without intervention

---

### 2x Growth (3.4M files) - Acceptable

| Component | Utilization | Status | Limiting Factor |
|-----------|-------------|--------|-----------------|
| **CPU** | 80-90% | ⚠️ High | Approaching saturation |
| **RAM** | 30% | ✅ Good | 40 GB available |
| **Storage** | 170 GB | ✅ Good | NVMe still comfortable |
| **DB Connections** | 32-34 / 34 | ⚠️ High | Saturation risk |
| **DB Queries** | 50-150ms | ⚠️ Moderate | Index scans degrading |

**Recommended Actions:**
- Increase connection pool to 48-64
- Add query caching (Redis)
- Monitor CPU during analysis

**Headroom:** 1.5-2x more files before hardware upgrade needed

---

### 5x Growth (8.6M files) - Bottlenecked

| Component | Utilization | Status | Limiting Factor |
|-----------|-------------|--------|-----------------|
| **CPU** | 100% | 🔴 Saturated | Sustained analysis load |
| **RAM** | 60% | ⚠️ High | Database growth + buffers |
| **Storage** | 425 GB | ⚠️ Moderate | 50% of 1TB drive |
| **DB Connections** | 48-64 / 96 | ⚠️ High | Approaching limit |
| **DB Queries** | 200-500ms | 🔴 Degraded | Large table scans |

**Mandatory Upgrades:**
- RAM: 60 GB → 128 GB ($200-400)
- CPU: Consider upgrade if analysis still slow ($500-2000)
- Database indexes: Add partitioning / covering indexes (free)

**Bottleneck:** CPU during analysis phase

---

### 10x Growth (17.2M files) - Architectural Limits

| Component | Utilization | Status | Limiting Factor |
|-----------|-------------|--------|-----------------|
| **CPU** | 100%+ | 🔴 Insufficient | Need 4-8x more cores |
| **RAM** | 80%+ | 🔴 Saturated | Need 400+ GB for DB |
| **Storage** | 850 GB | ⚠️ High | Need 4-8 TB drive |
| **DB Connections** | 256 / 256 | 🔴 Saturated | Single instance limit |
| **DB Queries** | 1000ms+ | 🔴 Critical | Single-node bottleneck |

**Required Architecture Change:**
- Distributed workers (8 nodes)
- Database cluster (1 primary + 2 replicas)
- Message queue (Redis)
- High-speed networking (10 Gbps)

**Cost:** $100,000-200,000 capital + $50,000-100,000/year operations

---

## Scaling Path Recommendations

### Phase 1: Optimize Current Setup (2-5x scale, 0-6 months)

**Timeline:** Now to ~5M files

**Actions:**
```bash
# 1. Database connection pool
ALTER SYSTEM SET max_connections = 256;  # Currently 100
ALTER SYSTEM SET max_parallel_workers = 32;

# 2. Index optimization
REINDEX INDEX CONCURRENTLY idx_file_tags_tag_id;
REINDEX INDEX CONCURRENTLY idx_files_hash;

# 3. Enable query parallelization (already in PG16)
ALTER SYSTEM SET max_parallel_workers_per_gather = 8;

# 4. Caching layer (add Redis)
docker run -d redis:7-alpine --port 6379

# 5. Query result caching in application
# Cache tag lookups, instrument queries, BPM searches
```

**Cost:** $0-200 (Redis docker container)
**Effort:** 20-40 hours
**Result:** Support 5x growth with current hardware

---

### Phase 2: Hardware Upgrade (5-10x scale, 6-12 months)

**Timeline:** 5-10M files

**Actions:**
```
1. Upgrade RAM: 60 GB → 128 GB
   - Cost: $200-400
   - Benefit: 2-3x faster database operations

2. (Optional) Upgrade CPU: 16 → 32 cores
   - Cost: $500-2000
   - Benefit: 2x faster analysis
   - Only if analysis is still bottleneck after RAM upgrade

3. Add external Meilisearch server
   - Cost: $500-1000
   - Benefit: Offload search indexing from main server

4. Setup read replica (optional)
   - Cost: $500-1000 hardware
   - Benefit: Distribute query load
```

**Cost:** $700-2000
**Effort:** 40-80 hours (implementation + testing)
**Result:** Support up to 10M files

---

### Phase 3: Distributed Architecture (10x+ scale, 12+ months)

**Timeline:** 10M+ files

**Actions:**
```
1. Architect distributed worker system
   - Design gRPC API for work distribution
   - Implement work queue (Redis/RabbitMQ)
   - Cost: $0 (engineering)

2. Deploy 8 worker nodes
   - Cost: $12,000-16,000
   - Setup: 4 weeks

3. Upgrade PostgreSQL to cluster
   - Primary + 2 Replicas
   - Cost: $3,000-5,000
   - Setup: 2 weeks

4. Production deployment
   - Blue-green deployment strategy
   - Monitoring (Prometheus, Grafana)
   - Cost: $2,000-5,000 tools/setup
```

**Cost:** $17,000-26,000
**Effort:** 400-600 hours (architects + engineers)
**Timeline:** 12-22 weeks
**Result:** Support 50M+ files (with continued optimization)

---

## Cost Projection by Scale

| Scale | Storage | DB Size | Hardware | Software | Total capEx | Annual OpEx |
|-------|---------|---------|----------|----------|-------------|-------------|
| **1.72M (Now)** | 100 GB | 5 GB | Current | $0 | $0 | $0 |
| **3.4M (2x)** | 170 GB | 10 GB | Current | $0 | $200 | $0 |
| **8.6M (5x)** | 425 GB | 30 GB | +$1K | $0 | $1,200 | $0 |
| **17.2M (10x)** | 850 GB | 100 GB | +$23K | +$5K | $29,000 | $50,000 |
| **50M (30x)** | 2.5 TB | 300 GB | +$50K | +$20K | $75,000 | $150,000 |

---

## Migration Path & Timeline

### Immediate Actions (Next 2 Weeks)

```bash
# 1. Increase PostgreSQL connections (5 min)
ALTER SYSTEM SET max_connections = 256;
SELECT pg_reload_conf();

# 2. Enable index-only scans (10 min)
ANALYZE;
REINDEX INDEX CONCURRENTLY idx_file_tags_tag_id;

# 3. Setup monitoring (1 hour)
# Install pg_stat_statements for query monitoring
CREATE EXTENSION pg_stat_statements;

# 4. Setup Redis caching (30 min)
docker run -d redis:7-alpine

# 5. Verify performance baseline (1 hour)
# Run import/analysis with monitoring enabled
```

**Result:** Ready to handle 3-4M files without changes

---

### 0-6 Month Plan (Support 2x-5x Growth)

**Month 1-2:**
- Implement query caching in application
- Add Redis layer for tag lookups
- Monitor database performance

**Month 3-4:**
- Stress test with 5M file dataset
- Identify slow queries with pg_stat_statements
- Create covering indexes for hot paths

**Month 5-6:**
- Plan hardware upgrade (if needed)
- Prepare upgrade strategy (minimal downtime)
- Document database performance baselines

---

### 6-12 Month Plan (Support 5x-10x Growth)

**Month 6-8:**
- Upgrade hardware (if needed for 5x)
  - RAM: 128 GB
  - Optional: CPU upgrade
- Deploy external Meilisearch
- Setup read replica

**Month 9-10:**
- Re-test with 10M file dataset
- Optimize for continued growth
- Document scaling limits

**Month 11-12:**
- Prepare distributed architecture design
- Start worker node planning
- Cost estimation for next phase

---

### 12+ Month Plan (Support 10x+ Growth)

**Phase 1 (Month 12-14):**
- Design distributed worker API
- Prototype work queue system
- Proof of concept with 2-4 workers

**Phase 2 (Month 15-18):**
- Production implementation
- Deploy 8 worker nodes
- Setup PostgreSQL cluster

**Phase 3 (Month 19-20):**
- Load testing and optimization
- Production rollout (blue-green)
- Monitoring and alerting setup

**Phase 4 (Month 21+):**
- Continuous optimization
- Monitor for new bottlenecks
- Plan 30x+ growth if needed

---

## Key Recommendations

### For 2x Growth (3.4M files) ✅ READY

**Status:** Current infrastructure sufficient

**Actions:**
- ✅ No hardware changes needed
- ✅ No architectural changes needed
- Increase connection pool from 34 to 48
- Add Redis caching for frequently accessed queries
- Estimated additional cost: $0-200

**Timeline:** Ready now

---

### For 5x Growth (8.6M files) ⚠️ PLAN NOW

**Status:** Will require hardware upgrade

**Actions:**
1. Upgrade RAM to 128 GB (cost: $200-400)
2. Optional: Upgrade CPU if analysis still slow
3. Add database partitioning for large tables
4. Implement query result caching
5. Setup monitoring (pg_stat_statements)

**Timeline:** 2-4 weeks to implement

**Cost:** $700-2,000 + 40-80 hours labor

---

### For 10x Growth (17.2M files) 🔴 REDESIGN REQUIRED

**Status:** Requires distributed architecture

**Actions:**
1. Refactor analysis pipeline for distributed workers
2. Implement work queue system (Redis)
3. Deploy 8 worker nodes (32 cores each)
4. Upgrade PostgreSQL to cluster (1 primary + 2 replicas)
5. Add network infrastructure (10 Gbps)

**Timeline:** 12-22 weeks to implement

**Cost:** $100,000-200,000 capital + $50,000-100,000/year ops

---

## Conclusion

**Current Infrastructure Status:** ✅ Optimal for 1.72M files

**Scaling Capacity:**
- **2x (3.4M):** No changes needed
- **5x (8.6M):** Minor hardware upgrade ($700-2,000)
- **10x (17.2M):** Major architectural redesign ($100,000+)

**Recommendation:**
Monitor system as files approach 3.4M, execute hardware upgrade at 5M, and plan distributed architecture for 10M+ files.

**Next Review:** When collection reaches 3.4M files (estimated: 3-6 months at current growth rate)

---

## Appendix A: Database Tuning Reference

```sql
-- Current settings optimized for 1.72M files
SHOW max_connections;           -- 100 (increase to 256 for 5x+)
SHOW shared_buffers;            -- ~8-12 GB (increase to 30-40 GB for 10x)
SHOW work_mem;                  -- 256 MB (sufficient)
SHOW maintenance_work_mem;      -- 2 GB (increase to 4 GB for 5x)
SHOW max_wal_size;              -- 4 GB (increase to 8 GB for 10x)
SHOW max_parallel_workers;      -- 8 (increase to 32 for 10x)

-- Performance monitoring
SELECT * FROM pg_stat_statements
ORDER BY total_exec_time DESC
LIMIT 20;

-- Index bloat analysis
SELECT schemaname, tablename, indexname, idx_blks_read
FROM pg_stat_user_indexes
ORDER BY idx_blks_read DESC;

-- Table size analysis
SELECT tablename, pg_size_pretty(pg_total_relation_size(tablename::regclass))
FROM pg_tables
WHERE tableschemaname = 'public'
ORDER BY pg_total_relation_size(tablename::regclass) DESC;
```

---

## Appendix B: Performance Tuning Checklist

### For 2x Growth
- [ ] Monitor database connection usage
- [ ] Run ANALYZE on all tables
- [ ] Check slow query log
- [ ] Verify index cache hit ratio (should be >99%)

### For 5x Growth
- [ ] Upgrade RAM to 128 GB
- [ ] Increase max_connections to 256
- [ ] Add partitioning to files table (256 partitions)
- [ ] Create covering indexes for tag queries
- [ ] Setup external Meilisearch server
- [ ] Implement query result caching

### For 10x Growth
- [ ] Design distributed worker architecture
- [ ] Prototype work queue system
- [ ] Plan PostgreSQL cluster setup
- [ ] Prepare network infrastructure
- [ ] Start distributed worker implementation
- [ ] Plan disaster recovery strategy

---

**Report prepared for production deployment**
**Last updated: November 29, 2025**
