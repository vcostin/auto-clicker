/// Platform-specific trait for window management and detection
pub trait PlatformOps {
    /// Get a list of all visible window titles
    fn get_windows() -> Result<Vec<String>, String>;

    /// Check if a window with the given title is currently focused
    fn is_window_focused(target_title: &str) -> bool;

    /// Check if the cursor is inside the bounds of a window with the given title
    fn is_cursor_in_window(target_title: &str) -> bool;
}

// Import the correct platform implementation based on the target OS
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform as Platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::MacOSPlatform as Platform;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxPlatform as Platform;

// Fallback for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod unsupported;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use unsupported::UnsupportedPlatform as Platform;
