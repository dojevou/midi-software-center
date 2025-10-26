# 🗂️ QUICK REFERENCE: FILE PLACEMENT GUIDE

**Your New Root:** `~/projects/midi-software-center`

---

## 📋 WHERE EACH FILE GOES

### **ROOT LEVEL FILES** (Keep at top)
```
README.md                 ← Main project overview
SETUP.md                  ← Getting started (5 min setup)
.env.example              ← Environment template
.gitignore                ← Git ignore rules
Makefile                  ← Main automation hub
Cargo.toml                ← Rust workspace
Cargo.lock                ← Dependency lock
package.json              ← Shared deps
pnpm-lock.yaml            ← pnpm lock
rustfmt.toml              ← Rust formatting rules
```

### **NEW ANALYSIS DOCUMENTS** (Top level or docs/)
```
OPTION 1: Keep at root for easy access
├── 00-DOCUMENT-INDEX.md
├── VISUAL_SUMMARY.md
├── ANALYSIS_SUMMARY.md
├── QUICK_REFERENCE.md
├── RESTRUCTURING_GAMEPLAN.md
├── SCRIPT_CONFIG_INVENTORY.md
├── PHASE_0_CHECKLIST.md
└── RECOMMENDED_PROJECT_STRUCTURE.md

OPTION 2: Move to docs/ after reading
docs/
├── INDEX.md
├── ANALYSIS_SUMMARY.md
├── RESTRUCTURING_GAMEPLAN.md
├── PROJECT_STRUCTURE.md
└── [other docs]

RECOMMENDED: Keep at root during Phase 0-2,
then move to docs/ in Phase 4
```

### **CONFIGURATION FILES** (config/)
```
config/
├── defaults.conf         ← Base settings (all envs)
├── development.conf      ← Dev overrides
├── production.conf       ← Prod overrides
├── testing.conf          ← Test overrides
└── load-config.sh        ← Config loader script
```

### **DOCUMENTATION** (docs/)
```
docs/
├── INDEX.md              ← Master documentation index
├── SETUP.md              ← Getting started
├── ARCHITECTURE.md       ← System design
├── DEVELOPMENT.md        ← Developer guide
├── DEPLOYMENT.md         ← Deployment guide
├── TROUBLESHOOTING.md    ← Common issues
│
├── api/                  ← API documentation
│   ├── pipeline-commands.md
│   ├── daw-commands.md
│   └── shared-types.md
│
├── architecture/         ← Architecture docs
│   ├── layers.md
│   ├── components.md
│   └── data-flow.md
│
├── database/             ← Database docs
│   ├── schema.md
│   ├── migrations.md
│   └── queries.md
│
├── guides/               ← How-to guides
│   ├── import-midi.md
│   ├── backup-restore.md
│   └── monitoring.md
│
└── workflows/            ← Common workflows
    ├── first-run.md
    ├── development.md
    └── deployment.md
```

### **DATABASE** (database/)
```
database/
├── docker-compose.yml    ← Keep existing (dev)
├── docker-compose.dev.yml
├── docker-compose.prod.yml
├── docker-compose.test.yml
│
├── migrations/           ← SQL schema changes
│   ├── 001_initial_schema.sql
│   ├── 002_*.sql
│   └── README.md
│
├── queries/              ← Utility queries
│   ├── audit-queries.sql
│   └── README.md
│
├── seeds/                ← Sample data
│   └── dev_sample_data.sql
│
├── scripts/              ← DB helper scripts
│   ├── setup.sh
│   └── backup.sh
│
└── config/               ← DB configuration
    └── meilisearch-index.json
```

### **SCRIPTS** (scripts/)
```
scripts/
├── task-o-matic.sh       ← Main dispatcher (NEW)
├── README.md
│
├── modules/              ← TRUSTY MODULES (reusable)
│   ├── log.sh            ← Logging (NEW)
│   ├── database.sh       ← DB ops (from db_helper.sh + scripts)
│   ├── docker.sh         ← Docker ops (NEW)
│   ├── build.sh          ← Build ops (from rust_build_optimizer.sh)
│   ├── validation.sh     ← Input validation (NEW)
│   ├── error-handler.sh  ← Error handling (NEW)
│   ├── secrets.sh        ← Secrets management (NEW)
│   └── README.md
│
├── tasks/                ← TASK-O-MATIC (dispatcher)
│   ├── db/
│   │   ├── migrate.task
│   │   ├── backup.task
│   │   └── restore.task
│   ├── build/
│   │   ├── compile.task
│   │   ├── pipeline.task
│   │   └── daw.task
│   ├── deploy/
│   │   ├── pipeline.task
│   │   └── daw.task
│   └── dev/
│       ├── watch.task
│       └── dev-all.task
│
├── launch/               ← LAUNCH SCRIPTS (move existing)
│   ├── daw.sh           (from ./launch-daw.sh)
│   ├── pipeline.sh      (from ./launch-pipeline.sh)
│   ├── all.sh           (from ./scripts/launch-all.sh)
│   ├── stop-all.sh
│   ├── status.sh
│   └── README.md
│
├── grown-up/             ← GROWN-UP SCRIPTS (NEW)
│   ├── backup-database.sh
│   ├── restore-database.sh
│   ├── deploy-pipeline.sh
│   ├── deploy-daw.sh
│   ├── monitor-services.sh
│   └── README.md
│
├── maintenance/          ← MAINTENANCE SCRIPTS
│   ├── cleanup.sh
│   ├── duplicate-analyzer.sh
│   ├── diagnostics.sh
│   └── README.md
│
└── legacy/               ← ARCHIVED SCRIPTS
    ├── emergency_fix.sh
    ├── fix-all-errors.sh
    ├── phase0-preparation.sh
    └── README.md
```

### **SHARED CODE** (shared/)
```
shared/
├── rust/                 ← Shared Rust code
│   ├── midi-parser/      ← Consolidate from pipeline + daw
│   ├── database-client/  ← Shared DB operations
│   └── Cargo.toml
│
├── ui/                   ← Shared UI components
│   ├── components/       ← Svelte components
│   ├── stores/           ← Shared state management
│   ├── types/            ← TypeScript types
│   └── lib/              ← Utility functions
│
└── types/                ← Shared TypeScript
    ├── api-types.ts
    ├── domain-types.ts
    └── index.ts
```

### **APPLICATIONS** (pipeline/, daw/)
```
pipeline/
├── src/                  ← Frontend (Svelte)
├── src-tauri/            ← Backend (Rust)
├── tests/                ← Tests
├── docs/                 ← Pipeline-specific docs
├── Cargo.toml
├── package.json
├── vite.config.ts
├── tsconfig.json
└── vitest.config.ts

daw/
├── src/                  ← Frontend (Svelte)
├── src-tauri/            ← Backend (Rust)
├── tests/                ← Tests
├── docs/                 ← DAW-specific docs
├── Cargo.toml
├── package.json
├── vite.config.ts
├── tsconfig.json
└── vitest.config.ts
```

### **INFRASTRUCTURE** (infrastructure/)
```
infrastructure/
├── docker/               ← Docker configs (NEW)
│   ├── pipeline.Dockerfile
│   ├── daw.Dockerfile
│   └── database.Dockerfile
│
├── kubernetes/           ← K8s configs (if applicable)
│   ├── pipeline-deployment.yml
│   └── daw-deployment.yml
│
├── github/               ← GitHub CI/CD (NEW)
│   └── workflows/
│       ├── test.yml
│       ├── build.yml
│       └── deploy.yml
│
└── nginx/                ← Reverse proxy (if needed)
    └── nginx.conf
```

### **TESTS** (tests/)
```
tests/
├── integration/          ← Integration tests
│   ├── pipeline-integration.test.ts
│   └── daw-integration.test.ts
│
├── e2e/                  ← End-to-end tests
│   ├── pipeline-e2e.test.ts
│   └── daw-e2e.test.ts
│
├── fixtures/             ← Test data
│   ├── midi-files/       ← Sample MIDI files
│   ├── sample-data.sql   ← Test database data
│   └── README.md
│
└── README.md
```

### **BACKUPS** (backups/)
```
backups/
├── .gitignore            ← Ignore all files here!
├── README.md             ← Instructions
└── database_backups/     ← Auto-created by backup scripts
    ├── backup_20251023_143022.sql
    ├── backup_20251022_143022.sql
    └── [archive/]        ← Older backups
```

---

## 🎯 FILE MIGRATION CHECKLIST

### **FROM ROOT → NEW LOCATION**
```
□ launch-daw.sh                → scripts/launch/daw.sh
□ launch-pipeline.sh           → scripts/launch/pipeline.sh
□ scripts/launch-all.sh        → scripts/launch/all.sh
□ scripts/stop-all.sh          → scripts/launch/stop-all.sh
□ scripts/status.sh            → scripts/launch/status.sh
□ scripts/install-launcher.sh  → scripts/launch/install-launcher.sh
□ db_helper.sh                 → scripts/modules/database.sh
□ daw/rust_build_optimizer.sh  → scripts/modules/build.sh
□ setup-claude.sh              → scripts/tasks/dev/setup.task
□ complete_setup.sh            → scripts/tasks/dev/complete-setup.task
□ import_midi_files.py         → scripts/tasks/db/import.task (wrapper)
□ import-full-collection.sh    → scripts/tasks/db/import-collection.task
```

### **ARCHIVE TO legacy/ → scripts/legacy/**
```
□ emergency_fix.sh
□ daw/emergency_fix.sh
□ fix-all-errors.sh
□ phase0-preparation.sh
□ extract-error-files.sh
□ SIMPLE-IMPORT-NOW.sh
□ fix_schema.sh
□ [other one-time scripts]
```

### **CONSOLIDATE DOCS**
```
□ 50+ existing .md files  → docs/ (organized by category)
□ Keep best overview docs → docs/INDEX.md
□ Create docs/SETUP.md    ← New start guide
□ Create docs/ARCHITECTURE.md ← New architecture guide
```

### **CREATE NEW CONFIGS**
```
□ .env.example            ← Template
□ .env.local              ← Local (gitignored)
□ config/defaults.conf    ← Base settings
□ config/development.conf ← Dev overrides
□ config/production.conf  ← Prod overrides
□ config/testing.conf     ← Test overrides
```

---

## ⏱️ MIGRATION TIMELINE

### **PHASE 0 (Week 1) - Setup**
```
□ Create folder structure (15 min)
□ Create .env.example
□ Move .vscode files to .vscode/
□ Create docs/ directory
```

### **PHASE 1 (Week 2) - Configuration**
```
□ Create config/ files
□ Create scripts/modules/ files
□ Implement log.sh module
□ Move database scripts
```

### **PHASE 2 (Week 3) - Scripts Organization**
```
□ Move launch scripts to scripts/launch/
□ Move grown-up scripts to scripts/grown-up/
□ Archive legacy scripts
□ Create scripts/README.md
```

### **PHASE 3 (Week 4) - Automation**
```
□ Create task-o-matic.sh
□ Create scripts/tasks/ structure
□ Implement individual tasks
□ Update Makefile
```

### **PHASE 4+ (Week 5+) - Consolidation**
```
□ Consolidate docs to docs/
□ Create infrastructure/
□ Setup GitHub workflows
□ Create tests/ structure
```

---

## 🚀 QUICK COMMANDS TO CREATE STRUCTURE

```bash
# Create all directories at once
bash create-structure.sh

# Or manually:
cd ~/projects/midi-software-center
mkdir -p config docs/{api,architecture,database,guides,workflows}
mkdir -p scripts/{modules,tasks/{db,build,deploy,dev,test},launch,grown-up,maintenance,legacy}
mkdir -p shared/{rust,ui,types}
mkdir -p infrastructure/{docker,kubernetes,github/workflows,nginx}
mkdir -p tests/{integration,e2e,fixtures/midi-files}
mkdir -p backups
```

---

## 📌 KEY PRINCIPLES

✅ **Everything in its place**
- Each file type has a clear home
- No more root clutter

✅ **Logical organization**
- Organized by function, not tools
- Easy to navigate

✅ **Scalable structure**
- Room for growth
- Can add new directories as needed

✅ **Clear separation**
- Configuration separate from code
- Scripts separate from applications
- Tests separate from implementation

✅ **Security-first**
- Backups in safe location
- .env never committed
- Secrets in config/

---

## 🎓 LEARNING PATH

**New developer onboarding:**
1. Read README.md (2 min)
2. Read SETUP.md (5 min)
3. Read docs/ARCHITECTURE.md (10 min)
4. Run `make dev-both` (5 min)
5. Explore the code! (30 min)

**Total:** 1 hour to full productivity

---

**Status:** Ready to implement  
**Complexity:** Low (mostly file moving)  
**Time:** 30-60 minutes for basic setup  
**Value:** Professional project structure

