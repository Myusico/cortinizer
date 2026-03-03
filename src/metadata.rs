use crate::error::{CortinizerError, Result};
use anyhow::Context;
use std::path::Path;
use tracing::{debug, warn};

/// Copy metadata from input file to output file
///
/// This function attempts to preserve audio metadata (tags) when generating
/// cortinas. It supports MP3 (ID3 tags) and FLAC (Vorbis comments) formats.
///
/// This is a best-effort operation - failures will log a warning but not
/// abort the processing.
///
/// # Arguments
///
/// * `input_path` - Path to the input audio file
/// * `output_path` - Path to the output cortina file
///
/// # Returns
///
/// Ok(()) if metadata was copied (or if copy failed but we want to continue)
/// Err only if there's a critical error
pub fn copy_metadata(input_path: &Path, output_path: &Path) -> Result<()> {
    debug!(
        "Attempting to copy metadata: {:?} -> {:?}",
        input_path, output_path
    );

    // Determine file format from extension
    let extension = input_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .ok_or_else(|| CortinizerError::UnsupportedFormat(
            "Could not determine file extension".to_string(),
        ))?;

    match extension.as_str() {
        "mp3" => copy_mp3_metadata(input_path, output_path),
        "flac" => copy_flac_metadata(input_path, output_path),
        _ => {
            debug!("Metadata copy not supported for format: {}", extension);
            Ok(())
        }
    }
}

/// Copy ID3 metadata from MP3 file
fn copy_mp3_metadata(input_path: &Path, output_path: &Path) -> Result<()> {
    use audiotags::Tag;

    let mut tag = match Tag::new().read_from_path(input_path) {
        Ok(t) => t,
        Err(e) => {
            warn!(
                "Failed to read metadata from {:?}: {}, skipping metadata copy",
                input_path, e
            );
            return Ok(()); // Don't abort on metadata read failure
        }
    };

    // Attempt to write metadata to output
    let output_path_str = output_path.to_str().ok_or_else(|| CortinizerError::MetadataError(
        "Output path contains invalid UTF-8 characters".to_string(),
    ))?;

    match tag.write_to_path(output_path_str) {
        Ok(_) => {
            debug!("Successfully copied MP3 metadata to {:?}", output_path);
            Ok(())
        }
        Err(e) => {
            warn!(
                "Failed to write metadata to {:?}: {}, continuing anyway",
                output_path, e
            );
            Ok(()) // Don't abort on metadata write failure
        }
    }
}

/// Copy Vorbis comments from FLAC file
fn copy_flac_metadata(input_path: &Path, output_path: &Path) -> Result<()> {
    use audiotags::Tag;

    let mut tag = match Tag::new().read_from_path(input_path) {
        Ok(t) => t,
        Err(e) => {
            warn!(
                "Failed to read metadata from {:?}: {}, skipping metadata copy",
                input_path, e
            );
            return Ok(()); // Don't abort on metadata read failure
        }
    };

    // Attempt to write metadata to output
    let output_path_str = output_path.to_str().ok_or_else(|| CortinizerError::MetadataError(
        "Output path contains invalid UTF-8 characters".to_string(),
    ))?;

    match tag.write_to_path(output_path_str) {
        Ok(_) => {
            debug!("Successfully copied FLAC metadata to {:?}", output_path);
            Ok(())
        }
        Err(e) => {
            warn!(
                "Failed to write metadata to {:?}: {}, continuing anyway",
                output_path, e
            );
            Ok(()) // Don't abort on metadata write failure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_metadata_unsupported_extension() {
        let input_path = Path::new("/test/song.wav");
        let output_path = Path::new("/output/cortina40_song.wav");

        // Should not error, just return Ok for unsupported formats
        let result = copy_metadata(input_path, output_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_copy_metadata_no_extension() {
        let input_path = Path::new("/test/song");
        let output_path = Path::new("/output/cortina40_song");

        let result = copy_metadata(input_path, output_path);
        assert!(result.is_err());
    }
}
