use super::PlatformOps;

/// Unsupported platform implementation (fallback)
pub struct UnsupportedPlatform;

impl PlatformOps for UnsupportedPlatform {
    fn get_windows() -> Result<Vec<String>, String> {
        Err("Window enumeration is not supported on this platform".to_string())
    }

    fn is_window_focused(_target_title: &str) -> bool {
        false
    }

    fn is_cursor_in_window(_target_title: &str) -> bool {
        false
    }
}
