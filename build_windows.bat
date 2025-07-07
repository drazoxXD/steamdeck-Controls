@echo off

REM Build script for Windows Controller Host

echo Building Windows Controller Host...

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust/Cargo is not installed
    echo Please install Rust from https://rustup.rs/
    exit /b 1
)

REM Navigate to windows directory
cd windows

REM Build the project
echo Building in release mode...
cargo build --release

if %errorlevel% equ 0 (
    echo Build successful!
    echo Run the host with: cargo run
    echo Or directly: .\target\release\windows.exe
) else (
    echo Build failed!
    exit /b 1
)
