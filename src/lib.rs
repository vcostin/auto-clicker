use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindowVisible,
};

#[derive(Clone, serde::Serialize)]
struct AutoClickerState {
    window_title: String,
    cps: u32,
    trigger_key: String,
}

struct AppState {
    running: Arc<AtomicBool>,
    state: Arc<Mutex<Option<AutoClickerState>>>,
}

#[cfg(windows)]
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

#[tauri::command]
fn get_windows() -> Result<Vec<String>, String> {
    #[cfg(windows)]
    {
        let mut windows: Vec<String> = Vec::new();
        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut windows as *mut _ as isize),
            );
        }
        Ok(windows)
    }

    #[cfg(not(windows))]
    {
        Err("Window enumeration is only supported on Windows".to_string())
    }
}

#[tauri::command]
fn start_autoclicker(
    state: tauri::State<AppState>,
    window_title: String,
    cps: u32,
    trigger_key: String,
) -> Result<(), String> {
    if state.running.load(Ordering::SeqCst) {
        return Err("AutoClicker is already running".to_string());
    }

    let clicker_state = AutoClickerState {
        window_title: window_title.clone(),
        cps,
        trigger_key: trigger_key.clone(),
    };

    *state.state.lock().unwrap() = Some(clicker_state);
    state.running.store(true, Ordering::SeqCst);

    let running = state.running.clone();
    let state_mutex = state.state.clone();

    thread::spawn(move || {
        autoclicker_loop(running, state_mutex);
    });

    Ok(())
}

#[tauri::command]
fn stop_autoclicker(state: tauri::State<AppState>) -> Result<(), String> {
    state.running.store(false, Ordering::SeqCst);
    *state.state.lock().unwrap() = None;
    Ok(())
}

fn autoclicker_loop(running: Arc<AtomicBool>, state: Arc<Mutex<Option<AutoClickerState>>>) {
    use enigo::{Button, Enigo, Mouse, Settings};
    use rdev::{listen, Event, EventType, Key};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();

    let key_tx = tx.clone();
    thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            if let Event {
                event_type: EventType::KeyPress(_key) | EventType::KeyRelease(_key),
                ..
            } = event
            {
                let _ = key_tx.send(event);
            }
        }) {
            eprintln!("Error listening to keyboard events: {:?}", error);
        }
    });

    let mut key_held = false;
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    while running.load(Ordering::SeqCst) {
        // Process ALL pending keyboard events before deciding to click
        loop {
            match rx.try_recv() {
                Ok(event) => {
                    let state_guard = state.lock().unwrap();
                    if let Some(clicker_state) = &*state_guard {
                        let target_key = match clicker_state.trigger_key.as_str() {
                            "Alt" => Some(Key::Alt),
                            "Shift" => Some(Key::ShiftLeft),
                            "Ctrl" => Some(Key::ControlLeft),
                            _ => None,
                        };

                        if let Some(target) = target_key {
                            match event.event_type {
                                EventType::KeyPress(_key)
                                    if matches!(
                                        _key,
                                        Key::Alt | Key::ShiftLeft | Key::ShiftRight | Key::ControlLeft | Key::ControlRight
                                    ) && is_key_match(_key, target) =>
                                {
                                    key_held = true;
                                }
                                EventType::KeyRelease(_key)
                                    if matches!(
                                        _key,
                                        Key::Alt | Key::ShiftLeft | Key::ShiftRight | Key::ControlLeft | Key::ControlRight
                                    ) && is_key_match(_key, target) =>
                                {
                                    key_held = false;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Err(_) => break, // No more events to process
            }
        }

        if key_held {
            let state_guard = state.lock().unwrap();
            if let Some(clicker_state) = &*state_guard {
                #[cfg(windows)]
                {
                    if is_window_focused(&clicker_state.window_title)
                        && is_cursor_in_window(&clicker_state.window_title)
                    {
                        let _ = enigo.button(Button::Left, enigo::Direction::Click);
                    }
                }

                #[cfg(not(windows))]
                {
                    let _ = enigo.button(Button::Left, enigo::Direction::Click);
                }

                let delay = Duration::from_millis(1000 / clicker_state.cps as u64);
                thread::sleep(delay);
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }
}

fn is_key_match(key: rdev::Key, target: rdev::Key) -> bool {
    use rdev::Key;
    match target {
        Key::Alt => matches!(key, Key::Alt),
        Key::ShiftLeft => matches!(key, Key::ShiftLeft | Key::ShiftRight),
        Key::ControlLeft => matches!(key, Key::ControlLeft | Key::ControlRight),
        _ => false,
    }
}

#[cfg(windows)]
fn is_window_focused(target_title: &str) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

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

#[cfg(windows)]
fn is_cursor_in_window(target_title: &str) -> bool {
    use windows::Win32::Foundation::{LPARAM, POINT};
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

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

#[cfg(windows)]
unsafe extern "system" fn enum_windows_cursor_check(
    hwnd: HWND,
    lparam: LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::Foundation::{POINT, RECT};
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, GetWindowTextW};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            get_windows,
            start_autoclicker,
            stop_autoclicker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
