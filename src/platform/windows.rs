use super::PlatformOps;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetCursorPos, GetForegroundWindow, GetWindowRect, GetWindowTextW, IsWindowVisible,
};

/// Windows platform implementation
pub struct WindowsPlatform;

impl PlatformOps for WindowsPlatform {
    fn get_windows() -> Result<Vec<String>, String> {
        let mut windows: Vec<String> = Vec::new();
        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut windows as *mut _ as isize),
            );
        }
        Ok(windows)
    }

    fn is_window_focused(target_title: &str) -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            let mut text = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut text);

            if len > 0 {
                let window_title = String::from_utf16_lossy(&text[..len as usize]);
                return window_title.contains(target_title);
            }
        }
        false
    }

    fn is_cursor_in_window(target_title: &str) -> bool {
        unsafe {
            // Get current cursor position
            let mut cursor_pos = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut cursor_pos).is_err() {
                return false;
            }

            // Find the target window and check if cursor is inside it
            let mut result = false;
            let target_title_clone = target_title.to_string();
            let mut data = (cursor_pos, target_title_clone, &mut result);
            let lparam = LPARAM(&mut data as *mut _ as isize);

            let _ = EnumWindows(Some(enum_windows_cursor_check), lparam);

            result
        }
    }
}

// Helper function to enumerate windows
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd).as_bool() {
        let mut text = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut text);

        if len > 0 {
            let window_title = String::from_utf16_lossy(&text[..len as usize]);
            if !window_title.is_empty() && window_title != "Program Manager" {
                let windows = &mut *(lparam.0 as *mut Vec<String>);
                windows.push(window_title);
            }
        }
    }
    BOOL(1)
}

// Helper function to check cursor position in window
unsafe extern "system" fn enum_windows_cursor_check(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let params = lparam.0 as *mut (POINT, String, *mut bool);
    let (cursor_pos, target_title, result_ptr) = &*params;

    let mut text = [0u16; 512];
    let len = GetWindowTextW(hwnd, &mut text);

    if len > 0 {
        let window_title = String::from_utf16_lossy(&text[..len as usize]);
        if window_title.contains(target_title) {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };

            if GetWindowRect(hwnd, &mut rect).is_ok() {
                // Check if cursor is inside window bounds
                if cursor_pos.x >= rect.left
                    && cursor_pos.x <= rect.right
                    && cursor_pos.y >= rect.top
                    && cursor_pos.y <= rect.bottom
                {
                    **result_ptr = true;
                    return BOOL(0); // Stop enumeration
                }
            }
        }
    }

    BOOL(1) // Continue enumeration
}
