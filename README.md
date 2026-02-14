# AutoClicker

A cross-platform autoclicker application built with Tauri 2 and Rust.

## Features

- **Window Selection**: Choose which application window to target with auto-clicks
- **Adjustable Click Speed**: Set clicks per second from 1 to 100 (default: 10)
- **Trigger Key Selection**: Choose between Alt, Shift, or Ctrl as the activation key
- **Safe Operation**: Auto-clicks only activate when:
  - The trigger key is held down
  - The selected window is in focus
  - The mouse cursor is inside the target window
- **Cross-Platform**: Full support for **Windows**, **macOS**, and **Linux**

## Platform Support

| Platform | Window Selection | Window Focus | Cursor Detection |
|----------|-----------------|--------------|------------------|
| Windows  | ✅ Full Support | ✅ Full Support | ✅ Full Support |
| macOS    | ✅ Full Support | ✅ Full Support | ⚠️ Basic (focus-based) |
| Linux    | ✅ Full Support (X11) | ✅ Full Support (X11) | ✅ Full Support (X11) |

**Note**: Linux support requires X11. Wayland support is not yet implemented.

## Code Architecture

The project follows clean code practices with platform-specific code isolated in separate modules:

```
src/
├── lib.rs                    # Main application logic (platform-agnostic)
├── main.rs                   # Entry point
└── platform/
    ├── mod.rs                # Platform trait definition
    ├── windows.rs            # Windows Win32 API implementation
    ├── macos.rs              # macOS Cocoa/Core Graphics implementation
    ├── linux.rs              # Linux X11 implementation
    └── unsupported.rs        # Fallback for unsupported platforms
```

### Platform Abstraction

All platform-specific functionality is defined through the `PlatformOps` trait:

```rust
pub trait PlatformOps {
    fn get_windows() -> Result<Vec<String>, String>;
    fn is_window_focused(target_title: &str) -> bool;
    fn is_cursor_in_window(target_title: &str) -> bool;
}
```

Each platform implementation is automatically selected at compile-time using Rust's `cfg` attributes.

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
- Platform-specific requirements:
  - **Windows**: Windows SDK
  - **macOS**: Xcode Command Line Tools
  - **Linux**: X11 development libraries (`libx11-dev`, `libxcb1-dev`)

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

The maximum CPS value (currently 100) can be changed:

- **Frontend**: `ui/index.html` - change the `max="100"` attribute on the slider
- **Backend**: `src/lib.rs` - change the `MAX_CPS` constant

## Technical Details

- **Frontend**: HTML, CSS, JavaScript
- **Backend**: Rust with Tauri
- **Mouse Control**: enigo library (cross-platform)
- **Keyboard Events**: rdev library (cross-platform)
- **Window Detection**:
  - Windows: Win32 API
  - macOS: Cocoa/Core Graphics
  - Linux: X11 (x11rb library)

## Dependencies

### Core Dependencies
- `tauri`: Application framework
- `enigo`: Cross-platform mouse/keyboard control
- `rdev`: Keyboard event detection

### Platform-Specific Dependencies
- **Windows**: `windows` crate (Win32 API)
- **macOS**: `cocoa`, `objc`, `core-graphics`
- **Linux**: `x11rb` (X11 protocol)

## Safety Notes

- The autoclicker only works when the selected window is focused
- Holding the trigger key activates clicking, releasing it pauses
- Always test with safe applications first
- Some games and applications may detect and block autoclickers

## Development

This software was created with the assistance of AI.

## License

This project is provided as-is for educational purposes.
