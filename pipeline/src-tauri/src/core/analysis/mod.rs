   /// Analysis modules for MIDI file processing

pub mod auto_tagger;
pub mod bpm_detector;
pub mod key_detector;
pub mod key_profiles;
pub mod drum_analyzer;

// Re-export main types
pub use auto_tagger::{AutoTagger, Tag};
pub use bpm_detector::{detect_bpm, BpmDetectionMethod, BpmDetectionResult, BpmMetadata};
pub use key_detector::{detect_key, KeyDetectionResult, ScaleType};
pub use drum_analyzer::{
    analyze_drum_midi, has_drum_channel, extract_drum_notes, note_to_drum_type,
    detect_cymbal_types, extract_time_signature_from_meta, detect_techniques,
    extract_time_signature_from_path, extract_bpm_from_filename, extract_pattern_type,
    extract_rhythmic_feel, extract_song_structure, generate_drum_tags,
    DrumAnalysis, DrumNote, PatternType, RhythmicFeel, DrumTechnique,
    TimeSignature, CymbalType, SongStructure,
};

// Test modules
#[cfg(test)]
mod tests;
