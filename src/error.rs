use std::path::PathBuf;
use thiserror::Error;

/// Type alias for Result with CortinizerError
pub type Result<T> = std::result::Result<T, CortinizerError>;

/// Custom error types for the cortinizer application
#[derive(Error, Debug)]
pub enum CortinizerError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// FFmpeg-related error
    #[error("FFmpeg error: {0}")]
    FfmpegError(String),

    /// Unsupported audio format
    #[error("Unsupported audio format: {0}")]
    UnsupportedFormat(String),

    /// Invalid audio file
    #[error("Invalid audio file: {path}")]
    InvalidAudioFile { path: PathBuf },

    /// FFprobe failed to get duration
    #[error("Failed to get audio duration for: {path}")]
    DurationError { path: PathBuf },

    /// Metadata copy error
    #[error("Failed to copy metadata: {0}")]
    MetadataError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Result of processing a single file
#[derive(Debug, Clone)]
pub struct ProcessResult {
    /// Input file path
    pub input_path: PathBuf,
    /// Output file path
    pub output_path: Option<PathBuf>,
    /// Processing status
    pub status: ProcessStatus,
}

/// Status of file processing
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    /// File processed successfully
    Success,
    /// File processing failed
    Failed(String),
}

impl ProcessResult {
    /// Create a new successful result
    pub fn success(input_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            input_path,
            output_path: Some(output_path),
            status: ProcessStatus::Success,
        }
    }

    /// Create a new failed result
    pub fn failed(input_path: PathBuf, error: String) -> Self {
        Self {
            input_path,
            output_path: None,
            status: ProcessStatus::Failed(error),
        }
    }

    /// Check if the processing was successful
    pub fn is_success(&self) -> bool {
        self.status == ProcessStatus::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_process_result_success() {
        let input = PathBuf::from("/input/test.mp3");
        let output = PathBuf::from("/output/cortina40_test.mp3");
        let result = ProcessResult::success(input.clone(), output.clone());

        assert!(result.is_success());
        assert_eq!(result.input_path, input);
        assert_eq!(result.output_path, Some(output));
    }

    #[test]
    fn test_process_result_failed() {
        let input = PathBuf::from("/input/test.mp3");
        let error = "FFmpeg failed".to_string();
        let result = ProcessResult::failed(input.clone(), error.clone());

        assert!(!result.is_success());
        assert_eq!(result.input_path, input);
        assert!(result.output_path.is_none());
        assert_eq!(result.status, ProcessStatus::Failed(error));
    }
}
