use super::PlatformOps;

/// Linux platform implementation (X11/Wayland)
pub struct LinuxPlatform;

impl PlatformOps for LinuxPlatform {
    fn get_windows() -> Result<Vec<String>, String> {
        #[cfg(target_os = "linux")]
        {
            use x11rb::connection::Connection;
            use x11rb::protocol::xproto::*;
            use x11rb::rust_connection::RustConnection;

            let (conn, screen_num) = match RustConnection::connect(None) {
                Ok(result) => result,
                Err(_) => {
                    return Err(
                        "Failed to connect to X11 server. Wayland is not yet supported.".to_string()
                    )
                }
            };

            let screen = &conn.setup().roots[screen_num];
            let root = screen.root;

            // Get _NET_CLIENT_LIST atom
            let net_client_list = conn
                .intern_atom(false, b"_NET_CLIENT_LIST")
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|reply| reply.atom);

            let net_wm_name = conn
                .intern_atom(false, b"_NET_WM_NAME")
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|reply| reply.atom);

            let utf8_string = conn
                .intern_atom(false, b"UTF8_STRING")
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|reply| reply.atom);

            if let (Some(client_list_atom), Some(wm_name_atom), Some(utf8_atom)) =
                (net_client_list, net_wm_name, utf8_string)
            {
                // Get the client list
                if let Ok(reply) = conn
                    .get_property(false, root, client_list_atom, AtomEnum::WINDOW, 0, u32::MAX)
                    .ok()
                    .and_then(|cookie| cookie.reply().ok())
                {
                    let windows_bytes = reply.value;
                    let window_ids: Vec<Window> = windows_bytes
                        .chunks_exact(4)
                        .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect();

                    let mut window_titles = Vec::new();

                    for &window_id in &window_ids {
                        if let Ok(title_reply) = conn
                            .get_property(false, window_id, wm_name_atom, utf8_atom, 0, 1024)
                            .ok()
                            .and_then(|cookie| cookie.reply().ok())
                        {
                            if let Ok(title) = String::from_utf8(title_reply.value) {
                                if !title.is_empty() {
                                    window_titles.push(title);
                                }
                            }
                        }
                    }

                    return Ok(window_titles);
                }
            }

            Err("Failed to enumerate X11 windows".to_string())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err("Linux platform code called on non-Linux system".to_string())
        }
    }

    fn is_window_focused(target_title: &str) -> bool {
        #[cfg(target_os = "linux")]
        {
            use x11rb::connection::Connection;
            use x11rb::protocol::xproto::*;
            use x11rb::rust_connection::RustConnection;

            let (conn, screen_num) = match RustConnection::connect(None) {
                Ok(result) => result,
                Err(_) => return false,
            };

            let screen = &conn.setup().roots[screen_num];
            let root = screen.root;

            // Get the active window
            let net_active_window = conn
                .intern_atom(false, b"_NET_ACTIVE_WINDOW")
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|reply| reply.atom);

            let net_wm_name = conn
                .intern_atom(false, b"_NET_WM_NAME")
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|reply| reply.atom);

            let utf8_string = conn
                .intern_atom(false, b"UTF8_STRING")
                .ok()
                .and_then(|cookie| cookie.reply().ok())
                .map(|reply| reply.atom);

            if let (Some(active_atom), Some(wm_name_atom), Some(utf8_atom)) =
                (net_active_window, net_wm_name, utf8_string)
            {
                if let Ok(active_reply) = conn
                    .get_property(false, root, active_atom, AtomEnum::WINDOW, 0, 1)
                    .ok()
                    .and_then(|cookie| cookie.reply().ok())
                {
                    if active_reply.value.len() >= 4 {
                        let active_window = u32::from_ne_bytes([
                            active_reply.value[0],
                            active_reply.value[1],
                            active_reply.value[2],
                            active_reply.value[3],
                        ]);

                        if let Ok(title_reply) = conn
                            .get_property(false, active_window, wm_name_atom, utf8_atom, 0, 1024)
                            .ok()
                            .and_then(|cookie| cookie.reply().ok())
                        {
                            if let Ok(title) = String::from_utf8(title_reply.value) {
                                return title.contains(target_title);
                            }
                        }
                    }
                }
            }

            false
        }

        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn is_cursor_in_window(target_title: &str) -> bool {
        #[cfg(target_os = "linux")]
        {
            use x11rb::connection::Connection;
            use x11rb::protocol::xproto::*;
            use x11rb::rust_connection::RustConnection;

            let (conn, screen_num) = match RustConnection::connect(None) {
                Ok(result) => result,
                Err(_) => return false,
            };

            let screen = &conn.setup().roots[screen_num];
            let root = screen.root;

            // Get cursor position
            if let Ok(pointer_reply) = conn.query_pointer(root).ok().and_then(|c| c.reply().ok()) {
                let cursor_x = pointer_reply.root_x as i32;
                let cursor_y = pointer_reply.root_y as i32;

                // Find target window
                let net_client_list = conn
                    .intern_atom(false, b"_NET_CLIENT_LIST")
                    .ok()
                    .and_then(|cookie| cookie.reply().ok())
                    .map(|reply| reply.atom);

                let net_wm_name = conn
                    .intern_atom(false, b"_NET_WM_NAME")
                    .ok()
                    .and_then(|cookie| cookie.reply().ok())
                    .map(|reply| reply.atom);

                let utf8_string = conn
                    .intern_atom(false, b"UTF8_STRING")
                    .ok()
                    .and_then(|cookie| cookie.reply().ok())
                    .map(|reply| reply.atom);

                if let (Some(client_list_atom), Some(wm_name_atom), Some(utf8_atom)) =
                    (net_client_list, net_wm_name, utf8_string)
                {
                    if let Ok(reply) = conn
                        .get_property(false, root, client_list_atom, AtomEnum::WINDOW, 0, u32::MAX)
                        .ok()
                        .and_then(|cookie| cookie.reply().ok())
                    {
                        let windows_bytes = reply.value;
                        let window_ids: Vec<Window> = windows_bytes
                            .chunks_exact(4)
                            .map(|chunk| {
                                u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
                            })
                            .collect();

                        for &window_id in &window_ids {
                            // Check window title
                            if let Ok(title_reply) = conn
                                .get_property(false, window_id, wm_name_atom, utf8_atom, 0, 1024)
                                .ok()
                                .and_then(|cookie| cookie.reply().ok())
                            {
                                if let Ok(title) = String::from_utf8(title_reply.value) {
                                    if title.contains(target_title) {
                                        // Check window geometry
                                        if let Ok(geom) = conn
                                            .get_geometry(window_id)
                                            .ok()
                                            .and_then(|c| c.reply().ok())
                                        {
                                            let x = geom.x as i32;
                                            let y = geom.y as i32;
                                            let width = geom.width as i32;
                                            let height = geom.height as i32;

                                            if cursor_x >= x
                                                && cursor_x <= x + width
                                                && cursor_y >= y
                                                && cursor_y <= y + height
                                            {
                                                return true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            false
        }

        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }
}
