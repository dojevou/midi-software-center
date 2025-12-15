/// Archive Format Detection
use std::path::Path;

/// Supported archive formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Rar,
    SevenZip,
    TarGz,
    Tar,
}

impl ArchiveFormat {
    /// Returns file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::Rar => "rar",
            ArchiveFormat::SevenZip => "7z",
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::Tar => "tar",
        }
    }
}

/// Detects archive format from file extension
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `Some(ArchiveFormat)` if recognized, `None` otherwise
pub fn detect_format(path: &Path) -> Option<ArchiveFormat> {
    let filename = path.file_name()?.to_str()?.to_lowercase();

    if filename.ends_with(".zip") {
        Some(ArchiveFormat::Zip)
    } else if filename.ends_with(".rar") {
        Some(ArchiveFormat::Rar)
    } else if filename.ends_with(".7z") {
        Some(ArchiveFormat::SevenZip)
    } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        Some(ArchiveFormat::TarGz)
    } else if filename.ends_with(".tar") {
        Some(ArchiveFormat::Tar)
    } else {
        None
    }
}

/// Checks if file is a supported archive
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `true` if file is a recognized archive format
pub fn is_archive(path: &Path) -> bool {
    detect_format(path).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_zip() {
        let path = PathBuf::from("test.zip");
        assert_eq!(detect_format(&path), Some(ArchiveFormat::Zip));
    }

    #[test]
    fn test_detect_rar() {
        let path = PathBuf::from("archive.rar");
        assert_eq!(detect_format(&path), Some(ArchiveFormat::Rar));
    }

    #[test]
    fn test_detect_7z() {
        let path = PathBuf::from("package.7z");
        assert_eq!(detect_format(&path), Some(ArchiveFormat::SevenZip));
    }

    #[test]
    fn test_detect_tar_gz() {
        let path = PathBuf::from("archive.tar.gz");
        assert_eq!(detect_format(&path), Some(ArchiveFormat::TarGz));
    }

    #[test]
    fn test_detect_tgz() {
        let path = PathBuf::from("archive.tgz");
        assert_eq!(detect_format(&path), Some(ArchiveFormat::TarGz));
    }

    #[test]
    fn test_detect_tar() {
        let path = PathBuf::from("archive.tar");
        assert_eq!(detect_format(&path), Some(ArchiveFormat::Tar));
    }

    #[test]
    fn test_not_archive() {
        let path = PathBuf::from("file.mid");
        assert_eq!(detect_format(&path), None);
    }

    #[test]
    fn test_is_archive() {
        assert!(is_archive(&PathBuf::from("test.zip")));
        assert!(!is_archive(&PathBuf::from("test.mid")));
    }

    #[test]
    fn test_extension() {
        assert_eq!(ArchiveFormat::Zip.extension(), "zip");
        assert_eq!(ArchiveFormat::Rar.extension(), "rar");
        assert_eq!(ArchiveFormat::SevenZip.extension(), "7z");
        assert_eq!(ArchiveFormat::TarGz.extension(), "tar.gz");
        assert_eq!(ArchiveFormat::Tar.extension(), "tar");
    }

    #[test]
    fn test_case_insensitive() {
        let path = PathBuf::from("TEST.ZIP");
        assert_eq!(detect_format(&path), Some(ArchiveFormat::Zip));
    }
}
