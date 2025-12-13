# Security Remediation Quick Start Guide

**Priority:** CRITICAL - Must complete before production deployment
**Time Required:** 1 week (40 hours for CRITICAL issues)
**Status:** 🔴 BLOCKING

---

## Step 1: Remove Hardcoded Credentials (Day 1-2)

### 1a. Change Database Password (1 hour)

```bash
# Generate strong random password
NEW_PASSWORD=$(openssl rand -base64 32)

# Connect to PostgreSQL
psql postgresql://postgres@localhost:5433/postgres

# Change password
ALTER USER midiuser WITH PASSWORD 'new_secure_password_here';
\q
```

### 1b. Update Environment Files (2 hours)

```bash
# Update .env (DO NOT COMMIT)
echo "DATABASE_URL=postgresql://midiuser:${NEW_PASSWORD}@localhost:5433/midi_library" > .env

# Update .env.example (SAFE TO COMMIT)
cat > .env.example << 'EOF'
# Database Configuration
DATABASE_URL=postgresql://username:password@localhost:5432/midi_library
DATABASE_PASSWORD=changeme
EOF

# Verify .env is in .gitignore
grep -q "^\.env$" .gitignore || echo ".env" >> .gitignore
```

### 1c. Remove Hardcoded Passwords from Code (4 hours)

```bash
# Find all files with hardcoded password
grep -rl "145278963" --include="*.rs" --include="*.md" --include="*.sh" . > /tmp/files_to_fix.txt

# Create backup
tar -czf security_fix_backup_$(date +%Y%m%d_%H%M%S).tar.gz \
  $(cat /tmp/files_to_fix.txt)

# Replace hardcoded password with environment variable placeholder
find . -type f \( -name "*.rs" -o -name "*.md" -o -name "*.sh" \) \
  -exec sed -i.bak 's/postgresql:\/\/midiuser:145278963@localhost:5433\/midi_library/postgresql:\/\/midiuser:${DATABASE_PASSWORD}@localhost:5433\/midi_library/g' {} +

# For Rust code, use env::var
find . -name "*.rs" -type f \
  -exec sed -i.bak 's/"postgresql:\/\/midiuser:\$\{DATABASE_PASSWORD\}@localhost:5433\/midi_library"/std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql:\/\/midiuser:password@localhost:5433\/midi_library".to_string())/g' {} +
```

### 1d. Add Pre-commit Hook (1 hour)

```bash
# Create pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook to prevent secrets from being committed

# Check for hardcoded passwords
if git diff --cached | grep -E "(password|secret|api_key|token).*=.*['\"][^'\"]{8,}"; then
    echo "❌ ERROR: Potential secret detected in commit!"
    echo "Please use environment variables instead."
    exit 1
fi

# Check for database connection strings with passwords
if git diff --cached | grep -E "postgresql://[^:]+:[^@]+@"; then
    echo "❌ ERROR: Database connection string with password detected!"
    echo "Use environment variables: DATABASE_URL=postgresql://..."
    exit 1
fi

exit 0
EOF

chmod +x .git/hooks/pre-commit
```

### 1e. Clean Git History (2 hours)

```bash
# WARNING: This rewrites git history!
# Coordinate with team before running

# Install BFG Repo-Cleaner
wget https://repo1.maven.org/maven2/com/madgag/bfg/1.14.0/bfg-1.14.0.jar

# Replace password in entire git history
java -jar bfg-1.14.0.jar --replace-text passwords.txt

# passwords.txt content:
cat > passwords.txt << 'EOF'
145278963==${DATABASE_PASSWORD}
EOF

# Force push (coordinate with team!)
git reflog expire --expire=now --all
git gc --prune=now --aggressive
# git push --force --all (ONLY after team approval)
```

---

## Step 2: Enable Content Security Policy (Day 3)

### 2a. Update Tauri Configuration (30 minutes)

```bash
# Edit app/src-tauri/tauri.conf.json
cat > app/src-tauri/tauri.conf.json << 'EOF'
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
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' http://localhost:* ws://localhost:*; font-src 'self'"
    }
  }
}
EOF
```

### 2b. Test Application (2 hours)

```bash
# Run development build
cd app
pnpm dev

# Open browser console (F12)
# Look for CSP violations
# Adjust policy if legitimate resources are blocked
```

### 2c. Adjust CSP for Svelte (if needed)

If you see CSP violations for Svelte's inline styles, use nonces:

```json
{
  "security": {
    "csp": {
      "default-src": "'self'",
      "script-src": ["'self'"],
      "style-src": ["'self'", "'unsafe-inline'"],
      "img-src": ["'self'", "data:"],
      "connect-src": ["'self'", "http://localhost:*", "ws://localhost:*"]
    }
  }
}
```

---

## Step 3: Add Path Validation (Day 4)

### 3a. Create Security Module (2 hours)

```rust
// pipeline/src-tauri/src/security/mod.rs
pub mod path_validator;

// pipeline/src-tauri/src/security/path_validator.rs
use std::path::{Path, PathBuf};

/// Validate file path to prevent path traversal attacks
///
/// # Arguments
/// * `path` - Path to validate
/// * `allowed_base` - Base directory that path must be within
///
/// # Returns
/// * `Ok(PathBuf)` - Canonicalized, validated path
/// * `Err(String)` - Error message if path is invalid
///
/// # Security
/// - Prevents path traversal (../)
/// - Prevents symlink attacks
/// - Ensures path is within allowed directory
pub fn validate_file_path(path: &str, allowed_base: &Path) -> Result<PathBuf, String> {
    let path = Path::new(path);

    // 1. Reject obvious path traversal attempts
    if path.components().any(|c| c.as_os_str() == "..") {
        return Err("Path traversal detected".to_string());
    }

    // 2. Canonicalize to resolve symlinks and ".."
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    // 3. Ensure path is within allowed base directory
    if !canonical.starts_with(allowed_base) {
        return Err(format!(
            "Path outside allowed directory: {} (allowed: {})",
            canonical.display(),
            allowed_base.display()
        ));
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_valid_path() {
        let base = env::current_dir().unwrap();
        let result = validate_file_path("Cargo.toml", &base);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_traversal() {
        let base = env::current_dir().unwrap();
        let result = validate_file_path("../../etc/passwd", &base);
        assert!(result.is_err());
    }

    #[test]
    fn test_absolute_path_outside_base() {
        let base = env::current_dir().unwrap();
        let result = validate_file_path("/etc/passwd", &base);
        assert!(result.is_err());
    }
}
```

### 3b. Apply to Import Commands (2 hours)

```rust
// pipeline/src-tauri/src/commands/file_import.rs
use crate::security::path_validator::validate_file_path;

pub async fn import_single_file_impl(
    file_path: String,
    category: Option<String>,
    state: &AppState,
) -> Result<FileMetadata, String> {
    // Define allowed base directory
    let allowed_base = std::env::var("MIDI_LIBRARY_PATH")
        .unwrap_or_else(|_| "/home/dojevou/projects/midi-software-center/midi-library".to_string());
    let allowed_base = Path::new(&allowed_base);

    // Validate path BEFORE any file operations
    let validated_path = validate_file_path(&file_path, allowed_base)?;

    // Check file exists
    if !validated_path.exists() {
        return Err(format!("File not found: {}", validated_path.display()));
    }

    // Check file type
    if !is_midi_file(&validated_path) {
        return Err("Not a MIDI file".to_string());
    }

    // Process the file (now safe)
    let processed = process_single_file(&validated_path, category.clone())
        .await
        .map_err(|e| format!("Failed to process file: {}", e))?;

    // ... rest of implementation
}
```

---

## Step 4: Add Rate Limiting (Day 5)

### 4a. Create Rate Limiter (3 hours)

```rust
// pipeline/src-tauri/src/security/rate_limiter.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct RateLimiter {
    /// Track requests per command
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    /// Maximum requests allowed in time window
    max_requests: usize,
    /// Time window for rate limiting
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window: Duration::from_secs(window_seconds),
        }
    }

    /// Check if request should be rate limited
    pub async fn check_rate_limit(&self, command: &str) -> Result<(), String> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();

        // Get or create request history for this command
        let command_requests = requests.entry(command.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the time window
        command_requests.retain(|&req| now.duration_since(req) < self.window);

        // Check if rate limit exceeded
        if command_requests.len() >= self.max_requests {
            return Err(format!(
                "Rate limit exceeded for {}: {} requests in {} seconds. Try again in {} seconds.",
                command,
                self.max_requests,
                self.window.as_secs(),
                (self.window.as_secs() as i64
                    - now.duration_since(command_requests[0]).as_secs() as i64)
                    .max(1)
            ));
        }

        // Add current request
        command_requests.push(now);

        Ok(())
    }

    /// Reset rate limit for a command (for testing)
    pub async fn reset(&self, command: &str) {
        let mut requests = self.requests.lock().await;
        requests.remove(command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit() {
        let limiter = RateLimiter::new(3, 60);

        // First 3 requests should succeed
        assert!(limiter.check_rate_limit("test_command").await.is_ok());
        assert!(limiter.check_rate_limit("test_command").await.is_ok());
        assert!(limiter.check_rate_limit("test_command").await.is_ok());

        // 4th request should fail
        assert!(limiter.check_rate_limit("test_command").await.is_err());

        // Different command should succeed
        assert!(limiter.check_rate_limit("other_command").await.is_ok());
    }
}
```

### 4b. Apply to Commands (2 hours)

```rust
// pipeline/src-tauri/src/main.rs
use security::rate_limiter::RateLimiter;

#[tokio::main]
async fn main() {
    // ... existing setup

    // Create rate limiters for different command types
    let import_limiter = RateLimiter::new(10, 60);  // 10 imports per minute
    let search_limiter = RateLimiter::new(100, 60); // 100 searches per minute
    let analysis_limiter = RateLimiter::new(20, 60); // 20 analyses per minute

    tauri::Builder::default()
        .manage(import_limiter)
        .manage(search_limiter)
        .manage(analysis_limiter)
        // ... rest of builder
}

// pipeline/src-tauri/src/commands/file_import.rs
use tauri::State;
use crate::security::rate_limiter::RateLimiter;

#[tauri::command]
pub async fn import_single_file(
    file_path: String,
    category: Option<String>,
    state: State<'_, AppState>,
    rate_limiter: State<'_, RateLimiter>,
) -> Result<FileMetadata, String> {
    // Check rate limit FIRST
    rate_limiter.check_rate_limit("import_single_file").await?;

    // Proceed with import
    import_single_file_impl(file_path, category, &state).await
}
```

---

## Step 5: Sanitize Error Messages (Day 6)

### 5a. Create Error Sanitizer (1 hour)

```rust
// pipeline/src-tauri/src/security/error_sanitizer.rs

/// Sanitize error messages to prevent information disclosure
pub fn sanitize_error(error: impl std::fmt::Display) -> String {
    let error_str = error.to_string();

    // Remove database connection strings
    let sanitized = regex::Regex::new(r"postgresql://[^@]+@[^\s]+")
        .unwrap()
        .replace_all(&error_str, "postgresql://***:***@***/***");

    // Remove file paths (optional - may break debugging)
    // let sanitized = regex::Regex::new(r"/[^\s]+")
    //     .unwrap()
    //     .replace_all(&sanitized, "/***");

    sanitized.to_string()
}

/// Log full error internally, return sanitized error to user
pub fn log_and_sanitize_error(error: impl std::fmt::Display, context: &str) -> String {
    // Log full error for debugging
    log::error!("{}: {}", context, error);

    // Return sanitized error to user
    sanitize_error(error)
}
```

### 5b. Apply to All Commands (2 hours)

```rust
// Replace all .map_err with sanitized errors
// BEFORE:
.map_err(|e| format!("Database error: {}", e))?

// AFTER:
.map_err(|e| log_and_sanitize_error(e, "Database connection"))?
```

---

## Step 6: Verify and Test (Day 7)

### 6a. Run Security Checks (2 hours)

```bash
# Verify no hardcoded credentials
grep -r "145278963" --include="*.rs" --include="*.md" .
# Expected: 0 results

# Verify CSP is enabled
grep -A 5 '"security"' app/src-tauri/tauri.conf.json
# Expected: "csp": "default-src 'self'; ..."

# Verify path validation
cargo test -p midi-pipeline security::path_validator
# Expected: All tests pass

# Verify rate limiting
cargo test -p midi-pipeline security::rate_limiter
# Expected: All tests pass
```

### 6b. Integration Testing (3 hours)

```bash
# Test import with valid path
cargo run --bin import -- --source /home/dojevou/projects/midi-software-center/midi-library

# Test import with invalid path (should fail)
cargo run --bin import -- --source /etc/passwd
# Expected: "Path outside allowed directory"

# Test rate limiting (should fail after 10 requests)
for i in {1..15}; do
  curl -X POST http://localhost:1420/import_single_file
done
# Expected: "Rate limit exceeded" on 11th request
```

### 6c. Update Documentation (1 hour)

```bash
# Update README.md
cat >> README.md << 'EOF'

## Security

This application implements the following security controls:

- **Credential Management:** All credentials stored in environment variables
- **Content Security Policy:** XSS protection enabled
- **Input Validation:** Path traversal prevention
- **Rate Limiting:** DoS protection on all commands
- **Error Handling:** Sanitized error messages

For security issues, contact: security@example.com
EOF
```

---

## Verification Checklist

Before marking security issues as resolved, verify:

### CRITICAL Issues:
- [ ] Database password changed to strong random value
- [ ] All 287 files updated to use environment variables
- [ ] .env file added to .gitignore
- [ ] Pre-commit hook installed and tested
- [ ] Git history cleaned (optional, coordinate with team)
- [ ] CSP enabled in tauri.conf.json
- [ ] Application tested with CSP (no violations)
- [ ] Path validation module created and tested
- [ ] Rate limiting implemented for all commands
- [ ] Error messages sanitized

### Testing:
- [ ] `cargo test --workspace` passes
- [ ] No hardcoded credentials found: `grep -r "145278963" .`
- [ ] CSP violations checked in browser console
- [ ] Path traversal tests pass
- [ ] Rate limiting tests pass
- [ ] Integration tests pass

### Documentation:
- [ ] README.md updated with security controls
- [ ] .env.example updated with placeholders
- [ ] SECURITY.md created with threat model
- [ ] Pre-commit hook documented

---

## Estimated Time

| Task | Time |
|------|------|
| Remove hardcoded credentials | 8 hours |
| Enable CSP | 3 hours |
| Add path validation | 4 hours |
| Add rate limiting | 5 hours |
| Sanitize error messages | 3 hours |
| Testing & verification | 5 hours |
| Documentation | 2 hours |
| **TOTAL** | **30 hours** |

---

## Next Steps

After completing CRITICAL fixes:

1. **HIGH Priority (Week 2):**
   - Implement authentication (if multi-user)
   - Add API versioning
   - Add file size limits

2. **MEDIUM Priority (Month 1):**
   - HTTPS enforcement (if networked)
   - CSRF protection (if web version)
   - Security headers

3. **Ongoing:**
   - Dependency vulnerability scanning
   - Penetration testing
   - Security training for team

---

## Support

For questions or issues:
- Full Report: `/home/dojevou/projects/midi-software-center/SECURITY_AUDIT_FINAL_REPORT.md`
- Executive Summary: `/home/dojevou/projects/midi-software-center/SECURITY_EXECUTIVE_SUMMARY.txt`
