mod audio_processor;
mod cli;
mod config;
mod error;
mod file_handler;
mod ffmpeg;
mod metadata;
mod output_manager;

use cli::Args;
use error::ProcessStatus;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    // Parse command-line arguments
    let args = Args::parse_args();

    // Initialize tracing/logging
    let log_level = if args.verbose {
        "debug"
    } else {
        "info"
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();

    info!("Cortinizer v{} starting", env!("CARGO_PKG_VERSION"));
    info!("Cortina length: {} seconds", args.length);

    // Create configuration
    let config = match config::Config::new(args.length) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    info!("Program directory: {:?}", config.program_dir);
    info!("FFmpeg path: {:?}", config.ffmpeg_path);

    // Categorize inputs
    let input_items = match file_handler::categorize_inputs(&args.paths) {
        Ok(items) => items,
        Err(e) => {
            error!("Failed to categorize inputs: {}", e);
            std::process::exit(1);
        }
    };

    if input_items.is_empty() {
        error!("No valid audio files or folders to process");
        std::process::exit(1);
    }

    info!("Found {} input item(s)", input_items.len());

    // Process inputs
    let results = audio_processor::process_inputs(config, input_items).await;

    // Print summary
    print_summary(&results);

    // Determine exit code
    let has_failures = results.iter().any(|r| !r.is_success());

    if has_failures {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

/// Print a summary of processing results
fn print_summary(results: &[error::ProcessResult]) {
    let total = results.len();
    let successes = results.iter().filter(|r| r.is_success()).count();
    let failures = total - successes;

    info!("========================================");
    info!("Processing complete!");
    info!("Total: {} | Success: {} | Failed: {}", total, successes, failures);

    if failures > 0 {
        warn!("Failed files:");
        for result in results {
            if let ProcessStatus::Failed(ref error) = result.status {
                warn!("  {:?}: {}", result.input_path, error);
            }
        }
    }

    info!("========================================");
}
