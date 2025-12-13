#!/bin/bash
# MIDI Software Center - Complete Analysis Runner
# Executes all diagnostic scripts and outputs to analysis_results.txt

PROJECT_ROOT=~/projects/midi-software-center
OUTPUT_FILE="$PROJECT_ROOT/analysis_results.txt"

cd "$PROJECT_ROOT" || exit 1

{
echo "================================================================================"
echo "MIDI SOFTWARE CENTER - COMPLETE ANALYSIS REPORT"
echo "Generated: $(date)"
echo "Project: $PROJECT_ROOT"
echo "================================================================================"
echo ""

echo "--------------------------------------------------------------------------------"
echo "1. COMPILATION STATUS - Workspace Errors"
echo "--------------------------------------------------------------------------------"
cargo check --workspace 2>&1 | grep -E "^error" | head -50
echo ""

echo "--------------------------------------------------------------------------------"
echo "2. COMPILATION STATUS - DAW Package Errors"
echo "--------------------------------------------------------------------------------"
cargo check -p midi-software-center-daw --tests 2>&1 | grep -E "^error"
echo ""

echo "--------------------------------------------------------------------------------"
echo "3. TEST FAILURES - All Failures with Panic Info"
echo "--------------------------------------------------------------------------------"
cargo test --workspace --lib -- --test-threads=1 2>&1 | grep -E "(FAILED|panicked|assertion failed)" | head -50
echo ""

echo "--------------------------------------------------------------------------------"
echo "4. TEST FAILURES - Count"
echo "--------------------------------------------------------------------------------"
echo -n "Total FAILED tests: "
cargo test --workspace --lib -- --test-threads=1 2>&1 | grep -c "FAILED"
echo ""

echo "--------------------------------------------------------------------------------"
echo "5. AUTO-TAGGER - tag_names.contains assertions"
echo "--------------------------------------------------------------------------------"
grep -n "tag_names.contains" pipeline/src-tauri/src/core/analysis/auto_tagger.rs | head -20
echo ""

echo "--------------------------------------------------------------------------------"
echo "6. AUTO-TAGGER - Failing Lines Context"
echo "--------------------------------------------------------------------------------"
echo "Line 1069 (test_filename_underscore_splitting):"
sed -n '1065,1072p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1093 (test_filename_space_splitting):"
sed -n '1089,1096p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1106 (test_filename_dot_splitting):"
sed -n '1102,1109p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1116 (test_filename_mixed_separators):"
sed -n '1112,1119p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1178 (test_filename_generic_tags_alphanumeric):"
sed -n '1174,1181p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1249 (test_filename_no_matches):"
sed -n '1245,1252p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1287 (test_extract_from_path):"
sed -n '1283,1290p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1366 (test_path_instrument_category):"
sed -n '1362,1369p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1842 (test_integration_vengeance_style):"
sed -n '1838,1845p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1865 (test_integration_splice_style):"
sed -n '1861,1868p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""
echo "Line 1924 (test_integration_comprehensive_file):"
sed -n '1920,1927p' pipeline/src-tauri/src/core/analysis/auto_tagger.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "7. CHORD ANALYZER - test_rapid_chord_changes (line 408)"
echo "--------------------------------------------------------------------------------"
sed -n '400,415p' pipeline/src-tauri/src/core/analysis/tests/chord_analyzer_extended_test.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "8. WINDOW STATE - test_database_window_pagination (line 437)"
echo "--------------------------------------------------------------------------------"
sed -n '425,445p' pipeline/src-tauri/src/database/window_state.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "9. ANALYZE COMMAND - test_get_key_name (line 1748)"
echo "--------------------------------------------------------------------------------"
sed -n '1740,1760p' pipeline/src-tauri/src/commands/analyze.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "10. FILENAME METADATA - test_number_classification (line 550)"
echo "--------------------------------------------------------------------------------"
sed -n '540,560p' pipeline/src-tauri/src/core/analysis/filename_metadata.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "11. STRUCT DEFINITIONS - Track"
echo "--------------------------------------------------------------------------------"
grep -A 15 "pub struct Track {" daw/src-tauri/src/models/sequencer.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "12. STRUCT DEFINITIONS - SequencerState"
echo "--------------------------------------------------------------------------------"
grep -A 12 "pub struct SequencerState {" daw/src-tauri/src/models/sequencer.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "13. STRUCT DEFINITIONS - MidiDevice"
echo "--------------------------------------------------------------------------------"
grep -A 6 "pub struct MidiDevice {" daw/src-tauri/src/models/midi.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "14. STRUCT DEFINITIONS - MidiEvent"
echo "--------------------------------------------------------------------------------"
grep -A 20 "pub struct MidiEvent {" daw/src-tauri/src/models/midi.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "15. STRUCT DEFINITIONS - SearchFilters"
echo "--------------------------------------------------------------------------------"
grep -A 20 "pub struct SearchFilters {" daw/src-tauri/src/models/search.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "16. ERROR SUMMARY - Count by Type"
echo "--------------------------------------------------------------------------------"
cargo check --workspace 2>&1 | grep -E "^error" | sort | uniq -c | sort -rn
echo ""

echo "--------------------------------------------------------------------------------"
echo "17. ERROR SUMMARY - All Error Locations"
echo "--------------------------------------------------------------------------------"
cargo check --workspace 2>&1 | grep -B2 "^error" | grep -E "error|-->" | head -30
echo ""

echo "--------------------------------------------------------------------------------"
echo "18. MODELS_TEST.RS - Compilation Check"
echo "--------------------------------------------------------------------------------"
cargo check -p midi-software-center-daw --test models_test 2>&1 | grep -E "error|Finished|warning.*models_test"
echo ""

echo "--------------------------------------------------------------------------------"
echo "19. MISSING FIELD ERRORS - Count"
echo "--------------------------------------------------------------------------------"
echo -n "Missing field errors: "
cargo check --workspace 2>&1 | grep -c "missing field"
echo ""

echo "--------------------------------------------------------------------------------"
echo "20. LARGEST SOURCE FILES"
echo "--------------------------------------------------------------------------------"
find . -name "*.rs" -type f ! -path "./target/*" | xargs wc -l 2>/dev/null | sort -rn | head -15
echo ""

echo "--------------------------------------------------------------------------------"
echo "21. TEST FILE SIZES"
echo "--------------------------------------------------------------------------------"
find . -name "*_test.rs" -o -name "*_tests.rs" | xargs wc -l 2>/dev/null | sort -rn | head -10
echo ""

echo "--------------------------------------------------------------------------------"
echo "22. AUTO-TAGGER FUNCTION - extract_from_filename"
echo "--------------------------------------------------------------------------------"
grep -n "pub fn extract_from_filename" pipeline/src-tauri/src/core/analysis/auto_tagger.rs
grep -A 50 "pub fn extract_from_filename" pipeline/src-tauri/src/core/analysis/auto_tagger.rs | head -60
echo ""

echo "--------------------------------------------------------------------------------"
echo "23. AUTO-TAGGER FUNCTION - extract_from_path"
echo "--------------------------------------------------------------------------------"
grep -n "pub fn extract_from_path" pipeline/src-tauri/src/core/analysis/auto_tagger.rs
grep -A 30 "pub fn extract_from_path" pipeline/src-tauri/src/core/analysis/auto_tagger.rs | head -40
echo ""

echo "--------------------------------------------------------------------------------"
echo "24. CHORD ANALYZER - analyze_chords function"
echo "--------------------------------------------------------------------------------"
grep -n "pub fn analyze_chords" pipeline/src-tauri/src/core/analysis/chord_analyzer.rs 2>/dev/null || echo "Not found in chord_analyzer.rs"
grep -n "pub fn analyze_chords" pipeline/src-tauri/src/core/analysis/*.rs 2>/dev/null | head -5
echo ""

echo "--------------------------------------------------------------------------------"
echo "25. WINDOW STATE - previous_page function"
echo "--------------------------------------------------------------------------------"
grep -n "fn previous_page" pipeline/src-tauri/src/database/window_state.rs
grep -A 10 "fn previous_page" pipeline/src-tauri/src/database/window_state.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "26. ANALYZE COMMAND - get_key_name function"
echo "--------------------------------------------------------------------------------"
grep -n "fn get_key_name" pipeline/src-tauri/src/commands/analyze.rs
grep -A 20 "fn get_key_name" pipeline/src-tauri/src/commands/analyze.rs | head -25
echo ""

echo "--------------------------------------------------------------------------------"
echo "27. FILENAME METADATA - classify_leading_number function"
echo "--------------------------------------------------------------------------------"
grep -n "fn classify_leading_number" pipeline/src-tauri/src/core/analysis/filename_metadata.rs
grep -A 15 "fn classify_leading_number" pipeline/src-tauri/src/core/analysis/filename_metadata.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "28. FILENAME METADATA - NumberType enum"
echo "--------------------------------------------------------------------------------"
grep -B 2 -A 10 "enum NumberType" pipeline/src-tauri/src/core/analysis/filename_metadata.rs
echo ""

echo "--------------------------------------------------------------------------------"
echo "29. TOKIO TEST ANNOTATION CHECK"
echo "--------------------------------------------------------------------------------"
echo "Tests using #[test] that may need #[tokio::test]:"
grep -rn "PgPool::connect_lazy" --include="*.rs" | grep -v target | head -10
echo ""

echo "--------------------------------------------------------------------------------"
echo "30. FULL TEST RESULTS SUMMARY"
echo "--------------------------------------------------------------------------------"
cargo test --workspace --lib -- --test-threads=1 2>&1 | tail -20
echo ""

echo "================================================================================"
echo "END OF ANALYSIS REPORT"
echo "================================================================================"

} > "$OUTPUT_FILE" 2>&1

echo "Analysis complete. Results saved to: $OUTPUT_FILE"
echo "File size: $(wc -c < "$OUTPUT_FILE") bytes"
echo "Line count: $(wc -l < "$OUTPUT_FILE") lines"
