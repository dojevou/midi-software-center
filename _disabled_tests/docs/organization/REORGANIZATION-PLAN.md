# Project Reorganization Plan

**Date:** 2025-11-13
**Purpose:** Reorganize 75+ loose files into proper directory structure
**Status:** Ready for execution

---

## 📊 Analysis Summary

**Loose Files Found:** 75+ documentation/script files in root directory
**Existing docs/** Structure: Well-organized with 23 subdirectories
**Goal:** Move all loose files to appropriate locations while maintaining organization

---

## 🗂️ Organization Categories

### 1. Documentation Files (*.md, *.txt) → docs/

#### A. Session/Progress Reports → docs/sessions/
- `kilo_code_task_nov-10-2025_7-27-37-pm.md`
- `kilo_code_task_nov-10-2025_10-50-54-pm.md`
- `SESSION-SUMMARY.md`
- `PROJECT-CLEANUP-SUMMARY.md`
- `midi_software_center_analysis.md`

#### B. Implementation/Planning Guides → docs/guides/
- `implementation-guide.md`
- `plan.md`
- `project-completion-plan.md`
- `verification.md`
- `IMPORT-GUIDE.md`
- `LAUNCHER-README.md`

#### C. Status/Final Reports → docs/deployment/
- `STATUS.md`
- `FINAL-GUI-STATUS.md`
- `START-HERE.md`
- `WEBVIEW-DEBUG-GUIDE.md`
- `WEBVIEW-DEBUG-STATUS.md`

#### D. GUI/Frontend Issues → docs/troubleshooting/
- `GUI-CRASH-FIX.md`
- `GUI-LAYOUT-ASCII.md`
- `GUI-LAUNCH-DEBUG-SUMMARY.md`
- `WHITE-SCREEN-*.md` (6 files)
- `WEBVIEW-WHITE-SCREEN-ROOT-CAUSE.md`

#### E. Tailwind/CSS Fixes → docs/troubleshooting/
- `TAILWIND-*.md` (5 files)

#### F. Database Schema Issues → docs/database/
- `SCHEMA-MISMATCH-*.md` (5 files)
- `SCHEMA-VS-CODE-COMPARISON.md`
- `DATABASE-CATEGORIES-COMPLETE.md`
- `ENHANCED-METADATA-PROPOSAL.md`
- `REAL-METADATA-EXAMPLE.md`

#### G. Error Handling & Fixes → docs/errors/
- `CRITICAL-ERROR-HANDLING-ISSUES.md`
- `PIPELINE-ERROR-HANDLING-AUDIT.md`
- `PIPELINE-ERROR-HANDLING-FIX-GUIDE.md`
- `ERROR-HANDLING-AUDIT-SUMMARY.txt`

#### H. SQLX Migration → docs/database/
- `SQLX-*.md` (9 files)

#### I. Testing Reports → docs/testing/
- `PIPELINE-TEST-SUCCESS.md`

#### J. Performance/Optimization → docs/guides/
- `ANALYSIS-OPTIMIZATION-GUIDE.md`
- `CONNECTION-POOL-OPTIMIZATION.md`
- `CPU-ONLY-SYSTEMS.md`
- `POOL-OPTIMIZATION-*.md` (4 files)

#### K. Phase Reports → docs/phase-reports/
- `PHASE-*.md` (9 files in root - should move to existing phase-reports/)
- `PGO-*.md` (5 files - already in docs/)

#### L. Audit/Analysis Reports → docs/code-quality/
- `AUDIT-REPORT-INDEX.md`
- `AUDIT_SUMMARY.md`
- `MULTI_LANGUAGE_AUDIT_REPORT.md`
- `TERMINOLOGY_AUDIT_REPORT.md`

#### M. Miscellaneous Summaries → docs/planning/
- `BENCHMARK-SETUP.txt`
- `ISSUES-QUICK-REFERENCE.txt`
- `MISMATCH-EXECUTIVE-SUMMARY.txt`
- `SCRIPTS-EXECUTION-SUCCESS.md`

### 2. Script Files (*.sh) → scripts/

#### A. Launcher Scripts → scripts/launch/
- `launch-midi-center.sh` (KEEP IN ROOT as main launcher)
- `frontend.sh` → scripts/launch/

#### B. Build/Setup Scripts → scripts/setup/
- None identified (existing scripts already organized)

### 3. Data Files

#### A. Database Backups → backups/ (directory already exists)
- `backup_20251111_074117.sql`

#### B. Configuration → config/ or root
- `project_terminology.json` → config/
- `playwright.config.ts` → root (build config)

#### C. Screenshots → docs/troubleshooting/screenshots/
- `Screenshot_20251110_185135.png`

### 4. Keep in Root (Essential Files)
- `README.md` ✅
- `CLAUDE.md` ✅
- `Cargo.toml` ✅
- `Makefile` ✅
- `docker-compose.yml` ✅
- `package.json` ✅
- `launch-midi-center.sh` ✅ (primary launcher)

---

## 📁 Target Directory Structure

```
docs/
├── sessions/              # Session summaries (5 files)
├── guides/                # Implementation guides (11 files)
├── deployment/            # Status/deployment docs (4 files)
├── troubleshooting/       # GUI/frontend issues (18 files)
│   └── screenshots/       # Screenshots (1 file)
├── database/              # DB schema/SQLX (15 files)
├── errors/                # Error handling (4 files)
├── testing/               # Test reports (1 file)
├── phase-reports/         # Phase summaries (9 files)
├── code-quality/          # Audits (4 files)
└── planning/              # Planning docs (4 files)

scripts/
├── launch/
│   └── frontend.sh        # Move from root

backups/
└── backup_20251111_074117.sql  # Move from root

config/
└── project_terminology.json    # Move from root
```

---

## 🔧 Execution Plan

### Phase 1: Create Missing Directories
```bash
mkdir -p docs/troubleshooting/screenshots
mkdir -p config
```

### Phase 2: Move Documentation Files (by category)
**Session Reports:** 5 files → docs/sessions/
**Implementation Guides:** 11 files → docs/guides/
**Deployment/Status:** 4 files → docs/deployment/
**GUI/Troubleshooting:** 18 files → docs/troubleshooting/
**Database Docs:** 15 files → docs/database/
**Error Handling:** 4 files → docs/errors/
**Testing:** 1 file → docs/testing/
**Phase Reports:** 9 files → docs/phase-reports/
**Code Quality:** 4 files → docs/code-quality/
**Planning:** 4 files → docs/planning/

### Phase 3: Move Non-Documentation Files
- `backup_20251111_074117.sql` → backups/
- `project_terminology.json` → config/
- `Screenshot_20251110_185135.png` → docs/troubleshooting/screenshots/
- `frontend.sh` → scripts/launch/

### Phase 4: Verification
- Check all moved files exist in new locations
- Verify root directory only contains essential files
- Update any broken internal links in documentation

---

## 📋 File Move Checklist

**Total Files to Move:** 75+
**Directories to Create:** 2 new
**Essential Root Files:** 7 (keep)

---

## ⚠️ Notes

1. **launch-midi-center.sh** stays in root as it's the primary launcher
2. **Screenshots directory** created under docs/troubleshooting/
3. **Config directory** created for JSON configs (following best practices)
4. All existing docs/ subdirectories preserved
5. No code files moved (only documentation/configs)
6. Database backup moved to proper backups/ directory

---

## ✅ Success Criteria

1. Root directory contains only 10-15 essential files
2. All documentation properly categorized in docs/
3. No duplicate files
4. All internal documentation links still work
5. Project still builds and runs correctly
