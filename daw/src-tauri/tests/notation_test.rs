//! Notation/Score rendering tests

use midi_software_center_daw::notation::{
    Clef, KeySignature, Measure, NotationNote, QuantizeLevel, Score, ScoreRenderer, Staff,
    TimeSignature,
};

#[test]
fn test_renderer_creation() {
    let _renderer = ScoreRenderer::new(480);
    // Just verify it creates without panic
}

#[test]
fn test_time_signature_default() {
    let ts = TimeSignature::default();
    assert_eq!(ts.numerator, 4);
    assert_eq!(ts.denominator, 4);
}

#[test]
fn test_key_signature_default() {
    let ks = KeySignature::default();
    assert_eq!(ks.sharps, 0);
    assert!(!ks.minor);
    assert_eq!(ks.to_key_name(), "C");
}

#[test]
fn test_key_signature_names() {
    // Test major keys
    assert_eq!(KeySignature { sharps: 0, minor: false }.to_key_name(), "C");
    assert_eq!(KeySignature { sharps: 1, minor: false }.to_key_name(), "G");
    assert_eq!(KeySignature { sharps: 2, minor: false }.to_key_name(), "D");
    assert_eq!(KeySignature { sharps: -1, minor: false }.to_key_name(), "F");
    assert_eq!(
        KeySignature { sharps: -2, minor: false }.to_key_name(),
        "Bb"
    );

    // Test minor keys
    assert_eq!(KeySignature { sharps: 0, minor: true }.to_key_name(), "Am");
    assert_eq!(KeySignature { sharps: 1, minor: true }.to_key_name(), "Em");
}

#[test]
fn test_quantize_level_ticks() {
    let ppq = 480;
    assert_eq!(QuantizeLevel::Whole.ticks(ppq), 1920); // 4 quarters
    assert_eq!(QuantizeLevel::Half.ticks(ppq), 960); // 2 quarters
    assert_eq!(QuantizeLevel::Quarter.ticks(ppq), 480); // 1 quarter
    assert_eq!(QuantizeLevel::Eighth.ticks(ppq), 240); // 1/2 quarter
    assert_eq!(QuantizeLevel::Sixteenth.ticks(ppq), 120); // 1/4 quarter
    assert_eq!(QuantizeLevel::ThirtySecond.ticks(ppq), 60); // 1/8 quarter
}

#[test]
fn test_quantize_triplets() {
    let ppq = 480;
    assert_eq!(QuantizeLevel::Triplet8th.ticks(ppq), 160); // ppq / 3
    assert_eq!(QuantizeLevel::Triplet16th.ticks(ppq), 80); // ppq / 6
}

#[test]
fn test_empty_score_render() {
    let renderer = ScoreRenderer::new(480);
    let score = Score {
        title: "Empty".to_string(),
        composer: String::new(),
        ppq: 480,
        staves: vec![
            Staff { clef: Clef::Treble, measures: vec![] },
            Staff { clef: Clef::Bass, measures: vec![] },
        ],
        key_signature: KeySignature::default(),
        time_signature: TimeSignature::default(),
        tempo: 120.0,
    };

    let svg = renderer.render_svg(&score, 800, 400);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn test_single_note_render() {
    let renderer = ScoreRenderer::new(480);
    let note = NotationNote {
        pitch: 60, // Middle C
        start_tick: 0,
        duration_ticks: 480,
        velocity: 100,
        voice: 0,
        tied_from: false,
        tied_to: false,
    };

    let score = Score {
        title: "Single Note".to_string(),
        composer: String::new(),
        ppq: 480,
        staves: vec![Staff {
            clef: Clef::Treble,
            measures: vec![Measure {
                number: 1,
                notes: vec![note],
                rests: vec![],
                time_signature: Some(TimeSignature::default()),
                key_signature: Some(KeySignature::default()),
                tempo: Some(120.0),
            }],
        }],
        key_signature: KeySignature::default(),
        time_signature: TimeSignature::default(),
        tempo: 120.0,
    };

    let svg = renderer.render_svg(&score, 800, 400);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("ellipse")); // Note head
    assert!(svg.contains("</svg>"));
}

#[test]
fn test_clef_types() {
    assert_eq!(Clef::Treble, Clef::Treble);
    assert_ne!(Clef::Treble, Clef::Bass);
    assert_ne!(Clef::Bass, Clef::Alto);
}

#[test]
fn test_musicxml_export() {
    let renderer = ScoreRenderer::new(480);
    let score = Score {
        title: "Test Score".to_string(),
        composer: String::new(),
        ppq: 480,
        staves: vec![Staff { clef: Clef::Treble, measures: vec![] }],
        key_signature: KeySignature::default(),
        time_signature: TimeSignature::default(),
        tempo: 120.0,
    };

    let xml = renderer.to_musicxml(&score);
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("score-partwise"));
    assert!(xml.contains("Test Score"));
}

#[test]
fn test_set_quantize() {
    let mut renderer = ScoreRenderer::new(480);
    renderer.set_quantize(QuantizeLevel::Eighth);
    // Just verify it works without panic
}

#[test]
fn test_notation_note_creation() {
    let note = NotationNote {
        pitch: 72, // C5
        start_tick: 0,
        duration_ticks: 480,
        velocity: 127,
        voice: 0,
        tied_from: true,
        tied_to: false,
    };

    assert_eq!(note.pitch, 72);
    assert_eq!(note.velocity, 127);
    assert!(note.tied_from);
    assert!(!note.tied_to);
}
