#!/bin/bash

# Build script for SteamDeck Controller Client

echo "Building SteamDeck Controller Client..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Navigate to steamdeck directory
cd steamdeck

# Ask user for build type
echo "Select build type:"
echo "1) GUI version (with ImGui interface)"
echo "2) Console version (text-only, no GUI dependencies)"
echo "3) Both versions"
read -p "Enter your choice (1-3): " choice

case $choice in
    1)
        echo "Building GUI version..."
        cargo build --release --features gui
        if [ $? -eq 0 ]; then
            echo "GUI version built successfully!"
            echo "Run with: cargo run --features gui"
            echo "Or directly: ./target/release/steamdeck"
        fi
        ;;
    2)
        echo "Building console version..."
        cargo build --release --no-default-features --bin steamdeck-console
        if [ $? -eq 0 ]; then
            echo "Console version built successfully!"
            echo "Run with: cargo run --no-default-features --bin steamdeck-console"
            echo "Or directly: ./target/release/steamdeck-console"
        fi
        ;;
    3)
        echo "Building both versions..."
        cargo build --release --features gui
        cargo build --release --no-default-features --bin steamdeck-console
        if [ $? -eq 0 ]; then
            echo "Both versions built successfully!"
            echo "GUI version: ./target/release/steamdeck"
            echo "Console version: ./target/release/steamdeck-console"
        fi
        ;;
    *)
        echo "Invalid choice. Building GUI version by default..."
        cargo build --release --features gui
        ;;
esac

if [ $? -ne 0 ]; then
    echo "Build failed!"
    echo ""
    echo "If you encounter GUI-related errors on SteamDeck, try:"
    echo "  cargo build --release --no-default-features --bin steamdeck-console"
    exit 1
fi
