# Project Report: shared-library

> Generated: 2025-11-30 09:07:19
> Path: `/home/dojevou/projects/midi-software-center/shared/rust`

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | **Excellent** - Score ≥ 8/10 or Maintainability ≥ 65 |
| ⚠️ | **Warning** - Score 5-8/10 or Maintainability 40-65 |
| ❌ | **Needs Work** - Score < 5/10 or Maintainability < 40 |
| 🔒 | **Security** - Security-related finding or issue |
| 🐛 | **Bug** - Potential bug or error detected |
| 📁 | **File/Folder** - File system related item |
| 📊 | **Metrics** - Statistical data or analysis |
| 📝 | **Documentation** - Docstring or comment related |
| 🔍 | **Analysis** - Currently being analyzed |
| 📦 | **Package** - Dependency or import related |
| 🚀 | **Performance** - Performance or optimization related |

## Table of Contents

- [Legend](#legend)
- [Summary](#summary)
- [Project Statistics](#project-statistics)
- [Code Quality](#code-quality)
- [Dependencies](#dependencies)
- [File Structure](#file-structure)
- [TODOs and FIXMEs](#todos-and-fixmes)
- [File Details](#file-details)

## Summary

| Metric | Value |
|--------|-------|
| Total Files | 26 |
| Total Lines | 5,049 |
| Lines of Code | 4,382 |
| Functions | 0 |
| Classes | 0 |
| Avg Pylint Score | 0.00/10 |
| Docstring Coverage | 0.0% |

## Project Statistics

### Files by Extension

| Extension | Count | Lines |
|-----------|-------|-------|
| .rs | 25 | 5,003 |
| .toml | 1 | 46 |

## Code Quality

## Dependencies

## File Structure

```
rust/
├── backups/
│   └── project_backup_20251130_090330.zip
├── src/
│   ├── core/
│   │   ├── analysis/
│   │   │   ├── auto_tagger.rs
│   │   │   ├── auto_tagger.rs.backup
│   │   │   ├── bpm_detector.rs
│   │   │   ├── bpm_detector.rs.backup
│   │   │   ├── key_detector.rs
│   │   │   ├── key_detector.rs.backup
│   │   │   ├── key_profiles.rs
│   │   │   └── mod.rs
│   │   ├── midi/
│   │   │   ├── error.rs
│   │   │   ├── error.rs.backup
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs
│   │   │   ├── parser.rs.backup
│   │   │   ├── text_metadata.rs
│   │   │   ├── types.rs
│   │   │   └── types.rs.backup
│   │   └── mod.rs
│   ├── db/
│   │   ├── models/
│   │   │   ├── analysis.rs
│   │   │   ├── error.rs
│   │   │   ├── midi.rs
│   │   │   ├── midi_file.rs
│   │   │   ├── mod.rs
│   │   │   ├── search.rs
│   │   │   └── sequencer.rs
│   │   ├── repositories/
│   │   │   ├── file_repository.rs
│   │   │   ├── metadata_repository.rs
│   │   │   ├── mod.rs
│   │   │   ├── search_repository.rs
│   │   │   └── tag_repository.rs
│   │   └── mod.rs
│   ├── lib.rs
│   └── lib.rs.backup
├── Cargo.toml
├── project_report_20251130_090330.json
└── project_report_20251130_090330.md
```

## TODOs and FIXMEs

*No TODOs or FIXMEs found*

## File Details

### `Cargo.toml` {#cargo-toml}

- **Lines**: 46 (code: 35, comments: 0, blank: 11)

### `src/core/analysis/auto_tagger.rs` {#src-core-analysis-auto-tagger-rs}

- **Lines**: 155 (code: 143, comments: 0, blank: 12)

### `src/core/analysis/bpm_detector.rs` {#src-core-analysis-bpm-detector-rs}

- **Lines**: 1702 (code: 1438, comments: 0, blank: 264)

### `src/core/analysis/key_detector.rs` {#src-core-analysis-key-detector-rs}

- **Lines**: 124 (code: 108, comments: 0, blank: 16)

### `src/core/analysis/key_profiles.rs` {#src-core-analysis-key-profiles-rs}

- **Lines**: 5 (code: 5, comments: 0, blank: 0)

### `src/core/analysis/mod.rs` {#src-core-analysis-mod-rs}

- **Lines**: 16 (code: 15, comments: 0, blank: 1)

### `src/core/midi/error.rs` {#src-core-midi-error-rs}

- **Lines**: 326 (code: 272, comments: 0, blank: 54)

### `src/core/midi/mod.rs` {#src-core-midi-mod-rs}

- **Lines**: 16 (code: 15, comments: 0, blank: 1)

### `src/core/midi/parser.rs` {#src-core-midi-parser-rs}

- **Lines**: 1319 (code: 1161, comments: 0, blank: 158)

### `src/core/midi/text_metadata.rs` {#src-core-midi-text-metadata-rs}

- **Lines**: 325 (code: 286, comments: 0, blank: 39)

### `src/core/midi/types.rs` {#src-core-midi-types-rs}

- **Lines**: 871 (code: 771, comments: 0, blank: 100)

### `src/core/mod.rs` {#src-core-mod-rs}

- **Lines**: 3 (code: 3, comments: 0, blank: 0)

### `src/db/mod.rs` {#src-db-mod-rs}

- **Lines**: 6 (code: 5, comments: 0, blank: 1)

### `src/db/models/analysis.rs` {#src-db-models-analysis-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

### `src/db/models/error.rs` {#src-db-models-error-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

### `src/db/models/midi.rs` {#src-db-models-midi-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

### `src/db/models/midi_file.rs` {#src-db-models-midi-file-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

### `src/db/models/mod.rs` {#src-db-models-mod-rs}

- **Lines**: 15 (code: 14, comments: 0, blank: 1)

### `src/db/models/search.rs` {#src-db-models-search-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

### `src/db/models/sequencer.rs` {#src-db-models-sequencer-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

### `src/db/repositories/file_repository.rs` {#src-db-repositories-file-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

### `src/db/repositories/metadata_repository.rs` {#src-db-repositories-metadata-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

### `src/db/repositories/mod.rs` {#src-db-repositories-mod-rs}

- **Lines**: 11 (code: 10, comments: 0, blank: 1)

### `src/db/repositories/search_repository.rs` {#src-db-repositories-search-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

### `src/db/repositories/tag_repository.rs` {#src-db-repositories-tag-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

### `src/lib.rs` {#src-lib-rs}

- **Lines**: 27 (code: 25, comments: 0, blank: 2)
