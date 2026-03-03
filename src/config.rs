use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Configuration for the cortinizer application
#[derive(Debug, Clone)]
pub struct Config {
    /// Length of the cortina in seconds
    pub cortina_length: u32,
    /// Directory containing the cortinizer executable
    pub program_dir: PathBuf,
    /// Path to FFmpeg executable
    pub ffmpeg_path: PathBuf,
}

impl Config {
    /// Create a new configuration with the specified cortina length
    pub fn new(cortina_length: u32) -> Result<Self> {
        // Validate cortina length
        if cortina_length == 0 {
            anyhow::bail!("Cortina length must be greater than 0");
        }

        // Get the program directory (directory containing the executable)
        let program_dir = Self::get_program_dir()?;

        // Find FFmpeg executable
        let ffmpeg_path = Self::find_ffmpeg(&program_dir)?;

        Ok(Self {
            cortina_length,
            program_dir,
            ffmpeg_path,
        })
    }

    /// Get the directory containing the cortinizer executable
    fn get_program_dir() -> Result<PathBuf> {
        let exe_path = env::current_exe()
            .context("Failed to get current executable path")?;

        let program_dir = exe_path
            .parent()
            .context("Could not determine program directory")?;

        // Normalize the path (handles Windows paths like \\?\ correctly)
        Ok(dunce::canonicalize(program_dir)?)
    }

    /// Find the FFmpeg executable
    /// - On Windows: look for ffmpeg.exe beside the executable first, then PATH
    /// - On Linux/macOS: use ffmpeg from PATH
    fn find_ffmpeg(program_dir: &Path) -> Result<PathBuf> {
        // On Windows, first try bundled ffmpeg.exe
        if cfg!(windows) {
            let bundled_ffmpeg = program_dir.join("ffmpeg.exe");
            if bundled_ffmpeg.exists() {
                tracing::debug!("Using bundled FFmpeg: {:?}", bundled_ffmpeg);
                return Ok(bundled_ffmpeg);
            }
        }

        // Try to find ffmpeg in PATH
        match which::which("ffmpeg") {
            Ok(ffmpeg_path) => {
                tracing::debug!("Using FFmpeg from PATH: {:?}", ffmpeg_path);
                Ok(ffmpeg_path)
            }
            Err(e) => {
                anyhow::bail!(
                    "Could not find FFmpeg. {}\n\
                     On Windows, place ffmpeg.exe beside cortinizer.exe.\n\
                     On Linux/macOS, install ffmpeg through your package manager.",
                    e
                );
            }
        }
    }

    /// Calculate the start time for the fade-out effect
    /// Returns the timestamp (in seconds) where the 5-second fade-out should begin
    pub fn calculate_fade_start(&self) -> f64 {
        // Fade starts 5 seconds before the end of the cortina
        // If cortina is shorter than 5 seconds, start at 0
        (self.cortina_length as f64 - 5.0).max(0.0)
    }

    /// Get the fade duration in seconds
    pub fn fade_duration(&self) -> f64 {
        // Fade duration is min(5, cortina_length)
        5.0_f64.min(self.cortina_length as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fade_start_normal() {
        let config = Config::new(40).unwrap();
        assert_eq!(config.calculate_fade_start(), 35.0);
    }

    #[test]
    fn test_calculate_fade_start_short() {
        let config = Config::new(3).unwrap();
        assert_eq!(config.calculate_fade_start(), 0.0);
    }

    #[test]
    fn test_calculate_fade_start_exactly_five() {
        let config = Config::new(5).unwrap();
        assert_eq!(config.calculate_fade_start(), 0.0);
    }

    #[test]
    fn test_fade_duration() {
        let config = Config::new(40).unwrap();
        assert_eq!(config.fade_duration(), 5.0);
    }

    #[test]
    fn test_fade_duration_short() {
        let config = Config::new(3).unwrap();
        assert_eq!(config.fade_duration(), 3.0);
    }

    #[test]
    fn test_zero_length_rejected() {
        let result = Config::new(0);
        assert!(result.is_err());
    }
}
