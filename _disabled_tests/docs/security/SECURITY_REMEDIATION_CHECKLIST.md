# Security Remediation Checklist

**Project:** MIDI Software Center
**Date:** 2025-11-29
**Reference:** SECURITY_AUDIT_REPORT.md

---

## CRITICAL - DO IMMEDIATELY (Today)

### [ ] CRITICAL-001: Rotate Database Password & Remove Hardcoded Credentials

**Time:** 1-2 hours
**Files Affected:** 221 files

```bash
# Step 1: Generate new password
NEW_PASSWORD=$(openssl rand -base64 32)
echo "Generated password: $NEW_PASSWORD"

# Step 2: Update PostgreSQL
psql -U postgres -c "ALTER USER midiuser WITH PASSWORD '$NEW_PASSWORD';"

# Step 3: Create .env file
cat > .env << EOF
DATABASE_URL=postgresql://midiuser:$NEW_PASSWORD@localhost:5433/midi_library
EOF

# Step 4: Create .env.example template
cat > .env.example << EOF
DATABASE_URL=postgresql://USERNAME:PASSWORD@localhost:5433/DATABASE_NAME
EOF

# Step 5: Add to .gitignore
echo ".env" >> .gitignore

# Step 6: Find all hardcoded instances
grep -rn "145278963" --include="*.rs" > /tmp/hardcoded_passwords.txt
echo "Found $(wc -l < /tmp/hardcoded_passwords.txt) instances"

# Step 7: Replace all instances with env var requirement
find . -name "*.rs" -type f -print0 | xargs -0 sed -i 's/unwrap_or_else(|_| {[^}]*"postgresql:\/\/midiuser:145278963@localhost:5433\/midi_library"[^}]*})/expect("DATABASE_URL environment variable must be set")/g'

# Step 8: Verify removal
grep -r "145278963" --include="*.rs" && echo "STILL FOUND!" || echo "All removed!"
```

**Verification:**
- [ ] No instances of "145278963" found in codebase
- [ ] .env file created and added to .gitignore
- [ ] All tests require DATABASE_URL environment variable
- [ ] PostgreSQL password changed

---

### [ ] CRITICAL-002: Implement Content Security Policy

**Time:** 30 minutes per app (1.5 hours total)
**Files:** 3 tauri.conf.json files

#### App 1: /home/dojevou/projects/midi-software-center/app/src-tauri/tauri.conf.json

```bash
# Backup original
cp app/src-tauri/tauri.conf.json app/src-tauri/tauri.conf.json.backup

# Update CSP
cat > /tmp/csp_patch.json << 'EOF'
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self' ws://localhost:* http://localhost:*; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'"
    }
EOF
```

**Manual Steps:**
1. [ ] Open `app/src-tauri/tauri.conf.json`
2. [ ] Replace `"csp": null` with the CSP string above
3. [ ] Repeat for `pipeline/src-tauri/tauri.conf.json`
4. [ ] Repeat for `daw/src-tauri/tauri.conf.json`
5. [ ] Test all apps with new CSP
6. [ ] Verify no console CSP violations

---

### [ ] CRITICAL-003: Fix SQL Injection in Test Helpers

**Time:** 15 minutes
**File:** pipeline/src-tauri/tests/helpers/db.rs:130

```rust
// REMOVE this function entirely
// pub async fn count_files_where(pool: &PgPool, condition: &str) -> Result<i64, sqlx::Error>

// REPLACE with type-safe alternatives:
pub async fn count_files_by_filename(pool: &PgPool, filename: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!("SELECT COUNT(*) FROM files WHERE filename = $1", filename)
        .fetch_one(pool)
        .await
}

pub async fn count_files_by_manufacturer(pool: &PgPool, manufacturer: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!("SELECT COUNT(*) FROM files WHERE manufacturer = $1", manufacturer)
        .fetch_one(pool)
        .await
}
```

**Steps:**
1. [ ] Open `pipeline/src-tauri/tests/helpers/db.rs`
2. [ ] Delete `count_files_where` function (lines 128-133)
3. [ ] Add type-safe alternatives above
4. [ ] Find all usages: `grep -rn "count_files_where" tests/`
5. [ ] Replace each usage with appropriate type-safe function
6. [ ] Run tests: `cargo test --package midi-pipeline`

---

## HIGH PRIORITY - This Week

### [ ] HIGH-001: Implement Path Traversal Validation

**Time:** 4-6 hours
**Files:** 4 command files + 1 shared module

**Create shared validation module:**

```bash
# Create new file
cat > pipeline/src-tauri/src/security/path_validation.rs << 'RUST'
use std::path::{Path, PathBuf};
use std::fs::canonicalize;

pub struct PathValidator {
    allowed_bases: Vec<PathBuf>,
}

impl PathValidator {
    pub fn new() -> Self {
        Self {
            allowed_bases: vec![
                PathBuf::from("/home/dojevou/projects/midi-software-center/midi-library"),
                PathBuf::from("/tmp/midi_splits_fast"),
            ],
        }
    }

    pub fn validate(&self, path: &str) -> Result<PathBuf, String> {
        let canonical = canonicalize(PathBuf::from(path))
            .map_err(|e| format!("Invalid path: {}", e))?;

        for base in &self.allowed_bases {
            if canonical.starts_with(base) {
                return Ok(canonical);
            }
        }

        Err(format!("Path traversal detected: {} is outside allowed directories", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_path_traversal() {
        let validator = PathValidator::new();
        assert!(validator.validate("../../../etc/passwd").is_err());
        assert!(validator.validate("..\\..\\Windows\\System32").is_err());
    }
}
RUST

# Add to mod.rs
echo "pub mod path_validation;" >> pipeline/src-tauri/src/security/mod.rs
```

**Update each command file:**
1. [ ] `pipeline/src-tauri/src/commands/archive_import.rs`
2. [ ] `pipeline/src-tauri/src/commands/file_import.rs`
3. [ ] `pipeline/src-tauri/src/commands/analyze.rs`
4. [ ] `pipeline/src-tauri/src/commands/split_file.rs`

**Pattern to apply:**
```rust
use crate::security::path_validation::PathValidator;

#[tauri::command]
pub async fn import_archive_collection(
    collection_path: String,
    state: State<'_, AppState>,
) -> Result<ArchiveImportSummary, String> {
    let validator = PathValidator::new();
    let validated_path = validator.validate(&collection_path)?;
    
    // Use validated_path for all operations
    // ...
}
```

---

### [ ] HIGH-002: Implement Tauri Capability System

**Time:** 8-12 hours
**Reference:** https://v2.tauri.app/concept/security/

**Steps:**
1. [ ] Create capabilities directory structure
2. [ ] Define capability sets for each operation category
3. [ ] Update tauri.conf.json with capability references
4. [ ] Test each capability set
5. [ ] Document permission requirements

```bash
mkdir -p app/src-tauri/capabilities
mkdir -p pipeline/src-tauri/capabilities
mkdir -p daw/src-tauri/capabilities
```

**Example capability file:**
```json
// app/src-tauri/capabilities/main.json
{
  "identifier": "main-capabilities",
  "description": "Main application permissions",
  "windows": ["main"],
  "permissions": [
    "core:window:allow-close",
    "core:window:allow-minimize",
    "fs:allow-read-text-file",
    {
      "identifier": "midi-import",
      "description": "MIDI file import operations",
      "commands": {
        "allow": [
          "import_archive_collection",
          "import_directory",
          "analyze_files"
        ],
        "deny": []
      }
    }
  ]
}
```

---

### [ ] HIGH-003: Implement Secure Error Handling

**Time:** 4-6 hours

**Create error module:**
```rust
// pipeline/src-tauri/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("Invalid input provided")]
    InvalidInput,

    #[error("Operation not permitted")]
    Forbidden,

    #[error("Operation failed")]
    OperationFailed,

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        match err {
            AppError::Internal(e) => {
                eprintln!("Internal error (not shown to user): {:?}", e);
                "An internal error occurred".to_string()
            }
            _ => err.to_string(),
        }
    }
}
```

**Update commands to use AppError:**
- [ ] Replace all `.map_err(|e| format!(...))` with `AppError::Internal`
- [ ] Remove path information from error messages
- [ ] Remove database schema details from errors

---

### [ ] HIGH-004: Implement Rate Limiting

**Time:** 4-6 hours

**Create rate limiter:**
```rust
// pipeline/src-tauri/src/security/rate_limiter.rs
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::{Instant, Duration};

pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    limits: HashMap<String, (usize, Duration)>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let mut limits = HashMap::new();
        
        // Define limits per command
        limits.insert("import_archive_collection".to_string(), (5, Duration::from_secs(60)));
        limits.insert("analyze_files".to_string(), (10, Duration::from_secs(60)));
        limits.insert("search_files".to_string(), (30, Duration::from_secs(60)));
        
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            limits,
        }
    }

    pub fn check_limit(&self, command: &str) -> Result<(), String> {
        let (max_requests, window) = self.limits
            .get(command)
            .ok_or_else(|| "Unknown command".to_string())?;

        let mut requests = self.requests.lock();
        let now = Instant::now();
        
        let cmd_requests = requests.entry(command.to_string()).or_insert_with(Vec::new);
        cmd_requests.retain(|&t| now.duration_since(t) < *window);
        
        if cmd_requests.len() >= *max_requests {
            return Err(format!("Rate limit exceeded: {} requests per {} seconds", 
                max_requests, window.as_secs()));
        }
        
        cmd_requests.push(now);
        Ok(())
    }
}
```

**Add to AppState and use in commands**

---

### [ ] HIGH-005: Fix Database Connection Pool

**Time:** 1 hour

**Create standardized pool creator:**
```rust
// shared/rust/src/db/pool.rs
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
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

**Replace all manual pool creation with this function**

---

## MEDIUM PRIORITY - Next Sprint

### [ ] MEDIUM-001: HTTPS Enforcement
- [ ] Add HTTPS redirect middleware
- [ ] Configure TLS certificates
- [ ] Update all localhost URLs to support HTTPS in dev

### [ ] MEDIUM-002: Security Event Logging
- [ ] Create security log module
- [ ] Log authentication failures
- [ ] Log path validation failures
- [ ] Log rate limit violations
- [ ] Log permission denials

### [ ] MEDIUM-003: Strong Password Policy
- [ ] Document password requirements (32+ chars, mixed case, symbols)
- [ ] Create password generation script
- [ ] Add password rotation reminder

### [ ] MEDIUM-004: Dependency Scanning
```yaml
# .github/workflows/security.yml
name: Security Audit

on: [push, pull_request]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - name: Security audit
        run: |
          cargo install cargo-audit
          cargo audit
      - name: Check outdated
        run: |
          cargo install cargo-outdated
          cargo outdated --exit-code 1
```

---

## LOW PRIORITY - Future Work

### [ ] LOW-001: Security Headers
### [ ] LOW-002: Input Length Validation
### [ ] LOW-003: Production Console Logging

---

## Testing Requirements

After each phase, run:

```bash
# Unit tests
cargo test --workspace

# Security tests
cargo test --package midi-pipeline security_tests

# Integration tests
cargo test --workspace --test '*'

# Manual verification
- [ ] Path traversal blocked
- [ ] Rate limiting enforced
- [ ] No credentials in errors
- [ ] CSP violations logged
```

---

## Completion Criteria

**Phase 1 (CRITICAL) Complete When:**
- [ ] No hardcoded credentials in codebase
- [ ] CSP implemented in all 3 apps
- [ ] SQL injection patched and tested
- [ ] All tests passing

**Phase 2 (HIGH) Complete When:**
- [ ] Path validation on all file operations
- [ ] Tauri capabilities configured
- [ ] Error handling sanitized
- [ ] Rate limiting active
- [ ] Connection pools standardized

**Phase 3 (MEDIUM) Complete When:**
- [ ] Security logging implemented
- [ ] Dependency scanning in CI/CD
- [ ] Strong passwords enforced

---

**Last Updated:** 2025-11-29
**Owner:** Development Team
**Review Date:** After Phase 1 completion
