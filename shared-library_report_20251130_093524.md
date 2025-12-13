# Project Report: shared-library

> Generated: 2025-11-30 09:35:25
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
| Total Files | 27 |
| Total Lines | 5,680 |
| Lines of Code | 5,013 |
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
| .json | 1 | 631 |

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

#### Source Code

```toml
[package]
name = "midi-library-shared"
version = "0.1.0"
edition = "2021"
authors = ["MIDI Library System"]
description = "Shared library for MIDI Library System (Pipeline + DAW)"

[lib]
name = "midi_library_shared"
path = "src/lib.rs"

[dependencies]
# MIDI parsing - for Priority 1 (MIDI Core)
midly = "0.5"

# Database - for Priority 2 (Database models & repositories)
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"], optional = true }

# Serialization - for all modules
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling - for all modules
thiserror = "1.0"
anyhow = "1.0"

# Async runtime - for Priority 2 (repositories)
tokio = { version = "1.35", features = ["full"], optional = true }

# Time - for database models
chrono = { version = "0.4", features = ["serde"] }

# UUID - for database models
uuid = { version = "1.6", features = ["v4", "serde"] }

# Logging - for all modules
tracing = "0.1"

[features]
default = []
database = ["sqlx", "tokio"]
full = ["database"]

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"

```

### `project_report_20251130_090330.json` {#project-report-20251130-090330-json}

- **Lines**: 631 (code: 631, comments: 0, blank: 0)

#### Source Code

```json
{
  "timestamp": "20251130_090330",
  "project": {
    "name": "rust",
    "path": "/home/dojevou/projects/midi-software-center/shared/rust",
    "total_files": 26,
    "total_lines": 5049,
    "total_loc": 4382,
    "total_functions": 0,
    "total_classes": 0,
    "avg_pylint_score": 0,
    "avg_complexity": 0,
    "avg_maintainability": 0,
    "docstring_coverage": 0,
    "files_by_extension": {
      ".toml": {
        "count": 1,
        "lines": 46
      },
      ".rs": {
        "count": 25,
        "lines": 5003
      }
    },
    "top_imports": [],
    "security_issues": [],
    "all_todos": []
  },
  "files": [
    {
      "path": "Cargo.toml",
      "name": "Cargo.toml",
      "extension": ".toml",
      "lines": 46,
      "lines_of_code": 35,
      "blank_lines": 11,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/analysis/auto_tagger.rs",
      "name": "auto_tagger.rs",
      "extension": ".rs",
      "lines": 155,
      "lines_of_code": 143,
      "blank_lines": 12,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/analysis/bpm_detector.rs",
      "name": "bpm_detector.rs",
      "extension": ".rs",
      "lines": 1702,
      "lines_of_code": 1438,
      "blank_lines": 264,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/analysis/key_detector.rs",
      "name": "key_detector.rs",
      "extension": ".rs",
      "lines": 124,
      "lines_of_code": 108,
      "blank_lines": 16,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/analysis/key_profiles.rs",
      "name": "key_profiles.rs",
      "extension": ".rs",
      "lines": 5,
      "lines_of_code": 5,
      "blank_lines": 0,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/analysis/mod.rs",
      "name": "mod.rs",
      "extension": ".rs",
      "lines": 16,
      "lines_of_code": 15,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/midi/error.rs",
      "name": "error.rs",
      "extension": ".rs",
      "lines": 326,
      "lines_of_code": 272,
      "blank_lines": 54,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/midi/mod.rs",
      "name": "mod.rs",
      "extension": ".rs",
      "lines": 16,
      "lines_of_code": 15,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/midi/parser.rs",
      "name": "parser.rs",
      "extension": ".rs",
      "lines": 1319,
      "lines_of_code": 1161,
      "blank_lines": 158,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/midi/text_metadata.rs",
      "name": "text_metadata.rs",
      "extension": ".rs",
      "lines": 325,
      "lines_of_code": 286,
      "blank_lines": 39,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/midi/types.rs",
      "name": "types.rs",
      "extension": ".rs",
      "lines": 871,
      "lines_of_code": 771,
      "blank_lines": 100,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/core/mod.rs",
      "name": "mod.rs",
      "extension": ".rs",
      "lines": 3,
      "lines_of_code": 3,
      "blank_lines": 0,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/mod.rs",
      "name": "mod.rs",
      "extension": ".rs",
      "lines": 6,
      "lines_of_code": 5,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/models/analysis.rs",
      "name": "analysis.rs",
      "extension": ".rs",
      "lines": 9,
      "lines_of_code": 8,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/models/error.rs",
      "name": "error.rs",
      "extension": ".rs",
      "lines": 9,
      "lines_of_code": 8,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/models/midi.rs",
      "name": "midi.rs",
      "extension": ".rs",
      "lines": 9,
      "lines_of_code": 8,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/models/midi_file.rs",
      "name": "midi_file.rs",
      "extension": ".rs",
      "lines": 9,
      "lines_of_code": 8,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/models/mod.rs",
      "name": "mod.rs",
      "extension": ".rs",
      "lines": 15,
      "lines_of_code": 14,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/models/search.rs",
      "name": "search.rs",
      "extension": ".rs",
      "lines": 9,
      "lines_of_code": 8,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/models/sequencer.rs",
      "name": "sequencer.rs",
      "extension": ".rs",
      "lines": 9,
      "lines_of_code": 8,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/repositories/file_repository.rs",
      "name": "file_repository.rs",
      "extension": ".rs",
      "lines": 7,
      "lines_of_code": 7,
      "blank_lines": 0,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/repositories/metadata_repository.rs",
      "name": "metadata_repository.rs",
      "extension": ".rs",
      "lines": 7,
      "lines_of_code": 7,
      "blank_lines": 0,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/repositories/mod.rs",
      "name": "mod.rs",
      "extension": ".rs",
      "lines": 11,
      "lines_of_code": 10,
      "blank_lines": 1,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/repositories/search_repository.rs",
      "name": "search_repository.rs",
      "extension": ".rs",
      "lines": 7,
      "lines_of_code": 7,
      "blank_lines": 0,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/db/repositories/tag_repository.rs",
      "name": "tag_repository.rs",
      "extension": ".rs",
      "lines": 7,
      "lines_of_code": 7,
      "blank_lines": 0,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "src/lib.rs",
      "name": "lib.rs",
      "extension": ".rs",
      "lines": 27,
      "lines_of_code": 25,
      "blank_lines": 2,
      "comment_lines": 0,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    }
  ],
  "git": {
    "is_git_repo": true,
    "current_branch": "main",
    "total_commits": 126,
    "contributors": [
      {
        "commits": 126,
        "name": "dojevou"
      }
    ],
    "recent_commits": [
      {
        "hash": "bc06830",
        "message": "docs(CLAUDE.md): comprehensive update with Nov 19-21 feature",
        "when": "9 days ago"
      },
      {
        "hash": "26d60bf",
        "message": "fix(pipeline): resolve database schema mismatch in Phase 4 o",
        "when": "3 weeks ago"
      },
      {
        "hash": "5483fe7",
        "message": "fix(gui): Add comprehensive webview debugging guide",
        "when": "3 weeks ago"
      },
      {
        "hash": "62bee00",
        "message": "debug(gui): Add GUI launch debugging and session summary",
        "when": "3 weeks ago"
      },
      {
        "hash": "9b24207",
        "message": "fix(drum-analyzer): resolve Phase 6 real-world validation te",
        "when": "3 weeks ago"
      },
      {
        "hash": "114ee96",
        "message": "docs(drum-analyzer): Phase 6 session summary - production va",
        "when": "3 weeks ago"
      },
      {
        "hash": "dd9f250",
        "message": "docs(drum-analyzer): Phase 6 real-world validation findings",
        "when": "3 weeks ago"
      },
      {
        "hash": "239f0b0",
        "message": "test(drum-analyzer): add Phase 6 real-world validation test ",
        "when": "3 weeks ago"
      },
      {
        "hash": "fe32245",
        "message": "fix(auto-tagging): Phase 5 integration tests - all 70 tests ",
        "when": "3 weeks ago"
      },
      {
        "hash": "0613ae2",
        "message": "feat(auto-tagging): integrate drum analyzer with auto_tagger",
        "when": "3 weeks ago"
      }
    ],
    "high_churn_files": [
      {
        "file": "derive_injector.py",
        "changes": 1
      },
      {
        "file": "error_parser.py",
        "changes": 1
      },
      {
        "file": "format_string_fixer.py",
        "changes": 1
      },
      {
        "file": "fix_list_files.py",
        "changes": 1
      },
      {
        "file": "fix_e0308_appstate.py",
        "changes": 1
      },
      {
        "file": "fix_e0308_pool.py",
        "changes": 1
      },
      {
        "file": "fix_add_tags_calls.py",
        "changes": 1
      },
      {
        "file": "ultra_supercharged_grok_reviewer.py",
        "changes": 1
      },
      {
        "file": "grok4_project_reviewer.py",
        "changes": 1
      },
      {
        "file": "midi_grok_reviewer.py",
        "changes": 1
      }
    ]
  },
  "backup": "/home/dojevou/projects/midi-software-center/shared/rust/backups/project_backup_20251130_090330.zip"
}
```

### `src/core/analysis/auto_tagger.rs` {#src-core-analysis-auto-tagger-rs}

- **Lines**: 155 (code: 143, comments: 0, blank: 12)

#### Source Code

```rust
/// Auto-tagging functionality wrapper
///
/// This is a simplified wrapper for the shared library.
/// The full implementation lives in the Pipeline component at
/// `pipeline/src-tauri/src/core/analysis/auto_tagger.rs`
///
/// Generates tags based on MIDI file content analysis including:
/// - Instrument detection from GM program changes
/// - Note density and patterns
/// - Tempo characteristics (if BPM detected)
/// - Channel usage patterns
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file structure
///
/// # Returns
/// * `Vec<String>` - List of detected tags (e.g., "drums", "piano", "fast", "melodic")
///
/// # Examples
///
/// ```
/// use midi_library_shared::core::midi::MidiFile;
/// use midi_library_shared::core::analysis::auto_tagger::generate_tags;
///
/// // let midi_file = MidiFile::parse(&data)?;
/// // let tags = generate_tags(&midi_file);
/// // println!("Tags: {:?}", tags);
/// ```
pub fn generate_tags(midi_file: &crate::core::midi::MidiFile) -> Vec<String> {
    let mut tags = Vec::new();

    // Track instruments detected from MIDI program changes
    let mut instruments_seen = std::collections::HashSet::new();
    let mut note_count = 0u32;

    // Analyze all tracks
    for (_track_idx, track) in midi_file.tracks.iter().enumerate() {
        for timed_event in &track.events {
            match &timed_event.event {
                crate::core::midi::Event::ProgramChange { channel, program } => {
                    // Check if this is the drum channel (channel 10 in GM)
                    if *channel == 9 {
                        instruments_seen.insert("drums");
                    } else {
                        // Map GM program numbers to instrument names
                        let instrument = map_gm_program_to_instrument(*program);
                        instruments_seen.insert(instrument);
                    }
                }
                crate::core::midi::Event::NoteOn { channel, .. } => {
                    // Channel 10 (9 in 0-indexed) is drums in GM
                    if *channel == 9 {
                        instruments_seen.insert("drums");
                    }
                    note_count = note_count.saturating_add(1);
                }
                crate::core::midi::Event::Text { text_type: _, text } => {
                    // Extract genre hints from track text in any track
                    let text_lower = text.to_lowercase();

                    // Extract genre hints from track text
                    if text_lower.contains("rock") { tags.push("rock".to_string()); }
                    if text_lower.contains("jazz") { tags.push("jazz".to_string()); }
                    if text_lower.contains("classical") { tags.push("classical".to_string()); }
                    if text_lower.contains("electronic") { tags.push("electronic".to_string()); }
                }
                _ => {}
            }
        }
    }

    // Add instrument tags
    for instrument in instruments_seen {
        tags.push(instrument.to_string());
    }

    // Add complexity tags based on note density
    if note_count > 1000 {
        tags.push("dense".to_string());
    } else if note_count > 500 {
        tags.push("moderate".to_string());
    } else if note_count > 0 {
        tags.push("sparse".to_string());
    }

    // Add track count tags
    let track_count = midi_file.tracks.len();
    if track_count > 10 {
        tags.push("multi-track".to_string());
    } else if track_count > 1 {
        tags.push("layered".to_string());
    } else {
        tags.push("single-track".to_string());
    }

    // Add tempo tags if we can determine BPM
    // Note: This is simplified - full BPM detection is in Pipeline
    if let Some(tempo) = detect_average_tempo(midi_file) {
        if tempo > 140.0 {
            tags.push("fast".to_string());
        } else if tempo > 100.0 {
            tags.push("moderate-tempo".to_string());
        } else if tempo > 60.0 {
            tags.push("slow".to_string());
        }
    }

    tags
}

/// Map GM program numbers to instrument names
fn map_gm_program_to_instrument(program: u8) -> &'static str {
    match program {
        0..=7 => "piano",
        8..=15 => "chromatic-percussion",
        16..=23 => "organ",
        24..=31 => "guitar",
        32..=39 => "bass",
        40..=47 => "strings",
        48..=55 => "ensemble",
        56..=63 => "brass",
        64..=71 => "reed",
        72..=79 => "pipe",
        80..=87 => "synth-lead",
        88..=95 => "synth-pad",
        96..=103 => "synth-effects",
        104..=111 => "ethnic",
        112..=119 => "percussive",
        120..=127 => "sound-effects",
        _ => "unknown",
    }
}

/// Detect average tempo from MIDI tempo events
fn detect_average_tempo(midi_file: &crate::core::midi::MidiFile) -> Option<f64> {
    let mut tempo_sum = 0.0;
    let mut tempo_count = 0;

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let crate::core::midi::Event::TempoChange { microseconds_per_quarter } = &timed_event.event {
                // Convert microseconds per quarter note to BPM
                let bpm = 60_000_000.0 / (*microseconds_per_quarter as f64);
                tempo_sum += bpm;
                tempo_count += 1;
            }
        }
    }

    if tempo_count > 0 {
        Some(tempo_sum / tempo_count as f64)
    } else {
        None
    }
}

```

### `src/core/analysis/bpm_detector.rs` {#src-core-analysis-bpm-detector-rs}

- **Lines**: 1702 (code: 1438, comments: 0, blank: 264)

#### Source Code

```rust
/// BPM Detection Module
///
/// This module provides BPM (Beats Per Minute) detection for MIDI files.
/// It analyzes tempo change events and provides confidence scores.
///
/// # Archetype: Trusty Module
/// - Pure functions with no side effects
/// - No I/O operations
/// - Highly testable
/// - Reusable across the application
use crate::core::midi::types::{Event, MidiFile};

/// Default BPM when no tempo events are found
const DEFAULT_BPM: f64 = 120.0;

/// Minimum valid BPM
const MIN_BPM: f64 = 20.0;

/// Maximum valid BPM
const MAX_BPM: f64 = 300.0;

/// Result of BPM detection
#[derive(Debug, Clone, PartialEq)]
pub struct BpmDetectionResult {
    /// Detected BPM (beats per minute)
    pub bpm: f64,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,

    /// Detection method used
    pub method: BpmDetectionMethod,

    /// Additional metadata
    pub metadata: BpmMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BpmDetectionMethod {
    /// Single tempo event found
    SingleTempo,

    /// Multiple tempo events, used weighted average
    WeightedAverage,

    /// No tempo events, used default
    DefaultTempo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BpmMetadata {
    /// All tempo changes in the file
    pub tempo_changes: Vec<TempoChange>,

    /// Whether tempo is constant throughout
    pub is_constant: bool,

    /// Tempo range (min, max) if multiple tempos
    pub tempo_range: Option<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TempoChange {
    pub tick: u32,
    pub bpm: f64,
}

/// Detects BPM from a parsed MIDI file
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file structure
///
/// # Returns
/// * `BpmDetectionResult` - Detection result with confidence and metadata
///
/// # Examples
/// ```ignore
/// use midi_library_shared::core::analysis::bpm_detector::detect_bpm;
/// use midi_library_shared::core::midi::types::MidiFile;
///
/// let result = detect_bpm(&midi_file);
/// println!("Detected BPM: {:.2}", result.bpm);
/// ```
pub fn detect_bpm(midi_file: &MidiFile) -> BpmDetectionResult {
    // Extract all tempo events from all tracks
    let tempo_events = extract_tempo_events(midi_file);

    if tempo_events.is_empty() {
        return BpmDetectionResult {
            bpm: DEFAULT_BPM,
            confidence: 0.3, // Low confidence for default tempo
            method: BpmDetectionMethod::DefaultTempo,
            metadata: BpmMetadata { tempo_changes: vec![], is_constant: true, tempo_range: None },
        };
    }

    // Convert tempo changes to BPM values
    let tempo_changes: Vec<TempoChange> = tempo_events
        .into_iter()
        .map(|(tick, microseconds_per_quarter)| TempoChange {
            tick,
            bpm: microseconds_to_bpm(microseconds_per_quarter),
        })
        .collect();

    // Calculate statistics
    let is_constant = tempo_changes.len() == 1;
    let bpms: Vec<f64> = tempo_changes.iter().map(|tc| tc.bpm).collect();
    let total_ticks = calculate_total_ticks(midi_file);
    let avg_bpm = calculate_weighted_average(&tempo_changes, total_ticks);

    let tempo_range = if tempo_changes.len() > 1 {
        let min = bpms.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = bpms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        Some((min, max))
    } else {
        None
    };

    // Determine confidence based on consistency
    let confidence = calculate_confidence(&tempo_changes);

    let method = if tempo_changes.len() == 1 {
        BpmDetectionMethod::SingleTempo
    } else {
        BpmDetectionMethod::WeightedAverage
    };

    BpmDetectionResult {
        bpm: avg_bpm,
        confidence,
        method,
        metadata: BpmMetadata { tempo_changes, is_constant, tempo_range },
    }
}

/// Extracts tempo events from all tracks in the MIDI file
fn extract_tempo_events(midi_file: &MidiFile) -> Vec<(u32, u32)> {
    let mut tempo_events = Vec::new();

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;

        for timed_event in &track.events {
            current_tick = current_tick.saturating_add(timed_event.delta_ticks);

            if let Event::TempoChange { microseconds_per_quarter } = timed_event.event {
                tempo_events.push((current_tick, microseconds_per_quarter));
            }
        }
    }

    // Sort by tick position
    tempo_events.sort_by_key(|(tick, _)| *tick);
    tempo_events
}

/// Calculates the total number of ticks in the MIDI file
fn calculate_total_ticks(midi_file: &MidiFile) -> u32 {
    let mut max_ticks = 0u32;

    for track in &midi_file.tracks {
        let mut track_ticks = 0u32;
        for timed_event in &track.events {
            track_ticks = track_ticks.saturating_add(timed_event.delta_ticks);
        }
        max_ticks = max_ticks.max(track_ticks);
    }

    max_ticks
}

/// Converts microseconds per quarter note to BPM
fn microseconds_to_bpm(microseconds_per_quarter: u32) -> f64 {
    // Explicitly handle zero to avoid division by zero
    if microseconds_per_quarter == 0 {
        return DEFAULT_BPM;
    }

    let bpm = 60_000_000.0 / microseconds_per_quarter as f64;

    // Clamp to valid range
    bpm.clamp(MIN_BPM, MAX_BPM)
}

/// Calculates weighted average BPM based on duration each tempo is active
fn calculate_weighted_average(tempo_changes: &[TempoChange], total_ticks: u32) -> f64 {
    if tempo_changes.is_empty() {
        return DEFAULT_BPM;
    }

    if tempo_changes.len() == 1 {
        return tempo_changes[0].bpm;
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for (i, tempo_change) in tempo_changes.iter().enumerate() {
        let duration = if i + 1 < tempo_changes.len() {
            tempo_changes[i + 1].tick - tempo_change.tick
        } else {
            total_ticks.saturating_sub(tempo_change.tick)
        };

        let weight = duration as f64;
        weighted_sum += tempo_change.bpm * weight;
        total_weight += weight;
    }

    if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        tempo_changes[0].bpm
    }
}

/// Calculates confidence score based on tempo consistency
fn calculate_confidence(tempo_changes: &[TempoChange]) -> f64 {
    if tempo_changes.is_empty() {
        return 0.3; // Low confidence for default
    }

    if tempo_changes.len() == 1 {
        return 1.0; // High confidence for single tempo
    }

    // Calculate variance in BPM values
    let bpms: Vec<f64> = tempo_changes.iter().map(|tc| tc.bpm).collect();
    let mean = bpms.iter().sum::<f64>() / bpms.len() as f64;
    let variance = bpms.iter().map(|bpm| (bpm - mean).powi(2)).sum::<f64>() / bpms.len() as f64;
    let std_dev = variance.sqrt();

    // Lower variance = higher confidence
    // Scale confidence based on coefficient of variation
    let cv = std_dev / mean;
    (1.0 - cv).clamp(0.5, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::{Header, MidiFile, TimedEvent, Track};

    // ============================================================================
    // Helper Functions for Building Test MIDI Files
    // ============================================================================

    /// Create a MIDI file with specific tempo events
    /// Each tuple is (tick, microseconds_per_quarter)
    /// If total_ticks is Some(n), extends file to n ticks with EndOfTrack
    fn create_test_midi_with_tempos(tempos: Vec<(u32, u32)>, total_ticks: Option<u32>) -> MidiFile {
        let mut events: Vec<TimedEvent> = Vec::new();
        let mut last_tick = 0u32;

        for (tick, microseconds) in tempos {
            let delta = tick - last_tick;
            events.push(TimedEvent {
                delta_ticks: delta,
                event: Event::TempoChange { microseconds_per_quarter: microseconds },
            });
            last_tick = tick;
        }

        // Add EndOfTrack (optionally extending to total_ticks)
        let eot_delta = if let Some(total) = total_ticks {
            total.saturating_sub(last_tick)
        } else {
            0
        };

        events.push(TimedEvent { delta_ticks: eot_delta, event: Event::EndOfTrack });

        MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events }],
        }
    }

    /// Create a MIDI file with no tempo events
    fn create_midi_no_tempo(num_tracks: u16) -> MidiFile {
        let mut tracks = Vec::new();

        for _ in 0..num_tracks {
            tracks.push(Track {
                events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }],
            });
        }

        MidiFile { header: Header { format: 1, num_tracks, ticks_per_quarter_note: 480 }, tracks }
    }

    /// Create a MIDI file with specific length (total ticks)
    fn create_midi_with_length(total_ticks: u32) -> MidiFile {
        MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![TimedEvent { delta_ticks: total_ticks, event: Event::EndOfTrack }],
            }],
        }
    }

    /// Assert BPM is approximately equal (floating point tolerance)
    fn assert_bpm_approx_eq(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() < tolerance,
            "BPM mismatch: expected {}, got {} (tolerance: {})",
            expected,
            actual,
            tolerance
        );
    }

    // ============================================================================
    // Category 1: microseconds_to_bpm() Conversion (12 tests)
    // ============================================================================

    #[test]
    fn test_microseconds_to_bpm() {
        // 120 BPM = 500,000 microseconds per quarter note
        assert_eq!(microseconds_to_bpm(500_000), 120.0);

        // 60 BPM = 1,000,000 microseconds
        assert_eq!(microseconds_to_bpm(1_000_000), 60.0);

        // 140 BPM ≈ 428,571 microseconds
        let bpm = microseconds_to_bpm(428_571);
        assert!((bpm - 140.0).abs() < 0.1);
    }

    #[test]
    fn test_microseconds_common_bpms() {
        // 90 BPM
        assert_bpm_approx_eq(microseconds_to_bpm(666_667), 90.0, 0.1);

        // 180 BPM
        assert_bpm_approx_eq(microseconds_to_bpm(333_333), 180.0, 0.1);

        // 200 BPM
        assert_eq!(microseconds_to_bpm(300_000), 200.0);
    }

    #[test]
    fn test_microseconds_exact_boundaries() {
        // Exact MIN_BPM (20 BPM = 3,000,000 µs)
        assert_eq!(microseconds_to_bpm(3_000_000), 20.0);

        // Exact MAX_BPM (300 BPM = 200,000 µs)
        assert_eq!(microseconds_to_bpm(200_000), 300.0);
    }

    #[test]
    fn test_bpm_clamping() {
        // Test minimum clamping
        let too_slow = microseconds_to_bpm(5_000_000); // Would be 12 BPM
        assert_eq!(too_slow, MIN_BPM);

        // Test maximum clamping
        let too_fast = microseconds_to_bpm(100_000); // Would be 600 BPM
        assert_eq!(too_fast, MAX_BPM);
    }

    #[test]
    fn test_microseconds_extreme_slow() {
        // 6 BPM (10,000,000 µs) → clamps to 20
        assert_eq!(microseconds_to_bpm(10_000_000), 20.0);

        // Near u32::MAX
        assert_eq!(microseconds_to_bpm(4_000_000_000), 20.0);
    }

    #[test]
    fn test_microseconds_extreme_fast() {
        // 1200 BPM (50,000 µs) → clamps to 300
        assert_eq!(microseconds_to_bpm(50_000), 300.0);

        // 6000 BPM (10,000 µs) → clamps to 300
        assert_eq!(microseconds_to_bpm(10_000), 300.0);

        // 60000 BPM (1,000 µs) → clamps to 300
        assert_eq!(microseconds_to_bpm(1_000), 300.0);
    }

    #[test]
    fn test_microseconds_precision() {
        // Test floating point precision for common tempos
        let bpm_128 = microseconds_to_bpm(468_750); // 128 BPM
        assert_bpm_approx_eq(bpm_128, 128.0, 0.01);

        let bpm_174 = microseconds_to_bpm(344_828); // 174 BPM (dnb tempo)
        assert_bpm_approx_eq(bpm_174, 174.0, 0.1);
    }

    // ============================================================================
    // Category 2: BPM Clamping Edge Cases (2 additional tests)
    // ============================================================================

    #[test]
    fn test_clamping_just_outside_bounds() {
        // Just below minimum: 19.99 BPM → clamps to 20
        assert_eq!(microseconds_to_bpm(3_001_501), 20.0);

        // Just above maximum: 300.1 BPM → clamps to 300
        assert_eq!(microseconds_to_bpm(199_900), 300.0);
    }

    #[test]
    fn test_clamping_within_bounds() {
        // Just above minimum: 20.1 BPM → no clamping
        let bpm = microseconds_to_bpm(2_985_075);
        assert!(bpm > 20.0 && bpm < 21.0);

        // Just below maximum: 299 BPM → no clamping
        let bpm = microseconds_to_bpm(200_669);
        assert!(bpm > 298.0 && bpm < 300.0);
    }

    // ============================================================================
    // Category 3: extract_tempo_events() - Empty Inputs (4 tests)
    // ============================================================================

    #[test]
    fn test_extract_tempo_events_empty_midi() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 0, ticks_per_quarter_note: 480 },
            tracks: vec![],
        };

        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_extract_tempo_events_empty_tracks() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }, Track { events: vec![] }],
        };

        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_extract_tempo_events_no_tempo_events() {
        let midi = create_midi_no_tempo(1);
        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_extract_tempo_events_non_tempo_only() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    // ============================================================================
    // Category 4: extract_tempo_events() - Single Track (6 tests)
    // ============================================================================

    #[test]
    fn test_extract_tempo_single_event_at_zero() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (0, 500_000));
    }

    #[test]
    fn test_extract_tempo_single_event_at_tick() {
        let midi = create_test_midi_with_tempos(vec![(1000, 428_571)], None);
        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (1000, 428_571));
    }

    #[test]
    fn test_extract_tempo_multiple_events_single_track() {
        let midi = create_test_midi_with_tempos(
            vec![(0, 500_000), (1000, 428_571), (2000, 333_333)],
            None,
        );

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1000, 428_571));
        assert_eq!(events[2], (2000, 333_333));
    }

    #[test]
    fn test_extract_tempo_mixed_events() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (960, 428_571)); // 480 + 480
    }

    #[test]
    fn test_extract_tempo_large_delta_ticks() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000), (1_000_000, 428_571)], None);

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1_000_000, 428_571));
    }

    #[test]
    fn test_extract_tempo_consecutive_events() {
        // Multiple tempo changes with no gap
        let midi =
            create_test_midi_with_tempos(vec![(0, 500_000), (0, 428_571), (0, 333_333)], None);

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (0, 428_571));
        assert_eq!(events[2], (0, 333_333));
    }

    // ============================================================================
    // Category 5: extract_tempo_events() - Multiple Tracks (5 tests)
    // ============================================================================

    #[test]
    fn test_extract_tempo_multiple_tracks_one_each() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        // Should be sorted by tick
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1000, 428_571));
    }

    #[test]
    fn test_extract_tempo_sorting_required() {
        // Tempo events from different tracks requiring sorting
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 2000,
                            event: Event::TempoChange { microseconds_per_quarter: 333_333 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 3);
        // Verify sorted order
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1000, 428_571));
        assert_eq!(events[2], (2000, 333_333));
    }

    #[test]
    fn test_extract_tempo_same_tick_different_tracks() {
        // Tempo events at same tick from different tracks
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, 1000);
        assert_eq!(events[1].0, 1000);
    }

    #[test]
    fn test_extract_tempo_some_tracks_empty() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 0, event: Event::EndOfTrack }] },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track { events: vec![TimedEvent { delta_ticks: 0, event: Event::EndOfTrack }] },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (0, 500_000));
    }

    #[test]
    fn test_extract_tempo_many_tracks() {
        // Test with 10 tracks, each with one tempo event
        let mut tracks = Vec::new();
        for i in 0..10 {
            tracks.push(Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: i * 100,
                        event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            });
        }

        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 10, ticks_per_quarter_note: 480 },
            tracks,
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 10);
        // Verify sorted
        for i in 0..10 {
            assert_eq!(events[i as usize].0, i * 100);
        }
    }

    // ============================================================================
    // Category 6: calculate_total_ticks() (8 tests)
    // ============================================================================

    #[test]
    fn test_total_ticks_empty_midi() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 0, ticks_per_quarter_note: 480 },
            tracks: vec![],
        };

        assert_eq!(calculate_total_ticks(&midi), 0);
    }

    #[test]
    fn test_total_ticks_empty_tracks() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }, Track { events: vec![] }],
        };

        assert_eq!(calculate_total_ticks(&midi), 0);
    }

    #[test]
    fn test_total_ticks_single_track() {
        let midi = create_midi_with_length(1920); // 1 bar at 480 TPPQN
        assert_eq!(calculate_total_ticks(&midi), 1920);
    }

    #[test]
    fn test_total_ticks_multiple_tracks_same_length() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
            ],
        };

        assert_eq!(calculate_total_ticks(&midi), 1000);
    }

    #[test]
    fn test_total_ticks_multiple_tracks_different_lengths() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 500, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 2000, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
            ],
        };

        // Should return max (2000)
        assert_eq!(calculate_total_ticks(&midi), 2000);
    }

    #[test]
    fn test_total_ticks_one_long_track() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 100, event: Event::EndOfTrack }] },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                        },
                        TimedEvent { delta_ticks: 8000, event: Event::EndOfTrack },
                    ],
                },
                Track { events: vec![TimedEvent { delta_ticks: 200, event: Event::EndOfTrack }] },
            ],
        };

        // Long track: 1000 + 1000 + 8000 = 10000
        assert_eq!(calculate_total_ticks(&midi), 10_000);
    }

    #[test]
    fn test_total_ticks_zero_delta() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        assert_eq!(calculate_total_ticks(&midi), 0);
    }

    #[test]
    fn test_total_ticks_large_value() {
        let midi = create_midi_with_length(100_000_000);
        assert_eq!(calculate_total_ticks(&midi), 100_000_000);
    }

    // ============================================================================
    // Category 7: calculate_weighted_average() - Edge Cases (8 tests)
    // ============================================================================

    #[test]
    fn test_weighted_average_empty() {
        let tempo_changes = vec![];
        let total_ticks = 1000;

        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            DEFAULT_BPM
        );
    }

    #[test]
    fn test_weighted_average_single_tempo() {
        let tempo_changes = vec![TempoChange { tick: 0, bpm: 140.0 }];
        let total_ticks = 1000;

        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            140.0
        );
    }

    #[test]
    fn test_weighted_average_single_tempo_mid_file() {
        let tempo_changes = vec![TempoChange { tick: 500, bpm: 150.0 }];
        let total_ticks = 1000;

        // Duration: 1000 - 500 = 500 ticks
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            150.0
        );
    }

    #[test]
    fn test_weighted_average_last_tempo_at_end() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 1000, bpm: 140.0 }];
        let total_ticks = 1000;

        // Second tempo has zero duration (at end)
        // weighted_sum = 120 * 1000 + 140 * 0 = 120000
        // total_weight = 1000
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            120.0
        );
    }

    #[test]
    fn test_weighted_average_all_zero_duration() {
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 0, bpm: 120.0 },
            TempoChange { tick: 0, bpm: 140.0 },
        ];
        let total_ticks = 0;

        // All durations are zero, should return first BPM
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            100.0
        );
    }

    #[test]
    fn test_weighted_average_total_ticks_zero() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 500, bpm: 140.0 }];
        let total_ticks = 0;

        // total_ticks < last tick, saturating_sub gives 0 for last tempo
        // Only first tempo has duration (500 - 0 = 500)
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            120.0
        );
    }

    #[test]
    fn test_weighted_average_total_less_than_last_tick() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 1000, bpm: 140.0 }];
        let total_ticks = 500; // Less than last tempo tick

        // First tempo: 1000 - 0 = 1000
        // Second tempo: 500.saturating_sub(1000) = 0
        // weighted_sum = 100 * 1000 + 140 * 0 = 100000
        // total_weight = 1000
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            100.0
        );
    }

    #[test]
    fn test_weighted_average_saturation_behavior() {
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 120.0 },
            TempoChange { tick: u32::MAX / 2, bpm: 140.0 },
        ];
        let total_ticks = u32::MAX;

        // First: u32::MAX/2 - 0 = u32::MAX/2
        // Second: u32::MAX - u32::MAX/2 ≈ u32::MAX/2
        // Both have roughly equal duration, average ≈ 130
        let result = calculate_weighted_average(&tempo_changes, total_ticks);
        assert_bpm_approx_eq(result, 130.0, 1.0);
    }

    // ============================================================================
    // Category 8: calculate_weighted_average() - Mathematical Correctness (7 tests)
    // ============================================================================

    #[test]
    fn test_weighted_average_equal_duration_50_50() {
        // 50% at 100 BPM, 50% at 120 BPM → average = 110
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 500, bpm: 120.0 }];
        let total_ticks = 1000;

        // (100 * 500 + 120 * 500) / 1000 = (50000 + 60000) / 1000 = 110
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            110.0
        );
    }

    #[test]
    fn test_weighted_average_90_10_split() {
        // 90% at 100 BPM, 10% at 120 BPM
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 900, bpm: 120.0 }];
        let total_ticks = 1000;

        // (100 * 900 + 120 * 100) / 1000 = (90000 + 12000) / 1000 = 102
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            102.0
        );
    }

    #[test]
    fn test_weighted_average_10_90_split() {
        // 10% at 100 BPM, 90% at 120 BPM
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 100, bpm: 120.0 }];
        let total_ticks = 1000;

        // (100 * 100 + 120 * 900) / 1000 = (10000 + 108000) / 1000 = 118
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            118.0
        );
    }

    #[test]
    fn test_weighted_average_three_equal_durations() {
        // 33.3% each at 80, 120, 160 BPM → average = 120
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 80.0 },
            TempoChange { tick: 300, bpm: 120.0 },
            TempoChange { tick: 700, bpm: 160.0 },
        ];
        let total_ticks = 1000;

        // (80 * 300 + 120 * 400 + 160 * 300) / 1000
        // = (24000 + 48000 + 48000) / 1000 = 120
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            120.0
        );
    }

    #[test]
    fn test_weighted_average_three_unequal_50_30_20() {
        // 50% at 100, 30% at 120, 20% at 140
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 500, bpm: 120.0 },
            TempoChange { tick: 800, bpm: 140.0 },
        ];
        let total_ticks = 1000;

        // (100 * 500 + 120 * 300 + 140 * 200) / 1000
        // = (50000 + 36000 + 28000) / 1000 = 114
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            114.0
        );
    }

    #[test]
    fn test_weighted_average_five_exponential_decay() {
        // Durations: 500, 250, 125, 62, 63 (exponentially decreasing)
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 500, bpm: 110.0 },
            TempoChange { tick: 750, bpm: 120.0 },
            TempoChange { tick: 875, bpm: 130.0 },
            TempoChange { tick: 937, bpm: 140.0 },
        ];
        let total_ticks = 1000;

        // (100*500 + 110*250 + 120*125 + 130*62 + 140*63) / 1000
        // = (50000 + 27500 + 15000 + 8060 + 8820) / 1000 = 109.38
        let result = calculate_weighted_average(&tempo_changes, total_ticks);
        assert_bpm_approx_eq(result, 109.38, 0.01);
    }

    #[test]
    fn test_weighted_average_precision_check() {
        // Verify floating point precision is maintained
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 123.456 },
            TempoChange { tick: 333, bpm: 156.789 },
            TempoChange { tick: 667, bpm: 178.901 },
        ];
        let total_ticks = 1000;

        // (123.456*333 + 156.789*334 + 178.901*333) / 1000
        // = (41110.848 + 52367.526 + 59574.033) / 1000 = 153.052407
        let result = calculate_weighted_average(&tempo_changes, total_ticks);
        assert_bpm_approx_eq(result, 153.052, 0.01);
    }

    // ============================================================================
    // Category 9: calculate_confidence() - Edge Cases (5 tests)
    // ============================================================================

    #[test]
    fn test_confidence_empty() {
        assert_eq!(calculate_confidence(&[]), 0.3);
    }

    #[test]
    fn test_confidence_single_tempo() {
        let tempo_changes = vec![TempoChange { tick: 0, bpm: 120.0 }];

        assert_eq!(calculate_confidence(&tempo_changes), 1.0);
    }

    #[test]
    fn test_confidence_two_identical() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 500, bpm: 120.0 }];

        // Variance = 0, cv = 0, confidence = 1.0
        assert_eq!(calculate_confidence(&tempo_changes), 1.0);
    }

    #[test]
    fn test_confidence_many_identical() {
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 140.0 },
            TempoChange { tick: 100, bpm: 140.0 },
            TempoChange { tick: 200, bpm: 140.0 },
            TempoChange { tick: 300, bpm: 140.0 },
            TempoChange { tick: 400, bpm: 140.0 },
        ];

        assert_eq!(calculate_confidence(&tempo_changes), 1.0);
    }

    #[test]
    fn test_confidence_extreme_variance_clamped() {
        // Extreme variance should clamp to minimum confidence (0.5)
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 20.0 }, TempoChange { tick: 500, bpm: 300.0 }];

        // Mean = 160, variance = 19600, std_dev = 140, cv = 0.875
        // confidence = 1.0 - 0.875 = 0.125 → clamped to 0.5
        assert_eq!(calculate_confidence(&tempo_changes), 0.5);
    }

    // ============================================================================
    // Category 10: calculate_confidence() - Variance Testing (7 tests)
    // ============================================================================

    #[test]
    fn test_confidence_very_low_variance() {
        // BPMs differ by only 1
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 500, bpm: 121.0 }];

        // Mean = 120.5, variance = 0.25, std_dev = 0.5, cv ≈ 0.00415
        // confidence ≈ 0.996
        let conf = calculate_confidence(&tempo_changes);
        assert!(conf > 0.99 && conf <= 1.0);
    }

    #[test]
    fn test_confidence_low_variance() {
        // 120, 121, 122 (±1 BPM around mean)
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 120.0 },
            TempoChange { tick: 333, bpm: 121.0 },
            TempoChange { tick: 667, bpm: 122.0 },
        ];

        // Mean = 121, variance ≈ 0.667, std_dev ≈ 0.816, cv ≈ 0.00675
        // confidence ≈ 0.993
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.993, 0.01);
    }

    #[test]
    fn test_confidence_medium_variance() {
        // 100, 140 (±20 from mean of 120)
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 500, bpm: 140.0 }];

        // Mean = 120, variance = 400, std_dev = 20, cv ≈ 0.167
        // confidence ≈ 0.833
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.833, 0.01);
    }

    #[test]
    fn test_confidence_high_variance_at_clamp() {
        // 60, 180 (±60 from mean of 120)
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 60.0 }, TempoChange { tick: 500, bpm: 180.0 }];

        // Mean = 120, variance = 3600, std_dev = 60, cv = 0.5
        // confidence = 1.0 - 0.5 = 0.5 (exactly at clamp boundary)
        assert_eq!(calculate_confidence(&tempo_changes), 0.5);
    }

    #[test]
    fn test_confidence_five_low_variance() {
        // 115, 118, 120, 122, 125 (±5 around mean of 120)
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 115.0 },
            TempoChange { tick: 200, bpm: 118.0 },
            TempoChange { tick: 400, bpm: 120.0 },
            TempoChange { tick: 600, bpm: 122.0 },
            TempoChange { tick: 800, bpm: 125.0 },
        ];

        // Mean = 120, variance = 11.2, std_dev ≈ 3.35, cv ≈ 0.0279
        // confidence ≈ 0.972
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.972, 0.01);
    }

    #[test]
    fn test_confidence_five_high_variance() {
        // Wide range: 80, 100, 120, 140, 160
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 80.0 },
            TempoChange { tick: 200, bpm: 100.0 },
            TempoChange { tick: 400, bpm: 120.0 },
            TempoChange { tick: 600, bpm: 140.0 },
            TempoChange { tick: 800, bpm: 160.0 },
        ];

        // Mean = 120, variance = 800, std_dev ≈ 28.28, cv ≈ 0.236
        // confidence ≈ 0.764
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.764, 0.01);
    }

    #[test]
    fn test_confidence_mathematical_validation() {
        // Hand-calculated test case
        // BPMs: [100, 110, 120]
        // Mean = 110
        // Variance = ((100-110)² + (110-110)² + (120-110)²) / 3 = (100 + 0 + 100) / 3 = 66.67
        // Std dev = √66.67 ≈ 8.165
        // CV = 8.165 / 110 ≈ 0.0742
        // Confidence = 1.0 - 0.0742 ≈ 0.926
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 333, bpm: 110.0 },
            TempoChange { tick: 667, bpm: 120.0 },
        ];

        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.926, 0.01);
    }

    // ============================================================================
    // Category 11: detect_bpm() - DefaultTempo Method (3 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_empty_midi() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 0, ticks_per_quarter_note: 480 },
            tracks: vec![],
        };

        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, DEFAULT_BPM);
        assert_eq!(result.confidence, 0.3);
        assert_eq!(result.method, BpmDetectionMethod::DefaultTempo);
        assert!(result.metadata.tempo_changes.is_empty());
        assert!(result.metadata.is_constant);
        assert_eq!(result.metadata.tempo_range, None);
    }

    #[test]
    fn test_detect_bpm_no_tempo_events() {
        let midi = create_midi_no_tempo(2);
        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.confidence, 0.3);
        assert_eq!(result.method, BpmDetectionMethod::DefaultTempo);
        assert!(result.metadata.tempo_changes.is_empty());
        assert!(result.metadata.is_constant);
        assert_eq!(result.metadata.tempo_range, None);
    }

    #[test]
    fn test_detect_bpm_only_non_tempo_events() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.method, BpmDetectionMethod::DefaultTempo);
        assert_eq!(result.confidence, 0.3);
    }

    // ============================================================================
    // Category 12: detect_bpm() - SingleTempo Method (4 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_single_tempo_at_zero() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.confidence, 1.0);
        assert_eq!(result.method, BpmDetectionMethod::SingleTempo);
        assert_eq!(result.metadata.tempo_changes.len(), 1);
        assert!(result.metadata.is_constant);
        assert_eq!(result.metadata.tempo_range, None);
    }

    #[test]
    fn test_detect_bpm_single_tempo_mid_file() {
        let midi = create_test_midi_with_tempos(vec![(1000, 428_571)], None);
        let result = detect_bpm(&midi);

        assert_bpm_approx_eq(result.bpm, 140.0, 0.1);
        assert_eq!(result.confidence, 1.0);
        assert_eq!(result.method, BpmDetectionMethod::SingleTempo);
        assert!(result.metadata.is_constant);
    }

    #[test]
    fn test_detect_bpm_single_tempo_end_of_file() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 1920, // 1 bar
                        event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.method, BpmDetectionMethod::SingleTempo);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_detect_bpm_single_tempo_multiple_tracks() {
        // Same tempo event in multiple tracks (should dedupe)
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let result = detect_bpm(&midi);

        // Both tracks have same tempo at tick 0, so we get 2 tempo changes
        // but they're identical, so high confidence
        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.metadata.tempo_changes.len(), 2);
        assert_eq!(result.confidence, 1.0); // Identical tempos
    }

    // ============================================================================
    // Category 13: detect_bpm() - WeightedAverage Method (6 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_two_tempos_equal_duration() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000), (500, 428_571)], Some(1000));

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 2);
        assert!(!result.metadata.is_constant);

        // Should be approximately 130 BPM (midpoint of 120 and 140)
        assert_bpm_approx_eq(result.bpm, 130.0, 1.0);

        // Confidence should be good (similar tempos)
        assert!(result.confidence > 0.8);

        // Tempo range should be (120, 140)
        let (min, max) = result.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 120.0, 0.1);
        assert_bpm_approx_eq(max, 140.0, 0.1);
    }

    #[test]
    fn test_detect_bpm_five_tempos() {
        let midi = create_test_midi_with_tempos(
            vec![
                (0, 500_000),   // 120 BPM
                (200, 468_750), // 128 BPM
                (400, 500_000), // 120 BPM
                (600, 468_750), // 128 BPM
                (800, 500_000), // 120 BPM
            ],
            None,
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 5);

        // Should be close to average of 120 and 128 (≈124)
        assert_bpm_approx_eq(result.bpm, 124.0, 1.0);

        // Confidence should be high (low variance)
        assert!(result.confidence > 0.95);
    }

    #[test]
    fn test_detect_bpm_ten_tempos() {
        let tempos: Vec<(u32, u32)> = (0..10)
            .map(|i| (i * 100, 500_000)) // All 120 BPM at different ticks
            .collect();

        let midi = create_test_midi_with_tempos(tempos, None);
        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 10);

        // All identical tempos → average = 120
        assert_eq!(result.bpm, 120.0);

        // All identical → maximum confidence
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_detect_bpm_weighted_average_90_10() {
        // Create MIDI with 90% at 100 BPM, 10% at 120 BPM
        let midi = create_test_midi_with_tempos(vec![(0, 600_000), (900, 500_000)], Some(1000));

        let result = detect_bpm(&midi);

        // weighted = (100 * 900 + 120 * 100) / 1000 = 102
        assert_bpm_approx_eq(result.bpm, 102.0, 0.5);
        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
    }

    #[test]
    fn test_detect_bpm_complex_real_world() {
        // Simulate a real-world MIDI with tempo changes
        // Classical piece with ritardando: 120 → 115 → 110 → 105
        let midi = create_test_midi_with_tempos(
            vec![
                (0, 500_000),    // 120 BPM (40% of file)
                (4000, 521_739), // 115 BPM (30%)
                (7000, 545_455), // 110 BPM (20%)
                (9000, 571_429), // 105 BPM (10%)
            ],
            Some(10000),
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 4);

        // Weighted average: (120*4000 + 115*3000 + 110*2000 + 105*1000) / 10000
        // = (480000 + 345000 + 220000 + 105000) / 10000 = 115
        assert_bpm_approx_eq(result.bpm, 115.0, 0.5);

        // Good confidence (gradual tempo change)
        assert!(result.confidence > 0.9);

        // Tempo range
        let (min, max) = result.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 105.0, 1.0);
        assert_bpm_approx_eq(max, 120.0, 1.0);
    }

    #[test]
    fn test_detect_bpm_high_variance_low_confidence() {
        // Extreme tempo changes: 60 → 180
        let midi = create_test_midi_with_tempos(
            vec![
                (0, 1_000_000), // 60 BPM
                (500, 333_333), // 180 BPM
            ],
            None,
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);

        // Should have low confidence due to high variance
        assert_eq!(result.confidence, 0.5); // Clamped to minimum

        let (min, max) = result.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 60.0, 1.0);
        assert_bpm_approx_eq(max, 180.0, 1.0);
    }

    // ============================================================================
    // Category 14: detect_bpm() - Integration & Metadata (6 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_metadata_tempo_changes_populated() {
        let midi = create_test_midi_with_tempos(
            vec![(0, 500_000), (1000, 428_571), (2000, 333_333)],
            None,
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.metadata.tempo_changes.len(), 3);
        assert_bpm_approx_eq(result.metadata.tempo_changes[0].bpm, 120.0, 0.1);
        assert_bpm_approx_eq(result.metadata.tempo_changes[1].bpm, 140.0, 0.1);
        assert_bpm_approx_eq(result.metadata.tempo_changes[2].bpm, 180.0, 0.1);
        assert_eq!(result.metadata.tempo_changes[0].tick, 0);
        assert_eq!(result.metadata.tempo_changes[1].tick, 1000);
        assert_eq!(result.metadata.tempo_changes[2].tick, 2000);
    }

    #[test]
    fn test_detect_bpm_metadata_is_constant_flag() {
        // Single tempo → is_constant = true
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let result_single = detect_bpm(&midi_single);
        assert!(result_single.metadata.is_constant);

        // Multiple tempos → is_constant = false
        let midi_multiple = create_test_midi_with_tempos(vec![(0, 500_000), (1000, 428_571)], None);
        let result_multiple = detect_bpm(&midi_multiple);
        assert!(!result_multiple.metadata.is_constant);

        // No tempos (default) → is_constant = true
        let midi_none = create_midi_no_tempo(1);
        let result_none = detect_bpm(&midi_none);
        assert!(result_none.metadata.is_constant);
    }

    #[test]
    fn test_detect_bpm_metadata_tempo_range() {
        // Single tempo → tempo_range = None
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let result_single = detect_bpm(&midi_single);
        assert_eq!(result_single.metadata.tempo_range, None);

        // Multiple tempos → tempo_range = Some((min, max))
        let midi_multiple = create_test_midi_with_tempos(
            vec![
                (0, 600_000),    // 100 BPM
                (500, 500_000),  // 120 BPM
                (1000, 428_571), // 140 BPM
            ],
            None,
        );
        let result_multiple = detect_bpm(&midi_multiple);
        let (min, max) = result_multiple.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 100.0, 0.1);
        assert_bpm_approx_eq(max, 140.0, 0.1);
    }

    #[test]
    fn test_detect_bpm_confidence_score_ranges() {
        // Default tempo → confidence = 0.3
        let midi_default = create_midi_no_tempo(1);
        assert_eq!(detect_bpm(&midi_default).confidence, 0.3);

        // Single tempo → confidence = 1.0
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        assert_eq!(detect_bpm(&midi_single).confidence, 1.0);

        // Multiple identical → confidence = 1.0
        let midi_identical =
            create_test_midi_with_tempos(vec![(0, 500_000), (500, 500_000), (1000, 500_000)], None);
        assert_eq!(detect_bpm(&midi_identical).confidence, 1.0);

        // High variance → confidence = 0.5 (clamped)
        let midi_variance = create_test_midi_with_tempos(
            vec![
                (0, 1_000_000), // 60 BPM
                (500, 333_333), // 180 BPM
            ],
            None,
        );
        assert_eq!(detect_bpm(&midi_variance).confidence, 0.5);
    }

    #[test]
    fn test_detect_bpm_method_selection_logic() {
        // No tempos → DefaultTempo
        let midi_default = create_midi_no_tempo(1);
        assert_eq!(
            detect_bpm(&midi_default).method,
            BpmDetectionMethod::DefaultTempo
        );

        // One tempo → SingleTempo
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        assert_eq!(
            detect_bpm(&midi_single).method,
            BpmDetectionMethod::SingleTempo
        );

        // Two or more tempos → WeightedAverage
        let midi_multiple = create_test_midi_with_tempos(vec![(0, 500_000), (1000, 428_571)], None);
        assert_eq!(
            detect_bpm(&midi_multiple).method,
            BpmDetectionMethod::WeightedAverage
        );
    }

    #[test]
    fn test_detect_bpm_full_integration() {
        // Real-world integration test with realistic MIDI structure
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                // Tempo track
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange {
                                microseconds_per_quarter: 500_000, // 120 BPM
                            },
                        },
                        TimedEvent {
                            delta_ticks: 1920, // 1 bar
                            event: Event::TempoChange {
                                microseconds_per_quarter: 468_750, // 128 BPM
                            },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                // Melody track
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 480,
                            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                // Bass track
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 1, note: 48, velocity: 80 },
                        },
                        TimedEvent {
                            delta_ticks: 960,
                            event: Event::NoteOff { channel: 1, note: 48, velocity: 0 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let result = detect_bpm(&midi);

        // Verify all aspects
        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 2);
        assert!(!result.metadata.is_constant);
        assert!(result.metadata.tempo_range.is_some());

        // Weighted average: (120 * 1920 + 128 * 0) / 1920 = 120 (last tempo at end has no duration)
        assert_bpm_approx_eq(result.bpm, 120.0, 0.1);

        // Very similar tempos → high confidence
        assert!(result.confidence > 0.95);
    }
}

```

### `src/core/analysis/key_detector.rs` {#src-core-analysis-key-detector-rs}

- **Lines**: 124 (code: 108, comments: 0, blank: 16)

#### Source Code

```rust
/// Key detection wrapper
///
/// This is a simplified wrapper for the shared library.
/// The full implementation lives in the Pipeline component at
/// `pipeline/src-tauri/src/core/analysis/key_detector.rs`
///
/// Returns the detected key as a string (e.g., "C", "Am", "F#")
/// or None if detection confidence is too low.
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file structure
///
/// # Returns
/// * `Some(String)` - Detected key (e.g., "C major", "A minor")
/// * `None` - No clear key detected (confidence < 0.5)
///
/// # Examples
///
/// ```
/// use midi_library_shared::core::midi::MidiFile;
/// use midi_library_shared::core::analysis::key_detector::detect_key;
///
/// // let midi_file = MidiFile::parse(&data)?;
/// // if let Some(key) = detect_key(&midi_file) {
/// //     println!("Detected key: {}", key);
/// // }
/// ```
pub fn detect_key(midi_file: &crate::core::midi::MidiFile) -> Option<String> {
    // Build pitch class histogram from MIDI events
    let mut pitch_class_counts = [0u32; 12];

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let crate::core::midi::Event::NoteOn { note, .. } = &timed_event.event {
                let pitch_class = (note % 12) as usize;
                pitch_class_counts[pitch_class] = pitch_class_counts[pitch_class].saturating_add(1);
            }
        }
    }

    // Check if we have enough notes for analysis
    let total_notes: u32 = pitch_class_counts.iter().sum();
    if total_notes < 10 {
        return None; // Not enough data for reliable key detection
    }

    // Normalize to probability distribution
    let mut pitch_class_dist = [0.0; 12];
    for (i, &count) in pitch_class_counts.iter().enumerate() {
        pitch_class_dist[i] = count as f64 / total_notes as f64;
    }

    // Krumhansl-Schmuckler major and minor profiles
    const MAJOR_PROFILE: [f64; 12] = [
        6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88
    ];
    const MINOR_PROFILE: [f64; 12] = [
        6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17
    ];

    // Find best correlation with all 24 keys (12 major + 12 minor)
    let mut best_correlation = -1.0;
    let mut best_key = String::new();

    const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

    for root in 0..12 {
        // Test major key
        let major_corr = calculate_correlation(&pitch_class_dist, &MAJOR_PROFILE, root);
        if major_corr > best_correlation {
            best_correlation = major_corr;
            best_key = format!("{} major", NOTE_NAMES[root]);
        }

        // Test minor key
        let minor_corr = calculate_correlation(&pitch_class_dist, &MINOR_PROFILE, root);
        if minor_corr > best_correlation {
            best_correlation = minor_corr;
            best_key = format!("{} minor", NOTE_NAMES[root]);
        }
    }

    // Only return if confidence is reasonable (correlation > 0.5)
    if best_correlation > 0.5 {
        Some(best_key)
    } else {
        None
    }
}

/// Calculate Pearson correlation between pitch class distribution and key profile
fn calculate_correlation(distribution: &[f64; 12], profile: &[f64; 12], rotation: usize) -> f64 {
    // Rotate profile to match the key
    let mut rotated = [0.0; 12];
    for i in 0..12 {
        rotated[i] = profile[(i + rotation) % 12];
    }

    // Calculate means
    let dist_mean: f64 = distribution.iter().sum::<f64>() / 12.0;
    let prof_mean: f64 = rotated.iter().sum::<f64>() / 12.0;

    // Calculate correlation coefficient
    let mut numerator = 0.0;
    let mut dist_sq_sum = 0.0;
    let mut prof_sq_sum = 0.0;

    for i in 0..12 {
        let dist_diff = distribution[i] - dist_mean;
        let prof_diff = rotated[i] - prof_mean;

        numerator += dist_diff * prof_diff;
        dist_sq_sum += dist_diff * dist_diff;
        prof_sq_sum += prof_diff * prof_diff;
    }

    let denominator = (dist_sq_sum * prof_sq_sum).sqrt();

    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

```

### `src/core/analysis/key_profiles.rs` {#src-core-analysis-key-profiles-rs}

- **Lines**: 5 (code: 5, comments: 0, blank: 0)

#### Source Code

```rust
/// Key profile data
///
/// Placeholder - will be populated in Phase 5 with Pipeline version
// Temporary stub to allow compilation
pub struct KeyProfileData;

```

### `src/core/analysis/mod.rs` {#src-core-analysis-mod-rs}

- **Lines**: 16 (code: 15, comments: 0, blank: 1)

#### Source Code

```rust
pub mod auto_tagger;
/// Musical analysis modules
///
/// This module provides:
/// - BPM detection
/// - Key detection
/// - Auto-tagging
/// - Key profile data
pub mod bpm_detector;
pub mod key_detector;
pub mod key_profiles;

// Re-export main functions
pub use auto_tagger::generate_tags;
pub use bpm_detector::detect_bpm;
pub use key_detector::detect_key;

```

### `src/core/midi/error.rs` {#src-core-midi-error-rs}

- **Lines**: 326 (code: 272, comments: 0, blank: 54)

#### Source Code

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MidiParseError {
    #[error("Invalid MIDI header: {0}")]
    InvalidHeader(String),

    #[error("Invalid track data at byte {position}: {reason}")]
    InvalidTrack { position: usize, reason: String },

    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(u16),

    #[error("Invalid event at byte {position}: {reason}")]
    InvalidEvent { position: usize, reason: String },

    #[error("Incomplete data: expected {expected} bytes, got {actual}")]
    IncompleteData { expected: usize, actual: usize },

    #[error("Invalid variable-length quantity at byte {0}")]
    InvalidVarLen(usize),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, MidiParseError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // ============================================================================
    // Error Construction Tests
    // ============================================================================

    #[test]
    fn test_invalid_header_construction() {
        let error = MidiParseError::InvalidHeader("bad magic number".to_string());
        assert!(matches!(error, MidiParseError::InvalidHeader(_)));
    }

    #[test]
    fn test_invalid_track_construction() {
        let error =
            MidiParseError::InvalidTrack { position: 42, reason: "unexpected end".to_string() };
        assert!(matches!(error, MidiParseError::InvalidTrack { .. }));
    }

    #[test]
    fn test_unsupported_format_construction() {
        let error = MidiParseError::UnsupportedFormat(99);
        assert!(matches!(error, MidiParseError::UnsupportedFormat(99)));
    }

    #[test]
    fn test_invalid_event_construction() {
        let error = MidiParseError::InvalidEvent {
            position: 100,
            reason: "invalid status byte".to_string(),
        };
        assert!(matches!(error, MidiParseError::InvalidEvent { .. }));
    }

    #[test]
    fn test_incomplete_data_construction() {
        let error = MidiParseError::IncompleteData { expected: 100, actual: 50 };
        assert!(matches!(error, MidiParseError::IncompleteData { .. }));
    }

    #[test]
    fn test_invalid_var_len_construction() {
        let error = MidiParseError::InvalidVarLen(256);
        assert!(matches!(error, MidiParseError::InvalidVarLen(256)));
    }

    // ============================================================================
    // Display Formatting Tests
    // ============================================================================

    #[test]
    fn test_invalid_header_message_format() {
        let error = MidiParseError::InvalidHeader("bad magic number".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Invalid MIDI header"));
        assert!(msg.contains("bad magic number"));
    }

    #[test]
    fn test_invalid_track_message_includes_position() {
        let error =
            MidiParseError::InvalidTrack { position: 42, reason: "unexpected end".to_string() };
        let msg = error.to_string();
        assert!(msg.contains("42"));
        assert!(msg.contains("unexpected end"));
        assert!(msg.contains("Invalid track data"));
    }

    #[test]
    fn test_unsupported_format_message() {
        let error = MidiParseError::UnsupportedFormat(99);
        let msg = error.to_string();
        assert!(msg.contains("Unsupported MIDI format"));
        assert!(msg.contains("99"));
    }

    #[test]
    fn test_invalid_event_message_includes_position() {
        let error = MidiParseError::InvalidEvent {
            position: 100,
            reason: "invalid status byte".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("invalid status byte"));
    }

    #[test]
    fn test_incomplete_data_shows_expected_vs_actual() {
        let error = MidiParseError::IncompleteData { expected: 100, actual: 50 };
        let msg = error.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
        assert!(msg.contains("Incomplete data"));
    }

    #[test]
    fn test_invalid_var_len_message() {
        let error = MidiParseError::InvalidVarLen(256);
        let msg = error.to_string();
        assert!(msg.contains("256"));
        assert!(msg.contains("Invalid variable-length quantity"));
    }

    #[test]
    fn test_io_error_message() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = MidiParseError::Io(io_error);
        let msg = error.to_string();
        assert!(msg.contains("IO error"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_utf8_error_message() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let error = MidiParseError::Utf8(utf8_error);
        let msg = error.to_string();
        assert!(msg.contains("UTF-8 decode error"));
    }

    // ============================================================================
    // Error Conversion Tests (From trait)
    // ============================================================================

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "test file");
        let midi_error: MidiParseError = io_error.into();

        assert!(matches!(midi_error, MidiParseError::Io(_)));
        assert!(midi_error.to_string().contains("test file"));
    }

    #[test]
    fn test_utf8_error_conversion() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let midi_error: MidiParseError = utf8_error.into();

        assert!(matches!(midi_error, MidiParseError::Utf8(_)));
    }

    // ============================================================================
    // Debug Formatting Tests
    // ============================================================================

    #[test]
    fn test_error_debug_format() {
        let error = MidiParseError::UnsupportedFormat(99);
        let debug = format!("{:?}", error);
        assert!(debug.contains("UnsupportedFormat"));
        assert!(debug.contains("99"));
    }

    #[test]
    fn test_error_debug_includes_variant_name() {
        let error = MidiParseError::InvalidHeader("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("InvalidHeader"));
    }

    #[test]
    fn test_error_debug_includes_data() {
        let error = MidiParseError::IncompleteData { expected: 100, actual: 50 };
        let debug = format!("{:?}", error);
        assert!(debug.contains("100"));
        assert!(debug.contains("50"));
    }

    // ============================================================================
    // Result Type Alias Tests
    // ============================================================================

    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<i32> = Ok(42);
        assert!(matches!(result, Ok(42)));
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<i32> = Err(MidiParseError::InvalidVarLen(0));
        assert!(result.is_err());
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_empty_error_messages() {
        let error = MidiParseError::InvalidHeader(String::new());
        let msg = error.to_string();
        assert!(msg.contains("Invalid MIDI header"));
    }

    #[test]
    fn test_very_long_error_message() {
        let long_msg = "x".repeat(10000);
        let error = MidiParseError::InvalidHeader(long_msg.clone());
        let msg = error.to_string();
        assert!(msg.contains(&long_msg));
        assert_eq!(msg.len(), "Invalid MIDI header: ".len() + 10000);
    }

    #[test]
    fn test_special_characters_in_error() {
        let error = MidiParseError::InvalidHeader("Line 1\nLine 2\tTab".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Line 1\nLine 2\tTab"));
    }

    #[test]
    fn test_unicode_in_error_message() {
        let error = MidiParseError::InvalidHeader("Invalid: 🎹 MIDI file".to_string());
        let msg = error.to_string();
        assert!(msg.contains("🎹"));
    }

    #[test]
    fn test_position_boundaries() {
        let error_min =
            MidiParseError::InvalidTrack { position: 0, reason: "start of file".to_string() };
        let error_max = MidiParseError::InvalidTrack {
            position: usize::MAX,
            reason: "end of file".to_string(),
        };

        assert!(error_min.to_string().contains("0"));
        assert!(error_max.to_string().contains(&usize::MAX.to_string()));
    }

    #[test]
    fn test_all_format_variants() {
        // Test format values 0-2 (valid) and beyond
        for format in [0, 1, 2, 3, 99, u16::MAX] {
            let error = MidiParseError::UnsupportedFormat(format);
            let msg = error.to_string();
            assert!(msg.contains(&format.to_string()));
        }
    }

    #[test]
    fn test_expected_actual_boundaries() {
        let cases = vec![(0, 0), (1, 0), (100, 50), (usize::MAX, 0), (1000, 999)];

        for (expected, actual) in cases {
            let error = MidiParseError::IncompleteData { expected, actual };
            let msg = error.to_string();
            assert!(msg.contains(&expected.to_string()));
            assert!(msg.contains(&actual.to_string()));
        }
    }

    // ============================================================================
    // Security Tests
    // ============================================================================

    #[test]
    fn test_error_message_no_memory_leak() {
        // Create large error messages to ensure no memory leak
        for _ in 0..1000 {
            let error = MidiParseError::InvalidHeader("x".repeat(1000));
            let _ = error.to_string();
        }
        // If we get here, no memory leak (would OOM otherwise)
    }

    #[test]
    fn test_malicious_position_values() {
        // Test extreme position values don't cause issues
        let positions = vec![0, 1, usize::MAX - 1, usize::MAX];

        for pos in positions {
            let error = MidiParseError::InvalidEvent { position: pos, reason: "test".to_string() };
            let msg = error.to_string();
            assert!(msg.contains(&pos.to_string()));
        }
    }

    #[test]
    fn test_error_size_is_reasonable() {
        // Ensure error type doesn't use excessive memory
        use std::mem;
        let size = mem::size_of::<MidiParseError>();

        // thiserror errors should be reasonably sized (< 200 bytes typical)
        assert!(size < 256, "MidiParseError is too large: {} bytes", size);
    }
}

```

### `src/core/midi/mod.rs` {#src-core-midi-mod-rs}

- **Lines**: 16 (code: 15, comments: 0, blank: 1)

#### Source Code

```rust
pub mod error;
/// MIDI file parsing and types
///
/// This module provides:
/// - MIDI file parsing
/// - MIDI data types
/// - Error handling
pub mod parser;
pub mod text_metadata;
pub mod types;

// Re-export commonly used items
pub use error::{MidiParseError, Result};
pub use parser::parse_midi_file;
pub use text_metadata::TextMetadata;
pub use types::{Event, MidiFile, Track};

```

### `src/core/midi/parser.rs` {#src-core-midi-parser-rs}

- **Lines**: 1319 (code: 1161, comments: 0, blank: 158)

#### Source Code

```rust
use super::error::{MidiParseError, Result};
use super::types::*;

/// Parse a MIDI file from raw bytes
///
/// This is the main entry point for MIDI parsing. It accepts raw file bytes
/// and returns a structured MidiFile or an error.
///
/// # Examples
/// ```ignore
/// use midi_library_shared::core::midi::parse_midi_file;
///
/// let data = std::fs::read("song.mid").unwrap();
/// let midi_file = parse_midi_file(&data)?;
/// println!("Format: {}, Tracks: {}", midi_file.header.format, midi_file.header.num_tracks);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_midi_file(data: &[u8]) -> Result<MidiFile> {
    if data.len() < 14 {
        return Err(MidiParseError::IncompleteData { expected: 14, actual: data.len() });
    }

    // Parse header chunk
    let header = parse_header(&data[0..14])?;

    // Parse tracks
    let mut tracks = Vec::with_capacity(header.num_tracks as usize);
    let mut pos = 14;

    for track_num in 0..header.num_tracks {
        let (track, bytes_read) = parse_track(&data[pos..]).map_err(|e| match e {
            MidiParseError::InvalidTrack { position, reason } => MidiParseError::InvalidTrack {
                position: pos + position,
                reason: format!("Track {}: {}", track_num, reason),
            },
            e => e,
        })?;

        tracks.push(track);
        pos += bytes_read;
    }

    Ok(MidiFile { header, tracks })
}

/// Parse MIDI header chunk (MThd)
fn parse_header(data: &[u8]) -> Result<Header> {
    // Check magic number "MThd"
    if &data[0..4] != b"MThd" {
        return Err(MidiParseError::InvalidHeader(format!(
            "Expected 'MThd', got {:?}",
            &data[0..4]
        )));
    }

    // Check header length (must be 6)
    let length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    if length != 6 {
        return Err(MidiParseError::InvalidHeader(format!(
            "Expected header length 6, got {}",
            length
        )));
    }

    let format = u16::from_be_bytes([data[8], data[9]]);
    let num_tracks = u16::from_be_bytes([data[10], data[11]]);
    let ticks_per_quarter_note = u16::from_be_bytes([data[12], data[13]]);

    // Validate format
    if format > 2 {
        return Err(MidiParseError::UnsupportedFormat(format));
    }

    Ok(Header { format, num_tracks, ticks_per_quarter_note })
}

/// Parse a single MIDI track (MTrk)
/// Returns (Track, bytes_consumed)
fn parse_track(data: &[u8]) -> Result<(Track, usize)> {
    if data.len() < 8 {
        return Err(MidiParseError::InvalidTrack {
            position: 0,
            reason: "Track too short".to_string(),
        });
    }

    // Check magic number "MTrk"
    if &data[0..4] != b"MTrk" {
        return Err(MidiParseError::InvalidTrack {
            position: 0,
            reason: format!("Expected 'MTrk', got {:?}", &data[0..4]),
        });
    }

    let track_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;

    if data.len() < 8 + track_length {
        return Err(MidiParseError::InvalidTrack {
            position: 0,
            reason: format!(
                "Track data incomplete: expected {} bytes, got {}",
                track_length,
                data.len() - 8
            ),
        });
    }

    let track_data = &data[8..8 + track_length];
    let events = parse_track_events(track_data)?;

    Ok((Track { events }, 8 + track_length))
}

/// Parse all events within a track
fn parse_track_events(data: &[u8]) -> Result<Vec<TimedEvent>> {
    let mut events = Vec::new();
    let mut pos = 0;
    let mut running_status: Option<u8> = None;

    while pos < data.len() {
        // Parse delta time (variable-length quantity)
        let (delta_ticks, delta_bytes) =
            read_var_len(&data[pos..]).ok_or(MidiParseError::InvalidVarLen(pos))?;
        pos += delta_bytes;

        // Parse event
        let (event, event_bytes, new_running_status) = parse_event(&data[pos..], running_status)
            .map_err(|e| match e {
                MidiParseError::InvalidEvent { position, reason } => {
                    MidiParseError::InvalidEvent { position: pos + position, reason }
                },
                e => e,
            })?;

        pos += event_bytes;
        running_status = new_running_status;

        events.push(TimedEvent { delta_ticks, event });

        // End of track?
        if matches!(
            events.last(),
            Some(TimedEvent { event: Event::EndOfTrack, .. })
        ) {
            break;
        }
    }

    Ok(events)
}

/// Parse a single MIDI event
/// Returns (Event, bytes_consumed, new_running_status)
fn parse_event(data: &[u8], running_status: Option<u8>) -> Result<(Event, usize, Option<u8>)> {
    if data.is_empty() {
        return Err(MidiParseError::InvalidEvent {
            position: 0,
            reason: "No data for event".to_string(),
        });
    }

    let mut status = data[0];
    let mut pos = 1;

    // Handle running status (reuse previous status byte if data byte encountered)
    if status < 0x80 {
        if let Some(rs) = running_status {
            status = rs;
            pos = 0; // Don't consume the byte, it's data
        } else {
            return Err(MidiParseError::InvalidEvent {
                position: 0,
                reason: "Data byte without running status".to_string(),
            });
        }
    }

    let event_type = status & 0xF0;
    let channel = status & 0x0F;

    match event_type {
        0x80 => {
            // Note Off
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::NoteOff { channel, note: data[pos], velocity: data[pos + 1] },
                pos + 2,
                Some(status),
            ))
        },
        0x90 => {
            // Note On
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::NoteOn { channel, note: data[pos], velocity: data[pos + 1] },
                pos + 2,
                Some(status),
            ))
        },
        0xA0 => {
            // Polyphonic Aftertouch
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::Aftertouch { channel, note: data[pos], pressure: data[pos + 1] },
                pos + 2,
                Some(status),
            ))
        },
        0xB0 => {
            // Control Change
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            Ok((
                Event::ControlChange { channel, controller: data[pos], value: data[pos + 1] },
                pos + 2,
                Some(status),
            ))
        },
        0xC0 => {
            // Program Change
            if data.len() < pos + 1 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 1,
                    actual: data.len(),
                });
            }
            Ok((
                Event::ProgramChange { channel, program: data[pos] },
                pos + 1,
                Some(status),
            ))
        },
        0xD0 => {
            // Channel Aftertouch
            if data.len() < pos + 1 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 1,
                    actual: data.len(),
                });
            }
            Ok((
                Event::ChannelAftertouch { channel, pressure: data[pos] },
                pos + 1,
                Some(status),
            ))
        },
        0xE0 => {
            // Pitch Bend
            if data.len() < pos + 2 {
                return Err(MidiParseError::IncompleteData {
                    expected: pos + 2,
                    actual: data.len(),
                });
            }
            let lsb = data[pos] as i16;
            let msb = data[pos + 1] as i16;
            let value = ((msb << 7) | lsb) - 8192; // Center at 0
            Ok((Event::PitchBend { channel, value }, pos + 2, Some(status)))
        },
        0xF0 => {
            // System/Meta events
            parse_meta_or_sysex(&data[pos - 1..])
        },
        _ => Err(MidiParseError::InvalidEvent {
            position: 0,
            reason: format!("Unknown event type: 0x{:02X}", status),
        }),
    }
}

/// Parse meta events and SysEx
fn parse_meta_or_sysex(data: &[u8]) -> Result<(Event, usize, Option<u8>)> {
    let status = data[0];

    match status {
        0xFF => {
            // Meta event
            if data.len() < 2 {
                return Err(MidiParseError::IncompleteData { expected: 2, actual: data.len() });
            }

            let meta_type = data[1];
            let (length, len_bytes) =
                read_var_len(&data[2..]).ok_or(MidiParseError::InvalidVarLen(2))?;

            let data_start = 2 + len_bytes;
            let data_end = data_start + length as usize;

            if data.len() < data_end {
                return Err(MidiParseError::IncompleteData {
                    expected: data_end,
                    actual: data.len(),
                });
            }

            let event_data = &data[data_start..data_end];

            let event = match meta_type {
                0x2F => Event::EndOfTrack,
                0x51 => {
                    if event_data.len() != 3 {
                        return Err(MidiParseError::InvalidEvent {
                            position: 0,
                            reason: "Tempo event must be 3 bytes".to_string(),
                        });
                    }
                    let microseconds_per_quarter =
                        u32::from_be_bytes([0, event_data[0], event_data[1], event_data[2]]);
                    Event::TempoChange { microseconds_per_quarter }
                },
                0x58 => {
                    if event_data.len() != 4 {
                        return Err(MidiParseError::InvalidEvent {
                            position: 0,
                            reason: "Time signature event must be 4 bytes".to_string(),
                        });
                    }
                    Event::TimeSignature {
                        numerator: event_data[0],
                        denominator: event_data[1],
                        clocks_per_click: event_data[2],
                        thirty_seconds_per_quarter: event_data[3],
                    }
                },
                0x59 => {
                    if event_data.len() != 2 {
                        return Err(MidiParseError::InvalidEvent {
                            position: 0,
                            reason: "Key signature event must be 2 bytes".to_string(),
                        });
                    }
                    Event::KeySignature {
                        sharps_flats: event_data[0] as i8,
                        is_minor: event_data[1] != 0,
                    }
                },
                0x01..=0x0F => {
                    // Text events - use lossy conversion to handle non-UTF8 encodings
                    let text = String::from_utf8_lossy(event_data).to_string();
                    let text_type = match meta_type {
                        0x01 => TextType::Text,
                        0x02 => TextType::Copyright,
                        0x03 => TextType::TrackName,
                        0x04 => TextType::InstrumentName,
                        0x05 => TextType::Lyric,
                        0x06 => TextType::Marker,
                        0x07 => TextType::CuePoint,
                        _ => TextType::Text,
                    };
                    Event::Text { text_type, text }
                },
                _ => Event::Unknown { status, data: event_data.to_vec() },
            };

            Ok((event, data_end, None)) // Meta events don't have running status
        },
        0xF0 | 0xF7 => {
            // SysEx
            let (length, len_bytes) =
                read_var_len(&data[1..]).ok_or(MidiParseError::InvalidVarLen(1))?;

            let data_start = 1 + len_bytes;
            let data_end = data_start + length as usize;

            if data.len() < data_end {
                return Err(MidiParseError::IncompleteData {
                    expected: data_end,
                    actual: data.len(),
                });
            }

            Ok((
                Event::SysEx { data: data[data_start..data_end].to_vec() },
                data_end,
                None, // SysEx doesn't have running status
            ))
        },
        _ => Err(MidiParseError::InvalidEvent {
            position: 0,
            reason: format!("Unknown system event: 0x{:02X}", status),
        }),
    }
}

/// Read a MIDI variable-length quantity
/// Returns (value, bytes_consumed) or None if invalid
fn read_var_len(data: &[u8]) -> Option<(u32, usize)> {
    let mut value = 0u32;
    let mut bytes_read = 0;

    for (i, &byte) in data.iter().enumerate() {
        if i >= 4 {
            // Variable length can be at most 4 bytes
            return None;
        }

        value = (value << 7) | (byte & 0x7F) as u32;
        bytes_read += 1;

        // If high bit is clear, we're done
        if byte & 0x80 == 0 {
            return Some((value, bytes_read));
        }
    }

    None // Ran out of data before finding end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_var_len() {
        // Single byte
        assert_eq!(read_var_len(&[0x00]), Some((0, 1)));
        assert_eq!(read_var_len(&[0x7F]), Some((127, 1)));

        // Two bytes
        assert_eq!(read_var_len(&[0x81, 0x00]), Some((128, 2)));
        assert_eq!(read_var_len(&[0xFF, 0x7F]), Some((16383, 2)));

        // Invalid (no terminating byte)
        assert_eq!(read_var_len(&[0x81, 0x82, 0x83, 0x84]), None);
    }

    #[test]
    fn test_parse_header() {
        let data = [
            b'M', b'T', b'h', b'd', // Magic
            0, 0, 0, 6, // Length
            0, 1, // Format 1
            0, 3, // 3 tracks
            0, 96, // 96 ticks per quarter note
        ];

        let header = parse_header(&data).unwrap();
        assert_eq!(header.format, 1);
        assert_eq!(header.num_tracks, 3);
        assert_eq!(header.ticks_per_quarter_note, 96);
    }

    #[test]
    fn test_parse_invalid_header_magic() {
        let data = [
            b'M', b'T', b'h', b'X', // Wrong magic
            0, 0, 0, 6, 0, 1, 0, 3, 0, 96,
        ];

        assert!(parse_header(&data).is_err());
    }

    #[test]
    fn test_parse_note_on() {
        // Delta time: 0, Note On channel 0, note 60, velocity 100
        let data = [0x00, 0x90, 0x3C, 0x64, 0x00, 0xFF, 0x2F, 0x00]; // Add End of Track

        let events = parse_track_events(&data).unwrap();
        assert_eq!(events.len(), 2); // NoteOn + EndOfTrack
        assert_eq!(events[0].delta_ticks, 0);

        match &events[0].event {
            Event::NoteOn { channel, note, velocity } => {
                assert_eq!(*channel, 0);
                assert_eq!(*note, 60);
                assert_eq!(*velocity, 100);
            },
            _ => panic!("Expected NoteOn event"),
        }
    }

    #[test]
    fn test_parse_minimal_file() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, // Format 0
            0, 1, // 1 track
            0, 96, // 96 TPPQN
            // Track
            b'M', b'T', b'r', b'k', 0, 0, 0, 4, 0x00, 0xFF, 0x2F, 0x00, // End of track
        ];

        let midi = parse_midi_file(&data).unwrap();
        assert_eq!(midi.header.format, 0);
        assert_eq!(midi.header.num_tracks, 1);
        assert_eq!(midi.tracks.len(), 1);
    }

    #[test]
    fn test_total_notes() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, // Format 0
            0, 1, // 1 track
            0, 96, // 96 TPPQN
            // Track
            b'M', b'T', b'r', b'k', 0, 0, 0, 12, // Note On
            0x00, 0x90, 0x3C, 0x64, // Note On
            0x00, 0x90, 0x40, 0x64, // End of track
            0x00, 0xFF, 0x2F, 0x00,
        ];

        let midi = parse_midi_file(&data).unwrap();
        assert_eq!(midi.total_notes(), 2);
    }

    #[test]
    fn test_channels_used() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, // Format 0
            0, 1, // 1 track
            0, 96, // 96 TPPQN
            // Track
            b'M', b'T', b'r', b'k', 0, 0, 0, 16, // Note On channel 0
            0x00, 0x90, 0x3C, 0x64, // Note On channel 1
            0x00, 0x91, 0x40, 0x64, // Note On channel 9 (drums)
            0x00, 0x99, 0x24, 0x64, // End of track
            0x00, 0xFF, 0x2F, 0x00,
        ];

        let midi = parse_midi_file(&data).unwrap();
        let channels = midi.channels_used();
        assert_eq!(channels, vec![0, 1, 9]);
    }

    // ============================================================================
    // Helper Functions for Building MIDI Test Data
    // ============================================================================

    fn encode_vlq(mut value: u32) -> Vec<u8> {
        if value == 0 {
            return vec![0];
        }

        let mut bytes = Vec::new();
        let mut buffer = value & 0x7F;
        value >>= 7;

        while value > 0 {
            buffer <<= 8;
            buffer |= (value & 0x7F) | 0x80;
            value >>= 7;
        }

        loop {
            bytes.push((buffer & 0xFF) as u8);
            if buffer & 0x80 == 0 {
                break;
            }
            buffer >>= 8;
        }

        bytes
    }

    #[allow(dead_code)]
    fn minimal_track_with_eot() -> Vec<u8> {
        vec![
            0x00, 0xFF, 0x2F, 0x00, // Delta=0, EndOfTrack
        ]
    }

    // ============================================================================
    // Variable-Length Quantity Tests (Extended)
    // ============================================================================

    #[test]
    fn test_read_var_len_3_bytes() {
        // 3-byte VLQ: values 0x4000 - 0x1FFFFF
        let data = &[0x81, 0x80, 0x00]; // 0x4000 (16384)
        assert_eq!(read_var_len(data), Some((16384, 3)));

        let data = &[0xFF, 0xFF, 0x7F]; // Maximum 3-byte VLQ
        assert_eq!(read_var_len(data), Some((2097151, 3)));
    }

    #[test]
    fn test_read_var_len_4_bytes() {
        // 4-byte VLQ: values 0x200000 - 0x0FFFFFFF
        let data = &[0x81, 0x80, 0x80, 0x00]; // 0x200000 (2097152)
        assert_eq!(read_var_len(data), Some((2097152, 4)));

        let data = &[0xFF, 0xFF, 0xFF, 0x7F]; // Maximum VLQ (0x0FFFFFFF)
        assert_eq!(read_var_len(data), Some((268435455, 4)));
    }

    #[test]
    fn test_read_var_len_edge_values() {
        assert_eq!(read_var_len(&[0x00]), Some((0, 1)));
        assert_eq!(read_var_len(&[0x7F]), Some((127, 1)));
        assert_eq!(read_var_len(&[0x81, 0x00]), Some((128, 2)));
        assert_eq!(read_var_len(&[0xFF, 0x7F]), Some((16383, 2)));
    }

    #[test]
    fn test_read_var_len_too_long() {
        // More than 4 bytes should fail
        let data = &[0x81, 0x82, 0x83, 0x84, 0x85];
        assert_eq!(read_var_len(data), None);
    }

    // ============================================================================
    // Header Parsing Tests (Extended)
    // ============================================================================

    #[test]
    fn test_parse_header_format_0() {
        let data = [
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, // Format 0
            0, 1, // 1 track
            0, 96,
        ];
        let header = parse_header(&data).unwrap();
        assert_eq!(header.format, 0);
        assert_eq!(header.num_tracks, 1);
    }

    #[test]
    fn test_parse_header_format_2() {
        let data = [
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 2, // Format 2
            0, 5, // 5 patterns
            1, 0xE0, // 480 TPPQN
        ];
        let header = parse_header(&data).unwrap();
        assert_eq!(header.format, 2);
        assert_eq!(header.num_tracks, 5);
        assert_eq!(header.ticks_per_quarter_note, 480);
    }

    #[test]
    fn test_parse_header_wrong_length() {
        let data = [
            b'M', b'T', b'h', b'd', 0, 0, 0, 8, // Wrong length (should be 6)
            0, 1, 0, 3, 0, 96,
        ];
        assert!(parse_header(&data).is_err());
    }

    #[test]
    fn test_parse_header_unsupported_format() {
        let data = [
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 3, // Format 3 (unsupported)
            0, 1, 0, 96,
        ];
        let result = parse_header(&data);
        assert!(matches!(result, Err(MidiParseError::UnsupportedFormat(3))));
    }

    #[test]
    fn test_parse_header_various_tppqn() {
        // Test different ticks-per-quarter-note values
        for tppqn in [96, 192, 384, 480, 960] {
            let data = [
                b'M',
                b'T',
                b'h',
                b'd',
                0,
                0,
                0,
                6,
                0,
                1,
                0,
                1,
                (tppqn >> 8) as u8,
                (tppqn & 0xFF) as u8,
            ];
            let header = parse_header(&data).unwrap();
            assert_eq!(header.ticks_per_quarter_note, tppqn);
        }
    }

    // ============================================================================
    // Channel Event Tests
    // ============================================================================

    #[test]
    fn test_parse_note_off() {
        let data = vec![
            0x00, 0x80, 0x3C, 0x40, // Delta=0, NoteOff channel 0, note 60, velocity 64
            0x00, 0xFF, 0x2F, 0x00, // EndOfTrack
        ];
        let events = parse_track_events(&data).unwrap();
        assert_eq!(events.len(), 2);

        match &events[0].event {
            Event::NoteOff { channel, note, velocity } => {
                assert_eq!(*channel, 0);
                assert_eq!(*note, 60);
                assert_eq!(*velocity, 64);
            },
            _ => panic!("Expected NoteOff event"),
        }
    }

    #[test]
    fn test_parse_aftertouch() {
        let data = vec![
            0x00, 0xA0, 0x3C, 0x50, // Delta=0, Aftertouch channel 0, note 60, pressure 80
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Aftertouch { channel, note, pressure } => {
                assert_eq!(*channel, 0);
                assert_eq!(*note, 60);
                assert_eq!(*pressure, 80);
            },
            _ => panic!("Expected Aftertouch event"),
        }
    }

    #[test]
    fn test_parse_control_change() {
        let data = vec![
            0x00, 0xB0, 0x07, 0x64, // Delta=0, CC channel 0, controller 7 (volume), value 100
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::ControlChange { channel, controller, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*controller, 7);
                assert_eq!(*value, 100);
            },
            _ => panic!("Expected ControlChange event"),
        }
    }

    #[test]
    fn test_parse_program_change() {
        let data = vec![
            0x00, 0xC0, 0x19, // Delta=0, ProgramChange channel 0, program 25
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::ProgramChange { channel, program } => {
                assert_eq!(*channel, 0);
                assert_eq!(*program, 25);
            },
            _ => panic!("Expected ProgramChange event"),
        }
    }

    #[test]
    fn test_parse_channel_aftertouch() {
        let data = vec![
            0x00, 0xD0, 0x40, // Delta=0, ChannelAftertouch channel 0, pressure 64
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::ChannelAftertouch { channel, pressure } => {
                assert_eq!(*channel, 0);
                assert_eq!(*pressure, 64);
            },
            _ => panic!("Expected ChannelAftertouch event"),
        }
    }

    #[test]
    fn test_parse_pitch_bend_center() {
        // Center position: LSB=0, MSB=64 → value=0
        let data = vec![
            0x00, 0xE0, 0x00, 0x40, // Delta=0, PitchBend channel 0, center
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::PitchBend { channel, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*value, 0); // Centered
            },
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_parse_pitch_bend_max_up() {
        // Maximum up: LSB=127, MSB=127 → value=8191
        let data = vec![
            0x00, 0xE0, 0x7F, 0x7F, // Delta=0, PitchBend channel 0, max up
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::PitchBend { channel, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*value, 8191); // Max up
            },
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_parse_pitch_bend_max_down() {
        // Maximum down: LSB=0, MSB=0 → value=-8192
        let data = vec![
            0x00, 0xE0, 0x00, 0x00, // Delta=0, PitchBend channel 0, max down
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::PitchBend { channel, value } => {
                assert_eq!(*channel, 0);
                assert_eq!(*value, -8192); // Max down
            },
            _ => panic!("Expected PitchBend event"),
        }
    }

    #[test]
    fn test_parse_all_16_channels() {
        // Test NoteOn on all 16 MIDI channels (0-15)
        for channel in 0..16 {
            let data = vec![0x00, 0x90 | channel, 0x3C, 0x64, 0x00, 0xFF, 0x2F, 0x00];
            let events = parse_track_events(&data).unwrap();

            match &events[0].event {
                Event::NoteOn { channel: ch, .. } => {
                    assert_eq!(*ch, channel);
                },
                _ => panic!("Expected NoteOn event"),
            }
        }
    }

    // ============================================================================
    // Meta Event Tests
    // ============================================================================

    #[test]
    fn test_parse_end_of_track() {
        let data = vec![0x00, 0xFF, 0x2F, 0x00]; // Delta=0, EndOfTrack
        let events = parse_track_events(&data).unwrap();

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event, Event::EndOfTrack));
    }

    #[test]
    fn test_parse_tempo_change() {
        // 500,000 microseconds/quarter note = 120 BPM
        let data = vec![
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Tempo event
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::TempoChange { microseconds_per_quarter } => {
                assert_eq!(*microseconds_per_quarter, 500000);
            },
            _ => panic!("Expected TempoChange event"),
        }
    }

    #[test]
    fn test_parse_time_signature() {
        // 4/4 time
        let data = vec![
            0x00, 0xFF, 0x58, 0x04, 0x04, // Numerator (4)
            0x02, // Denominator (2^2 = 4)
            0x18, // Clocks per click (24)
            0x08, // 32nds per quarter (8)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::TimeSignature {
                numerator,
                denominator,
                clocks_per_click,
                thirty_seconds_per_quarter,
            } => {
                assert_eq!(*numerator, 4);
                assert_eq!(*denominator, 2); // 2^2 = 4
                assert_eq!(*clocks_per_click, 24);
                assert_eq!(*thirty_seconds_per_quarter, 8);
            },
            _ => panic!("Expected TimeSignature event"),
        }
    }

    #[test]
    fn test_parse_key_signature_sharps() {
        // D major (2 sharps)
        let data = vec![
            0x00, 0xFF, 0x59, 0x02, 0x02, // 2 sharps
            0x00, // Major
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::KeySignature { sharps_flats, is_minor } => {
                assert_eq!(*sharps_flats, 2);
                assert!(!(*is_minor));
            },
            _ => panic!("Expected KeySignature event"),
        }
    }

    #[test]
    fn test_parse_key_signature_flats() {
        // B-flat major (2 flats), represented as -2
        let data = vec![
            0x00, 0xFF, 0x59, 0x02, 0xFE, // -2 (2 flats) as two's complement
            0x00, // Major
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::KeySignature { sharps_flats, is_minor } => {
                assert_eq!(*sharps_flats, -2);
                assert!(!(*is_minor));
            },
            _ => panic!("Expected KeySignature event"),
        }
    }

    #[test]
    fn test_parse_text_event_track_name() {
        let text = "Piano";
        let data = vec![0x00, 0xFF, 0x03, text.len() as u8]
            .into_iter()
            .chain(text.bytes())
            .chain([0x00, 0xFF, 0x2F, 0x00].iter().copied())
            .collect::<Vec<u8>>();

        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Text { text_type, text: t } => {
                assert!(matches!(text_type, TextType::TrackName));
                assert_eq!(t, text);
            },
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_parse_text_event_lyric() {
        let lyric = "Hello world";
        let data = vec![0x00, 0xFF, 0x05, lyric.len() as u8]
            .into_iter()
            .chain(lyric.bytes())
            .chain([0x00, 0xFF, 0x2F, 0x00].iter().copied())
            .collect::<Vec<u8>>();

        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Text { text_type, text: t } => {
                assert!(matches!(text_type, TextType::Lyric));
                assert_eq!(t, lyric);
            },
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_parse_text_empty() {
        let data = vec![
            0x00, 0xFF, 0x01, 0x00, // Text event with 0 length
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Text { text, .. } => {
                assert_eq!(text, "");
            },
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_parse_text_invalid_utf8() {
        // Parser uses from_utf8_lossy which replaces invalid UTF-8 with replacement chars
        let data = vec![
            0x00, 0xFF, 0x01, 0x03, 0xFF, 0xFE, 0xFD, // Invalid UTF-8 bytes
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        // from_utf8_lossy never fails - it replaces invalid sequences with U+FFFD
        assert!(result.is_ok());
        let events = result.unwrap();
        // Check that the text contains replacement characters
        match &events[0].event {
            Event::Text { text_type: _, text } => {
                assert!(text.contains('\u{FFFD}'), "Should contain replacement character");
            }
            _ => panic!("Expected Text event"),
        }
    }

    #[test]
    fn test_parse_unknown_meta_event() {
        let data = vec![
            0x00, 0xFF, 0x7E, 0x02, // Unknown meta type 0x7E
            0x12, 0x34, // Some data
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::Unknown { status, data: d } => {
                assert_eq!(*status, 0xFF);
                assert_eq!(d, &vec![0x12, 0x34]);
            },
            _ => panic!("Expected Unknown event"),
        }
    }

    // ============================================================================
    // SysEx Event Tests
    // ============================================================================

    #[test]
    fn test_parse_sysex_f0() {
        let data = vec![
            0x00, 0xF0, 0x05, // Delta=0, SysEx, length=5
            0x43, 0x12, 0x00, 0x01, 0xF7, // SysEx data
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::SysEx { data: d } => {
                assert_eq!(d, &vec![0x43, 0x12, 0x00, 0x01, 0xF7]);
            },
            _ => panic!("Expected SysEx event"),
        }
    }

    #[test]
    fn test_parse_sysex_f7() {
        let data = vec![
            0x00, 0xF7, 0x03, // Delta=0, SysEx escape, length=3
            0x01, 0x02, 0x03, 0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::SysEx { data: d } => {
                assert_eq!(d, &vec![0x01, 0x02, 0x03]);
            },
            _ => panic!("Expected SysEx event"),
        }
    }

    #[test]
    fn test_parse_sysex_empty() {
        let data = vec![
            0x00, 0xF0, 0x00, // Delta=0, SysEx, length=0
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::SysEx { data: d } => {
                assert!(d.is_empty());
            },
            _ => panic!("Expected SysEx event"),
        }
    }

    // ============================================================================
    // Running Status Tests
    // ============================================================================

    #[test]
    fn test_running_status_continuation() {
        let data = vec![
            0x00, 0x90, 0x3C, 0x64, // NoteOn C4 with status byte
            0x00, 0x3E, 0x64, // NoteOn D4 without status (running status)
            0x00, 0x40, 0x64, // NoteOn E4 without status (running status)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        // Should have 3 NoteOn events + EndOfTrack
        assert_eq!(events.len(), 4);
        for event in events.iter().take(3) {
            assert!(matches!(event.event, Event::NoteOn { .. }));
        }
    }

    #[test]
    fn test_running_status_cleared_by_meta() {
        let data = vec![
            0x00, 0x90, 0x3C, 0x64, // NoteOn with status
            0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, // Tempo (clears running status)
            0x00, 0x90, 0x3E, 0x64, // NoteOn - needs status byte again
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();
        assert_eq!(events.len(), 4); // 2 NoteOns + Tempo + EndOfTrack
    }

    #[test]
    fn test_running_status_error_without_prior() {
        let data = vec![
            0x00, 0x3C, 0x64, // Data byte without prior status
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    // ============================================================================
    // Track Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_empty_track() {
        let data = vec![
            b'M', b'T', b'r', b'k', 0, 0, 0, 4, // Length = 4
            0x00, 0xFF, 0x2F, 0x00, // Just EndOfTrack
        ];
        let (track, bytes_consumed) = parse_track(&data).unwrap();
        assert_eq!(track.events.len(), 1);
        assert_eq!(bytes_consumed, 12); // 8 header + 4 data
    }

    #[test]
    fn test_parse_track_invalid_magic() {
        let data = vec![
            b'M', b'T', b'r', b'X', // Wrong magic
            0, 0, 0, 4, 0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidTrack { .. })));
    }

    #[test]
    fn test_parse_track_too_short() {
        let data = vec![b'M', b'T', b'r', b'k', 0, 0]; // Only 6 bytes
        let result = parse_track(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidTrack { .. })));
    }

    #[test]
    fn test_parse_track_incomplete_data() {
        let data = vec![
            b'M', b'T', b'r', b'k', 0, 0, 0, 10, // Claims 10 bytes
            0x00, 0xFF, 0x2F, 0x00, // Only 4 bytes provided
        ];
        let result = parse_track(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidTrack { .. })));
    }

    // ============================================================================
    // File Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_file_too_short() {
        let data = vec![b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0]; // Only 10 bytes
        let result = parse_midi_file(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_parse_format_1_multiple_tracks() {
        let data = vec![
            // Header
            b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 1, // Format 1
            0, 2, // 2 tracks
            0, 96, // Track 1
            b'M', b'T', b'r', b'k', 0, 0, 0, 4, 0x00, 0xFF, 0x2F, 0x00, // Track 2
            b'M', b'T', b'r', b'k', 0, 0, 0, 4, 0x00, 0xFF, 0x2F, 0x00,
        ];

        let midi = parse_midi_file(&data).unwrap();
        assert_eq!(midi.header.format, 1);
        assert_eq!(midi.tracks.len(), 2);
    }

    // ============================================================================
    // Error Condition Tests
    // ============================================================================

    #[test]
    fn test_incomplete_note_on() {
        let data = vec![0x00, 0x90, 0x3C]; // Missing velocity byte
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_incomplete_control_change() {
        let data = vec![0x00, 0xB0, 0x07]; // Missing value byte
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_incomplete_pitch_bend() {
        let data = vec![0x00, 0xE0, 0x00]; // Missing MSB
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::IncompleteData { .. })));
    }

    #[test]
    fn test_invalid_tempo_length() {
        let data = vec![
            0x00, 0xFF, 0x51, 0x02, // Tempo with wrong length (should be 3)
            0x07, 0xA1, 0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    #[test]
    fn test_invalid_time_signature_length() {
        let data = vec![
            0x00, 0xFF, 0x58, 0x03, // TimeSignature with wrong length (should be 4)
            0x04, 0x02, 0x18, 0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    #[test]
    fn test_invalid_key_signature_length() {
        let data = vec![
            0x00, 0xFF, 0x59, 0x01, // KeySignature with wrong length (should be 2)
            0x02, 0x00, 0xFF, 0x2F, 0x00,
        ];
        let result = parse_track_events(&data);
        assert!(matches!(result, Err(MidiParseError::InvalidEvent { .. })));
    }

    // ============================================================================
    // Edge Cases Tests
    // ============================================================================

    #[test]
    fn test_zero_delta_ticks() {
        // Multiple events with delta=0 (simultaneous)
        let data = vec![
            0x00, 0x90, 0x3C, 0x64, // Delta=0, NoteOn
            0x00, 0x90, 0x40, 0x64, // Delta=0, NoteOn (simultaneous)
            0x00, 0x90, 0x43, 0x64, // Delta=0, NoteOn (simultaneous)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        assert_eq!(events.len(), 4);
        assert_eq!(events[0].delta_ticks, 0);
        assert_eq!(events[1].delta_ticks, 0);
        assert_eq!(events[2].delta_ticks, 0);
    }

    #[test]
    fn test_large_delta_ticks() {
        // Large delta time using VLQ
        let large_delta = encode_vlq(100000);
        let data = large_delta
            .into_iter()
            .chain([0x90, 0x3C, 0x64])
            .chain([0x00, 0xFF, 0x2F, 0x00])
            .collect::<Vec<u8>>();

        let events = parse_track_events(&data).unwrap();
        assert_eq!(events[0].delta_ticks, 100000);
    }

    #[test]
    fn test_max_note_velocity_values() {
        let data = vec![
            0x00, 0x90, 0x7F, 0x7F, // Note 127, velocity 127 (maximum values)
            0x00, 0xFF, 0x2F, 0x00,
        ];
        let events = parse_track_events(&data).unwrap();

        match &events[0].event {
            Event::NoteOn { note, velocity, .. } => {
                assert_eq!(*note, 127);
                assert_eq!(*velocity, 127);
            },
            _ => panic!("Expected NoteOn event"),
        }
    }
}

```

### `src/core/midi/text_metadata.rs` {#src-core-midi-text-metadata-rs}

- **Lines**: 325 (code: 286, comments: 0, blank: 39)

#### Source Code

```rust
/// Text Metadata Extractor - Trusty Module
///
/// Extracts text metadata from MIDI files:
/// - Track names
/// - Copyright notices
/// - Lyrics
/// - Markers
/// - Cue points
/// - Instrument names
///
/// This module complements the musical analysis by capturing human-readable
/// text information embedded in MIDI files.
use super::types::{Event, MidiFile, TextType};

/// Text metadata extracted from a MIDI file
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextMetadata {
    /// Track names from each track
    pub track_names: Vec<String>,

    /// Copyright notice (usually in first track)
    pub copyright: Option<String>,

    /// Instrument names assigned to tracks
    pub instrument_names: Vec<String>,

    /// Lyrics embedded in the MIDI file
    pub lyrics: Vec<String>,

    /// Markers (section labels, rehearsal marks)
    pub markers: Vec<String>,

    /// Cue points (timing references)
    pub cue_points: Vec<String>,

    /// General text messages
    pub text_messages: Vec<String>,
}

impl TextMetadata {
    /// Extract all text metadata from a MIDI file
    ///
    /// # Examples
    /// ```
    /// use midi_library_shared::core::midi::parser::parse_midi_file;
    /// use midi_library_shared::core::midi::text_metadata::TextMetadata;
    ///
    /// let data = std::fs::read("song.mid").unwrap();
    /// let midi_file = parse_midi_file(&data).unwrap();
    /// let metadata = TextMetadata::extract(&midi_file);
    ///
    /// println!("Track names: {:?}", metadata.track_names);
    /// println!("Copyright: {:?}", metadata.copyright);
    /// ```
    pub fn extract(midi_file: &MidiFile) -> Self {
        let mut metadata = TextMetadata::default();

        for track in &midi_file.tracks {
            for timed_event in &track.events {
                if let Event::Text { text_type, text } = &timed_event.event {
                    Self::process_text_event(&mut metadata, text_type, text);
                }
            }
        }

        // Deduplicate all vectors
        metadata.track_names.sort();
        metadata.track_names.dedup();
        metadata.instrument_names.sort();
        metadata.instrument_names.dedup();
        metadata.lyrics.dedup();
        metadata.markers.dedup();
        metadata.cue_points.dedup();
        metadata.text_messages.sort();
        metadata.text_messages.dedup();

        metadata
    }

    /// Process a single text event
    fn process_text_event(metadata: &mut TextMetadata, text_type: &TextType, text: &str) {
        // Skip empty strings
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        match text_type {
            TextType::TrackName => {
                if !metadata.track_names.contains(&text.to_string()) {
                    metadata.track_names.push(text.to_string());
                }
            },
            TextType::Copyright => {
                // Keep first copyright notice found
                if metadata.copyright.is_none() {
                    metadata.copyright = Some(text.to_string());
                }
            },
            TextType::InstrumentName => {
                if !metadata.instrument_names.contains(&text.to_string()) {
                    metadata.instrument_names.push(text.to_string());
                }
            },
            TextType::Lyric => {
                metadata.lyrics.push(text.to_string());
            },
            TextType::Marker => {
                if !metadata.markers.contains(&text.to_string()) {
                    metadata.markers.push(text.to_string());
                }
            },
            TextType::CuePoint => {
                if !metadata.cue_points.contains(&text.to_string()) {
                    metadata.cue_points.push(text.to_string());
                }
            },
            TextType::Text => {
                if !metadata.text_messages.contains(&text.to_string()) {
                    metadata.text_messages.push(text.to_string());
                }
            },
        }
    }

    /// Check if any text metadata exists
    pub fn is_empty(&self) -> bool {
        self.track_names.is_empty()
            && self.copyright.is_none()
            && self.instrument_names.is_empty()
            && self.lyrics.is_empty()
            && self.markers.is_empty()
            && self.cue_points.is_empty()
            && self.text_messages.is_empty()
    }

    /// Get a summary of text metadata for display
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.track_names.is_empty() {
            parts.push(format!("{} tracks", self.track_names.len()));
        }

        if let Some(copyright) = &self.copyright {
            parts.push(format!("© {}", copyright));
        }

        if !self.instrument_names.is_empty() {
            parts.push(format!("{} instruments", self.instrument_names.len()));
        }

        if !self.markers.is_empty() {
            parts.push(format!("{} markers", self.markers.len()));
        }

        if !self.lyrics.is_empty() {
            parts.push(format!("{} lyric lines", self.lyrics.len()));
        }

        if parts.is_empty() {
            "No text metadata".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::{Header, MidiFile, TimedEvent, Track};

    fn create_test_midi_with_text() -> MidiFile {
        MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::TrackName,
                                text: "Piano Track".to_string(),
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::Copyright,
                                text: "2025 Test Artist".to_string(),
                            },
                        },
                        TimedEvent {
                            delta_ticks: 100,
                            event: Event::Text {
                                text_type: TextType::Marker,
                                text: "Verse 1".to_string(),
                            },
                        },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::TrackName,
                                text: "Bass Track".to_string(),
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::Text {
                                text_type: TextType::InstrumentName,
                                text: "Electric Bass".to_string(),
                            },
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_extract_track_names() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.track_names.len(), 2);
        assert!(metadata.track_names.contains(&"Piano Track".to_string()));
        assert!(metadata.track_names.contains(&"Bass Track".to_string()));
    }

    #[test]
    fn test_extract_copyright() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.copyright, Some("2025 Test Artist".to_string()));
    }

    #[test]
    fn test_extract_instrument_names() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.instrument_names.len(), 1);
        assert_eq!(metadata.instrument_names[0], "Electric Bass");
    }

    #[test]
    fn test_extract_markers() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        assert_eq!(metadata.markers.len(), 1);
        assert_eq!(metadata.markers[0], "Verse 1");
    }

    #[test]
    fn test_empty_metadata() {
        let midi = MidiFile {
            header: Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }],
        };

        let metadata = TextMetadata::extract(&midi);
        assert!(metadata.is_empty());
    }

    #[test]
    fn test_deduplication() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![TimedEvent {
                        delta_ticks: 0,
                        event: Event::Text {
                            text_type: TextType::TrackName,
                            text: "Duplicate".to_string(),
                        },
                    }],
                },
                Track {
                    events: vec![TimedEvent {
                        delta_ticks: 0,
                        event: Event::Text {
                            text_type: TextType::TrackName,
                            text: "Duplicate".to_string(),
                        },
                    }],
                },
            ],
        };

        let metadata = TextMetadata::extract(&midi);
        assert_eq!(metadata.track_names.len(), 1);
        assert_eq!(metadata.track_names[0], "Duplicate");
    }

    #[test]
    fn test_summary() {
        let midi = create_test_midi_with_text();
        let metadata = TextMetadata::extract(&midi);

        let summary = metadata.summary();
        assert!(summary.contains("2 tracks"));
        assert!(summary.contains("© 2025 Test Artist"));
        assert!(summary.contains("1 instruments"));
        assert!(summary.contains("1 markers"));
    }

    #[test]
    fn test_empty_summary() {
        let midi = MidiFile {
            header: Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }],
        };

        let metadata = TextMetadata::extract(&midi);
        assert_eq!(metadata.summary(), "No text metadata");
    }
}

```

### `src/core/midi/types.rs` {#src-core-midi-types-rs}

- **Lines**: 871 (code: 771, comments: 0, blank: 100)

#### Source Code

```rust
use serde::{Deserialize, Serialize};

/// Represents a complete MIDI file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiFile {
    pub header: Header,
    pub tracks: Vec<Track>,
}

/// MIDI header chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub format: u16, // 0, 1, or 2
    pub num_tracks: u16,
    pub ticks_per_quarter_note: u16,
}

/// A single MIDI track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub events: Vec<TimedEvent>,
}

/// Event with delta time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    pub delta_ticks: u32,
    pub event: Event,
}

/// MIDI events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // Channel events
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    Aftertouch {
        channel: u8,
        note: u8,
        pressure: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    ProgramChange {
        channel: u8,
        program: u8,
    },
    ChannelAftertouch {
        channel: u8,
        pressure: u8,
    },
    PitchBend {
        channel: u8,
        value: i16,
    },

    // Meta events
    TempoChange {
        microseconds_per_quarter: u32,
    },
    TimeSignature {
        numerator: u8,
        denominator: u8,
        clocks_per_click: u8,
        thirty_seconds_per_quarter: u8,
    },
    KeySignature {
        sharps_flats: i8,
        is_minor: bool,
    },
    Text {
        text_type: TextType,
        text: String,
    },
    EndOfTrack,

    // SysEx
    SysEx {
        data: Vec<u8>,
    },

    // Unknown/unsupported
    Unknown {
        status: u8,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextType {
    Text,
    Copyright,
    TrackName,
    InstrumentName,
    Lyric,
    Marker,
    CuePoint,
}

impl MidiFile {
    /// Calculate total duration in seconds
    pub fn duration_seconds(&self, _default_tempo_bpm: f64) -> f64 {
        let mut total_ticks = 0u64;
        let mut current_tempo_us_per_qn = 500_000u32; // Default: 120 BPM

        for track in &self.tracks {
            let mut track_ticks = 0u64;

            for timed_event in &track.events {
                track_ticks += timed_event.delta_ticks as u64;

                // Update tempo if we encounter a tempo change
                if let Event::TempoChange { microseconds_per_quarter } = timed_event.event {
                    current_tempo_us_per_qn = microseconds_per_quarter;
                }
            }

            total_ticks = total_ticks.max(track_ticks);
        }

        // Convert ticks to seconds
        let seconds_per_tick = (current_tempo_us_per_qn as f64 / 1_000_000.0)
            / self.header.ticks_per_quarter_note as f64;
        total_ticks as f64 * seconds_per_tick
    }

    /// Count total notes across all tracks
    pub fn total_notes(&self) -> usize {
        self.tracks
            .iter()
            .flat_map(|track| &track.events)
            .filter(|event| matches!(event.event, Event::NoteOn { velocity, .. } if velocity > 0))
            .count()
    }

    /// Get all unique MIDI channels used
    pub fn channels_used(&self) -> Vec<u8> {
        let mut channels = std::collections::HashSet::new();

        for track in &self.tracks {
            for timed_event in &track.events {
                if let Some(channel) = timed_event.event.channel() {
                    channels.insert(channel);
                }
            }
        }

        let mut result: Vec<u8> = channels.into_iter().collect();
        result.sort();
        result
    }
}

impl Event {
    /// Get the MIDI channel for channel events, None for meta/sysex
    pub fn channel(&self) -> Option<u8> {
        match self {
            Event::NoteOn { channel, .. }
            | Event::NoteOff { channel, .. }
            | Event::Aftertouch { channel, .. }
            | Event::ControlChange { channel, .. }
            | Event::ProgramChange { channel, .. }
            | Event::ChannelAftertouch { channel, .. }
            | Event::PitchBend { channel, .. } => Some(*channel),
            _ => None,
        }
    }

    /// Check if this is a note event
    pub fn is_note(&self) -> bool {
        matches!(self, Event::NoteOn { .. } | Event::NoteOff { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a basic MIDI file for testing
    fn create_basic_midi() -> MidiFile {
        MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }],
        }
    }

    /// Helper function to create a MIDI file with notes
    fn create_midi_with_notes() -> MidiFile {
        MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 1, note: 64, velocity: 80 },
                    },
                ],
            }],
        }
    }

    mod midi_file_tests {
        use super::*;

        #[test]
        fn test_duration_seconds_empty_file() {
            let midi = create_basic_midi();
            let duration = midi.duration_seconds(120.0);

            // Empty file should have 0 duration
            assert_eq!(duration, 0.0);
        }

        #[test]
        fn test_duration_seconds_with_default_tempo() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 1920, // 4 quarters = 1 bar at 480 tpq
                            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                        },
                    ],
                }],
            };

            let duration = midi.duration_seconds(120.0);

            // At 120 BPM (500,000 μs/quarter), 1920 ticks = 4 quarters = 2 seconds
            // seconds_per_tick = 500_000 / 1_000_000 / 480 = 0.00104166...
            // duration = 1920 * 0.00104166... ≈ 2.0 seconds
            assert!(
                (duration - 2.0).abs() < 0.01,
                "Duration should be ~2.0 seconds, got {}",
                duration
            );
        }

        #[test]
        fn test_duration_seconds_with_tempo_change() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange {
                                microseconds_per_quarter: 600_000, // 100 BPM
                            },
                        },
                        TimedEvent {
                            delta_ticks: 1920, // 4 quarters at 100 BPM
                            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                        },
                    ],
                }],
            };

            let duration = midi.duration_seconds(120.0);

            // At 100 BPM (600,000 μs/quarter), 1920 ticks = 4 quarters = 2.4 seconds
            // seconds_per_tick = 600_000 / 1_000_000 / 480 = 0.00125
            // duration = 1920 * 0.00125 = 2.4 seconds
            assert!(
                (duration - 2.4).abs() < 0.01,
                "Duration should be ~2.4 seconds, got {}",
                duration
            );
        }

        #[test]
        fn test_duration_seconds_multiple_tracks() {
            // Duration should be the length of the longest track
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
                tracks: vec![
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 960, // Short track
                            event: Event::EndOfTrack,
                        }],
                    },
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 1920, // Longer track
                            event: Event::EndOfTrack,
                        }],
                    },
                ],
            };

            let duration = midi.duration_seconds(120.0);

            // Should use longest track (1920 ticks)
            assert!(
                (duration - 2.0).abs() < 0.01,
                "Duration should be ~2.0 seconds (longest track), got {}",
                duration
            );
        }

        #[test]
        fn test_total_notes_empty_file() {
            let midi = create_basic_midi();
            assert_eq!(midi.total_notes(), 0);
        }

        #[test]
        fn test_total_notes_with_notes() {
            let midi = create_midi_with_notes();

            // Should count 2 NoteOn events (velocity > 0)
            assert_eq!(midi.total_notes(), 2);
        }

        #[test]
        fn test_total_notes_excludes_zero_velocity() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 60,
                                velocity: 0, // Zero velocity = note off
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 0,
                                note: 64,
                                velocity: 100, // Real note on
                            },
                        },
                    ],
                }],
            };

            // Should only count note with velocity > 0
            assert_eq!(midi.total_notes(), 1);
        }

        #[test]
        fn test_total_notes_excludes_note_off() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 480,
                            event: Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                        },
                    ],
                }],
            };

            // Should only count NoteOn events
            assert_eq!(midi.total_notes(), 1);
        }

        #[test]
        fn test_total_notes_multiple_tracks() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
                tracks: vec![
                    Track {
                        events: vec![
                            TimedEvent {
                                delta_ticks: 0,
                                event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                            },
                            TimedEvent {
                                delta_ticks: 0,
                                event: Event::NoteOn { channel: 0, note: 64, velocity: 80 },
                            },
                        ],
                    },
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 1, note: 67, velocity: 90 },
                        }],
                    },
                ],
            };

            // Should count notes across all tracks
            assert_eq!(midi.total_notes(), 3);
        }

        #[test]
        fn test_channels_used_empty_file() {
            let midi = create_basic_midi();
            assert_eq!(midi.channels_used(), Vec::<u8>::new());
        }

        #[test]
        fn test_channels_used_single_channel() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 5, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 480,
                            event: Event::NoteOff { channel: 5, note: 60, velocity: 0 },
                        },
                    ],
                }],
            };

            assert_eq!(midi.channels_used(), vec![5]);
        }

        #[test]
        fn test_channels_used_multiple_channels() {
            let midi = create_midi_with_notes();

            // Should return sorted unique channels
            assert_eq!(midi.channels_used(), vec![0, 1]);
        }

        #[test]
        fn test_channels_used_all_16_channels() {
            let mut events = Vec::new();

            // Add events on all 16 MIDI channels (0-15)
            for channel in 0..16 {
                events.push(TimedEvent {
                    delta_ticks: 0,
                    event: Event::NoteOn { channel, note: 60, velocity: 100 },
                });
            }

            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track { events }],
            };

            assert_eq!(midi.channels_used(), (0..16).collect::<Vec<u8>>());
        }

        #[test]
        fn test_channels_used_excludes_meta_events() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                }],
            };

            // Should only include channel from NoteOn
            assert_eq!(midi.channels_used(), vec![0]);
        }

        #[test]
        fn test_channels_used_deduplicates() {
            let midi = MidiFile {
                header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
                tracks: vec![Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 3, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 1, note: 64, velocity: 80 },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 3, // Duplicate
                                note: 67,
                                velocity: 90,
                            },
                        },
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn {
                                channel: 1, // Duplicate
                                note: 70,
                                velocity: 85,
                            },
                        },
                    ],
                }],
            };

            // Should return sorted unique channels
            assert_eq!(midi.channels_used(), vec![1, 3]);
        }
    }

    mod event_tests {
        use super::*;

        #[test]
        fn test_channel_note_on() {
            let event = Event::NoteOn { channel: 5, note: 60, velocity: 100 };
            assert_eq!(event.channel(), Some(5));
        }

        #[test]
        fn test_channel_note_off() {
            let event = Event::NoteOff { channel: 3, note: 60, velocity: 0 };
            assert_eq!(event.channel(), Some(3));
        }

        #[test]
        fn test_channel_aftertouch() {
            let event = Event::Aftertouch { channel: 7, note: 60, pressure: 50 };
            assert_eq!(event.channel(), Some(7));
        }

        #[test]
        fn test_channel_control_change() {
            let event = Event::ControlChange { channel: 10, controller: 7, value: 100 };
            assert_eq!(event.channel(), Some(10));
        }

        #[test]
        fn test_channel_program_change() {
            let event = Event::ProgramChange { channel: 15, program: 0 };
            assert_eq!(event.channel(), Some(15));
        }

        #[test]
        fn test_channel_channel_aftertouch() {
            let event = Event::ChannelAftertouch { channel: 2, pressure: 64 };
            assert_eq!(event.channel(), Some(2));
        }

        #[test]
        fn test_channel_pitch_bend() {
            let event = Event::PitchBend { channel: 8, value: 0 };
            assert_eq!(event.channel(), Some(8));
        }

        #[test]
        fn test_channel_tempo_change_returns_none() {
            let event = Event::TempoChange { microseconds_per_quarter: 500_000 };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_time_signature_returns_none() {
            let event = Event::TimeSignature {
                numerator: 4,
                denominator: 4,
                clocks_per_click: 24,
                thirty_seconds_per_quarter: 8,
            };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_key_signature_returns_none() {
            let event = Event::KeySignature { sharps_flats: 0, is_minor: false };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_text_returns_none() {
            let event = Event::Text { text_type: TextType::TrackName, text: "Piano".to_string() };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_end_of_track_returns_none() {
            let event = Event::EndOfTrack;
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_sysex_returns_none() {
            let event = Event::SysEx { data: vec![0xF0, 0x7E, 0x7F, 0xF7] };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_channel_unknown_returns_none() {
            let event = Event::Unknown { status: 0xFF, data: vec![0x01, 0x02] };
            assert_eq!(event.channel(), None);
        }

        #[test]
        fn test_is_note_for_note_on() {
            let event = Event::NoteOn { channel: 0, note: 60, velocity: 100 };
            assert!(event.is_note());
        }

        #[test]
        fn test_is_note_for_note_off() {
            let event = Event::NoteOff { channel: 0, note: 60, velocity: 0 };
            assert!(event.is_note());
        }

        #[test]
        fn test_is_note_for_control_change() {
            let event = Event::ControlChange { channel: 0, controller: 7, value: 100 };
            assert!(!event.is_note());
        }

        #[test]
        fn test_is_note_for_program_change() {
            let event = Event::ProgramChange { channel: 0, program: 5 };
            assert!(!event.is_note());
        }

        #[test]
        fn test_is_note_for_tempo_change() {
            let event = Event::TempoChange { microseconds_per_quarter: 500_000 };
            assert!(!event.is_note());
        }

        #[test]
        fn test_is_note_for_aftertouch() {
            let event = Event::Aftertouch { channel: 0, note: 60, pressure: 50 };
            assert!(!event.is_note());
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_serialize_deserialize_midi_file() {
            let original = MidiFile {
                header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
                tracks: vec![
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        }],
                    },
                    Track {
                        events: vec![TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        }],
                    },
                ],
            };

            let json = serde_json::to_string(&original).expect("Failed to serialize");
            let deserialized: MidiFile =
                serde_json::from_str(&json).expect("Failed to deserialize");

            // Verify structure
            assert_eq!(deserialized.header.format, 1);
            assert_eq!(deserialized.header.num_tracks, 2);
            assert_eq!(deserialized.header.ticks_per_quarter_note, 480);
            assert_eq!(deserialized.tracks.len(), 2);
            assert_eq!(deserialized.tracks[0].events.len(), 1);
            assert_eq!(deserialized.tracks[1].events.len(), 1);
        }

        #[test]
        fn test_serialize_deserialize_all_event_types() {
            let events = vec![
                Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                Event::NoteOff { channel: 0, note: 60, velocity: 64 },
                Event::Aftertouch { channel: 0, note: 60, pressure: 50 },
                Event::ControlChange { channel: 0, controller: 7, value: 100 },
                Event::ProgramChange { channel: 0, program: 5 },
                Event::ChannelAftertouch { channel: 0, pressure: 64 },
                Event::PitchBend { channel: 0, value: 8192 },
                Event::TempoChange { microseconds_per_quarter: 500_000 },
                Event::TimeSignature {
                    numerator: 4,
                    denominator: 4,
                    clocks_per_click: 24,
                    thirty_seconds_per_quarter: 8,
                },
                Event::KeySignature { sharps_flats: -2, is_minor: true },
                Event::Text { text_type: TextType::TrackName, text: "Piano".to_string() },
                Event::EndOfTrack,
                Event::SysEx { data: vec![0xF0, 0x7E, 0x7F, 0xF7] },
                Event::Unknown { status: 0xFF, data: vec![0x01, 0x02] },
            ];

            for original_event in events {
                let json =
                    serde_json::to_string(&original_event).expect("Failed to serialize event");
                let _deserialized: Event =
                    serde_json::from_str(&json).expect("Failed to deserialize event");
                // If we get here, serialization round-trip succeeded
            }
        }

        #[test]
        fn test_serialize_text_types() {
            let text_types = vec![
                TextType::Text,
                TextType::Copyright,
                TextType::TrackName,
                TextType::InstrumentName,
                TextType::Lyric,
                TextType::Marker,
                TextType::CuePoint,
            ];

            for original_type in text_types {
                let json =
                    serde_json::to_string(&original_type).expect("Failed to serialize TextType");
                let _deserialized: TextType =
                    serde_json::from_str(&json).expect("Failed to deserialize TextType");
                // If we get here, serialization round-trip succeeded
            }
        }
    }

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_header_format_0() {
            let header = Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 96 };
            assert_eq!(header.format, 0);
        }

        #[test]
        fn test_header_format_2() {
            let header = Header { format: 2, num_tracks: 5, ticks_per_quarter_note: 960 };
            assert_eq!(header.format, 2);
        }

        #[test]
        fn test_high_ticks_per_quarter() {
            let header = Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 960, // High resolution
            };
            assert_eq!(header.ticks_per_quarter_note, 960);
        }

        #[test]
        fn test_low_ticks_per_quarter() {
            let header = Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: 96, // Low resolution
            };
            assert_eq!(header.ticks_per_quarter_note, 96);
        }

        #[test]
        fn test_pitch_bend_positive() {
            let event = Event::PitchBend {
                channel: 0,
                value: 8192, // Center position
            };
            if let Event::PitchBend { value, .. } = event {
                assert_eq!(value, 8192);
            }
        }

        #[test]
        fn test_pitch_bend_negative() {
            let event = Event::PitchBend {
                channel: 0,
                value: -8192, // Max down
            };
            if let Event::PitchBend { value, .. } = event {
                assert_eq!(value, -8192);
            }
        }

        #[test]
        fn test_key_signature_sharps() {
            let event = Event::KeySignature {
                sharps_flats: 4, // E major
                is_minor: false,
            };
            if let Event::KeySignature { sharps_flats, is_minor } = event {
                assert_eq!(sharps_flats, 4);
                assert!(!is_minor);
            }
        }

        #[test]
        fn test_key_signature_flats() {
            let event = Event::KeySignature {
                sharps_flats: -3, // Eb major
                is_minor: false,
            };
            if let Event::KeySignature { sharps_flats, is_minor } = event {
                assert_eq!(sharps_flats, -3);
                assert!(!is_minor);
            }
        }

        #[test]
        fn test_very_large_delta_ticks() {
            let event = TimedEvent { delta_ticks: u32::MAX, event: Event::EndOfTrack };
            assert_eq!(event.delta_ticks, u32::MAX);
        }

        #[test]
        fn test_empty_sysex() {
            let event = Event::SysEx { data: vec![] };
            if let Event::SysEx { data } = event {
                assert!(data.is_empty());
            }
        }

        #[test]
        fn test_empty_text() {
            let event = Event::Text { text_type: TextType::Text, text: String::new() };
            if let Event::Text { text, .. } = event {
                assert!(text.is_empty());
            }
        }

        #[test]
        fn test_unicode_text() {
            let event =
                Event::Text { text_type: TextType::TrackName, text: "Piano Track".to_string() };
            if let Event::Text { text, .. } = event {
                assert_eq!(text, "Piano Track");
            }
        }
    }
}

```

### `src/core/mod.rs` {#src-core-mod-rs}

- **Lines**: 3 (code: 3, comments: 0, blank: 0)

#### Source Code

```rust
pub mod analysis;
/// Core MIDI processing modules
pub mod midi;

```

### `src/db/mod.rs` {#src-db-mod-rs}

- **Lines**: 6 (code: 5, comments: 0, blank: 1)

#### Source Code

```rust
/// Database models and repositories
pub mod models;
pub mod repositories;

// Re-export commonly used types
pub use models::{File, MidiMetadata};

```

### `src/db/models/analysis.rs` {#src-db-models-analysis-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

#### Source Code

```rust
/// Analysis result model
///
/// Placeholder - will be populated in Phase 5 with DAW version

// Temporary stub to allow compilation
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    // Will be filled in Phase 5
}

```

### `src/db/models/error.rs` {#src-db-models-error-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

#### Source Code

```rust
/// Database error types
///
/// Placeholder - will be populated in Phase 5 with DAW version

// Temporary stub to allow compilation
#[derive(Debug, Clone)]
pub struct DbError {
    // Will be filled in Phase 5
}

```

### `src/db/models/midi.rs` {#src-db-models-midi-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

#### Source Code

```rust
/// MIDI metadata model
///
/// Placeholder - will be populated in Phase 5 with DAW version

// Temporary stub to allow compilation
#[derive(Debug, Clone)]
pub struct MidiMetadata {
    // Will be filled in Phase 5
}

```

### `src/db/models/midi_file.rs` {#src-db-models-midi-file-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

#### Source Code

```rust
/// MIDI file database model
///
/// Placeholder - will be populated in Phase 5 with DAW version (better modular structure)

// Temporary stub to allow compilation
#[derive(Debug, Clone)]
pub struct File {
    // Will be filled in Phase 5
}

```

### `src/db/models/mod.rs` {#src-db-models-mod-rs}

- **Lines**: 15 (code: 14, comments: 0, blank: 1)

#### Source Code

```rust
pub mod analysis;
pub mod error;
pub mod midi;
/// Database model types
pub mod midi_file;
pub mod search;
pub mod sequencer;

// Re-export main types
pub use analysis::AnalysisResult;
pub use error::DbError;
pub use midi::MidiMetadata;
pub use midi_file::File;
pub use search::SearchFilters;
pub use sequencer::SequencerTrack;

```

### `src/db/models/search.rs` {#src-db-models-search-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

#### Source Code

```rust
/// Search filters model
///
/// Placeholder - will be populated in Phase 5 with DAW version

// Temporary stub to allow compilation
#[derive(Debug, Clone)]
pub struct SearchFilters {
    // Will be filled in Phase 5
}

```

### `src/db/models/sequencer.rs` {#src-db-models-sequencer-rs}

- **Lines**: 9 (code: 8, comments: 0, blank: 1)

#### Source Code

```rust
/// Sequencer model
///
/// Placeholder - will be populated in Phase 5 with DAW version

// Temporary stub to allow compilation
#[derive(Debug, Clone)]
pub struct SequencerTrack {
    // Will be filled in Phase 5
}

```

### `src/db/repositories/file_repository.rs` {#src-db-repositories-file-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

#### Source Code

```rust
/// File repository
///
/// Placeholder - will be populated in Phase 5 with Pipeline version
// Temporary stub to allow compilation
pub struct FileRepository {
    // Will be filled in Phase 5
}

```

### `src/db/repositories/metadata_repository.rs` {#src-db-repositories-metadata-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

#### Source Code

```rust
/// Metadata repository
///
/// Placeholder - will be populated in Phase 5 with Pipeline version
// Temporary stub to allow compilation
pub struct MetadataRepository {
    // Will be filled in Phase 5
}

```

### `src/db/repositories/mod.rs` {#src-db-repositories-mod-rs}

- **Lines**: 11 (code: 10, comments: 0, blank: 1)

#### Source Code

```rust
/// Database repository layer
pub mod file_repository;
pub mod metadata_repository;
pub mod search_repository;
pub mod tag_repository;

// Re-export repository types
pub use file_repository::FileRepository;
pub use metadata_repository::MetadataRepository;
pub use search_repository::SearchRepository;
pub use tag_repository::TagRepository;

```

### `src/db/repositories/search_repository.rs` {#src-db-repositories-search-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

#### Source Code

```rust
/// Search repository
///
/// Placeholder - will be populated in Phase 5 with Pipeline version
// Temporary stub to allow compilation
pub struct SearchRepository {
    // Will be filled in Phase 5
}

```

### `src/db/repositories/tag_repository.rs` {#src-db-repositories-tag-repository-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

#### Source Code

```rust
/// Tag repository
///
/// Placeholder - will be populated in Phase 5 with Pipeline version
// Temporary stub to allow compilation
pub struct TagRepository {
    // Will be filled in Phase 5
}

```

### `src/lib.rs` {#src-lib-rs}

- **Lines**: 27 (code: 25, comments: 0, blank: 2)

#### Source Code

```rust
/// MIDI Library Shared Code
///
/// This crate contains all shared functionality used by:
/// - Pipeline (import, process, analyze)
/// - DAW (playback, sequence, MIDI out)
///
/// ## Structure
///
/// - `core::midi` - MIDI parsing and types
/// - `core::analysis` - Musical analysis (BPM, key detection, etc.)
/// - `db::models` - Database model types
/// - `db::repositories` - Database access layer
pub mod core;
pub mod db;

// Re-export top-level modules for convenience
pub use core::analysis;
pub use core::midi;
pub use db::{models, repositories};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

```
