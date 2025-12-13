/// Auto-Repair Module - TRUSTY MODULE
///
/// Automatic repair of corrupted MIDI files before splitting.
/// Fixes common issues like missing End-of-Track markers and trailing garbage.
///
/// # Archetype: TRUSTY MODULE
/// - ✅ Pure functions, no side effects
/// - ✅ No I/O operations
/// - ✅ Operates on byte slices
/// - ✅ Comprehensive error handling
/// - ✅ Well-documented
use super::track_splitter::{split_tracks, SplitTrack};
use thiserror::Error;

/// Result of an auto-repair attempt
#[derive(Debug, Clone, PartialEq)]
pub enum RepairResult {
    /// File was valid, no repair needed
    Valid,

    /// File was repaired successfully
    Repaired {
        /// Description of what was fixed
        fix_description: String,
        /// Repaired MIDI bytes
        repaired_bytes: Vec<u8>,
    },

    /// File is corrupt and cannot be automatically repaired
    Corrupt {
        /// Description of the corruption
        reason: String,
    },
}

/// Errors that can occur during auto-repair operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AutoRepairError {
    /// File is corrupt and cannot be repaired
    #[error("File is corrupt and cannot be repaired: {0}")]
    UnrepairableCorruption(String),

    /// Repair was attempted but failed
    #[error("Repair failed: {0}")]
    RepairFailed(String),

    /// File is not a valid MIDI file
    #[error("Not a MIDI file")]
    NotMidi,
}

/// Split tracks with automatic repair on failure.
///
/// This function attempts to split MIDI tracks, and if that fails due to
/// corruption, it automatically attempts to repair the file and retry the split.
///
/// # Workflow
/// 1. Try to split tracks normally
/// 2. If split fails, attempt to repair the file
/// 3. If repair succeeds, retry the split operation
/// 4. Return split tracks with repair information
///
/// # Arguments
///
/// * `midi_bytes` - Complete MIDI file as byte slice
///
/// # Returns
///
/// `Ok((tracks, repair_result))` with the split tracks and repair status
/// `Err(AutoRepairError)` if the file is corrupt and unrepairable
///
/// # Examples
///
/// ```
/// use pipeline::core::splitting::auto_repair::split_tracks_with_repair;
///
/// let midi_bytes = include_bytes!("test_data/corrupt.mid");
/// match split_tracks_with_repair(midi_bytes) {
///     Ok((tracks, repair_result)) => {
///         println!("Split {} tracks", tracks.len());
///         match repair_result {
///             RepairResult::Valid => println!("File was valid"),
///             RepairResult::Repaired { fix_description, .. } => {
///                 println!("Repaired: {}", fix_description);
///             },
///             _ => {}
///         }
///     },
///     Err(e) => eprintln!("Failed to split: {}", e),
/// }
/// # Ok::<(), pipeline::core::splitting::auto_repair::AutoRepairError>(())
/// ```
pub fn split_tracks_with_repair(
    midi_bytes: &[u8],
) -> Result<(Vec<SplitTrack>, RepairResult), AutoRepairError> {
    // Try to split normally first
    match split_tracks(midi_bytes) {
        Ok(tracks) => Ok((tracks, RepairResult::Valid)),
        Err(original_error) => {
            // Attempt to repair the file
            match attempt_repair(midi_bytes) {
                Ok((repaired_bytes, fix_description)) => {
                    // Try splitting the repaired version
                    match split_tracks(&repaired_bytes) {
                        Ok(tracks) => Ok((
                            tracks,
                            RepairResult::Repaired {
                                fix_description: fix_description.clone(),
                                repaired_bytes: repaired_bytes.clone(),
                            },
                        )),
                        Err(repair_split_error) => Err(AutoRepairError::RepairFailed(format!(
                            "Original error: {}. Repair attempted: {}. Post-repair error: {}",
                            original_error, fix_description, repair_split_error
                        ))),
                    }
                },
                Err(repair_error) => Err(AutoRepairError::UnrepairableCorruption(format!(
                    "Original error: {}. Repair error: {}",
                    original_error, repair_error
                ))),
            }
        },
    }
}

/// Attempt to repair common MIDI file corruption issues.
///
/// Applies the following fixes in order:
/// 1. Add missing End-of-Track markers (0xFF 0x2F 0x00)
/// 2. Trim trailing garbage data after the last track
///
/// # Arguments
///
/// * `data` - Original MIDI file bytes
///
/// # Returns
///
/// `Ok((repaired_bytes, fix_description))` if repair was successful
/// `Err(reason)` if no repair was needed or possible
///
/// # Examples
///
/// ```
/// use pipeline::core::splitting::auto_repair::attempt_repair;
///
/// let corrupt_midi = b"MThd..."; // Corrupt MIDI bytes
/// match attempt_repair(corrupt_midi) {
///     Ok((repaired, desc)) => println!("Fixed: {}", desc),
///     Err(e) => println!("Cannot repair: {}", e),
/// }
/// ```
pub fn attempt_repair(data: &[u8]) -> Result<(Vec<u8>, String), String> {
    let mut repaired = data.to_vec();
    let mut fixes = Vec::new();

    // Minimum size check
    if repaired.len() < 14 {
        return Err(format!(
            "File too small ({} bytes, need 14+)",
            repaired.len()
        ));
    }

    // Check if it's a MIDI file
    if &repaired[0..4] != b"MThd" {
        return Err("Not a MIDI file (missing MThd header)".to_string());
    }

    // Fix 1: Add missing End-of-Track marker (FF 2F 00)
    // This is the most common issue
    if repaired.len() >= 14 {
        // Check if file has proper header
        if &repaired[0..4] == b"MThd" {
            // Look for track chunks
            let mut pos = 14; // After header
            while pos < repaired.len() {
                if pos + 8 > repaired.len() {
                    break;
                }

                if &repaired[pos..pos + 4] == b"MTrk" {
                    let track_len = u32::from_be_bytes([
                        repaired[pos + 4],
                        repaired[pos + 5],
                        repaired[pos + 6],
                        repaired[pos + 7],
                    ]) as usize;

                    let track_end = pos + 8 + track_len;
                    if track_end <= repaired.len() {
                        // Check if track ends with End-of-Track (FF 2F 00)
                        let has_eot = if track_end >= 3 {
                            repaired[track_end - 3..track_end] == [0xFF, 0x2F, 0x00]
                        } else {
                            false
                        };

                        if !has_eot && track_end < repaired.len() {
                            // Insert End-of-Track at proper position
                            repaired
                                .splice(track_end..track_end, [0xFF, 0x2F, 0x00].iter().cloned());

                            // Update track length in header
                            let new_len = track_len + 3;
                            let len_bytes = (new_len as u32).to_be_bytes();
                            repaired[pos + 4] = len_bytes[0];
                            repaired[pos + 5] = len_bytes[1];
                            repaired[pos + 6] = len_bytes[2];
                            repaired[pos + 7] = len_bytes[3];

                            fixes.push("Added missing End-of-Track marker".to_string());
                        }
                        pos = track_end;
                    } else {
                        break;
                    }
                } else {
                    pos += 1;
                }
            }
        }
    }

    // Fix 2: Trim trailing garbage data
    if repaired.len() > 14 && &repaired[0..4] == b"MThd" {
        let header_len =
            u32::from_be_bytes([repaired[4], repaired[5], repaired[6], repaired[7]]) as usize;
        if header_len == 6 {
            let num_tracks = u16::from_be_bytes([repaired[10], repaired[11]]) as usize;

            // Calculate expected file size
            let mut expected_size = 14; // Header
            let mut pos = 14;

            for _ in 0..num_tracks {
                if pos + 8 > repaired.len() {
                    break;
                }
                if &repaired[pos..pos + 4] == b"MTrk" {
                    let track_len = u32::from_be_bytes([
                        repaired[pos + 4],
                        repaired[pos + 5],
                        repaired[pos + 6],
                        repaired[pos + 7],
                    ]) as usize;
                    expected_size = pos + 8 + track_len;
                    pos = expected_size;
                } else {
                    break;
                }
            }

            if expected_size < repaired.len() {
                let trimmed = repaired.len() - expected_size;
                repaired.truncate(expected_size);
                fixes.push(format!("Trimmed {} bytes of trailing garbage", trimmed));
            }
        }
    }

    if fixes.is_empty() {
        Err("No repairs needed or possible".to_string())
    } else {
        Ok((repaired, fixes.join(", ")))
    }
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attempt_repair_too_small() {
        let data = b"MThd";
        let result = attempt_repair(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too small"));
    }

    #[test]
    fn test_attempt_repair_not_midi() {
        let data = b"Not a MIDI file at all, just random bytes here";
        let result = attempt_repair(data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not a MIDI file"));
    }

    #[test]
    fn test_attempt_repair_no_fixes_needed() {
        // Create a minimal valid MIDI file
        let mut data = Vec::new();
        // MThd header
        data.extend_from_slice(b"MThd");
        data.extend_from_slice(&6u32.to_be_bytes()); // Header length
        data.extend_from_slice(&0u16.to_be_bytes()); // Format 0
        data.extend_from_slice(&1u16.to_be_bytes()); // 1 track
        data.extend_from_slice(&480u16.to_be_bytes()); // Ticks per quarter

        // MTrk chunk with proper End-of-Track
        data.extend_from_slice(b"MTrk");
        data.extend_from_slice(&3u32.to_be_bytes()); // Track length
        data.extend_from_slice(&[0xFF, 0x2F, 0x00]); // End-of-Track

        let result = attempt_repair(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No repairs needed"));
    }

    #[test]
    fn test_repair_result_equality() {
        let valid = RepairResult::Valid;
        assert_eq!(valid, RepairResult::Valid);

        let repaired1 = RepairResult::Repaired {
            fix_description: "Fixed".to_string(),
            repaired_bytes: vec![1, 2, 3],
        };
        let repaired2 = RepairResult::Repaired {
            fix_description: "Fixed".to_string(),
            repaired_bytes: vec![1, 2, 3],
        };
        assert_eq!(repaired1, repaired2);

        let corrupt1 = RepairResult::Corrupt { reason: "Bad".to_string() };
        let corrupt2 = RepairResult::Corrupt { reason: "Bad".to_string() };
        assert_eq!(corrupt1, corrupt2);
    }

    #[test]
    fn test_auto_repair_error_display() {
        let err = AutoRepairError::NotMidi;
        assert_eq!(err.to_string(), "Not a MIDI file");

        let err = AutoRepairError::UnrepairableCorruption("test".to_string());
        assert!(err.to_string().contains("corrupt"));

        let err = AutoRepairError::RepairFailed("test".to_string());
        assert!(err.to_string().contains("Repair failed"));
    }
}
