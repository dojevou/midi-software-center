//! General MIDI instrument information

/// Get instrument information from GM program number
pub fn get_instrument_info(program: u8) -> (String, String, String) {
    match program {
        0..=7 => (
            "Piano".to_string(),
            "Keyboard".to_string(),
            "Acoustic Piano".to_string(),
        ),
        8..=15 => (
            "Chromatic Percussion".to_string(),
            "Keyboard".to_string(),
            "Celesta/Glockenspiel".to_string(),
        ),
        16..=23 => (
            "Organ".to_string(),
            "Keyboard".to_string(),
            "Drawbar Organ".to_string(),
        ),
        24..=31 => (
            "Guitar".to_string(),
            "Strings".to_string(),
            "Acoustic Guitar".to_string(),
        ),
        32..=39 => (
            "Bass".to_string(),
            "Strings".to_string(),
            "Electric Bass".to_string(),
        ),
        40..=47 => (
            "Strings".to_string(),
            "Strings".to_string(),
            "Violin/Viola".to_string(),
        ),
        48..=55 => (
            "Ensemble".to_string(),
            "Ensemble".to_string(),
            "String Ensemble".to_string(),
        ),
        56..=63 => (
            "Brass".to_string(),
            "Brass".to_string(),
            "Trumpet/Trombone".to_string(),
        ),
        64..=71 => (
            "Reed".to_string(),
            "Reed".to_string(),
            "Saxophone".to_string(),
        ),
        72..=79 => (
            "Pipe".to_string(),
            "Pipe".to_string(),
            "Flute/Piccolo".to_string(),
        ),
        80..=87 => (
            "Synth Lead".to_string(),
            "Synth".to_string(),
            "Lead Synth".to_string(),
        ),
        88..=95 => (
            "Synth Pad".to_string(),
            "Synth".to_string(),
            "Pad Synth".to_string(),
        ),
        96..=103 => (
            "Synth Effects".to_string(),
            "Synth".to_string(),
            "FX Synth".to_string(),
        ),
        104..=111 => (
            "Ethnic".to_string(),
            "Ethnic".to_string(),
            "Sitar/Shamisen".to_string(),
        ),
        112..=119 => (
            "Percussive".to_string(),
            "Percussion".to_string(),
            "Timpani/Taiko".to_string(),
        ),
        120..=127 => (
            "Sound Effects".to_string(),
            "SFX".to_string(),
            "Sound Effect".to_string(),
        ),
        _ => (
            "Unknown".to_string(),
            "Unknown".to_string(),
            "Unknown".to_string(),
        ),
    }
}

/// Map MIDI General MIDI program number to instrument name
pub fn program_to_instrument_name(program: u8) -> Option<String> {
    match program {
        0..=7 => Some("Piano".to_string()),
        8..=15 => Some("Keys".to_string()),
        16..=23 => Some("Organ".to_string()),
        24..=31 => Some("Guitar".to_string()),
        32..=39 => Some("Bass".to_string()),
        40..=47 => Some("Strings".to_string()),
        48..=55 => Some("Ensemble".to_string()),
        56..=63 => Some("Brass".to_string()),
        64..=71 => Some("Woodwind".to_string()),
        72..=79 => Some("Flute".to_string()),
        80..=87 => Some("Lead".to_string()),
        88..=95 => Some("Pad".to_string()),
        96..=103 => Some("FX".to_string()),
        104..=111 => Some("Ethnic".to_string()),
        112..=119 => Some("Percussion".to_string()),
        120..=127 => Some("FX".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_to_instrument_name() {
        assert_eq!(program_to_instrument_name(0), Some("Piano".to_string()));
        assert_eq!(program_to_instrument_name(32), Some("Bass".to_string()));
        assert_eq!(program_to_instrument_name(80), Some("Lead".to_string()));
    }

    #[test]
    fn test_get_instrument_info() {
        let (name, family, inst_type) = get_instrument_info(0);
        assert_eq!(name, "Piano");
        assert_eq!(family, "Keyboard");
        assert_eq!(inst_type, "Acoustic Piano");
    }
}
