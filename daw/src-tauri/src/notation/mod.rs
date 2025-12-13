#![allow(dead_code)]

// =============================================================================
// SCORE RENDERER & MUSICXML EXPORT
// =============================================================================
// Render MIDI as traditional music notation and export to MusicXML.
//
// CLAUDE CODE INSTRUCTIONS:
// 1. Location: daw/src-tauri/src/notation/mod.rs
// 2. Uses SVG for rendering notation
// 3. Integrates with existing MIDI parsing
//
// FEATURES:
// - MIDI to notation conversion
// - SVG rendering of score
// - MusicXML export for Finale/Sibelius
// - Quantization options
// - Key/time signature detection
// - Multi-voice support
// =============================================================================

use midly::Smf;
use std::collections::HashMap;

/// Note representation for notation
#[derive(Debug, Clone)]
pub struct NotationNote {
    pub pitch: u8,
    pub start_tick: u64,
    pub duration_ticks: u64,
    pub velocity: u8,
    pub voice: u8,
    pub tied_from: bool,
    pub tied_to: bool,
}

/// Rest representation
#[derive(Debug, Clone)]
pub struct NotationRest {
    pub start_tick: u64,
    pub duration_ticks: u64,
    pub voice: u8,
}

/// Time signature
#[derive(Debug, Clone, Copy)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self { numerator: 4, denominator: 4 }
    }
}

/// Key signature
#[derive(Debug, Clone, Copy, Default)]
pub struct KeySignature {
    pub sharps: i8, // Negative for flats
    pub minor: bool,
}

impl KeySignature {
    pub fn to_key_name(self) -> &'static str {
        match (self.sharps, self.minor) {
            (0, false) => "C",
            (0, true) => "Am",
            (1, false) => "G",
            (1, true) => "Em",
            (2, false) => "D",
            (2, true) => "Bm",
            (3, false) => "A",
            (3, true) => "F#m",
            (4, false) => "E",
            (4, true) => "C#m",
            (5, false) => "B",
            (5, true) => "G#m",
            (-1, false) => "F",
            (-1, true) => "Dm",
            (-2, false) => "Bb",
            (-2, true) => "Gm",
            (-3, false) => "Eb",
            (-3, true) => "Cm",
            (-4, false) => "Ab",
            (-4, true) => "Fm",
            (-5, false) => "Db",
            (-5, true) => "Bbm",
            _ => "C",
        }
    }
}

/// Measure containing notes and rests
#[derive(Debug, Clone)]
pub struct Measure {
    pub number: u32,
    pub notes: Vec<NotationNote>,
    pub rests: Vec<NotationRest>,
    pub time_signature: Option<TimeSignature>,
    pub key_signature: Option<KeySignature>,
    pub tempo: Option<f64>,
}

/// Staff (treble, bass, etc.)
#[derive(Debug, Clone)]
pub struct Staff {
    pub clef: Clef,
    pub measures: Vec<Measure>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Clef {
    Treble,
    Bass,
    Alto,
    Tenor,
    Percussion,
}

/// Complete score
#[derive(Debug, Clone)]
pub struct Score {
    pub title: String,
    pub composer: String,
    pub ppq: u32,
    pub staves: Vec<Staff>,
    pub key_signature: KeySignature,
    pub time_signature: TimeSignature,
    pub tempo: f64,
}

/// Quantization level
#[derive(Debug, Clone, Copy)]
pub enum QuantizeLevel {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    Triplet8th,
    Triplet16th,
}

impl QuantizeLevel {
    pub fn ticks(&self, ppq: u32) -> u64 {
        match self {
            QuantizeLevel::Whole => ppq as u64 * 4,
            QuantizeLevel::Half => ppq as u64 * 2,
            QuantizeLevel::Quarter => ppq as u64,
            QuantizeLevel::Eighth => ppq as u64 / 2,
            QuantizeLevel::Sixteenth => ppq as u64 / 4,
            QuantizeLevel::ThirtySecond => ppq as u64 / 8,
            QuantizeLevel::Triplet8th => ppq as u64 / 3,
            QuantizeLevel::Triplet16th => ppq as u64 / 6,
        }
    }
}

/// Score renderer
pub struct ScoreRenderer {
    #[allow(dead_code)]
    ppq: u32,
    quantize_level: QuantizeLevel,
}

impl ScoreRenderer {
    pub fn new(ppq: u32) -> Self {
        Self { ppq, quantize_level: QuantizeLevel::Sixteenth }
    }

    /// Set quantization level
    pub fn set_quantize(&mut self, level: QuantizeLevel) {
        self.quantize_level = level;
    }

    /// Convert MIDI to score
    pub fn midi_to_score(&self, midi: &Smf, title: &str) -> Score {
        let ppq = match midi.header.timing {
            midly::Timing::Metrical(tpq) => tpq.as_int() as u32,
            _ => 480,
        };

        // Extract notes from all tracks
        let mut all_notes: Vec<(u8, NotationNote)> = Vec::new();
        let mut tempo = 120.0;
        let mut time_sig = TimeSignature::default();
        let mut key_sig = KeySignature::default();

        for track in &midi.tracks {
            let mut current_tick: u64 = 0;
            let mut active_notes: HashMap<(u8, u8), (u64, u8)> = HashMap::new(); // (channel, note) -> (start, velocity)

            for event in track {
                current_tick += event.delta.as_int() as u64;

                match event.kind {
                    midly::TrackEventKind::Midi { channel, message } => {
                        let ch = channel.as_int();
                        match message {
                            midly::MidiMessage::NoteOn { key, vel } => {
                                if vel.as_int() > 0 {
                                    active_notes
                                        .insert((ch, key.as_int()), (current_tick, vel.as_int()));
                                } else {
                                    // Note off
                                    if let Some((start, velocity)) =
                                        active_notes.remove(&(ch, key.as_int()))
                                    {
                                        let note = NotationNote {
                                            pitch: key.as_int(),
                                            start_tick: start,
                                            duration_ticks: current_tick - start,
                                            velocity,
                                            voice: ch,
                                            tied_from: false,
                                            tied_to: false,
                                        };
                                        all_notes.push((ch, note));
                                    }
                                }
                            },
                            midly::MidiMessage::NoteOff { key, .. } => {
                                if let Some((start, velocity)) =
                                    active_notes.remove(&(ch, key.as_int()))
                                {
                                    let note = NotationNote {
                                        pitch: key.as_int(),
                                        start_tick: start,
                                        duration_ticks: current_tick - start,
                                        velocity,
                                        voice: ch,
                                        tied_from: false,
                                        tied_to: false,
                                    };
                                    all_notes.push((ch, note));
                                }
                            },
                            _ => {},
                        }
                    },
                    midly::TrackEventKind::Meta(meta) => match meta {
                        midly::MetaMessage::Tempo(t) => {
                            tempo = 60_000_000.0 / t.as_int() as f64;
                        },
                        midly::MetaMessage::TimeSignature(num, denom, _, _) => {
                            time_sig = TimeSignature {
                                numerator: num,
                                denominator: 2u8.pow(denom as u32),
                            };
                        },
                        midly::MetaMessage::KeySignature(sharps, minor) => {
                            key_sig = KeySignature { sharps, minor };
                        },
                        _ => {},
                    },
                    _ => {},
                }
            }
        }

        // Quantize notes
        let quantize_ticks = self.quantize_level.ticks(ppq);
        for (_, note) in &mut all_notes {
            note.start_tick = (note.start_tick / quantize_ticks) * quantize_ticks;
            note.duration_ticks =
                ((note.duration_ticks + quantize_ticks / 2) / quantize_ticks) * quantize_ticks;
            note.duration_ticks = note.duration_ticks.max(quantize_ticks);
        }

        // Split into treble and bass staves
        let split_point = 60; // Middle C
        let treble_notes: Vec<NotationNote> = all_notes
            .iter()
            .filter(|(_, n)| n.pitch >= split_point)
            .map(|(_, n)| n.clone())
            .collect();
        let bass_notes: Vec<NotationNote> = all_notes
            .iter()
            .filter(|(_, n)| n.pitch < split_point)
            .map(|(_, n)| n.clone())
            .collect();

        // Create measures
        let ticks_per_measure =
            ppq as u64 * 4 * time_sig.numerator as u64 / time_sig.denominator as u64;

        let treble_measures =
            self.notes_to_measures(&treble_notes, ticks_per_measure, &time_sig, &key_sig, tempo);
        let bass_measures =
            self.notes_to_measures(&bass_notes, ticks_per_measure, &time_sig, &key_sig, tempo);

        Score {
            title: title.to_string(),
            composer: String::new(),
            ppq,
            staves: vec![
                Staff { clef: Clef::Treble, measures: treble_measures },
                Staff { clef: Clef::Bass, measures: bass_measures },
            ],
            key_signature: key_sig,
            time_signature: time_sig,
            tempo,
        }
    }

    fn notes_to_measures(
        &self,
        notes: &[NotationNote],
        ticks_per_measure: u64,
        time_sig: &TimeSignature,
        key_sig: &KeySignature,
        tempo: f64,
    ) -> Vec<Measure> {
        if notes.is_empty() {
            return vec![];
        }

        let max_tick = notes.iter().map(|n| n.start_tick + n.duration_ticks).max().unwrap_or(0);
        let num_measures = ((max_tick / ticks_per_measure) + 1) as u32;

        let mut measures = Vec::new();

        for m in 0..num_measures {
            let measure_start = m as u64 * ticks_per_measure;
            let measure_end = measure_start + ticks_per_measure;

            let measure_notes: Vec<NotationNote> = notes
                .iter()
                .filter(|n| n.start_tick >= measure_start && n.start_tick < measure_end)
                .cloned()
                .collect();

            measures.push(Measure {
                number: m + 1,
                notes: measure_notes,
                rests: vec![], // Simplified - would calculate rests
                time_signature: if m == 0 { Some(*time_sig) } else { None },
                key_signature: if m == 0 { Some(*key_sig) } else { None },
                tempo: if m == 0 { Some(tempo) } else { None },
            });
        }

        measures
    }

    /// Render score to SVG
    pub fn render_svg(&self, score: &Score, width: u32, height: u32) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            width, height, width, height
        );

        // Style - enhanced with velocity opacity, ties, rests, and voice colors
        svg.push_str(r#"
            <style>
                .staff-line { stroke: #000; stroke-width: 1; }
                .note-head { fill: #000; }
                .stem { stroke: #000; stroke-width: 1.5; }
                .beam { stroke: #000; stroke-width: 4; }
                .clef { font-family: 'Bravura', 'Opus', serif; font-size: 48px; }
                .time-sig { font-family: 'Times New Roman', serif; font-size: 24px; font-weight: bold; }
                .key-sig { font-family: 'Bravura', serif; font-size: 24px; }
                .key-name { font-family: 'Times New Roman', serif; font-size: 14px; font-style: italic; }
                .rest { fill: #000; }
                .tie { stroke: #000; stroke-width: 1.5; fill: none; }
                .voice-0 { fill: #000; }
                .voice-1 { fill: #0066cc; }
                .voice-2 { fill: #cc6600; }
                .voice-3 { fill: #660099; }
            </style>
        "#);

        let staff_spacing = 80.0;
        let line_spacing = 10.0;
        let margin_left = 80.0;
        let margin_top = 60.0;

        for (staff_idx, staff) in score.staves.iter().enumerate() {
            let staff_y = margin_top + staff_idx as f64 * staff_spacing * 2.0;

            // Draw staff lines
            for line in 0..5 {
                let y = staff_y + line as f64 * line_spacing;
                svg.push_str(&format!(
                    r#"<line class="staff-line" x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                    margin_left - 20.0,
                    y,
                    width as f64 - 20.0,
                    y
                ));
            }

            // Draw clef
            let clef_char = match staff.clef {
                Clef::Treble => "𝄞",
                Clef::Bass => "𝄢",
                Clef::Alto => "𝄡",
                _ => "𝄞",
            };
            svg.push_str(&format!(
                r#"<text class="clef" x="{}" y="{}">{}</text>"#,
                margin_left - 15.0,
                staff_y + 30.0,
                clef_char
            ));

            // Draw notes
            let note_spacing =
                (width as f64 - margin_left - 40.0) / score.staves[0].measures.len().max(1) as f64;

            for (measure_idx, measure) in staff.measures.iter().enumerate() {
                let measure_x = margin_left + measure_idx as f64 * note_spacing;

                // Draw time signature and key signature on first measure
                if measure_idx == 0 {
                    if let Some(ts) = &measure.time_signature {
                        svg.push_str(&format!(
                            r#"<text class="time-sig" x="{}" y="{}">{}</text>"#,
                            margin_left + 25.0,
                            staff_y + 15.0,
                            ts.numerator
                        ));
                        svg.push_str(&format!(
                            r#"<text class="time-sig" x="{}" y="{}">{}</text>"#,
                            margin_left + 25.0,
                            staff_y + 35.0,
                            ts.denominator
                        ));
                    }
                    // Display key signature name (using to_key_name())
                    if let Some(ks) = &measure.key_signature {
                        let key_name = ks.to_key_name();
                        svg.push_str(&format!(
                            r#"<text class="key-name" x="{}" y="{}">{}</text>"#,
                            margin_left + 50.0,
                            staff_y - 5.0,
                            key_name
                        ));
                        // Draw sharps/flats symbols
                        let sharp_positions = [0, 3, 6, 2, 5, 1, 4]; // F C G D A E B
                        let flat_positions = [6, 3, 7, 4, 1, 5, 2]; // B E A D G C F
                        if ks.sharps > 0 {
                            for (i, &pos) in
                                sharp_positions.iter().enumerate().take(ks.sharps.min(7) as usize)
                            {
                                let y_offset = pos as f64 * (line_spacing / 2.0);
                                svg.push_str(&format!(
                                    r#"<text class="key-sig" x="{}" y="{}">♯</text>"#,
                                    margin_left + 60.0 + i as f64 * 8.0,
                                    staff_y + y_offset
                                ));
                            }
                        } else if ks.sharps < 0 {
                            for (i, &pos) in
                                flat_positions.iter().enumerate().take((-ks.sharps).min(7) as usize)
                            {
                                let y_offset = pos as f64 * (line_spacing / 2.0);
                                svg.push_str(&format!(
                                    r#"<text class="key-sig" x="{}" y="{}">♭</text>"#,
                                    margin_left + 60.0 + i as f64 * 8.0,
                                    staff_y + y_offset
                                ));
                            }
                        }
                    }
                }

                // Draw bar line
                if measure_idx > 0 {
                    svg.push_str(&format!(
                        r#"<line class="staff-line" x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                        measure_x,
                        staff_y,
                        measure_x,
                        staff_y + 40.0
                    ));
                }

                // Draw rests in this measure (using NotationRest)
                for (rest_idx, rest) in measure.rests.iter().enumerate() {
                    let rest_x = measure_x + 20.0 + rest_idx as f64 * 20.0;
                    let rest_y = staff_y + 20.0; // Center on staff

                    // Quarter rest symbol (simplified path)
                    svg.push_str(&format!(
                        r#"<text class="rest voice-{}" x="{}" y="{}" font-size="24">𝄽</text>"#,
                        rest.voice.min(3),
                        rest_x,
                        rest_y
                    ));
                }

                // Draw notes in this measure
                for (note_idx, note) in measure.notes.iter().enumerate() {
                    let note_x = measure_x + 20.0 + note_idx as f64 * 20.0;
                    let note_y = self.pitch_to_y(note.pitch, staff_y, &staff.clef, line_spacing);

                    // Calculate opacity based on velocity (0.3 to 1.0 range)
                    let velocity_opacity = 0.3 + (note.velocity as f64 / 127.0) * 0.7;

                    // Voice-based color class (0-3)
                    let voice_class = format!("voice-{}", note.voice.min(3));

                    // Note head with velocity opacity and voice color
                    svg.push_str(&format!(
                        r#"<ellipse class="note-head {}" cx="{}" cy="{}" rx="6" ry="5" transform="rotate(-15 {} {})" style="opacity: {};"/>"#,
                        voice_class, note_x, note_y, note_x, note_y, velocity_opacity
                    ));

                    // Stem with matching opacity
                    let stem_up = note_y > staff_y + 20.0;
                    let stem_x = if stem_up { note_x + 5.5 } else { note_x - 5.5 };
                    let stem_y1 = note_y;
                    let stem_y2 = if stem_up {
                        note_y - 35.0
                    } else {
                        note_y + 35.0
                    };
                    svg.push_str(&format!(
                        r#"<line class="stem" x1="{}" y1="{}" x2="{}" y2="{}" style="opacity: {};"/>"#,
                        stem_x, stem_y1, stem_x, stem_y2, velocity_opacity
                    ));

                    // Tie mark if note is tied to next note
                    if note.tied_to {
                        let tie_x1 = note_x + 8.0;
                        let tie_x2 = note_x + 18.0;
                        let tie_y_offset = if stem_up { 5.0 } else { -5.0 };
                        let tie_curve = if stem_up { 8.0 } else { -8.0 };
                        svg.push_str(&format!(
                            r#"<path class="tie" d="M{},{} Q{},{} {},{}"/>"#,
                            tie_x1,
                            note_y + tie_y_offset,
                            (tie_x1 + tie_x2) / 2.0,
                            note_y + tie_y_offset + tie_curve,
                            tie_x2,
                            note_y + tie_y_offset
                        ));
                    }

                    // Ledger lines if needed
                    if note_y < staff_y - line_spacing / 2.0 {
                        let num_ledgers = ((staff_y - note_y) / line_spacing).ceil() as i32;
                        for i in 1..=num_ledgers {
                            let ledger_y = staff_y - i as f64 * line_spacing;
                            svg.push_str(&format!(
                                r#"<line class="staff-line" x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                                note_x - 10.0,
                                ledger_y,
                                note_x + 10.0,
                                ledger_y
                            ));
                        }
                    } else if note_y > staff_y + 40.0 + line_spacing / 2.0 {
                        let num_ledgers = ((note_y - staff_y - 40.0) / line_spacing).ceil() as i32;
                        for i in 1..=num_ledgers {
                            let ledger_y = staff_y + 40.0 + i as f64 * line_spacing;
                            svg.push_str(&format!(
                                r#"<line class="staff-line" x1="{}" y1="{}" x2="{}" y2="{}"/>"#,
                                note_x - 10.0,
                                ledger_y,
                                note_x + 10.0,
                                ledger_y
                            ));
                        }
                    }
                }
            }
        }

        svg.push_str("</svg>");
        svg
    }

    fn pitch_to_y(&self, pitch: u8, staff_y: f64, clef: &Clef, line_spacing: f64) -> f64 {
        // Calculate Y position based on pitch
        // Middle C (60) should be on the ledger line below treble staff
        let half_step = line_spacing / 2.0;

        let reference = match clef {
            Clef::Treble => 71, // B4 on middle line
            Clef::Bass => 50,   // D3 on middle line
            _ => 60,
        };

        // Calculate position in half steps from reference
        let pitch_class = pitch % 12;
        let octave = pitch / 12;
        let ref_class = reference % 12;
        let ref_octave = reference / 12;

        // Map to staff positions (C D E F G A B = 0 1 2 3 4 5 6)
        let note_map = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6]; // Chromatic to diatonic

        let pitch_position = note_map[pitch_class as usize] + (octave * 7);
        let ref_position = note_map[ref_class as usize] + (ref_octave * 7);

        let offset = ref_position - pitch_position;

        staff_y + 20.0 + offset as f64 * half_step
    }

    /// Export to MusicXML
    pub fn to_musicxml(&self, score: &Score) -> String {
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 4.0 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="4.0">
"#,
        );

        // Work info
        xml.push_str(&format!(
            r#"  <work>
    <work-title>{}</work-title>
  </work>
"#,
            score.title
        ));

        // Part list
        xml.push_str(
            r#"  <part-list>
"#,
        );
        for (i, staff) in score.staves.iter().enumerate() {
            let clef_name = match staff.clef {
                Clef::Treble => "Treble",
                Clef::Bass => "Bass",
                _ => "Part",
            };
            xml.push_str(&format!(
                r#"    <score-part id="P{}">
      <part-name>{}</part-name>
    </score-part>
"#,
                i + 1,
                clef_name
            ));
        }
        xml.push_str(
            r#"  </part-list>
"#,
        );

        // Parts
        for (i, staff) in score.staves.iter().enumerate() {
            xml.push_str(&format!(
                r#"  <part id="P{}">
"#,
                i + 1
            ));

            for measure in &staff.measures {
                xml.push_str(&format!(
                    r#"    <measure number="{}">
"#,
                    measure.number
                ));

                // Attributes on first measure
                if measure.number == 1 {
                    xml.push_str(
                        r#"      <attributes>
        <divisions>1</divisions>
"#,
                    );
                    if let Some(ts) = &measure.time_signature {
                        xml.push_str(&format!(
                            r#"        <time>
          <beats>{}</beats>
          <beat-type>{}</beat-type>
        </time>
"#,
                            ts.numerator, ts.denominator
                        ));
                    }

                    // Clef
                    let (sign, line) = match staff.clef {
                        Clef::Treble => ("G", 2),
                        Clef::Bass => ("F", 4),
                        Clef::Alto => ("C", 3),
                        _ => ("G", 2),
                    };
                    xml.push_str(&format!(
                        r#"        <clef>
          <sign>{}</sign>
          <line>{}</line>
        </clef>
"#,
                        sign, line
                    ));

                    xml.push_str(
                        r#"      </attributes>
"#,
                    );
                }

                // Notes
                for note in &measure.notes {
                    let (step, octave, alter) = self.midi_to_musicxml_pitch(note.pitch);
                    let duration = (note.duration_ticks / (score.ppq as u64 / 4)).max(1);
                    let note_type = self.duration_to_type(note.duration_ticks, score.ppq);

                    xml.push_str(
                        r#"      <note>
        <pitch>
"#,
                    );
                    xml.push_str(&format!(
                        r#"          <step>{}</step>
"#,
                        step
                    ));
                    if alter != 0 {
                        xml.push_str(&format!(
                            r#"          <alter>{}</alter>
"#,
                            alter
                        ));
                    }
                    xml.push_str(&format!(
                        r#"          <octave>{}</octave>
        </pitch>
        <duration>{}</duration>
        <type>{}</type>
      </note>
"#,
                        octave, duration, note_type
                    ));
                }

                xml.push_str(
                    r#"    </measure>
"#,
                );
            }

            xml.push_str(
                r#"  </part>
"#,
            );
        }

        xml.push_str(
            r#"</score-partwise>
"#,
        );
        xml
    }

    fn midi_to_musicxml_pitch(&self, midi: u8) -> (&'static str, i32, i32) {
        let octave = (midi / 12) as i32 - 1;
        let pitch_class = midi % 12;

        match pitch_class {
            0 => ("C", octave, 0),
            1 => ("C", octave, 1),
            2 => ("D", octave, 0),
            3 => ("E", octave, -1),
            4 => ("E", octave, 0),
            5 => ("F", octave, 0),
            6 => ("F", octave, 1),
            7 => ("G", octave, 0),
            8 => ("A", octave, -1),
            9 => ("A", octave, 0),
            10 => ("B", octave, -1),
            11 => ("B", octave, 0),
            _ => ("C", octave, 0),
        }
    }

    fn duration_to_type(&self, ticks: u64, ppq: u32) -> &'static str {
        let quarter = ppq as u64;

        if ticks >= quarter * 4 {
            "whole"
        } else if ticks >= quarter * 2 {
            "half"
        } else if ticks >= quarter {
            "quarter"
        } else if ticks >= quarter / 2 {
            "eighth"
        } else if ticks >= quarter / 4 {
            "16th"
        } else {
            "32nd"
        }
    }
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

use std::sync::Mutex;
use tauri::State;

pub struct ScoreRendererState(pub Mutex<ScoreRenderer>);

#[tauri::command]
pub fn render_score_svg(
    state: State<ScoreRendererState>,
    midi_bytes: Vec<u8>,
    title: String,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let renderer = state.0.lock().map_err(|e| e.to_string())?;
    let smf = Smf::parse(&midi_bytes).map_err(|e| e.to_string())?;
    let score = renderer.midi_to_score(&smf, &title);
    Ok(renderer.render_svg(&score, width, height))
}

#[tauri::command]
pub fn export_musicxml(
    state: State<ScoreRendererState>,
    midi_bytes: Vec<u8>,
    title: String,
) -> Result<String, String> {
    let renderer = state.0.lock().map_err(|e| e.to_string())?;
    let smf = Smf::parse(&midi_bytes).map_err(|e| e.to_string())?;
    let score = renderer.midi_to_score(&smf, &title);
    Ok(renderer.to_musicxml(&score))
}

#[tauri::command]
pub fn set_quantize_level(state: State<ScoreRendererState>, level: String) -> Result<(), String> {
    let mut renderer = state.0.lock().map_err(|e| e.to_string())?;
    let quantize = match level.to_lowercase().as_str() {
        "whole" => QuantizeLevel::Whole,
        "half" => QuantizeLevel::Half,
        "quarter" => QuantizeLevel::Quarter,
        "eighth" | "8th" => QuantizeLevel::Eighth,
        "sixteenth" | "16th" => QuantizeLevel::Sixteenth,
        "thirtysecond" | "32nd" => QuantizeLevel::ThirtySecond,
        "triplet8th" | "triplet-8th" => QuantizeLevel::Triplet8th,
        "triplet16th" | "triplet-16th" => QuantizeLevel::Triplet16th,
        _ => return Err(format!("Unknown quantize level: {}", level)),
    };
    renderer.set_quantize(quantize);
    Ok(())
}

#[tauri::command]
pub fn get_quantize_levels() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"id": "whole", "name": "Whole Note", "ticks_at_480": 1920}),
        serde_json::json!({"id": "half", "name": "Half Note", "ticks_at_480": 960}),
        serde_json::json!({"id": "quarter", "name": "Quarter Note", "ticks_at_480": 480}),
        serde_json::json!({"id": "eighth", "name": "Eighth Note", "ticks_at_480": 240}),
        serde_json::json!({"id": "sixteenth", "name": "Sixteenth Note", "ticks_at_480": 120}),
        serde_json::json!({"id": "thirtysecond", "name": "32nd Note", "ticks_at_480": 60}),
        serde_json::json!({"id": "triplet8th", "name": "Triplet 8th", "ticks_at_480": 160}),
        serde_json::json!({"id": "triplet16th", "name": "Triplet 16th", "ticks_at_480": 80}),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_signature() {
        let ks = KeySignature { sharps: 2, minor: false };
        assert_eq!(ks.to_key_name(), "D");
    }

    #[test]
    fn test_quantize_level() {
        let ppq = 480;
        assert_eq!(QuantizeLevel::Quarter.ticks(ppq), 480);
        assert_eq!(QuantizeLevel::Eighth.ticks(ppq), 240);
        assert_eq!(QuantizeLevel::Sixteenth.ticks(ppq), 120);
    }
}
