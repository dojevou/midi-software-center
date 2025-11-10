#!/bin/bash
set -e

# Create additional subdirectories
mkdir -p docs/{phase-reports,sessions,kilo-code,deployment,errors,analysis,guides/grok4,database,architecture/windows,testing/frontend,code-quality}

# Move PHASE-* files
mv PHASE-*.md docs/phase-reports/ 2>/dev/null || true

# Move KILO-CODE-* files
mv KILO-CODE-*.md docs/kilo-code/ 2>/dev/null || true

# Move TEST-* files
mv TEST-*.md docs/testing/ 2>/dev/null || true

# Move DEPLOYMENT-* files
mv DEPLOYMENT-*.md docs/deployment/ 2>/dev/null || true

# Move ERROR-* and E0308-* files
mv ERROR-*.md E0308-*.md docs/errors/ 2>/dev/null || true

# Move SESSION-* files
mv SESSION-*.md docs/sessions/ 2>/dev/null || true

# Move GROK4-* files
mv GROK4-*.md docs/guides/grok4/ 2>/dev/null || true

# Move FRONTEND-* files
mv FRONTEND-*.md docs/testing/frontend/ 2>/dev/null || true

# Move WINDOW-* files
mv WINDOW-*.md docs/architecture/windows/ 2>/dev/null || true

# Move QUANTUM-* files
mv QUANTUM-*.md docs/analysis/ 2>/dev/null || true

# Move WRAPPER-* files
mv WRAPPER-*.md docs/code-quality/ 2>/dev/null || true

# Move METADATA-* files
mv METADATA-*.md docs/database/ 2>/dev/null || true

# Move ARCHITECTURE-* files
mv ARCHITECTURE-*.md APPSTATE*.md ARC_*.md docs/architecture/ 2>/dev/null || true

# Move DATABASE-* files
mv DATABASE-*.md docs/database/ 2>/dev/null || true

# Move DRUM-* and AUTO_TAGGING-* files
mv DRUM-*.md AUTO_TAGGING*.md CHANGELOG_AUTO_TAGGING*.md docs/guides/ 2>/dev/null || true

# Move ARCHIVE-* files
mv ARCHIVE-*.md docs/workflows/ 2>/dev/null || true

# Move DOCUMENTATION-* files
mv DOCUMENTATION-*.md docs/ 2>/dev/null || true

# Move DEVELOPMENT-*, CRITICAL-*, PROJECT-* files
mv DEVELOPMENT-*.md CRITICAL-*.md PROJECT-*.md docs/guides/ 2>/dev/null || true

# Move COMPLETE-* and COMPREHENSIVE-* files
mv COMPLETE-*.md COMPREHENSIVE-*.md docs/ 2>/dev/null || true

# Move WEEK-*, NEXT-STEPS-*, FINAL-*, EXECUTIVE-* files
mv WEEK-*.md NEXT-STEPS-*.md FINAL-*.md EXECUTIVE-*.md docs/planning/ 2>/dev/null || true

# Move AUDIT-*, UNWRAP-*, SECURITY-* files
mv AUDIT-*.md UNWRAP-*.md SECURITY-*.md docs/code-quality/ 2>/dev/null || true

# Move DAW-*, PRODUCTION-*, REAL-* files  
mv DAW-*.md PRODUCTION-*.md REAL-*.md docs/deployment/ 2>/dev/null || true

# Move analysis and report files
mv analysis_report.md ADD_TAGS_*.md CONTINUATION-*.md docs/sessions/ 2>/dev/null || true

# Move MIDI_TAG_TAXONOMY and similar
mv MIDI_*.md docs/guides/ 2>/dev/null || true

# Move README-* files to guides (except README.md which stays in root)
mv README-*.md docs/guides/ 2>/dev/null || true

# Move GUI-CONSOLIDATION-SUMMARY to docs
mv GUI-CONSOLIDATION-SUMMARY.md docs/ 2>/dev/null || true

# Move TROUBLESHOOTING-* files
mv TROUBLESHOOTING*.md docs/troubleshooting/ 2>/dev/null || true

# Move GUIDE-* files
mv GUIDE-*.md docs/guides/ 2>/dev/null || true

# Move QUICK-* files
mv QUICK-*.md docs/ 2>/dev/null || true

# Move INDEX files
mv INDEX*.md docs/ 2>/dev/null || true

# Move MIGRATION-* files
mv MIGRATION-*.md docs/migration/ 2>/dev/null || true

# Move EXPLORATION-* files
mv EXPLORATION-*.md docs/ 2>/dev/null || true

# Move RELEASE-* files
mv RELEASE-*.md docs/deployment/ 2>/dev/null || true

# Move TYPE_SAFETY files
mv TYPE_SAFETY*.md docs/code-quality/ 2>/dev/null || true

# Move PERFORMANCE-* files
mv PERFORMANCE-*.md docs/workflows/ 2>/dev/null || true

# Move INTEGRATION-* files
mv INTEGRATION-*.md docs/testing/ 2>/dev/null || true

# Move CLIPPY files
mv CLIPPY*.md docs/code-quality/ 2>/dev/null || true

# Move DELIVERY files
mv DELIVERY*.md docs/deployment/ 2>/dev/null || true

echo "✅ Documentation organized!"
