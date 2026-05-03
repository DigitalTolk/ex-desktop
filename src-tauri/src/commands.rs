use keyring::Entry;
use tauri::{image::Image, AppHandle, Emitter};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

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

/// Payload emitted when OAuth completes successfully.
#[derive(serde::Serialize, Clone)]
pub struct OAuthComplete {
    pub token: String,
    pub user: serde_json::Value,
}

/// Starts a one-shot local HTTP server for the OAuth redirect callback.
/// Returns the port. When the browser hits /callback?token=..., the server
/// fetches /api/v1/users/me from `server_url` (bypassing browser CORS), then
/// emits `oauth-complete` with { token, user } and responds with a success page.
#[tauri::command]
pub async fn start_oauth_server(app: AppHandle, server_url: String) -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]);

            // First line: "GET /callback?token=...&refresh=... HTTP/1.1"
            let query_string = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|path| path.split('?').nth(1))
                .map(String::from)
                .unwrap_or_default();

            let get_param = |key: &str| -> Option<String> {
                query_string.split('&').find_map(|kv| {
                    let mut it = kv.splitn(2, '=');
                    if it.next()? == key { it.next().map(String::from) } else { None }
                })
            };

            let token = get_param("token");
            let refresh = get_param("refresh");

            // Store refresh token in keychain immediately so tryRestore() works on reload.
            if let Some(ref rt) = refresh {
                if let Ok(entry) = Entry::new(KEYRING_SERVICE, KEYRING_REFRESH_TOKEN) {
                    let _ = entry.set_password(rt);
                }
            }

            let result: Option<OAuthComplete> = if let Some(ref t) = token {
                let me_url = format!("{}/api/v1/users/me", server_url.trim_end_matches('/'));
                match reqwest::Client::new()
                    .get(&me_url)
                    .bearer_auth(t)
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() => {
                        resp.json::<serde_json::Value>().await.ok().map(|user| OAuthComplete {
                            token: t.clone(),
                            user,
                        })
                    }
                    Ok(resp) => {
                        log::warn!("start_oauth_server: /users/me returned {}", resp.status());
                        None
                    }
                    Err(e) => {
                        log::warn!("start_oauth_server: /users/me request failed: {e}");
                        None
                    }
                }
            } else {
                None
            };

            let html = if result.is_some() {
                "<html><head><title>Signed in</title></head><body>\
                 <h2>Sign-in successful — you can close this tab.</h2>\
                 <script>window.close()</script></body></html>"
            } else {
                "<html><body><h2>Sign-in failed — please try again.</h2></body></html>"
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.flush().await;

            if let Some(payload) = result {
                let _ = app.emit("oauth-complete", payload);
            }
        }
    });

    Ok(port)
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
