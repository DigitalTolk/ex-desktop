use keyring::Entry;
use tauri::{image::Image, AppHandle};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

const KEYRING_SERVICE: &str = "com.digitaltolk.ex";
const KEYRING_REFRESH_TOKEN: &str = "refresh_token";

const STORE_FILE: &str = "config.json";
const KEY_SERVER_URL: &str = "serverUrl";

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! ex desktop is running.", name)
}

/// Returns the configured server URL, or None on first launch.
#[tauri::command]
pub fn get_server_url(app: AppHandle) -> Option<String> {
    let store = app.store(STORE_FILE).ok()?;
    store
        .get(KEY_SERVER_URL)
        .and_then(|v| v.as_str().map(String::from))
}

/// Persists the server URL for future launches.
#[tauri::command]
pub fn set_server_url(app: AppHandle, url: String) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(KEY_SERVER_URL, serde_json::Value::String(url));
    store.save().map_err(|e| e.to_string())
}

/// Returns whether launch-at-login is currently enabled.
#[tauri::command]
pub fn get_autostart(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

/// Enables or disables launch-at-login.
#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let al = app.autolaunch();
    if enabled {
        al.enable().map_err(|e| e.to_string())
    } else {
        al.disable().map_err(|e| e.to_string())
    }
}

/// Reads the refresh token from the OS keychain.
#[tauri::command]
pub fn get_refresh_token() -> Option<String> {
    Entry::new(KEYRING_SERVICE, KEYRING_REFRESH_TOKEN)
        .ok()
        .and_then(|e| e.get_password().ok())
}

/// Stores the refresh token in the OS keychain.
#[tauri::command]
pub fn set_refresh_token(token: String) -> Result<(), String> {
    Entry::new(KEYRING_SERVICE, KEYRING_REFRESH_TOKEN)
        .map_err(|e| e.to_string())?
        .set_password(&token)
        .map_err(|e| e.to_string())
}

/// Deletes the refresh token from the OS keychain.
#[tauri::command]
pub fn delete_refresh_token() -> Result<(), String> {
    match Entry::new(KEYRING_SERVICE, KEYRING_REFRESH_TOKEN) {
        Ok(e) => e.delete_credential().map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Updates the tray icon and tooltip to reflect the unread message count.
#[tauri::command]
pub fn set_badge_count(app: AppHandle, count: u32) -> Result<(), String> {
    let Some(tray) = app.tray_by_id("main") else { return Ok(()) };
    if count > 0 {
        let icon = Image::from_bytes(include_bytes!("../icons/badge.png"))
            .map_err(|e| e.to_string())?;
        tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
        tray.set_tooltip(Some(&format!(
            "ex — {} unread message{}",
            count,
            if count == 1 { "" } else { "s" }
        )))
        .map_err(|e| e.to_string())?;
    } else {
        let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
            .map_err(|e| e.to_string())?;
        tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;
        tray.set_tooltip(Some("ex")).map_err(|e| e.to_string())?;
    }
    Ok(())
}
