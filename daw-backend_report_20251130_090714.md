# Project Report: daw-backend

> Generated: 2025-11-30 09:07:14
> Path: `/home/dojevou/projects/midi-software-center/daw/src-tauri`

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
| Total Files | 103 |
| Total Lines | 34,425 |
| Lines of Code | 29,706 |
| Functions | 0 |
| Classes | 0 |
| Avg Pylint Score | 0.00/10 |
| Docstring Coverage | 0.0% |

## Project Statistics

### Files by Extension

| Extension | Count | Lines |
|-----------|-------|-------|
| .rs | 102 | 34,378 |
| .toml | 1 | 47 |

## Code Quality

## Dependencies

## File Structure

```
src-tauri/
├── backups/
│   └── project_backup_20251130_090327.zip
├── gen/
│   └── schemas/
│       ├── acl-manifests.json
│       ├── capabilities.json
│       ├── desktop-schema.json
│       └── linux-schema.json
├── icons/
│   ├── 128x128.png
│   ├── 128x128@2x.png
│   ├── 32x32.png
│   ├── icon.ico
│   └── icon.png
├── logs/
│   ├── daw.log.2025-11-03
│   └── daw.log.2025-11-09
├── migrations/
│   └── 20251111044328_initial_schema.sql
├── src/
│   ├── bin/
│   │   └── profile_queries.rs
│   ├── browsers/
│   │   ├── loop_browser.rs
│   │   ├── mod.rs
│   │   └── project_browser.rs
│   ├── commands/
│   │   ├── analysis.rs
│   │   ├── automation.rs
│   │   ├── database.rs
│   │   ├── daw.rs
│   │   ├── export.rs
│   │   ├── midi.rs
│   │   ├── mixer.rs
│   │   ├── mod.rs
│   │   ├── pipeline.rs
│   │   ├── project.rs
│   │   ├── search.rs
│   │   ├── sequencer.rs
│   │   ├── system.rs
│   │   └── window.rs
│   ├── core/
│   │   ├── compatibility/
│   │   │   ├── mod.rs
│   │   │   ├── music.rs
│   │   │   ├── scoring.rs
│   │   │   └── types.rs
│   │   ├── midi/
│   │   │   ├── loader.rs
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs
│   │   │   ├── types.rs
│   │   │   ├── validator.rs
│   │   │   └── writer.rs
│   │   ├── sequencer/
│   │   │   ├── mod.rs
│   │   │   └── timing.rs
│   │   └── mod.rs
│   ├── editors/
│   │   ├── controller.rs
│   │   ├── mod.rs
│   │   ├── piano_roll.rs
│   │   ├── tempo.rs
│   │   └── velocity.rs
│   ├── hardware/
│   │   ├── device_manager.rs
│   │   ├── HARDWARE_SUMMARY.md
│   │   ├── midi_monitor.rs
│   │   ├── midi_router.rs
│   │   └── mod.rs
│   ├── midi/
│   │   ├── manager.rs
│   │   └── mod.rs
│   ├── models/
│   │   ├── analysis.rs
│   │   ├── error.rs
│   │   ├── midi.rs
│   │   ├── midi_file.rs
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   └── sequencer.rs
│   ├── profiling/
│   │   ├── commands.rs
│   │   ├── memory.rs
│   │   ├── mod.rs
│   │   ├── query_analyzer.rs
│   │   ├── query_cache.rs
│   │   └── render_metrics.rs
│   ├── sequencer/
│   │   ├── engine.rs
│   │   ├── mod.rs
│   │   ├── scheduler.rs
│   │   └── track.rs
│   ├── settings/
│   │   ├── advanced.rs
│   │   ├── audio.rs
│   │   ├── display.rs
│   │   ├── general.rs
│   │   ├── import_export.rs
│   │   ├── keyboard.rs
│   │   ├── library.rs
│   │   ├── midi.rs
│   │   ├── mixer.rs
│   │   ├── mod.rs
│   │   ├── performance.rs
│   │   ├── playback.rs
│   │   ├── privacy.rs
│   │   ├── recording.rs
│   │   ├── sync.rs
│   │   └── track.rs
│   ├── undo_redo/
│   │   ├── commands.rs
│   │   ├── controller.rs
│   │   ├── core.rs
│   │   ├── performance.rs
│   │   ├── piano_roll.rs
│   │   ├── serialization.rs
│   │   ├── tempo.rs
│   │   ├── track.rs
│   │   └── velocity.rs
│   ├── windows/
│   │   ├── mod.rs
│   │   └── state.rs
│   ├── automation.rs
│   ├── command_palette.rs
│   ├── lib.rs
│   ├── main.rs
│   └── undo_redo.rs
├── tests/
│   ├── commands/
│   │   └── mod.rs
│   ├── common/
│   │   ├── assertions.rs
│   │   ├── builders.rs
│   │   ├── database.rs
│   │   ├── fixtures.rs
│   │   ├── mocks.rs
│   │   └── mod.rs
│   ├── daw_database_integration_test.rs
│   ├── lib.rs
│   └── models_test.rs
├── build.rs
├── Cargo.toml
├── project_report_20251130_090327.json
├── project_report_20251130_090327.md
└── tauri.conf.json
```

## TODOs and FIXMEs

*No TODOs or FIXMEs found*

## File Details

### `Cargo.toml` {#cargo-toml}

- **Lines**: 47 (code: 41, comments: 0, blank: 6)

### `build.rs` {#build-rs}

- **Lines**: 3 (code: 3, comments: 0, blank: 0)

### `src/automation.rs` {#src-automation-rs}

- **Lines**: 1236 (code: 1050, comments: 0, blank: 186)

### `src/bin/profile_queries.rs` {#src-bin-profile-queries-rs}

- **Lines**: 231 (code: 199, comments: 0, blank: 32)

### `src/browsers/loop_browser.rs` {#src-browsers-loop-browser-rs}

- **Lines**: 484 (code: 409, comments: 0, blank: 75)

### `src/browsers/mod.rs` {#src-browsers-mod-rs}

- **Lines**: 59 (code: 52, comments: 0, blank: 7)

### `src/browsers/project_browser.rs` {#src-browsers-project-browser-rs}

- **Lines**: 514 (code: 429, comments: 0, blank: 85)

### `src/command_palette.rs` {#src-command-palette-rs}

- **Lines**: 833 (code: 744, comments: 0, blank: 89)

### `src/commands/analysis.rs` {#src-commands-analysis-rs}

- **Lines**: 365 (code: 332, comments: 0, blank: 33)

### `src/commands/automation.rs` {#src-commands-automation-rs}

- **Lines**: 495 (code: 416, comments: 0, blank: 79)

### `src/commands/database.rs` {#src-commands-database-rs}

- **Lines**: 237 (code: 203, comments: 0, blank: 34)

### `src/commands/daw.rs` {#src-commands-daw-rs}

- **Lines**: 530 (code: 464, comments: 0, blank: 66)

### `src/commands/export.rs` {#src-commands-export-rs}

- **Lines**: 196 (code: 179, comments: 0, blank: 17)

### `src/commands/midi.rs` {#src-commands-midi-rs}

- **Lines**: 83 (code: 74, comments: 0, blank: 9)

### `src/commands/mixer.rs` {#src-commands-mixer-rs}

- **Lines**: 355 (code: 308, comments: 0, blank: 47)

### `src/commands/mod.rs` {#src-commands-mod-rs}

- **Lines**: 46 (code: 42, comments: 0, blank: 4)

### `src/commands/pipeline.rs` {#src-commands-pipeline-rs}

- **Lines**: 383 (code: 330, comments: 0, blank: 53)

### `src/commands/project.rs` {#src-commands-project-rs}

- **Lines**: 172 (code: 152, comments: 0, blank: 20)

### `src/commands/search.rs` {#src-commands-search-rs}

- **Lines**: 345 (code: 315, comments: 0, blank: 30)

### `src/commands/sequencer.rs` {#src-commands-sequencer-rs}

- **Lines**: 187 (code: 162, comments: 0, blank: 25)

### `src/commands/system.rs` {#src-commands-system-rs}

- **Lines**: 293 (code: 266, comments: 0, blank: 27)

### `src/commands/window.rs` {#src-commands-window-rs}

- **Lines**: 578 (code: 495, comments: 0, blank: 83)

### `src/core/compatibility/mod.rs` {#src-core-compatibility-mod-rs}

- **Lines**: 18 (code: 17, comments: 0, blank: 1)

### `src/core/compatibility/music.rs` {#src-core-compatibility-music-rs}

- **Lines**: 302 (code: 252, comments: 0, blank: 50)

### `src/core/compatibility/scoring.rs` {#src-core-compatibility-scoring-rs}

- **Lines**: 262 (code: 230, comments: 0, blank: 32)

### `src/core/compatibility/types.rs` {#src-core-compatibility-types-rs}

- **Lines**: 179 (code: 158, comments: 0, blank: 21)

### `src/core/midi/loader.rs` {#src-core-midi-loader-rs}

- **Lines**: 267 (code: 241, comments: 0, blank: 26)

### `src/core/midi/mod.rs` {#src-core-midi-mod-rs}

- **Lines**: 22 (code: 21, comments: 0, blank: 1)

### `src/core/midi/parser.rs` {#src-core-midi-parser-rs}

- **Lines**: 934 (code: 765, comments: 0, blank: 169)

### `src/core/midi/types.rs` {#src-core-midi-types-rs}

- **Lines**: 157 (code: 143, comments: 0, blank: 14)

### `src/core/midi/validator.rs` {#src-core-midi-validator-rs}

- **Lines**: 153 (code: 135, comments: 0, blank: 18)

### `src/core/midi/writer.rs` {#src-core-midi-writer-rs}

- **Lines**: 292 (code: 254, comments: 0, blank: 38)

### `src/core/mod.rs` {#src-core-mod-rs}

- **Lines**: 7 (code: 7, comments: 0, blank: 0)

### `src/core/sequencer/mod.rs` {#src-core-sequencer-mod-rs}

- **Lines**: 12 (code: 11, comments: 0, blank: 1)

### `src/core/sequencer/timing.rs` {#src-core-sequencer-timing-rs}

- **Lines**: 286 (code: 257, comments: 0, blank: 29)

### `src/editors/controller.rs` {#src-editors-controller-rs}

- **Lines**: 410 (code: 347, comments: 0, blank: 63)

### `src/editors/mod.rs` {#src-editors-mod-rs}

- **Lines**: 62 (code: 54, comments: 0, blank: 8)

### `src/editors/piano_roll.rs` {#src-editors-piano-roll-rs}

- **Lines**: 590 (code: 509, comments: 0, blank: 81)

### `src/editors/tempo.rs` {#src-editors-tempo-rs}

- **Lines**: 411 (code: 347, comments: 0, blank: 64)

### `src/editors/velocity.rs` {#src-editors-velocity-rs}

- **Lines**: 372 (code: 306, comments: 0, blank: 66)

### `src/hardware/device_manager.rs` {#src-hardware-device-manager-rs}

- **Lines**: 584 (code: 480, comments: 0, blank: 104)

### `src/hardware/midi_monitor.rs` {#src-hardware-midi-monitor-rs}

- **Lines**: 584 (code: 497, comments: 0, blank: 87)

### `src/hardware/midi_router.rs` {#src-hardware-midi-router-rs}

- **Lines**: 826 (code: 695, comments: 0, blank: 131)

### `src/hardware/mod.rs` {#src-hardware-mod-rs}

- **Lines**: 121 (code: 103, comments: 0, blank: 18)

### `src/lib.rs` {#src-lib-rs}

- **Lines**: 30 (code: 27, comments: 0, blank: 3)

### `src/main.rs` {#src-main-rs}

- **Lines**: 258 (code: 234, comments: 0, blank: 24)

### `src/midi/manager.rs` {#src-midi-manager-rs}

- **Lines**: 304 (code: 265, comments: 0, blank: 39)

### `src/midi/mod.rs` {#src-midi-mod-rs}

- **Lines**: 7 (code: 6, comments: 0, blank: 1)

### `src/models/analysis.rs` {#src-models-analysis-rs}

- **Lines**: 157 (code: 142, comments: 0, blank: 15)

### `src/models/error.rs` {#src-models-error-rs}

- **Lines**: 53 (code: 43, comments: 0, blank: 10)

### `src/models/midi.rs` {#src-models-midi-rs}

- **Lines**: 92 (code: 85, comments: 0, blank: 7)

### `src/models/midi_file.rs` {#src-models-midi-file-rs}

- **Lines**: 114 (code: 101, comments: 0, blank: 13)

### `src/models/mod.rs` {#src-models-mod-rs}

- **Lines**: 32 (code: 30, comments: 0, blank: 2)

### `src/models/search.rs` {#src-models-search-rs}

- **Lines**: 77 (code: 66, comments: 0, blank: 11)

### `src/models/sequencer.rs` {#src-models-sequencer-rs}

- **Lines**: 90 (code: 83, comments: 0, blank: 7)

### `src/profiling/commands.rs` {#src-profiling-commands-rs}

- **Lines**: 358 (code: 334, comments: 0, blank: 24)

### `src/profiling/memory.rs` {#src-profiling-memory-rs}

- **Lines**: 1472 (code: 1341, comments: 0, blank: 131)

### `src/profiling/mod.rs` {#src-profiling-mod-rs}

- **Lines**: 34 (code: 29, comments: 0, blank: 5)

### `src/profiling/query_analyzer.rs` {#src-profiling-query-analyzer-rs}

- **Lines**: 1380 (code: 1192, comments: 0, blank: 188)

### `src/profiling/query_cache.rs` {#src-profiling-query-cache-rs}

- **Lines**: 683 (code: 623, comments: 0, blank: 60)

### `src/profiling/render_metrics.rs` {#src-profiling-render-metrics-rs}

- **Lines**: 956 (code: 895, comments: 0, blank: 61)

### `src/sequencer/engine.rs` {#src-sequencer-engine-rs}

- **Lines**: 428 (code: 354, comments: 0, blank: 74)

### `src/sequencer/mod.rs` {#src-sequencer-mod-rs}

- **Lines**: 12 (code: 11, comments: 0, blank: 1)

### `src/sequencer/scheduler.rs` {#src-sequencer-scheduler-rs}

- **Lines**: 300 (code: 243, comments: 0, blank: 57)

### `src/sequencer/track.rs` {#src-sequencer-track-rs}

- **Lines**: 311 (code: 262, comments: 0, blank: 49)

### `src/settings/advanced.rs` {#src-settings-advanced-rs}

- **Lines**: 354 (code: 299, comments: 0, blank: 55)

### `src/settings/audio.rs` {#src-settings-audio-rs}

- **Lines**: 308 (code: 258, comments: 0, blank: 50)

### `src/settings/display.rs` {#src-settings-display-rs}

- **Lines**: 282 (code: 236, comments: 0, blank: 46)

### `src/settings/general.rs` {#src-settings-general-rs}

- **Lines**: 247 (code: 205, comments: 0, blank: 42)

### `src/settings/import_export.rs` {#src-settings-import-export-rs}

- **Lines**: 247 (code: 206, comments: 0, blank: 41)

### `src/settings/keyboard.rs` {#src-settings-keyboard-rs}

- **Lines**: 372 (code: 318, comments: 0, blank: 54)

### `src/settings/library.rs` {#src-settings-library-rs}

- **Lines**: 221 (code: 178, comments: 0, blank: 43)

### `src/settings/midi.rs` {#src-settings-midi-rs}

- **Lines**: 249 (code: 201, comments: 0, blank: 48)

### `src/settings/mixer.rs` {#src-settings-mixer-rs}

- **Lines**: 215 (code: 179, comments: 0, blank: 36)

### `src/settings/mod.rs` {#src-settings-mod-rs}

- **Lines**: 126 (code: 109, comments: 0, blank: 17)

### `src/settings/performance.rs` {#src-settings-performance-rs}

- **Lines**: 245 (code: 207, comments: 0, blank: 38)

### `src/settings/playback.rs` {#src-settings-playback-rs}

- **Lines**: 231 (code: 194, comments: 0, blank: 37)

### `src/settings/privacy.rs` {#src-settings-privacy-rs}

- **Lines**: 244 (code: 202, comments: 0, blank: 42)

### `src/settings/recording.rs` {#src-settings-recording-rs}

- **Lines**: 292 (code: 243, comments: 0, blank: 49)

### `src/settings/sync.rs` {#src-settings-sync-rs}

- **Lines**: 208 (code: 172, comments: 0, blank: 36)

### `src/settings/track.rs` {#src-settings-track-rs}

- **Lines**: 248 (code: 208, comments: 0, blank: 40)

### `src/undo_redo.rs` {#src-undo-redo-rs}

- **Lines**: 51 (code: 47, comments: 0, blank: 4)

### `src/undo_redo/commands.rs` {#src-undo-redo-commands-rs}

- **Lines**: 247 (code: 200, comments: 0, blank: 47)

### `src/undo_redo/controller.rs` {#src-undo-redo-controller-rs}

- **Lines**: 318 (code: 264, comments: 0, blank: 54)

### `src/undo_redo/core.rs` {#src-undo-redo-core-rs}

- **Lines**: 753 (code: 622, comments: 0, blank: 131)

### `src/undo_redo/performance.rs` {#src-undo-redo-performance-rs}

- **Lines**: 357 (code: 288, comments: 0, blank: 69)

### `src/undo_redo/piano_roll.rs` {#src-undo-redo-piano-roll-rs}

- **Lines**: 697 (code: 565, comments: 0, blank: 132)

### `src/undo_redo/serialization.rs` {#src-undo-redo-serialization-rs}

- **Lines**: 200 (code: 163, comments: 0, blank: 37)

### `src/undo_redo/tempo.rs` {#src-undo-redo-tempo-rs}

- **Lines**: 418 (code: 351, comments: 0, blank: 67)

### `src/undo_redo/track.rs` {#src-undo-redo-track-rs}

- **Lines**: 402 (code: 331, comments: 0, blank: 71)

### `src/undo_redo/velocity.rs` {#src-undo-redo-velocity-rs}

- **Lines**: 338 (code: 278, comments: 0, blank: 60)

### `src/windows/mod.rs` {#src-windows-mod-rs}

- **Lines**: 52 (code: 47, comments: 0, blank: 5)

### `src/windows/state.rs` {#src-windows-state-rs}

- **Lines**: 559 (code: 482, comments: 0, blank: 77)

### `tests/commands/mod.rs` {#tests-commands-mod-rs}

- **Lines**: 6 (code: 6, comments: 0, blank: 0)

### `tests/common/assertions.rs` {#tests-common-assertions-rs}

- **Lines**: 152 (code: 135, comments: 0, blank: 17)

### `tests/common/builders.rs` {#tests-common-builders-rs}

- **Lines**: 223 (code: 193, comments: 0, blank: 30)

### `tests/common/database.rs` {#tests-common-database-rs}

- **Lines**: 183 (code: 163, comments: 0, blank: 20)

### `tests/common/fixtures.rs` {#tests-common-fixtures-rs}

- **Lines**: 171 (code: 147, comments: 0, blank: 24)

### `tests/common/mocks.rs` {#tests-common-mocks-rs}

- **Lines**: 234 (code: 200, comments: 0, blank: 34)

### `tests/common/mod.rs` {#tests-common-mod-rs}

- **Lines**: 18 (code: 16, comments: 0, blank: 2)

### `tests/daw_database_integration_test.rs` {#tests-daw-database-integration-test-rs}

- **Lines**: 399 (code: 343, comments: 0, blank: 56)

### `tests/lib.rs` {#tests-lib-rs}

- **Lines**: 16 (code: 15, comments: 0, blank: 1)

### `tests/models_test.rs` {#tests-models-test-rs}

- **Lines**: 2871 (code: 2575, comments: 0, blank: 296)
