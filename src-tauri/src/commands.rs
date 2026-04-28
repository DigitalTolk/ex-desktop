use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

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
