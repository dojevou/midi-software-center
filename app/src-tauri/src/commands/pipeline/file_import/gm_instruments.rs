//! General MIDI Level 1 instrument mapping
//!
//! Maps MIDI program numbers (0-127) to instrument category names.

/// Map MIDI General MIDI program number to instrument name
pub fn program_to_instrument_name(program: u8) -> Option<String> {
    let name = match program {
        // Piano (0-7)
        0..=7 => "Piano",
        // Chromatic Percussion (8-15)
        8..=15 => "Keys",
        // Organ (16-23)
        16..=23 => "Organ",
        // Guitar (24-31)
        24..=31 => "Guitar",
        // Bass (32-39)
        32..=39 => "Bass",
        // Strings (40-47)
        40..=47 => "Strings",
        // Ensemble (48-55)
        48..=55 => "Ensemble",
        // Brass (56-63)
        56..=63 => "Brass",
        // Reed (64-71)
        64..=71 => "Woodwind",
        // Pipe (72-79)
        72..=79 => "Flute",
        // Synth Lead (80-87)
        80..=87 => "Lead",
        // Synth Pad (88-95)
        88..=95 => "Pad",
        // Synth Effects (96-103)
        96..=103 => "FX",
        // Ethnic (104-111)
        104..=111 => "Ethnic",
        // Percussive (112-119)
        112..=119 => "Percussion",
        // Sound Effects (120-127)
        120..=127 => "FX",
        _ => return None,
    };
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piano_range() {
        for i in 0..=7 {
            assert_eq!(program_to_instrument_name(i), Some("Piano".to_string()));
        }
    }

    #[test]
    fn test_bass_range() {
        for i in 32..=39 {
            assert_eq!(program_to_instrument_name(i), Some("Bass".to_string()));
        }
    }

    #[test]
    fn test_all_programs_mapped() {
        for i in 0..=127 {
            assert!(program_to_instrument_name(i).is_some());
        }
    }
}
