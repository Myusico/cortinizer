use crate::error::{CortinizerError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Ensure the output directory exists, creating it if necessary
pub fn ensure_output_dir(output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| CortinizerError::IoError(e))?;
            tracing::debug!("Created output directory: {:?}", parent);
        }
    }
    Ok(())
}

/// Get a unique output path by adding numeric suffixes if the file exists
///
/// If the output path already exists, this function will append _1, _2, etc.
/// to the filename stem until a unique path is found.
///
/// # Arguments
///
/// * `output_path` - The desired output path
///
/// # Returns
///
/// A unique path that doesn't conflict with existing files
///
/// # Examples
///
/// ```
/// // If cortina40_song.mp3 exists, returns cortina40_song_1.mp3
/// // If that exists, returns cortina40_song_2.mp3, etc.
/// ```
pub fn get_unique_output_path(output_path: PathBuf) -> PathBuf {
    if !output_path.exists() {
        return output_path;
    }

    // Extract file components
    let parent = output_path.parent().unwrap_or(Path::new(""));
    let file_stem = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let extension = output_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default();

    // Try numeric suffixes starting from 1
    let mut counter = 1;
    loop {
        let new_filename = format!("{}_{}{}", file_stem, counter, extension);
        let new_path = parent.join(&new_filename);

        if !new_path.exists() {
            tracing::debug!(
                "Output file exists, using unique path: {:?}",
                new_path
            );
            return new_path;
        }

        counter += 1;

        // Safety limit to prevent infinite loops
        if counter > 10000 {
            tracing::warn!(
                "Could not find unique output path after 10000 attempts, using: {:?}",
                output_path
            );
            return output_path;
        }
    }
}

/// Get a unique output folder path by adding numeric suffixes if the folder exists
///
/// Similar to get_unique_output_path but for directories. Used when multiple
/// folders with the same name are dropped on the executable.
///
/// # Arguments
///
/// * `folder_path` - The desired output folder path
///
/// # Returns
///
/// A unique folder path that doesn't conflict with existing folders
pub fn get_unique_folder_path(folder_path: PathBuf) -> PathBuf {
    if !folder_path.exists() {
        return folder_path;
    }

    let parent = folder_path.parent().unwrap_or(Path::new(""));
    let folder_name = folder_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    // Try numeric suffixes starting from 1
    let mut counter = 1;
    loop {
        let new_folder_name = format!("{}_{}", folder_name, counter);
        let new_path = parent.join(&new_folder_name);

        if !new_path.exists() {
            tracing::debug!(
                "Output folder exists, using unique path: {:?}",
                new_path
            );
            return new_path;
        }

        counter += 1;

        // Safety limit to prevent infinite loops
        if counter > 10000 {
            tracing::warn!(
                "Could not find unique folder path after 10000 attempts, using: {:?}",
                folder_path
            );
            return folder_path;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_ensure_output_dir_creates_directory() {
        let temp_dir = std::env::temp_dir().join("cortinizer_test_create_dir");
        let test_path = temp_dir.join("subdir1").join("subdir2").join("output.mp3");

        // Clean up any existing test directory
        let _ = fs::remove_dir_all(&temp_dir);

        let result = ensure_output_dir(&test_path);
        assert!(result.is_ok());
        assert!(test_path.parent().unwrap().exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_unique_output_path_no_collision() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join(format!("unique_test_{}.mp3", std::process::id()));

        // Remove file if it exists
        let _ = fs::remove_file(&test_path);

        let unique_path = get_unique_output_path(test_path.clone());
        assert_eq!(unique_path, test_path);

        // Clean up
        let _ = fs::remove_file(&test_path);
    }

    #[test]
    fn test_get_unique_output_path_with_collision() {
        let temp_dir = std::env::temp_dir();
        let test_id = std::process::id();
        let base_name = format!("collision_test_{}", test_id);
        let test_path = temp_dir.join(format!("{}.mp3", base_name));

        // Clean up any existing files
        for i in 0..5 {
            let path = temp_dir.join(format!("{}_{}.mp3", base_name, i));
            let _ = fs::remove_file(&path);
        }

        // Create the base file
        File::create(&test_path).unwrap();

        let unique_path = get_unique_output_path(test_path.clone());
        assert_eq!(unique_path, temp_dir.join(format!("{}_1.mp3", base_name)));

        // Create the _1 file and test again
        File::create(&unique_path).unwrap();
        let unique_path2 = get_unique_output_path(test_path.clone());
        assert_eq!(unique_path2, temp_dir.join(format!("{}_2.mp3", base_name)));

        // Clean up
        let _ = fs::remove_file(&test_path);
        let _ = fs::remove_file(&unique_path);
        let _ = fs::remove_file(&unique_path2);
    }

    #[test]
    fn test_get_unique_output_path_preserves_extension() {
        let temp_dir = std::env::temp_dir();
        let test_id = std::process::id();
        let base_name = format!("ext_test_{}", test_id);
        let test_path = temp_dir.join(format!("{}.flac", base_name));

        // Create the base file
        File::create(&test_path).unwrap();

        let unique_path = get_unique_output_path(test_path.clone());
        assert_eq!(unique_path.extension().unwrap().to_str().unwrap(), "flac");

        // Clean up
        let _ = fs::remove_file(&test_path);
        let _ = fs::remove_file(&unique_path);
    }
}
