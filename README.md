# AutoClicker

A cross-platform autoclicker application built with Tauri and Rust.

## Features

- **Window Selection**: Choose which application window to target with auto-clicks
- **Adjustable Click Speed**: Set clicks per second from 1 to 50 (default: 10)
- **Trigger Key Selection**: Choose between Alt, Shift, or Ctrl as the activation key
- **Safe Operation**: Auto-clicks only activate when:
  - The trigger key is held down
  - The selected window is in focus

## How to Use

1. **Launch the application**
2. **Click "Refresh Windows"** to get a list of open windows
3. **Select a target window** from the dropdown
4. **Choose your trigger key** (Alt, Shift, or Ctrl)
5. **Set clicks per second** using the slider or number input
6. **Click "Start"** to activate the autoclicker
7. **Switch to your target window** and hold the trigger key to start clicking
8. **Release the trigger key** to pause clicking
9. **Click "Stop"** to deactivate the autoclicker

## Building

### Prerequisites

- Rust (latest stable version)
- Node.js and npm (for Tauri CLI)

### Build Instructions

1. Install Tauri CLI:
   ```bash
   cargo install tauri-cli --version "^2.0.0"
   ```

2. Build the application:
   ```bash
   cargo tauri build
   ```

### Development Mode

To run in development mode:

```bash
cargo tauri dev
```

## Configuration

The maximum CPS value (currently 50) can be changed in the code:

- **Frontend**: `ui/index.html` and `ui/script.js` - change the `max="50"` attribute
- **Backend**: No changes needed, it accepts any CPS value from the frontend

## Technical Details

- **Frontend**: HTML, CSS, JavaScript
- **Backend**: Rust with Tauri
- **Mouse Control**: enigo library
- **Keyboard Events**: rdev library
- **Window Detection**: Windows Win32 API (Windows only)

## Platform Support

- **Windows**: Full support with window selection
- **Linux/Mac**: Basic support (window selection may not work)

## Safety Notes

- The autoclicker only works when the selected window is focused
- Holding the trigger key activates clicking, releasing it pauses
- Always test with safe applications first
- Some games and applications may detect and block autoclickers

## License

This project is provided as-is for educational purposes.
