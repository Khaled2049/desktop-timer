use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub fn dismiss_break(app: tauri::AppHandle) -> Result<(), String> {
    let mut errors = Vec::new();
    for (label, window) in app.webview_windows() {
        if label.starts_with("break-") {
            if let Err(error) = window.close() {
                errors.push(error.to_string());
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub fn show_break(app: &tauri::AppHandle) -> tauri::Result<()> {
    let monitors = app.available_monitors()?;
    for (index, monitor) in monitors.iter().enumerate() {
        let label = format!("break-{index}");
        if let Some(window) = app.get_webview_window(&label) {
            window.show()?;
            continue;
        }
        let size = monitor.size().to_logical::<f64>(monitor.scale_factor());
        let window =
            WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html#break".into()))
                .title("Time for a break")
                .inner_size(size.width, size.height)
                .decorations(false)
                .resizable(false)
                .maximizable(false)
                .minimizable(false)
                .shadow(false)
                .transparent(true)
                .always_on_top(true)
                .visible_on_all_workspaces(true)
                .skip_taskbar(true)
                .visible(false)
                .effects(WindowEffectsConfig {
                    effects: vec![Effect::FullScreenUI],
                    state: Some(EffectState::Active),
                    ..Default::default()
                })
                .on_page_load(|window, payload| {
                    if payload.event() == tauri::webview::PageLoadEvent::Finished {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                })
                .build()?;
        #[cfg(target_os = "macos")]
        {
            use objc2_app_kit::{NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior};
            // This runs on the main thread; Tauri owns the NSWindow for this window's lifetime.
            let native = unsafe { &*(window.ns_window()? as *const NSWindow) };
            native.setLevel(NSScreenSaverWindowLevel);
            native.setCollectionBehavior(
                (native.collectionBehavior() & !NSWindowCollectionBehavior::FullScreenPrimary)
                    | NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            );
        }
        window.set_position(*monitor.position())?;
        window.set_size(*monitor.size())?;
    }
    Ok(())
}
