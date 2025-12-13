/// Analysis modules for MIDI file processing
// TODO: Fix arena_midi lifetime issues before enabling
// pub mod arena_midi;
pub mod auto_tagger;
pub mod bpm_detector;
pub mod chord_analyzer;
pub mod drum_analyzer;
pub mod filename_metadata;
pub mod key_detector;
pub mod key_profiles;
pub mod ml_features;
pub mod optimized_analyzer;
pub mod simd_bpm;

// Re-export main types
// pub use arena_midi::{ArenaEvent, ArenaMidiFile, ArenaParser, ArenaTimedEvent, ArenaTrack};
pub use auto_tagger::{AutoTagger, Tag};
pub use bpm_detector::{
    detect_bpm, detect_bpm_hybrid, detect_bpm_with_onsets, BpmDetectionMethod, BpmDetectionResult,
    BpmMetadata,
};
pub use chord_analyzer::{analyze_chords, ChordAnalysis};
pub use drum_analyzer::{
    analyze_drum_midi, detect_cymbal_types, detect_techniques, extract_drum_notes,
    extract_time_signature_from_meta, extract_time_signature_from_path, generate_drum_tags,
    has_drum_channel, note_to_drum_type, CymbalType, DrumAnalysis, DrumNote, DrumTechnique,
    PatternType, RhythmicFeel, SongStructure, TimeSignature,
};
pub use filename_metadata::{
    classify_leading_number, extract_genres_from_filename, extract_key_from_filename,
    extract_leading_number, extract_structure_tags, normalize_key_signature, validate_bpm,
    validate_bpm_for_genre, validate_key_signature, FilenameMetadata, KeyValidationResult,
    NumberType,
};
pub use key_detector::{detect_key, KeyDetectionResult, ScaleType};
pub use ml_features::{
    HeuristicClassifier, MidiFeatures, MlClassification, MlTagger, GENRE_LABELS, MODEL_INPUT_SIZE,
    MOOD_LABELS,
};
pub use simd_bpm::{
    batch_detect_onsets_simd, detect_bpm_from_onsets, detect_onsets_simd_vectorized,
    extract_onsets_simd, Onset, OnsetBpmResult,
};

// Test modules
#[cfg(test)]
mod tests;
