/// Placeholder command — will be replaced by auth/keychain commands in Phase 1.
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! ex desktop is running.", name)
}
