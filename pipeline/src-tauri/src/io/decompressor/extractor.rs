   /// Archive Extraction Logic
   ///
   /// # Archetype: Grown-up Script
   /// - Performs I/O operations (file extraction)
   /// - Separates I/O logic from business logic
   /// - Both runnable AND importable
   /// - Returns Result types for error handling

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::io::decompressor::{formats, temp_manager};
use crate::io::{IoError, Result};

/// Configuration for extraction
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// Maximum recursion depth for nested archives
    pub max_depth: usize,

    /// Whether to extract nested archives
    pub recursive: bool,

    /// Extensions to extract (e.g., ["mid", "midi"])
    pub target_extensions: Vec<String>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            max_depth: 10, // Increased to handle deeply nested archives (up to 8 layers observed)
            recursive: true,
            target_extensions: vec!["mid".to_string(), "midi".to_string()],
        }
    }
}

/// Result of extraction operation
#[derive(Debug)]
pub struct ExtractionResult {
    /// Paths to extracted MIDI files
    pub midi_files: Vec<PathBuf>,

    /// Number of archives processed
    pub archives_processed: usize,

    /// Errors encountered
    pub errors: Vec<String>,
}

/// Extracts MIDI files from an archive
///
/// # Arguments
/// * `archive_path` - Path to archive file
/// * `output_dir` - Where to extract files
/// * `config` - Extraction configuration
///
/// # Returns
/// * `ExtractionResult` - List of extracted MIDI files
///
/// # Examples
/// ```no_run
/// use std::path::Path;
/// use pipeline::io::decompressor::extractor::*;
///
/// let config = ExtractionConfig::default();
/// let result = extract_archive(
///     Path::new("samples.zip"),
///     Path::new("/output"),
///     &config
/// ).unwrap();
///
/// println!("Extracted {} MIDI files", result.midi_files.len());
/// ```
pub fn extract_archive(
    archive_path: &Path,
    output_dir: &Path,
    config: &ExtractionConfig,
) -> Result<ExtractionResult> {
    let format = formats::detect_format(archive_path)
        .ok_or_else(|| IoError::UnsupportedFormat {
            path: archive_path.to_path_buf(),
        })?;

    let mut result = ExtractionResult {
        midi_files: Vec::new(),
        archives_processed: 0,
        errors: Vec::new(),
    };

    extract_recursive(archive_path, output_dir, config, 0, &mut result, format)?;

    Ok(result)
}

/// Internal recursive extraction function
fn extract_recursive(
    archive_path: &Path,
    output_dir: &Path,
    config: &ExtractionConfig,
    current_depth: usize,
    result: &mut ExtractionResult,
    format: formats::ArchiveFormat,
) -> Result<()> {
    if current_depth >= config.max_depth {
        result
            .errors
            .push(format!("Max depth reached at: {}", archive_path.display()));
        return Ok(());
    }

    result.archives_processed += 1;

    match format {
        formats::ArchiveFormat::Zip => {
            extract_zip(archive_path, output_dir, config, current_depth, result)?;
        }
        _ => {
            result
                .errors
                .push(format!("Format {:?} not yet implemented", format));
        }
    }

    Ok(())
}

/// Extracts ZIP archive
fn extract_zip(
    archive_path: &Path,
    output_dir: &Path,
    config: &ExtractionConfig,
    current_depth: usize,
    result: &mut ExtractionResult,
) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    fs::create_dir_all(output_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            // Directory
            fs::create_dir_all(&outpath)?;
        } else {
            // File
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;

            // Check if it's a MIDI file
            if is_target_file(&outpath, &config.target_extensions) {
                result.midi_files.push(outpath.clone());
            }

            // Check if it's a nested archive
            if config.recursive && formats::is_archive(&outpath) {
                if let Some(nested_format) = formats::detect_format(&outpath) {
                    let _ = extract_recursive(
                        &outpath,
                        output_dir,
                        config,
                        current_depth + 1,
                        result,
                        nested_format,
                    );
                }
            }
        }
    }

    Ok(())
}

/// Checks if file has target extension
fn is_target_file(path: &Path, target_extensions: &[String]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext_str| {
            let ext_lower = ext_str.to_lowercase();
            target_extensions.iter().any(|target| target == &ext_lower)
        })
        .unwrap_or(false)
}

/// Convenience function for extracting to temporary directory
///
/// # Arguments
/// * `archive_path` - Path to archive file
/// * `config` - Extraction configuration
///
/// # Returns
/// * `(ExtractionResult, PathBuf)` - Extraction result and temp directory path
pub fn extract_to_temp(
    archive_path: &Path,
    config: &ExtractionConfig,
) -> Result<(ExtractionResult, PathBuf)> {
    let mut temp_mgr = temp_manager::TempManager::new()?;
    let temp_dir = temp_mgr.create_temp_dir()?;

    let result = extract_archive(archive_path, &temp_dir, config)?;

    // Note: temp_mgr will be dropped but we return temp_dir
    // Caller is responsible for cleanup
    std::mem::forget(temp_mgr);

    Ok((result, temp_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_target_file() {
        let path = PathBuf::from("test.mid");
        let extensions = vec!["mid".to_string(), "midi".to_string()];

        assert!(is_target_file(&path, &extensions));
    }

    #[test]
    fn test_is_target_file_case_insensitive() {
        let path = PathBuf::from("test.MID");
        let extensions = vec!["mid".to_string()];

        assert!(is_target_file(&path, &extensions));
    }

    #[test]
    fn test_not_target_file() {
        let path = PathBuf::from("test.txt");
        let extensions = vec!["mid".to_string()];

        assert!(!is_target_file(&path, &extensions));
    }

    #[test]
    fn test_default_config() {
        let config = ExtractionConfig::default();

        assert_eq!(config.max_depth, 10);
        assert!(config.recursive);
        assert_eq!(config.target_extensions, vec!["mid", "midi"]);
    }

    #[test]
    fn test_extraction_result() {
        let result = ExtractionResult {
            midi_files: vec![PathBuf::from("test.mid")],
            archives_processed: 1,
            errors: vec![],
        };

        assert_eq!(result.midi_files.len(), 1);
        assert_eq!(result.archives_processed, 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_unsupported_format() {
        let path = PathBuf::from("test.txt");
        let output = PathBuf::from("/tmp/output");
        let config = ExtractionConfig::default();

        let result = extract_archive(&path, &output, &config);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported archive format"));
    }
}
