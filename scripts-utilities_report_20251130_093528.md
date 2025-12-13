# Project Report: scripts-utilities

> Generated: 2025-11-30 09:37:13
> Path: `/home/dojevou/projects/midi-software-center/scripts`

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
| Total Files | 32 |
| Total Lines | 10,508 |
| Lines of Code | 7,777 |
| Functions | 208 |
| Classes | 19 |
| Avg Pylint Score | 7.85/10 |
| Docstring Coverage | 77.9% |

## Project Statistics

### Files by Extension

| Extension | Count | Lines |
|-----------|-------|-------|
| .py | 24 | 6,990 |
| .rs | 4 | 1,249 |
| .toml | 2 | 27 |
| .ts | 1 | 77 |
| .json | 1 | 2,165 |

## Code Quality

### Pylint Score Distribution

- Excellent (9-10): 8 files
- Good (7-9): 7 files
- Fair (5-7): 6 files
- Needs Work (<5): 2 files

### Files Needing Attention

| File | Score | Issues |
|------|-------|--------|
| `fixes/fix_e0308_pool.py` | 4.3 | 0 |
| `fixes/format_string_fixer.py` | 4.5 | 0 |
| `derive_injector.py` | 5.7 | 0 |
| `fixes/fix_list_files.py` | 5.8 | 0 |
| `fixes/fix_e0308_appstate.py` | 6.0 | 0 |
| `grok/supercharged_grok_reviewer.py` | 6.3 | 0 |
| `analysis/error_parser.py` | 6.3 | 0 |
| `grok/ultra_supercharged_grok_reviewer.py` | 6.4 | 0 |

## Dependencies

### Most Used Imports

| Package | Usage Count |
|---------|-------------|
| `pathlib` | 20 |
| `sys` | 20 |
| `re` | 15 |
| `os` | 12 |
| `collections` | 8 |
| `json` | 8 |
| `psycopg2` | 7 |
| `subprocess` | 6 |
| `time` | 5 |
| `typing` | 4 |
| `datetime` | 4 |
| `httpx` | 4 |
| `concurrent.futures` | 2 |
| `multiprocessing` | 2 |
| `argparse` | 2 |
| `psycopg2.extras` | 2 |
| `traceback` | 2 |
| `csv` | 1 |
| `dataclasses` | 1 |
| `shutil` | 1 |

## File Structure

```
scripts/
├── analysis/
│   ├── analyze_files.py
│   └── error_parser.py
├── analyze-tool/
│   └── src/
│       ├── analyzer.rs
│       └── tag_extractor.rs
├── backups/
│   └── project_backup_20251130_090335.zip
├── fixes/
│   ├── apply_test_fixes.sh
│   ├── auto_fix.sh
│   ├── fix_add_tags_calls.py
│   ├── fix_e0308_appstate.py
│   ├── fix_e0308_calls.sh
│   ├── fix_e0308_pool.py
│   ├── fix_lint.sh
│   ├── fix_list_files.py
│   ├── fix_workflows_imports.sh
│   ├── fix_workflows_manually.sh
│   ├── format_string_fixer.py
│   └── master_fixer.sh
├── grok/
│   ├── grok4-review.sh
│   ├── grok4_project_reviewer.py
│   ├── midi_grok_reviewer.py
│   ├── supercharged_grok_reviewer.py
│   └── ultra_supercharged_grok_reviewer.py
├── import/
│   └── import-collection.sh
├── import-tool/
│   ├── src/
│   │   ├── main.rs
│   │   └── main.rs.backup
│   └── Cargo.toml
├── launch/
│   ├── frontend.sh
│   ├── launch-all.sh
│   ├── launch-daw.sh
│   ├── launch-midi-center.sh
│   ├── launch-pipeline.sh
│   ├── status.sh
│   └── stop-all.sh
├── modules/
│   └── database.sh
├── organization/
│   ├── cleanup_root.sh
│   ├── organize-files.sh
│   ├── organize_docs.sh
│   └── organize_remaining.sh
├── setup/
│   ├── 00_COMPLETE_RESET_AND_SETUP.sh
│   ├── 01_COMPLETE_MCP_RESET.sh
│   ├── 02_SETUP_MCP_CORRECT.sh
│   ├── 03_FULL_SETUP_ALL_SERVERS.sh
│   ├── complete_setup.sh
│   ├── install-launcher.sh
│   ├── setup-agents.sh
│   ├── setup-project-structure.sh
│   └── uninstall-launcher.sh
├── tasks/
├── test-midi-files/
│   ├── src/
│   │   ├── main.rs
│   │   └── main.rs.backup
│   └── Cargo.toml
├── verify/
│   ├── integration_test.sh
│   └── quick_check.sh
├── analyze_filenames.py
├── audit-core-io.sh
├── audit-missing-imports.ts
├── benchmark-comparison.sh
├── BENCHMARK-EXAMPLES.md
└── BENCHMARK-GUIDE.md
    ... and 81 more
```

## TODOs and FIXMEs

*No TODOs or FIXMEs found*

## File Details

### `analysis/analyze_files.py` {#analysis-analyze-files-py}

- **Lines**: 40 (code: 30, comments: 3, blank: 7)

<details>
<summary>Issues (1)</summary>

```
SyntaxError: Could not parse file
```
</details>

#### Source Code

```python
#!/bin/bash

echo "=== COMPREHENSIVE ERROR ANALYSIS ==="
echo "Timestamp: $(date)"
echo

# Total error count
total=$(cargo check --tests -p midi-pipeline 2>&1 | grep -c "error\[E")
echo "Total Errors: $total"
echo

# Error breakdown by type
echo "=== ERROR BREAKDOWN BY TYPE ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E" | sed 's/.*error\[//' | sed 's/\].*//' | sort | uniq -c | sort -nr

echo
echo "=== ERROR BREAKDOWN BY FILE ==="
cargo check --tests -p midi-pipeline 2>&1 | grep -o "pipeline/src-tauri/tests/[a-zA-Z_]*\.rs" | sort | uniq -c | sort -nr

echo
echo "=== TOP E0061 ERROR PATTERNS ==="
cargo check --tests -p midi-pipeline 2>&1 | grep -A 1 "error\[E0061\]" | grep "this function" | sort | uniq -c | sort -nr | head -10

echo
echo "=== TOP E0425 ERROR PATTERNS (missing items) ==="
cargo check --tests -p midi-pipeline 2>&1 | grep "error\[E0425\]" | head -10 | while read line; do
    echo "$line"
done

echo
echo "=== SUMMARY ==="
echo "Session Progress:"
echo "  Initial: 362 errors"
echo "  Current: $total errors"
echo "  Fixed: $((362 - total)) errors ($(( (362 - total) * 100 / 362 ))%)"
echo
echo "Major Accomplishments:"
echo "  ✓ Fixed 62 E0061 errors (121 → 59)"
echo "  ✓ Eliminated all window.clone() parameter issues"
echo "  ✓ Fixed import_single_file_impl argument order in workflows_test.rs"

```

### `analysis/error_parser.py` {#analysis-error-parser-py}

- **Lines**: 261 (code: 206, comments: 20, blank: 35)
- **Pylint Score**: 6.31/10 ⚠️
- **Docstring Coverage**: 71.4%
- **Functions**: __post_init__, __init__, _compile_patterns, parse, _categorize_error, _get_error_type, _extract_affected_items, _generate_report, export_csv, export_json
  *... and 2 more*
- **Classes**: ErrorInfo, QuantumErrorParser

#### Source Code

```python
#!/usr/bin/env python3
"""
Error Parser - Extracts detailed error information from Quantum Analyzer output
Generates a structured CSV for systematic fixing
"""

import re
import json
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Dict, Set
from collections import defaultdict

@dataclass
class ErrorInfo:
    error_id: int
    category: str
    severity: str
    error_type: str
    error_message: str
    file_path: str = ""
    line_number: int = 0
    affected_items: List[str] = None
    
    def __post_init__(self):
        if self.affected_items is None:
            self.affected_items = []

class QuantumErrorParser:
    def __init__(self, errors_file: str):
        self.errors_file = errors_file
        self.errors: List[ErrorInfo] = []
        self.categories: Dict[str, List[ErrorInfo]] = defaultdict(list)
        self.error_patterns = self._compile_patterns()
    
    def _compile_patterns(self) -> Dict[str, re.Pattern]:
        """Compile regex patterns for different error types"""
        return {
            'format_string': re.compile(
                r'invalid reference to positional argument (\d+) \(no arguments were given\)'
            ),
            'missing_type': re.compile(
                r"cannot find type `(\w+)` in module `([^`]+)`"
            ),
            'missing_method': re.compile(
                r"no method named `(\w+)` found for struct `(\w+)`"
            ),
            'unresolved_import': re.compile(
                r"unresolved import `([^`]+)`"
            ),
            'trait_bound': re.compile(
                r"the trait bound `([^`]+)` is not satisfied"
            ),
            'doc_comment': re.compile(
                r'expected outer doc comment'
            ),
            'binary_op': re.compile(
                r"binary operation `([^`]+)` cannot be applied to type `([^`]+)`"
            ),
            'iterator': re.compile(
                r"`([^`]+)` is not an iterator"
            ),
        }
    
    def parse(self) -> Dict[str, any]:
        """Parse the error file and categorize"""
        with open(self.errors_file, 'r') as f:
            content = f.read()
        
        # Extract critical errors
        critical_pattern = r'🔴 CRITICAL \[Build\] Build error\s*\n\s*(.+?)(?=\n\n|🔴|🟡|🟢|$)'
        matches = re.finditer(critical_pattern, content)
        
        error_id = 0
        for match in matches:
            error_message = match.group(1).strip()
            category = self._categorize_error(error_message)
            error_info = ErrorInfo(
                error_id=error_id,
                category=category,
                severity='CRITICAL',
                error_type=self._get_error_type(error_message),
                error_message=error_message,
                affected_items=self._extract_affected_items(error_message)
            )
            self.errors.append(error_info)
            self.categories[category].append(error_info)
            error_id += 1
        
        return self._generate_report()
    
    def _categorize_error(self, message: str) -> str:
        """Categorize error by type"""
        if 'positional argument' in message:
            return 'Format String Errors'
        elif 'cannot find type' in message:
            return 'Missing Type Definitions'
        elif 'cannot find struct' in message or 'cannot find value' in message:
            return 'Missing Type/Value Definitions'
        elif 'unresolved import' in message or 'failed to resolve' in message:
            return 'Unresolved Imports'
        elif 'no method named' in message:
            return 'Missing Repository Methods'
        elif 'no clone' in message or 'cannot be cloned' in message:
            return 'AppState & Clone Issues'
        elif 'the trait bound' in message or 'trait' in message.lower():
            return 'Trait Bound & Type Mismatch'
        elif 'expected outer doc comment' in message or 'doc comment' in message:
            return 'Documentation Comment Errors'
        elif 'iterator' in message or 'Iterator' in message:
            return 'Iterator & Type Conversion'
        else:
            return 'Other/Uncategorized'
    
    def _get_error_type(self, message: str) -> str:
        """Extract specific error type"""
        for error_type, pattern in self.error_patterns.items():
            if pattern.search(message):
                return error_type
        return 'unknown'
    
    def _extract_affected_items(self, message: str) -> List[str]:
        """Extract affected types/methods from error message"""
        items = []
        
        # Extract type names
        type_pattern = r'`(\w+)`'
        items.extend(re.findall(type_pattern, message))
        
        return list(set(items))  # Remove duplicates
    
    def _generate_report(self) -> Dict:
        """Generate summary report"""
        category_summary = {
            cat: len(errors) 
            for cat, errors in self.categories.items()
        }
        
        return {
            'total_errors': len(self.errors),
            'category_summary': category_summary,
            'categories': self.categories,
            'errors': self.errors
        }
    
    def export_csv(self, output_file: str):
        """Export errors as CSV for tracking"""
        import csv
        with open(output_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow([
                'ID', 'Category', 'Error Type', 'Message', 
                'Affected Items', 'Status', 'Priority'
            ])
            for i, error in enumerate(self.errors, 1):
                priority = self._calculate_priority(error.category)
                writer.writerow([
                    i,
                    error.category,
                    error.error_type,
                    error.error_message[:100],
                    '|'.join(error.affected_items),
                    'TODO',
                    priority
                ])
    
    def export_json(self, output_file: str):
        """Export structured error data as JSON"""
        report = self._generate_report()
        
        # Convert categories dict to list for JSON serialization
        categories_list = []
        for category, errors in report['categories'].items():
            categories_list.append({
                'category': category,
                'count': len(errors),
                'errors': [asdict(e) for e in errors]
            })
        
        output = {
            'timestamp': Path(self.errors_file).stat().st_mtime,
            'total_errors': report['total_errors'],
            'summary': report['category_summary'],
            'categories': categories_list
        }
        
        with open(output_file, 'w') as f:
            json.dump(output, f, indent=2, default=str)
    
    def print_summary(self):
        """Print summary to console"""
        print("\n" + "="*60)
        print("QUANTUM ERROR ANALYSIS SUMMARY")
        print("="*60)
        print(f"Total Critical Errors: {len(self.errors)}\n")
        
        print("Errors by Category (Priority Order):")
        print("-" * 60)
        
        priority_order = [
            'Format String Errors',
            'Missing Type Definitions',
            'Unresolved Imports',
            'AppState & Clone Issues',
            'Missing Repository Methods',
            'Trait Bound & Type Mismatch',
            'Iterator & Type Conversion',
            'Documentation Comment Errors',
            'Missing Type/Value Definitions',
            'Other/Uncategorized'
        ]
        
        for priority, category in enumerate(priority_order, 1):
            count = len(self.categories.get(category, []))
            if count > 0:
                print(f"{priority}. {category:.<40} {count:>3} errors")
        
        print("\n" + "="*60)
    
    def _calculate_priority(self, category: str) -> int:
        """Calculate priority (1=highest, 10=lowest)"""
        priorities = {
            'Format String Errors': 1,
            'Missing Type Definitions': 2,
            'Unresolved Imports': 3,
            'AppState & Clone Issues': 4,
            'Missing Repository Methods': 5,
            'Trait Bound & Type Mismatch': 6,
            'Iterator & Type Conversion': 7,
            'Documentation Comment Errors': 8,
            'Missing Type/Value Definitions': 9,
            'Other/Uncategorized': 10
        }
        return priorities.get(category, 10)


if __name__ == '__main__':
    import sys
    
    if len(sys.argv) < 2:
        print("Usage: python3 error_parser.py <errors_file> [output_dir]")
        print("\nExample: python3 error_parser.py eroors ./error_reports")
        sys.exit(1)
    
    errors_file = sys.argv[1]
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path('error_reports')
    output_dir.mkdir(exist_ok=True)
    
    parser = QuantumErrorParser(errors_file)
    report = parser.parse()
    
    # Export reports
    parser.export_csv(output_dir / 'errors.csv')
    parser.export_json(output_dir / 'errors.json')
    
    # Print summary
    parser.print_summary()
    
    print(f"\n✅ Reports generated in: {output_dir}/")
    print(f"   - errors.csv (for spreadsheet review)")
    print(f"   - errors.json (for programmatic use)")

```

### `analyze-tool/src/analyzer.rs` {#analyze-tool-src-analyzer-rs}

- **Lines**: 347 (code: 287, comments: 0, blank: 60)

#### Source Code

```rust
   /// MIDI Analyzer - Trusty Module
   ///
   /// Extracts complete musical metadata from MIDI files

use midly::{Smf, Timing, MetaMessage, TrackEventKind, MidiMessage};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MidiAnalysis {
    pub bpm: Option<f64>,
    pub bpm_confidence: f32,
    pub has_tempo_changes: bool,

    pub key_signature: Option<String>,
    pub key_confidence: f32,

    pub time_sig_num: i16,
    pub time_sig_den: i16,

    pub duration_seconds: f64,
    pub duration_ticks: i64,

    pub total_notes: i32,
    pub unique_pitches: i32,
    pub pitch_range_min: i16,
    pub pitch_range_max: i16,

    pub avg_velocity: f64,
    pub polyphony_max: i16,
    pub polyphony_avg: f64,

    pub is_monophonic: bool,
    pub is_polyphonic: bool,
    pub is_percussive: bool,
    pub has_chords: bool,

    pub note_density: f64,
    pub instruments: Vec<String>,
}

impl Default for MidiAnalysis {
    fn default() -> Self {
        Self {
            bpm: None,
            bpm_confidence: 0.0,
            has_tempo_changes: false,
            key_signature: None,
            key_confidence: 0.0,
            time_sig_num: 4,
            time_sig_den: 4,
            duration_seconds: 0.0,
            duration_ticks: 0,
            total_notes: 0,
            unique_pitches: 0,
            pitch_range_min: 127,
            pitch_range_max: 0,
            avg_velocity: 0.0,
            polyphony_max: 0,
            polyphony_avg: 0.0,
            is_monophonic: false,
            is_polyphonic: false,
            is_percussive: false,
            has_chords: false,
            note_density: 0.0,
            instruments: Vec::new(),
        }
    }
}

pub fn analyze_midi(data: &[u8]) -> Result<MidiAnalysis, String> {
    let smf = Smf::parse(data).map_err(|e| format!("Parse error: {}", e))?;

    let mut analysis = MidiAnalysis::default();

    // Get PPQ
    let ppq = match smf.header.timing {
        Timing::Metrical(tpb) => tpb.as_int() as i64,
        Timing::Timecode(_, _) => 480,
    };

    // Extract tempo
    let tempo_us = extract_tempo(&smf);
    if tempo_us > 0 {
        analysis.bpm = Some(60_000_000.0 / tempo_us as f64);
        analysis.bpm_confidence = 1.0;
    }

    // Extract time signature
    let (num, den) = extract_time_signature(&smf);
    analysis.time_sig_num = num;
    analysis.time_sig_den = den;

    // Collect all notes
    let notes = collect_notes(&smf);
    analysis.total_notes = notes.len() as i32;

    if !notes.is_empty() {
        // Pitch analysis
        let unique_pitches: std::collections::HashSet<u8> =
            notes.iter().map(|(_, pitch, _)| *pitch).collect();
        analysis.unique_pitches = unique_pitches.len() as i32;

        analysis.pitch_range_min = notes.iter().map(|(_, p, _)| *p).min().unwrap_or(127) as i16;
        analysis.pitch_range_max = notes.iter().map(|(_, p, _)| *p).max().unwrap_or(0) as i16;

        // Velocity analysis
        let total_vel: u32 = notes.iter().map(|(_, _, v)| *v as u32).sum();
        analysis.avg_velocity = total_vel as f64 / notes.len() as f64;

        // Polyphony analysis
        analysis.polyphony_max = calculate_max_polyphony(&notes);
        analysis.polyphony_avg = calculate_avg_polyphony(&notes);

        analysis.is_monophonic = analysis.polyphony_max <= 1;
        analysis.is_polyphonic = analysis.polyphony_max >= 3;
        analysis.has_chords = analysis.polyphony_max >= 3;

        // Check for percussion (channel 10 or low pitch range)
        analysis.is_percussive = notes.iter().any(|(_, pitch, _)| *pitch < 36);

        // Duration
        let max_tick = notes.iter().map(|(tick, _, _)| *tick).max().unwrap_or(0);
        analysis.duration_ticks = max_tick as i64;

        if let Some(bpm) = analysis.bpm {
            if bpm > 0.0 {
                analysis.duration_seconds = (max_tick as f64 / ppq as f64) * (60.0 / bpm);
            }
        }

        // Note density
        if analysis.duration_seconds > 0.0 {
            analysis.note_density = analysis.total_notes as f64 / analysis.duration_seconds;
        }

        // Key detection
        if let Some((key, confidence)) = detect_key(&notes) {
            analysis.key_signature = Some(key);
            analysis.key_confidence = confidence;
        }

        // Instruments
        analysis.instruments = extract_instruments(&smf);
    }

    Ok(analysis)
}

fn extract_tempo(smf: &Smf) -> u32 {
    for track in &smf.tracks {
        for event in track {
            if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event.kind {
                return tempo.as_int();
            }
        }
    }
    500_000 // Default 120 BPM
}

fn extract_time_signature(smf: &Smf) -> (i16, i16) {
    for track in &smf.tracks {
        for event in track {
            if let TrackEventKind::Meta(MetaMessage::TimeSignature(num, den, _, _)) = event.kind {
                return (num as i16, 2_i16.pow(den as u32));
            }
        }
    }
    (4, 4)
}

fn collect_notes(smf: &Smf) -> Vec<(u32, u8, u8)> {
    let mut notes = Vec::new();

    for track in &smf.tracks {
        let mut tick = 0u32;
        for event in track {
            tick += event.delta.as_int();

            if let TrackEventKind::Midi { message, .. } = event.kind {
                if let MidiMessage::NoteOn { key, vel } = message {
                    if vel.as_int() > 0 {
                        notes.push((tick, key.as_int(), vel.as_int()));
                    }
                }
            }
        }
    }

    notes
}

fn calculate_max_polyphony(notes: &[(u32, u8, u8)]) -> i16 {
    let mut note_states: HashMap<u32, Vec<u8>> = HashMap::new();

    for (tick, pitch, _) in notes {
        note_states.entry(*tick).or_insert_with(Vec::new).push(*pitch);
    }

    note_states.values()
        .map(|pitches| {
            pitches.iter().collect::<std::collections::HashSet<_>>().len() as i16
        })
        .max()
        .unwrap_or(0)
}

fn calculate_avg_polyphony(notes: &[(u32, u8, u8)]) -> f64 {
    if notes.is_empty() {
        return 0.0;
    }

    let max = calculate_max_polyphony(notes);
    max as f64 * 0.6 // Rough estimate
}

fn detect_key(notes: &[(u32, u8, u8)]) -> Option<(String, f32)> {
    if notes.is_empty() {
        return None;
    }

    // Krumhansl-Schmuckler algorithm
    const MAJOR_PROFILE: [f32; 12] = [
        6.35, 2.23, 3.48, 2.33, 4.38, 4.09,
        2.52, 5.19, 2.39, 3.66, 2.29, 2.88
    ];

    const MINOR_PROFILE: [f32; 12] = [
        6.33, 2.68, 3.52, 5.38, 2.60, 3.53,
        2.54, 4.75, 3.98, 2.69, 3.34, 3.17
    ];

    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F",
        "F#", "G", "G#", "A", "A#", "B"
    ];

    // Count pitch class occurrences
    let mut pitch_class_counts = [0u32; 12];
    for (_, pitch, _) in notes {
        pitch_class_counts[(pitch % 12) as usize] += 1;
    }

    // Normalize
    let total: u32 = pitch_class_counts.iter().sum();
    if total == 0 {
        return None;
    }

    let distribution: Vec<f32> = pitch_class_counts
        .iter()
        .map(|&c| c as f32 / total as f32)
        .collect();

    // Find best correlation
    let mut best_key = String::new();
    let mut best_corr = -1.0f32;

    for tonic in 0..12 {
        // Test major
        let major_corr = correlate(&distribution, &rotate_profile(&MAJOR_PROFILE, tonic));
        if major_corr > best_corr {
            best_corr = major_corr;
            best_key = NOTE_NAMES[tonic].to_string();
        }

        // Test minor
        let minor_corr = correlate(&distribution, &rotate_profile(&MINOR_PROFILE, tonic));
        if minor_corr > best_corr {
            best_corr = minor_corr;
            best_key = format!("{}m", NOTE_NAMES[tonic]);
        }
    }

    Some((best_key, best_corr.max(0.0).min(1.0)))
}

fn correlate(a: &[f32], b: &[f32]) -> f32 {
    let mean_a: f32 = a.iter().sum::<f32>() / a.len() as f32;
    let mean_b: f32 = b.iter().sum::<f32>() / b.len() as f32;

    let mut numerator = 0.0f32;
    let mut denom_a = 0.0f32;
    let mut denom_b = 0.0f32;

    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        numerator += da * db;
        denom_a += da * da;
        denom_b += db * db;
    }

    if denom_a == 0.0 || denom_b == 0.0 {
        return 0.0;
    }

    numerator / (denom_a * denom_b).sqrt()
}

fn rotate_profile(profile: &[f32; 12], steps: usize) -> [f32; 12] {
    let mut rotated = [0.0f32; 12];
    for i in 0..12 {
        rotated[i] = profile[(i + steps) % 12];
    }
    rotated
}

fn extract_instruments(smf: &Smf) -> Vec<String> {
    let mut instruments = Vec::new();

    for track in &smf.tracks {
        for event in track {
            if let TrackEventKind::Midi { message, .. } = event.kind {
                if let MidiMessage::ProgramChange { program } = message {
                    let instrument = get_instrument_name(program.as_int());
                    if !instruments.contains(&instrument) {
                        instruments.push(instrument);
                    }
                }
            }
        }
    }

    instruments
}

fn get_instrument_name(program: u8) -> String {
    match program {
        0..=7 => "Piano".to_string(),
        8..=15 => "Chromatic Percussion".to_string(),
        16..=23 => "Organ".to_string(),
        24..=31 => "Guitar".to_string(),
        32..=39 => "Bass".to_string(),
        40..=47 => "Strings".to_string(),
        48..=55 => "Ensemble".to_string(),
        56..=63 => "Brass".to_string(),
        64..=71 => "Reed".to_string(),
        72..=79 => "Pipe".to_string(),
        80..=87 => "Synth Lead".to_string(),
        88..=95 => "Synth Pad".to_string(),
        96..=103 => "Synth Effects".to_string(),
        104..=111 => "Ethnic".to_string(),
        112..=119 => "Percussive".to_string(),
        120..=127 => "Sound Effects".to_string(),
        _ => "Unknown".to_string(),
    }
}

```

### `analyze-tool/src/tag_extractor.rs` {#analyze-tool-src-tag-extractor-rs}

- **Lines**: 151 (code: 122, comments: 0, blank: 29)

#### Source Code

```rust
///! Tag Extractor - Trusty Module
   ///
   /// Extracts tags from file paths and folder names

use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExtractedTags {
    pub manufacturer: Option<String>,
    pub collection: Option<String>,
    pub genres: Vec<String>,
    pub category: Option<String>,
    pub bpm_hint: Option<i16>,
    pub descriptors: Vec<String>,
}

pub fn extract_tags_from_path(filepath: &str) -> ExtractedTags {
    let path = Path::new(filepath);
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    let mut tags = ExtractedTags {
        manufacturer: None,
        collection: None,
        genres: Vec::new(),
        category: None,
        bpm_hint: None,
        descriptors: Vec::new(),
    };

    // Extract manufacturer
    tags.manufacturer = detect_manufacturer(&components);

    // Extract genres
    tags.genres = extract_genres(&components);

    // Extract category
    tags.category = detect_category(&components);

    // Extract BPM hint
    tags.bpm_hint = extract_bpm_hint(filepath);

    // Extract descriptors
    tags.descriptors = extract_descriptors(&components);

    tags
}

fn detect_manufacturer(components: &[&str]) -> Option<String> {
    let manufacturers = [
        "DMS", "Loopmasters", "Vengeance", "Sample Magic", "Singomakers",
        "Hy2rogen", "Production Master", "Function Loops", "Audentity",
        "Chill Samples", "Class A Samples", "Diginoiz", "Prime Loops",
        "Producer Loops", "Sonic Academy", "Black Octopus", "MIDI Focus",
        "Zenhiser"
    ];

    for component in components {
        for mfr in &manufacturers {
            if component.contains(mfr) {
                return Some(mfr.to_string());
            }
        }
    }

    None
}

fn extract_genres(components: &[&str]) -> Vec<String> {
    let genres = [
        "Trance", "House", "Techno", "Drum and Bass", "DnB", "Dubstep",
        "Hardcore", "Progressive", "Electro", "Tech House", "Deep House",
        "Psytrance", "Minimal", "Ambient", "Chill", "Lo-Fi", "Trap",
        "Future Bass", "Liquid"
    ];

    let mut found = Vec::new();
    let path_str = components.join("/").to_lowercase();

    for genre in &genres {
        if path_str.contains(&genre.to_lowercase()) {
            found.push(genre.to_string());
        }
    }

    found
}

fn detect_category(components: &[&str]) -> Option<String> {
    let categories = [
        ("Bass", vec!["bass", "sub", "reese"]),
        ("Melody", vec!["melody", "lead", "melodic"]),
        ("Pad", vec!["pad", "atmosphere", "ambient"]),
        ("Chord", vec!["chord", "progression"]),
        ("Arp", vec!["arp", "arpegg"]),
        ("Drum", vec!["drum", "kick", "snare", "hat", "perc"]),
        ("FX", vec!["fx", "effect", "riser", "sweep"]),
    ];

    let path_str = components.join("/").to_lowercase();

    for (category, keywords) in &categories {
        for keyword in keywords {
            if path_str.contains(keyword) {
                return Some(category.to_string());
            }
        }
    }

    None
}

fn extract_bpm_hint(filepath: &str) -> Option<i16> {
    // Look for patterns like "140 BPM", "140BPM", "140bpm"
    let re = regex::Regex::new(r"(\d{2,3})\s*[Bb][Pp][Mm]").ok()?;

    if let Some(cap) = re.captures(filepath) {
        return cap[1].parse().ok();
    }

    // Also check for folder names like "140 BPM"
    let re2 = regex::Regex::new(r"/(\d{2,3})\s*[Bb][Pp][Mm]/").ok()?;
    if let Some(cap) = re2.captures(filepath) {
        return cap[1].parse().ok();
    }

    None
}

fn extract_descriptors(components: &[&str]) -> Vec<String> {
    let descriptors = [
        "Hard", "Soft", "Deep", "Bright", "Dark", "Warm", "Cold",
        "Fat", "Thin", "Punchy", "Smooth", "Rough", "Clean", "Dirty",
        "Melodic", "Atmospheric", "Epic", "Minimal", "Complex",
        "Liquid", "Uplifting", "Driving", "Bouncy", "Rolling",
        "Variation", "Straight", "Shuffle", "Triplet", "Syncopation"
    ];

    let mut found = Vec::new();
    let path_str = components.join("/").to_lowercase();

    for desc in &descriptors {
        if path_str.contains(&desc.to_lowercase()) && !found.contains(&desc.to_string()) {
            found.push(desc.to_string());
        }
    }

    found
}

```

### `analyze_filenames.py` {#analyze-filenames-py}

- **Lines**: 569 (code: 427, comments: 61, blank: 81)
- **Pylint Score**: 9.47/10 ✅
- **Docstring Coverage**: 78.6%
- **Functions**: extract_bpm, extract_key, extract_time_signature, extract_instruments, extract_genres, extract_patterns, extract_drum_elements, analyze_file, process_batch, generate_report
  *... and 3 more*
- **Classes**: FilenameAnalyzer

#### Source Code

```python
#!/usr/bin/env python3
"""
Analyze MIDI filenames to extract instruments, genres, keys, BPM, and descriptions
"""

import re
import sys
import json
from pathlib import Path
from collections import Counter, defaultdict
from concurrent.futures import ProcessPoolExecutor
import multiprocessing as mp

class FilenameAnalyzer:
    def __init__(self):
        # Counters for all metadata
        self.instruments = Counter()
        self.genres = Counter()
        self.keys = Counter()
        self.bpms = Counter()
        self.patterns = Counter()
        self.time_signatures = Counter()
        self.drum_elements = Counter()

        self.total_files = 0
        self.drum_files = 0

    def merge(self, other):
        """Merge results from another analyzer"""
        self.instruments.update(other.instruments)
        self.genres.update(other.genres)
        self.keys.update(other.keys)
        self.bpms.update(other.bpms)
        self.patterns.update(other.patterns)
        self.time_signatures.update(other.time_signatures)
        self.drum_elements.update(other.drum_elements)
        self.total_files += other.total_files
        self.drum_files += other.drum_files

def extract_bpm(filename):
    """Extract BPM from filename"""
    # Pattern 1: _120bpm_ or _120_bpm_
    match = re.search(r'[_\-](\d{2,3})[-_]?bpm[_\-]', filename, re.IGNORECASE)
    if match:
        bpm = int(match.group(1))
        if 30 <= bpm <= 300:
            return bpm

    # Pattern 2: _120_ (number between underscores)
    match = re.search(r'[_\-](\d{2,3})[_\-]', filename)
    if match:
        bpm = int(match.group(1))
        if 30 <= bpm <= 300:
            return bpm

    # Pattern 3: starts with number
    match = re.search(r'^(\d{2,3})[-_\.]', filename)
    if match:
        bpm = int(match.group(1))
        if 30 <= bpm <= 300:
            return bpm

    return None

def extract_key(filename):
    """Extract musical key from filename"""
    # Major keys: C, C#, Db, etc.
    match = re.search(r'[_\-]([A-G][#b]?)(?:maj|major)?[_\-]', filename, re.IGNORECASE)
    if match:
        return match.group(1).upper()

    # Minor keys: Cm, C#m, Dbm, Amin, etc.
    match = re.search(r'[_\-]([A-G][#b]?)m(?:in|inor)?[_\-]', filename, re.IGNORECASE)
    if match:
        return match.group(1).upper() + 'm'

    return None

def extract_time_signature(filename):
    """Extract time signature from filename"""
    lower = filename.lower()

    if 'threefour' in lower or '3-4' in lower or '3_4' in lower:
        return '3/4'
    if 'sixeight' in lower or '6-8' in lower or '6_8' in lower:
        return '6/8'
    if 'fivefour' in lower or '5-4' in lower or '5_4' in lower:
        return '5/4'
    if 'seveneight' in lower or '7-8' in lower or '7_8' in lower:
        return '7/8'

    return None

def extract_instruments(filename, path):
    """Extract instrument names from filename and path"""
    text = (filename + ' ' + path).lower()
    instruments = []

    # Drums
    if 'drum' in text and 'syndrome' not in text:
        instruments.append('drums')
    if 'percussion' in text:
        instruments.append('percussion')
    if 'snare' in text:
        instruments.append('snare')
    if 'kick' in text:
        instruments.append('kick')
    if 'hat' in text or 'hihat' in text or 'hi-hat' in text:
        instruments.append('hat')
    if 'cymbal' in text:
        instruments.append('cymbal')
    if 'tom' in text and 'atom' not in text and 'custom' not in text and 'bottom' not in text:
        instruments.append('tom')
    if 'ride' in text:
        instruments.append('ride')
    if 'crash' in text:
        instruments.append('crash')

    # Bass
    if 'bass' in text and 'bass drum' not in text and 'double bass' not in text:
        instruments.append('bass')

    # Synth
    if 'synth' in text:
        instruments.append('synth')
    if 'pad' in text and 'ipad' not in text:
        instruments.append('pad')
    if 'lead' in text:
        instruments.append('lead')
    if 'arp' in text and 'sharp' not in text and 'harp' not in text:
        instruments.append('arp')

    # Keys
    if 'piano' in text:
        instruments.append('piano')
    if 'organ' in text and 'organic' not in text and 'organize' not in text:
        instruments.append('organ')
    if 'chord' in text:
        instruments.append('chords')
    if 'keys' in text or 'keyboard' in text:
        instruments.append('keys')

    # Strings & Brass
    if 'string' in text:
        instruments.append('strings')
    if 'brass' in text:
        instruments.append('brass')
    if 'violin' in text or 'viola' in text or 'cello' in text:
        instruments.append('strings')
    if 'trumpet' in text or 'trombone' in text or 'horn' in text:
        instruments.append('brass')

    # Guitar
    if 'guitar' in text:
        instruments.append('guitar')

    # Vocals
    if 'vocal' in text or 'voice' in text or 'choir' in text:
        instruments.append('vocals')

    # FX
    if 'fx' in text or 'effect' in text or 'sfx' in text:
        instruments.append('fx')

    return instruments

def extract_genres(filename, path):
    """Extract genres from filename and path"""
    text = (filename + ' ' + path).lower()
    genres = []

    # Electronic
    if 'house' in text:
        genres.append('house')
    if 'techno' in text:
        genres.append('techno')
    if 'trance' in text:
        genres.append('trance')
    if 'dubstep' in text:
        genres.append('dubstep')
    if 'dnb' in text or ('drum' in text and 'bass' in text and 'drum bass' not in text):
        genres.append('dnb')
    if 'jungle' in text:
        genres.append('jungle')
    if 'breakbeat' in text or 'breaks' in text:
        genres.append('breakbeat')
    if 'garage' in text:
        genres.append('garage')
    if 'glitch' in text:
        genres.append('glitch')
    if 'ambient' in text:
        genres.append('ambient')
    if 'edm' in text:
        genres.append('edm')
    if 'electro' in text:
        genres.append('electro')

    # Hip-Hop & Urban
    if 'hiphop' in text or 'hip-hop' in text or 'hip hop' in text:
        genres.append('hip-hop')
    if 'trap' in text:
        genres.append('trap')
    if 'rnb' in text or 'r&b' in text or 'r and b' in text:
        genres.append('rnb')

    # Rock & Metal
    if 'rock' in text:
        genres.append('rock')
    if 'metal' in text:
        genres.append('metal')
    if 'punk' in text:
        genres.append('punk')
    if 'blues' in text:
        genres.append('blues')
    if 'funk' in text and 'funky' not in text:
        genres.append('funk')

    # Jazz
    if 'jazz' in text:
        genres.append('jazz')
    if 'fusion' in text:
        genres.append('fusion')
    if 'swing' in text:
        genres.append('swing')

    # World
    if 'latin' in text:
        genres.append('latin')
    if 'africa' in text:
        genres.append('african')
    if 'asia' in text:
        genres.append('asian')
    if 'world' in text:
        genres.append('world')
    if 'ethnic' in text or 'ethno' in text:
        genres.append('ethnic')
    if 'reggae' in text:
        genres.append('reggae')
    if 'caribbean' in text:
        genres.append('caribbean')

    # Other
    if 'pop' in text:
        genres.append('pop')
    if 'disco' in text:
        genres.append('disco')
    if 'progressive' in text:
        genres.append('progressive')
    if 'industrial' in text:
        genres.append('industrial')
    if 'indie' in text:
        genres.append('indie')

    return genres

def extract_patterns(filename, path):
    """Extract pattern types from filename and path"""
    text = (filename + ' ' + path).lower()
    patterns = []

    if 'fill' in text:
        patterns.append('fill')
    if 'groove' in text:
        patterns.append('groove')
    if 'intro' in text:
        patterns.append('intro')
    if 'outro' in text or 'ending' in text:
        patterns.append('ending')
    if 'breakdown' in text:
        patterns.append('breakdown')
    if 'turnaround' in text:
        patterns.append('turnaround')
    if 'verse' in text:
        patterns.append('verse')
    if 'chorus' in text:
        patterns.append('chorus')
    if 'bridge' in text:
        patterns.append('bridge')
    if 'loop' in text:
        patterns.append('loop')
    if 'one-shot' in text or 'oneshot' in text or 'one shot' in text:
        patterns.append('one-shot')
    if 'build' in text and 'building' not in text:
        patterns.append('build')
    if 'drop' in text:
        patterns.append('drop')

    return patterns

def extract_drum_elements(filename, path):
    """Extract drum-specific elements"""
    text = (filename + ' ' + path).lower()
    elements = []

    # Cymbals
    if 'crash' in text:
        elements.append('crash')
    if 'ride' in text:
        elements.append('ride')
    if 'china' in text:
        elements.append('china')
    if 'splash' in text:
        elements.append('splash')

    # Hi-hats
    if 'closed' in text and ('hat' in text or 'hihat' in text):
        elements.append('closed-hat')
    if 'open' in text and ('hat' in text or 'hihat' in text):
        elements.append('open-hat')
    if 'pedal' in text and ('hat' in text or 'hihat' in text):
        elements.append('pedal-hat')

    # Techniques
    if 'ghost' in text:
        elements.append('ghost-notes')
    if 'double' in text and 'bass' in text:
        elements.append('double-bass')
    if 'flam' in text:
        elements.append('flam')
    if 'roll' in text:
        elements.append('roll')

    # Feel
    if 'swing' in text:
        elements.append('swing')
    if 'shuffle' in text:
        elements.append('shuffle')
    if 'triplet' in text:
        elements.append('triplet')
    if 'straight' in text:
        elements.append('straight')

    return elements

def analyze_file(filepath):
    """Analyze a single file"""
    analyzer = FilenameAnalyzer()
    analyzer.total_files = 1

    path = filepath
    filename = Path(filepath).name.lower()

    # Extract BPM
    bpm = extract_bpm(filename)
    if bpm:
        analyzer.bpms[bpm] += 1

    # Extract key
    key = extract_key(filename)
    if key:
        analyzer.keys[key] += 1

    # Extract time signature
    ts = extract_time_signature(filename)
    if ts:
        analyzer.time_signatures[ts] += 1

    # Extract instruments
    instruments = extract_instruments(filename, path)
    for inst in instruments:
        analyzer.instruments[inst] += 1

    # Extract genres
    genres = extract_genres(filename, path)
    for genre in genres:
        analyzer.genres[genre] += 1

    # Extract patterns
    patterns = extract_patterns(filename, path)
    for pattern in patterns:
        analyzer.patterns[pattern] += 1

    # Check if drum file
    is_drum = any(inst in ['drums', 'percussion', 'snare', 'kick', 'hat'] for inst in instruments)
    if is_drum:
        analyzer.drum_files = 1
        elements = extract_drum_elements(filename, path)
        for elem in elements:
            analyzer.drum_elements[elem] += 1

    return analyzer

def process_batch(filepaths):
    """Process a batch of files"""
    batch_analyzer = FilenameAnalyzer()
    for filepath in filepaths:
        result = analyze_file(filepath)
        batch_analyzer.merge(result)
    return batch_analyzer

def generate_report(analyzer, output_path):
    """Generate markdown report"""
    total = analyzer.total_files

    lines = []
    lines.append("# Complete MIDI Collection Analysis\n")
    lines.append(f"**Total Files Analyzed:** {total:,}\n")
    lines.append(f"**Drum Files:** {analyzer.drum_files:,} ({analyzer.drum_files/total*100:.1f}%)\n")
    lines.append("\n---\n\n")

    # ALL Instruments (no limit)
    lines.append("## ALL Instruments Found\n\n")
    lines.append("| Instrument | Count | Percentage |\n")
    lines.append("|------------|-------|------------|\n")
    for inst, count in analyzer.instruments.most_common():
        lines.append(f"| {inst} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL Genres (no limit)
    lines.append("## ALL Genres Found\n\n")
    lines.append("| Genre | Count | Percentage |\n")
    lines.append("|-------|-------|------------|\n")
    for genre, count in analyzer.genres.most_common():
        lines.append(f"| {genre} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL Patterns (no limit)
    lines.append("## ALL Pattern Types\n\n")
    lines.append("| Pattern | Count | Percentage |\n")
    lines.append("|---------|-------|------------|\n")
    for pattern, count in analyzer.patterns.most_common():
        lines.append(f"| {pattern} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL Musical Keys (no limit)
    lines.append("## ALL Musical Keys Found\n\n")
    lines.append("| Key | Count | Percentage |\n")
    lines.append("|-----|-------|------------|\n")
    for key, count in analyzer.keys.most_common():
        lines.append(f"| {key} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # ALL BPM Values (no limit)
    lines.append("## ALL BPM Values Found\n\n")
    lines.append("| BPM | Count | Percentage |\n")
    lines.append("|-----|-------|------------|\n")
    for bpm, count in analyzer.bpms.most_common():
        lines.append(f"| {bpm} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # BPM Ranges
    lines.append("## BPM Ranges Summary\n\n")
    bpm_ranges = defaultdict(int)
    for bpm, count in analyzer.bpms.items():
        if 30 <= bpm <= 60:
            bpm_ranges['Very Slow (30-60)'] += count
        elif 61 <= bpm <= 90:
            bpm_ranges['Slow (61-90)'] += count
        elif 91 <= bpm <= 120:
            bpm_ranges['Mid-Tempo (91-120)'] += count
        elif 121 <= bpm <= 140:
            bpm_ranges['Upbeat (121-140)'] += count
        elif 141 <= bpm <= 180:
            bpm_ranges['Fast (141-180)'] += count
        elif 181 <= bpm <= 300:
            bpm_ranges['Very Fast (181-300)'] += count

    lines.append("| BPM Range | Count | Percentage |\n")
    lines.append("|-----------|-------|------------|\n")
    for range_name, count in sorted(bpm_ranges.items(), key=lambda x: -x[1]):
        lines.append(f"| {range_name} | {count:,} | {count/total*100:.2f}% |\n")
    lines.append("\n")

    # Time Signatures
    if analyzer.time_signatures:
        lines.append("## Time Signatures Found\n\n")
        lines.append("| Time Signature | Count | Percentage |\n")
        lines.append("|----------------|-------|------------|\n")
        for ts, count in analyzer.time_signatures.most_common():
            lines.append(f"| {ts} | {count:,} | {count/total*100:.2f}% |\n")
        lines.append("\n")

    # ALL Drum Elements (no limit)
    if analyzer.drum_elements:
        lines.append("## ALL Drum Elements & Techniques\n\n")
        lines.append("| Element | Count | Percentage (of drum files) |\n")
        lines.append("|---------|-------|---------------------------|\n")
        drum_total = analyzer.drum_files
        for elem, count in analyzer.drum_elements.most_common():
            lines.append(f"| {elem} | {count:,} | {count/drum_total*100:.2f}% |\n")
        lines.append("\n")

    # Write to file
    with open(output_path, 'w') as f:
        f.writelines(lines)

def main():
    import time

    print("MIDI Filename Analysis")
    print("=" * 50)

    # Read file list from stdin or find files
    if not sys.stdin.isatty():
        print("Reading filenames from stdin...")
        filepaths = [line.strip() for line in sys.stdin if line.strip()]
    else:
        print("Error: Please pipe file list to this script")
        print("Usage: find /path -name '*.mid' | python3 analyze_filenames.py")
        sys.exit(1)

    total_files = len(filepaths)
    print(f"Total files to analyze: {total_files:,}")

    # Determine number of workers
    num_workers = mp.cpu_count()
    print(f"Using {num_workers} parallel workers\n")

    # Split into batches
    batch_size = max(1, total_files // (num_workers * 10))
    batches = [filepaths[i:i+batch_size] for i in range(0, total_files, batch_size)]
    print(f"Processing in {len(batches)} batches of ~{batch_size} files each\n")

    # Process in parallel
    start_time = time.time()
    main_analyzer = FilenameAnalyzer()

    with ProcessPoolExecutor(max_workers=num_workers) as executor:
        futures = [executor.submit(process_batch, batch) for batch in batches]

        completed = 0
        for future in futures:
            result = future.result()
            main_analyzer.merge(result)
            completed += 1

            if completed % 10 == 0 or completed == len(futures):
                elapsed = time.time() - start_time
                rate = main_analyzer.total_files / elapsed
                pct = completed / len(futures) * 100
                print(f"Progress: {pct:.1f}% ({main_analyzer.total_files:,}/{total_files:,} files) - {rate:.0f} files/sec")

    elapsed = time.time() - start_time
    rate = total_files / elapsed

    print(f"\nAnalysis complete!")
    print(f"Total files: {total_files:,}")
    print(f"Time: {elapsed:.1f}s")
    print(f"Rate: {rate:.0f} files/sec")
    print()

    # Generate report
    output_path = "COMPLETE_COLLECTION_ANALYSIS.md"
    print(f"Generating report to {output_path}...")
    generate_report(main_analyzer, output_path)
    print(f"Done! Report saved to {output_path}")

    # Save complete JSON data for comprehensive reporting
    json_path = "COMPLETE_COLLECTION_ANALYSIS.json"
    print(f"\nSaving complete JSON data to {json_path}...")
    json_data = {
        'total_files': main_analyzer.total_files,
        'drum_files': main_analyzer.drum_files,
        'instruments': dict(main_analyzer.instruments.most_common()),
        'genres': dict(main_analyzer.genres.most_common()),
        'patterns': dict(main_analyzer.patterns.most_common()),
        'keys': dict(main_analyzer.keys.most_common()),
        'bpms': dict(sorted(main_analyzer.bpms.items())),
        'time_signatures': dict(main_analyzer.time_signatures.most_common()),
        'drum_elements': dict(main_analyzer.drum_elements.most_common()),
    }

    with open(json_path, 'w') as f:
        json.dump(json_data, f, indent=2)

    print(f"Complete JSON data saved! ({len(json_data['instruments'])} instruments, {len(json_data['genres'])} genres, {len(json_data['bpms'])} BPM values)")

if __name__ == "__main__":
    main()

```

### `audit-missing-imports.ts` {#audit-missing-imports-ts}

- **Lines**: 77 (code: 69, comments: 0, blank: 8)

#### Source Code

```typescript
import fs from 'fs';
import path from 'path';
import { parse } from 'acorn';
import { simple } from 'acorn-walk';

const projectRoot = path.resolve('.');
const componentFiles = [
  'app/src/lib/windows/MixerWindow.svelte',
  'app/src/lib/windows/DAWWindow.svelte',
  'app/src/lib/windows/DatabaseWindow.svelte',
  'app/src/lib/windows/PipelineWindow.svelte',
  'app/src/lib/components/StatusBar.svelte'
];

function findFile(pathToCheck: string): boolean {
  try {
    fs.accessSync(pathToCheck, fs.constants.F_OK);
    return true;
  } catch {
    return false;
  }
}

function auditFile(filePath: string) {
  const code = fs.readFileSync(filePath, 'utf8');
  const ast = parse(code, { 
    ecmaVersion: 2020, 
    sourceType: 'module',
    onInsertedSemicolon: true 
  });

  const missingImports = [];

  simple(ast, {
    ImportDeclaration(node) {
      if (node.source && node.source.value) {
        const importPath = node.source.value;
        // Check if it's a relative import or absolute
        if (importPath.startsWith('$lib/')) {
          const fullPath = path.join(projectRoot, importPath.replace('$lib', 'src/lib'));
          if (!findFile(fullPath + '.ts') && !findFile(fullPath + '.js') && !findFile(fullPath + '.svelte')) {
            missingImports.push({
              type: 'module',
              name: importPath,
              node
            });
          }
        } else if (importPath.startsWith('@tauri-apps/')) {
          // Check if package.json has it
          const packageJson = JSON.parse(fs.readFileSync(path.join(projectRoot, 'app/package.json'), 'utf8'));
          if (!packageJson.dependencies[importPath] && !packageJson.devDependencies[importPath]) {
            missingImports.push({
              type: 'package',
              name: importPath,
              node
            });
          }
        }
      }
    },
    ImportSpecifier(node) {
      // Check if the imported name is exported from the module
      // This is more complex, skip for now or implement simple check
    }
  });

  if (missingImports.length > 0) {
    console.log(`\nFile: ${filePath}`);
    missingImports.forEach(missing => {
      console.log(`Missing: ${missing.type} - ${missing.name}`);
    });
  }
}

componentFiles.forEach(auditFile);

console.log('\nAudit complete. Check the output above for missing dependencies.');
```

### `build_tag_expansions.py` {#build-tag-expansions-py}

- **Lines**: 441 (code: 292, comments: 73, blank: 76)
- **Pylint Score**: 7.83/10 ⚠️
- **Docstring Coverage**: 83.3%
- **Functions**: query_files_for_pack, get_bpm_folder, convert_and_organize, build_pack, list_packs, main

#### Source Code

```python
#!/usr/bin/env python3
"""
Build MPC Expansion Packs from Database Tags

Queries PostgreSQL database for files matching tag criteria,
converts MIDI → .mpcpattern, and organizes into expansion pack structure.

Usage:
    python3 scripts/build_tag_expansions.py --pack rock-drums --limit 1000
    python3 scripts/build_tag_expansions.py --pack all --output /path/to/expansions
    python3 scripts/build_tag_expansions.py --list  # Show available packs
"""

import psycopg2
import subprocess
import json
import os
import sys
from pathlib import Path
from collections import defaultdict
import argparse
import datetime

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

# Converter path
CONVERTER = "./target/release/midi_to_mpcpattern"

# Expansion pack definitions
EXPANSION_PACKS = {
    'rock-drums': {
        'name': 'Rock Drum Patterns',
        'description': 'Essential rock drum patterns across all tempos',
        'tags': ['rock'],
        'tag_category': 'genre',
        'bpm_min': 100,
        'bpm_max': 160,
        'limit': 1000,
        'organize_by': 'bpm',  # or 'key', 'flat'
        'bpm_folders': ['100-120', '120-140', '140-160']
    },

    'ride-library': {
        'name': 'Ride Cymbal Library',
        'description': 'Comprehensive ride cymbal patterns for all styles',
        'tags': ['ride'],
        'tag_category': 'drums',
        'bpm_min': 60,
        'bpm_max': 200,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['060-100', '100-140', '140-180', '180+']
    },

    'kick-collection': {
        'name': 'Kick Drum Collection',
        'description': 'Kick drum patterns from soft to hard',
        'tags': ['kick'],
        'tag_category': 'drums',
        'bpm_min': 80,
        'bpm_max': 180,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['080-100', '100-120', '120-140', '140-180']
    },

    'house-grooves': {
        'name': 'House Grooves 120-130',
        'description': 'Classic house drum patterns in the perfect BPM range',
        'tags': ['house'],
        'tag_category': 'genre',
        'bpm_min': 120,
        'bpm_max': 130,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['120-124', '124-128', '128-130']
    },

    'fill-patterns': {
        'name': 'Fill Patterns Collection',
        'description': 'Drum fills for transitions and breaks',
        'tags': ['fill'],
        'tag_category': 'drums',
        'bpm_min': 80,
        'bpm_max': 180,
        'limit': 500,
        'organize_by': 'bpm',
        'bpm_folders': ['080-110', '110-140', '140-180']
    },

    'funk-drums': {
        'name': 'Funk Drum Patterns',
        'description': 'Funky grooves and pocket patterns',
        'tags': ['funk'],
        'tag_category': 'genre',
        'bpm_min': 90,
        'bpm_max': 120,
        'limit': 400,
        'organize_by': 'bpm',
        'bpm_folders': ['090-100', '100-110', '110-120']
    },

    'tom-patterns': {
        'name': 'Tom Patterns',
        'description': 'Tom drum patterns and fills',
        'tags': ['tom'],
        'tag_category': 'drums',
        'bpm_min': 80,
        'bpm_max': 180,
        'limit': 400,
        'organize_by': 'bpm',
        'bpm_folders': ['080-120', '120-150', '150-180']
    },

    'groove-library': {
        'name': 'Groove Library',
        'description': 'Essential groove patterns for all styles',
        'tags': ['groove'],
        'tag_category': 'pattern',
        'bpm_min': 80,
        'bpm_max': 160,
        'limit': 400,
        'organize_by': 'bpm',
        'bpm_folders': ['080-100', '100-120', '120-140', '140-160']
    },

    'edm-drums': {
        'name': 'EDM Drums 120-140',
        'description': 'Electronic dance music drum patterns',
        'tags': ['edm'],
        'tag_category': 'genre',
        'bpm_min': 120,
        'bpm_max': 140,
        'limit': 300,
        'organize_by': 'bpm',
        'bpm_folders': ['120-128', '128-135', '135-140']
    },

    'hiphop-beats': {
        'name': 'Hip-Hop Beats 85-100',
        'description': 'Classic hip-hop drum patterns',
        'tags': ['beat', 'drum'],  # Multiple tags
        'tag_category': ['keyword', 'drums'],
        'bpm_min': 85,
        'bpm_max': 100,
        'limit': 300,
        'organize_by': 'bpm',
        'bpm_folders': ['085-090', '090-095', '095-100']
    }
}


def query_files_for_pack(config):
    """Query database for files matching pack criteria"""

    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    tags = config['tags']
    tag_category = config['tag_category']
    bpm_min = config['bpm_min']
    bpm_max = config['bpm_max']
    limit = config['limit']

    # Build query
    if isinstance(tag_category, list):
        # Multiple tag categories
        tag_conditions = " OR ".join([f"t.category = '{cat}'" for cat in tag_category])
    else:
        tag_conditions = f"t.category = '{tag_category}'"

    tag_names = "', '".join(tags)

    query = f"""
    SELECT DISTINCT
        f.id,
        f.filepath,
        m.bpm,
        m.key_signature,
        string_agg(DISTINCT t.name, ', ') as tags
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('{tag_names}')
      AND ({tag_conditions})
      AND m.bpm BETWEEN {bpm_min} AND {bpm_max}
      AND m.bpm IS NOT NULL
    GROUP BY f.id, f.filepath, m.bpm, m.key_signature
    ORDER BY m.bpm, f.filepath
    LIMIT {limit};
    """

    print(f"Executing query...")
    cur.execute(query)
    results = cur.fetchall()

    cur.close()
    conn.close()

    return results


def get_bpm_folder(bpm, bpm_folders):
    """Determine which BPM folder a file belongs to"""
    for folder in bpm_folders:
        if '+' in folder:
            # e.g., "180+"
            min_bpm = int(folder.replace('+', ''))
            if bpm >= min_bpm:
                return folder
        else:
            # e.g., "120-140"
            parts = folder.split('-')
            min_bpm = int(parts[0])
            max_bpm = int(parts[1])
            if min_bpm <= bpm < max_bpm:
                return folder

    # Fallback
    return 'Other'


def convert_and_organize(files, pack_id, config, output_base):
    """Convert MIDI files and organize into expansion pack structure"""

    pack_name = config['name']
    pack_dir = output_base / pack_name

    # Create expansion folder
    pack_dir.mkdir(parents=True, exist_ok=True)

    # Create [Previews] folder
    previews_dir = pack_dir / "[Previews]"
    previews_dir.mkdir(exist_ok=True)

    print(f"\nBuilding expansion: {pack_name}")
    print(f"  Output: {pack_dir}")
    print(f"  Files to convert: {len(files)}")

    # Organize files by BPM folder
    if config['organize_by'] == 'bpm':
        files_by_folder = defaultdict(list)
        for file_data in files:
            file_id, filepath, bpm, key_sig, tags = file_data
            folder_name = get_bpm_folder(bpm, config['bpm_folders'])
            files_by_folder[folder_name].append(file_data)

        # Create BPM subfolders and convert
        total_converted = 0
        total_failed = 0

        for folder_name, folder_files in files_by_folder.items():
            folder_path = pack_dir / folder_name
            folder_path.mkdir(exist_ok=True)

            print(f"\n  Converting {len(folder_files)} files to {folder_name}/")

            for i, file_data in enumerate(folder_files):
                file_id, filepath, bpm, key_sig, tags = file_data

                # Generate output filename
                file_stem = Path(filepath).stem
                output_name = f"{int(bpm):03d}bpm-{file_stem}.mpcpattern"
                output_path = folder_path / output_name

                # Convert using Rust converter
                try:
                    result = subprocess.run(
                        [CONVERTER, filepath, str(output_path)],
                        capture_output=True,
                        text=True,
                        timeout=30
                    )

                    if result.returncode == 0:
                        total_converted += 1
                        if (i + 1) % 50 == 0:
                            print(f"    Progress: {i+1}/{len(folder_files)}")
                    else:
                        total_failed += 1
                        print(f"    ✗ Error converting: {filepath}")

                except Exception as e:
                    total_failed += 1
                    print(f"    ✗ Exception: {e}")

        print(f"\n  ✓ Converted: {total_converted}")
        print(f"  ✗ Failed: {total_failed}")

    else:
        # Flat organization
        print(f"  Converting all files to root...")
        total_converted = 0
        total_failed = 0

        for i, file_data in enumerate(files):
            file_id, filepath, bpm, key_sig, tags = file_data

            file_stem = Path(filepath).stem
            output_name = f"{int(bpm):03d}bpm-{file_stem}.mpcpattern"
            output_path = pack_dir / output_name

            try:
                result = subprocess.run(
                    [CONVERTER, filepath, str(output_path)],
                    capture_output=True,
                    text=True,
                    timeout=30
                )

                if result.returncode == 0:
                    total_converted += 1
                    if (i + 1) % 50 == 0:
                        print(f"    Progress: {i+1}/{len(files)}")
                else:
                    total_failed += 1

            except Exception as e:
                total_failed += 1

        print(f"\n  ✓ Converted: {total_converted}")
        print(f"  ✗ Failed: {total_failed}")

    # Create Cache.json (minimal for now)
    cache_data = {
        "name": pack_name,
        "description": config['description'],
        "version": "1.0",
        "files": total_converted
    }

    with open(pack_dir / "Cache.json", 'w') as f:
        json.dump(cache_data, f, indent=2)

    # Create README
    readme_content = f"""# {pack_name}

{config['description']}

## Contents
- {total_converted} .mpcpattern files
- Organized by: {config['organize_by']}
- BPM range: {config['bpm_min']}-{config['bpm_max']}
- Tags: {', '.join(config['tags'])}

## Installation
1. Copy this folder to your Force/MPC drive: /Expansions/
2. The expansion will appear in your Expansion Browser

## Generated by
MIDI Software Center - Tag-Based Expansion Builder
Date: {datetime.datetime.now().strftime('%Y-%m-%d')}
"""

    with open(pack_dir / "README.txt", 'w') as f:
        f.write(readme_content)

    return total_converted, total_failed


def build_pack(pack_id, output_dir):
    """Build a single expansion pack"""

    if pack_id not in EXPANSION_PACKS:
        print(f"Error: Unknown pack '{pack_id}'")
        print(f"Available packs: {', '.join(EXPANSION_PACKS.keys())}")
        return False

    config = EXPANSION_PACKS[pack_id]

    print(f"="*70)
    print(f"Building: {config['name']}")
    print(f"="*70)

    # Query database
    files = query_files_for_pack(config)
    print(f"Found {len(files)} matching files")

    if len(files) == 0:
        print("No files found matching criteria!")
        return False

    # Convert and organize
    converted, failed = convert_and_organize(files, pack_id, config, output_dir)

    print(f"\n✓ Pack complete: {config['name']}")
    print(f"  Location: {output_dir / config['name']}")
    print(f"  Patterns: {converted}")

    return True


def list_packs():
    """List all available expansion packs"""
    print("\nAvailable Expansion Packs:\n")
    for pack_id, config in EXPANSION_PACKS.items():
        print(f"  {pack_id:20s} - {config['name']}")
        print(f"  {' '*20}   {config['description']}")
        print(f"  {' '*20}   Limit: {config['limit']}, BPM: {config['bpm_min']}-{config['bpm_max']}")
        print()


def main():
    parser = argparse.ArgumentParser(description='Build MPC expansion packs from database tags')
    parser.add_argument('--pack', help='Pack ID to build (or "all" for all packs)')
    parser.add_argument('--output', default='/media/dojevou/RYXSTR/Expansions',
                       help='Output directory for expansions')
    parser.add_argument('--list', action='store_true', help='List available packs')

    args = parser.parse_args()

    if args.list:
        list_packs()
        return

    if not args.pack:
        print("Error: --pack required (or use --list to see available packs)")
        return

    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    if args.pack == 'all':
        print(f"Building ALL expansion packs to: {output_dir}\n")
        success_count = 0
        for pack_id in EXPANSION_PACKS.keys():
            if build_pack(pack_id, output_dir):
                success_count += 1
            print("\n")

        print(f"="*70)
        print(f"Completed: {success_count}/{len(EXPANSION_PACKS)} packs")
        print(f"="*70)
    else:
        build_pack(args.pack, output_dir)


if __name__ == '__main__':
    main()

```

### `convert_instruments_python.py` {#convert-instruments-python-py}

- **Lines**: 165 (code: 108, comments: 27, blank: 30)
- **Pylint Score**: 8.70/10 ✅
- **Functions**: main

#### Source Code

```python
#!/usr/bin/env python3
"""
Convert 2,000 patterns per instrument for all 97 expansions
Created: November 23, 2025
"""

import subprocess
import sys
from pathlib import Path

# Configuration
DB_HOST = "localhost"
DB_PORT = "5433"
DB_USER = "midiuser"
DB_PASS = "145278963"
DB_NAME = "midi_library"

PROJECT_ROOT = Path("/home/dojevou/projects/midi-software-center")
FORCE_DRIVE = Path("/media/dojevou/RYXSTR")
EXPANSIONS_DIR = FORCE_DRIVE / "Expansions"
CONVERTER = PROJECT_ROOT / "target/release/midi_to_mpcpattern_parallel"
INSTRUMENT_LIST = PROJECT_ROOT / "INSTRUMENT_LIST.txt"
PATTERNS_PER_INSTRUMENT = 2000

def main():
    print("=" * 50)
    print("  Converting 97 Instrument Expansions")
    print(f"  {PATTERNS_PER_INSTRUMENT:,} patterns per instrument")
    print(f"  ~{97 * PATTERNS_PER_INSTRUMENT:,} total patterns")
    print("=" * 50)
    print()

    # Read instrument list
    instruments = []
    with open(INSTRUMENT_LIST) as f:
        for line in f:
            line = line.strip()
            if not line or ':' not in line:
                continue
            instrument, count = line.split(':', 1)
            instruments.append((instrument.strip(), int(count.strip())))

    print(f"Found {len(instruments)} instruments")
    print()

    total_converted = 0
    total_failed = 0

    for idx, (instrument, db_count) in enumerate(instruments, 1):
        # Convert instrument name to expansion format
        expansion_name = f"MIDI_{instrument.upper().replace('-', '_').replace('&', 'AND')}"
        expansion_dir = EXPANSIONS_DIR / expansion_name
        patterns_dir = expansion_dir / "Patterns"

        if not expansion_dir.exists():
            print(f"⚠️  [{idx}/97] Skipping {expansion_name} (folder not found)")
            continue

        print(f"[{idx}/97] Processing: {expansion_name}")
        print(f"  Available in database: {db_count:,}")

        # Query database for files
        query = f"""
            SELECT f.filepath
            FROM files f
            LEFT JOIN file_tags ft ON f.id = ft.file_id
            LEFT JOIN tags t ON ft.tag_id = t.id
            WHERE f.num_tracks = 1
              AND (
                t.name = '{instrument}'
                OR f.filename ILIKE '%{instrument}%'
              )
            GROUP BY f.filepath
            ORDER BY RANDOM()
            LIMIT {PATTERNS_PER_INSTRUMENT};
        """

        try:
            import os
            env = os.environ.copy()
            env["PGPASSWORD"] = DB_PASS

            result = subprocess.run(
                ["psql", "-h", DB_HOST, "-p", DB_PORT, "-U", DB_USER, "-d", DB_NAME, "-t", "-c", query],
                env=env,
                capture_output=True,
                text=True
            )

            if result.returncode != 0:
                print(f"  ❌ Database query failed: {result.stderr.strip()}")
                continue

            filepaths = [line.strip() for line in result.stdout.strip().split('\n') if line.strip()]

            if not filepaths:
                print(f"  ⚠️  No files found")
                print()
                continue

            print(f"  Files to convert: {len(filepaths)}")

            # Convert files
            converted = 0
            failed = 0

            for filepath in filepaths:
                filepath_obj = Path(filepath)

                if not filepath_obj.exists():
                    failed += 1
                    continue

                filename = filepath_obj.stem
                output_file = patterns_dir / f"{filename}.mpcpattern"

                # Convert
                result = subprocess.run(
                    [str(CONVERTER), str(filepath), str(output_file)],
                    capture_output=True
                )

                if result.returncode == 0:
                    converted += 1
                else:
                    failed += 1

                # Progress dots
                if converted % 100 == 0 and converted > 0:
                    print(".", end="", flush=True)

            print()
            print(f"  ✓ Converted: {converted:,} patterns")
            if failed > 0:
                print(f"  ✗ Failed: {failed:,} files")

            total_converted += converted
            total_failed += failed

        except Exception as e:
            print(f"  ❌ Error: {e}")

        print()

    # Final summary
    print("=" * 50)
    print("  CONVERSION COMPLETE!")
    print("=" * 50)
    print()
    print("📊 Summary:")
    print(f"  - Instruments processed: {idx}/97")
    print(f"  - Total patterns converted: {total_converted:,}")
    print(f"  - Failed conversions: {total_failed:,}")
    print()
    print(f"📂 Location: {EXPANSIONS_DIR}")
    print()
    print("🎯 Next Steps:")
    print("  1. Test expansions on Force hardware")
    print("  2. Browser > Expansions > MIDI_[INSTRUMENT]")
    print("  3. Enjoy your library!")
    print()
    print("✓ All done!")

if __name__ == "__main__":
    main()

```

### `derive_injector.py` {#derive-injector-py}

- **Lines**: 249 (code: 168, comments: 32, blank: 49)
- **Pylint Score**: 5.73/10 ⚠️
- **Docstring Coverage**: 66.7%
- **Functions**: __init__, find_struct_definitions, get_existing_derives, inject_derives, fix_tagged_response, fix_import_progress, fix_emitter_generics, fix_file, run, replace_tag_response
  *... and 1 more*
- **Classes**: DeriveMacroInjector

#### Source Code

```python
#!/usr/bin/env python3
"""
Derive Macro Injector - Automatically adds missing derive macros
Category 6: 18 errors related to missing trait implementations
"""

import re
from pathlib import Path
from typing import List, Set, Dict, Tuple
import sys

class DeriveMacroInjector:
    def __init__(self, rust_root: str = '.'):
        self.rust_root = Path(rust_root)
        self.fixes_applied = 0
        self.files_modified = 0
        
        # Map error patterns to required derive macros
        self.error_trait_map = {
            "cannot apply '=='": ['PartialEq'],
            "is not satisfied": ['Debug', 'Clone', 'Serialize', 'Deserialize'],
            "binary operation '==' cannot be applied": ['PartialEq'],
            "the trait bound": ['Debug', 'Clone', 'Serialize', 'Deserialize'],
        }
    
    def find_struct_definitions(self, content: str) -> List[Tuple[str, int, str]]:
        """
        Find all struct definitions
        Returns: List of (struct_name, line_number, full_definition)
        """
        pattern = re.compile(
            r'(?:pub\s+)?(?:async\s+)?struct\s+(\w+)[^{]*\{[^}]*\}',
            re.MULTILINE | re.DOTALL
        )
        matches = []
        for match in pattern.finditer(content):
            struct_name = match.group(1)
            line_num = content[:match.start()].count('\n')
            matches.append((struct_name, line_num, match.group(0)))
        
        return matches
    
    def get_existing_derives(self, struct_def: str) -> Set[str]:
        """Extract existing derive macros from struct definition"""
        derive_pattern = r'#\[derive\(([^)]+)\)\]'
        match = re.search(derive_pattern, struct_def)
        
        if match:
            derives = match.group(1)
            return set(d.strip() for d in derives.split(','))
        
        return set()
    
    def inject_derives(self, struct_def: str, required_derives: List[str]) -> Tuple[str, int]:
        """
        Add required derive macros to struct definition
        Returns: (modified_def, number_of_derives_added)
        """
        existing = self.get_existing_derives(struct_def)
        new_derives = set(required_derives) - existing
        
        if not new_derives:
            return struct_def, 0
        
        all_derives = sorted(list(existing | set(required_derives)))
        derives_str = ', '.join(all_derives)
        
        # Check if struct already has #[derive(...)]
        if '#[derive(' in struct_def:
            # Replace existing derive
            new_def = re.sub(
                r'#\[derive\([^)]+\)\]',
                f'#[derive({derives_str})]',
                struct_def
            )
        else:
            # Add new derive before struct
            new_def = f'#[derive({derives_str})]\n{struct_def}'
        
        return new_def, len(new_derives)
    
    def fix_tagged_response(self, content: str) -> Tuple[str, int]:
        """Fix TagResponse struct - needs PartialEq"""
        fixes = 0
        
        # Find TagResponse struct
        pattern = re.compile(
            r'(#\[derive\(([^)]*)\)\]\s*)?(?:pub\s+)?struct\s+TagResponse\s*\{[^}]*\}',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_tag_response(match):
            nonlocal fixes
            struct_def = match.group(0)
            existing_derives = match.group(2) or ""
            
            required = {'Debug', 'Clone', 'Serialize', 'Deserialize', 'PartialEq'}
            existing_set = set(d.strip() for d in existing_derives.split(',') if d.strip())
            needed = required - existing_set
            
            if needed:
                all_derives = sorted(list(existing_set | required))
                derives_str = ', '.join(all_derives)
                new_struct = re.sub(
                    r'(#\[derive\([^)]*\)\]\s*)?(?=(?:pub\s+)?struct)',
                    f'#[derive({derives_str})]\n',
                    struct_def
                )
                fixes += len(needed)
                return new_struct
            
            return struct_def
        
        new_content = pattern.sub(replace_tag_response, content)
        return new_content, fixes
    
    def fix_import_progress(self, content: str) -> Tuple[str, int]:
        """Fix ImportProgress struct - needs Deserialize"""
        fixes = 0
        
        pattern = re.compile(
            r'(#\[derive\(([^)]*)\)\]\s*)?(?:pub\s+)?struct\s+ImportProgress\s*\{[^}]*\}',
            re.MULTILINE | re.DOTALL
        )
        
        def replace_import_progress(match):
            nonlocal fixes
            struct_def = match.group(0)
            existing_derives = match.group(2) or ""
            
            required = {'Debug', 'Clone', 'Serialize', 'Deserialize'}
            existing_set = set(d.strip() for d in existing_derives.split(',') if d.strip())
            needed = required - existing_set
            
            if needed:
                all_derives = sorted(list(existing_set | required))
                derives_str = ', '.join(all_derives)
                new_struct = re.sub(
                    r'(#\[derive\([^)]*\)\]\s*)?(?=(?:pub\s+)?struct)',
                    f'#[derive({derives_str})]\n',
                    struct_def
                )
                fixes += len(needed)
                return new_struct
            
            return struct_def
        
        new_content = pattern.sub(replace_import_progress, content)
        return new_content, fixes
    
    def fix_emitter_generics(self, content: str) -> Tuple[str, int]:
        """Fix missing generics in Emitter trait usage"""
        fixes = 0
        
        # Pattern: impl Emitter without generic type
        pattern = re.compile(
            r'impl\s+Emitter\s*<(?!\w)([^>]*)>',
            re.MULTILINE
        )
        
        # This requires manual inspection - just mark for review
        if pattern.search(content):
            print("  ⚠️  Manual review needed for Emitter<T> generics")
        
        return content, fixes
    
    def fix_file(self, file_path: Path) -> int:
        """Apply all derive macro fixes to a file"""
        try:
            content = file_path.read_text(encoding='utf-8')
        except Exception as e:
            print(f"  ❌ Cannot read {file_path}: {e}")
            return 0
        
        original_content = content
        total_fixes = 0
        
        # Apply specific struct fixes
        content, tag_fixes = self.fix_tagged_response(content)
        total_fixes += tag_fixes
        
        content, import_fixes = self.fix_import_progress(content)
        total_fixes += import_fixes
        
        # Try generic struct fixes for PartialEq
        # Look for structs used in comparisons
        comparison_pattern = re.compile(
            r'assert_eq!\(\s*(\w+),\s*(\w+)\s*\)',
            re.MULTILINE
        )
        
        for match in comparison_pattern.finditer(content):
            # These need PartialEq and Eq if not already derived
            struct_name = match.group(1)
            pattern = re.compile(
                rf'(?:pub\s+)?struct\s+{struct_name}\s*\{{',
                re.MULTILINE
            )
            if pattern.search(content):
                # Found the struct, ensure it has PartialEq
                # This is approximate - real solution would be more sophisticated
                pass
        
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            self.files_modified += 1
            return total_fixes
        
        return 0
    
    def run(self) -> bool:
        """Run the injector on all Rust files"""
        rust_files = list(self.rust_root.rglob('*.rs'))
        
        if not rust_files:
            print(f"❌ No Rust files found in {self.rust_root}")
            return False
        
        print(f"🔍 Found {len(rust_files)} Rust files")
        print(f"🔧 Injecting missing derive macros...\n")
        
        for file_path in rust_files:
            fixes = self.fix_file(file_path)
            if fixes > 0:
                print(f"  ✅ {file_path.relative_to(self.rust_root)}: {fixes} macros added")
                self.fixes_applied += fixes
        
        print(f"\n{'='*60}")
        print(f"📊 RESULTS")
        print(f"{'='*60}")
        print(f"Files modified:     {self.files_modified}")
        print(f"Total fixes:        {self.fixes_applied}")
        
        if self.fixes_applied > 0:
            print(f"\n✅ Derive macros injected!")
            print(f"Next: Run 'cargo build' to verify")
            return True
        else:
            print(f"\n⚠️  No missing derives found")
            return False


if __name__ == '__main__':
    rust_root = sys.argv[1] if len(sys.argv) > 1 else '.'
    
    injector = DeriveMacroInjector(rust_root)
    success = injector.run()
    
    sys.exit(0 if success else 1)

```

### `export-force-midi.py` {#export-force-midi-py}

- **Lines**: 329 (code: 196, comments: 81, blank: 52)
- **Pylint Score**: 9.18/10 ✅
- **Docstring Coverage**: 80.0%
- **Functions**: connect_db, export_progressions, export_arp_patterns, export_drum_patterns, main

#### Source Code

```python
#!/usr/bin/env python3
"""
Export MIDI files from database to Akai Force folder structure.
Exports to:
  - Progressions/ (chord progressions by key)
  - Arp Patterns/ (arpeggiator patterns by BPM)
  - MIDI_Patterns/ (drums, bass, keys organized)
"""

import psycopg2
import psycopg2.extras
import shutil
import os
from pathlib import Path
import sys

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

# Export destination (Akai Force drive)
FORCE_ROOT = "/media/dojevou/RYXSTR"

# Export limits (adjust as needed)
LIMITS = {
    'progressions': 10000,  # Top 10,000 chord progressions
    'arp_patterns': 5000,   # Top 5,000 arp patterns
    'drum_patterns': 20000, # Top 20,000 drum patterns
    'bass_patterns': 5000,  # Top 5,000 bass patterns
    'key_patterns': 5000,   # Top 5,000 key/piano patterns
}

def connect_db():
    """Connect to PostgreSQL database"""
    return psycopg2.connect(DB_URL)

def export_progressions(conn, limit=10000, dry_run=False):
    """Export chord progressions organized by key"""
    print(f"\n{'='*70}")
    print(f"EXPORTING CHORD PROGRESSIONS (limit: {limit:,})")
    print(f"{'='*70}\n")

    query = """
    SELECT
        f.filepath,
        f.filename,
        m.key_signature,
        m.bpm,
        COUNT(ft.tag_id) as tag_count,
        string_agg(t.name, ', ') as tags
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('chord', 'progression', 'piano', 'keys', 'pad', 'synth')
      AND m.key_signature IS NOT NULL
    GROUP BY f.id, f.filepath, f.filename, m.key_signature, m.bpm
    HAVING COUNT(ft.tag_id) >= 3
    ORDER BY m.key_signature, COUNT(ft.tag_id) DESC, m.bpm
    LIMIT %s;
    """

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)
    cursor.execute(query, (limit,))

    results = cursor.fetchall()
    print(f"Found {len(results):,} chord progression files\n")

    # Map database keys to folder names
    key_map = {
        'C': 'C Major', 'Cm': 'C Minor',
        'C#': 'C# Major', 'C#m': 'C# Minor',
        'D': 'D Major', 'Dm': 'D Minor',
        'D#': 'D# Major', 'D#m': 'D# Minor',
        'E': 'E Major', 'Em': 'E Minor',
        'F': 'F Major', 'Fm': 'F Minor',
        'F#': 'F# Major', 'F#m': 'F# Minor',
        'G': 'G Major', 'Gm': 'G Minor',
        'G#': 'G# Major', 'G#m': 'G# Minor',
        'A': 'A Major', 'Am': 'A Minor',
        'A#': 'A# Major', 'A#m': 'A# Minor',
        'B': 'B Major', 'Bm': 'B Minor',
    }

    stats = {}
    copied = 0
    skipped = 0
    errors = 0

    for row in results:
        source = row['filepath']
        key_sig = row['key_signature']
        bpm = row['bpm'] or 120

        # Map key signature to folder
        folder_key = key_map.get(key_sig)
        if not folder_key:
            # Try to infer (e.g., "Cmaj" → "C Major")
            if 'maj' in key_sig.lower():
                folder_key = key_sig.replace('maj', '').strip() + ' Major'
            elif 'min' in key_sig.lower():
                folder_key = key_sig.replace('min', '').strip() + ' Minor'
            else:
                folder_key = 'Unknown'

        dest_dir = Path(FORCE_ROOT) / "Progressions" / folder_key
        dest_file = dest_dir / f"{int(bpm)}bpm_{Path(source).name}"

        stats[folder_key] = stats.get(folder_key, 0) + 1

        if not dry_run:
            try:
                if not dest_file.exists():
                    dest_dir.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source, dest_file)
                    copied += 1
                else:
                    skipped += 1
            except Exception as e:
                print(f"Error copying {source}: {e}")
                errors += 1
        else:
            copied += 1

    print("Progressions by key:")
    for key in sorted(stats.keys()):
        print(f"  {key:20s}: {stats[key]:,} files")

    print(f"\nTotal: {copied:,} copied, {skipped:,} skipped, {errors:,} errors")
    return copied

def export_arp_patterns(conn, limit=5000, dry_run=False):
    """Export arpeggiator patterns organized by BPM"""
    print(f"\n{'='*70}")
    print(f"EXPORTING ARP PATTERNS (limit: {limit:,})")
    print(f"{'='*70}\n")

    query = """
    SELECT
        f.filepath,
        f.filename,
        m.bpm,
        m.key_signature,
        COUNT(ft.tag_id) as tag_count
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('arp', 'arpeggiat', 'pattern', 'sequence')
      AND m.bpm IS NOT NULL
      AND m.bpm BETWEEN 60 AND 180
    GROUP BY f.id, f.filepath, f.filename, m.bpm, m.key_signature
    HAVING COUNT(ft.tag_id) >= 2
    ORDER BY m.bpm, COUNT(ft.tag_id) DESC
    LIMIT %s;
    """

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)
    cursor.execute(query, (limit,))

    results = cursor.fetchall()
    print(f"Found {len(results):,} arp pattern files\n")

    stats = {}
    copied = 0
    skipped = 0
    errors = 0

    for row in results:
        source = row['filepath']
        bpm = int(row['bpm'])

        # Determine BPM folder
        if bpm < 80:
            bpm_folder = '60-80_BPM'
        elif bpm < 100:
            bpm_folder = '80-100_BPM'
        elif bpm < 120:
            bpm_folder = '100-120_BPM'
        elif bpm < 140:
            bpm_folder = '120-140_BPM'
        elif bpm < 160:
            bpm_folder = '140-160_BPM'
        else:
            bpm_folder = '160-180_BPM'

        dest_dir = Path(FORCE_ROOT) / "Arp Patterns" / bpm_folder
        dest_file = dest_dir / f"{bpm}bpm_{Path(source).name}"

        stats[bpm_folder] = stats.get(bpm_folder, 0) + 1

        if not dry_run:
            try:
                if not dest_file.exists():
                    dest_dir.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source, dest_file)
                    copied += 1
                else:
                    skipped += 1
            except Exception as e:
                print(f"Error copying {source}: {e}")
                errors += 1
        else:
            copied += 1

    print("Arp patterns by BPM:")
    for folder in sorted(stats.keys()):
        print(f"  {folder:20s}: {stats[folder]:,} files")

    print(f"\nTotal: {copied:,} copied, {skipped:,} skipped, {errors:,} errors")
    return copied

def export_drum_patterns(conn, limit=20000, dry_run=False):
    """Export drum patterns organized by BPM"""
    print(f"\n{'='*70}")
    print(f"EXPORTING DRUM PATTERNS (limit: {limit:,})")
    print(f"{'='*70}\n")

    query = """
    SELECT
        f.filepath,
        f.filename,
        m.bpm,
        COUNT(ft.tag_id) as tag_count,
        string_agg(t.name, ', ') as tags
    FROM files f
    JOIN musical_metadata m ON f.id = m.file_id
    JOIN file_tags ft ON f.id = ft.file_id
    JOIN tags t ON ft.tag_id = t.id
    WHERE t.name IN ('drums', 'drum', 'groove', 'fill', 'kick', 'snare', 'hihat', 'cymbal', 'ride', 'crash', 'tom')
      AND m.bpm IS NOT NULL
      AND m.bpm BETWEEN 80 AND 180
    GROUP BY f.id, f.filepath, f.filename, m.bpm
    HAVING COUNT(ft.tag_id) >= 2
    ORDER BY m.bpm, COUNT(ft.tag_id) DESC
    LIMIT %s;
    """

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)
    cursor.execute(query, (limit,))

    results = cursor.fetchall()
    print(f"Found {len(results):,} drum pattern files\n")

    stats = {}
    copied = 0
    skipped = 0
    errors = 0

    for row in results:
        source = row['filepath']
        bpm = int(row['bpm'])

        # Determine BPM folder
        if bpm < 100:
            bpm_folder = '80-100_BPM'
        elif bpm < 120:
            bpm_folder = '100-120_BPM'
        elif bpm < 140:
            bpm_folder = '120-140_BPM'
        elif bpm < 160:
            bpm_folder = '140-160_BPM'
        else:
            bpm_folder = '160-180_BPM'

        dest_dir = Path(FORCE_ROOT) / "MIDI_Patterns" / "Drums" / bpm_folder
        dest_file = dest_dir / f"{bpm}bpm_{Path(source).name}"

        stats[bpm_folder] = stats.get(bpm_folder, 0) + 1

        if not dry_run:
            try:
                if not dest_file.exists():
                    dest_dir.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(source, dest_file)
                    copied += 1
                else:
                    skipped += 1
            except Exception as e:
                print(f"Error copying {source}: {e}")
                errors += 1
        else:
            copied += 1

    print("Drum patterns by BPM:")
    for folder in sorted(stats.keys()):
        print(f"  {folder:20s}: {stats[folder]:,} files")

    print(f"\nTotal: {copied:,} copied, {skipped:,} skipped, {errors:,} errors")
    return copied

def main():
    dry_run = '--dry-run' in sys.argv or '--test' in sys.argv

    if dry_run:
        print("\n" + "="*70)
        print("DRY RUN MODE - No files will be copied")
        print("="*70 + "\n")

    conn = connect_db()

    try:
        # Export progressions
        prog_count = export_progressions(conn, LIMITS['progressions'], dry_run)

        # Export arp patterns
        arp_count = export_arp_patterns(conn, LIMITS['arp_patterns'], dry_run)

        # Export drum patterns
        drum_count = export_drum_patterns(conn, LIMITS['drum_patterns'], dry_run)

        # Summary
        print(f"\n{'='*70}")
        print("EXPORT COMPLETE")
        print(f"{'='*70}")
        print(f"Chord Progressions: {prog_count:,} files")
        print(f"Arp Patterns:       {arp_count:,} files")
        print(f"Drum Patterns:      {drum_count:,} files")
        print(f"{'='*70}")
        print(f"Total exported:     {prog_count + arp_count + drum_count:,} files")
        print(f"{'='*70}\n")

        if dry_run:
            print("This was a DRY RUN. Run without --dry-run to actually copy files.\n")

    finally:
        conn.close()

if __name__ == "__main__":
    main()

```

### `fast_multi_level_tagger.py` {#fast-multi-level-tagger-py}

- **Lines**: 361 (code: 177, comments: 119, blank: 65)
- **Pylint Score**: 9.22/10 ✅
- **Docstring Coverage**: 100.0%
- **Functions**: normalize_keyword, extract_path_components, load_curated_tags, process_files, insert_batch, verify_results, main

#### Source Code

```python
#!/usr/bin/env python3
"""
Fast Multi-Level Tagging Script

Extracts keywords from grandparent folders, parent folders, and filenames
in a single pass through all files, then batch inserts tags.

Performance: ~5-15 minutes for 1.79M files (vs 8 hours for sequential)
"""

import psycopg2
import psycopg2.extras
import re
import sys
from pathlib import Path
from collections import defaultdict
from typing import Set, Dict, List, Tuple
import time

# Configuration
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"
BATCH_SIZE = 10000  # Insert 10k tags at once
CHUNK_SIZE = 5000   # Process 5k files at a time
MIN_KEYWORD_LENGTH = 3
MAX_KEYWORD_LENGTH = 50

# Tag categories
CATEGORY_GRANDPARENT = "grandparent_folder"
CATEGORY_PARENT = "parent_folder"
CATEGORY_FILENAME = "filename"

def normalize_keyword(text: str) -> Set[str]:
    """
    Normalize a string into keywords.

    Examples:
        "120bpm_Bass_Loop" -> {"120bpm", "bass", "loop"}
        "4-4 Grooves" -> {"grooves"}
        "EDM Midi pack" -> {"edm", "midi", "pack"}
    """
    if not text:
        return set()

    # Lowercase
    text = text.lower()

    # Replace delimiters with spaces
    text = re.sub(r'[_\-\(\)\[\]@#]', ' ', text)

    # Split on spaces and filter
    keywords = set()
    for word in text.split():
        # Clean word
        word = word.strip()

        # Skip if too short or too long
        if len(word) < MIN_KEYWORD_LENGTH or len(word) > MAX_KEYWORD_LENGTH:
            continue

        # Skip numbers-only
        if word.isdigit():
            continue

        # Skip common noise words
        if word in {'the', 'and', 'for', 'with', 'from', 'midi', 'mid'}:
            continue

        keywords.add(word)

    return keywords

def extract_path_components(filepath: str) -> Tuple[str, str, str]:
    """
    Extract grandparent folder, parent folder, and filename from path.

    Example: "./archives/Linear Drums/House/track.mid"
    Returns: ("Linear Drums", "House", "track")
    """
    path = Path(filepath)

    # Filename without extension
    filename = path.stem

    # Parent folder (1 level up)
    parent = path.parent.name if path.parent != Path('.') else ""

    # Grandparent folder (2 levels up)
    grandparent = ""
    if path.parent != Path('.') and path.parent.parent != Path('.'):
        grandparent = path.parent.parent.name

    return (grandparent, parent, filename)

def load_curated_tags(conn) -> Dict[str, int]:
    """
    Load curated tags from master_tag_list.txt and insert into database.
    Returns mapping of tag_name -> tag_id
    """
    print("📋 Loading curated tags...")

    # Check if master tag list exists
    tag_file = Path("/tmp/master_tag_list.txt")
    if not tag_file.exists():
        print("❌ ERROR: /tmp/master_tag_list.txt not found!")
        print("   Run: ./scripts/create-curated-tags.sh first")
        sys.exit(1)

    # Load tags from file
    tags_to_insert = []
    with open(tag_file, 'r') as f:
        for line in f:
            tag_name = line.strip()
            if tag_name and len(tag_name) >= MIN_KEYWORD_LENGTH:
                # Determine category (just use 'keyword' for now)
                tags_to_insert.append((tag_name, 'keyword'))

    print(f"   Found {len(tags_to_insert)} tags to insert")

    # Insert tags (ignore duplicates)
    cursor = conn.cursor()
    inserted = 0
    for tag_name, category in tags_to_insert:
        try:
            cursor.execute(
                "INSERT INTO tags (name, category) VALUES (%s, %s) ON CONFLICT (name) DO NOTHING",
                (tag_name, category)
            )
            if cursor.rowcount > 0:
                inserted += 1
        except Exception as e:
            print(f"   Warning: Failed to insert tag '{tag_name}': {e}")

    conn.commit()
    print(f"   ✅ Inserted {inserted} new tags")

    # Load tag mapping
    cursor.execute("SELECT id, LOWER(name) FROM tags")
    tag_map = {name: tag_id for tag_id, name in cursor.fetchall()}

    print(f"   ✅ Loaded {len(tag_map)} total tags into memory")
    return tag_map

def process_files(conn, tag_map: Dict[str, int]):
    """
    Process all files and extract tags from paths.
    """
    print("\n📂 Processing files...")

    cursor = conn.cursor(cursor_factory=psycopg2.extras.DictCursor)

    # Get total file count
    cursor.execute("SELECT COUNT(*) FROM files")
    total_files = cursor.fetchone()[0]
    print(f"   Total files: {total_files:,}")

    # Statistics
    files_processed = 0
    tags_found = 0
    batch = []

    # Process files in chunks
    offset = 0
    start_time = time.time()

    while True:
        # Fetch chunk of files
        cursor.execute(
            """
            SELECT id, filepath, filename
            FROM files
            ORDER BY id
            LIMIT %s OFFSET %s
            """,
            (CHUNK_SIZE, offset)
        )

        rows = cursor.fetchall()
        if not rows:
            break

        # Process each file
        for row in rows:
            file_id = row['id']
            filepath = row['filepath'] or ""
            filename = row['filename'] or ""

            # Extract path components
            grandparent, parent, fname = extract_path_components(filepath)

            # Collect all keywords
            all_keywords = set()

            # Grandparent keywords
            all_keywords.update(normalize_keyword(grandparent))

            # Parent keywords
            all_keywords.update(normalize_keyword(parent))

            # Filename keywords
            all_keywords.update(normalize_keyword(fname))
            all_keywords.update(normalize_keyword(filename))

            # Match keywords against tag map
            for keyword in all_keywords:
                if keyword in tag_map:
                    tag_id = tag_map[keyword]
                    batch.append((file_id, tag_id))
                    tags_found += 1

            files_processed += 1

            # Batch insert when batch is full
            if len(batch) >= BATCH_SIZE:
                insert_batch(conn, batch)
                batch = []

            # Progress reporting
            if files_processed % 10000 == 0:
                elapsed = time.time() - start_time
                rate = files_processed / elapsed
                eta = (total_files - files_processed) / rate if rate > 0 else 0
                print(f"   Progress: {files_processed:,}/{total_files:,} files "
                      f"({files_processed/total_files*100:.1f}%) | "
                      f"{rate:.0f} files/sec | "
                      f"ETA: {eta/60:.1f} min | "
                      f"Tags: {tags_found:,}")

        offset += CHUNK_SIZE

    # Insert remaining batch
    if batch:
        insert_batch(conn, batch)

    elapsed = time.time() - start_time
    print(f"\n   ✅ Processed {files_processed:,} files in {elapsed:.1f} seconds")
    print(f"   ✅ Found {tags_found:,} tag relationships")
    print(f"   ✅ Average: {files_processed/elapsed:.0f} files/sec")

def insert_batch(conn, batch: List[Tuple[int, int]]):
    """
    Batch insert file_tags relationships.
    """
    if not batch:
        return

    cursor = conn.cursor()

    # Use execute_values for fast batch insert
    psycopg2.extras.execute_values(
        cursor,
        """
        INSERT INTO file_tags (file_id, tag_id)
        VALUES %s
        ON CONFLICT (file_id, tag_id) DO NOTHING
        """,
        batch,
        page_size=1000
    )

    conn.commit()

def verify_results(conn):
    """
    Verify tagging results and show statistics.
    """
    print("\n📊 Verification and Statistics")
    print("━" * 60)

    cursor = conn.cursor()

    # Files tagged
    cursor.execute("""
        SELECT
            COUNT(DISTINCT file_id) as files_tagged,
            COUNT(*) as total_tags,
            ROUND(AVG(tags_per_file), 2) as avg_tags_per_file
        FROM (
            SELECT file_id, COUNT(*) as tags_per_file
            FROM file_tags
            GROUP BY file_id
        ) t
    """)

    row = cursor.fetchone()
    print(f"Files tagged:        {row[0]:,}")
    print(f"Total tag assignments: {row[1]:,}")
    print(f"Average tags/file:   {row[2]}")

    # Tag distribution
    cursor.execute("""
        SELECT COUNT(*) as tag_count, COUNT(file_id) as file_count
        FROM (
            SELECT file_id, COUNT(*) as tag_count
            FROM file_tags
            GROUP BY file_id
        ) t
        GROUP BY tag_count
        ORDER BY tag_count
        LIMIT 10
    """)

    print("\nTag distribution (files by tag count):")
    for row in cursor.fetchall():
        print(f"  {row[0]} tags: {row[1]:,} files")

    # Top 20 tags
    cursor.execute("""
        SELECT t.name, COUNT(*) as file_count
        FROM file_tags ft
        JOIN tags t ON ft.tag_id = t.id
        GROUP BY t.name
        ORDER BY file_count DESC
        LIMIT 20
    """)

    print("\nTop 20 most common tags:")
    for i, row in enumerate(cursor.fetchall(), 1):
        print(f"  {i:2}. {row[0]:20} - {row[1]:,} files")

def main():
    """
    Main execution function.
    """
    print("━" * 60)
    print("  Fast Multi-Level Tagging")
    print("━" * 60)
    print()

    # Connect to database
    print("🔌 Connecting to database...")
    try:
        conn = psycopg2.connect(DB_URL)
        conn.set_session(autocommit=False)
        print("   ✅ Connected")
    except Exception as e:
        print(f"   ❌ Connection failed: {e}")
        sys.exit(1)

    try:
        # Load curated tags
        tag_map = load_curated_tags(conn)

        # Process files
        process_files(conn, tag_map)

        # Verify results
        verify_results(conn)

        print("\n✅ Tagging complete!")

    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback
        traceback.print_exc()
        conn.rollback()
        sys.exit(1)
    finally:
        conn.close()

if __name__ == "__main__":
    main()

```

### `fixes/fix_add_tags_calls.py` {#fixes-fix-add-tags-calls-py}

- **Lines**: 143 (code: 85, comments: 29, blank: 29)
- **Pylint Score**: 9.47/10 ✅
- **Docstring Coverage**: 50.0%
- **Functions**: fix_add_tags_to_file, fix_add_tags_to_file_impl, process_file, main, replacer1, replacer2, replacer3, replacer

#### Source Code

```python
#!/usr/bin/env python3
"""
Fix add_tags_to_file and add_tags_to_file_impl function call signatures.

Correct signature: (file_id, tag_names, state)
Wrong patterns:
  - add_tags_to_file(&state, file_id, tags) → add_tags_to_file(file_id, tags, &state)
  - add_tags_to_file_impl(file_id, tags) → add_tags_to_file_impl(file_id, tags, &state)
"""

import re
import sys
from pathlib import Path

def fix_add_tags_to_file(content: str) -> tuple[str, int]:
    """Fix add_tags_to_file calls with state-first pattern."""
    fixes = 0

    # Pattern 1: add_tags_to_file(&state, file_id, vec![...])
    pattern1 = r'add_tags_to_file\(\s*&state,\s*(\w+),\s*(vec!\[.*?\]),\s*\)'

    def replacer1(match):
        nonlocal fixes
        file_id = match.group(1)
        tags = match.group(2)
        fixes += 1
        return f'add_tags_to_file({file_id}, {tags}, &state)'

    # Pattern 2: add_tags_to_file(tauri::State(&state), file_id, vec![...])
    pattern2 = r'add_tags_to_file\(\s*tauri::State\(&state\),\s*(\w+),\s*(vec!\[.*?\]),\s*\)'

    def replacer2(match):
        nonlocal fixes
        file_id = match.group(1)
        tags = match.group(2)
        fixes += 1
        return f'add_tags_to_file({file_id}, {tags}, tauri::State(&state))'

    # Apply pattern 1 (simple &state)
    new_content = re.sub(pattern1, replacer1, content, flags=re.DOTALL)

    # Apply pattern 2 (tauri::State wrapper)
    new_content = re.sub(pattern2, replacer2, new_content, flags=re.DOTALL)

    # Handle multi-line formatted calls for pattern 1
    multiline_pattern1 = r'add_tags_to_file\(\s+&state,\s+([^,]+),\s+(vec!\[[^\]]*\]),\s+\)'
    new_content = re.sub(multiline_pattern1, replacer1, new_content, flags=re.DOTALL)

    # Handle multi-line formatted calls for pattern 2
    multiline_pattern2 = r'add_tags_to_file\(\s+tauri::State\(&state\),\s+([^,]+),\s+(vec!\[[^\]]*\]),\s+\)'
    new_content = re.sub(multiline_pattern2, replacer2, new_content, flags=re.DOTALL)

    # Pattern 3: Multi-line with newlines between arguments
    # add_tags_to_file(
    #     tauri::State(&state),
    #     file_id,
    #     tags,
    # )
    multiline_pattern3 = r'add_tags_to_file\(\s+tauri::State\(&state\),\s+([^,]+),\s+([^,\)]+),\s+\)'

    def replacer3(match):
        nonlocal fixes
        file_id = match.group(1).strip()
        tags = match.group(2).strip()
        fixes += 1
        return f'add_tags_to_file({file_id}, {tags}, tauri::State(&state))'

    new_content = re.sub(multiline_pattern3, replacer3, new_content, flags=re.DOTALL)

    return new_content, fixes

def fix_add_tags_to_file_impl(content: str) -> tuple[str, int]:
    """Fix add_tags_to_file_impl calls missing state parameter."""
    fixes = 0

    # Pattern: add_tags_to_file_impl(file_id, vec![...]).await
    # Should be: add_tags_to_file_impl(file_id, vec![...], &state).await
    pattern = r'add_tags_to_file_impl\(([^,]+),\s*(vec!\[[^\]]*\])\)\.await'

    def replacer(match):
        nonlocal fixes
        file_id = match.group(1)
        tags = match.group(2)
        fixes += 1
        return f'add_tags_to_file_impl({file_id}, {tags}, &state).await'

    return re.sub(pattern, replacer, content), fixes

def process_file(filepath: Path) -> bool:
    """Process a single file and return True if changes were made."""
    try:
        content = filepath.read_text()
        original = content

        # Apply both fixes
        content, fixes1 = fix_add_tags_to_file(content)
        content, fixes2 = fix_add_tags_to_file_impl(content)

        total_fixes = fixes1 + fixes2

        if content != original:
            filepath.write_text(content)
            print(f"✓ {filepath.relative_to(Path.cwd())}: {total_fixes} fixes")
            return True
        else:
            print(f"  {filepath.relative_to(Path.cwd())}: no changes needed")
            return False
    except Exception as e:
        print(f"✗ {filepath}: ERROR - {e}", file=sys.stderr)
        return False

def main():
    """Process all test files."""
    base = Path("/home/dojevou/projects/midi-software-center")

    # Find all test files that might contain add_tags_to_file calls
    test_files = [
        base / "pipeline/src-tauri/tests/workflows_test.rs",
        base / "_disabled_tests/workflows_test.rs",
        base / "_disabled_tests/workflows_extended_test.rs",
        base / "_disabled_tests/journey_test.rs",
        base / "_disabled_tests/performance_test.rs",
        base / "_disabled_tests/stress_test.rs",
    ]

    total_files = 0
    changed_files = 0

    print("Fixing add_tags_to_file function signatures...\n")

    for filepath in test_files:
        if filepath.exists():
            total_files += 1
            if process_file(filepath):
                changed_files += 1
        else:
            print(f"⚠ {filepath}: file not found (skipping)")

    print(f"\nSummary: {changed_files}/{total_files} files modified")
    return 0 if changed_files > 0 else 1

if __name__ == "__main__":
    sys.exit(main())

```

### `fixes/fix_e0308_appstate.py` {#fixes-fix-e0308-appstate-py}

- **Lines**: 20 (code: 10, comments: 4, blank: 6)
- **Pylint Score**: 6.00/10 ⚠️

#### Source Code

```python
#!/usr/bin/env python3

import re

file_path = "pipeline/src-tauri/tests/file_import_test.rs"

with open(file_path, 'r') as f:
    lines = f.readlines()

# Pattern: find lines with just "state," and replace with "&state,"
# But be careful not to replace lines that already have &state
for i, line in enumerate(lines):
    # Match lines that have "state," but not "&state,"
    if re.search(r'^\s+state,\s*$', line) and '&state' not in line:
        lines[i] = line.replace('state,', '&state,')

with open(file_path, 'w') as f:
    f.writelines(lines)

print("Fixed E0308 errors - added & to state parameters")

```

### `fixes/fix_e0308_pool.py` {#fixes-fix-e0308-pool-py}

- **Lines**: 46 (code: 27, comments: 10, blank: 9)
- **Pylint Score**: 4.29/10 ❌

#### Source Code

```python
#!/usr/bin/env python3

import re

# Fix pool references in test files
test_files = [
    "pipeline/src-tauri/tests/search_repository_test.rs",
    "pipeline/src-tauri/tests/tag_repository_test.rs",
    "pipeline/src-tauri/tests/file_repository_test.rs",
    "pipeline/src-tauri/tests/metadata_repository_test.rs",
]

for file_path in test_files:
    try:
        with open(file_path, 'r') as f:
            content = f.read()
        
        # Replace patterns like:
        # some_function(pool) with some_function(&pool)
        # But skip if already has &pool
        # This is tricky, let me just fix common patterns
        
        # Pattern: Repository method calls passing pool without &
        # like: search_repository::SearchRepository::search(pool, ...)
        # should be: search_repository::SearchRepository::search(&pool, ...)
        content = re.sub(
            r'SearchRepository::search\(([^&])',
            r'SearchRepository::search(&\1',
            content
        )
        
        # Similar for other repository methods
        content = re.sub(
            r'TagRepository::new\(([^&])',
            r'TagRepository::new(&\1',
            content
        )
        
        with open(file_path, 'w') as f:
            f.write(content)
        
        print(f"Fixed {file_path}")
    except FileNotFoundError:
        pass

print("Fixed pool reference E0308 errors")

```

### `fixes/fix_list_files.py` {#fixes-fix-list-files-py}

- **Lines**: 82 (code: 41, comments: 28, blank: 13)
- **Pylint Score**: 5.75/10 ⚠️
- **Docstring Coverage**: 33.3%
- **Functions**: fix_list_files_calls, main, replace_fn

#### Source Code

```python
#!/usr/bin/env python3
"""
Fix list_files() E0061 errors in workflows_test.rs
Converts test calls from list_files() to list_files_impl() with correct argument order

FUNCTION SIGNATURES:
  ✓ list_files_impl(limit: Option<i64>, offset: Option<i64>, state: &AppState)
  ✗ list_files(limit: Option<i64>, offset: Option<i64>, state: State<AppState>)

TEST PATTERN FIX:
  BEFORE: list_files(&state, Some(1), Some(10), None).await
  AFTER:  list_files_impl(Some(1), Some(10), &state).await

AFFECTED FILE:
  pipeline/src-tauri/tests/workflows_test.rs (Line 283)

ERRORS FIXED: 1 E0061 error (part of larger 82 → 38 reduction via _impl migration)
"""

import re
import sys

def fix_list_files_calls(content):
    """
    Convert list_files(&state, Some(x), Some(y), ...) 
    to list_files_impl(Some(x), Some(y), &state)
    
    Regex Pattern:
    - Match: list_files( followed by &state, and two Some(...) arguments
    - Capture: the two Some arguments and any trailing args
    - Replace: with list_files_impl(arg1, arg2, &state)
    - Handles: multiline formatting via DOTALL flag
    """
    
    pattern = r'list_files\(\s*&state,\s*(Some\([^)]*\)),\s*(Some\([^)]*\)),\s*([^)]*)\s*\)\.await'
    
    def replace_fn(match):
        limit_arg = match.group(1)
        offset_arg = match.group(2)
        return f"list_files_impl({limit_arg}, {offset_arg}, &state).await"
    
    return re.sub(pattern, replace_fn, content, flags=re.MULTILINE | re.DOTALL)

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 fix_list_files.py <file_path>")
        print("Example: python3 fix_list_files.py pipeline/src-tauri/tests/workflows_test.rs")
        sys.exit(1)
    
    file_path = sys.argv[1]
    
    try:
        with open(file_path, 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"ERROR: File not found: {file_path}")
        sys.exit(1)
    
    original_content = content
    count_before = len(re.findall(r'list_files\(\s*&state,', content))
    
    content = fix_list_files_calls(content)
    count_after = len(re.findall(r'list_files\(\s*&state,', content))
    
    if content != original_content:
        with open(file_path, 'w') as f:
            f.write(content)
        
        fixed_count = count_before - count_after
        print(f"\n✓ TRANSFORMATION COMPLETE")
        print(f"  Fixed: {fixed_count} list_files() call(s)")
        print(f"  File: {file_path}")
        print(f"\n  Pattern: list_files(&state, ...) → list_files_impl(..., &state)")
        print(f"  Line 283: let files = list_files_impl(Some(1), Some(10), &state).await.unwrap();")
        return 0
    else:
        print(f"✗ No matching calls found: list_files(&state, ...)")
        print(f"  File: {file_path}")
        return 1

if __name__ == '__main__':
    sys.exit(main())

```

### `fixes/format_string_fixer.py` {#fixes-format-string-fixer-py}

- **Lines**: 198 (code: 123, comments: 34, blank: 41)
- **Pylint Score**: 4.51/10 ❌
- **Docstring Coverage**: 50.0%
- **Functions**: __init__, _compile_patterns, find_rust_files, analyze_format_string, fix_file, run, replace_indexed_with_empty, replace_indexed_format, replace_write_indexed
- **Classes**: FormatStringFixer

#### Source Code

```python
#!/usr/bin/env python3
"""
Format String Fixer - Automatically fixes format string errors in Rust code
Category 1: 28 errors of type "invalid reference to positional argument N"
"""

import re
import sys
from pathlib import Path
from typing import List, Tuple, Dict
import subprocess

class FormatStringFixer:
    def __init__(self, rust_root: str = '.'):
        self.rust_root = Path(rust_root)
        self.patterns = self._compile_patterns()
        self.fixes_applied = 0
        self.files_modified = 0
    
    def _compile_patterns(self) -> Dict[str, re.Pattern]:
        """Compile regex patterns for format string errors"""
        return {
            # format!("{0}") with no args
            'format_indexed': re.compile(r'format!\("([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # println!("{0}") with no args
            'println_indexed': re.compile(r'println!\("([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # eprintln!("{0}") with no args
            'eprintln_indexed': re.compile(r'eprintln!\("([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # write!/writeln! with indexed args but no values
            'write_indexed': re.compile(r'write[ln]?\(&?[^,]+,\s*"([^"]*\{(\d+)[^"]*)"(?:\s*,?\s*)?(?=\))', re.MULTILINE),
            
            # format!("{1}") but only 1 arg exists - this is trickier
            'format_mismatch': re.compile(
                r'(format|println|eprintln|writeln?)\!\("([^"]*\{(\d+)[^"]*)"(?:,\s*([^\)]+))?\)',
                re.MULTILINE
            ),
        }
    
    def find_rust_files(self) -> List[Path]:
        """Find all Rust files in project"""
        return list(self.rust_root.rglob('*.rs'))
    
    def analyze_format_string(self, format_str: str, args: str = "") -> Tuple[int, List[int]]:
        """
        Analyze format string and extract referenced indices
        Returns: (max_index, list_of_indices)
        """
        # Find all {N} patterns
        indices = [int(m.group(1)) for m in re.finditer(r'\{(\d+)\}', format_str)]
        
        # Count actual arguments
        if not args:
            arg_count = 0
        else:
            # Simple count: split by comma, but handle nested structures
            # This is approximate - real parsing would be more complex
            arg_count = len([a.strip() for a in args.split(',') if a.strip()])
        
        return (max(indices) if indices else -1, indices, arg_count)
    
    def fix_file(self, file_path: Path) -> int:
        """
        Fix format string errors in a single file
        Returns number of fixes applied
        """
        try:
            content = file_path.read_text(encoding='utf-8')
        except Exception as e:
            print(f"  ❌ Cannot read {file_path}: {e}")
            return 0
        
        original_content = content
        fixes = 0
        
        # Strategy 1: Remove indexed arguments that reference nothing
        # Pattern: {0} with no arguments → {} (empty placeholder)
        pattern1 = re.compile(
            r'(format|println|eprintln|writeln?|write)\!\(\s*"([^"]*\{(\d+)[^"]*)"(?:\s*(?:,\s*(&?\w+))?)\s*\)',
            re.MULTILINE
        )
        
        def replace_indexed_with_empty(match):
            nonlocal fixes
            macro_name = match.group(1)
            format_str = match.group(2)
            indexed = match.group(3)
            args = match.group(4) if match.group(4) else ""
            
            # Replace {N} with {} if no corresponding args
            new_format = re.sub(r'\{\d+\}', '{}', format_str)
            
            # Build new macro call
            if args:
                result = f'{macro_name}!("{new_format}", {args})'
            else:
                result = f'{macro_name}!("{new_format}")'
            
            fixes += 1
            return result
        
        content = pattern1.sub(replace_indexed_with_empty, content)
        
        # Strategy 2: Handle format!("text {0}", value) → format!("text {}", value)
        pattern2 = re.compile(
            r'(format|println|eprintln)\!\("([^"]*)\{(\d+)\}([^"]*)"([^)]*)\)',
            re.MULTILINE
        )
        
        def replace_indexed_format(match):
            nonlocal fixes
            macro = match.group(1)
            before = match.group(2)
            index = match.group(3)
            after = match.group(4)
            args = match.group(5)
            
            # Check if we have enough args
            arg_list = [a.strip() for a in args.split(',') if a.strip() and a.strip() != '']
            
            if len(arg_list) > 0:
                # We have args, so just remove the index
                new_format = f'{before}{{}}{after}'
                fixes += 1
                return f'{macro}!("{new_format}"{args})'
            
            return match.group(0)  # Leave unchanged if no args
        
        content = pattern2.sub(replace_indexed_format, content)
        
        # Strategy 3: writeln!(stream, "{0}") with args
        pattern3 = re.compile(
            r'writeln?\(&?(\w+),\s*"([^"]*\{(\d+)[^"]*)"([^)]*)\)',
            re.MULTILINE
        )
        
        def replace_write_indexed(match):
            nonlocal fixes
            stream = match.group(1)
            format_str = match.group(2)
            index = match.group(3)
            args = match.group(4)
            
            # Replace {N} with {}
            new_format = re.sub(r'\{(\d+)\}', '{}', format_str)
            fixes += 1
            return f'writeln!(&{stream}, "{new_format}"{args})'
        
        content = pattern3.sub(replace_write_indexed, content)
        
        if content != original_content:
            file_path.write_text(content, encoding='utf-8')
            self.files_modified += 1
            return fixes
        
        return 0
    
    def run(self) -> bool:
        """Run the fixer on all Rust files"""
        rust_files = self.find_rust_files()
        
        if not rust_files:
            print(f"❌ No Rust files found in {self.rust_root}")
            return False
        
        print(f"🔍 Found {len(rust_files)} Rust files")
        print(f"🔧 Fixing format string errors...\n")
        
        for file_path in rust_files:
            fixes = self.fix_file(file_path)
            if fixes > 0:
                print(f"  ✅ {file_path.relative_to(self.rust_root)}: {fixes} fixes")
                self.fixes_applied += fixes
        
        print(f"\n{'='*60}")
        print(f"📊 RESULTS")
        print(f"{'='*60}")
        print(f"Files modified:     {self.files_modified}")
        print(f"Total fixes:        {self.fixes_applied}")
        
        if self.fixes_applied > 0:
            print(f"\n✅ Format string errors fixed!")
            print(f"Next: Run 'cargo build' to verify compilation")
            return True
        else:
            print(f"\n⚠️  No format string errors found/fixed")
            return False


if __name__ == '__main__':
    rust_root = sys.argv[1] if len(sys.argv) > 1 else '.'
    
    fixer = FormatStringFixer(rust_root)
    success = fixer.run()
    
    sys.exit(0 if success else 1)

```

### `generate_detailed_report.py` {#generate-detailed-report-py}

- **Lines**: 356 (code: 275, comments: 21, blank: 60)
- **Pylint Score**: 9.35/10 ✅
- **Docstring Coverage**: 100.0%
- **Functions**: load_analysis, generate_comprehensive_report

#### Source Code

```python
#!/usr/bin/env python3
"""
Generate detailed, comprehensive MIDI collection report with full categorization
"""

import json
from pathlib import Path

def load_analysis(filepath):
    """Load the basic analysis results"""
    # For now, we'll read from the existing report
    # In production, we'd load from JSON
    return {}

def generate_comprehensive_report():
    """Generate the full detailed report"""

    # Read the basic analysis
    basic_report = Path("COMPLETE_COLLECTION_ANALYSIS.md").read_text()

    lines = []

    # Header with collection overview
    lines.append("# Complete MIDI Collection Analysis - All 9.3 Million Files\n\n")
    lines.append("**Total Files Analyzed:** 9,301,753\n")
    lines.append("**Drum Files:** 7,155,382 (76.9%)\n")
    lines.append("**Collection Size:** 72GB\n")
    lines.append("**Analysis Speed:** 83,405 files/sec\n")
    lines.append("**Processing Time:** 111.5 seconds\n\n")
    lines.append("---\n\n")

    # Add the basic statistics section
    lines.append(basic_report.split("---\n\n")[1].split("\n## BPM Ranges Summary")[0])

    # Add detailed BPM ranges
    lines.append("\n---\n\n")
    lines.append("## 🎵 BPM RANGES (Specific Values Found in Collection)\n\n")

    lines.append("### Very Slow (30-60 BPM)\n")
    lines.append("30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60\n\n")

    lines.append("### Slow (61-90 BPM)\n")
    lines.append("61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90\n\n")

    lines.append("### Mid-Tempo (91-120 BPM)\n")
    lines.append("91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120\n\n")

    lines.append("### Upbeat (121-140 BPM)\n")
    lines.append("121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140\n\n")

    lines.append("### Fast (141-180 BPM)\n")
    lines.append("141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180\n\n")

    lines.append("### Very Fast (181-300 BPM)\n")
    lines.append("181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 210, 212, 214, 216, 220, 224, 225, 226, 228, 230, 240, 250, 260, 270, 280, 290, 300\n\n")

    lines.append("---\n\n")

    # GENRE TAGS with categorization
    lines.append("## 🎵 GENRE TAGS (Real-World Categories)\n\n")

    lines.append("### Electronic Dance Music (EDM)\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| house | House, Deep House, Tech House | 120-130 | 219,166 |\n")
    lines.append("| techno | Techno, Detroit Techno | 124-135 | 95,534 |\n")
    lines.append("| trance | Trance, Psy Trance | 130-140 | 118,993 |\n")
    lines.append("| dubstep | Dubstep, Brostep | 140 | 57,964 |\n")
    lines.append("| dnb | Drum & Bass, DnB, Jungle | 170-180 | 89,658 |\n")
    lines.append("| glitch | Glitch Hop, IDM | 90-110 | 4,209 |\n")
    lines.append("| ambient | Ambient, Atmospheric | 60-90 | 24,750 |\n")
    lines.append("| breakbeat | Breakbeat, Breaks | 130-140 | 84,184 |\n")
    lines.append("| garage | Garage, UK Garage | 130-140 | 41,835 |\n")
    lines.append("| jungle | Jungle, Ragga Jungle | 160-180 | 30,312 |\n")
    lines.append("| electro | Electro, Electronica | 120-130 | 340,841 |\n")
    lines.append("| edm | EDM, Festival EDM | 128-140 | 146,238 |\n\n")

    lines.append("### Hip-Hop & Urban\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| hip-hop | Hip Hop, Hip-Hop | 80-100 | 161,768 |\n")
    lines.append("| trap | Trap, Melodic Trap | 140-160 | 99,070 |\n")
    lines.append("| rnb | R&B, RnB, Soul | 70-90 | 132,210 |\n")
    lines.append("| pop | Pop, Future Pop | 100-128 | 337,125 |\n\n")

    lines.append("### Rock & Metal\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| rock | Rock, Hard Rock, Alt-Rock | 120-140 | 1,239,243 |\n")
    lines.append("| metal | Metal, Black Metal, Death Metal | 140-200 | 918,097 |\n")
    lines.append("| punk | Punk, Punk Rock | 160-180 | 150,035 |\n")
    lines.append("| blues | Blues, Blues Rock | 80-120 | 281,876 |\n")
    lines.append("| funk | Funk, Funk Rock | 100-120 | 479,729 |\n")
    lines.append("| progressive | Progressive Rock/Metal | 140-180 | 530,243 |\n")
    lines.append("| indie | Indie Rock, Indie Pop | 100-140 | 25,261 |\n\n")

    lines.append("### Jazz & Traditional\n")
    lines.append("| Tag | Variations Found | BPM Range | Count |\n")
    lines.append("|-----|------------------|-----------|-------|\n")
    lines.append("| jazz | Jazz, Swing Jazz, Bebop | 100-180 | 445,284 |\n")
    lines.append("| fusion | Jazz Fusion, Fusion | 110-140 | 233,135 |\n")
    lines.append("| swing | Swing, Big Band | 120-180 | 393,187 |\n\n")

    lines.append("### World & Ethnic\n")
    lines.append("| Tag | Variations Found | Count |\n")
    lines.append("|-----|------------------|-------|\n")
    lines.append("| african | Africa, African Rhythms | 33,479 |\n")
    lines.append("| asian | Asia, Asian Traditional | 1,812 |\n")
    lines.append("| latin | Latin, Latin Percussion | 196,982 |\n")
    lines.append("| world | World, World Music | 202,454 |\n")
    lines.append("| ethnic | Ethnic, World Beats, Ethno | 12,419 |\n")
    lines.append("| reggae | Reggae, Dub | 20,393 |\n")
    lines.append("| caribbean | Caribbean, Island | 464 |\n\n")

    lines.append("### Specialized\n")
    lines.append("| Tag | Description | Count |\n")
    lines.append("|-----|-------------|-------|\n")
    lines.append("| disco | Disco, 80s style | 43,424 |\n")
    lines.append("| industrial | Industrial, EBM | 3,833 |\n\n")

    lines.append("---\n\n")

    # DRUM CATEGORIES
    lines.append("## 🥁 DRUM CATEGORIES (from 7.2M+ Drum Files)\n\n")

    lines.append("### Drum Kit Types\n")
    lines.append("| Category | Sub-Categories | Common BPM | Files |\n")
    lines.append("|----------|----------------|------------|-------|\n")
    lines.append("| **Acoustic Drums** | Blues, Jazz, Rock, Funk, Metal | Varies | ~3.5M |\n")
    lines.append("| **Electronic Drums** | 808, 909, Synthetic, Electric | 120-140 | ~2.8M |\n")
    lines.append("| **Analogue Drums** | Vintage, Retro | 100-130 | ~0.6M |\n")
    lines.append("| **World Percussion** | Latin, African, Asian, Middle East | Varies | ~0.3M |\n\n")

    lines.append("### Drum Components\n")
    lines.append("| Instrument | Variations | Count |\n")
    lines.append("|------------|------------|-------|\n")
    lines.append("| kick | Bass Drum, Kick Drum, Sub Kick | 426,266 |\n")
    lines.append("| snare | Snare, Sidestick, Rimshot, Clap | 121,923 |\n")
    lines.append("| hat | Hi-Hat, Closed Hat, Open Hat, Pedal Hat | 1,650,996 |\n")
    lines.append("| cymbal | Crash, Ride, China, Splash | 75,181 |\n")
    lines.append("| tom | Floor Tom, Rack Tom, Hi Tom, Low Tom | 391,885 |\n")
    lines.append("| percussion | Conga, Bongo, Tambourine, Cowbell, Shaker | 237,299 |\n")
    lines.append("| crash | Crash Cymbal, Crash 1, Crash 2 | 427,053 |\n")
    lines.append("| ride | Ride Cymbal, Ride Bell | 1,155,507 |\n\n")

    lines.append("### Drum Styles (from folder/path analysis)\n")
    lines.append("- Blues Drums (281,876+ files)\n")
    lines.append("- Jazz Drums (445,284+ files)\n")
    lines.append("- Rock Drums (1,239,243+ files)\n")
    lines.append("- Metal Drums (918,097+ files)\n")
    lines.append("- Funk Drums (479,729+ files)\n")
    lines.append("- Electronic Drums (340,841+ files)\n")
    lines.append("- Progressive Drums (530,243+ files)\n")
    lines.append("- Punk Drums (150,035+ files)\n")
    lines.append("- Swing Drums (393,187+ files)\n")
    lines.append("- World/Ethnic Drums (202,454+ files)\n\n")

    lines.append("---\n\n")

    # MELODIC INSTRUMENTS
    lines.append("## 🎹 MELODIC INSTRUMENTS\n\n")

    lines.append("### Bass\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Synth Bass** | bass, synth-bass, sub-bass, wobble | 278,455 |\n")
    lines.append("| **Acoustic Bass** | acoustic-bass, upright-bass | (included) |\n")
    lines.append("| **Electric Bass** | electric-bass, bass-guitar, slap-bass | (included) |\n\n")

    lines.append("### Keys\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Piano** | piano, grand-piano, electric-piano | 138,020 |\n")
    lines.append("| **Organ** | organ, hammond, church-organ | 6,645 |\n")
    lines.append("| **Synth** | synth, lead, pad, arp | 829,614 |\n")
    lines.append("| **Chords** | chords, stabs, progressions | 185,066 |\n")
    lines.append("| **Keys** | keys, keyboard | 48,088 |\n\n")

    lines.append("### Leads & Melody\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Lead** | lead, synth-lead, melody | 104,200 |\n")
    lines.append("| **Arp** | arp, arpeggios, arpeggiator | 30,091 |\n")
    lines.append("| **Pad** | pad, synth-pad, atmospheric | 55,768 |\n\n")

    lines.append("### Strings & Brass\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Strings** | strings, orchestra, violin, cello | 26,823 |\n")
    lines.append("| **Brass** | brass, trumpet, trombone, horn | 16,584 |\n\n")

    lines.append("### Guitar\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Electric** | guitar, electric-guitar, distortion | 17,552 |\n")
    lines.append("| **Acoustic** | acoustic-guitar, nylon, steel | (included) |\n\n")

    lines.append("### Vocals & FX\n")
    lines.append("| Type | Common Tags | Count |\n")
    lines.append("|------|-------------|-------|\n")
    lines.append("| **Vocals** | vocal, voice, choir | 16,020 |\n")
    lines.append("| **FX** | fx, effect, sfx | 26,937 |\n\n")

    lines.append("---\n\n")

    # MUSICAL ELEMENTS
    lines.append("## 🎼 MUSICAL ELEMENTS\n\n")

    lines.append("### Pattern Types\n")
    lines.append("| Tag | Description | Count |\n")
    lines.append("|-----|-------------|-------|\n")
    lines.append("| groove | Main rhythmic pattern | 2,717,157 |\n")
    lines.append("| fill | Transitional fill | 1,626,253 |\n")
    lines.append("| intro | Song introduction | 219,442 |\n")
    lines.append("| ending | Song outro/ending | 101,387 |\n")
    lines.append("| breakdown | Breakdown section | 3,777 |\n")
    lines.append("| turnaround | Turnaround pattern | 374 |\n")
    lines.append("| verse | Verse section | 171,772 |\n")
    lines.append("| chorus | Chorus section | 139,403 |\n")
    lines.append("| bridge | Bridge section | 83,561 |\n")
    lines.append("| build | Build-up section | 31,831 |\n")
    lines.append("| drop | Drop section | 34,969 |\n\n")

    lines.append("### Musical Characteristics\n")
    lines.append("| Tag | Description | Count |\n")
    lines.append("|-----|-------------|-------|\n")
    lines.append("| loop | Loopable content | 537,350 |\n")
    lines.append("| one-shot | Single hit/sample | 465 |\n\n")

    lines.append("---\n\n")

    # TIME SIGNATURES
    lines.append("## 🎼 TIME SIGNATURES (Found in Collection)\n\n")
    lines.append("| Time Signature | Common Usage | Count |\n")
    lines.append("|----------------|--------------|-------|\n")
    lines.append("| 4/4 (Four-Four) | Most common - Rock, Pop, EDM, Hip-Hop | (default - majority) |\n")
    lines.append("| 3/4 (Three-Four) | Waltz, some Metal | 346,689 |\n")
    lines.append("| 6/8 (Six-Eight) | Ballads, Blues, Swing | 312,328 |\n")
    lines.append("| 5/4 (Five-Four) | Progressive, Jazz | 59,982 |\n")
    lines.append("| 7/8 (Seven-Eight) | Progressive, Fusion | 51,463 |\n\n")

    lines.append("---\n\n")

    # KEY SIGNATURES
    lines.append("## 🎹 KEY SIGNATURES (Musical Keys)\n\n")

    lines.append("### Major Keys (Top 12)\n")
    lines.append("| Key | Count | Common Usage |\n")
    lines.append("|-----|-------|-------------|\n")
    lines.append("| C | 78,698 | Most common, no sharps/flats |\n")
    lines.append("| A | 36,628 | 3 sharps |\n")
    lines.append("| F | 28,089 | 1 flat |\n")
    lines.append("| B | 23,778 | 5 sharps |\n")
    lines.append("| D | 23,324 | 2 sharps |\n")
    lines.append("| G | 21,229 | 1 sharp |\n")
    lines.append("| E | 17,177 | 4 sharps |\n")
    lines.append("| Bb | 5,613 | 2 flats |\n")
    lines.append("| D# | 5,492 | 3 sharps |\n")
    lines.append("| Eb | 5,019 | 3 flats |\n")
    lines.append("| F# | 4,827 | 6 sharps |\n")
    lines.append("| A# | 3,621 | 4 sharps |\n\n")

    lines.append("### Minor Keys (Top 12)\n")
    lines.append("| Key | Count | Relative Major |\n")
    lines.append("|-----|-------|----------------|\n")
    lines.append("| Am | 7,977 | C major |\n")
    lines.append("| Fm | 7,097 | Ab major |\n")
    lines.append("| Cm | 6,819 | Eb major |\n")
    lines.append("| Gm | 5,515 | Bb major |\n")
    lines.append("| Dm | 5,428 | F major |\n")
    lines.append("| Em | 5,058 | G major |\n")
    lines.append("| Bm | 2,613 | D major |\n")
    lines.append("| F#m | 1,420 | A major |\n")
    lines.append("| Bbm | 1,377 | Db major |\n")
    lines.append("| C#m | 1,208 | E major |\n")
    lines.append("| D#m | 1,188 | F# major |\n")
    lines.append("| Ebm | 1,137 | Gb major |\n\n")

    lines.append("### Common Notation Patterns (found in filenames)\n")
    lines.append("- `_C_` / `_Cmaj_` / `_C_major_`\n")
    lines.append("- `_Cm_` / `_Cmin_` / `_C_minor_`\n")
    lines.append("- `_F#_` / `_Fsharp_` / `_F#maj_`\n")
    lines.append("- `_Am_` / `_Amin_` / `_A_minor_`\n")
    lines.append("- `_Bb_` / `_Bflat_` / `_Bb_major_`\n\n")

    lines.append("**Note:** Only ~330,000 files (3.5%) have keys explicitly in filenames. The remaining files will need Phase 4 analysis for key detection.\n\n")

    lines.append("---\n\n")

    # DRUM TECHNIQUES
    lines.append("## 🥁 DRUM TECHNIQUES & FEEL (from 7.2M drum files)\n\n")

    lines.append("### Rhythmic Feel\n")
    lines.append("| Feel | Description | Count | % of Drums |\n")
    lines.append("|------|-------------|-------|------------|\n")
    lines.append("| straight | Straight time, even 8ths/16ths | 2,146,238 | 29.99% |\n")
    lines.append("| shuffle | Shuffle feel, triplet-based | 405,721 | 5.67% |\n")
    lines.append("| swing | Swing feel, jazz swing | 332,419 | 4.65% |\n")
    lines.append("| triplet | Triplet-based patterns | 160,571 | 2.24% |\n\n")

    lines.append("### Cymbal Techniques\n")
    lines.append("| Technique | Description | Count | % of Drums |\n")
    lines.append("|-----------|-------------|-------|------------|\n")
    lines.append("| ride | Ride cymbal patterns | 1,076,591 | 15.05% |\n")
    lines.append("| crash | Crash cymbal accents | 396,045 | 5.53% |\n")
    lines.append("| open-hat | Open hi-hat | 160,347 | 2.24% |\n")
    lines.append("| closed-hat | Closed hi-hat | 114,435 | 1.60% |\n")
    lines.append("| china | China cymbal | 47,493 | 0.66% |\n")
    lines.append("| splash | Splash cymbal | 1,474 | 0.02% |\n\n")

    lines.append("### Special Techniques\n")
    lines.append("| Technique | Description | Count | % of Drums |\n")
    lines.append("|-----------|-------------|-------|------------|\n")
    lines.append("| ghost-notes | Ghost notes, quiet accents | 105,060 | 1.47% |\n")
    lines.append("| roll | Drum rolls | 85,307 | 1.19% |\n")
    lines.append("| flam | Flam rudiments | 42,080 | 0.59% |\n")
    lines.append("| double-bass | Double bass drum | 42 | 0.00% |\n\n")

    lines.append("---\n\n")

    # COLLECTION INSIGHTS
    lines.append("## 📊 COLLECTION INSIGHTS\n\n")

    lines.append("### Top Findings\n")
    lines.append("1. **Drum-Heavy Collection**: 76.9% (7.2M files) are drum/percussion files\n")
    lines.append("2. **Rock Dominance**: Rock (13.3%) and Metal (9.9%) are top genres\n")
    lines.append("3. **Groove-Focused**: 29.2% of files are labeled as grooves, 17.5% as fills\n")
    lines.append("4. **Key Information Sparse**: Only 3.5% have keys in filenames\n")
    lines.append("5. **BPM Coverage**: Wide range from 30-300 BPM, with peaks at 120, 140, 128\n")
    lines.append("6. **Time Signatures**: Majority 4/4, but 770K+ files use non-standard signatures\n\n")

    lines.append("### Genre Distribution Analysis\n")
    lines.append("- **Rock/Metal**: 23.2% (2.16M files)\n")
    lines.append("- **Electronic/EDM**: 11.2% (1.04M files)\n")
    lines.append("- **Jazz/Fusion**: 7.3% (678K files)\n")
    lines.append("- **Funk/Blues**: 8.2% (761K files)\n")
    lines.append("- **World/Ethnic**: 4.9% (458K files)\n")
    lines.append("- **Hip-Hop/Urban**: 4.2% (393K files)\n\n")

    lines.append("### Recommendations for Pipeline\n")
    lines.append("1. **Prioritize Drum Analysis**: Use enhanced drum_analyzer.rs for 7.2M drum files\n")
    lines.append("2. **Key Detection Critical**: 96.5% need Phase 4 analysis for key detection\n")
    lines.append("3. **BPM Extraction**: Already good coverage from filenames (14% have BPM)\n")
    lines.append("4. **Genre Auto-Tagging**: Strong folder/path structure provides genre context\n")
    lines.append("5. **Pattern Recognition**: Groove/fill detection will benefit from MIDI analysis\n\n")

    lines.append("---\n\n")
    lines.append("**Analysis Complete** | Generated from 9,301,753 MIDI files | Total collection: 72GB\n")

    # Write report
    output = ''.join(lines)
    Path("COMPLETE_COLLECTION_ANALYSIS_DETAILED.md").write_text(output)
    print("Detailed report generated: COMPLETE_COLLECTION_ANALYSIS_DETAILED.md")

if __name__ == "__main__":
    generate_comprehensive_report()

```

### `grok/grok4_project_reviewer.py` {#grok-grok4-project-reviewer-py}

- **Lines**: 184 (code: 115, comments: 38, blank: 31)
- **Pylint Score**: 9.43/10 ✅
- **Docstring Coverage**: 100.0%
- **Functions**: main, __init__, call_grok, analyze_compilation_errors, close
- **Classes**: Grok4ProjectReviewer

#### Source Code

```python
#!/usr/bin/env python3
"""
Grok 4 Project Reviewer for MIDI Software Center
Analyzes compilation errors, architecture, and code quality
Uses xAI's Grok 4 Fast Reasoning API for intelligent code review
"""

import os
import sys
from pathlib import Path
import httpx
import time


class Grok4ProjectReviewer:
    """Integrates Grok API for comprehensive project review"""

    def __init__(self, api_key=None):
        """Initialize Grok API client"""
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path("/home/dojevou/projects/midi-software-center")

        # Browser-like headers that xAI expects
        self.default_headers = {
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Accept": "application/json, text/plain, */*",
            "Accept-Language": "en-US,en;q=0.9",
            "Accept-Encoding": "gzip, deflate, br",
            "Connection": "keep-alive",
            "Sec-Fetch-Dest": "empty",
            "Sec-Fetch-Mode": "cors",
            "Sec-Fetch-Site": "same-site",
            "Cache-Control": "no-cache",
            "Pragma": "no-cache",
        }

        self.client = httpx.Client(timeout=120.0, http2=True, follow_redirects=True)

        if not self.api_key:
            print("⚠️  GROK_API_KEY not set. Set it with:")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    def call_grok(self, prompt: str, model: str = "grok-4-fast-reasoning") -> str:
        """Call Grok API and return response with retry logic"""
        if not self.api_key:
            return "ERROR: Grok API key not configured"

        # Merge default headers with auth
        headers = {
            **self.default_headers,
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 3000,
        }

        max_retries = 3
        retry_count = 0

        while retry_count < max_retries:
            try:
                response = self.client.post(
                    "https://api.x.ai/v1/chat/completions",
                    json=payload,
                    headers=headers,
                )

                # Check for rate limiting
                if response.status_code == 429:
                    wait_time = 2**retry_count  # Exponential backoff
                    print(f"⏳ Rate limited. Waiting {wait_time}s before retry...")
                    time.sleep(wait_time)
                    retry_count += 1
                    continue

                # Check for auth errors
                if response.status_code == 401:
                    return "❌ ERROR: Invalid API key. Check GROK_API_KEY."

                if response.status_code == 403:
                    return "❌ ERROR: 403 Forbidden - API key may lack permissions or account has restrictions."

                response.raise_for_status()
                result = response.json()

                if "choices" not in result or len(result["choices"]) == 0:
                    return f"ERROR: Unexpected API response format: {result}"

                return result["choices"][0]["message"]["content"]

            except httpx.ConnectError as e:
                print(f"⚠️  Connection error: {e}")
                if retry_count < max_retries - 1:
                    wait_time = 2**retry_count
                    print(f"⏳ Retrying in {wait_time}s...")
                    time.sleep(wait_time)
                    retry_count += 1
                else:
                    return f"ERROR: Failed to connect after {max_retries} retries"

            except httpx.HTTPStatusError as e:
                return f"ERROR: HTTP {e.response.status_code} - {e.response.text}"

            except Exception as e:
                return f"ERROR: {str(e)}"

        return "ERROR: Max retries exceeded"

    def analyze_compilation_errors(self):
        """Analyze current compilation errors using Grok"""
        print("🔍 Collecting compilation errors...")
        print("📊 Analyzing your MIDI software project...\n")

        prompt = """You are an expert Rust/Tauri developer. Analyze this MIDI software project:

CONTEXT:
- Desktop MIDI production software (Tauri + Rust + PostgreSQL)
- Phase 9 Extended (final quality push)
- Recent work: Phase 5 created _impl wrapper functions for tests
- Current issue: ~500+ test compilation errors remaining

ANALYSIS NEEDED:
1. What are the PRIMARY blocking issues?
2. What should be fixed FIRST (highest impact)?
3. Estimated effort and timeline
4. Specific files/patterns to focus on
5. Success metrics

PROVIDE:
- Clear, actionable recommendations
- Quick wins vs long-term fixes
- Realistic timeline to reach 0 errors
- Architecture improvements after fixes

Be direct and concise. Focus on what matters most."""

        print("🚀 Sending to Grok for analysis...\n")
        print("=" * 70)
        analysis = self.call_grok(prompt)
        print(analysis)
        print("=" * 70)
        print("\n✅ Analysis Complete!")

    def close(self):
        """Close HTTP client"""
        self.client.close()


def main():
    """Main entry point"""
    reviewer = Grok4ProjectReviewer()

    try:
        if len(sys.argv) > 1:
            if sys.argv[1] in ["--analyze-errors", "errors"]:
                reviewer.analyze_compilation_errors()
            elif sys.argv[1] in ["--full-audit", "full"]:
                reviewer.analyze_compilation_errors()
            else:
                print("Grok Project Reviewer")
                print("\nUsage: python3 grok4_project_reviewer.py [command]")
                print("\nCommands:")
                print("  errors              Analyze compilation errors")
                print("  --analyze-errors    Analyze compilation errors")
                print("  full                Run complete audit")
                print("  --full-audit        Run complete audit")
        else:
            print("Grok Project Reviewer")
            print("\nAvailable options:")
            print("  python3 grok4_project_reviewer.py errors")
            print("  python3 grok4_project_reviewer.py full")
            print("\nExample: python3 grok4_project_reviewer.py errors")
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()

```

### `grok/midi_grok_reviewer.py` {#grok-midi-grok-reviewer-py}

- **Lines**: 797 (code: 495, comments: 209, blank: 93)
- **Pylint Score**: 8.94/10 ✅
- **Docstring Coverage**: 97.1%
- **Functions**: main, __init__, gather_project_context, detect_project_phase, analyze_midi_architecture, analyze_rust_backends, analyze_backend, analyze_tauri_frontends, analyze_frontend, analyze_database_layer
  *... and 24 more*
- **Classes**: MIDIProjectReviewer

#### Source Code

```python
#!/usr/bin/env python3
"""
Enhanced Grok 4 Project Reviewer for MIDI Software Center
Targets: Rust backends, Tauri apps, Svelte frontends, PostgreSQL database
"""

import os
import sys
import json
import time
import subprocess
from pathlib import Path
import httpx
from datetime import datetime


class MIDIProjectReviewer:
    """Enhanced code reviewer specifically for MIDI Software Center"""

    def __init__(self, api_key=None):
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path("/home/dojevou/projects/midi-software-center")
        self.analysis_history = []

        # Browser-like headers that xAI expects
        self.default_headers = {
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Accept": "application/json, text/plain, */*",
            "Accept-Language": "en-US,en;q=0.9",
            "Accept-Encoding": "gzip, deflate, br",
            "Connection": "keep-alive",
            "Sec-Fetch-Dest": "empty",
            "Sec-Fetch-Mode": "cors",
            "Sec-Fetch-Site": "same-site",
            "Cache-Control": "no-cache",
            "Pragma": "no-cache",
        }

        self.client = httpx.Client(timeout=120.0, http2=True, follow_redirects=True)

        if not self.api_key:
            print("⚠️  GROK_API_KEY not set. Set it with:")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    # 🚀 IMPROVEMENT 1: TARGETED MIDI PROJECT ANALYSIS
    def gather_project_context(self):
        """Collect specific data for MIDI software project"""
        print("🎵 Gathering MIDI Software Center context...")

        context = {
            "timestamp": datetime.now().isoformat(),
            "project_phase": self.detect_project_phase(),
            "architecture": self.analyze_midi_architecture(),
            "build_errors": self.collect_midi_build_errors(),
            "test_status": self.analyze_test_health(),
            "database_state": self.analyze_database(),
            "key_components": self.extract_critical_components(),
            "recent_changes": self.get_recent_changes(),
        }
        return context

    def detect_project_phase(self):
        """Detect current project phase from documentation"""
        phase_files = [
            "PHASE-9-EXECUTION-PLAN.md",
            "PHASE-9-100-PERCENT-QUALITY-PLAN.md",
            "PHASE-9-FINAL-STATUS.md",
        ]

        phase_info = {}
        for phase_file in phase_files:
            full_path = self.project_root / phase_file
            if full_path.exists():
                try:
                    content = full_path.read_text()[:1000]
                    phase_info[phase_file] = {"exists": True, "summary": content[:500]}
                except Exception as e:
                    phase_info[phase_file] = {"exists": True, "error": "read_failed"}
            else:
                phase_info[phase_file] = {"exists": False}

        return phase_info

    def analyze_midi_architecture(self):
        """Analyze the specific MIDI software architecture"""
        architecture = {
            "backends": self.analyze_rust_backends(),
            "frontends": self.analyze_tauri_frontends(),
            "database": self.analyze_database_layer(),
            "shared_components": self.analyze_shared_components(),
            "build_system": self.analyze_build_system(),
        }
        return architecture

    def analyze_rust_backends(self):
        """Analyze the three Rust backends"""
        backends = {
            "midi-library-shared": self.analyze_backend("midi-library-shared"),
            "midi-pipeline": self.analyze_backend("midi-pipeline"),
            "midi-daw": self.analyze_backend("midi-daw"),
        }
        return backends

    def analyze_backend(self, backend_name):
        """Analyze a specific Rust backend"""
        backend_path = self.project_root / backend_name
        if not backend_path.exists():
            return {"exists": False}

        try:
            cargo_file = backend_path / "Cargo.toml"
            src_dir = backend_path / "src"

            rust_files = list(backend_path.glob("**/*.rs"))
            test_files = list(backend_path.glob("**/*test*.rs"))

            return {
                "exists": True,
                "rust_files": len(rust_files),
                "test_files": len(test_files),
                "has_cargo": cargo_file.exists(),
                "has_src": src_dir.exists(),
                "main_files": [f.name for f in rust_files[:5]],
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_tauri_frontends(self):
        """Analyze Tauri frontend applications"""
        frontends = {
            "daw": self.analyze_frontend("daw"),
            "pipeline": self.analyze_frontend("pipeline"),
        }
        return frontends

    def analyze_frontend(self, frontend_name):
        """Analyze a specific Tauri frontend"""
        frontend_path = self.project_root / frontend_name
        if not frontend_path.exists():
            return {"exists": False}

        try:
            tauri_dir = frontend_path / "src-tauri"
            src_dir = frontend_path / "src"

            svelte_files = list(frontend_path.glob("**/*.svelte"))
            typescript_files = list(frontend_path.glob("**/*.ts"))
            rust_files = list(tauri_dir.glob("**/*.rs")) if tauri_dir.exists() else []

            return {
                "exists": True,
                "svelte_files": len(svelte_files),
                "typescript_files": len(typescript_files),
                "rust_files": len(rust_files),
                "has_tauri": tauri_dir.exists(),
                "has_package_json": (frontend_path / "package.json").exists(),
                "components": [f.name for f in (src_dir / "Components").iterdir()]
                if (src_dir / "Components").exists()
                else [],
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_database_layer(self):
        """Analyze database and migrations"""
        db_path = self.project_root / "database"

        try:
            migrations = (
                list((db_path / "migrations").glob("*.sql"))
                if (db_path / "migrations").exists()
                else []
            )
            seeds = (
                list((db_path / "seeds").glob("*.sql"))
                if (db_path / "seeds").exists()
                else []
            )

            return {
                "exists": db_path.exists(),
                "migrations": len(migrations),
                "seeds": len(seeds),
                "migration_files": [f.name for f in migrations],
                "has_docker_compose": (db_path / "docker-compose.yml").exists(),
                "has_scripts": len(list(db_path.glob("scripts/*.sh"))) > 0,
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_shared_components(self):
        """Analyze shared libraries and components"""
        shared_path = self.project_root / "shared"

        try:
            rust_shared = shared_path / "rust"
            types_shared = shared_path / "types"
            ui_shared = shared_path / "ui"

            return {
                "exists": shared_path.exists(),
                "rust_shared": rust_shared.exists(),
                "types_shared": types_shared.exists(),
                "ui_shared": ui_shared.exists(),
                "rust_files": len(list(rust_shared.glob("**/*.rs")))
                if rust_shared.exists()
                else 0,
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_build_system(self):
        """Analyze build and deployment configuration"""
        build_info = {
            "makefile": (self.project_root / "Makefile").exists(),
            "cargo_workspace": (self.project_root / "Cargo.toml").exists(),
            "scripts": len(list((self.project_root / "scripts").glob("*.sh"))),
            "launch_scripts": len(
                list((self.project_root / "scripts" / "launch").glob("*.sh"))
            ),
            "docker_config": (self.project_root / "infrastructure" / "docker").exists(),
        }
        return build_info

    def collect_midi_build_errors(self):
        """Collect build errors across all components"""
        print("🔧 Running MIDI-specific builds...")

        errors = {
            "rust_workspace": self.build_rust_workspace(),
            "frontend_daw": self.build_frontend("daw"),
            "frontend_pipeline": self.build_frontend("pipeline"),
            "type_check": self.check_typescript(),
            "test_compilation": self.check_test_compilation(),
        }

        return errors

    def build_rust_workspace(self):
        """Build the entire Rust workspace"""
        try:
            result = subprocess.run(
                ["cargo", "build", "--workspace", "--message-format=json"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=180,
            )

            errors = []
            if result.stderr:
                for line in result.stderr.splitlines():
                    if "error[" in line or "ERROR" in line.upper():
                        errors.append(line.strip())

            return {
                "success": result.returncode == 0,
                "error_count": len(errors),
                "sample_errors": errors[:10],
                "raw_output": result.stderr[:4000] if result.stderr else "No errors",
            }
        except Exception as e:
            return {"error": f"Rust workspace build failed: {str(e)}"}

    def build_frontend(self, frontend_name):
        """Build a specific frontend"""
        frontend_path = self.project_root / frontend_name

        try:
            # Try TypeScript compilation first
            ts_result = subprocess.run(
                ["npx", "tsc", "--noEmit"],
                cwd=frontend_path,
                capture_output=True,
                text=True,
                timeout=60,
            )

            # Try Svelte check
            svelte_result = subprocess.run(
                ["npx", "svelte-check"],
                cwd=frontend_path,
                capture_output=True,
                text=True,
                timeout=60,
            )

            return {
                "typescript_ok": ts_result.returncode == 0,
                "svelte_ok": svelte_result.returncode == 0,
                "ts_errors": ts_result.stderr[:2000] if ts_result.stderr else None,
                "svelte_errors": svelte_result.stdout[:2000]
                if svelte_result.stdout
                else None,
            }
        except Exception as e:
            return {"error": f"Frontend {frontend_name} check failed: {str(e)}"}

    def check_typescript(self):
        """Check TypeScript across project"""
        try:
            result = subprocess.run(
                [
                    "find",
                    ".",
                    "-name",
                    "*.ts",
                    "-not",
                    "-path",
                    "*/node_modules/*",
                    "-exec",
                    "npx",
                    "tsc",
                    "--noEmit",
                    "{}",
                    ";",
                ],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30,
            )
            return {
                "output": result.stderr[:1000]
                if result.stderr
                else "No TypeScript errors"
            }
        except Exception as e:
            return {"error": "TypeScript check skipped"}

    def check_test_compilation(self):
        """Check test compilation status"""
        try:
            # Check disabled tests
            disabled_tests = list((self.project_root / "_disabled_tests").glob("*.rs"))

            # Try compiling tests
            result = subprocess.run(
                ["cargo", "test", "--no-run", "--workspace"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=120,
            )

            return {
                "disabled_tests_count": len(disabled_tests),
                "test_compilation_ok": result.returncode == 0,
                "compilation_errors": result.stderr[:3000] if result.stderr else None,
            }
        except Exception as e:
            return {"error": f"Test compilation check failed: {str(e)}"}

    def analyze_test_health(self):
        """Analyze test suite health"""
        try:
            # Count test files
            rust_tests = list(self.project_root.glob("**/*test*.rs"))
            frontend_tests = list(self.project_root.glob("**/*test*.ts"))

            # Check test directories
            test_dirs = {
                "e2e": (self.project_root / "tests" / "e2e").exists(),
                "integration": (self.project_root / "tests" / "integration").exists(),
                "fixtures": (self.project_root / "tests" / "fixtures").exists(),
            }

            return {
                "rust_tests": len(rust_tests),
                "frontend_tests": len(frontend_tests),
                "test_directories": test_dirs,
                "coverage_report": (
                    self.project_root / "coverage_error" / "tarpaulin-report.html"
                ).exists(),
            }
        except Exception as e:
            return {"error": str(e)}

    def analyze_database(self):
        """Analyze database state"""
        db_path = self.project_root / "database"

        try:
            # Check if database is running
            docker_result = subprocess.run(
                ["docker", "compose", "ps", "--services", "--filter", "status=running"],
                cwd=db_path,
                capture_output=True,
                text=True,
            )

            return {
                "docker_services_running": docker_result.stdout.strip().split("\n")
                if docker_result.returncode == 0
                else [],
                "migrations_applied": len(list((db_path / "migrations").glob("*.sql"))),
                "has_sample_data": (db_path / "seeds").exists(),
            }
        except Exception as e:
            return {"error": f"Database analysis failed: {str(e)}"}

    def extract_critical_components(self):
        """Extract critical project components"""
        critical_files = {}

        # Rust backend critical files
        rust_critical = [
            "Cargo.toml",
            "midi-library-shared/Cargo.toml",
            "midi-pipeline/Cargo.toml",
            "midi-daw/Cargo.toml",
            "shared/rust/Cargo.toml",
        ]

        # Frontend critical files
        frontend_critical = [
            "daw/package.json",
            "pipeline/package.json",
            "daw/src-tauri/tauri.conf.json",
            "pipeline/src-tauri/tauri.conf.json",
        ]

        # Config critical files
        config_critical = ["database/docker-compose.yml", "rustfmt.toml", "Makefile"]

        all_critical = rust_critical + frontend_critical + config_critical

        for file_path in all_critical:
            full_path = self.project_root / file_path
            if full_path.exists():
                try:
                    content = full_path.read_text()[:2000]
                    critical_files[file_path] = content
                except Exception as e:
                    critical_files[file_path] = f"Error reading: {str(e)}"

        return critical_files

    def get_recent_changes(self):
        """Get recent git changes for context"""
        try:
            # Get recent commits
            commits = subprocess.run(
                ["git", "log", "--oneline", "-10", "--graph"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
            )

            # Get current branch
            branch = subprocess.run(
                ["git", "branch", "--show-current"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
            )

            return {
                "recent_commits": commits.stdout
                if commits.returncode == 0
                else "No git history",
                "current_branch": branch.stdout.strip()
                if branch.returncode == 0
                else "Unknown",
            }
        except Exception as e:
            return {"error": f"Git check failed: {str(e)}"}

    # 🔄 IMPROVEMENT 3: ITERATIVE MIDI-SPECIFIC PIPELINE
    def run_midi_analysis_pipeline(self, context):
        """MIDI-specific analysis pipeline"""
        print("\n🔄 Starting MIDI-specific analysis pipeline...")

        stages = [
            ("🎵 MIDI Architecture Review", self.midi_architecture_review),
            ("🔧 Build System Analysis", self.build_system_analysis),
            ("🧪 Test Suite Health Check", self.test_health_analysis),
            ("🚀 Deployment Readiness", self.deployment_readiness),
        ]

        results = {}
        for stage_name, stage_func in stages:
            print(f"\n{stage_name}")
            print("=" * len(stage_name))
            results[stage_name] = stage_func(context)
            self.save_analysis_stage(stage_name, results[stage_name])
            time.sleep(1)

        return results

    def midi_architecture_review(self, context):
        """Stage 1: MIDI-specific architecture review"""
        prompt = f"""
MIDI SOFTWARE CENTER - ARCHITECTURE REVIEW

PROJECT STRUCTURE:
- Backends: {context["architecture"]["backends"]}
- Frontends: {context["architecture"]["frontends"]}
- Database: {context["architecture"]["database"]}
- Shared: {context["architecture"]["shared_components"]}

CURRENT PHASE: {context["project_phase"]}

ANALYZE:
1. Architecture cohesion between 3 backends and 2 frontends
2. MIDI data flow through the system
3. Shared code usage and dependencies
4. Database schema alignment with MIDI domain
5. Tauri desktop app architecture

Focus on MIDI-specific architectural patterns and potential integration issues.
"""
        return self.call_grok(prompt)

    def build_system_analysis(self, context):
        """Stage 2: Build system and compilation analysis"""
        prompt = f"""
MIDI SOFTWARE CENTER - BUILD SYSTEM ANALYSIS

BUILD STATUS:
- Rust Workspace: {context["build_errors"]["rust_workspace"].get("success", False)}
- DAW Frontend: {context["build_errors"]["frontend_daw"]}
- Pipeline Frontend: {context["build_errors"]["frontend_pipeline"]}
- Test Compilation: {context["build_errors"]["test_compilation"]}

SPECIFIC ERRORS:
{context["build_errors"]["rust_workspace"].get("sample_errors", [])}

ANALYZE:
1. Root causes of compilation failures
2. Dependency conflicts between backends
3. Frontend-backend integration issues
4. Test compilation blockers
5. Build system optimization opportunities

Provide specific fix recommendations for MIDI project structure.
"""
        return self.call_grok(prompt)

    def test_health_analysis(self, context):
        """Stage 3: Test suite health analysis"""
        prompt = f"""
MIDI SOFTWARE CENTER - TEST SUITE HEALTH

TEST STATUS:
- Rust Tests: {context["test_status"].get("rust_tests", 0)}
- Frontend Tests: {context["test_status"].get("frontend_tests", 0)}
- Disabled Tests: {context["build_errors"]["test_compilation"].get("disabled_tests_count", 0)}
- Test Directories: {context["test_status"].get("test_directories", {})}

TEST COMPILATION:
{context["build_errors"]["test_compilation"]}

ANALYZE:
1. Test coverage gaps for MIDI functionality
2. Integration test strategy
3. Performance testing needs
4. MIDI file parsing tests
5. Database transaction tests

Recommend test strategy for music production software.
"""
        return self.call_grok(prompt)

    def deployment_readiness(self, context):
        """Stage 4: Deployment readiness assessment"""
        prompt = f"""
MIDI SOFTWARE CENTER - DEPLOYMENT READINESS

CURRENT STATE:
- Project Phase: {context["project_phase"]}
- Build Status: {context["build_errors"]["rust_workspace"].get("success", False)}
- Database: {context["database_state"]}
- Recent Changes: {context["recent_changes"]}

ASSESS READINESS FOR:
1. Development deployment
2. Testing environment
3. Production release

PROVIDE:
- Critical blockers to address
- Recommended deployment strategy
- Performance considerations for MIDI apps
- Desktop app distribution plan

Focus on music production software deployment specifics.
"""
        return self.call_grok(prompt)

    # 🔍 IMPROVEMENT 4: ERROR PATTERN DETECTION
    def analyze_error_patterns(self, context):
        """Advanced error pattern detection for MIDI project"""
        prompt = f"""
MIDI SOFTWARE CENTER - ERROR PATTERN DETECTION

COMPILATION ERRORS:
{context["build_errors"]["rust_workspace"].get("raw_output", "No errors")}

PROJECT CONTEXT:
- 3 Rust backends, 2 Tauri frontends
- MIDI processing, database, real-time audio
- Complex type system with shared libraries

CATEGORIZE ERRORS BY:
1. Type system issues (traits, generics, lifetimes)
2. Module and visibility problems
3. Database and async conflicts
4. MIDI-specific domain errors
5. Frontend-backend type mismatches
6. Build dependency conflicts

For each category:
- Identify root cause patterns
- Provide specific code fixes
- Suggest architectural improvements

Focus on patterns that affect music software reliability.
"""
        return self.call_grok(prompt)

    # 📋 IMPROVEMENT 9: PRIORITIZED MIDI ACTION PLAN
    def generate_midi_action_plan(self, context, pipeline_results, error_patterns):
        """Generate MIDI-specific prioritized action plan"""
        prompt = f"""
MIDI SOFTWARE CENTER - PRIORITIZED ACTION PLAN

CURRENT STATUS:
- Phase: {context["project_phase"]}
- Architecture: {context["architecture"]}
- Build: {"✅" if context["build_errors"]["rust_workspace"].get("success") else "❌"}
- Tests: {context["test_status"]}

ANALYSIS INSIGHTS:
{pipeline_results}

ERROR PATTERNS:
{error_patterns}

CREATE TIME-BOXED ACTION PLAN:

🎯 IMMEDIATE (Next 8 hours):
[Critical fixes for MIDI functionality]

🚀 SHORT TERM (This week):
[Architecture improvements, test enablement]

🏗️ MEDIUM TERM (Next 2 weeks):
[Performance optimization, feature completion]

🎊 LONG TERM (Release ready):
[Polish, documentation, deployment]

INCLUDE:
- Specific file paths to fix
- Exact commands to run
- Success metrics for music software
- Risk mitigation for real-time audio

Prioritize MIDI core functionality and audio reliability.
"""
        return self.call_grok(prompt)

    def save_analysis_stage(self, stage_name, result):
        """Save analysis stage results"""
        self.analysis_history.append(
            {
                "stage": stage_name,
                "timestamp": datetime.now().isoformat(),
                "result": result[:1000] + "..." if len(result) > 1000 else result,
            }
        )

    def call_grok(self, prompt: str, model: str = "grok-4-fast-reasoning") -> str:
        """Call Grok API"""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 4000,
        }

        try:
            response = self.client.post(
                "https://api.x.ai/v1/chat/completions", json=payload, headers=headers
            )
            response.raise_for_status()
            result = response.json()
            return result["choices"][0]["message"]["content"]
        except Exception as e:
            return f"❌ API Error: {str(e)}"

    def run_enhanced_midi_analysis(self):
        """Main enhanced analysis for MIDI project"""
        print("🎵 ENHANCED MIDI SOFTWARE CENTER ANALYSIS")
        print("=" * 50)

        # 1. Gather project context
        context = self.gather_project_context()

        # 2. Run MIDI-specific pipeline
        pipeline_results = self.run_midi_analysis_pipeline(context)

        # 3. Advanced error pattern detection
        print("\n🔍 Analyzing error patterns...")
        error_patterns = self.analyze_error_patterns(context)

        # 4. Generate prioritized action plan
        print("\n📋 Generating MIDI-specific action plan...")
        action_plan = self.generate_midi_action_plan(
            context, pipeline_results, error_patterns
        )

        # 5. Display comprehensive results
        self.display_midi_results(pipeline_results, error_patterns, action_plan)

        # 6. Save analysis
        self.save_analysis_history()

        print(
            f"\n✅ MIDI analysis complete! {len(self.analysis_history)} stages executed."
        )

    def display_midi_results(self, pipeline_results, error_patterns, action_plan):
        """Display comprehensive MIDI analysis results"""
        print("\n" + "=" * 60)
        print("🎵 MIDI SOFTWARE CENTER - COMPREHENSIVE ANALYSIS")
        print("=" * 60)

        print("\n📊 PIPELINE RESULTS:")
        for stage_name, result in pipeline_results.items():
            print(f"\n{stage_name}")
            print("-" * len(stage_name))
            print(result)

        print("\n🔍 ERROR PATTERNS:")
        print("=" * 20)
        print(error_patterns)

        print("\n📋 ACTION PLAN:")
        print("=" * 15)
        print(action_plan)

    def save_analysis_history(self):
        """Save analysis history"""
        history_file = self.project_root / "grok_midi_analysis.json"
        try:
            history_data = {
                "timestamp": datetime.now().isoformat(),
                "analysis_stages": self.analysis_history,
            }
            history_file.write_text(json.dumps(history_data, indent=2))
            print(f"💾 Analysis saved to: {history_file}")
        except Exception as e:
            print(f"⚠️ Could not save history: {e}")

    def close(self):
        """Cleanup"""
        self.client.close()


def main():
    """Main entry point"""
    reviewer = MIDIProjectReviewer()

    try:
        if len(sys.argv) > 1:
            if sys.argv[1] in ["midi", "full", "enhanced"]:
                reviewer.run_enhanced_midi_analysis()
            elif sys.argv[1] == "quick":
                context = reviewer.gather_project_context()
                result = reviewer.midi_architecture_review(context)
                print(result)
            elif sys.argv[1] == "errors":
                context = reviewer.gather_project_context()
                result = reviewer.analyze_error_patterns(context)
                print(result)
            else:
                print("Usage: python3 midi_grok_reviewer.py [command]")
                print("\nCommands:")
                print("  midi     - Full MIDI-specific analysis (recommended)")
                print("  quick    - Quick architecture review")
                print("  errors   - Error pattern analysis only")
        else:
            reviewer.run_enhanced_midi_analysis()
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()

```

### `grok/supercharged_grok_reviewer.py` {#grok-supercharged-grok-reviewer-py}

- **Lines**: 878 (code: 557, comments: 173, blank: 148)
- **Pylint Score**: 6.29/10 ⚠️
- **Docstring Coverage**: 88.1%
- **Functions**: main, __init__, apply_recommended_fixes, should_apply_fix, fix_tauri_ts_conflicts, rewrite_tauri_types, fix_rust_dead_code, create_missing_backends, fix_build_config, enable_disabled_tests
  *... and 27 more*
- **Classes**: MIDIAutoFixer, PredictiveAnalyzer, AnalysisLearner, DevelopmentWorkflowIntegrator, SuperchargedMIDIReviewer

#### Source Code

```python
#!/usr/bin/env python3
"""
SUPERCHARGED Grok 4 Project Reviewer for MIDI Software Center
Enhanced with: Auto-Fix Mode, Intelligent Code Modification, Predictive Error Detection,
Learning from Past Analyses, Intelligent Prompt Generation, Auto-Generated Fix Scripts,
and CI/CD Integration
"""

import os
import sys
import json
import subprocess
from pathlib import Path
from datetime import datetime
from collections import defaultdict
import httpx


class MIDIAutoFixer:
    """Automatically applies fixes based on analysis results"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def apply_recommended_fixes(self, analysis_results):
        """Auto-apply the top recommended fixes from analysis"""
        print("🔧 AUTO-FIX MODE ACTIVATED")
        
        fixes = {
            "tauri_typescript": self.fix_tauri_ts_conflicts,
            "rust_dead_code": self.fix_rust_dead_code,
            "missing_backends": self.create_missing_backends,
            "build_config": self.fix_build_config,
            "test_enablement": self.enable_disabled_tests
        }
        
        applied_fixes = []
        for fix_name, fix_func in fixes.items():
            if self.should_apply_fix(analysis_results, fix_name):
                print(f"🛠️  Applying {fix_name}...")
                try:
                    if fix_func():
                        applied_fixes.append(fix_name)
                except Exception as e:
                    print(f"⚠️  Failed to apply {fix_name}: {e}")
        
        return applied_fixes
    
    def should_apply_fix(self, analysis_results, fix_name):
        """Determine if a fix should be applied based on analysis"""
        analysis_text = str(analysis_results).lower()
        
        fix_conditions = {
            "tauri_typescript": any(term in analysis_text for term in ["tauri", "typescript", "window", "__tauri__"]),
            "rust_dead_code": any(term in analysis_text for term in ["dead code", "unused", "warning"]),
            "missing_backends": any(term in analysis_text for term in ["missing backends", "backend stubs", "non-existent"]),
            "build_config": any(term in analysis_text for term in ["cargo.toml", "manifest", "build config"]),
            "test_enablement": any(term in analysis_text for term in ["disabled tests", "test compilation"])
        }
        
        return fix_conditions.get(fix_name, False)
    
    def fix_tauri_ts_conflicts(self):
        """Auto-fix the Tauri TypeScript conflicts"""
        ts_files = list(self.project_root.glob("**/*.ts")) + list(self.project_root.glob("**/*.tsx"))
        fixed_count = 0
        
        for file_path in ts_files:
            try:
                content = file_path.read_text()
                if "interface Window" in content and "__TAURI__" in content:
                    # Fix the Tauri type conflicts
                    fixed_content = self.rewrite_tauri_types(content)
                    if fixed_content != content:
                        file_path.write_text(fixed_content)
                        print(f"✅ Fixed Tauri types in {file_path.relative_to(self.project_root)}")
                        fixed_count += 1
            except Exception as e:
                print(f"⚠️  Could not process {file_path}: {e}")
        
        return fixed_count > 0
    
    def rewrite_tauri_types(self, content):
        """Rewrite Tauri TypeScript types to avoid conflicts"""
        # Remove conflicting Window interface declarations
        lines = content.split('\n')
        in_window_interface = False
        new_lines = []
        
        for line in lines:
            if "interface Window" in line and "__TAURI__" in line:
                in_window_interface = True
                # Replace with proper declaration
                new_lines.append("// Fixed Tauri types - remove conflicting declarations")
                continue
            elif in_window_interface and line.strip() == "}":
                in_window_interface = False
                continue
            elif not in_window_interface:
                new_lines.append(line)
        
        return '\n'.join(new_lines)
    
    def fix_rust_dead_code(self):
        """Fix Rust dead code warnings"""
        rust_files = list(self.project_root.glob("**/*.rs"))
        fixed_count = 0
        
        for file_path in rust_files:
            try:
                content = file_path.read_text()
                
                # Pattern 1: Add #[allow(dead_code)] to structs with unused fields
                if "struct " in content and "unused" in content.lower():
                    lines = content.split('\n')
                    new_lines = []
                    
                    for i, line in enumerate(lines):
                        if "struct " in line and i > 0 and "#[allow(dead_code)]" not in lines[i-1]:
                            new_lines.append("#[allow(dead_code)]")
                        new_lines.append(line)
                    
                    new_content = '\n'.join(new_lines)
                    if new_content != content:
                        file_path.write_text(new_content)
                        print(f"✅ Fixed dead code in {file_path.relative_to(self.project_root)}")
                        fixed_count += 1
                        
            except Exception as e:
                print(f"⚠️  Could not process {file_path}: {e}")
        
        return fixed_count > 0
    
    def create_missing_backends(self):
        """Create missing backend crate structure"""
        backends = ["midi-library-shared", "midi-pipeline", "midi-daw"]
        created_count = 0
        
        for backend in backends:
            backend_path = self.project_root / backend
            if not backend_path.exists():
                backend_path.mkdir(exist_ok=True)
                
                # Create basic Cargo.toml
                cargo_content = f"""[package]
name = "{backend}"
version = "0.1.0"
edition = "2021"

[dependencies]
"""
                if backend == "midi-library-shared":
                    cargo_content += "serde = { version = \"1.0\", features = [\"derive\"] }\n"
                
                (backend_path / "Cargo.toml").write_text(cargo_content)
                
                # Create src directory
                src_path = backend_path / "src"
                src_path.mkdir(exist_ok=True)
                
                # Create lib.rs or main.rs
                if backend == "midi-library-shared":
                    (src_path / "lib.rs").write_text("// MIDI shared library\n")
                else:
                    (src_path / "main.rs").write_text("fn main() {\n    println!(\"Hello from {backend}!\");\n}\n")
                
                print(f"✅ Created missing backend: {backend}")
                created_count += 1
        
        return created_count > 0
    
    def fix_build_config(self):
        """Fix build configuration issues"""
        # Fix Cargo.toml issues
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        fixed_count = 0
        
        for cargo_file in cargo_files:
            try:
                content = cargo_file.read_text()
                
                # Remove invalid bin entries
                if "[[bin]]" in content and "walkdir" in content:
                    lines = content.split('\n')
                    new_lines = []
                    skip_next = False
                    
                    for line in lines:
                        if "name = \"walkdir\"" in line:
                            skip_next = True
                            continue
                        elif skip_next and line.strip() == "":
                            skip_next = False
                            continue
                        elif not skip_next:
                            new_lines.append(line)
                    
                    new_content = '\n'.join(new_lines)
                    if new_content != content:
                        cargo_file.write_text(new_content)
                        print(f"✅ Fixed build config in {cargo_file.relative_to(self.project_root)}")
                        fixed_count += 1
                        
            except Exception as e:
                print(f"⚠️  Could not process {cargo_file}: {e}")
        
        return fixed_count > 0
    
    def enable_disabled_tests(self):
        """Enable some disabled tests"""
        disabled_tests_dir = self.project_root / "_disabled_tests"
        if disabled_tests_dir.exists():
            test_files = list(disabled_tests_dir.glob("*.rs"))
            enabled_count = 0
            
            for test_file in test_files[:3]:  # Enable first 3 as a start
                target_path = self.project_root / "tests" / test_file.name
                try:
                    test_file.rename(target_path)
                    print(f"✅ Enabled test: {test_file.name}")
                    enabled_count += 1
                except Exception as e:
                    print(f"⚠️  Could not enable {test_file.name}: {e}")
            
            return enabled_count > 0
        return False


class PredictiveAnalyzer:
    """Predicts future errors based on patterns"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def predict_compilation_issues(self, context):
        """Predict what will break before it happens"""
        predictions = []
        
        # Analyze dependency chains
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        for cargo_file in cargo_files:
            if self.has_misconfigured_dependencies(cargo_file):
                predictions.append(f"🔧 Misconfigured dependencies in {cargo_file.relative_to(self.project_root)}")
        
        # Predict TypeScript issues
        ts_config = self.project_root / "tsconfig.json"
        if ts_config.exists():
            if self.will_break_on_strict_mode(ts_config):
                predictions.append("⚡ TypeScript strict mode may cause breaks")
        
        # Check for common MIDI project issues
        if self.has_missing_midi_dependencies():
            predictions.append("🎵 Missing MIDI-specific dependencies")
        
        # Check build configuration
        if self.has_build_config_issues():
            predictions.append("🔨 Build configuration issues detected")
        
        return predictions
    
    def has_misconfigured_dependencies(self, cargo_file):
        """Detect misconfigured dependencies"""
        try:
            content = cargo_file.read_text()
            # Check for common issues
            issues = [
                "walkdir" in content and "[[bin]]" in content,  # walkdir in bin section
                "dependencies]" not in content and "[[bin]]" in content,  # missing dependencies section
            ]
            return any(issues)
        except Exception as e:
            return False
    
    def will_break_on_strict_mode(self, ts_config):
        """Predict if TypeScript strict mode will cause issues"""
        try:
            content = ts_config.read_text()
            return "\"strict\": true" in content
        except Exception as e:
            return False
    
    def has_missing_midi_dependencies(self):
        """Check for missing MIDI-specific dependencies"""
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        for cargo_file in cargo_files:
            content = cargo_file.read_text()
            # Common MIDI crates that might be missing
            midi_crates = ["midly", "midir", "cpal"]
            if any(crate in content for crate in midi_crates):
                return False
        return True
    
    def has_build_config_issues(self):
        """Check for build configuration issues"""
        # Check if workspace is properly configured
        root_cargo = self.project_root / "Cargo.toml"
        if root_cargo.exists():
            content = root_cargo.read_text()
            return "[workspace]" not in content
        return True


class AnalysisLearner:
    """Learns from previous analyses to improve future ones"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.history_file = self.project_root / "grok_analysis_history.json"
        self.patterns = self.load_historical_patterns()
    
    def load_historical_patterns(self):
        """Learn from past analysis patterns"""
        patterns = {
            "common_errors": defaultdict(int),
            "fix_effectiveness": {},
            "recurring_issues": []
        }
        
        if self.history_file.exists():
            try:
                history = json.loads(self.history_file.read_text())
                patterns = self.extract_patterns(history)
            except Exception as e:
                print(f"⚠️  Could not load analysis history: {e}")
        
        return patterns
    
    def extract_patterns(self, history):
        """Extract recurring patterns from history"""
        patterns = {
            "common_errors": defaultdict(int),
            "fix_effectiveness": {},
            "recurring_issues": []
        }
        
        # Extract from analysis stages
        for stage in history.get("analysis_stages", []):
            result = stage.get("result", "")
            error_types = self.categorize_errors(result)
            for error_type in error_types:
                patterns["common_errors"][error_type] += 1
        
        # Identify recurring issues (appear in multiple analyses)
        common_issues = [issue for issue, count in patterns["common_errors"].items() if count > 1]
        patterns["recurring_issues"] = common_issues
        
        return patterns
    
    def categorize_errors(self, analysis_text):
        """Categorize errors from analysis text"""
        categories = []
        text_lower = analysis_text.lower()
        
        if any(term in text_lower for term in ["typescript", "tauri", "window"]):
            categories.append("typescript_tauri_conflict")
        if any(term in text_lower for term in ["dead code", "unused"]):
            categories.append("rust_dead_code")
        if any(term in text_lower for term in ["missing", "backend", "stub"]):
            categories.append("missing_components")
        if any(term in text_lower for term in ["build", "compilation", "cargo"]):
            categories.append("build_issues")
        if any(term in text_lower for term in ["test", "coverage"]):
            categories.append("test_issues")
        
        return categories
    
    def get_most_likely_issues(self):
        """Predict issues based on historical patterns"""
        common_issues = sorted(
            self.patterns["common_errors"].items(), 
            key=lambda x: x[1], 
            reverse=True
        )[:5]
        return common_issues
    
    def save_analysis(self, analysis_data):
        """Save current analysis to history"""
        try:
            history = {}
            if self.history_file.exists():
                history = json.loads(self.history_file.read_text())
            
            # Add current analysis
            if "analysis_stages" not in history:
                history["analysis_stages"] = []
            
            history["analysis_stages"].extend(analysis_data.get("stages", []))
            history["last_updated"] = datetime.now().isoformat()
            
            # Keep only last 10 analyses to prevent file from growing too large
            if len(history["analysis_stages"]) > 10:
                history["analysis_stages"] = history["analysis_stages"][-10:]
            
            self.history_file.write_text(json.dumps(history, indent=2))
        except Exception as e:
            print(f"⚠️  Could not save analysis history: {e}")


class DevelopmentWorkflowIntegrator:
    """Integrates with existing development workflows"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.ci_files = [
            ".github/workflows/ci.yml",
            ".gitlab-ci.yml", 
            ".circleci/config.yml",
            "azure-pipelines.yml"
        ]
    
    def integrate_with_ci(self):
        """Add Grok analysis to CI pipeline"""
        print("🔄 Integrating with CI/CD workflow...")
        
        for ci_file in self.ci_files:
            ci_path = self.project_root / ci_file
            if ci_path.exists():
                if self.add_grok_step_to_ci(ci_path):
                    print(f"✅ Added Grok analysis to {ci_path}")
                    return True
        
        # If no CI file exists, create a basic GitHub Actions workflow
        self.create_github_workflow()
        return True
    
    def add_grok_step_to_ci(self, ci_path):
        """Add Grok analysis step to CI configuration"""
        try:
            content = ci_path.read_text()
            
            # Check if Grok step already exists
            if "Grok Analysis" in content:
                return False
            
            grok_step = """
      - name: AI Code Analysis
        run: |
          echo "🔍 Running AI-powered code analysis..."
          python3 midi_grok_reviewer.py ci-check
          if [ -f "grok_analysis_report.json" ]; then
            echo "Analysis report generated"
          fi
"""
            
            # Insert after build step (simplistic approach)
            if "name: Build" in content:
                new_content = content.replace(
                    "      - name: Build", 
                    "      - name: Build" + grok_step
                )
                ci_path.write_text(new_content)
                return True
                
        except Exception as e:
            print(f"⚠️  Could not modify {ci_path}: {e}")
        
        return False
    
    def create_github_workflow(self):
        """Create a GitHub Actions workflow if none exists"""
        workflows_dir = self.project_root / ".github" / "workflows"
        workflows_dir.mkdir(parents=True, exist_ok=True)
        
        workflow_content = """name: CI with AI Analysis

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Build
      run: cargo build --verbose
    
    - name: Run Tests
      run: cargo test --verbose
    
    - name: AI Code Analysis
      run: |
        pip3 install httpx
        python3 midi_grok_reviewer.py ci-check
        if [ -f "grok_analysis_report.json" ]; then
          echo "📊 AI analysis completed"
        fi
"""
        
        workflow_file = workflows_dir / "ci.yml"
        workflow_file.write_text(workflow_content)
        print(f"✅ Created GitHub Actions workflow: {workflow_file}")


class SuperchargedMIDIReviewer:
    """Main class with all enhanced capabilities"""
    
    def __init__(self, api_key=None):
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path("/home/dojevou/projects/midi-software-center")
        
        # Initialize all enhanced components
        self.auto_fixer = MIDIAutoFixer(self.project_root)
        self.predictive_analyzer = PredictiveAnalyzer(self.project_root)
        self.analysis_learner = AnalysisLearner(self.project_root)
        self.workflow_integrator = DevelopmentWorkflowIntegrator(self.project_root)
        
        self.client = httpx.Client(timeout=120.0, http2=True)
        self.analysis_history = []
        
        if not self.api_key:
            print("❌ GROK_API_KEY not set")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    def generate_intelligent_prompt(self, context, analysis_type):
        """Generate context-aware prompts using historical patterns"""
        historical_patterns = self.analysis_learner.get_most_likely_issues()
        predictions = self.predictive_analyzer.predict_compilation_issues(context)
        
        base_prompts = {
            "architecture": """
MIDI SOFTWARE ARCHITECTURE REVIEW - ENHANCED ANALYSIS

HISTORICAL PATTERNS:
{historical_patterns}

FUTURE PREDICTIONS:
{predictions}

PROJECT CONTEXT:
{context}

FOCUS ON:
1. Architecture patterns that have caused issues before
2. Preventing predicted future problems
3. MIDI-specific reliability concerns
4. Long-term maintainability given historical trends

Provide actionable insights that address both current state and future risks.
""",
            "errors": """
COMPREHENSIVE ERROR ANALYSIS - LEARNING FROM HISTORY

RECURRING ISSUES:
{historical_patterns}

PREDICTED ISSUES:
{predictions}

CURRENT ERROR CONTEXT:
{context}

ANALYSIS APPROACH:
- Identify if current errors match historical patterns
- Suggest fixes that address root causes, not just symptoms
- Consider MIDI domain implications of each error type
- Prioritize based on historical fix effectiveness

Provide both immediate fixes and long-term prevention strategies.
"""
        }
        
        template = base_prompts.get(analysis_type, base_prompts["architecture"])
        return template.format(
            historical_patterns=historical_patterns,
            predictions=predictions,
            context=context
        )

    def generate_fix_scripts(self, analysis_results):
        """Generate executable fix scripts based on analysis"""
        print("📜 Generating automated fix scripts...")
        
        fixes = self.extract_actionable_fixes(analysis_results)
        
        script_content = "#!/bin/bash\n# Auto-generated fix script for MIDI Software Center\n# Generated: {}\n\n".format(datetime.now().isoformat())
        script_content += "set -e\n\n"
        script_content += "echo '🎵 MIDI Software Center - Automated Fixes'\n"
        script_content += "echo '========================================'\n\n"
        
        for i, fix in enumerate(fixes, 1):
            script_content += "echo '\\n🔧 Fix {}: {}'\n".format(i, fix['description'])
            script_content += "{}\n".format(fix['command'])
            
            if fix.get('verify'):
                script_content += "echo '✅ {}'\n".format(fix['verify'])
            
            script_content += "sleep 1\n\n"
        
        script_content += "echo '\\n🎉 All fixes applied!'\n"
        script_content += "echo 'Run ./midi_grok_reviewer.py midi to verify improvements'\n"
        
        script_path = self.project_root / "auto_fix.sh"
        script_path.write_text(script_content)
        script_path.chmod(0o755)
        
        print(f"✅ Fix script generated: {script_path}")
        return script_path

    def extract_actionable_fixes(self, analysis):
        """Extract commands that can be automated from analysis"""
        actionable_fixes = []
        
        # Parse analysis for specific fix patterns
        lines = str(analysis).split('\n')
        
        for line in lines:
            line = line.strip()
            
            # Rust/Cargo fixes
            if line.startswith('cargo ') and 'fix' in line.lower():
                actionable_fixes.append({
                    'description': f"Run cargo fix: {line}",
                    'command': line,
                    'verify': "Cargo fix completed"
                })
            elif line.startswith('cargo ') and 'clippy' in line:
                actionable_fixes.append({
                    'description': f"Run clippy: {line}",
                    'command': line,
                    'verify': "Clippy analysis completed"
                })
            
            # TypeScript fixes
            elif line.startswith('npm ') and ('fix' in line.lower() or 'lint' in line.lower()):
                actionable_fixes.append({
                    'description': f"Run npm fix: {line}",
                    'command': f"cd {self.project_root / 'pipeline'} && {line}",
                    'verify': "NPM fix completed"
                })
        
        # Add common MIDI project fixes
        common_fixes = [
            {
                'description': "Clean cargo build artifacts",
                'command': "cargo clean",
                'verify': "Build artifacts cleaned"
            },
            {
                'description': "Update cargo dependencies",
                'command': "cargo update",
                'verify': "Dependencies updated"
            },
            {
                'description': "Check TypeScript compilation",
                'command': "cd pipeline && npx tsc --noEmit",
                'verify': "TypeScript check completed"
            }
        ]
        
        actionable_fixes.extend(common_fixes)
        return actionable_fixes

    def run_super_analysis(self):
        """Run all enhanced analysis modes"""
        print("🚀 SUPERCHARGED MIDI ANALYSIS ACTIVATED")
        print("=" * 60)
        
        # 1. Gather project context
        print("\n📁 Gathering enhanced project context...")
        context = self.gather_project_context()
        
        # 2. Predictive analysis
        print("\n🔮 Running predictive analysis...")
        predictions = self.predictive_analyzer.predict_compilation_issues(context)
        if predictions:
            print("📊 PREDICTED ISSUES:")
            for prediction in predictions:
                print(f"  • {prediction}")
        
        # 3. Historical pattern analysis
        print("\n📚 Analyzing historical patterns...")
        historical_issues = self.analysis_learner.get_most_likely_issues()
        if historical_issues:
            print("🕰️  HISTORICAL PATTERNS:")
            for issue, count in historical_issues:
                print(f"  • {issue}: {count} occurrences")
        
        # 4. Run enhanced analysis with intelligent prompts
        print("\n🧠 Running enhanced AI analysis...")
        enhanced_prompt = self.generate_intelligent_prompt(context, "architecture")
        analysis_results = self.call_grok(enhanced_prompt)
        
        # 5. Auto-apply fixes
        print("\n🔧 Applying automated fixes...")
        applied_fixes = self.auto_fixer.apply_recommended_fixes(analysis_results)
        if applied_fixes:
            print(f"✅ APPLIED FIXES: {', '.join(applied_fixes)}")
        
        # 6. Generate fix scripts
        print("\n📜 Generating automated fix scripts...")
        script_path = self.generate_fix_scripts(analysis_results)
        
        # 7. Integrate with CI/CD
        print("\n🔄 Integrating with development workflow...")
        self.workflow_integrator.integrate_with_ci()
        
        # 8. Save analysis for learning
        analysis_data = {
            "timestamp": datetime.now().isoformat(),
            "stages": [
                {"stage": "predictive_analysis", "result": str(predictions)},
                {"stage": "historical_patterns", "result": str(historical_issues)},
                {"stage": "ai_analysis", "result": analysis_results[:1000] + "..." if len(analysis_results) > 1000 else analysis_results},
                {"stage": "applied_fixes", "result": str(applied_fixes)}
            ]
        }
        self.analysis_learner.save_analysis(analysis_data)
        
        # Display results
        self.display_enhanced_results(analysis_results, predictions, historical_issues, applied_fixes, script_path)
        
        print("\n🎉 SUPERCHARGED ANALYSIS COMPLETE!")
        print(f"📁 Fix script: {script_path}")
        print("💾 Analysis saved to history for future learning")

    def gather_project_context(self):
        """Gather basic project context for analysis"""
        return {
            "timestamp": datetime.now().isoformat(),
            "rust_files": len(list(self.project_root.glob("**/*.rs"))),
            "typescript_files": len(list(self.project_root.glob("**/*.ts"))),
            "svelte_files": len(list(self.project_root.glob("**/*.svelte"))),
            "cargo_files": len(list(self.project_root.glob("**/Cargo.toml"))),
            "has_tauri": any(self.project_root.glob("**/tauri.conf.json")),
            "build_status": self.check_build_status()
        }

    def check_build_status(self):
        """Check current build status"""
        try:
            result = subprocess.run(
                ["cargo", "check", "--workspace"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            return {
                "success": result.returncode == 0,
                "has_errors": result.returncode != 0,
                "error_count": len([line for line in result.stderr.split('\n') if 'error[' in line])
            }
        except Exception as e:
            return {"success": False, "has_errors": True, "error_count": "unknown"}

    def call_grok(self, prompt: str, model: str = "grok-4-fast-reasoning") -> str:
        """Call Grok API"""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }
        
        payload = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 4000,
        }
        
        try:
            response = self.client.post(
                "https://api.x.ai/v1/chat/completions",
                json=payload,
                headers=headers
            )
            response.raise_for_status()
            result = response.json()
            return result["choices"][0]["message"]["content"]
        except Exception as e:
            return f"❌ API Error: {str(e)}"

    def display_enhanced_results(self, analysis_results, predictions, historical_issues, applied_fixes, script_path):
        """Display comprehensive results from all enhanced analyses"""
        print("\n" + "="*70)
        print("🚀 ENHANCED MIDI ANALYSIS - COMPREHENSIVE RESULTS")
        print("="*70)
        
        print(f"\n🔮 PREDICTIVE INSIGHTS ({len(predictions)} issues):")
        for prediction in predictions:
            print(f"  • {prediction}")
        
        print(f"\n📚 HISTORICAL PATTERNS ({len(historical_issues)} patterns):")
        for issue, count in historical_issues:
            print(f"  • {issue}: {count} past occurrences")
        
        print(f"\n🔧 AUTOMATED FIXES ({len(applied_fixes)} applied):")
        for fix in applied_fixes:
            print(f"  • {fix}")
        
        print("\n🧠 AI ANALYSIS:")
        print("-" * 40)
        print(analysis_results)
        
        print("\n📜 GENERATED SCRIPTS:")
        print(f"  • {script_path} - Run this to apply all fixes")

    def run_ci_check(self):
        """Run a quick check suitable for CI pipelines"""
        print("🔍 CI Check Mode - Quick Analysis")
        
        context = self.gather_project_context()
        predictions = self.predictive_analyzer.predict_compilation_issues(context)
        
        # Generate a quick status report
        report = {
            "timestamp": datetime.now().isoformat(),
            "build_status": context["build_status"],
            "predictions": predictions,
            "file_counts": {k: v for k, v in context.items() if k.endswith('_files')}
        }
        
        report_path = self.project_root / "grok_ci_report.json"
        report_path.write_text(json.dumps(report, indent=2))
        
        print(f"✅ CI report generated: {report_path}")
        
        # Exit with error code if build is failing
        if not context["build_status"]["success"]:
            print("❌ Build is failing - exiting with error")
            sys.exit(1)

    def close(self):
        """Cleanup resources"""
        self.client.close()


def main():
    """Main entry point with enhanced command options"""
    reviewer = SuperchargedMIDIReviewer()
    
    try:
        if len(sys.argv) > 1:
            if sys.argv[1] in ["super", "enhanced", "all"]:
                reviewer.run_super_analysis()
            elif sys.argv[1] == "ci-check":
                reviewer.run_ci_check()
            elif sys.argv[1] == "predict":
                context = reviewer.gather_project_context()
                predictions = reviewer.predictive_analyzer.predict_compilation_issues(context)
                print("🔮 PREDICTED ISSUES:")
                for prediction in predictions:
                    print(f"  • {prediction}")
            elif sys.argv[1] == "history":
                patterns = reviewer.analysis_learner.get_most_likely_issues()
                print("📚 HISTORICAL PATTERNS:")
                for issue, count in patterns:
                    print(f"  • {issue}: {count} occurrences")
            elif sys.argv[1] == "fix":
                context = reviewer.gather_project_context()
                applied = reviewer.auto_fixer.apply_recommended_fixes(context)
                print(f"🔧 APPLIED FIXES: {applied}")
            else:
                print("Usage: python3 supercharged_grok_reviewer.py [command]")
                print("\nEnhanced Commands:")
                print("  super        - Full supercharged analysis (recommended)")
                print("  ci-check     - Quick analysis for CI pipelines")
                print("  predict      - Show predicted issues only")
                print("  history      - Show historical patterns only")
                print("  fix          - Apply automated fixes only")
        else:
            reviewer.run_super_analysis()
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()

```

### `grok/ultra_supercharged_grok_reviewer.py` {#grok-ultra-supercharged-grok-reviewer-py}

- **Lines**: 744 (code: 429, comments: 197, blank: 118)
- **Pylint Score**: 6.38/10 ⚠️
- **Docstring Coverage**: 84.1%
- **Functions**: main, __init__, start_capture, stop_capture, write, flush, __init__, start_analysis, update_progress, __init__
  *... and 27 more*
- **Classes**: TerminalOutputCapture, ProgressTracker, RealErrorCapture, SimpleAutoFixer, PredictiveAnalyzer, AnalysisLearner, UltraSuperchargedMIDIReviewer

#### Source Code

```python
#!/usr/bin/env python3
"""
ULTRA SUPERCHARGED Grok 4 Project Reviewer for MIDI Software Center - PRODUCTION VERSION
Captures real build errors, applies fixes, and generates AI-powered analysis reports.
"""

import os
import sys
import json
import time
import re
import subprocess
from pathlib import Path
from datetime import datetime
import httpx
from io import StringIO


class TerminalOutputCapture:
    """Captures all terminal output to a Markdown file"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.original_stdout = sys.stdout
        self.original_stderr = sys.stderr
        self.capture_buffer = StringIO()
        self.markdown_content = []
        self.start_time = datetime.now()
    
    def start_capture(self):
        """Start capturing all terminal output"""
        sys.stdout = self
        sys.stderr = self
        self.markdown_content = [
            "# MIDI Software Center Analysis Report",
            f"**Generated**: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}",
            f"**Project**: {self.project_root}",
            "",
            "## Terminal Output",
            "```bash"
        ]
    
    def stop_capture(self):
        """Stop capturing and save to Markdown file"""
        sys.stdout = self.original_stdout
        sys.stderr = self.original_stderr
        
        # Close the code block
        self.markdown_content.append("```")
        
        # Add summary section
        self.markdown_content.extend([
            "",
            "## Analysis Summary",
            f"- **Analysis completed**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"- **Duration**: {(datetime.now() - self.start_time).total_seconds():.1f} seconds",
            f"- **Output captured**: {len(self.capture_buffer.getvalue())} characters",
        ])
        
        # Save to file
        report_path = self.project_root / "analysis_report.md"
        report_path.write_text('\n'.join(self.markdown_content))
        print(f"📄 Analysis report saved: {report_path}")
    
    def write(self, text):
        """Capture write operations"""
        self.original_stdout.write(text)
        self.capture_buffer.write(text)
        
        # Add to markdown, handling special characters
        clean_text = text.replace('`', '\\`')
        self.markdown_content.append(clean_text.rstrip())
    
    def flush(self):
        """Flush buffer"""
        self.original_stdout.flush()


class ProgressTracker:
    """Tracks and displays real-time progress"""
    
    def __init__(self):
        self.start_time = time.time()
        self.completed_steps = 0
        self.total_steps = 0
    
    def start_analysis(self, total_steps):
        """Start tracking analysis progress"""
        self.total_steps = total_steps
        self.completed_steps = 0
        self.start_time = time.time()
    
    def update_progress(self, step_name, results=None):
        """Update progress with current step"""
        self.completed_steps += 1
        progress = (self.completed_steps / self.total_steps) * 100
        elapsed = time.time() - self.start_time
        
        status_line = f"📊 [{progress:.1f}%] {step_name}"
        if results:
            status_line += f" | {results}"
        
        print(status_line)


class RealErrorCapture:
    """Captures and analyzes real build errors"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def capture_live_errors(self):
        """Run actual builds and capture real errors"""
        print("🔍 Capturing live build errors...")
        
        # Run cargo check and test compilation
        builds = {
            "rust_check": self.run_cargo_check(),
            "test_compilation": self.run_test_compilation()
        }
        
        # Extract ACTUAL error messages
        actual_errors = []
        for build_name, result in builds.items():
            if not result["success"]:
                errors = self.parse_compiler_errors(result["output"])
                for error in errors:
                    error["source"] = build_name
                actual_errors.extend(errors)
        
        print(f"✅ Captured {len(actual_errors)} actual errors from builds")
        return actual_errors
    
    def run_cargo_check(self):
        """Run cargo check for quick compilation check"""
        try:
            result = subprocess.run(
                ["cargo", "check", "--workspace"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            return {
                "success": result.returncode == 0,
                "output": result.stderr,
                "returncode": result.returncode
            }
        except Exception as e:
            return {"success": False, "output": str(e), "returncode": -1}
    
    def run_test_compilation(self):
        """Check test compilation"""
        try:
            result = subprocess.run(
                ["cargo", "test", "--no-run", "--workspace"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            return {
                "success": result.returncode == 0,
                "output": result.stderr,
                "returncode": result.returncode
            }
        except Exception as e:
            return {"success": False, "output": str(e), "returncode": -1}
    
    def parse_compiler_errors(self, output):
        """Parse actual compiler error messages"""
        errors = []
        lines = output.split('\n')
        
        for line in lines:
            line = line.strip()
            if not line:
                continue
                
            # Rust errors
            if 'error[' in line or 'ERROR' in line:
                errors.append({
                    "type": "rust_compilation",
                    "message": line[:200],
                    "file": self.extract_file_from_error(line),
                    "line": self.extract_line_from_error(line)
                })
            # General error patterns
            elif 'error:' in line.lower() and len(line) > 20:
                errors.append({
                    "type": "general",
                    "message": line[:200],
                    "file": "unknown",
                    "line": "unknown"
                })
        
        return errors[:50]
    
    def extract_file_from_error(self, error_line):
        """Extract filename from Rust error"""
        match = re.search(r'--> ([^:]+\.rs):', error_line)
        return match.group(1) if match else "unknown"
    
    def extract_line_from_error(self, error_line):
        """Extract line number from Rust error"""
        match = re.search(r':(\d+):\d+', error_line)
        return match.group(1) if match else "unknown"


class SimpleAutoFixer:
    """Auto-fixer for common compilation issues"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def apply_quick_fixes(self, actual_errors):
        """Apply quick fixes based on actual errors"""
        print("🔧 Applying quick fixes...")
        applied_fixes = []
        
        # Fix 1: Clean build artifacts
        if self.run_cargo_clean():
            applied_fixes.append("cargo_clean")
        
        # Fix 2: Update dependencies
        if self.run_cargo_update():
            applied_fixes.append("cargo_update")
        
        # Fix 3: Fix dead code warnings
        dead_code_fixed = self.fix_dead_code_warnings(actual_errors)
        if dead_code_fixed:
            applied_fixes.append("dead_code_fixes")
        
        return applied_fixes
    
    def run_cargo_clean(self):
        """Run cargo clean"""
        try:
            result = subprocess.run(
                ["cargo", "clean"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            if result.returncode == 0:
                print("✅ Cleaned build artifacts")
                return True
        except:
            pass
        return False
    
    def run_cargo_update(self):
        """Run cargo update"""
        try:
            result = subprocess.run(
                ["cargo", "update"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=60
            )
            if result.returncode == 0:
                print("✅ Updated dependencies")
                return True
        except:
            pass
        return False
    
    def fix_dead_code_warnings(self, actual_errors):
        """Report dead code warnings"""
        dead_code_errors = [e for e in actual_errors if "dead code" in e["message"].lower()]
        
        if not dead_code_errors:
            return False
            
        print(f"🔧 Found {len(dead_code_errors)} dead code warnings")
        for error in dead_code_errors[:5]:
            print(f"   🛠️  {error['message'][:100]}...")
        
        return len(dead_code_errors) > 0


class PredictiveAnalyzer:
    """Predicts future errors based on patterns"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
    
    def predict_compilation_issues(self):
        """Predict what will break before it happens"""
        print("🔮 Running predictive analysis...")
        predictions = []
        
        # Check for common issues
        if self.has_build_config_issues():
            predictions.append("🔨 Build configuration issues detected")
        
        if self.has_missing_midi_dependencies():
            predictions.append("🎵 Missing MIDI-specific dependencies")
        
        if not self.is_workspace_configured():
            predictions.append("🏗️  Workspace not properly configured")
        
        return predictions
    
    def has_build_config_issues(self):
        """Check for build configuration issues"""
        root_cargo = self.project_root / "Cargo.toml"
        if root_cargo.exists():
            content = root_cargo.read_text()
            return "[workspace]" not in content
        return True
    
    def has_missing_midi_dependencies(self):
        """Check for missing MIDI-specific dependencies"""
        cargo_files = list(self.project_root.glob("**/Cargo.toml"))
        for cargo_file in cargo_files:
            content = cargo_file.read_text()
            midi_crates = ["midly", "midir", "cpal"]
            if any(crate in content for crate in midi_crates):
                return False
        return True
    
    def is_workspace_configured(self):
        """Check if Cargo workspace is properly configured"""
        root_cargo = self.project_root / "Cargo.toml"
        if root_cargo.exists():
            content = root_cargo.read_text()
            return "[workspace]" in content
        return False


class AnalysisLearner:
    """Learns from previous analyses to improve future ones"""
    
    def __init__(self, project_root):
        self.project_root = Path(project_root)
        self.history_file = self.project_root / "grok_analysis_history.json"
    
    def get_most_likely_issues(self):
        """Get historical patterns"""
        patterns = []
        if self.history_file.exists():
            try:
                history = json.loads(self.history_file.read_text())
                error_counts = {}
                for stage in history.get("analysis_stages", []):
                    result = stage.get("result", "")
                    if "error" in result.lower():
                        error_counts["errors"] = error_counts.get("errors", 0) + 1
                    if "dead code" in result.lower():
                        error_counts["dead_code"] = error_counts.get("dead_code", 0) + 1
                
                for issue, count in error_counts.items():
                    if count > 0:
                        patterns.append((issue, count))
            except:
                pass
        
        return patterns
    
    def save_analysis(self, analysis_data):
        """Save current analysis to history"""
        try:
            history = {}
            if self.history_file.exists():
                try:
                    history = json.loads(self.history_file.read_text())
                except:
                    pass
            
            if "analysis_stages" not in history:
                history["analysis_stages"] = []
            
            history["analysis_stages"].append({
                "timestamp": datetime.now().isoformat(),
                "data": analysis_data
            })
            
            # Keep only last 5 analyses
            if len(history["analysis_stages"]) > 5:
                history["analysis_stages"] = history["analysis_stages"][-5:]
            
            self.history_file.write_text(json.dumps(history, indent=2))
        except Exception as e:
            print(f"⚠️  Could not save analysis history: {e}")


class UltraSuperchargedMIDIReviewer:
    """Main class for comprehensive project analysis"""
    
    def __init__(self, api_key=None, project_root=None):
        self.api_key = api_key or os.getenv("GROK_API_KEY")
        self.project_root = Path(project_root) if project_root else Path.cwd()
        
        # Initialize components
        self.terminal_capture = TerminalOutputCapture(self.project_root)
        self.progress_tracker = ProgressTracker()
        self.error_capture = RealErrorCapture(self.project_root)
        self.auto_fixer = SimpleAutoFixer(self.project_root)
        self.predictive_analyzer = PredictiveAnalyzer(self.project_root)
        self.analysis_learner = AnalysisLearner(self.project_root)
        
        self.client = httpx.Client(timeout=120.0, http2=True)
        
        if not self.api_key:
            print("❌ GROK_API_KEY not set")
            print("   export GROK_API_KEY='xai-...'")
            sys.exit(1)

    def generate_intelligent_prompt(self, actual_errors, predictions, historical_patterns):
        """Generate comprehensive analysis prompt"""
        
        error_summary = "\n".join([
            f"{i+1}. {error['type']}: {error['message']}" 
            for i, error in enumerate(actual_errors[:10])
        ])
        
        prompt = f"""
🚨 MIDI SOFTWARE CENTER - COMPREHENSIVE ANALYSIS

ACTUAL ERRORS FOUND ({len(actual_errors)} total):
{error_summary if error_summary else "No errors captured"}

PREDICTED ISSUES:
{chr(10).join(f"- {prediction}" for prediction in predictions) if predictions else "No predictions"}

HISTORICAL PATTERNS:
{chr(10).join(f"- {issue[0]}: {issue[1]} occurrences" for issue in historical_patterns) if historical_patterns else "No historical patterns"}

PROJECT OVERVIEW:
- Rust project with Tauri desktop apps
- MIDI music production software
- Multiple workspaces (DAW, Pipeline, etc.)

REQUESTED ANALYSIS:
1. Root cause analysis of the main compilation errors
2. Specific fix recommendations for Rust/Tauri projects
3. MIDI-specific architectural considerations
4. Prioritized action plan

Please provide concrete, actionable advice.
"""
        return prompt

    def generate_fix_script(self):
        """Generate executable fix script"""
        print("📜 Generating fix script...")
        
        script_content = """#!/bin/bash
# MIDI Software Center Fix Script
# Generated: {date}

set -e

echo "🎵 MIDI Software Center - Automated Fixes"
echo "========================================"

# Fix 1: Clean build artifacts
echo "🔧 Cleaning build artifacts..."
cargo clean

# Fix 2: Update dependencies
echo "🔧 Updating dependencies..."
cargo update

# Fix 3: Check compilation
echo "🔍 Checking compilation..."
cargo check --workspace

# Fix 4: Configure workspace if needed
if ! grep -q "[workspace]" Cargo.toml 2>/dev/null; then
    echo "🔧 Adding workspace configuration..."
    cat >> Cargo.toml << 'EOF'

[workspace]
members = [
    "daw/src-tauri",
    "pipeline/src-tauri", 
    "shared/rust",
    "scripts/test-midi-files"
]
EOF
fi

echo ""
echo "✅ Fixes applied!"
echo ""
echo "📊 Next steps:"
echo "   1. Run 'cargo build --workspace' to verify"
echo "   2. Run 'cargo test --workspace' to check tests"
echo "   3. Address any remaining errors manually"
""".format(date=datetime.now().isoformat())
        
        script_path = self.project_root / "auto_fix.sh"
        script_path.write_text(script_content)
        script_path.chmod(0o755)
        
        print(f"✅ Fix script generated: {script_path}")
        return script_path

    def generate_troubleshooting_guide(self, analysis_results, actual_errors, applied_fixes):
        """Generate comprehensive troubleshooting guide"""
        
        guide = f"""# MIDI Software Center Troubleshooting Guide
Generated: {datetime.now().isoformat()}

## Current Status
- **Errors Found**: {len(actual_errors)}
- **Fixes Applied**: {len(applied_fixes)}
- **Main Error Types**: {', '.join(set(e['type'] for e in actual_errors)) if actual_errors else 'None'}

## Applied Fixes
{chr(10).join(f"- {fix}" for fix in applied_fixes) if applied_fixes else "No fixes applied yet"}

## Common Solutions

### 1. Build Configuration Issues
Add workspace configuration to Cargo.toml:
```toml
[workspace]
members = [
    "daw/src-tauri",
    "pipeline/src-tauri",
    "shared/rust", 
    "scripts/test-midi-files"
]
```

### 2. Dependency Issues
```bash
# Update all dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --workspace
```

### 3. Dead Code Warnings
Add `#[allow(dead_code)]` above structs or functions that are intentionally unused.

### 4. MIDI-Specific Issues
Ensure MIDI crates are in Cargo.toml:
```toml
[dependencies]
midir = "0.7"
midly = "0.2"
cpal = "0.17"
```

## Verification Steps
1. Run `cargo check --workspace` to verify compilation
2. Run `cargo test --workspace` to check tests
3. Run `cargo build --workspace` for full build
4. Run `cargo clippy --workspace` for linting advice

## Generated Artifacts
- `auto_fix.sh` - Executable fix script
- `analysis_report.md` - Full analysis output
- `grok_analysis_history.json` - Historical data

## AI Analysis
{analysis_results}
"""

        guide_path = self.project_root / "TROUBLESHOOTING_GUIDE.md"
        guide_path.write_text(guide)
        print(f"📖 Troubleshooting guide generated: {guide_path}")
        return guide_path

    def call_grok(self, prompt: str) -> str:
        """Call Grok API with proper error handling"""
        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }
        
        payload = {
            "model": "grok-4-fast-reasoning",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.7,
            "max_tokens": 3000,
        }
        
        try:
            response = self.client.post(
                "https://api.x.ai/v1/chat/completions",
                json=payload,
                headers=headers,
                timeout=60.0
            )
            response.raise_for_status()
            result = response.json()
            return result["choices"][0]["message"]["content"]
        except Exception as e:
            return f"❌ API Error: {str(e)}"

    def run_analysis(self):
        """Run complete ultra supercharged analysis"""
        print("🚀 ULTRA SUPERCHARGED MIDI ANALYSIS ACTIVATED")
        print("=" * 60)
        
        # Start terminal output capture
        self.terminal_capture.start_capture()
        
        try:
            # Define analysis steps
            self.progress_tracker.start_analysis(6)
            
            # 1. Capture ACTUAL errors
            actual_errors = self.error_capture.capture_live_errors()
            self.progress_tracker.update_progress("Captured live errors", f"{len(actual_errors)} errors")
            
            # 2. Run predictive analysis
            predictions = self.predictive_analyzer.predict_compilation_issues()
            self.progress_tracker.update_progress("Predictive analysis", f"{len(predictions)} predictions")
            
            # 3. Load historical patterns
            historical_patterns = self.analysis_learner.get_most_likely_issues()
            self.progress_tracker.update_progress("Historical analysis", f"{len(historical_patterns)} patterns")
            
            # 4. Apply quick fixes
            applied_fixes = self.auto_fixer.apply_quick_fixes(actual_errors)
            self.progress_tracker.update_progress("Applied fixes", f"{len(applied_fixes)} fixes")
            
            # 5. Get AI analysis
            prompt = self.generate_intelligent_prompt(actual_errors, predictions, historical_patterns)
            ai_analysis = self.call_grok(prompt)
            self.progress_tracker.update_progress("AI analysis", "Complete")
            
            # 6. Generate output files
            script_path = self.generate_fix_script()
            guide_path = self.generate_troubleshooting_guide(ai_analysis, actual_errors, applied_fixes)
            self.progress_tracker.update_progress("Generated outputs", "Script + Guide")
            
            # Save analysis for learning
            self.analysis_learner.save_analysis({
                "error_count": len(actual_errors),
                "applied_fixes": applied_fixes,
                "predictions": predictions
            })
            
            # Display results
            self.display_results(ai_analysis, predictions, historical_patterns, applied_fixes, actual_errors, script_path, guide_path)
            
            print("\n🎉 ANALYSIS COMPLETE!")
            print("📁 Generated files:")
            print(f"   - {script_path}")
            print(f"   - {guide_path}")
            print("   - analysis_report.md")
            print("   - grok_analysis_history.json")
            
        except Exception as e:
            print(f"❌ Analysis failed: {e}")
            import traceback
            traceback.print_exc()
        finally:
            # Always stop capture
            self.terminal_capture.stop_capture()

    def display_results(self, analysis_results, predictions, historical_patterns, applied_fixes, actual_errors, script_path, guide_path):
        """Display comprehensive results"""
        print("\n" + "="*70)
        print("📊 ANALYSIS RESULTS")
        print("="*70)
        
        print("\n🔮 PREDICTIONS:")
        for prediction in predictions:
            print(f"  • {prediction}")
        
        print("\n📚 HISTORICAL PATTERNS:")
        for pattern, count in historical_patterns:
            print(f"  • {pattern}: {count} occurrences")
        
        print("\n🔧 APPLIED FIXES:")
        for fix in applied_fixes:
            print(f"  • {fix}")
        
        print(f"\n❌ CURRENT ERRORS: {len(actual_errors)}")
        for i, error in enumerate(actual_errors[:5]):
            print(f"  {i+1}. [{error['type']}] {error['message']}")
        if len(actual_errors) > 5:
            print(f"  ... and {len(actual_errors) - 5} more errors")
        
        print("\n🧠 AI ANALYSIS:")
        print("-" * 40)
        print(analysis_results[:1500])
        if len(analysis_results) > 1500:
            print("\n... (see TROUBLESHOOTING_GUIDE.md for full analysis)")
        
        print("\n📜 GENERATED FILES:")
        print(f"  • {script_path}")
        print(f"  • {guide_path}")

    def close(self):
        """Cleanup resources"""
        self.client.close()


def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="ULTRA SUPERCHARGED Grok 4 Project Reviewer"
    )
    parser.add_argument(
        "--project",
        default=None,
        help="Project root directory (default: current directory)"
    )
    parser.add_argument(
        "--api-key",
        default=None,
        help="Grok API key (default: GROK_API_KEY env var)"
    )
    parser.add_argument(
        "command",
        nargs="?",
        default="analyze",
        choices=["analyze", "ultra", "super"],
        help="Command to run"
    )
    
    args = parser.parse_args()
    
    reviewer = UltraSuperchargedMIDIReviewer(
        api_key=args.api_key,
        project_root=args.project
    )
    
    try:
        if args.command in ["analyze", "ultra", "super"]:
            reviewer.run_analysis()
        else:
            parser.print_help()
    finally:
        reviewer.close()


if __name__ == "__main__":
    main()

```

### `import-split-files.py` {#import-split-files-py}

- **Lines**: 276 (code: 177, comments: 56, blank: 43)
- **Pylint Score**: 8.88/10 ✅
- **Docstring Coverage**: 100.0%
- **Functions**: calculate_blake3_hash, parse_midi_basic, extract_parent_info_from_filename, process_file, get_missing_files, import_files

#### Source Code

```python
#!/usr/bin/env python3
"""
Import Missing Split Files into Database

Imports split track files that exist on disk but are not yet in the database.
Uses BLAKE3 for deduplication and parses MIDI for basic metadata.
"""

import os
import sys
import psycopg2
from psycopg2 import IntegrityError
from pathlib import Path
from multiprocessing import Pool, cpu_count
import hashlib
import mido
import re

DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"
SPLITS_DIR = "/home/dojevou/tmp/midi_splits_fast"

def calculate_blake3_hash(filepath):
    """Calculate BLAKE3 hash of file (fallback to SHA256 if blake3 not available)"""
    try:
        import blake3
        with open(filepath, 'rb') as f:
            return blake3.blake3(f.read()).digest()
    except ImportError:
        # Fallback to SHA256 if blake3 not available
        with open(filepath, 'rb') as f:
            return hashlib.sha256(f.read()).digest()

def parse_midi_basic(filepath):
    """Parse MIDI file for basic metadata"""
    try:
        mid = mido.MidiFile(filepath)

        # Get duration in ticks
        total_ticks = 0
        for track in mid.tracks:
            track_ticks = sum(msg.time for msg in track)
            if track_ticks > total_ticks:
                total_ticks = track_ticks

        # Get duration in seconds (approximate)
        tempo = 500000  # Default tempo (120 BPM)
        for track in mid.tracks:
            for msg in track:
                if msg.type == 'set_tempo':
                    tempo = msg.tempo
                    break

        ticks_per_beat = mid.ticks_per_beat
        seconds_per_tick = tempo / (ticks_per_beat * 1_000_000)
        duration_seconds = total_ticks * seconds_per_tick

        return {
            'format': mid.type,
            'num_tracks': len(mid.tracks),
            'ticks_per_quarter_note': mid.ticks_per_beat,
            'duration_seconds': round(duration_seconds, 3),
            'duration_ticks': total_ticks,
        }
    except Exception as e:
        print(f"Error parsing {filepath}: {e}")
        return None

def extract_parent_info_from_filename(filename):
    """Extract parent file ID and track info from split filename
    Format: {parent_id}_{original_name}_{track_num}_{instrument}.mid
    """
    # Example: 6614730_mikeshiver-feelings_bren-f_03_Bass pad.mid
    match = re.match(r'^(\d+)_', filename)
    if match:
        parent_id = int(match.group(1))

        # Try to extract track number (look for _XX_ pattern)
        track_match = re.search(r'_(\d{2})_', filename)
        track_number = int(track_match.group(1)) if track_match else 1

        return parent_id, track_number
    return None, None

def process_file(filepath):
    """Process a single split file and return data for insertion"""
    try:
        path = Path(filepath)
        filename = path.name

        # Get parent file ID from filename
        parent_id, track_number = extract_parent_info_from_filename(filename)

        # Calculate hash
        content_hash = calculate_blake3_hash(filepath)

        # Get file size
        file_size = path.stat().st_size

        # Parse MIDI
        midi_data = parse_midi_basic(filepath)
        if not midi_data:
            return None

        return {
            'filename': filename,
            'filepath': str(path),
            'content_hash': content_hash,
            'file_size_bytes': file_size,
            'format': midi_data['format'],
            'num_tracks': midi_data['num_tracks'],
            'ticks_per_quarter_note': midi_data['ticks_per_quarter_note'],
            'duration_seconds': midi_data['duration_seconds'],
            'duration_ticks': midi_data['duration_ticks'],
            'parent_file_id': parent_id,
            'track_number': track_number,
        }
    except Exception as e:
        print(f"Error processing {filepath}: {e}")
        return None

def get_missing_files(limit=None):
    """Get list of files that exist on disk but not in database"""
    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Get all files currently in database from splits directory
    cur.execute("""
        SELECT filepath
        FROM files
        WHERE filepath LIKE '/home/dojevou/tmp/midi_splits_fast/%'
    """)

    db_files = set(row[0] for row in cur.fetchall())
    cur.close()
    conn.close()

    print(f"Found {len(db_files):,} split files in database")

    # Get all files on disk
    disk_files = []
    for file in Path(SPLITS_DIR).glob("*.mid"):
        if str(file) not in db_files:
            disk_files.append(str(file))
            if limit and len(disk_files) >= limit:
                break

    print(f"Found {len(disk_files):,} files on disk not in database")
    return disk_files

def import_files(files_to_import, batch_size=1000, workers=8):
    """Import missing files into database"""
    conn = psycopg2.connect(DB_URL)

    total = len(files_to_import)
    imported = 0
    errors = 0
    duplicates_skipped = 0

    print(f"\n{'='*70}")
    print(f"IMPORTING {total:,} SPLIT FILES")
    print(f"{'='*70}")
    print(f"Workers: {workers}")
    print(f"Batch size: {batch_size}\n")

    # Process files in parallel
    with Pool(workers) as pool:
        for i in range(0, total, batch_size):
            batch = files_to_import[i:i+batch_size]
            results = pool.map(process_file, batch)

            # Filter out None results (errors)
            valid_results = [r for r in results if r is not None]
            errors += len(results) - len(valid_results)

            if not valid_results:
                continue

            # Insert batch into database
            cur = conn.cursor()

            for data in valid_results:
                try:
                    # Check if hash already exists (deduplication)
                    cur.execute("""
                        SELECT id FROM files WHERE content_hash = %s
                    """, (data['content_hash'],))

                    if cur.fetchone():
                        duplicates_skipped += 1
                        continue

                    # Insert into files table
                    cur.execute("""
                        INSERT INTO files (
                            filename, filepath, original_filename, content_hash,
                            file_size_bytes, format, num_tracks, ticks_per_quarter_note,
                            duration_seconds, duration_ticks, parent_file_id
                        ) VALUES (
                            %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s
                        ) RETURNING id
                    """, (
                        data['filename'], data['filepath'], data['filename'],
                        data['content_hash'], data['file_size_bytes'],
                        data['format'], data['num_tracks'],
                        data['ticks_per_quarter_note'], data['duration_seconds'],
                        data['duration_ticks'], data['parent_file_id']
                    ))

                    file_id = cur.fetchone()[0]

                    # Create track_splits relationship if parent exists
                    if data['parent_file_id']:
                        cur.execute("""
                            INSERT INTO track_splits (
                                parent_file_id, split_file_id, track_number
                            ) VALUES (%s, %s, %s)
                            ON CONFLICT DO NOTHING
                        """, (data['parent_file_id'], file_id, data['track_number']))

                    imported += 1

                except IntegrityError:
                    conn.rollback()
                    duplicates_skipped += 1
                except Exception as e:
                    conn.rollback()
                    errors += 1
                    if errors <= 10:
                        print(f"Error importing {data['filename']}: {e}")

            conn.commit()
            cur.close()

            # Progress update
            progress = min(i + batch_size, total)
            pct = (progress / total * 100)
            print(f"Progress: {progress:,}/{total:,} ({pct:.1f}%) | "
                  f"Imported: {imported:,} | Skipped: {duplicates_skipped:,} | "
                  f"Errors: {errors:,}")

    conn.close()

    print(f"\n{'='*70}")
    print(f"IMPORT COMPLETE")
    print(f"{'='*70}")
    print(f"Files imported:      {imported:,}")
    print(f"Duplicates skipped:  {duplicates_skipped:,}")
    print(f"Errors:              {errors:,}")
    print(f"Total processed:     {imported + duplicates_skipped + errors:,}")
    print()

if __name__ == "__main__":
    if "--test" in sys.argv:
        print("Testing on 100 files...")
        missing = get_missing_files(limit=100)
        if missing:
            import_files(missing, batch_size=50, workers=4)
    elif "--run" in sys.argv:
        auto_yes = "--yes" in sys.argv
        if not auto_yes:
            response = input("Import ALL missing split files? Type 'yes': ")
            confirmed = response.lower() == 'yes'
        else:
            confirmed = True

        if confirmed:
            missing = get_missing_files()
            if missing:
                import_files(missing, batch_size=1000, workers=8)
        else:
            print("Cancelled.")
    else:
        print("Usage:")
        print("  --test    Test import on 100 files")
        print("  --run     Import all missing files")
        print("  --run --yes    Import all missing files without confirmation")

```

### `import-tool/Cargo.toml` {#import-tool-cargo-toml}

- **Lines**: 12 (code: 11, comments: 0, blank: 1)

#### Source Code

```toml
[package]
name = "import-tool"
version = "1.0.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
sqlx = { workspace = true }
chrono = { workspace = true }
walkdir = "2"
sha2 = { workspace = true }
regex = "1.10"

```

### `import-tool/src/main.rs` {#import-tool-src-main-rs}

- **Lines**: 483 (code: 419, comments: 0, blank: 64)

#### Source Code

```rust
/// Simple High-Performance MIDI Importer
///
/// Usage: ./import_midi ~/midi_extraction
///
/// This script:
/// - Scans recursively for .mid/.midi files
/// - Calculates SHA-256 hash for deduplication
/// - Inserts into database with parallel processing
/// - Shows real-time progress
use chrono::Utc;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        eprintln!("Example: {} ~/midi_extraction", args[0]);
        std::process::exit(1);
    }

    let directory = &args[1];

    println!("🎵 MIDI Library Importer");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Connect to database
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string()
    });

    println!("🔌 Connecting to database...");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(32)
        .connect(&db_url)
        .await?;
    println!("✓ Connected\n");

    // Collect MIDI files
    println!("📂 Scanning: {}", directory);
    let files: Vec<PathBuf> = walkdir::WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("mid") || s.eq_ignore_ascii_case("midi"))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let total = files.len();
    println!("✓ Found {} MIDI files\n", total);

    if total == 0 {
        println!("⚠️  No MIDI files found!");
        return Ok(());
    }

    println!("🚀 Starting import with 32 workers...\n");

    // Counters
    let imported = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let errors = Arc::new(AtomicUsize::new(0));

    let semaphore = Arc::new(Semaphore::new(32));
    let start_time = std::time::Instant::now();

    // Process files
    let mut tasks = vec![];

    for (idx, file_path) in files.into_iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let pool_clone = pool.clone();
        let imported = Arc::clone(&imported);
        let skipped = Arc::clone(&skipped);
        let errors = Arc::clone(&errors);

        let task = tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    eprintln!("Warning: Semaphore closed during import");
                    return;
                },
            };

            match process_file(&file_path, &pool_clone).await {
                Ok(true) => {
                    imported.fetch_add(1, Ordering::SeqCst);
                },
                Ok(false) => {
                    skipped.fetch_add(1, Ordering::SeqCst);
                },
                Err(_) => {
                    errors.fetch_add(1, Ordering::SeqCst);
                },
            }

            // Print progress every 100 files
            if idx % 100 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let processed = idx + 1;
                let rate = processed as f64 / elapsed;
                println!(
                    "Progress: {}/{} ({:.1}%) - {:.0} files/sec",
                    processed,
                    total,
                    (processed as f64 / total as f64) * 100.0,
                    rate
                );
            }
        });

        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await?;
    }

    let elapsed = start_time.elapsed();
    let imported_count = imported.load(Ordering::SeqCst);
    let skipped_count = skipped.load(Ordering::SeqCst);
    let error_count = errors.load(Ordering::SeqCst);

    // Final report
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Import Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total:     {}", total);
    println!("Imported:  {} ✓", imported_count);
    println!("Skipped:   {} (duplicates)", skipped_count);
    println!("Errors:    {}", error_count);
    println!();
    println!("⏱️  Time:  {:.2}s", elapsed.as_secs_f64());
    println!(
        "⚡ Rate:  {:.0} files/sec",
        total as f64 / elapsed.as_secs_f64()
    );
    println!();

    Ok(())
}

async fn process_file(
    file_path: &std::path::Path,
    pool: &sqlx::PgPool,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Read file
    let file_data = tokio::fs::read(file_path).await?;

    // Calculate hash
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let content_hash = hasher.finalize().to_vec();

    // Parse basic MIDI info
    let (num_tracks, tpqn) = parse_midi_basic(&file_data);

    // Get parent folder name
    let parent_folder = file_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // Get filename (normalize .midi -> .mid)
    let original_filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.mid")
        .to_string();

    let filename = if original_filename.to_lowercase().ends_with(".midi") {
        original_filename[..original_filename.len() - 1].to_string() // .midi -> .mid
    } else {
        original_filename.clone()
    };

    let filepath = file_path.to_str().unwrap_or("").to_string();

    // Extract filename metadata (Phase 2)
    let metadata = FilenameMetadata::extract_from_filename(&filename);

    // Calculate metadata source
    let metadata_source = calculate_metadata_source(
        metadata.bpm,
        metadata.key.as_deref(),
        &metadata.genres,
        &metadata.structure_tags,
    );

    // Insert into database with Phase 2 columns
    let result = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO files (
            filename,
            original_filename,
            filepath,
            parent_folder,
            content_hash,
            file_size_bytes,
            num_tracks,
            ticks_per_quarter_note,
            filename_bpm,
            filename_key,
            filename_genres,
            structure_tags,
            track_number,
            metadata_source,
            created_at,
            updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        ON CONFLICT (content_hash) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(&filename)
    .bind(&original_filename)
    .bind(&filepath)
    .bind(parent_folder)
    .bind(&content_hash)
    .bind(file_data.len() as i64)
    .bind(num_tracks)
    .bind(tpqn)
    .bind(metadata.bpm)
    .bind(metadata.key)
    .bind(&metadata.genres)
    .bind(&metadata.structure_tags)
    .bind(metadata.track_number)
    .bind(metadata_source)
    .bind(Utc::now())
    .bind(Utc::now())
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

// ============================================================================
// Filename Metadata Extraction (Phase 2)
// ============================================================================

/// Filename metadata extracted from a MIDI file name
#[derive(Debug, Clone)]
struct FilenameMetadata {
    bpm: Option<f32>,
    key: Option<String>,
    genres: Vec<String>,
    structure_tags: Vec<String>,
    track_number: Option<i32>,
}

impl FilenameMetadata {
    fn extract_from_filename(filename: &str) -> Self {
        FilenameMetadata {
            bpm: extract_bpm_from_filename(filename),
            key: extract_key_from_filename(filename),
            genres: extract_genres_from_filename(filename),
            structure_tags: extract_structure_tags(filename),
            track_number: extract_leading_number(filename),
        }
    }
}

// BPM Extraction
static BPM_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_bpm_regex() -> &'static Regex {
    BPM_REGEX.get_or_init(|| {
        Regex::new(
            r"(?i)([0-9]{2,3})[\s_]*(bpm|beats|tempo)|(?:^|_|\s|-|/)([0-9]{2,3})(?:_|\s|-|/|\.)",
        )
        .unwrap()
    })
}

fn extract_bpm_from_filename(filename: &str) -> Option<f32> {
    let regex = get_bpm_regex();

    // First, try to find explicit BPM notation
    for caps in regex.captures_iter(filename) {
        if let Some(m) = caps.get(1) {
            if let Ok(bpm) = m.as_str().parse::<f32>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    // If no explicit BPM found, try implicit numbers
    for caps in regex.captures_iter(filename) {
        if let Some(m) = caps.get(3) {
            if let Ok(bpm) = m.as_str().parse::<f32>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    None
}

// Key Signature Extraction
static KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static KEY_NORMALIZATION_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_key_regex() -> &'static Regex {
    KEY_REGEX
        .get_or_init(|| Regex::new(r"(?i)(?:^|_|\s|-|/)([A-G](?:#|b)?m?)(?:_|\s|-|/|\.)").unwrap())
}

fn get_key_map() -> &'static HashMap<&'static str, &'static str> {
    KEY_NORMALIZATION_MAP.get_or_init(|| {
        HashMap::from([
            ("amin", "Am"),
            ("am", "Am"),
            ("bmin", "Bm"),
            ("bm", "Bm"),
            ("cmin", "Cm"),
            ("cm", "Cm"),
            ("dmin", "Dm"),
            ("dm", "Dm"),
            ("emin", "Em"),
            ("em", "Em"),
            ("fmin", "Fm"),
            ("fm", "Fm"),
            ("gmin", "Gm"),
            ("gm", "Gm"),
            ("amaj", "A"),
            ("cmaj", "C"),
            ("dmaj", "D"),
            ("emaj", "E"),
            ("fmaj", "F"),
            ("gmaj", "G"),
            ("bmaj", "B"),
            ("bb", "Bb"),
            ("a#", "A#"),
            ("c#", "C#"),
            ("d#", "D#"),
            ("f#", "F#"),
            ("g#", "G#"),
            ("ab", "Ab"),
            ("db", "Db"),
            ("eb", "Eb"),
            ("gb", "Gb"),
            ("a", "A"),
            ("b", "B"),
            ("c", "C"),
            ("d", "D"),
            ("e", "E"),
            ("f", "F"),
            ("g", "G"),
        ])
    })
}

fn normalize_key_signature(raw_key: &str) -> Option<String> {
    get_key_map().get(raw_key.to_lowercase().as_str()).map(|&s| s.to_string())
}

fn extract_key_from_filename(filename: &str) -> Option<String> {
    get_key_regex()
        .captures(filename)
        .and_then(|caps| caps.get(1))
        .and_then(|m| normalize_key_signature(m.as_str()))
}

// Genre Extraction
static GENRE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_genre_regex() -> &'static Regex {
    GENRE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(house|techno|trance|hip.?hop|trap|dubstep|dnb|drum.?n.?bass|jazz|funk|soul|rock|pop|edm|ambient|downtempo|break|jungle|garage|electro|acid|minimal|deep|progressive)").unwrap()
    })
}

fn normalize_genre(raw: &str) -> String {
    match raw {
        "hip hop" | "hiphop" | "hip-hop" => "hip-hop".to_string(),
        "dnb" | "drum n bass" | "drum and bass" | "drum-n-bass" | "drum_n_bass" => {
            "dnb".to_string()
        },
        genre => genre.to_string(),
    }
}

fn extract_genres_from_filename(filename: &str) -> Vec<String> {
    get_genre_regex()
        .find_iter(filename)
        .map(|m| m.as_str().to_lowercase())
        .map(|g| normalize_genre(&g))
        .collect()
}

// Structure Tag Extraction
static STRUCTURE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_structure_regex() -> &'static Regex {
    STRUCTURE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(verse|chorus|bridge|intro|outro|drop|build|breakdown|fill|loop|one.?shot|sample|melody|hook|riff|lick|main|full|short|long)").unwrap()
    })
}

fn normalize_structure_tag(raw: &str) -> String {
    match raw {
        "one shot" | "one-shot" | "oneshot" => "oneshot".to_string(),
        tag => tag.to_string(),
    }
}

fn extract_structure_tags(filename: &str) -> Vec<String> {
    get_structure_regex()
        .find_iter(filename)
        .map(|m| m.as_str().to_lowercase())
        .map(|s| normalize_structure_tag(&s))
        .collect()
}

// Track Number Extraction
static LEADING_NUMBER_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_leading_number_regex() -> &'static Regex {
    LEADING_NUMBER_REGEX.get_or_init(|| Regex::new(r"^([0-9]+)").unwrap())
}

fn extract_leading_number(filename: &str) -> Option<i32> {
    get_leading_number_regex()
        .captures(filename)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<i32>().ok())
}

/// Calculate metadata source based on what data is available
fn calculate_metadata_source(
    filename_bpm: Option<f32>,
    filename_key: Option<&str>,
    filename_genres: &[String],
    structure_tags: &[String],
) -> &'static str {
    let has_filename_metadata = filename_bpm.is_some()
        || filename_key.is_some()
        || !filename_genres.is_empty()
        || !structure_tags.is_empty();

    if has_filename_metadata {
        "filename"
    } else {
        "none"
    }
}

// ============================================================================
// MIDI Parsing
// ============================================================================

/// Parse basic MIDI info (tracks and TPPQ)
fn parse_midi_basic(data: &[u8]) -> (i16, i32) {
    if data.len() < 14 || &data[0..4] != b"MThd" {
        return (1, 480);
    }

    let num_tracks = i16::from_be_bytes([data[10], data[11]]);
    let ticks = u16::from_be_bytes([data[12], data[13]]);

    (num_tracks, ticks as i32)
}

```

### `mpcpattern_to_midi.py` {#mpcpattern-to-midi-py}

- **Lines**: 131 (code: 79, comments: 21, blank: 31)
- **Pylint Score**: 9.00/10 ✅
- **Docstring Coverage**: 50.0%
- **Functions**: mpcpattern_to_midi, main

#### Source Code

```python
#!/usr/bin/env python3
"""
Convert .mpcpattern (JSON) to standard MIDI file

This validates our understanding of the .mpcpattern format by converting it
back to MIDI and comparing with originals.
"""

import json
import sys
from pathlib import Path
from midiutil import MIDIFile

def mpcpattern_to_midi(mpcpattern_path, midi_output_path):
    """Convert .mpcpattern JSON to MIDI file"""

    # Read .mpcpattern file
    with open(mpcpattern_path, 'r') as f:
        data = json.load(f)

    pattern = data['pattern']
    events = pattern['events']

    # Create MIDI file (1 track)
    midi = MIDIFile(1)
    track = 0
    channel = 0
    time = 0

    # Set tempo (120 BPM default)
    tempo = 120
    midi.addTempo(track, time, tempo)

    # PPQN (pulses per quarter note) - MPC uses 480
    # MIDIFile uses beats, so we need to convert ticks to beats
    ppqn = 480

    print(f"\nConverting: {mpcpattern_path.name}")
    print(f"Total events: {len(events)}")

    # Separate events by type
    type1_events = [e for e in events if e.get('type') == 1]
    type2_events = [e for e in events if e.get('type') == 2]

    print(f"  Type 1 (note off): {len(type1_events)}")
    print(f"  Type 2 (note on):  {len(type2_events)}")

    # Process Type 2 events (Note On with duration)
    for event in type2_events:
        note_num = event['1']
        velocity_normalized = event['2']
        time_ticks = event['time']
        duration_ticks = event['len']

        # Convert normalized velocity (0.0-1.0) back to MIDI (0-127)
        velocity = int(velocity_normalized * 127)

        # Convert ticks to beats (quarter notes)
        time_beats = time_ticks / ppqn
        duration_beats = duration_ticks / ppqn

        # Add note to MIDI
        midi.addNote(
            track=track,
            channel=channel,
            pitch=note_num,
            time=time_beats,
            duration=duration_beats,
            volume=velocity
        )

    print(f"  Added {len(type2_events)} notes")

    # Write MIDI file
    with open(midi_output_path, 'wb') as f:
        midi.writeFile(f)

    print(f"  ✓ Saved: {midi_output_path.name}")

    return len(type2_events)

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 mpcpattern_to_midi.py <file.mpcpattern> [output.mid]")
        print("   or: python3 mpcpattern_to_midi.py --batch <input_dir> <output_dir>")
        sys.exit(1)

    if sys.argv[1] == '--batch':
        if len(sys.argv) < 4:
            print("Batch mode requires input and output directories")
            sys.exit(1)

        input_dir = Path(sys.argv[2])
        output_dir = Path(sys.argv[3])

        print(f"Batch converting .mpcpattern files...")
        print(f"  Input:  {input_dir}")
        print(f"  Output: {output_dir}")
        print()

        output_dir.mkdir(parents=True, exist_ok=True)

        # Find all .mpcpattern files
        pattern_files = list(input_dir.rglob('*.mpcpattern'))
        print(f"Found {len(pattern_files)} .mpcpattern files\n")

        total_notes = 0
        for i, pattern_file in enumerate(pattern_files):
            output_file = output_dir / f"{pattern_file.stem}.mid"

            try:
                notes = mpcpattern_to_midi(pattern_file, output_file)
                total_notes += notes
            except Exception as e:
                print(f"  ✗ Error: {e}")

        print(f"\n✓ Batch conversion complete: {len(pattern_files)} files, {total_notes} total notes")

    else:
        # Single file mode
        input_path = Path(sys.argv[1])

        if len(sys.argv) >= 3:
            output_path = Path(sys.argv[2])
        else:
            output_path = input_path.with_suffix('.mid')

        mpcpattern_to_midi(input_path, output_path)

if __name__ == '__main__':
    main()

```

### `normalize-files-and-database.py` {#normalize-files-and-database-py}

- **Lines**: 217 (code: 134, comments: 47, blank: 36)
- **Pylint Score**: 8.93/10 ✅
- **Docstring Coverage**: 100.0%
- **Functions**: sanitize_strict, normalize_files_and_database

#### Source Code

```python
#!/usr/bin/env python3
"""
Normalize Files AND Database Together
Keeps files and database in perfect sync during normalization.

Normalizes:
- Extensions: .MID, .MIDI, .midi → .mid
- Spaces: replaced with underscores
- Special chars: removed (keep only a-z, A-Z, 0-9, _, -, .)
"""

import os
import sys
import psycopg2
from pathlib import Path
import re
from collections import defaultdict

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def sanitize_strict(filename):
    """
    Apply same sanitization as Rust normalize_filenames.rs
    """
    result = filename

    # 1. Replace spaces with underscores
    result = result.replace(' ', '_')

    # 2. Remove special characters (keep only a-z, A-Z, 0-9, _, -, .)
    result = re.sub(r'[^a-zA-Z0-9_\-.]+', '', result)

    # 3. Normalize extension to lowercase .mid
    result = re.sub(r'\.(mid|MID|MIDI|midi|Mid|MiD)$', '.mid', result, flags=re.IGNORECASE)

    # 4. Remove consecutive underscores/dashes
    result = re.sub(r'_+', '_', result)
    result = re.sub(r'-+', '-', result)

    # 5. Remove leading/trailing underscores/dashes
    result = re.sub(r'^[_-]+', '', result)
    result = re.sub(r'[_-]+$', '', result)

    return result

def normalize_files_and_database(limit=None, dry_run=True, commit_every=1000):
    """
    Normalize files on disk AND database records together.

    Args:
        limit: Number of files to process (None = all)
        dry_run: If True, only simulate
        commit_every: Commit database transaction every N files
    """
    conn = psycopg2.connect(DB_URL)

    # Track duplicates per directory
    dir_filenames = defaultdict(set)

    stats = {
        'processed': 0,
        'normalized': 0,
        'skipped': 0,
        'errors': 0,
        'db_updated': 0,
        'disk_renamed': 0,
    }

    print(f"\n{'='*70}")
    print(f"NORMALIZE FILES AND DATABASE TOGETHER")
    print(f"{'='*70}")
    print(f"Mode: {'DRY RUN' if dry_run else 'LIVE MODE'}")
    print(f"Commit every: {commit_every} files")
    print()

    try:
        cur = conn.cursor()

        # Get all files (or limited batch)
        query = "SELECT id, filename, filepath FROM files ORDER BY id"
        if limit:
            query += f" LIMIT {limit}"

        cur.execute(query)
        all_files = cur.fetchall()
        total = len(all_files)

        print(f"Found {total:,} files to process")
        print()

        for idx, (file_id, db_filename, db_filepath) in enumerate(all_files, 1):
            try:
                # Calculate normalized filename
                normalized_filename = sanitize_strict(db_filename)

                # Skip if no change needed
                if normalized_filename == db_filename:
                    stats['skipped'] += 1
                    stats['processed'] += 1
                    continue

                # Get paths
                db_path = Path(db_filepath)
                parent_dir = db_path.parent
                new_path = parent_dir / normalized_filename

                # Handle duplicates by appending counter
                final_filename = normalized_filename
                final_path = new_path
                counter = 1

                # Check both disk and our tracking dict for duplicates
                while (final_path.exists() and final_path != db_path) or \
                      final_filename in dir_filenames[str(parent_dir)]:
                    stem = Path(normalized_filename).stem
                    ext = Path(normalized_filename).suffix
                    final_filename = f"{stem}_{counter}{ext}"
                    final_path = parent_dir / final_filename
                    counter += 1

                # Mark this filename as used in this directory
                dir_filenames[str(parent_dir)].add(final_filename)

                if dry_run:
                    if idx <= 20:  # Only print first 20 in dry run
                        print(f"Would normalize:")
                        print(f"  {db_filename} → {final_filename}")
                        if counter > 1:
                            print(f"  (duplicate handled with _{counter-1})")
                    stats['normalized'] += 1
                else:
                    # STEP 1: Rename file on disk
                    if not db_path.exists():
                        print(f"⚠️  File not found on disk: {db_path}")
                        stats['errors'] += 1
                        stats['processed'] += 1
                        continue

                    os.rename(db_path, final_path)
                    stats['disk_renamed'] += 1

                    # STEP 2: Update database
                    new_filepath = str(final_path)

                    update_query = """
                        UPDATE files
                        SET
                            filename = %s,
                            filepath = %s,
                            updated_at = NOW()
                        WHERE id = %s
                    """
                    cur.execute(update_query, (final_filename, new_filepath, file_id))
                    stats['db_updated'] += 1
                    stats['normalized'] += 1

                    # Commit periodically
                    if stats['normalized'] % commit_every == 0:
                        conn.commit()
                        print(f"Normalized {stats['normalized']:,} / {total:,} files ({stats['normalized']/total*100:.1f}%)")

                stats['processed'] += 1

            except Exception as e:
                print(f"❌ Error processing file {file_id}: {e}")
                stats['errors'] += 1
                if not dry_run:
                    conn.rollback()
                stats['processed'] += 1

        # Final commit
        if not dry_run:
            conn.commit()
            print(f"Normalized {stats['normalized']:,} / {total:,} files (100%)")

        cur.close()

    finally:
        conn.close()

    print(f"\n{'='*70}")
    print(f"NORMALIZATION {'SIMULATION' if dry_run else 'COMPLETE'}")
    print(f"{'='*70}")
    print(f"Files processed:     {stats['processed']:,}")
    print(f"Files normalized:    {stats['normalized']:,}")
    print(f"Files skipped:       {stats['skipped']:,}")
    print(f"Disk renames:        {stats['disk_renamed']:,}")
    print(f"Database updates:    {stats['db_updated']:,}")
    print(f"Errors:              {stats['errors']:,}")
    print()

    return stats

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        print("Testing on 100 files (dry run)...")
        normalize_files_and_database(limit=100, dry_run=True)
    elif len(sys.argv) > 1 and sys.argv[1] == "--test-live":
        print("Testing on 100 files (LIVE MODE)...")
        response = input("This will rename 100 files. Type 'yes' to confirm: ")
        if response.lower() == 'yes':
            normalize_files_and_database(limit=100, dry_run=False)
        else:
            print("Cancelled.")
    elif len(sys.argv) > 1 and sys.argv[1] == "--run":
        print("Running full normalization (LIVE MODE)...")
        response = input("This will rename ~650K files AND update database. Type 'yes' to confirm: ")
        if response.lower() == 'yes':
            normalize_files_and_database(limit=None, dry_run=False, commit_every=5000)
        else:
            print("Cancelled.")
    else:
        print("Usage:")
        print("  --test        Test on 100 files (dry run)")
        print("  --test-live   Test on 100 files (LIVE MODE)")
        print("  --run         Run full normalization (LIVE MODE)")

```

### `project_report_20251130_090335.json` {#project-report-20251130-090335-json}

- **Lines**: 2165 (code: 2165, comments: 0, blank: 0)

#### Source Code

```json
{
  "timestamp": "20251130_090335",
  "project": {
    "name": "scripts",
    "path": "/home/dojevou/projects/midi-software-center/scripts",
    "total_files": 31,
    "total_lines": 8343,
    "total_loc": 5612,
    "total_functions": 208,
    "total_classes": 19,
    "avg_pylint_score": 7.85217391304348,
    "avg_complexity": 0,
    "avg_maintainability": 0,
    "docstring_coverage": 77.88311688311688,
    "files_by_extension": {
      ".py": {
        "count": 24,
        "lines": 6990
      },
      ".rs": {
        "count": 4,
        "lines": 1249
      },
      ".ts": {
        "count": 1,
        "lines": 77
      },
      ".toml": {
        "count": 2,
        "lines": 27
      }
    },
    "top_imports": [
      [
        "pathlib",
        20
      ],
      [
        "sys",
        20
      ],
      [
        "re",
        15
      ],
      [
        "os",
        12
      ],
      [
        "collections",
        8
      ],
      [
        "json",
        8
      ],
      [
        "psycopg2",
        7
      ],
      [
        "subprocess",
        6
      ],
      [
        "time",
        5
      ],
      [
        "typing",
        4
      ],
      [
        "datetime",
        4
      ],
      [
        "httpx",
        4
      ],
      [
        "concurrent.futures",
        2
      ],
      [
        "multiprocessing",
        2
      ],
      [
        "argparse",
        2
      ],
      [
        "psycopg2.extras",
        2
      ],
      [
        "traceback",
        2
      ],
      [
        "csv",
        1
      ],
      [
        "dataclasses",
        1
      ],
      [
        "shutil",
        1
      ],
      [
        "io",
        1
      ],
      [
        "blake3",
        1
      ],
      [
        "hashlib",
        1
      ],
      [
        "mido",
        1
      ],
      [
        "midiutil",
        1
      ],
      [
        "normalize_files_and_database",
        1
      ],
      [
        "tempfile",
        1
      ]
    ],
    "security_issues": [],
    "all_todos": []
  },
  "files": [
    {
      "path": "analysis/analyze_files.py",
      "name": "analyze_files.py",
      "extension": ".py",
      "lines": 40,
      "lines_of_code": 30,
      "blank_lines": 7,
      "comment_lines": 3,
      "imports": [],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": null,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": [
        "SyntaxError: Could not parse file"
      ]
    },
    {
      "path": "analysis/error_parser.py",
      "name": "error_parser.py",
      "extension": ".py",
      "lines": 261,
      "lines_of_code": 206,
      "blank_lines": 35,
      "comment_lines": 20,
      "imports": [
        "collections",
        "csv",
        "dataclasses",
        "json",
        "pathlib",
        "re",
        "sys",
        "typing"
      ],
      "functions": [
        {
          "name": "__post_init__",
          "line": 25,
          "args": 1
        },
        {
          "name": "__init__",
          "line": 30,
          "args": 2
        },
        {
          "name": "_compile_patterns",
          "line": 36,
          "args": 1
        },
        {
          "name": "parse",
          "line": 65,
          "args": 1
        },
        {
          "name": "_categorize_error",
          "line": 92,
          "args": 2
        },
        {
          "name": "_get_error_type",
          "line": 115,
          "args": 2
        },
        {
          "name": "_extract_affected_items",
          "line": 122,
          "args": 2
        },
        {
          "name": "_generate_report",
          "line": 132,
          "args": 1
        },
        {
          "name": "export_csv",
          "line": 146,
          "args": 2
        },
        {
          "name": "export_json",
          "line": 167,
          "args": 2
        },
        {
          "name": "print_summary",
          "line": 190,
          "args": 1
        },
        {
          "name": "_calculate_priority",
          "line": 220,
          "args": 2
        }
      ],
      "classes": [
        {
          "name": "ErrorInfo",
          "line": 15,
          "methods": 1
        },
        {
          "name": "QuantumErrorParser",
          "line": 29,
          "methods": 11
        }
      ],
      "todos": [],
      "pylint_score": 6.31,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 71.42857142857143,
      "issues": []
    },
    {
      "path": "analyze-tool/src/analyzer.rs",
      "name": "analyzer.rs",
      "extension": ".rs",
      "lines": 347,
      "lines_of_code": 287,
      "blank_lines": 60,
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
      "path": "analyze-tool/src/tag_extractor.rs",
      "name": "tag_extractor.rs",
      "extension": ".rs",
      "lines": 151,
      "lines_of_code": 122,
      "blank_lines": 29,
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
      "path": "analyze_filenames.py",
      "name": "analyze_filenames.py",
      "extension": ".py",
      "lines": 569,
      "lines_of_code": 427,
      "blank_lines": 81,
      "comment_lines": 61,
      "imports": [
        "collections",
        "concurrent.futures",
        "json",
        "multiprocessing",
        "pathlib",
        "re",
        "sys",
        "time"
      ],
      "functions": [
        {
          "name": "extract_bpm",
          "line": 40,
          "args": 1
        },
        {
          "name": "extract_key",
          "line": 65,
          "args": 1
        },
        {
          "name": "extract_time_signature",
          "line": 79,
          "args": 1
        },
        {
          "name": "extract_instruments",
          "line": 94,
          "args": 2
        },
        {
          "name": "extract_genres",
          "line": 167,
          "args": 2
        },
        {
          "name": "extract_patterns",
          "line": 256,
          "args": 2
        },
        {
          "name": "extract_drum_elements",
          "line": 290,
          "args": 2
        },
        {
          "name": "analyze_file",
          "line": 335,
          "args": 1
        },
        {
          "name": "process_batch",
          "line": 383,
          "args": 1
        },
        {
          "name": "generate_report",
          "line": 391,
          "args": 2
        },
        {
          "name": "main",
          "line": 487,
          "args": 0
        },
        {
          "name": "__init__",
          "line": 15,
          "args": 1
        },
        {
          "name": "merge",
          "line": 28,
          "args": 2
        }
      ],
      "classes": [
        {
          "name": "FilenameAnalyzer",
          "line": 14,
          "methods": 2
        }
      ],
      "todos": [],
      "pylint_score": 9.47,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 78.57142857142857,
      "issues": []
    },
    {
      "path": "audit-missing-imports.ts",
      "name": "audit-missing-imports.ts",
      "extension": ".ts",
      "lines": 77,
      "lines_of_code": 69,
      "blank_lines": 8,
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
      "path": "build_tag_expansions.py",
      "name": "build_tag_expansions.py",
      "extension": ".py",
      "lines": 441,
      "lines_of_code": 292,
      "blank_lines": 76,
      "comment_lines": 73,
      "imports": [
        "argparse",
        "collections",
        "datetime",
        "json",
        "os",
        "pathlib",
        "psycopg2",
        "subprocess",
        "sys"
      ],
      "functions": [
        {
          "name": "query_files_for_pack",
          "line": 154,
          "args": 1
        },
        {
          "name": "get_bpm_folder",
          "line": 205,
          "args": 2
        },
        {
          "name": "convert_and_organize",
          "line": 225,
          "args": 4
        },
        {
          "name": "build_pack",
          "line": 363,
          "args": 2
        },
        {
          "name": "list_packs",
          "line": 395,
          "args": 0
        },
        {
          "name": "main",
          "line": 405,
          "args": 0
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 7.83,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 83.33333333333334,
      "issues": []
    },
    {
      "path": "convert_instruments_python.py",
      "name": "convert_instruments_python.py",
      "extension": ".py",
      "lines": 165,
      "lines_of_code": 108,
      "blank_lines": 30,
      "comment_lines": 27,
      "imports": [
        "os",
        "pathlib",
        "subprocess",
        "sys"
      ],
      "functions": [
        {
          "name": "main",
          "line": 25,
          "args": 0
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 8.7,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "derive_injector.py",
      "name": "derive_injector.py",
      "extension": ".py",
      "lines": 249,
      "lines_of_code": 168,
      "blank_lines": 49,
      "comment_lines": 32,
      "imports": [
        "pathlib",
        "re",
        "sys",
        "typing"
      ],
      "functions": [
        {
          "name": "__init__",
          "line": 13,
          "args": 2
        },
        {
          "name": "find_struct_definitions",
          "line": 26,
          "args": 2
        },
        {
          "name": "get_existing_derives",
          "line": 43,
          "args": 2
        },
        {
          "name": "inject_derives",
          "line": 54,
          "args": 3
        },
        {
          "name": "fix_tagged_response",
          "line": 82,
          "args": 2
        },
        {
          "name": "fix_import_progress",
          "line": 117,
          "args": 2
        },
        {
          "name": "fix_emitter_generics",
          "line": 151,
          "args": 2
        },
        {
          "name": "fix_file",
          "line": 167,
          "args": 2
        },
        {
          "name": "run",
          "line": 211,
          "args": 1
        },
        {
          "name": "replace_tag_response",
          "line": 92,
          "args": 1
        },
        {
          "name": "replace_import_progress",
          "line": 126,
          "args": 1
        }
      ],
      "classes": [
        {
          "name": "DeriveMacroInjector",
          "line": 12,
          "methods": 9
        }
      ],
      "todos": [],
      "pylint_score": 5.73,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 66.66666666666666,
      "issues": []
    },
    {
      "path": "export-force-midi.py",
      "name": "export-force-midi.py",
      "extension": ".py",
      "lines": 329,
      "lines_of_code": 196,
      "blank_lines": 52,
      "comment_lines": 81,
      "imports": [
        "os",
        "pathlib",
        "psycopg2",
        "psycopg2.extras",
        "shutil",
        "sys"
      ],
      "functions": [
        {
          "name": "connect_db",
          "line": 32,
          "args": 0
        },
        {
          "name": "export_progressions",
          "line": 36,
          "args": 3
        },
        {
          "name": "export_arp_patterns",
          "line": 131,
          "args": 3
        },
        {
          "name": "export_drum_patterns",
          "line": 212,
          "args": 3
        },
        {
          "name": "main",
          "line": 291,
          "args": 0
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 9.18,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 80.0,
      "issues": []
    },
    {
      "path": "fast_multi_level_tagger.py",
      "name": "fast_multi_level_tagger.py",
      "extension": ".py",
      "lines": 361,
      "lines_of_code": 177,
      "blank_lines": 65,
      "comment_lines": 119,
      "imports": [
        "collections",
        "pathlib",
        "psycopg2",
        "psycopg2.extras",
        "re",
        "sys",
        "time",
        "traceback",
        "typing"
      ],
      "functions": [
        {
          "name": "normalize_keyword",
          "line": 32,
          "args": 1
        },
        {
          "name": "extract_path_components",
          "line": 72,
          "args": 1
        },
        {
          "name": "load_curated_tags",
          "line": 94,
          "args": 1
        },
        {
          "name": "process_files",
          "line": 143,
          "args": 2
        },
        {
          "name": "insert_batch",
          "line": 239,
          "args": 2
        },
        {
          "name": "verify_results",
          "line": 262,
          "args": 1
        },
        {
          "name": "main",
          "line": 320,
          "args": 0
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 9.22,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 100.0,
      "issues": []
    },
    {
      "path": "fixes/fix_add_tags_calls.py",
      "name": "fix_add_tags_calls.py",
      "extension": ".py",
      "lines": 143,
      "lines_of_code": 85,
      "blank_lines": 29,
      "comment_lines": 29,
      "imports": [
        "pathlib",
        "re",
        "sys"
      ],
      "functions": [
        {
          "name": "fix_add_tags_to_file",
          "line": 15,
          "args": 1
        },
        {
          "name": "fix_add_tags_to_file_impl",
          "line": 72,
          "args": 1
        },
        {
          "name": "process_file",
          "line": 89,
          "args": 1
        },
        {
          "name": "main",
          "line": 112,
          "args": 0
        },
        {
          "name": "replacer1",
          "line": 22,
          "args": 1
        },
        {
          "name": "replacer2",
          "line": 32,
          "args": 1
        },
        {
          "name": "replacer3",
          "line": 61,
          "args": 1
        },
        {
          "name": "replacer",
          "line": 80,
          "args": 1
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 9.47,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 50.0,
      "issues": []
    },
    {
      "path": "fixes/fix_e0308_appstate.py",
      "name": "fix_e0308_appstate.py",
      "extension": ".py",
      "lines": 20,
      "lines_of_code": 10,
      "blank_lines": 6,
      "comment_lines": 4,
      "imports": [
        "re"
      ],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": 6.0,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "fixes/fix_e0308_pool.py",
      "name": "fix_e0308_pool.py",
      "extension": ".py",
      "lines": 46,
      "lines_of_code": 27,
      "blank_lines": 9,
      "comment_lines": 10,
      "imports": [
        "re"
      ],
      "functions": [],
      "classes": [],
      "todos": [],
      "pylint_score": 4.29,
      "complexity": null,
      "maintainability": null,
      "has_docstring": false,
      "docstring_coverage": 0.0,
      "issues": []
    },
    {
      "path": "fixes/fix_list_files.py",
      "name": "fix_list_files.py",
      "extension": ".py",
      "lines": 82,
      "lines_of_code": 41,
      "blank_lines": 13,
      "comment_lines": 28,
      "imports": [
        "re",
        "sys"
      ],
      "functions": [
        {
          "name": "fix_list_files_calls",
          "line": 23,
          "args": 1
        },
        {
          "name": "main",
          "line": 44,
          "args": 0
        },
        {
          "name": "replace_fn",
          "line": 37,
          "args": 1
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 5.75,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 33.33333333333333,
      "issues": []
    },
    {
      "path": "fixes/format_string_fixer.py",
      "name": "format_string_fixer.py",
      "extension": ".py",
      "lines": 198,
      "lines_of_code": 123,
      "blank_lines": 41,
      "comment_lines": 34,
      "imports": [
        "pathlib",
        "re",
        "subprocess",
        "sys",
        "typing"
      ],
      "functions": [
        {
          "name": "__init__",
          "line": 14,
          "args": 2
        },
        {
          "name": "_compile_patterns",
          "line": 20,
          "args": 1
        },
        {
          "name": "find_rust_files",
          "line": 42,
          "args": 1
        },
        {
          "name": "analyze_format_string",
          "line": 46,
          "args": 3
        },
        {
          "name": "fix_file",
          "line": 64,
          "args": 2
        },
        {
          "name": "run",
          "line": 160,
          "args": 1
        },
        {
          "name": "replace_indexed_with_empty",
          "line": 85,
          "args": 1
        },
        {
          "name": "replace_indexed_format",
          "line": 112,
          "args": 1
        },
        {
          "name": "replace_write_indexed",
          "line": 139,
          "args": 1
        }
      ],
      "classes": [
        {
          "name": "FormatStringFixer",
          "line": 13,
          "methods": 6
        }
      ],
      "todos": [],
      "pylint_score": 4.51,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 50.0,
      "issues": []
    },
    {
      "path": "generate_detailed_report.py",
      "name": "generate_detailed_report.py",
      "extension": ".py",
      "lines": 356,
      "lines_of_code": 275,
      "blank_lines": 60,
      "comment_lines": 21,
      "imports": [
        "json",
        "pathlib"
      ],
      "functions": [
        {
          "name": "load_analysis",
          "line": 9,
          "args": 1
        },
        {
          "name": "generate_comprehensive_report",
          "line": 15,
          "args": 0
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 9.35,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 100.0,
      "issues": []
    },
    {
      "path": "grok/grok4_project_reviewer.py",
      "name": "grok4_project_reviewer.py",
      "extension": ".py",
      "lines": 184,
      "lines_of_code": 115,
      "blank_lines": 31,
      "comment_lines": 38,
      "imports": [
        "httpx",
        "os",
        "pathlib",
        "sys",
        "time"
      ],
      "functions": [
        {
          "name": "main",
          "line": 155,
          "args": 0
        },
        {
          "name": "__init__",
          "line": 18,
          "args": 2
        },
        {
          "name": "call_grok",
          "line": 44,
          "args": 3
        },
        {
          "name": "analyze_compilation_errors",
          "line": 115,
          "args": 1
        },
        {
          "name": "close",
          "line": 150,
          "args": 1
        }
      ],
      "classes": [
        {
          "name": "Grok4ProjectReviewer",
          "line": 15,
          "methods": 4
        }
      ],
      "todos": [],
      "pylint_score": 9.43,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 100.0,
      "issues": []
    },
    {
      "path": "grok/midi_grok_reviewer.py",
      "name": "midi_grok_reviewer.py",
      "extension": ".py",
      "lines": 797,
      "lines_of_code": 495,
      "blank_lines": 93,
      "comment_lines": 209,
      "imports": [
        "datetime",
        "httpx",
        "json",
        "os",
        "pathlib",
        "subprocess",
        "sys",
        "time"
      ],
      "functions": [
        {
          "name": "main",
          "line": 768,
          "args": 0
        },
        {
          "name": "__init__",
          "line": 20,
          "args": 2
        },
        {
          "name": "gather_project_context",
          "line": 47,
          "args": 1
        },
        {
          "name": "detect_project_phase",
          "line": 63,
          "args": 1
        },
        {
          "name": "analyze_midi_architecture",
          "line": 85,
          "args": 1
        },
        {
          "name": "analyze_rust_backends",
          "line": 96,
          "args": 1
        },
        {
          "name": "analyze_backend",
          "line": 105,
          "args": 2
        },
        {
          "name": "analyze_tauri_frontends",
          "line": 129,
          "args": 1
        },
        {
          "name": "analyze_frontend",
          "line": 137,
          "args": 2
        },
        {
          "name": "analyze_database_layer",
          "line": 165,
          "args": 1
        },
        {
          "name": "analyze_shared_components",
          "line": 192,
          "args": 1
        },
        {
          "name": "analyze_build_system",
          "line": 213,
          "args": 1
        },
        {
          "name": "collect_midi_build_errors",
          "line": 226,
          "args": 1
        },
        {
          "name": "build_rust_workspace",
          "line": 240,
          "args": 1
        },
        {
          "name": "build_frontend",
          "line": 266,
          "args": 2
        },
        {
          "name": "check_typescript",
          "line": 300,
          "args": 1
        },
        {
          "name": "check_test_compilation",
          "line": 332,
          "args": 1
        },
        {
          "name": "analyze_test_health",
          "line": 355,
          "args": 1
        },
        {
          "name": "analyze_database",
          "line": 380,
          "args": 1
        },
        {
          "name": "extract_critical_components",
          "line": 403,
          "args": 1
        },
        {
          "name": "get_recent_changes",
          "line": 440,
          "args": 1
        },
        {
          "name": "run_midi_analysis_pipeline",
          "line": 471,
          "args": 2
        },
        {
          "name": "midi_architecture_review",
          "line": 492,
          "args": 2
        },
        {
          "name": "build_system_analysis",
          "line": 516,
          "args": 2
        },
        {
          "name": "test_health_analysis",
          "line": 541,
          "args": 2
        },
        {
          "name": "deployment_readiness",
          "line": 566,
          "args": 2
        },
        {
          "name": "analyze_error_patterns",
          "line": 593,
          "args": 2
        },
        {
          "name": "generate_midi_action_plan",
          "line": 624,
          "args": 4
        },
        {
          "name": "save_analysis_stage",
          "line": 665,
          "args": 3
        },
        {
          "name": "call_grok",
          "line": 675,
          "args": 3
        },
        {
          "name": "run_enhanced_midi_analysis",
          "line": 699,
          "args": 1
        },
        {
          "name": "display_midi_results",
          "line": 730,
          "args": 4
        },
        {
          "name": "save_analysis_history",
          "line": 750,
          "args": 1
        },
        {
          "name": "close",
          "line": 763,
          "args": 1
        }
      ],
      "classes": [
        {
          "name": "MIDIProjectReviewer",
          "line": 17,
          "methods": 33
        }
      ],
      "todos": [],
      "pylint_score": 8.94,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 97.14285714285714,
      "issues": []
    },
    {
      "path": "grok/supercharged_grok_reviewer.py",
      "name": "supercharged_grok_reviewer.py",
      "extension": ".py",
      "lines": 878,
      "lines_of_code": 557,
      "blank_lines": 148,
      "comment_lines": 173,
      "imports": [
        "collections",
        "datetime",
        "httpx",
        "json",
        "os",
        "pathlib",
        "subprocess",
        "sys"
      ],
      "functions": [
        {
          "name": "main",
          "line": 838,
          "args": 0
        },
        {
          "name": "__init__",
          "line": 22,
          "args": 2
        },
        {
          "name": "apply_recommended_fixes",
          "line": 25,
          "args": 2
        },
        {
          "name": "should_apply_fix",
          "line": 49,
          "args": 3
        },
        {
          "name": "fix_tauri_ts_conflicts",
          "line": 63,
          "args": 1
        },
        {
          "name": "rewrite_tauri_types",
          "line": 83,
          "args": 2
        },
        {
          "name": "fix_rust_dead_code",
          "line": 104,
          "args": 1
        },
        {
          "name": "create_missing_backends",
          "line": 134,
          "args": 1
        },
        {
          "name": "fix_build_config",
          "line": 172,
          "args": 1
        },
        {
          "name": "enable_disabled_tests",
          "line": 209,
          "args": 1
        },
        {
          "name": "__init__",
          "line": 232,
          "args": 2
        },
        {
          "name": "predict_compilation_issues",
          "line": 235,
          "args": 2
        },
        {
          "name": "has_misconfigured_dependencies",
          "line": 261,
          "args": 2
        },
        {
          "name": "will_break_on_strict_mode",
          "line": 274,
          "args": 2
        },
        {
          "name": "has_missing_midi_dependencies",
          "line": 282,
          "args": 1
        },
        {
          "name": "has_build_config_issues",
          "line": 293,
          "args": 1
        },
        {
          "name": "__init__",
          "line": 306,
          "args": 2
        },
        {
          "name": "load_historical_patterns",
          "line": 311,
          "args": 1
        },
        {
          "name": "extract_patterns",
          "line": 328,
          "args": 2
        },
        {
          "name": "categorize_errors",
          "line": 349,
          "args": 2
        },
        {
          "name": "get_most_likely_issues",
          "line": 367,
          "args": 1
        },
        {
          "name": "save_analysis",
          "line": 376,
          "args": 2
        },
        {
          "name": "__init__",
          "line": 402,
          "args": 2
        },
        {
          "name": "integrate_with_ci",
          "line": 411,
          "args": 1
        },
        {
          "name": "add_grok_step_to_ci",
          "line": 426,
          "args": 2
        },
        {
          "name": "create_github_workflow",
          "line": 459,
          "args": 1
        },
        {
          "name": "__init__",
          "line": 507,
          "args": 2
        },
        {
          "name": "generate_intelligent_prompt",
          "line": 525,
          "args": 3
        },
        {
          "name": "generate_fix_scripts",
          "line": 580,
          "args": 2
        },
        {
          "name": "extract_actionable_fixes",
          "line": 610,
          "args": 2
        },
        {
          "name": "run_super_analysis",
          "line": 664,
          "args": 1
        },
        {
          "name": "gather_project_context",
          "line": 727,
          "args": 1
        },
        {
          "name": "check_build_status",
          "line": 739,
          "args": 1
        },
        {
          "name": "call_grok",
          "line": 757,
          "args": 3
        },
        {
          "name": "display_enhanced_results",
          "line": 783,
          "args": 6
        },
        {
          "name": "run_ci_check",
          "line": 808,
          "args": 1
        },
        {
          "name": "close",
          "line": 833,
          "args": 1
        }
      ],
      "classes": [
        {
          "name": "MIDIAutoFixer",
          "line": 19,
          "methods": 9
        },
        {
          "name": "PredictiveAnalyzer",
          "line": 229,
          "methods": 6
        },
        {
          "name": "AnalysisLearner",
          "line": 303,
          "methods": 6
        },
        {
          "name": "DevelopmentWorkflowIntegrator",
          "line": 399,
          "methods": 4
        },
        {
          "name": "SuperchargedMIDIReviewer",
          "line": 504,
          "methods": 11
        }
      ],
      "todos": [],
      "pylint_score": 6.29,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 88.09523809523809,
      "issues": []
    },
    {
      "path": "grok/ultra_supercharged_grok_reviewer.py",
      "name": "ultra_supercharged_grok_reviewer.py",
      "extension": ".py",
      "lines": 744,
      "lines_of_code": 429,
      "blank_lines": 118,
      "comment_lines": 197,
      "imports": [
        "argparse",
        "datetime",
        "httpx",
        "io",
        "json",
        "os",
        "pathlib",
        "re",
        "subprocess",
        "sys",
        "time",
        "traceback"
      ],
      "functions": [
        {
          "name": "main",
          "line": 702,
          "args": 0
        },
        {
          "name": "__init__",
          "line": 22,
          "args": 2
        },
        {
          "name": "start_capture",
          "line": 30,
          "args": 1
        },
        {
          "name": "stop_capture",
          "line": 43,
          "args": 1
        },
        {
          "name": "write",
          "line": 65,
          "args": 2
        },
        {
          "name": "flush",
          "line": 74,
          "args": 1
        },
        {
          "name": "__init__",
          "line": 82,
          "args": 1
        },
        {
          "name": "start_analysis",
          "line": 87,
          "args": 2
        },
        {
          "name": "update_progress",
          "line": 93,
          "args": 3
        },
        {
          "name": "__init__",
          "line": 109,
          "args": 2
        },
        {
          "name": "capture_live_errors",
          "line": 112,
          "args": 1
        },
        {
          "name": "run_cargo_check",
          "line": 134,
          "args": 1
        },
        {
          "name": "run_test_compilation",
          "line": 152,
          "args": 1
        },
        {
          "name": "parse_compiler_errors",
          "line": 170,
          "args": 2
        },
        {
          "name": "extract_file_from_error",
          "line": 199,
          "args": 2
        },
        {
          "name": "extract_line_from_error",
          "line": 204,
          "args": 2
        },
        {
          "name": "__init__",
          "line": 213,
          "args": 2
        },
        {
          "name": "apply_quick_fixes",
          "line": 216,
          "args": 2
        },
        {
          "name": "run_cargo_clean",
          "line": 236,
          "args": 1
        },
        {
          "name": "run_cargo_update",
          "line": 253,
          "args": 1
        },
        {
          "name": "fix_dead_code_warnings",
          "line": 270,
          "args": 2
        },
        {
          "name": "__init__",
          "line": 287,
          "args": 2
        },
        {
          "name": "predict_compilation_issues",
          "line": 290,
          "args": 1
        },
        {
          "name": "has_build_config_issues",
          "line": 307,
          "args": 1
        },
        {
          "name": "has_missing_midi_dependencies",
          "line": 315,
          "args": 1
        },
        {
          "name": "is_workspace_configured",
          "line": 325,
          "args": 1
        },
        {
          "name": "__init__",
          "line": 337,
          "args": 2
        },
        {
          "name": "get_most_likely_issues",
          "line": 341,
          "args": 1
        },
        {
          "name": "save_analysis",
          "line": 363,
          "args": 2
        },
        {
          "name": "__init__",
          "line": 393,
          "args": 3
        },
        {
          "name": "generate_intelligent_prompt",
          "line": 412,
          "args": 4
        },
        {
          "name": "generate_fix_script",
          "line": 447,
          "args": 1
        },
        {
          "name": "generate_troubleshooting_guide",
          "line": 503,
          "args": 4
        },
        {
          "name": "call_grok",
          "line": 573,
          "args": 2
        },
        {
          "name": "run_analysis",
          "line": 600,
          "args": 1
        },
        {
          "name": "display_results",
          "line": 663,
          "args": 8
        },
        {
          "name": "close",
          "line": 697,
          "args": 1
        }
      ],
      "classes": [
        {
          "name": "TerminalOutputCapture",
          "line": 19,
          "methods": 5
        },
        {
          "name": "ProgressTracker",
          "line": 79,
          "methods": 3
        },
        {
          "name": "RealErrorCapture",
          "line": 106,
          "methods": 7
        },
        {
          "name": "SimpleAutoFixer",
          "line": 210,
          "methods": 5
        },
        {
          "name": "PredictiveAnalyzer",
          "line": 284,
          "methods": 5
        },
        {
          "name": "AnalysisLearner",
          "line": 334,
          "methods": 3
        },
        {
          "name": "UltraSuperchargedMIDIReviewer",
          "line": 390,
          "methods": 8
        }
      ],
      "todos": [],
      "pylint_score": 6.38,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 84.0909090909091,
      "issues": []
    },
    {
      "path": "import-split-files.py",
      "name": "import-split-files.py",
      "extension": ".py",
      "lines": 276,
      "lines_of_code": 177,
      "blank_lines": 43,
      "comment_lines": 56,
      "imports": [
        "blake3",
        "hashlib",
        "mido",
        "multiprocessing",
        "os",
        "pathlib",
        "psycopg2",
        "re",
        "sys"
      ],
      "functions": [
        {
          "name": "calculate_blake3_hash",
          "line": 22,
          "args": 1
        },
        {
          "name": "parse_midi_basic",
          "line": 33,
          "args": 1
        },
        {
          "name": "extract_parent_info_from_filename",
          "line": 68,
          "args": 1
        },
        {
          "name": "process_file",
          "line": 84,
          "args": 1
        },
        {
          "name": "get_missing_files",
          "line": 121,
          "args": 1
        },
        {
          "name": "import_files",
          "line": 150,
          "args": 3
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 8.88,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 100.0,
      "issues": []
    },
    {
      "path": "import-tool/Cargo.toml",
      "name": "Cargo.toml",
      "extension": ".toml",
      "lines": 12,
      "lines_of_code": 11,
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
      "path": "import-tool/src/main.rs",
      "name": "main.rs",
      "extension": ".rs",
      "lines": 483,
      "lines_of_code": 419,
      "blank_lines": 64,
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
      "path": "mpcpattern_to_midi.py",
      "name": "mpcpattern_to_midi.py",
      "extension": ".py",
      "lines": 131,
      "lines_of_code": 79,
      "blank_lines": 31,
      "comment_lines": 21,
      "imports": [
        "json",
        "midiutil",
        "pathlib",
        "sys"
      ],
      "functions": [
        {
          "name": "mpcpattern_to_midi",
          "line": 14,
          "args": 2
        },
        {
          "name": "main",
          "line": 82,
          "args": 0
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 9.0,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 50.0,
      "issues": []
    },
    {
      "path": "normalize-files-and-database.py",
      "name": "normalize-files-and-database.py",
      "extension": ".py",
      "lines": 217,
      "lines_of_code": 134,
      "blank_lines": 36,
      "comment_lines": 47,
      "imports": [
        "collections",
        "os",
        "pathlib",
        "psycopg2",
        "re",
        "sys"
      ],
      "functions": [
        {
          "name": "sanitize_strict",
          "line": 22,
          "args": 1
        },
        {
          "name": "normalize_files_and_database",
          "line": 47,
          "args": 3
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 8.93,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 100.0,
      "issues": []
    },
    {
      "path": "restore-original-filenames.py",
      "name": "restore-original-filenames.py",
      "extension": ".py",
      "lines": 154,
      "lines_of_code": 90,
      "blank_lines": 26,
      "comment_lines": 38,
      "imports": [
        "os",
        "pathlib",
        "psycopg2",
        "re",
        "sys"
      ],
      "functions": [
        {
          "name": "sanitize_strict",
          "line": 17,
          "args": 1
        },
        {
          "name": "restore_files",
          "line": 43,
          "args": 2
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 8.85,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 100.0,
      "issues": []
    },
    {
      "path": "strict-sanitize-filenames.py",
      "name": "strict-sanitize-filenames.py",
      "extension": ".py",
      "lines": 182,
      "lines_of_code": 117,
      "blank_lines": 32,
      "comment_lines": 33,
      "imports": [
        "collections",
        "concurrent.futures",
        "os",
        "pathlib",
        "re",
        "sys"
      ],
      "functions": [
        {
          "name": "sanitize_filename_strict",
          "line": 17,
          "args": 1
        },
        {
          "name": "sanitize_file",
          "line": 56,
          "args": 2
        },
        {
          "name": "find_midi_files",
          "line": 90,
          "args": 1
        },
        {
          "name": "main",
          "line": 100,
          "args": 0
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 9.3,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 75.0,
      "issues": []
    },
    {
      "path": "test-midi-files/Cargo.toml",
      "name": "Cargo.toml",
      "extension": ".toml",
      "lines": 15,
      "lines_of_code": 13,
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
    },
    {
      "path": "test-midi-files/src/main.rs",
      "name": "main.rs",
      "extension": ".rs",
      "lines": 268,
      "lines_of_code": 234,
      "blank_lines": 34,
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
      "path": "test-normalization-sample.py",
      "name": "test-normalization-sample.py",
      "extension": ".py",
      "lines": 167,
      "lines_of_code": 99,
      "blank_lines": 32,
      "comment_lines": 36,
      "imports": [
        "collections",
        "normalize_files_and_database",
        "os",
        "pathlib",
        "psycopg2",
        "re",
        "sys",
        "tempfile"
      ],
      "functions": [
        {
          "name": "test_on_files_needing_normalization",
          "line": 27,
          "args": 2
        },
        {
          "name": "sanitize_strict",
          "line": 64,
          "args": 1
        }
      ],
      "classes": [],
      "todos": [],
      "pylint_score": 8.79,
      "complexity": null,
      "maintainability": null,
      "has_docstring": true,
      "docstring_coverage": 50.0,
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
  "backup": "/home/dojevou/projects/midi-software-center/scripts/backups/project_backup_20251130_090335.zip"
}
```

### `restore-original-filenames.py` {#restore-original-filenames-py}

- **Lines**: 154 (code: 90, comments: 38, blank: 26)
- **Pylint Score**: 8.85/10 ✅
- **Docstring Coverage**: 100.0%
- **Functions**: sanitize_strict, restore_files

#### Source Code

```python
#!/usr/bin/env python3
"""
Restore Original Filenames
Reverts files back to their original database names after normalization.
Uses database as source of truth.
"""

import os
import sys
import psycopg2
from pathlib import Path
import re

# Database connection
DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def sanitize_strict(filename):
    """
    Apply same sanitization as Rust normalize_filenames.rs
    This tells us what the normalized filename would be.
    """
    result = filename

    # 1. Replace spaces with underscores
    result = result.replace(' ', '_')

    # 2. Remove special characters (keep only a-z, A-Z, 0-9, _, -, .)
    result = re.sub(r'[^a-zA-Z0-9_\-.]+', '', result)

    # 3. Normalize extension to lowercase .mid
    result = re.sub(r'\.(mid|MID|MIDI|midi|Mid|MiD)$', '.mid', result, flags=re.IGNORECASE)

    # 4. Remove consecutive underscores/dashes
    result = re.sub(r'_+', '_', result)
    result = re.sub(r'-+', '-', result)

    # 5. Remove leading/trailing underscores/dashes
    result = re.sub(r'^[_-]+', '', result)
    result = re.sub(r'[_-]+$', '', result)

    return result

def restore_files(limit=None, dry_run=True):
    """
    Restore files to original database names.

    Args:
        limit: Number of files to process (None = all)
        dry_run: If True, only print what would be done
    """
    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Get files where normalized name differs from database name
    query = """
        SELECT id, filename, filepath
        FROM files
        ORDER BY id
    """
    if limit:
        query += f" LIMIT {limit}"

    cur.execute(query)

    restored = 0
    errors = 0
    skipped = 0
    not_found = 0

    print(f"\n{'='*70}")
    print(f"RESTORING ORIGINAL FILENAMES FROM DATABASE")
    print(f"{'='*70}")
    print(f"Mode: {'DRY RUN' if dry_run else 'LIVE MODE'}")
    print()

    for file_id, db_filename, db_filepath in cur:
        # Calculate what the normalized filename would be
        normalized_filename = sanitize_strict(db_filename)

        # If they're the same, skip
        if normalized_filename == db_filename:
            skipped += 1
            continue

        # Get directory path
        db_path = Path(db_filepath)
        parent_dir = db_path.parent

        # What the normalized file would be called
        normalized_path = parent_dir / normalized_filename

        # Check if normalized file exists
        if not normalized_path.exists():
            # File might not have been renamed, check if original exists
            if db_path.exists():
                skipped += 1
                continue
            else:
                not_found += 1
                if not_found <= 10:  # Only print first 10
                    print(f"❌ Not found: {normalized_path}")
                continue

        # Restore to original name
        if dry_run:
            print(f"Would restore: {normalized_filename} → {db_filename}")
        else:
            try:
                # Handle case where original filename already exists
                if db_path.exists() and db_path != normalized_path:
                    print(f"⚠️  Original exists: {db_filename}, skipping")
                    skipped += 1
                    continue

                os.rename(normalized_path, db_path)
                restored += 1

                if restored % 10000 == 0:
                    print(f"Restored {restored:,} files...")
            except Exception as e:
                errors += 1
                if errors <= 10:
                    print(f"❌ Error restoring {normalized_filename}: {e}")

    cur.close()
    conn.close()

    print(f"\n{'='*70}")
    print(f"RESTORATION {'SIMULATION' if dry_run else 'COMPLETE'}")
    print(f"{'='*70}")
    print(f"Files restored:    {restored:,}")
    print(f"Files skipped:     {skipped:,}")
    print(f"Files not found:   {not_found:,}")
    print(f"Errors:            {errors:,}")
    print()

    return restored, skipped, not_found, errors

if __name__ == "__main__":
    # Test on small batch first
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        print("Testing on 100 files (dry run)...")
        restore_files(limit=100, dry_run=True)
    elif len(sys.argv) > 1 and sys.argv[1] == "--run":
        print("Running full restoration (LIVE MODE)...")
        response = input("Are you sure? This will rename ~650K files. Type 'yes' to confirm: ")
        if response.lower() == 'yes':
            restore_files(limit=None, dry_run=False)
        else:
            print("Cancelled.")
    else:
        print("Usage:")
        print("  --test    Test on 100 files (dry run)")
        print("  --run     Run full restoration (LIVE MODE)")

```

### `strict-sanitize-filenames.py` {#strict-sanitize-filenames-py}

- **Lines**: 182 (code: 117, comments: 33, blank: 32)
- **Pylint Score**: 9.30/10 ✅
- **Docstring Coverage**: 75.0%
- **Functions**: sanitize_filename_strict, sanitize_file, find_midi_files, main

#### Source Code

```python
#!/usr/bin/env python3
"""
Strict MIDI filename sanitizer
Rules:
- Only allowed: letters, numbers, underscores, hyphens, periods
- No consecutive special characters (__, --, .., -_, _., etc.)
- No spaces
"""

import os
import re
import sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from collections import defaultdict

def sanitize_filename_strict(filename):
    """
    Apply strict sanitization rules to filename.

    Rules:
    1. Keep only: a-z, A-Z, 0-9, _, -, .
    2. Replace all other chars with underscore
    3. Collapse consecutive special chars to single char
    4. Preserve .mid extension
    """
    # Split into name and extension
    if filename.endswith('.mid'):
        name = filename[:-4]
        ext = '.mid'
    elif filename.endswith('.midi'):
        name = filename[:-5]
        ext = '.mid'
    else:
        name = filename
        ext = ''

    # Step 1: Replace all non-allowed characters with underscore
    # Allowed: a-z A-Z 0-9 _ - .
    name = re.sub(r'[^a-zA-Z0-9_.\-]', '_', name)

    # Step 2: Collapse consecutive special characters
    # Replace __, --, .., or any mix of consecutive special chars with single underscore
    name = re.sub(r'[_.\-]{2,}', '_', name)

    # Step 3: Remove leading/trailing underscores
    name = name.strip('_')

    # If name is empty after sanitization, use default
    if not name:
        name = 'unnamed'

    return name + ext


def sanitize_file(file_path, dry_run=False):
    """Sanitize a single file."""
    try:
        parent = file_path.parent
        old_name = file_path.name
        new_name = sanitize_filename_strict(old_name)

        if old_name == new_name:
            return None, None  # No change needed

        new_path = parent / new_name

        # Handle conflicts by appending counter
        if new_path.exists() and new_path != file_path:
            base = new_name[:-4] if new_name.endswith('.mid') else new_name
            ext = '.mid' if new_name.endswith('.mid') else ''
            counter = 1
            while new_path.exists():
                new_name = f"{base}_{counter}{ext}"
                new_path = parent / new_name
                counter += 1

        if dry_run:
            return old_name, new_name

        # Rename the file
        file_path.rename(new_path)
        return old_name, new_name

    except Exception as e:
        print(f"Error processing {file_path}: {e}", file=sys.stderr)
        return None, None


def find_midi_files(directory):
    """Find all MIDI files recursively."""
    midi_files = []
    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.lower().endswith(('.mid', '.midi')):
                midi_files.append(Path(root) / file)
    return midi_files


def main():
    if len(sys.argv) < 2:
        print("Usage: ./strict-sanitize-filenames.py <directory> [--dry-run] [workers]")
        print("Example: ./strict-sanitize-filenames.py /path/to/midi --dry-run 64")
        sys.exit(1)

    directory = Path(sys.argv[1])
    dry_run = '--dry-run' in sys.argv
    workers = 64

    # Check for worker count
    for arg in sys.argv[2:]:
        if arg.isdigit():
            workers = int(arg)

    if not directory.exists():
        print(f"Error: Directory {directory} does not exist")
        sys.exit(1)

    print("🧹 STRICT MIDI FILENAME SANITIZATION")
    print("═" * 80)
    print(f"📂 Directory: {directory}")
    print(f"⚙️  Workers: {workers}")
    print(f"🔍 Mode: {'DRY RUN (preview only)' if dry_run else 'LIVE (will rename files)'}")
    print()

    # Find all MIDI files
    print("📊 Scanning for MIDI files...")
    midi_files = find_midi_files(directory)
    print(f"✓ Found {len(midi_files):,} MIDI files")
    print()

    if dry_run:
        print("🔍 DRY RUN - Preview of changes:")
        print("-" * 80)

    # Process files in parallel
    renamed_count = 0
    errors = 0
    changes = []

    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(sanitize_file, f, dry_run): f for f in midi_files}

        for i, future in enumerate(as_completed(futures), 1):
            try:
                old_name, new_name = future.result()
                if old_name and new_name:
                    renamed_count += 1
                    changes.append((old_name, new_name))
                    if dry_run and renamed_count <= 50:  # Show first 50 in dry run
                        print(f"  {old_name}")
                        print(f"  → {new_name}")
                        print()

                # Progress indicator
                if i % 10000 == 0:
                    print(f"  Processed: {i:,} / {len(midi_files):,} ({i*100//len(midi_files)}%)")
            except Exception as e:
                errors += 1
                print(f"Error: {e}", file=sys.stderr)

    # Summary
    print()
    print("═" * 80)
    if dry_run:
        print("✅ DRY RUN COMPLETE")
        print(f"   Files that would be renamed: {renamed_count:,}")
        if renamed_count > 50:
            print(f"   (showing first 50 of {renamed_count:,} changes)")
        print()
        print("Run without --dry-run to apply changes:")
        print(f"  ./strict-sanitize-filenames.py {directory} {workers}")
    else:
        print("✅ SANITIZATION COMPLETE")
        print(f"   Files renamed: {renamed_count:,}")
        print(f"   Files unchanged: {len(midi_files) - renamed_count:,}")
        print(f"   Errors: {errors}")
    print("═" * 80)


if __name__ == "__main__":
    main()

```

### `test-midi-files/Cargo.toml` {#test-midi-files-cargo-toml}

- **Lines**: 15 (code: 13, comments: 0, blank: 2)

#### Source Code

```toml
[package]
name = "test-midi-files"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "test-midi-files"
path = "src/main.rs"

[dependencies]
midi-library-shared = { path = "../../shared/rust" }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
walkdir = "2.4"

```

### `test-midi-files/src/main.rs` {#test-midi-files-src-main-rs}

- **Lines**: 268 (code: 234, comments: 0, blank: 34)

#### Source Code

```rust
use anyhow::{Context, Result};
use midi_library_shared::core::analysis::detect_bpm;
use midi_library_shared::core::midi::{parse_midi_file, Event, MidiFile};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

// Helper function to count notes in a MIDI file
fn count_notes(midi_file: &MidiFile) -> usize {
    midi_file
        .tracks
        .iter()
        .flat_map(|track| &track.events)
        .filter(|event| matches!(event.event, Event::NoteOn { .. }))
        .count()
}

// Helper function to get total duration in ticks
fn get_duration_ticks(midi_file: &MidiFile) -> u32 {
    midi_file
        .tracks
        .iter()
        .flat_map(|track| &track.events)
        .map(|event| event.delta_ticks)
        .sum()
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    file_path: String,
    file_name: String,
    success: bool,
    error: Option<String>,
    file_size: u64,
    parse_time_ms: u128,
    bpm: Option<f64>,
    key: Option<String>,
    duration_ms: Option<u64>,
    track_count: Option<usize>,
    note_count: Option<usize>,
}

fn analyze_file(file_path: &Path) -> Result<TestResult> {
    let start = Instant::now();
    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();

    println!("\n📄 Analyzing: {}", file_name);

    // Read file
    let data = fs::read(file_path).context("Failed to read file")?;
    let file_size = data.len() as u64;
    println!("  📦 Size: {} bytes", file_size);

    // Parse MIDI
    let midi_file = parse_midi_file(&data).context("Failed to parse MIDI")?;
    println!("  ✅ Parse successful");

    // Get basic info
    let track_count = midi_file.tracks.len();
    let note_count = count_notes(&midi_file);
    let duration_ticks = get_duration_ticks(&midi_file);

    println!("  🎵 Tracks: {}", track_count);
    println!("  🎹 Notes: {}", note_count);

    // Detect BPM
    let bpm_result = detect_bpm(&midi_file);
    let bpm = if bpm_result.confidence > 0.5 {
        Some(bpm_result.bpm)
    } else {
        None
    };

    if let Some(bpm_value) = bpm {
        println!(
            "  ⏱️  BPM: {:.1} (confidence: {:.1}%)",
            bpm_value,
            bpm_result.confidence * 100.0
        );
    } else {
        println!("  ⚠️  BPM: Not detected");
    }

    // Detect key using the shared library implementation
    use midi_library_shared::core::analysis::key_detector::detect_key;

    let key = detect_key(&midi_file);
    if let Some(ref key_str) = key {
        println!("  🎹 Key: {}", key_str);
    } else {
        println!("  ⚠️  Key: Unable to detect (low confidence or insufficient notes)");
    }

    let parse_time = start.elapsed();
    println!("  ⏱️  Processing time: {}ms", parse_time.as_millis());

    Ok(TestResult {
        file_path: file_path.to_string_lossy().to_string(),
        file_name,
        success: true,
        error: None,
        file_size,
        parse_time_ms: parse_time.as_millis(),
        bpm,
        key,
        duration_ms: Some((duration_ticks as u64 * 500) / 1000), // Rough estimate
        track_count: Some(track_count),
        note_count: Some(note_count),
    })
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <midi_file1> [midi_file2] ...", args[0]);
        eprintln!("       {} <directory>", args[0]);
        std::process::exit(1);
    }

    let mut results = Vec::new();
    let mut total_files = 0;
    let mut successful = 0;
    let mut failed = 0;
    let mut bpm_detected = 0;
    let mut key_detected = 0;
    let mut total_time_ms = 0u128;

    println!("🎵 MIDI Pipeline Real-World Testing");
    println!("====================================\n");

    // Process each file/directory argument
    for arg in &args[1..] {
        let path = Path::new(arg);

        let files: Vec<_> = if path.is_dir() {
            // Find all MIDI files in directory
            walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.eq_ignore_ascii_case("mid") || s.eq_ignore_ascii_case("midi"))
                        .unwrap_or(false)
                })
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            vec![path.to_path_buf()]
        };

        for file_path in files {
            total_files += 1;

            match analyze_file(&file_path) {
                Ok(result) => {
                    successful += 1;
                    total_time_ms += result.parse_time_ms;

                    if result.bpm.is_some() {
                        bpm_detected += 1;
                    }
                    if result.key.is_some() {
                        key_detected += 1;
                    }

                    results.push(result);
                },
                Err(e) => {
                    failed += 1;
                    println!("\n❌ Error: {}", e);

                    results.push(TestResult {
                        file_path: file_path.to_string_lossy().to_string(),
                        file_name: file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        success: false,
                        error: Some(e.to_string()),
                        file_size: 0,
                        parse_time_ms: 0,
                        bpm: None,
                        key: None,
                        duration_ms: None,
                        track_count: None,
                        note_count: None,
                    });
                },
            }
        }
    }

    // Print summary
    println!("\n");
    println!("📊 SUMMARY");
    println!("====================================");
    println!("✅ Files tested: {}", total_files);
    println!(
        "✅ Successful parses: {} ({:.1}%)",
        successful,
        (successful as f64 / total_files as f64) * 100.0
    );
    println!(
        "❌ Failed parses: {} ({:.1}%)",
        failed,
        (failed as f64 / total_files as f64) * 100.0
    );
    println!(
        "⏱️  BPM detection rate: {}/{} ({:.1}%)",
        bpm_detected,
        total_files,
        (bpm_detected as f64 / total_files as f64) * 100.0
    );
    println!(
        "🎼 Key detection rate: {}/{} ({:.1}%)",
        key_detected,
        total_files,
        (key_detected as f64 / total_files as f64) * 100.0
    );

    if total_files > 0 {
        let avg_time = total_time_ms / total_files as u128;
        println!("⏱️  Average processing time: {}ms/file", avg_time);
        println!("⏱️  Total processing time: {}ms", total_time_ms);
    }

    println!("\n");

    // Production readiness assessment
    println!("🎯 PRODUCTION READINESS ASSESSMENT");
    println!("====================================");

    let success_rate = (successful as f64 / total_files as f64) * 100.0;
    if success_rate >= 95.0 {
        println!("✅ Parse success rate: EXCELLENT ({:.1}%)", success_rate);
    } else if success_rate >= 85.0 {
        println!("⚠️  Parse success rate: ACCEPTABLE ({:.1}%)", success_rate);
    } else {
        println!(
            "❌ Parse success rate: NEEDS IMPROVEMENT ({:.1}%)",
            success_rate
        );
    }

    if total_files > 0 {
        let avg_time = total_time_ms / total_files as u128;
        if avg_time <= 100 {
            println!("✅ Performance: EXCELLENT ({}ms avg)", avg_time);
        } else if avg_time <= 500 {
            println!("✅ Performance: GOOD ({}ms avg)", avg_time);
        } else {
            println!("⚠️  Performance: NEEDS OPTIMIZATION ({}ms avg)", avg_time);
        }
    }

    // Write JSON results
    let results_json = serde_json::to_string_pretty(&results)?;
    fs::write("/tmp/midi_test_results.json", results_json)?;
    println!("\n📄 Detailed results written to: /tmp/midi_test_results.json");

    Ok(())
}

```

### `test-normalization-sample.py` {#test-normalization-sample-py}

- **Lines**: 167 (code: 99, comments: 36, blank: 32)
- **Pylint Score**: 8.79/10 ✅
- **Docstring Coverage**: 50.0%
- **Functions**: test_on_files_needing_normalization, sanitize_strict

#### Source Code

```python
#!/usr/bin/env python3
"""
Test normalization on a sample of files that actually need it
"""

import os
import sys

# Add the scripts directory to Python path
scripts_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, scripts_dir)

# Try to import the normalization module, with helpful error if it fails
try:
    from normalize_files_and_database import normalize_files_and_database
except ImportError as e:
    # Module might be named with hyphens on disk
    print(f"Note: Could not import normalize_files_and_database: {e}")
    print("This script may work without it - using inline implementation.")
    normalize_files_and_database = None

import psycopg2
import tempfile

DB_URL = "postgresql://midiuser:145278963@localhost:5433/midi_library"

def test_on_files_needing_normalization(count=100, dry_run=True):
    """Test on files that actually need normalization"""

    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Create temp table with files that need normalization
    cur.execute("""
        CREATE TEMP TABLE test_files AS
        SELECT id
        FROM files
        WHERE filename LIKE '% %'
           OR filename ~ '\\.MID$'
           OR filename ~ '\\.MIDI$'
           OR filename ~ '\\.Mid$'
        LIMIT %s
    """, (count,))

    # Get IDs for testing
    cur.execute("SELECT id FROM test_files")
    test_ids = [row[0] for row in cur.fetchall()]

    print(f"Testing on {len(test_ids)} files that need normalization")
    print()

    cur.close()
    conn.close()

    # Now run normalization on just these files
    conn = psycopg2.connect(DB_URL)
    cur = conn.cursor()

    # Process each test file
    from pathlib import Path
    import re
    from collections import defaultdict

    def sanitize_strict(filename):
        result = filename
        result = result.replace(' ', '_')
        result = re.sub(r'[^a-zA-Z0-9_\-.]+', '', result)
        result = re.sub(r'\.(mid|MID|MIDI|midi|Mid|MiD)$', '.mid', result, flags=re.IGNORECASE)
        result = re.sub(r'_+', '_', result)
        result = re.sub(r'-+', '-', result)
        result = re.sub(r'^[_-]+', '', result)
        result = re.sub(r'[_-]+$', '', result)
        return result

    stats = {'normalized': 0, 'skipped': 0, 'errors': 0}
    dir_filenames = defaultdict(set)

    cur.execute("""
        SELECT id, filename, filepath
        FROM files
        WHERE id = ANY(%s)
        ORDER BY id
    """, (test_ids,))

    print(f"{'='*70}")
    print(f"NORMALIZATION TEST - {count} FILES")
    print(f"Mode: {'DRY RUN' if dry_run else 'LIVE MODE'}")
    print(f"{'='*70}\n")

    for idx, (file_id, db_filename, db_filepath) in enumerate(cur.fetchall(), 1):
        normalized_filename = sanitize_strict(db_filename)

        if normalized_filename == db_filename:
            stats['skipped'] += 1
            continue

        db_path = Path(db_filepath)
        parent_dir = db_path.parent
        new_path = parent_dir / normalized_filename

        # Handle duplicates
        final_filename = normalized_filename
        final_path = new_path
        counter = 1

        while (final_path.exists() and final_path != db_path) or \
              final_filename in dir_filenames[str(parent_dir)]:
            stem = Path(normalized_filename).stem
            ext = Path(normalized_filename).suffix
            final_filename = f"{stem}_{counter}{ext}"
            final_path = parent_dir / final_filename
            counter += 1

        dir_filenames[str(parent_dir)].add(final_filename)

        if idx <= 20:  # Print first 20
            print(f"{idx}. {db_filename}")
            print(f"   → {final_filename}")
            if counter > 1:
                print(f"   (duplicate, added _{counter-1})")

        if not dry_run:
            try:
                if not db_path.exists():
                    print(f"   ❌ File not found: {db_path}")
                    stats['errors'] += 1
                    continue

                # Rename file
                os.rename(db_path, final_path)

                # Update database
                cur.execute("""
                    UPDATE files
                    SET filename = %s, filepath = %s, updated_at = NOW()
                    WHERE id = %s
                """, (final_filename, str(final_path), file_id))

                stats['normalized'] += 1
            except Exception as e:
                print(f"   ❌ Error: {e}")
                stats['errors'] += 1
                conn.rollback()
        else:
            stats['normalized'] += 1

    if not dry_run:
        conn.commit()

    cur.close()
    conn.close()

    print(f"\n{'='*70}")
    print(f"TEST {'SIMULATION' if dry_run else 'COMPLETE'}")
    print(f"{'='*70}")
    print(f"Files normalized: {stats['normalized']}")
    print(f"Files skipped:    {stats['skipped']}")
    print(f"Errors:           {stats['errors']}")
    print()

if __name__ == "__main__":
    if "--live" in sys.argv:
        response = input("Run on 100 files in LIVE MODE? Type 'yes': ")
        if response.lower() == 'yes':
            test_on_files_needing_normalization(100, dry_run=False)
    else:
        test_on_files_needing_normalization(100, dry_run=True)

```
