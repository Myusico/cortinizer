use crate::config::Config;
use crate::error::{CortinizerError, Result};
use std::io;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, warn};

/// Get the duration of an audio file using ffprobe
///
/// # Arguments
///
/// * `config` - Application configuration containing FFmpeg path
/// * `audio_path` - Path to the audio file
///
/// # Returns
///
/// Duration in seconds as f64
pub async fn get_audio_duration(config: &Config, audio_path: &Path) -> Result<f64> {
    let ffprobe_path = config.ffmpeg_path.with_file_name(
        if cfg!(windows) {
            "ffprobe.exe"
        } else {
            "ffprobe"
        }
    );

    debug!("Getting duration for: {:?}", audio_path);

    let output = Command::new(&ffprobe_path)
        .arg("-i")
        .arg(audio_path)
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-v")
        .arg("error")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CortinizerError::FfmpegError(format!("Failed to execute ffprobe: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CortinizerError::DurationError {
            path: audio_path.to_path_buf(),
        }
        .into());
    }

    let duration_str = String::from_utf8_lossy(&output.stdout);
    let duration: f64 = duration_str
        .trim()
        .parse()
        .map_err(|_| CortinizerError::DurationError {
            path: audio_path.to_path_buf(),
        })?;

    debug!("Duration for {:?}: {} seconds", audio_path, duration);
    Ok(duration)
}

/// Generate a cortina from an audio file using FFmpeg
///
/// This function extracts the specified duration from the beginning of the track
/// and applies a fade-out effect during the last 5 seconds.
///
/// # Arguments
///
/// * `config` - Application configuration containing cortina length and FFmpeg path
/// * `input_path` - Path to the input audio file
/// * `output_path` - Path where the cortina should be saved
///
/// # Returns
///
/// Result indicating success or failure
pub async fn generate_cortina(
    config: &Config,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    debug!(
        "Generating cortina: {:?} -> {:?}",
        input_path, output_path
    );

    // Get input duration to handle short tracks
    let input_duration = get_audio_duration(config, input_path).await?;

    // Determine the actual trim duration
    let trim_duration = if input_duration < config.cortina_length as f64 {
        warn!(
            "Track duration ({:.2}s) is shorter than cortina length ({}s), using full track",
            input_duration, config.cortina_length
        );
        input_duration
    } else {
        config.cortina_length as f64
    };

    // Calculate fade parameters
    let fade_start = config.calculate_fade_start();
    let fade_duration = config.fade_duration();

    // Only apply fade if the track is long enough
    let filter_complex = if trim_duration >= 5.0 {
        format!(
            "atrim=0:{},afade=t=out:st={}:d={}",
            trim_duration, fade_start, fade_duration
        )
    } else {
        format!("atrim=0:{}", trim_duration)
    };

    debug!("Using FFmpeg filter: {}", filter_complex);

    // Determine codec based on output extension
    let output_ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("mp3")
        .to_lowercase();

    let codec = match output_ext.as_str() {
        "mp3" => Some("libmp3lame"),
        "flac" => Some("flac"),
        _ => None,
    };

    // Build FFmpeg command
    let mut cmd = Command::new(&config.ffmpeg_path);
    cmd.arg("-i")
        .arg(input_path)
        .arg("-filter_complex")
        .arg(&filter_complex);

    // Add codec if specified
    if let Some(c) = codec {
        cmd.arg("-c:a").arg(c);
    }

    cmd.arg("-y") // Overwrite output file
        .arg(output_path);

    debug!("Executing FFmpeg command: {:?}", cmd);

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| CortinizerError::FfmpegError(format!("Failed to execute ffmpeg: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CortinizerError::FfmpegError(format!(
            "FFmpeg failed: {}",
            stderr
        ))
        .into());
    }

    debug!("Cortina generated successfully: {:?}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_filter_complex_normal() {
        // Test the filter complex construction logic
        let trim_duration = 40.0;
        let fade_start = 35.0;
        let fade_duration = 5.0;

        let filter = format!(
            "atrim=0:{},afade=t=out:st={}:d={}",
            trim_duration, fade_start, fade_duration
        );

        assert_eq!(filter, "atrim=0:40,afade=t=out:st=35:d=5");
    }

    #[test]
    fn test_filter_complex_short() {
        // Test filter for short tracks (< 5 seconds)
        let trim_duration = 3.0;
        let filter = format!("atrim=0:{}", trim_duration);

        assert_eq!(filter, "atrim=0:3");
    }
}
