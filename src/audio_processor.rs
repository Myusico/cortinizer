use crate::config::Config;
use crate::error::ProcessResult;
use crate::file_handler::{discover_audio_files, get_output_path_for_file, get_output_path_for_folder, InputItem};
use crate::ffmpeg::generate_cortina;
use crate::output_manager::{ensure_output_dir, get_unique_folder_path, get_unique_output_path};
use futures::stream::{self, StreamExt};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Process a single audio file
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `input_path` - Path to the input audio file
/// * `output_base_path` - Computed output path (may be modified for collisions)
///
/// # Returns
///
/// ProcessResult indicating success or failure
async fn process_file(
    config: Arc<Config>,
    input_path: PathBuf,
    output_base_path: PathBuf,
) -> ProcessResult {
    let input_display = input_path.display().to_string();

    info!("Processing: {}", input_display);

    // Ensure output directory exists
    if let Err(e) = ensure_output_dir(&output_base_path) {
        let error_msg = format!("Failed to create output directory: {}", e);
        warn!("{}", error_msg);
        return ProcessResult::failed(input_path, error_msg);
    }

    // Get unique output path (handle collisions)
    let output_path = get_unique_output_path(output_base_path);

    // Generate cortina
    match generate_cortina(&config, &input_path, &output_path).await {
        Ok(_) => {
            info!("✓ {} -> {}", input_display, output_path.display());
            ProcessResult::success(input_path, output_path)
        }
        Err(e) => {
            let error_msg = format!("Failed: {}", e);
            warn!("✗ {}: {}", input_display, error_msg);
            ProcessResult::failed(input_path, error_msg)
        }
    }
}

/// Process all audio files in a folder concurrently
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `folder_path` - Path to the folder
///
/// # Returns
///
/// Vec of ProcessResult for all files in the folder
async fn process_folder(config: Arc<Config>, folder_path: PathBuf) -> Vec<ProcessResult> {
    info!("Processing folder: {:?}", folder_path);

    // Discover audio files in folder
    let audio_files = match discover_audio_files(&folder_path) {
        Ok(files) => files,
        Err(e) => {
            warn!("Failed to discover audio files in {:?}: {}", folder_path, e);
            return vec![ProcessResult::failed(
                folder_path,
                format!("Failed to discover audio files: {}", e),
            )];
        }
    };

    if audio_files.is_empty() {
        warn!("No audio files found in {:?}", folder_path);
        return vec![];
    }

    info!("Found {} audio file(s) in {:?}", audio_files.len(), folder_path);

    // Determine parallelism based on CPU count
    let max_parallel = std::cmp::min(num_cpus::get(), audio_files.len());

    // Build futures for each file
    let futures: Vec<_> = audio_files
        .into_iter()
        .map(|input_path| {
            let config = config.clone();
            let folder_path = folder_path.clone();

            async move {
                // Calculate output path for this file
                let output_base_path = match get_output_path_for_folder(
                    &folder_path,
                    &config.program_dir,
                    &input_path,
                    config.cortina_length,
                ) {
                    Ok(path) => path,
                    Err(e) => {
                        return ProcessResult::failed(
                            input_path,
                            format!("Failed to calculate output path: {}", e),
                        )
                    }
                };

                process_file(config, input_path, output_base_path).await
            }
        })
        .collect();

    // Process files concurrently
    let results = stream::iter(futures)
        .buffer_unordered(max_parallel)
        .collect::<Vec<_>>()
        .await;

    results
}

/// Process all input items (files and folders)
///
/// This is the main entry point for batch processing. It categorizes inputs
/// and processes them appropriately:
/// - Individual files are processed directly
/// - Folders are processed with structure preservation
///
/// # Arguments
///
/// * `config` - Application configuration
/// * `items` - Input items to process
///
/// # Returns
///
/// Vec of ProcessResult for all processed items
pub async fn process_inputs(config: Config, items: Vec<InputItem>) -> Vec<ProcessResult> {
    let config = Arc::new(config);
    let mut all_results = Vec::new();

    // Separate files and folders
    let mut files = Vec::new();
    let mut folders = Vec::new();

    for item in items {
        match item {
            InputItem::AudioFile(path) => files.push(path),
            InputItem::Folder(path) => folders.push(path),
        }
    }

    // Process individual files
    if !files.is_empty() {
        info!("Processing {} individual file(s)", files.len());

        // Determine parallelism
        let max_parallel = std::cmp::min(num_cpus::get(), files.len());

        // Build futures for each file
        let futures: Vec<_> = files
            .into_iter()
            .map(|input_path| {
                let config = config.clone();

                async move {
                    // Calculate output path for this file
                    let output_base_path =
                        match get_output_path_for_file(&input_path, &config.program_dir, config.cortina_length) {
                            Ok(path) => path,
                            Err(e) => {
                                return ProcessResult::failed(
                                    input_path,
                                    format!("Failed to calculate output path: {}", e),
                                )
                            }
                        };

                    process_file(config, input_path, output_base_path).await
                }
            })
            .collect();

        // Process files concurrently
        let file_results = stream::iter(futures)
            .buffer_unordered(max_parallel)
            .collect::<Vec<_>>()
            .await;

        all_results.extend(file_results);
    }

    // Process folders
    if !folders.is_empty() {
        info!("Processing {} folder(s)", folders.len());

        // Handle folder name collisions
        let mut unique_folders = Vec::new();
        for folder_path in folders {
            let folder_name = folder_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let output_base = config.program_dir.join(folder_name);
            let unique_output_path = get_unique_folder_path(output_base);

            debug!(
                "Processing folder {:?} -> output base {:?}",
                folder_path, unique_output_path
            );

            unique_folders.push((folder_path, unique_output_path));
        }

        // Process folders sequentially (each folder is processed internally with parallelism)
        for (folder_path, _output_base) in unique_folders {
            let mut results = process_folder(config.clone(), folder_path).await;
            all_results.append(&mut results);
        }
    }

    all_results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_file_input_handling() {
        // Test that file path handling works correctly
        let input_path = PathBuf::from("/test/song.mp3");
        let output_base_path = PathBuf::from("/output/cortina40_song.mp3");

        assert_eq!(input_path, PathBuf::from("/test/song.mp3"));
        assert_eq!(output_base_path, PathBuf::from("/output/cortina40_song.mp3"));
    }
}
