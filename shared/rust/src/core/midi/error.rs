use thiserror::Error;

#[derive(Error, Debug)]
pub enum MidiParseError {
    #[error("Invalid MIDI header: {0}")]
    InvalidHeader(String),

    #[error("Invalid track data at byte {position}: {reason}")]
    InvalidTrack { position: usize, reason: String },

    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(u16),

    #[error("Invalid event at byte {position}: {reason}")]
    InvalidEvent { position: usize, reason: String },

    #[error("Incomplete data: expected {expected} bytes, got {actual}")]
    IncompleteData { expected: usize, actual: usize },

    #[error("Invalid variable-length quantity at byte {0}")]
    InvalidVarLen(usize),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, MidiParseError>;
