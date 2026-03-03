@echo off
REM Cortinizer - Generate 50-second tango cortinas
REM Drag and drop audio files or folders onto this file

"%~dp0cortinizer.exe" --length 50 %*

IF ERRORLEVEL 1 (
    echo.
    echo Error: Cortinizer failed to process the files.
    echo.
    echo Possible issues:
    echo   - FFmpeg is not installed or not in PATH
    echo   - Invalid audio file format (only MP3 and FLAC supported)
    echo   - cortinizer.exe is not in the same directory as this batch file
    echo.
) ELSE (
    echo.
    echo Success! All files processed.
    echo.
)

pause
