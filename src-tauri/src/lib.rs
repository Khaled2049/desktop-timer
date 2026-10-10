mod break_reminder;
mod timer;
mod tray_icon;

use std::{
    sync::Mutex,
    time::{Duration, SystemTime},
};
use tauri::{
    Emitter, Manager,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use timer::{Snapshot, Timer};

#[tauri::command]
fn get_timer(state: tauri::State<'_, Mutex<Timer>>) -> Result<Snapshot, String> {
    Ok(state
        .lock()
        .map_err(|_| "Timer unavailable")?
        .snapshot(SystemTime::now()))
}

#[tauri::command]
fn timer_action(
    action: String,
    seconds: Option<u64>,
    state: tauri::State<'_, Mutex<Timer>>,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|_| "Timer unavailable")?
        .action(&action, seconds, SystemTime::now())
}

fn show_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(Timer::default()))
        .invoke_handler(tauri::generate_handler![get_timer, timer_action, break_reminder::dismiss_break])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyS);
            let shortcut_down = std::sync::atomic::AtomicBool::new(false);
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, pressed_shortcut, event| {
                        if pressed_shortcut != &shortcut {
                            return;
                        }
                        match event.state() {
                            ShortcutState::Pressed => {
                                if !shortcut_down.swap(true, std::sync::atomic::Ordering::Relaxed) {
                                    let state = app.state::<Mutex<Timer>>();
                                    if let Ok(mut timer) = state.lock() {
                                        let _ = timer.action("toggle", None, SystemTime::now());
                                    };
                                }
                            }
                            ShortcutState::Released => {
                                shortcut_down.store(false, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    })
                    .build(),
            )?;
            app.global_shortcut().register(shortcut).map_err(|error| {
                std::io::Error::other(format!("Could not register Control+S: {error}"))
            })?;

            let toggle = MenuItem::with_id(app, "toggle", "Start", true, None::<&str>)?;
            let pomodoro_mode = CheckMenuItem::with_id(app, "toggle_pomodoro", "Pomodoro mode", true, false, None::<&str>)?;
            let reset = MenuItem::with_id(app, "reset", "Reset", true, None::<&str>)?;
            let five = MenuItem::with_id(app, "preset_5", "Start 5 minutes", true, None::<&str>)?;
            let fifteen =
                MenuItem::with_id(app, "preset_15", "Start 15 minutes", true, None::<&str>)?;
            let twenty_five =
                MenuItem::with_id(app, "preset_25", "Start 25 minutes", true, None::<&str>)?;
            let settings =
                MenuItem::with_id(app, "settings", "Timer settings…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Desktop Timer", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let separator2 = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(
                app,
                &[
                    &toggle,
                    &reset,
                    &pomodoro_mode,
                    &separator,
                    &five,
                    &fifteen,
                    &twenty_five,
                    &separator2,
                    &settings,
                    &quit,
                ],
            )?;
            let tray = TrayIconBuilder::with_id("timer")
                .title("25:00")
                .tooltip("Desktop Timer")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    if id == "quit" {
                        app.exit(0);
                        return;
                    }
                    if id == "settings" {
                        show_window(app);
                        return;
                    }
                    let state = app.state::<Mutex<Timer>>();
                    if let Ok(mut timer) = state.lock() {
                        let now = SystemTime::now();
                        let action = id;
                        if let Some(minutes) = id
                            .strip_prefix("preset_")
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            let _ = timer.action("set_duration", Some(minutes * 60), now);
                            let _ = timer.action("start", None, now);
                        } else {
                            let _ = timer.action(action, None, now);
                        }
                    };
                })
                .build(app)?;
            if let Err(error) = tray_icon::update(&tray, "clock") {
                eprintln!("Could not set menu bar icon: {error}");
            }

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut previous: Option<Snapshot> = None;
                loop {
                    let app = handle.clone();
                    let toggle = toggle.clone();
                    let pomodoro_mode = pomodoro_mode.clone();
                    // Wait for each UI update so queued ticks cannot pile up during sleep.
                    let (done_tx, done_rx) = std::sync::mpsc::channel();
                    let previous_snapshot = previous.clone();
                    if handle
                        .run_on_main_thread(move || {
                            let state = app.state::<Mutex<Timer>>();
                            let (snapshot, finished) = match state.lock() {
                                Ok(mut timer) => {
                                    let now = SystemTime::now();
                                    let finished = timer.tick(now);
                                    (timer.snapshot(now), finished)
                                }
                                Err(_) => return,
                            };
                            if previous_snapshot.as_ref().is_some_and(|previous| previous.status == "finished")
                                && snapshot.status != "finished"
                            {
                                if let Err(error) = break_reminder::dismiss_break(app.clone()) {
                                    eprintln!("Could not dismiss break reminder: {error}");
                                }
                            }
                            if previous_snapshot.as_ref() != Some(&snapshot) {
                                let title = format!(
                                    "{:02}:{:02}",
                                    snapshot.remaining_seconds / 60,
                                    snapshot.remaining_seconds % 60
                                );
                                if let Some(tray) = app.tray_by_id("timer") {
                                    if let Err(error) = tray.set_title(Some(&title)) {
                                        eprintln!("Tray update failed: {error}");
                                    }
                                    if previous_snapshot.as_ref().is_none_or(|previous| previous.status != snapshot.status || previous.phase != snapshot.phase || previous.mode != snapshot.mode) {
                                        let symbol = match snapshot.status {
                                            "paused" => "pause.circle",
                                            "finished" => "checkmark.circle",
                                            _ if snapshot.mode == "pomodoro" && snapshot.phase != "focus" => "cup.and.saucer",
                                            _ => "clock",
                                        };
                                        if let Err(error) = tray_icon::update(&tray, symbol) {
                                            eprintln!("Tray icon update failed: {error}");
                                        }
                                    }
                                }
                                let label = match snapshot.status {
                                    "running" => "Pause",
                                    "paused" => "Resume",
                                    "finished" => "Start again",
                                    _ => "Start",
                                };
                                let label = if snapshot.mode == "pomodoro" && snapshot.status == "finished" {
                                    match snapshot.next_phase {
                                        "short_break" => "Start short break",
                                        "long_break" => "Start long break",
                                        _ => "Start focus",
                                    }
                                } else { label };
                                let _ = pomodoro_mode.set_checked(snapshot.mode == "pomodoro");
                                let _ = toggle.set_text(label);
                                let _ = app.emit("timer-updated", &snapshot);
                            }
                            if finished {
                                if let Err(error) = break_reminder::show_break(&app) {
                                    eprintln!("Could not show break reminder: {error}");
                                    show_window(&app);
                                    let _ = app.emit("break-reminder-error", "Could not open the break reminder. Time to move and look around.");
                                }
                                #[cfg(target_os = "macos")]
                                std::thread::spawn(|| {
                                    let _ = std::process::Command::new("/usr/bin/afplay")
                                        .arg("/System/Library/Sounds/Glass.aiff")
                                        .status();
                                });
                            }
                            let _ = done_tx.send(snapshot);
                        })
                        .is_err()
                    {
                        break;
                    }
                    match done_rx.recv() {
                        Ok(snapshot) => previous = Some(snapshot),
                        Err(_) => break,
                    }
                    std::thread::sleep(Duration::from_millis(250));
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Desktop Timer");
}
