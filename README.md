# Cortinizer

A Windows-friendly drag-and-drop tool that generates tango cortinas from MP3 and FLAC files.

## Features

- **Drag-and-drop simplicity**: Drop audio files or folders on the executable to process
- **Format preservation**: MP3 files produce MP3 cortinas, FLAC files produce FLAC cortinas
- **Folder structure support**: Drop entire folders and preserve subdirectory structure
- **Collision handling**: Automatically adds numeric suffixes to prevent overwriting
- **Best-effort metadata**: Attempts to preserve audio tags (ID3 for MP3, Vorbis for FLAC)
- **Parallel processing**: Uses all CPU cores for fast batch processing
- **Configurable length**: Default 40 seconds, adjustable via command-line or batch scripts

## Installation

### Prerequisites

- **FFmpeg** must be available on your system
  - Download FFmpeg from [https://www.ffmpeg.org/download.html](https://www.ffmpeg.org/download.html)
  - **Windows**: Place `ffmpeg.exe` in the same directory as `cortinizer.exe`
  - **Linux/macOS**: Install FFmpeg through your package manager:

    ```bash
    # Ubuntu/Debian
    sudo apt install ffmpeg

    # macOS
    brew install ffmpeg
    ```

### Building from Source

```bash
cargo build --release
```

The built binary will be at `target/release/cortinizer` (or `cortinizer.exe` on Windows).

## Usage

### Command-Line Usage

```bash
# Process a single file
cortinizer --length 40 song.mp3

# Process multiple files
cortinizer --length 40 song1.mp3 song2.flac

# Process folders
cortinizer --length 40 ./music/songs

# Specify cortina length
cortinizer --length 30 song.mp3   # 30 second cortina
cortinizer --length 50 song.flac  # 50 second cortina

# Verbose logging
cortinizer --verbose --length 40 song.mp3
```

### Batch Scripts (Windows)

Copy the `.bat` files from `batch_scripts/` to your program directory for quick access:

- **cortinizer30.bat** - Generate 30-second cortinas
- **cortinizer40.bat** - Generate 40-second cortinas (default)
- **cortinizer50.bat** - Generate 50-second cortinas
- **cortinizer60.bat** - Generate 60-second cortinas

Drop files on these batch files instead of the main executable.

### Linux and macOS: Folder Processing

On Linux and macOS, we recommend using **folder processing** for convenience:

1. **Prepare** - Copy or move all audio files you want to process into a single folder
2. **Process** - Open a terminal and run:
   ```bash
   cortinizer --length 40 /path/to/your/folder
   ```

The tool will process all MP3 and FLAC files in the folder, preserving the subdirectory structure in the output.

**Example:**

```bash
# macOS/Linux - Process a folder of tango music
cortinizer --length 40 ~/Music/Tangos

# All cortinas will be created in the same directory as cortinizer:
# Tangos/cortina40_song1.mp3
# Tangos/cortina40_song2.flac
```

**Optional: Create shell aliases for convenience**

Add these to your `~/.bashrc` or `~/.zshrc`:

```bash
# Quick aliases for common cortina lengths
alias cortina30='cortinizer --length 30'
alias cortina40='cortinizer --length 40'
alias cortina50='cortinizer --length 50'
alias cortina60='cortinizer --length 60'
```

Then you can simply run:

```bash
cortina40 ~/Music/Tangos
```

## Output

### Location

All output files are created in the same directory as the `cortinizer` executable.

### Naming Convention

- **Single files**: `cortina{length}_{filename}.{ext}`
  - Example: `cortina40_song.mp3`
- **Folders**: `{folder_name}/cortina{length}_{filename}.{ext}`
  - Example: `MyTangos/cortina40_song.mp3`
  - Preserves subdirectory structure: `MyTangos/Subfolder/cortina40_song.mp3`

### Collision Handling

If an output file already exists, Cortinizer adds a numeric suffix:

- First attempt: `cortina40_song.mp3`
- If exists: `cortina40_song_1.mp3`
- If that exists: `cortina40_song_2.mp3`
- And so on...

The same collision handling applies to folder names when processing multiple folders.

## Audio Processing

### Fade-Out Effect

Cortinas include a fade-out effect during the last 5 seconds:

- **40-second cortina**: 35 seconds of full audio, 5-second fade-out
- **30-second cortina**: 25 seconds of full audio, 5-second fade-out

### Short Tracks

If a track is shorter than the target cortina length, Cortinizer will:

- Use the full track duration
- Apply fade-out for the last 5 seconds
- **Cortinas ≤5 seconds**: Fade starts immediately (duration = cortina length)

## Supported Formats

| Input Format | Output Format | Codec Used |
| ------------ | ------------- | ---------- |
| MP3          | MP3           | libmp3lame |
| FLAC         | FLAC          | flac       |

## Exit Codes

- **0**: All files processed successfully
- **1**: One or more files failed to process

## Examples

### Process a Single File

```
Input:  C:\Music\song.mp3 (3:45)
Output: C:\Cortinizer\cortina40_song.mp3 (0:40)
```

### Process a Folder

```
Input:  C:\Music\Tangos\
        └── D'Arienzo\song1.mp3
        └── Troilo\song2.flac

Output: C:\Cortinizer\Tangos\
        └── D'Arienzo\cortina40_song1.mp3
        └── Troilo\cortina40_song2.flac
```

### Run Multiple Times

```
First run:   cortina40_song.mp3
Second run:  cortina40_song_1.mp3
Third run:   cortina40_song_2.mp3
```

## Troubleshooting

### "Could not find FFmpeg"

- **Windows**: Place `ffmpeg.exe` in the same folder as `cortinizer.exe`
- **Linux/macOS**: Install FFmpeg using your package manager

### "No valid audio files or folders to process"

- Ensure you're dropping MP3 or FLAC files
- Check that files have `.mp3` or `.flac` extensions (case-insensitive)
- Verify files actually exist at the specified paths

### Processing Fails for a File

- Ensure the file is a valid, uncorrupted audio file
- Check file permissions (read access required)
- Try running with `--verbose` for detailed error messages

## License

This project is free for personal and open-source use. Commercial use is discouraged but not legally restricted.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
