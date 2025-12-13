# MIDI Software Center - Feedback & Issue Tracking

This document describes how to collect feedback, report issues, and track feature requests for MIDI Software Center.

## Overview

Feedback mechanisms ensure continuous improvement of the system:
- **Bug Reports** - Issue tracking and resolution
- **Feature Requests** - Community-driven development
- **Performance Reports** - Optimization opportunities
- **Usage Analytics** - Understanding user workflows

---

## Bug Reporting

### How to Report a Bug

**Required Information:**
1. **Summary** - Brief description of the issue
2. **Environment** - OS, Rust version, PostgreSQL version
3. **Steps to Reproduce** - Exact sequence of actions
4. **Expected Behavior** - What should happen
5. **Actual Behavior** - What actually happened
6. **Logs** - Relevant log excerpts

### Bug Report Template

```markdown
## Bug Report

### Summary
[Brief description of the bug]

### Environment
- OS: Ubuntu 22.04 / macOS 14 / Windows 11
- Rust: 1.70+
- PostgreSQL: 16.x
- MIDI Software Center: v1.0.0

### Steps to Reproduce
1. [First step]
2. [Second step]
3. [...]

### Expected Behavior
[What you expected to happen]

### Actual Behavior
[What actually happened]

### Logs
```
[Paste relevant log output here]
```

### Additional Context
[Any other information that might be helpful]
```

### Bug Severity Levels

| Level | Description | Response Time |
|-------|-------------|---------------|
| Critical | System crash, data loss | 24 hours |
| High | Major feature broken | 48 hours |
| Medium | Feature partially broken | 1 week |
| Low | Minor issue, workaround exists | 2 weeks |

---

## Feature Requests

### How to Request a Feature

**Required Information:**
1. **Problem Statement** - What problem does this solve?
2. **Proposed Solution** - How should it work?
3. **Use Case** - Who benefits and how?
4. **Alternatives** - Other ways to solve this
5. **Priority** - How important is this?

### Feature Request Template

```markdown
## Feature Request

### Problem Statement
[Describe the problem or need this feature addresses]

### Proposed Solution
[Describe your proposed solution]

### Use Case
[Describe who would use this and how]

### Alternatives Considered
[Describe any alternatives you've considered]

### Priority
- [ ] Nice to have
- [ ] Important
- [ ] Critical

### Additional Context
[Any mockups, examples, or references]
```

---

## Performance Reports

### Reporting Performance Issues

When reporting performance issues, include:

```bash
# 1. System Information
uname -a
nproc
free -h

# 2. Database Statistics
psql "$DATABASE_URL" -c "
SELECT
  relname as table,
  n_live_tup as rows,
  pg_size_pretty(pg_total_relation_size(relid)) as size
FROM pg_stat_user_tables
ORDER BY n_live_tup DESC
LIMIT 10;
"

# 3. Current Performance Metrics
psql "$DATABASE_URL" -c "
SELECT
  (SELECT COUNT(*) FROM files) as total_files,
  (SELECT COUNT(*) FROM musical_metadata) as analyzed,
  (SELECT COUNT(*) FROM file_tags) as tag_assignments;
"

# 4. Import/Analysis Speed
grep "files/sec" /tmp/import_log.txt | tail -10
```

### Performance Report Template

```markdown
## Performance Report

### Issue
[Describe the performance issue]

### Environment
- CPU: [cores/model]
- RAM: [total GB]
- Storage: [HDD/SSD/NVMe]
- Database size: [GB]
- Total files: [count]

### Expected Performance
- Import: 7,830 files/sec
- Analysis: 181-360 files/sec
- Query: < 10ms

### Actual Performance
- Import: [actual] files/sec
- Analysis: [actual] files/sec
- Query: [actual] ms

### Database Statistics
[Paste output from queries above]

### Logs
[Relevant log excerpts]
```

---

## Usage Analytics

### What We Track (Opt-In)

With user consent, we track:
- Import counts and durations
- Analysis completion rates
- Search query patterns (anonymized)
- Error frequencies by type
- Feature usage statistics

### Analytics Schema

```sql
-- Usage events table (local only)
CREATE TABLE IF NOT EXISTS usage_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Common events
INSERT INTO usage_events (event_type, event_data) VALUES
('import_complete', '{"files": 1000, "duration_ms": 128}'),
('search_query', '{"category": "drums", "bpm_range": [118, 122]}'),
('analysis_complete', '{"files": 500, "duration_ms": 2800}');
```

### Generate Usage Report

```sql
-- Usage summary
SELECT
    event_type,
    COUNT(*) as count,
    MIN(created_at) as first_seen,
    MAX(created_at) as last_seen
FROM usage_events
GROUP BY event_type
ORDER BY count DESC;

-- Import statistics
SELECT
    DATE(created_at) as date,
    SUM((event_data->>'files')::int) as files_imported,
    AVG((event_data->>'duration_ms')::int) as avg_duration_ms
FROM usage_events
WHERE event_type = 'import_complete'
GROUP BY DATE(created_at)
ORDER BY date DESC
LIMIT 7;
```

---

## Feedback Channels

### Communication Methods

| Channel | Use For | Response Time |
|---------|---------|---------------|
| GitHub Issues | Bugs, features | 24-48 hours |
| Discussions | Questions, ideas | 1-3 days |
| Email | Security issues | 24 hours |

### Issue Labels

| Label | Description |
|-------|-------------|
| `bug` | Something isn't working |
| `enhancement` | New feature or improvement |
| `performance` | Performance-related issue |
| `documentation` | Documentation improvement |
| `question` | Question about usage |
| `good first issue` | Good for newcomers |
| `help wanted` | Community help needed |
| `priority: critical` | Needs immediate attention |
| `priority: high` | Important issue |
| `priority: medium` | Normal priority |
| `priority: low` | Nice to have |

---

## Known Issues

### Current Known Issues

| Issue | Status | Workaround |
|-------|--------|------------|
| LUDICROUS mode requires manual index rebuild | By design | Run `./scripts/rebuild-indexes.sh` after import |
| Large archives (>10GB) may timeout | Investigation | Split into smaller archives |
| Some exotic MIDI formats not supported | Backlog | Convert to Standard MIDI File (SMF) |

### Issue Resolution Process

1. **Triage** - Issue reviewed within 24-48 hours
2. **Prioritize** - Severity and impact assessed
3. **Assign** - Developer assigned to issue
4. **Fix** - Code changes implemented
5. **Test** - Fix verified with test cases
6. **Release** - Fix included in next release
7. **Close** - Issue closed with resolution notes

---

## Contributing Fixes

### How to Contribute

1. **Fork** the repository
2. **Branch** from `main` with descriptive name
3. **Fix** the issue with tests
4. **Test** with `make check && make test`
5. **Submit** pull request with description
6. **Review** address feedback

### Pull Request Template

```markdown
## Pull Request

### Related Issue
Fixes #[issue number]

### Changes Made
- [Change 1]
- [Change 2]

### Testing Done
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

### Screenshots (if applicable)
[Add screenshots for UI changes]

### Checklist
- [ ] Code follows project style
- [ ] Tests added for new code
- [ ] Documentation updated
- [ ] Changelog updated
```

---

## Feedback Metrics

### Key Performance Indicators

| KPI | Target | Current |
|-----|--------|---------|
| Bug resolution time (critical) | < 24h | TBD |
| Bug resolution time (high) | < 48h | TBD |
| Feature request response | < 1 week | TBD |
| User satisfaction | > 4.0/5.0 | TBD |
| Issue backlog | < 50 open | TBD |

### Monthly Feedback Report

```sql
-- Generate monthly feedback summary
WITH monthly_stats AS (
    SELECT
        DATE_TRUNC('month', created_at) as month,
        event_type,
        COUNT(*) as count
    FROM usage_events
    WHERE created_at > NOW() - INTERVAL '3 months'
    GROUP BY 1, 2
)
SELECT
    month,
    SUM(CASE WHEN event_type = 'import_complete' THEN count ELSE 0 END) as imports,
    SUM(CASE WHEN event_type = 'search_query' THEN count ELSE 0 END) as searches,
    SUM(CASE WHEN event_type = 'error' THEN count ELSE 0 END) as errors
FROM monthly_stats
GROUP BY month
ORDER BY month DESC;
```

---

## Contact

### For Different Types of Feedback

- **Bug Reports**: GitHub Issues
- **Feature Requests**: GitHub Discussions
- **Security Issues**: security@example.com (do not use public channels)
- **General Questions**: GitHub Discussions
- **Partnership Inquiries**: contact@example.com

---

Generated: 2025-12-10 | MIDI Software Center v1.0.0
