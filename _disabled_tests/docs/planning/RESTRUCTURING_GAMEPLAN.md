# MIDI Library System - Complete Restructuring Gameplan

**Date:** October 23, 2025  
**Project:** MIDI Library System (3-tier architecture with Pipeline + DAW)  
**OS:** Ubuntu Studio 25.04  
**Available Tools:** Claude Code, Kilo Code, VS Code

---

## 📋 EXECUTIVE SUMMARY

Your project has **solid foundations** with well-organized documentation and clear phasing. The restructure.txt guide provides proven patterns for CLI tools, configuration management, and deployment. This gameplan identifies:

1. **Reusable scripts/configs** from your existing project ✅
2. **New patterns to implement** from restructure.txt 📝
3. **Migration strategy** (no data loss, backward compatible) 🔄
4. **Implementation roadmap** (phased, testable) 📊

**Key Finding:** Your project is 60% structured, 40% in progress. We can modernize without breaking current functionality.

---

## 📊 PHASE BREAKDOWN & LANGUAGE USAGE

### Database Layer (Phase 1)
- **SQL:** 90% (schema, migrations, queries)
- **Bash:** 5% (setup scripts)
- **YAML:** 5% (Docker Compose)

### Pipeline Backend (Phase 2)
- **Rust:** 95% (core batch processing)
- **TOML:** 5% (Cargo.toml config)

### Pipeline Frontend (Phase 3)
- **Svelte:** 40%
- **TypeScript:** 40%
- **HTML/CSS:** 20%

### DAW Backend (Phase 4)
- **Rust:** 95% (MIDI I/O, real-time sequencer)
- **TOML:** 5%

### DAW Frontend (Phase 5)
- **Svelte:** 35%
- **TypeScript:** 45% (complex state management)
- **HTML/CSS:** 20%

---

## ✅ REUSABLE SCRIPTS & CONFIGS FROM YOUR PROJECT

### **CURRENTLY FUNCTIONAL & REUSABLE**

#### Database Layer
```
✅ database/docker-compose.yml
   - PostgreSQL 16 + pgvector setup
   - Meilisearch configuration
   - Status: Production-ready
   - Action: Use as-is or enhance with environment variables

✅ database/migrations/*.sql
   - Schema definitions (001_initial_schema.sql)
   - Status: Partially complete
   - Action: Audit and finalize; add versioning

✅ database/scripts/*.sh
   - Database utility scripts
   - Status: Functional but needs review
   - Action: Refactor into task-o-matic pattern
```

#### Backend (Rust)
```
✅ Cargo.toml (root)
   - Workspace configuration
   - Dependencies: Tauri, sqlx, rimd, midly, tokio
   - Status: Complete
   - Action: Add version pinning for reproducibility

✅ api/*.rs files
   - Tauri command handlers
   - API types and serialization
   - Status: Needs unification
   - Action: Consolidate into single command module

✅ src-tauri/src/main.rs (both)
   - Application entry points
   - Status: Duplicate setup
   - Action: Refactor into shared template
```

#### Frontend (Svelte/TypeScript)
```
✅ pipeline/package.json & daw/package.json
   - Dependency management
   - Status: Duplicated dependencies
   - Action: Create shared package.json pattern or monorepo

✅ pipeline/src/* & daw/src/*
   - UI components and logic
   - Status: Similar patterns, some duplication
   - Action: Extract shared components into /shared/ui

✅ tsconfig.json (both)
   - TypeScript configuration
   - Status: Similar but not identical
   - Action: Create base tsconfig with extends pattern
```

#### Configuration & Setup
```
✅ Makefile
   - Build targets defined
   - Status: Basic but incomplete
   - Action: Expand with full lifecycle targets

✅ scripts/*.sh (various)
   - Launcher scripts
   - Status: Functional
   - Action: Modernize with error handling and logging

✅ .vscode-* files
   - VS Code configuration
   - Status: Complete but needs integration
   - Action: Convert to .vscode/settings.json pattern
```

### **PARTIALLY FUNCTIONAL & NEEDS REFACTORING**

```
⚠️ Database connection logic
   - Currently scattered across modules
   - Recommendation: Centralize in connection pool module

⚠️ MIDI file parsing
   - Split between pipeline and DAW
   - Recommendation: Extract to /shared/rust/midi-parser

⚠️ State management
   - DAW state complex and scattered
   - Recommendation: Implement unified state module with derive-macros

⚠️ Documentation
   - Extensive but fragmented across 50+ .md files
   - Recommendation: Create single source of truth index
```

---

## 📥 SCRIPTS & CONFIGS FROM restructure.txt TO IMPLEMENT

### **High Priority (Do First)**

#### 1. **Task-O-Matic Pattern** (CLI tool for common tasks)
```
Location: scripts/tasks/
Creates: Standardized command interface

Files to Create:
- scripts/tasks/task-o-matic.sh (main dispatcher)
- scripts/tasks/db/migrate.task (database migrations)
- scripts/tasks/build/compile.task (Rust compilation)
- scripts/tasks/frontend/bundle.task (Svelte/TS build)
- scripts/tasks/dev/watch.task (development watching)
- scripts/tasks/test/run.task (testing suite)

Benefits:
✓ Unified command interface
✓ Consistent error handling
✓ Progress reporting
✓ Easy to chain operations
```

#### 2. **Configuration Management Pattern**
```
Location: config/
Creates: Centralized config with environment overrides

Files to Create:
- config/defaults.conf (all default values)
- config/development.conf (dev overrides)
- config/production.conf (prod overrides)
- config/testing.conf (test overrides)
- config/load-config.sh (loader with fallbacks)

Environment Variables Needed:
- DATABASE_URL (PostgreSQL connection)
- MEILISEARCH_HOST (search engine host)
- RUST_LOG (logging level)
- NODE_ENV (frontend environment)
- API_PORT (Tauri IPC port)
```

#### 3. **Grown-Up Script Pattern** (Complex operational scripts)
```
Location: scripts/grown-up/
Creates: Production-grade scripts with robust error handling

Files to Create:
- scripts/grown-up/backup-database.sh
  Purpose: Full PostgreSQL backup with versioning
  Features: Compression, retention policy, error recovery

- scripts/grown-up/migrate-production.sh
  Purpose: Safe database migration with rollback
  Features: Validation, dry-run mode, automatic backup

- scripts/grown-up/deploy-pipeline.sh
  Purpose: Build and deploy pipeline application
  Features: Testing, versioning, health checks

- scripts/grown-up/deploy-daw.sh
  Purpose: Build and deploy DAW application
  Features: Testing, versioning, health checks

- scripts/grown-up/monitor-services.sh
  Purpose: Health check and auto-recovery
  Features: Process monitoring, log rotation, alerts
```

#### 4. **Trusty Module Pattern** (Reusable Bash modules)
```
Location: scripts/modules/
Creates: Sourced modules for common operations

Files to Create:
- scripts/modules/log.sh (logging utilities)
  Functions: log_info, log_error, log_debug, log_warn
  
- scripts/modules/database.sh (DB operations)
  Functions: db_connect, db_query, db_migrate, db_rollback
  
- scripts/modules/docker.sh (Docker operations)
  Functions: docker_start, docker_stop, docker_logs, docker_clean
  
- scripts/modules/build.sh (Build operations)
  Functions: build_rust, build_frontend, build_all
  
- scripts/modules/validation.sh (Input validation)
  Functions: validate_path, validate_url, validate_port
  
- scripts/modules/error-handler.sh (Error handling)
  Functions: trap_error, cleanup_on_exit, assert_success
```

### **Medium Priority (Second Wave)**

#### 5. **Unified Makefile Targets**
```
Current: Basic Makefile
New: Comprehensive lifecycle management

Targets to Add:
help                 # Display all available targets
setup                # Initial project setup
clean                # Remove build artifacts
build-all            # Build database, pipeline, DAW
test-all             # Run all tests
run-dev              # Start development environment
run-prod             # Start production environment
migrate-db           # Run database migrations
backup-db            # Backup database
docs-generate        # Generate documentation
deploy-pipeline      # Deploy pipeline to production
deploy-daw           # Deploy DAW to production
```

#### 6. **Docker Compose Enhancement**
```
Current: Basic PostgreSQL + Meilisearch
New: Full-featured development & production setups

Files to Create:
- docker-compose.dev.yml
  Services: PostgreSQL, Meilisearch, pgAdmin (optional)
  Features: Hot reload, detailed logging, seed data
  
- docker-compose.prod.yml
  Services: PostgreSQL, Meilisearch, Redis (caching)
  Features: Backups, monitoring, security hardening
  
- docker-compose.test.yml
  Services: PostgreSQL (clean state), Meilisearch
  Features: Parallel test support, cleanup after run
```

#### 7. **Environment & Secrets Management**
```
Files to Create:
- .env.example (template with all required variables)
- .env.local (local development overrides - gitignored)
- .env.test (test environment setup)
- scripts/modules/secrets.sh (load secrets securely)
- docs/ENVIRONMENT_SETUP.md (setup instructions)

Example .env.example:
```
DATABASE_URL=postgresql://user:password@localhost:5432/midi_library
DATABASE_LOG_LEVEL=info
MEILISEARCH_HOST=http://localhost:7700
MEILISEARCH_API_KEY=your-api-key
RUST_LOG=info,sqlx=debug
NODE_ENV=development
API_PORT=8080
```

#### 8. **Unified Project Configuration (Claude/Kilo/Cline)**
```
Files to Create:
- .claude/rules.md (Claude Code best practices)
- .kilo/config.json (Kilo Code configuration)
- .cline/settings.json (Cline settings)

Standardizes:
- File access permissions
- Code review rules
- Deployment procedures
- Testing requirements
```

### **Low Priority (Polish Phase)**

#### 9. **Documentation System**
```
Files to Create:
- docs/INDEX.md (single source of truth)
- docs/SETUP.md (getting started)
- docs/ARCHITECTURE.md (system design)
- docs/API.md (API reference)
- docs/DEVELOPMENT.md (developer guide)
- docs/DEPLOYMENT.md (production deployment)
- docs/TROUBLESHOOTING.md (common issues)

Action: Consolidate existing 50+ docs into this structure
```

#### 10. **CI/CD Pipeline Setup**
```
Files to Create:
- .github/workflows/test.yml (on every push)
- .github/workflows/build.yml (compile check)
- .github/workflows/deploy.yml (production deployment)

Implements:
✓ Automated testing
✓ Build verification
✓ Deploy gates
✓ Rollback capability
```

---

## 🔄 IMPLEMENTATION STRATEGY

### **PHASE 0: PREPARATION** (Week 1)
- [ ] Audit all existing scripts (functionality, dependencies)
- [ ] Extract project-specific variables
- [ ] Create backup of current working state
- [ ] Document current deployment process
- [ ] List all secrets/credentials needed

### **PHASE 1: FOUNDATION** (Week 1-2)
- [ ] Create config/ directory structure
- [ ] Implement config/defaults.conf
- [ ] Create scripts/modules/ directory
- [ ] Implement log.sh module
- [ ] Implement validation.sh module
- [ ] Implement error-handler.sh module
- [ ] Test modules in isolation

### **PHASE 2: BUILD SYSTEM** (Week 2-3)
- [ ] Implement scripts/modules/build.sh
- [ ] Enhance Makefile with new targets
- [ ] Create task-o-matic.sh dispatcher
- [ ] Implement build/*.task files
- [ ] Test build process end-to-end

### **PHASE 3: DATABASE OPERATIONS** (Week 3)
- [ ] Implement scripts/modules/database.sh
- [ ] Create db migration tasks
- [ ] Implement backup-database.sh
- [ ] Test migrations with rollback
- [ ] Document migration procedures

### **PHASE 4: DEPLOYMENT** (Week 4)
- [ ] Implement deploy-pipeline.sh
- [ ] Implement deploy-daw.sh
- [ ] Implement monitor-services.sh
- [ ] Create health check system
- [ ] Document deployment procedures

### **PHASE 5: POLISH** (Week 4+)
- [ ] Consolidate documentation
- [ ] Implement CI/CD workflows
- [ ] Security audit
- [ ] Performance optimization
- [ ] Final testing & sign-off

---

## 🎯 PROJECT-SPECIFIC CUSTOMIZATIONS

### **Database Configuration**
Your current setup uses:
```
- PostgreSQL 16 (with pgvector)
- Meilisearch 1.x
- MIDI file metadata storage
- 3M+ file support planned
```

Recommendations:
```bash
# In config/defaults.conf, add:
DATABASE_POOL_SIZE=20
DATABASE_TIMEOUT=30
MEILISEARCH_INDEX_SIZE=5000
MEILISEARCH_BATCH_SIZE=1000
BACKUP_RETENTION_DAYS=30
```

### **Rust Compilation Optimization**
Your Cargo.toml includes expensive dependencies (Tauri, tokio, sqlx).

Recommendations:
```toml
# Add to Cargo.toml [profile.dev]
opt-level = 2              # Faster dev builds
incremental = true         # Enable incremental compilation

# Add to Cargo.toml [profile.release]
lto = true                 # Better optimization
codegen-units = 1          # Slower but better optimization
```

### **Frontend Bundle Optimization**
Pipeline & DAW both use Svelte + Vite.

Recommendations:
```javascript
// In vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor': ['svelte', 'svelte/animate'],
          'api': ['./src/api'],
        }
      }
    }
  }
})
```

---

## 📋 INVENTORY OF REUSABLE SCRIPTS & CONFIGS

### **Category: Database**

| Script | Current Location | Status | Action |
|--------|-----------------|--------|--------|
| docker-compose.yml | database/ | ✅ Complete | Enhance with .dev & .prod versions |
| 001_initial_schema.sql | database/migrations/ | ✅ Complete | Audit schema, finalize |
| Database query helpers | database/scripts/ | ⚠️ Partial | Refactor into database.sh module |

### **Category: Build & Compilation**

| Script | Current Location | Status | Action |
|--------|-----------------|--------|--------|
| Cargo.toml (root) | root | ✅ Complete | Add profile optimizations |
| Makefile | root | ⚠️ Basic | Expand with all lifecycle targets |
| build-rust scripts | daw/src-tauri/ | ⚠️ Partial | Consolidate into build.sh |
| vite.config.ts (×2) | pipeline/ & daw/ | ⚠️ Duplicate | Create base config, use extends |

### **Category: Development**

| Script | Current Location | Status | Action |
|--------|-----------------|--------|--------|
| launch-daw.sh | root | ✅ Functional | Move to scripts/launch/ |
| launch-pipeline.sh | root | ✅ Functional | Move to scripts/launch/ |
| verify_integration.sh | pipeline/ | ✅ Functional | Generalize for reuse |

### **Category: Configuration**

| Config | Current Location | Status | Action |
|--------|-----------------|--------|--------|
| .vscode-settings.json | root | ✅ Complete | Convert to .vscode/settings.json |
| .vscode-keybindings.json | root | ✅ Complete | Convert to .vscode/keybindings.json |
| tsconfig.json (×2) | pipeline/ & daw/ | ⚠️ Duplicate | Create base tsconfig with extends |
| package.json (×2) | pipeline/ & daw/ | ⚠️ Duplicate | Extract shared dependencies |

### **Category: Deployment & Operations**

| Script | Current Location | Status | Action |
|--------|-----------------|--------|--------|
| setup-claude.sh | root | ✅ Functional | Move to scripts/deploy/ |
| import_midi_files.py | root | ✅ Functional | Enhance with error recovery |
| fix-all-errors.sh | root | ⚠️ Cleanup | Archive to /legacy |

### **Category: Documentation**

| Doc | Current Location | Count | Action |
|-----|------------------|-------|--------|
| Implementation guides | daw/, pipeline/ | 50+ | Consolidate into docs/INDEX.md |
| API documentation | api/ | 5+ | Create unified API reference |
| Setup guides | root | 7+ | Create single SETUP.md |

---

## 🔍 QUESTIONS & ANSWERS

### **Q: Should I break existing scripts?**
A: No. All existing scripts remain functional. New infrastructure wraps around them.

### **Q: How do I handle existing workflows?**
A: Gradual migration. Old scripts work alongside new ones until fully transitioned.

### **Q: What about configuration conflicts?**
A: New config system with fallback to existing .env files ensures backward compatibility.

### **Q: Do I need to change my Rust code?**
A: No breaking changes needed. Optional refactoring for code quality.

### **Q: What about the extensive documentation?**
A: Consolidate into unified index with cross-references. Existing docs archived but kept.

---

## 📅 NEXT STEPS

1. **Review this gameplan** → Confirm alignment with your goals
2. **Answer clarification questions** → Below section
3. **Start PHASE 0** → Backup and audit
4. **Execute PHASE 1-5** → Phased implementation

---

## ❓ QUESTIONS FOR YOU

Before we proceed, please answer:

1. **Project Scope Confirmation**
   - Are you building for 3M MIDI files from day 1?
   - Or starting smaller and scaling?
   - Timeline expectations?

2. **Deployment Target**
   - Local development only?
   - Ubuntu Studio desktop?
   - Linux server deployment?
   - Docker containerization?

3. **Team & Tools**
   - Solo developer or team?
   - Using Claude Code + Kilo Code + VS Code?
   - Any CI/CD platform preference (GitHub, GitLab, etc.)?

4. **Database Specifics**
   - PostgreSQL version preference?
   - Backup strategy (daily, hourly)?
   - Archive old data?

5. **Performance Requirements**
   - DAW real-time latency target?
   - Pipeline throughput target (files/sec)?
   - Search response time requirement (<50ms)?

6. **Priority Order**
   - Most critical: Database stability? Pipeline performance? DAW reliability?
   - What should we tackle first after Phase 0?

---

## 📌 KEY DECISIONS MADE

✅ **Keep existing code** - Only modernize structure  
✅ **Backward compatible** - All current scripts still work  
✅ **Gradual migration** - Implement new patterns alongside old  
✅ **Configuration first** - Centralize settings before other changes  
✅ **Documentation consolidation** - Single source of truth  
✅ **No database schema changes** - Current schema is solid  
✅ **Rust code unchanged** - Optional refactoring only  

---

## 📚 REFERENCED PATTERNS FROM restructure.txt

- ✅ Task-O-Matic (CLI dispatcher pattern)
- ✅ Grown-Up Scripts (robust operational scripts)
- ✅ Trusty Modules (reusable bash functions)
- ✅ Configuration management (environment-based setup)
- ✅ Docker Compose patterns (multiple environments)
- ✅ Makefile lifecycle targets (build automation)
- ✅ Error handling patterns (trap and recovery)

---

**Status:** Ready for implementation 🚀  
**Complexity:** Medium (organizational restructure, no code rewrites)  
**Risk Level:** Low (backward compatible, phased approach)  
**Timeline:** 4 weeks (phased implementation)

