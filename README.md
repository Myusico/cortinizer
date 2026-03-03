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

### Basic Usage

Drop audio files or folders onto the `cortinizer.exe` executable.

### Command-Line Usage

```bash
# Process a single file
cortinizer --length 40 song.mp3

# Process multiple files
cortinizer --length 40 song1.mp3 song2.flac

# Process folders
cortinizer --length 40 ./music/tangos

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
- **Cortinas ≤5 seconds**: Fade starts immediately (duration = cortina length)

### Short Tracks

If a track is shorter than the target cortina length, Cortinizer will:
- Use the full track duration
- Apply fade-out only if the track is ≥5 seconds long

## Supported Formats

| Input Format | Output Format | Codec Used |
|-------------|---------------|------------|
| MP3         | MP3           | libmp3lame |
| FLAC        | FLAC          | flac       |

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

MIT License - See LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
