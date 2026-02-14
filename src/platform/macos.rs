use super::PlatformOps;

/// macOS platform implementation
pub struct MacOSPlatform;

impl PlatformOps for MacOSPlatform {
    fn get_windows() -> Result<Vec<String>, String> {
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::{NSRunningApplication, NSWorkspace};
            use cocoa::base::{id, nil};
            use cocoa::foundation::{NSArray, NSString};
            use objc::{msg_send, sel, sel_impl};

            unsafe {
                let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
                let running_apps: id = msg_send![workspace, runningApplications];
                let count: usize = msg_send![running_apps, count];

                let mut windows = Vec::new();

                for i in 0..count {
                    let app: id = msg_send![running_apps, objectAtIndex: i];
                    let localized_name: id = msg_send![app, localizedName];

                    if localized_name != nil {
                        let name_ptr: *const i8 = msg_send![localized_name, UTF8String];
                        if !name_ptr.is_null() {
                            let name = std::ffi::CStr::from_ptr(name_ptr)
                                .to_string_lossy()
                                .to_string();
                            if !name.is_empty() {
                                windows.push(name);
                            }
                        }
                    }
                }

                Ok(windows)
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err("macOS platform code called on non-macOS system".to_string())
        }
    }

    fn is_window_focused(target_title: &str) -> bool {
        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::NSWorkspace;
            use cocoa::base::id;
            use objc::{msg_send, sel, sel_impl};

            unsafe {
                let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
                let front_app: id = msg_send![workspace, frontmostApplication];

                if front_app.is_null() {
                    return false;
                }

                let localized_name: id = msg_send![front_app, localizedName];
                if localized_name.is_null() {
                    return false;
                }

                let name_ptr: *const i8 = msg_send![localized_name, UTF8String];
                if name_ptr.is_null() {
                    return false;
                }

                let name = std::ffi::CStr::from_ptr(name_ptr)
                    .to_string_lossy()
                    .to_string();

                name.contains(target_title)
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn is_cursor_in_window(target_title: &str) -> bool {
        #[cfg(target_os = "macos")]
        {
            use core_graphics::display::CGDisplay;
            use core_graphics::event::{CGEvent, CGEventType};
            use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

            // For simplicity, on macOS we'll just check if window is focused
            // Full implementation would require accessing window server API
            // which is more complex and requires additional frameworks
            Self::is_window_focused(target_title)
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}
