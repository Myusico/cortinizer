use crate::error::{CortinizerError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents an input item (either a file or a folder)
#[derive(Debug, Clone)]
pub enum InputItem {
    AudioFile(PathBuf),
    Folder(PathBuf),
}

impl InputItem {
    /// Get the path of this input item
    pub fn path(&self) -> &Path {
        match self {
            InputItem::AudioFile(path) => path,
            InputItem::Folder(path) => path,
        }
    }

    /// Check if this item is a folder
    pub fn is_folder(&self) -> bool {
        matches!(self, InputItem::Folder(_))
    }
}

/// Supported audio file extensions
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac"];

/// Categorize input paths into files and folders
pub fn categorize_inputs(paths: &[PathBuf]) -> Result<Vec<InputItem>> {
    let mut items = Vec::new();

    for path in paths {
        // Normalize the path
        let normalized_path = dunce::canonicalize(path)
            .unwrap_or_else(|_| path.clone());

        if !normalized_path.exists() {
            eprintln!("Warning: Path does not exist: {:?}", normalized_path);
            continue;
        }

        if normalized_path.is_file() {
            // Check if it's an audio file
            if is_audio_file(&normalized_path) {
                items.push(InputItem::AudioFile(normalized_path));
            } else {
                eprintln!("Warning: Not a supported audio file: {:?}", normalized_path);
            }
        } else if normalized_path.is_dir() {
            items.push(InputItem::Folder(normalized_path));
        }
    }

    Ok(items)
}

/// Check if a file is an audio file based on its extension
pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            AUDIO_EXTENSIONS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

/// Recursively discover all audio files in a directory
pub fn discover_audio_files(folder_path: &Path) -> Result<Vec<PathBuf>> {
    let mut audio_files = Vec::new();

    for entry in WalkDir::new(folder_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_audio_file(path) {
            audio_files.push(path.to_path_buf());
        }
    }

    if audio_files.is_empty() {
        eprintln!("Warning: No audio files found in: {:?}", folder_path);
    }

    Ok(audio_files)
}

/// Get the output extension for an input file (preserves format)
pub fn get_output_extension(input_path: &Path) -> Result<String> {
    input_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .ok_or_else(|| CortinizerError::UnsupportedFormat(
            "Could not determine file extension".to_string(),
        ))
}

/// Get the output path for a single audio file
pub fn get_output_path_for_file(
    input_path: &Path,
    program_dir: &Path,
    cortina_length: u32,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CortinizerError::UnsupportedFormat(
            "Could not determine file name".to_string(),
        ))?;

    let extension = get_output_extension(input_path)?;

    let output_filename = format!("cortina{}_{}.{}", cortina_length, file_stem, extension);
    Ok(program_dir.join(output_filename))
}

/// Get the output path for a folder (preserves relative structure)
pub fn get_output_path_for_folder(
    folder_path: &Path,
    program_dir: &Path,
    audio_file_path: &Path,
    cortina_length: u32,
) -> Result<PathBuf> {
    // Get the folder name
    let folder_name = folder_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CortinizerError::UnsupportedFormat(
            "Could not determine folder name".to_string(),
        ))?;

    // Create output folder base
    let output_base = program_dir.join(folder_name);

    // Calculate relative path from folder to audio file
    let relative_path = audio_file_path
        .strip_prefix(folder_path)
        .map_err(|_| CortinizerError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not calculate relative path",
        )))?;

    // Get the parent directory structure (if any)
    let parent_dir = relative_path.parent();
    let file_stem = audio_file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| CortinizerError::UnsupportedFormat(
            "Could not determine file name".to_string(),
        ))?;

    let extension = get_output_extension(audio_file_path)?;

    // Build output path
    let output_path = if let Some(parent) = parent_dir {
        output_base
            .join(parent)
            .join(format!("cortina{}_{}.{}", cortina_length, file_stem, extension))
    } else {
        output_base.join(format!("cortina{}_{}.{}", cortina_length, file_stem, extension))
    };

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_audio_file_mp3() {
        let path = Path::new("/test/song.mp3");
        assert!(is_audio_file(path));
    }

    #[test]
    fn test_is_audio_file_flac() {
        let path = Path::new("/test/song.flac");
        assert!(is_audio_file(path));
    }

    #[test]
    fn test_is_audio_file_case_insensitive() {
        let path = Path::new("/test/song.MP3");
        assert!(is_audio_file(path));
    }

    #[test]
    fn test_is_audio_file_unsupported() {
        let path = Path::new("/test/song.wav");
        assert!(!is_audio_file(path));
    }

    #[test]
    fn test_get_output_extension_mp3() {
        let path = Path::new("/test/song.mp3");
        assert_eq!(get_output_extension(path).unwrap(), "mp3");
    }

    #[test]
    fn test_get_output_extension_flac() {
        let path = Path::new("/test/song.flac");
        assert_eq!(get_output_extension(path).unwrap(), "flac");
    }
}
