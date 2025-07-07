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

# Build the project
echo "Building in release mode..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Run the client with: cargo run"
    echo "Or directly: ./target/release/steamdeck"
else
    echo "Build failed!"
    exit 1
fi
