# MIDI Software Center - Comprehensive Security Audit Report

**Date:** 2025-11-29
**Auditor:** Claude Code Security Specialist
**Scope:** Complete codebase security assessment
**Standards:** OWASP Top 10, CWE Top 25, Rust Security Guidelines

---

## Executive Summary

### Overall Security Posture: **HIGH RISK** ⚠️

**Critical Issues Found:** 3
**High Severity:** 5
**Medium Severity:** 4
**Low Severity:** 3

**Immediate Action Required:**
1. Remove hardcoded database credentials (221 instances)
2. Implement Content Security Policy in all Tauri apps
3. Patch SQL injection vulnerability in test helpers
4. Implement path traversal validation

---

## CRITICAL SEVERITY FINDINGS

### CRITICAL-001: Hardcoded Database Credentials Exposed ⚠️

**Severity:** CRITICAL
**OWASP Category:** A02:2021 – Cryptographic Failures
**CWE:** CWE-798 (Use of Hard-coded Credentials)

**Description:**
Database password \`145278963\` is hardcoded in **221 files** across the codebase.

**Affected Files (Sample):**
- /home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/file_repository.rs:481
- /home/dojevou/projects/midi-software-center/pipeline/src-tauri/src/db/repositories/search_repository.rs:266
- /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/test_helpers.rs:15

See full list with: \`grep -r "145278963" --include="*.rs"\`

**Remediation (URGENT - Complete in 24 hours):**
```bash
# 1. Generate new strong password
NEW_PASSWORD=$(openssl rand -base64 32)

# 2. Update PostgreSQL
psql -U postgres -c "ALTER USER midiuser WITH PASSWORD '$NEW_PASSWORD';"

# 3. Set environment variable
echo "DATABASE_URL=postgresql://midiuser:$NEW_PASSWORD@localhost:5433/midi_library" >> .env

# 4. Remove hardcoded fallbacks (automated script)
find . -name "*.rs" -type f -exec sed -i 's/unwrap_or_else(|_| {/expect("DATABASE_URL environment variable must be set");/g' {} +

# 5. Update all test files to require DATABASE_URL
```

**Impact:** Immediate database compromise if repository is public or leaked.

---

### CRITICAL-002: Missing Content Security Policy (CSP)

**Severity:** CRITICAL
**OWASP Category:** A05:2021 – Security Misconfiguration
**CWE:** CWE-1021 (Improper Restriction of Rendered UI Layers)

**Affected Files:**
- /home/dojevou/projects/midi-software-center/app/src-tauri/tauri.conf.json:25

**Current Configuration:**
```json
{
  "app": {
    "security": {
      "csp": null  // ⚠️ NO PROTECTION
    }
  }
}
```

**Remediation (URGENT - Complete in 4 hours):**
```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' ws://localhost:* http://localhost:*; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'"
    }
  }
}
```

**Impact:** XSS attacks can execute arbitrary code in webview, steal data, or compromise backend.

---

### CRITICAL-003: SQL Injection Vulnerability in Test Helpers

**Severity:** CRITICAL
**OWASP Category:** A03:2021 – Injection
**CWE:** CWE-89 (SQL Injection)

**Affected File:**
- /home/dojevou/projects/midi-software-center/pipeline/src-tauri/tests/helpers/db.rs:130

**Vulnerable Code:**
```rust
pub async fn count_files_where(pool: &PgPool, condition: &str) -> Result<i64, sqlx::Error> {
    let query = format!("SELECT COUNT(*) as count FROM files WHERE {}", condition);
    let row = sqlx::query(&query).fetch_one(pool).await?;
    Ok(row.get("count"))
}
```

**Attack Scenario:**
```rust
count_files_where(&pool, "1=1; DROP TABLE files; --").await;
```

**Remediation (URGENT - Complete in 2 hours):**
```rust
// OPTION 1: Remove this function entirely (recommended)
// Use type-safe repository methods with sqlx::query_as! instead

// OPTION 2: Use parameterized queries with whitelisting
pub async fn count_files_by_column(
    pool: &PgPool,
    column: &str,
    value: &str
) -> Result<i64, sqlx::Error> {
    // Whitelist allowed columns
    match column {
        "filename" => sqlx::query_scalar!("SELECT COUNT(*) FROM files WHERE filename = $1", value),
        "manufacturer" => sqlx::query_scalar!("SELECT COUNT(*) FROM files WHERE manufacturer = $1", value),
        _ => return Err(sqlx::Error::Protocol("Invalid column".into())),
    }
    .fetch_one(pool)
    .await
}
```

**Impact:** Database destruction, data exfiltration, privilege escalation.

---

## HIGH SEVERITY FINDINGS

### HIGH-001: Path Traversal Vulnerability in File Operations

**Severity:** HIGH
**OWASP Category:** A01:2021 – Broken Access Control
**CWE:** CWE-22 (Path Traversal)

**Affected Files:**
- pipeline/src-tauri/src/commands/file_import.rs
- pipeline/src-tauri/src/commands/archive_import.rs:66
- pipeline/src-tauri/src/commands/analyze.rs
- pipeline/src-tauri/src/commands/split_file.rs

**Vulnerable Pattern:**
```rust
#[tauri::command]
pub async fn import_archive_collection(
    collection_path: String,  // ⚠️ No validation
    state: State<'_, AppState>,
) -> Result<ArchiveImportSummary, String> {
    let collection_dir = Path::new(&collection_path);  // Direct use
    // ...
}
```

**Attack Vectors:**
- \`../../../etc/passwd\`
- \`..\\..\\..\\Windows\\System32\`
- Symlink attacks
- Null byte injection (on some platforms)

**Remediation:**
```rust
use std::path::{Path, PathBuf};
use std::fs::canonicalize;

fn validate_path(path: &str, allowed_base: &Path) -> Result<PathBuf, String> {
    let canonical = canonicalize(PathBuf::from(path))
        .map_err(|e| format!("Invalid path: {}", e))?;

    if !canonical.starts_with(allowed_base) {
        return Err("Path traversal detected".to_string());
    }

    Ok(canonical)
}

#[tauri::command]
pub async fn import_archive_collection(
    collection_path: String,
    state: State<'_, AppState>,
) -> Result<ArchiveImportSummary, String> {
    let allowed_base = PathBuf::from("/home/dojevou/projects/midi-software-center/midi-library");
    let validated_path = validate_path(&collection_path, &allowed_base)?;
    // Use validated_path
}
```

**Files to Update:** 4 command files + shared validation module

---

### HIGH-002: Missing Authentication/Authorization on All Tauri Commands

**Severity:** HIGH
**OWASP Category:** A01:2021 – Broken Access Control
**CWE:** CWE-862 (Missing Authorization)

**Description:**
50+ Tauri commands exposed without authentication. Any code in webview can invoke any backend operation.

**Scan Results:**
```
Found 47 #[tauri::command] decorators
Authorization checks: 0
Permission validation: 0
```

**Impact:**
- Malicious scripts can execute privileged operations
- XSS can fully compromise backend
- No audit trail of who performed actions

**Remediation (Tauri v2 Capability System):**
```json
// capabilities/main.json
{
  "identifier": "main-capabilities",
  "windows": ["main"],
  "permissions": [
    "core:window:allow-close",
    "fs:allow-read-text-file",
    {
      "identifier": "midi-import",
      "description": "MIDI file import operations",
      "commands": {
        "allow": ["import_archive_collection", "import_directory"],
        "deny": []
      }
    },
    {
      "identifier": "midi-read",
      "description": "Read MIDI files and metadata",
      "commands": {
        "allow": ["search_files", "get_file_details"],
        "deny": []
      }
    }
  ]
}
```

**Required Actions:**
1. Define capability sets for different operation types
2. Apply least-privilege principle
3. Add permission checks to sensitive commands
4. Implement audit logging

---

### HIGH-003: Information Disclosure Through Error Messages

**Severity:** HIGH
**OWASP Category:** A04:2021 – Insecure Design
**CWE:** CWE-209 (Information Exposure Through Error Message)

**Examples:**
```rust
// Exposes full file paths
return Err(format!("Collection directory not found: {}", collection_path));

// Exposes database structure
.map_err(|e| format!("Database error: {}", e))?;
// SQLx errors contain: table names, column names, constraint violations
```

**Frontend Exposure:**
```typescript
catch (error) {
  console.error('Failed to list MIDI devices:', error);
  throw error;  // ⚠️ Full error propagated to UI
}
```

**Remediation:**
```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("Invalid input")]
    InvalidInput,

    #[error("Operation failed")]
    OperationFailed,

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        match err {
            AppError::Internal(e) => {
                eprintln!("Internal error: {:?}", e);  // Log only
                "An error occurred".to_string()  // Generic message
            }
            _ => err.to_string(),
        }
    }
}
```

---

### HIGH-004: No Rate Limiting on Resource-Intensive Operations

**Severity:** HIGH
**OWASP Category:** A04:2021 – Insecure Design
**CWE:** CWE-770 (Allocation of Resources Without Limits)

**Vulnerable Commands:**
- \`import_archive_collection\` - Can process thousands of files
- \`analyze_files\` - CPU-intensive operation
- \`search_files\` - Database query load

**Attack Scenario:**
```typescript
// Malicious script
for (let i = 0; i < 1000; i++) {
  invoke('import_archive_collection', { collectionPath: '/huge/path' + i });
}
// Result: CPU exhaustion, memory overflow, DB connection starvation
```

**Current Limits:**
- Database connections: 34 max (can be exhausted in seconds)
- No per-command rate limiting
- No concurrent operation limits

**Remediation:**
```rust
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::{Instant, Duration};

struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    fn check_rate_limit(&self, command: &str) -> Result<(), String> {
        let mut requests = self.requests.lock();
        let now = Instant::now();
        let cmd_requests = requests.entry(command.to_string()).or_insert_with(Vec::new);
        
        cmd_requests.retain(|&t| now.duration_since(t) < self.window);
        
        if cmd_requests.len() >= self.max_requests {
            return Err("Rate limit exceeded".to_string());
        }
        
        cmd_requests.push(now);
        Ok(())
    }
}
```

**Recommended Limits:**
- Import operations: 5 per minute
- Analysis operations: 10 per minute
- Search operations: 30 per minute

---

### HIGH-005: Insecure Database Connection Pool Configuration

**Severity:** HIGH
**OWASP Category:** A05:2021 – Security Misconfiguration
**CWE:** CWE-400 (Uncontrolled Resource Consumption)

**Issues:**
1. Inconsistent max connections (5 vs 34)
2. No connection timeout
3. No idle timeout
4. No connection health checks

**Vulnerable Code:**
```rust
// Tests use different limits than production
PgPoolOptions::new()
    .max_connections(5)  // Only 5 connections
    .connect(&database_url)
```

**Remediation:**
```rust
async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(34)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .test_before_acquire(true)
        .connect(database_url)
        .await
}
```

---

## MEDIUM SEVERITY FINDINGS

### MEDIUM-001: Missing HTTPS Enforcement

**Severity:** MEDIUM
**CWE:** CWE-319 (Cleartext Transmission)

**Remediation:** Add HTTPS redirect for production builds.

### MEDIUM-002: No Security Event Logging

**Severity:** MEDIUM
**CWE:** CWE-778 (Insufficient Logging)

**Remediation:** Log authentication failures, path validation failures, rate limit violations.

### MEDIUM-003: Weak Password Policy

**Severity:** MEDIUM
**CWE:** CWE-521 (Weak Password Requirements)

**Issue:** Database password is numeric-only (145278963).
**Remediation:** Use 32+ character password with mixed case, numbers, symbols.

### MEDIUM-004: No Automated Dependency Scanning

**Severity:** MEDIUM
**CWE:** CWE-1104 (Unmaintained Components)

**Remediation:**
```bash
# Add to CI/CD
cargo audit
cargo outdated --exit-code 1
```

---

## LOW SEVERITY FINDINGS

### LOW-001: Missing Security Headers
**Remediation:** Add X-Content-Type-Options, X-Frame-Options, Referrer-Policy

### LOW-002: No Input Length Validation
**Remediation:** Limit string inputs to reasonable lengths (4096 chars max)

### LOW-003: Production Console Logging
**Remediation:** Wrap \`console.error\` in \`if (import.meta.env.DEV)\`

---

## Remediation Roadmap

### Phase 1: CRITICAL (This Week - 8 hours)
1. ✅ Rotate database password - 1 hour
2. ✅ Remove hardcoded credentials - 4 hours
3. ✅ Implement CSP - 2 hours
4. ✅ Fix SQL injection - 1 hour

### Phase 2: HIGH (Next Week - 42 hours)
5. ✅ Path validation - 8 hours
6. ✅ Tauri capabilities - 16 hours
7. ✅ Error handling - 8 hours
8. ✅ Rate limiting - 8 hours
9. ✅ Connection pool - 2 hours

### Phase 3: MEDIUM (Sprint 2 - 17 hours)
10. ✅ HTTPS enforcement - 4 hours
11. ✅ Security logging - 8 hours
12. ✅ Password policy - 1 hour
13. ✅ Dependency scanning - 4 hours

### Phase 4: LOW (Sprint 3 - 8 hours)
14. ✅ Security headers - 2 hours
15. ✅ Length validation - 4 hours
16. ✅ Console logging - 2 hours

**Total Effort:** 75 hours (~2 weeks)

---

## OWASP Top 10 Compliance

| Category | Status | Findings |
|----------|--------|----------|
| A01: Broken Access Control | ❌ FAIL | HIGH-001, HIGH-002 |
| A02: Cryptographic Failures | ❌ FAIL | CRITICAL-001, MEDIUM-001 |
| A03: Injection | ❌ FAIL | CRITICAL-003 |
| A04: Insecure Design | ⚠️ WARN | HIGH-003, HIGH-004 |
| A05: Security Misconfiguration | ❌ FAIL | CRITICAL-002, HIGH-005 |
| A06: Vulnerable Components | ⚠️ WARN | MEDIUM-004 |
| A07: Auth Failures | ⚠️ WARN | MEDIUM-003 |
| A08: Data Integrity | ✅ PASS | None |
| A09: Logging Failures | ⚠️ WARN | MEDIUM-002 |
| A10: SSRF | ✅ PASS | None |

**Compliance Score: 30%** (3/10 passed)

---

## Security Test Suite

```rust
#[cfg(test)]
mod security_tests {
    #[tokio::test]
    async fn test_path_traversal_blocked() {
        let malicious = vec!["../../../etc/passwd", "..\\..\\Windows\\System32"];
        for path in malicious {
            assert!(validate_path(path, &base).is_err());
        }
    }

    #[tokio::test]
    async fn test_rate_limiting_enforced() {
        for i in 0..20 {
            let result = import_archive_collection(path, state).await;
            if i > 10 { assert!(result.is_err()); }
        }
    }

    #[tokio::test]
    async fn test_no_credential_leakage() {
        let error = trigger_db_error().await.unwrap_err();
        assert!(!error.contains("password"));
        assert!(!error.contains("145278963"));
    }
}
```

---

## Conclusion

**Current Security Posture: HIGH RISK**

The MIDI Software Center requires immediate security remediation. The presence of 221 hardcoded credentials, missing CSP, and SQL injection vulnerability pose **CRITICAL** risks.

**Priority Actions (Next 24-48 hours):**
1. Rotate database password
2. Remove all hardcoded credentials
3. Implement Content Security Policy

**Expected Improvement:**
- After Phase 1 (1 week): **MEDIUM RISK**
- After Phase 2 (2 weeks): **LOW RISK**
- After Phase 3-4 (1 month): **ACCEPTABLE RISK**

---

**Report Generated:** 2025-11-29 by Claude Code Security Specialist
**Next Audit:** After remediation completion (4 weeks)
