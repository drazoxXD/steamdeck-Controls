# SteamDeck Controller - Client

This is the SteamDeck client application that reads controller input and sends it to the Windows host.

## Features

- Lists all available controllers on the SteamDeck
- Reads controller input using Steam Input API
- Sends controller data over USB/Network to Windows host
- Real-time ImGui debug console showing button presses
- Supports both USB and local network communication

## Building

### Prerequisites

- Rust (latest stable)
- SteamOS/Arch Linux environment
- Controller connected to SteamDeck

### Build Steps

```bash
# Navigate to steamdeck directory
cd steamdeck

# Build the project
cargo build --release

# Run the application
cargo run
```

## Usage

1. Connect your controllers to the SteamDeck
2. Run the application: `cargo run`
3. The UI will show:
   - List of detected controllers
   - Real-time controller state
   - Debug information for button presses
4. The application will listen for connections from Windows host

## Configuration

The application uses the following default settings:
- Network port: 12345
- USB Vendor ID: 0x1234
- USB Product ID: 0x5678

## Network Communication

The client can communicate with the Windows host via:
- **Local Network**: Listens on port 12345
- **USB**: (Future implementation)

## Steam Input

The application uses the `gilrs` library to access controller input. This works with:
- Steam Controller
- Xbox controllers
- PlayStation controllers
- Any controller supported by Steam Input

## Debug Console

The ImGui interface shows:
- Available controllers list
- Real-time button states
- Analog stick positions
- Trigger values
- Timestamp information

## Troubleshooting

### No Controllers Detected
- Ensure controllers are properly connected
- Check that Steam recognizes the controllers
- Verify controller permissions in SteamOS

### Network Connection Issues
- Check firewall settings
- Ensure both devices are on the same network
- Verify port 12345 is available

### Performance Issues
- The application runs at 60 FPS by default
- Network updates are sent at 60 Hz
- Consider adjusting update rates for better performance
