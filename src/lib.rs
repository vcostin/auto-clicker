use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod platform;
use platform::{Platform, PlatformOps};

#[derive(Clone, serde::Serialize)]
struct AutoClickerState {
    window_title: String,
    cps: u32,
    trigger_key: String,
}

struct AppState {
    running: Arc<AtomicBool>,
    state: Arc<Mutex<Option<AutoClickerState>>>,
    last_command_time: Arc<Mutex<Option<Instant>>>,
}

// Constants for validation
const MAX_CPS: u32 = 100;
const MIN_CPS: u32 = 1;
const MAX_WINDOW_TITLE_LENGTH: usize = 256;
const RATE_LIMIT_MS: u128 = 500; // Minimum 500ms between commands

#[tauri::command]
fn get_windows() -> Result<Vec<String>, String> {
    Platform::get_windows()
}

#[tauri::command]
fn start_autoclicker(
    state: tauri::State<AppState>,
    window_title: String,
    cps: u32,
    trigger_key: String,
) -> Result<(), String> {
    // Rate limiting check
    {
        let mut last_time = state.last_command_time.lock().unwrap();
        if let Some(last) = *last_time {
            let elapsed = last.elapsed().as_millis();
            if elapsed < RATE_LIMIT_MS {
                return Err(format!(
                    "Rate limit: Please wait {}ms between commands",
                    RATE_LIMIT_MS - elapsed
                ));
            }
        }
        *last_time = Some(Instant::now());
    }

    // Check if already running
    if state.running.load(Ordering::SeqCst) {
        return Err("AutoClicker is already running".to_string());
    }

    // Input validation
    if cps < MIN_CPS || cps > MAX_CPS {
        return Err(format!(
            "CPS must be between {} and {}",
            MIN_CPS, MAX_CPS
        ));
    }

    if window_title.is_empty() {
        return Err("Window title cannot be empty".to_string());
    }

    if window_title.len() > MAX_WINDOW_TITLE_LENGTH {
        return Err(format!(
            "Window title too long (max {} characters)",
            MAX_WINDOW_TITLE_LENGTH
        ));
    }

    // Validate trigger key
    if !matches!(trigger_key.as_str(), "Alt" | "Shift" | "Ctrl") {
        return Err("Invalid trigger key. Must be Alt, Shift, or Ctrl".to_string());
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
                // Check platform-specific conditions
                if Platform::is_window_focused(&clicker_state.window_title)
                    && Platform::is_cursor_in_window(&clicker_state.window_title)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(None)),
            last_command_time: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            get_windows,
            start_autoclicker,
            stop_autoclicker
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
