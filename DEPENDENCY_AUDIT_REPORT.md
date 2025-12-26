# Dependency Audit Report

**Date:** December 26, 2025
**Total Rust Dependencies:** 907 packages
**Frontend Dependencies:** 30 packages

---

## Critical Security Vulnerabilities

### Frontend (npm/pnpm)

| Severity | Package | Current | Fixed | Issue |
|----------|---------|---------|-------|-------|
| **CRITICAL** | happy-dom | ^12.10.3 | >=20.0.0 | VM Context Escape leading to RCE |
| **CRITICAL** | happy-dom | ^12.10.3 | >=15.10.2 | Script tag allows server-side code execution |
| **HIGH** | glob (via tailwindcss) | 10.2.x | >=10.5.0 | Command injection via -c/--cmd |
| **MODERATE** | esbuild (via vite) | <=0.24.2 | >=0.25.0 | Dev server request vulnerability |

**Immediate Actions Required:**
```json
// package.json - Update these dev dependencies:
"happy-dom": "^20.0.11"
```

### Backend (Rust)

| Severity | Package | Current | Fixed | Issue |
|----------|---------|---------|-------|-------|
| **HIGH** | sqlx | 0.7.4 | >=0.8.1 | Binary Protocol Misinterpretation (RUSTSEC-2024-0363) - potential exploit for Postgres |

**Immediate Action Required:**
```toml
# Cargo.toml - Update sqlx:
sqlx = { version = "0.8", features = [...] }
```

---

## Outdated Dependencies

### Frontend - Major Version Updates Available

| Package | Current | Latest | Breaking Changes |
|---------|---------|--------|------------------|
| svelte | ^4.2.8 | 5.46.1 | Svelte 5 has new reactivity system |
| eslint | ^8.56.0 | 9.39.2 | Flat config required |
| vite | ^5.0.8 | 7.3.0 | Major version upgrade |
| vitest | ^1.0.4 | 4.0.16 | Test runner changes |
| tailwindcss | ^3.4.17 | 4.1.18 | New config format |
| @sveltejs/vite-plugin-svelte | ^3.0.1 | 6.2.1 | Requires Svelte 5 |
| @typescript-eslint/* | ^6.0.0 | 8.50.1 | ESLint 9 compatibility |

### Rust - Notable Updates

| Package | Current | Recommended | Notes |
|---------|---------|-------------|-------|
| tokio | 1.35 | 1.48+ | Security fix for broadcast channel |
| reqwest | 0.11.27 | 0.12.x | Using both 0.11 and 0.12 (duplication) |
| tauri | 2.0 | 2.8 | Bug fixes and features |
| nom | 8.0.0 | Latest | Recently updated major version |

---

## Dependency Bloat Analysis

### Unused or Potentially Unused Dependencies

These dependencies are declared in Cargo.toml but appear unused in the source code:

| Package | Reason | Recommendation |
|---------|--------|----------------|
| `rimd = "0.0.1"` | No imports found | **Remove** - use `midly` only |
| `rust-music-theory = "0.3"` | No imports found | **Remove** - unused |
| `unrar = "0.5"` | CLI tool used instead of crate | **Remove** - calls `unrar` binary |
| `sevenz-rust = "0.5"` | No imports found | **Remove** if not needed |
| `monoio = "0.2.4"` | No imports found | **Remove** - alternative runtime |
| `snmalloc-rs = "0.3.8"` | Using mimalloc instead | **Remove** - redundant allocator |
| `simdeez = "2.0.0"` | No imports found | **Remove** - unused SIMD abstraction |
| `lockfree = "0.5.1"` | No imports found | **Remove** - unmaintained crate |
| `instant = "0.1"` | Deprecated crate | **Remove** - use `std::time::Instant` |
| `highway = "1.3.0"` | No imports found | Consider removing |
| `wyhash = "0.6.0"` | No imports found | Consider removing |
| `ultraviolet = "0.10.0"` | No imports found | Consider removing |

### Duplicate Dependencies (Multiple Versions)

The following packages have multiple versions in Cargo.lock, increasing binary size:

| Package | Versions | Impact |
|---------|----------|--------|
| base64 | 0.13.1, 0.21.7, 0.22.1 | Low |
| hashbrown | 0.14.x, 0.15.x | Medium |
| http | 0.2.12, 1.0.3 | Medium |
| hyper | 0.14.32, 1.8.1 | High |
| reqwest | 0.11.27, 0.12.24 | High |
| syn | 1.0.109, 2.0.111 | Expected |
| rand | 0.8.x, 0.9.x | Medium |
| indexmap | 1.x, 2.x | Medium |

### Redundant Functionality

| Category | Current | Consolidation |
|----------|---------|---------------|
| Directory paths | `directories + dirs` | Use only `dirs` |
| Compression | 8+ compression crates | Keep only what's used |
| String types | `smartstring + compact_str` | Choose one |
| Hash functions | `ahash + xxhash + blake3 + sha2` | Keep essential ones |

---

## Recommendations by Priority

### P0 - Security Critical (Do Immediately)

1. **Update sqlx to 0.8.1+** - Binary protocol vulnerability affects Postgres
2. **Update happy-dom to 20.0+** - RCE vulnerability in test environment

### P1 - Security Important (This Week)

3. Update vite to get esbuild fix (or pin esbuild >= 0.25.0)
4. Update tailwindcss to fix glob vulnerability in sucrase

### P2 - Cleanup (This Sprint)

5. Remove unused dependencies:
   ```toml
   # Remove these from app/src-tauri/Cargo.toml:
   rimd = "0.0.1"
   rust-music-theory = "0.3"
   unrar = "0.5"
   sevenz-rust = "0.5"
   monoio = "0.2.4"
   snmalloc-rs = "0.3.8"
   simdeez = "2.0.0"
   lockfree = "0.5.1"
   instant = "0.1"
   ```

6. Consolidate to single version:
   ```toml
   # Use only workspace reqwest 0.12:
   reqwest = { version = "0.12", ... }
   ```

### P3 - Technical Debt (Future)

7. Consider Svelte 5 migration (breaking changes)
8. Consider ESLint 9 migration (flat config)
9. Evaluate vite 7.x upgrade path
10. Remove deprecated crates (`atty`, etc.)

---

## Estimated Impact

| Action | Binary Size | Compile Time | Security |
|--------|-------------|--------------|----------|
| Remove unused deps | -5-10% | -10-15% | Improved |
| Fix duplicates | -3-5% | Minimal | Neutral |
| Update sqlx | Neutral | Neutral | **Critical** |
| Update happy-dom | N/A | N/A | **Critical** |

---

## Commands to Apply Fixes

```bash
# Frontend security fixes
cd app
pnpm update happy-dom@latest

# Rust security fix
# Edit Cargo.toml then:
cargo update -p sqlx

# Check for more vulnerabilities (when network permits)
cargo install cargo-audit
cargo audit

# Find unused dependencies
cargo install cargo-udeps
cargo +nightly udeps
```

---

## Sources

- [RustSec Advisory Database](https://rustsec.org/advisories/)
- [RUSTSEC-2024-0363: SQLx Binary Protocol Vulnerability](https://rustsec.org/advisories/RUSTSEC-2024-0363.html)
- [GitHub Advisory: happy-dom RCE](https://github.com/advisories/GHSA-37j7-fg3j-429f)
- [GitHub Advisory: glob command injection](https://github.com/advisories/GHSA-5j98-mcp5-4vw2)
