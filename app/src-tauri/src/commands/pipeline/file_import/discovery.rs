//! MIDI file discovery utilities

use std::path::{Path, PathBuf};

/// Check if a file is a MIDI file based on extension
pub fn is_midi_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("mid") || ext.eq_ignore_ascii_case("midi"))
        .unwrap_or(false)
}

/// Recursively collect all MIDI files in a directory
pub fn find_midi_files_recursive(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            match find_midi_files_recursive(&path) {
                Ok(subfiles) => files.extend(subfiles),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to read directory {}: {}",
                        path.display(),
                        e
                    );
                },
            }
        } else if is_midi_file(&path) {
            files.push(path);
        }
    }

    Ok(files)
}

/// Finds MIDI files in directory (non-recursive)
pub fn find_midi_files_shallow(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_midi_file(&path) {
            files.push(path);
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_midi_file() {
        assert!(is_midi_file(Path::new("test.mid")));
        assert!(is_midi_file(Path::new("test.MID")));
        assert!(is_midi_file(Path::new("test.midi")));
        assert!(is_midi_file(Path::new("test.MIDI")));
        assert!(!is_midi_file(Path::new("test.txt")));
        assert!(!is_midi_file(Path::new("test")));
    }

    #[test]
    fn test_find_midi_files_shallow() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path();

        std::fs::write(test_dir.join("file1.mid"), b"").unwrap();
        std::fs::write(test_dir.join("file2.midi"), b"").unwrap();
        std::fs::write(test_dir.join("file3.txt"), b"").unwrap();

        let files = find_midi_files_shallow(test_dir).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_find_midi_files_recursive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_dir = temp_dir.path();
        let sub_dir = test_dir.join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        std::fs::write(test_dir.join("file1.mid"), b"").unwrap();
        std::fs::write(sub_dir.join("file2.mid"), b"").unwrap();

        let files = find_midi_files_recursive(test_dir).unwrap();
        assert_eq!(files.len(), 2);
    }
}
