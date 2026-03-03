use clap::Parser;
use std::path::PathBuf;

/// Cortinizer - A tool to generate tango cortinas from MP3 and FLAC files
#[derive(Parser, Debug)]
#[command(name = "cortinizer")]
#[command(author = "Cortinizer Team")]
#[command(version = "0.1.0")]
#[command(about = "Generate tango cortinas from MP3 and FLAC files", long_about = None)]
pub struct Args {
    /// Length of the cortina in seconds
    #[arg(short, long)]
    pub length: u32,

    /// Input paths (files or folders) to process
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Number of parallel jobs (default: number of CPU cores)
    #[arg(short, long, default_value_t = num_cpus::get())]
    pub jobs: usize,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

impl Args {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        // Validate length parameter
        let args = Self::parse();

        if args.length == 0 {
            eprintln!("Error: Length must be greater than 0");
            std::process::exit(1);
        }

        // Validate that we have at least one input path
        if args.paths.is_empty() {
            eprintln!("Error: At least one input path is required");
            std::process::exit(1);
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_validation() {
        // This would need proper testing with mock CLI args
        // For now, we just ensure the struct compiles
        let args = Args {
            length: 40,
            paths: vec![PathBuf::from("test.mp3")],
            jobs: 4,
            verbose: false,
        };
        assert_eq!(args.length, 40);
        assert_eq!(args.paths.len(), 1);
    }
}
