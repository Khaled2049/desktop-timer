#[cfg(target_os = "macos")]
pub fn update(tray: &tauri::tray::TrayIcon, symbol: &'static str) -> Result<(), String> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSCellImagePosition, NSImage};
    use objc2_foundation::{NSSize, NSString};

    tray.with_inner_tray_icon(move |inner| {
        let main_thread =
            MainThreadMarker::new().ok_or("Menu bar update requires the main thread")?;
        let item = inner.ns_status_item().ok_or("Menu bar item unavailable")?;
        let button = item
            .button(main_thread)
            .ok_or("Menu bar button unavailable")?;
        let name = NSString::from_str(symbol);
        let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(&name, None)
            .ok_or("Menu bar symbol unavailable")?;
        image.setSize(NSSize::new(20.0, 20.0));
        image.setTemplate(true);
        button.setImage(Some(&image));
        button.setImagePosition(NSCellImagePosition::ImageLeft);
        Ok(())
    })
    .map_err(|error| error.to_string())?
}

#[cfg(not(target_os = "macos"))]
pub fn update(_tray: &tauri::tray::TrayIcon, _symbol: &'static str) -> Result<(), String> {
    Ok(())
}
