use std::path::PathBuf;
use tauri::webview::{DownloadEvent, WebviewWindowBuilder};
use tauri::{image::Image, webview::PageLoadEvent, AppHandle, Manager, WebviewUrl, WebviewWindow};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use url::Url;

use crate::config::normalize_server_url;

const STORE_FILE: &str = "config.json";
const KEY_SERVER_URL: &str = "serverUrl";
const LEGACY_KEY_REFRESH_TOKEN: &str = "refreshToken";

#[allow(clippy::useless_format)]
fn remote_main_init_script() -> Result<String, String> {
    Ok(format!(
        r#"
(() => {{
  const serverOrigin = globalThis.location.origin;
  const serverWsOrigin = serverOrigin.replace(/^https:/, 'wss:').replace(/^http:/, 'ws:');

  Object.defineProperty(globalThis, 'isTauri', {{
    value: false,
    writable: false,
    configurable: false
  }});

  let authRequiredShown = false;
  function showAuthRequired() {{
    if (authRequiredShown || document.getElementById('__ex-desktop-auth-required')) {{
      return;
    }}
    authRequiredShown = true;

    const panel = document.createElement('div');
    panel.id = '__ex-desktop-auth-required';
    panel.setAttribute('role', 'alert');
    Object.assign(panel.style, {{
      position: 'fixed',
      right: '16px',
      bottom: '16px',
      zIndex: '2147483647',
      maxWidth: '340px',
      padding: '16px',
      borderRadius: '18px',
      border: '1px solid rgba(15, 23, 42, 0.14)',
      background: 'rgba(255, 255, 255, 0.96)',
      color: '#0f172a',
      font: '13px/1.45 system-ui, sans-serif',
      boxShadow: '0 18px 50px rgba(15, 23, 42, 0.20)',
      backdropFilter: 'blur(14px)'
    }});

    const title = document.createElement('div');
    title.textContent = 'Sign in required';
    Object.assign(title.style, {{
      fontWeight: '700',
      marginBottom: '4px'
    }});

    const message = document.createElement('div');
    message.textContent = 'Your session expired. Sign in again to continue.';
    Object.assign(message.style, {{
      marginBottom: '12px',
      color: '#475569'
    }});

    const button = document.createElement('button');
    button.type = 'button';
    button.textContent = 'Sign in again';
    Object.assign(button.style, {{
      minHeight: '36px',
      border: '0',
      borderRadius: '12px',
      padding: '0 12px',
      background: '#0f172a',
      color: '#fff',
      font: '600 13px/1 system-ui, sans-serif',
      cursor: 'pointer'
    }});
    button.addEventListener('click', async () => {{
      button.disabled = true;
      button.textContent = 'Opening sign-in...';
      try {{
        await globalThis.__TAURI_INTERNALS__?.invoke('start_relogin');
      }} catch {{
        button.disabled = false;
        button.textContent = 'Sign in again';
      }}
    }});

    panel.append(title, message, button);

    const append = () => {{
      if (document.body && !document.getElementById(panel.id)) {{
        document.body.appendChild(panel);
      }}
    }};
    if (document.readyState === 'loading') {{
      document.addEventListener('DOMContentLoaded', append, {{ once: true }});
    }} else {{
      append();
    }}
  }}

  function rewriteHttpUrl(raw) {{
    try {{
      const value = String(raw);
      if (value.startsWith('/api/') || value.startsWith('/auth/')) {{
        return `${{serverOrigin}}${{value}}`;
      }}
      if (
        value.startsWith('http://localhost') ||
        value.startsWith('https://localhost') ||
        value.startsWith('http://127.0.0.1') ||
        value.startsWith('https://127.0.0.1')
      ) {{
        const parsed = new URL(value);
        return `${{serverOrigin}}${{parsed.pathname}}${{parsed.search}}${{parsed.hash}}`;
      }}
    }} catch {{}}
    return raw;
  }}

  function isServerPath(raw, expectedPath) {{
    try {{
      const parsed = new URL(String(raw), serverOrigin);
      return parsed.origin === serverOrigin && parsed.pathname === expectedPath;
    }} catch {{
      return false;
    }}
  }}

  function mountWorkspaceButton() {{
    const existing = document.getElementById('__ex-desktop-change-workspace');
    if (existing) {{
      return;
    }}

    const button = document.createElement('button');
    button.id = '__ex-desktop-change-workspace';
    button.type = 'button';
    button.textContent = 'Change workspace';
    button.setAttribute('aria-label', 'Change workspace');
    Object.assign(button.style, {{
      position: 'fixed',
      top: '12px',
      right: '12px',
      zIndex: '2147483647',
      padding: '7px 11px',
      borderRadius: '999px',
      border: '1px solid rgba(15, 23, 42, 0.14)',
      background: 'rgba(255, 255, 255, 0.92)',
      color: '#0f172a',
      font: '600 12px/1.2 system-ui, sans-serif',
      boxShadow: '0 10px 30px rgba(15, 23, 42, 0.18)',
      cursor: 'pointer',
      backdropFilter: 'blur(10px)'
    }});
    button.addEventListener('mouseenter', () => {{
      button.style.background = '#ffffff';
    }});
    button.addEventListener('mouseleave', () => {{
      button.style.background = 'rgba(255, 255, 255, 0.92)';
    }});
    button.addEventListener('click', async () => {{
      try {{
        await globalThis.__TAURI_INTERNALS__?.invoke('show_setup_window');
      }} catch {{
        // ignore
      }}
    }});

    const append = () => {{
      if (!document.body || document.getElementById(button.id)) {{
        return;
      }}
      document.body.appendChild(button);
    }};

    const ensureVisible = () => {{
      append();
    }};

    if (document.readyState === 'loading') {{
      document.addEventListener('DOMContentLoaded', ensureVisible, {{ once: true }});
    }} else {{
      ensureVisible();
    }}

    const observer = new MutationObserver(() => {{
      ensureVisible();
    }});

    const root = document.documentElement;
    if (root) {{
      observer.observe(root, {{ childList: true, subtree: true }});
    }}

    window.addEventListener(
      'pagehide',
      () => {{
        observer.disconnect();
      }},
      {{ once: true }}
    );
  }}

  mountWorkspaceButton();
  function rewriteWsUrl(raw) {{
    try {{
      const value = String(raw);
      if (
        value.startsWith('ws://localhost') ||
        value.startsWith('wss://localhost') ||
        value.startsWith('ws://127.0.0.1') ||
        value.startsWith('wss://127.0.0.1')
      ) {{
        const parsed = new URL(value);
        return `${{serverWsOrigin}}${{parsed.pathname}}${{parsed.search}}${{parsed.hash}}`;
      }}
    }} catch {{}}
    return raw;
  }}

  const nativeFetch = globalThis.fetch?.bind(globalThis);
  if (nativeFetch) {{
    globalThis.fetch = async (input, init) => {{
      const rawUrl = input instanceof Request ? input.url : input;
      const rewrittenUrl = rewriteHttpUrl(rawUrl);
      const isRefreshRequest = isServerPath(rewrittenUrl, '/auth/token/refresh');

      if (isRefreshRequest) {{
        const response = input instanceof Request
          ? await nativeFetch(
              rewrittenUrl !== input.url ? new Request(rewrittenUrl, input) : input,
              init
            )
          : await nativeFetch(rewrittenUrl, init);
        if (response.status === 401) {{
          showAuthRequired();
        }}
        return response;
      }}

      const response =
        input instanceof Request
          ? await nativeFetch(
              rewrittenUrl !== input.url ? new Request(rewrittenUrl, input) : input,
              init
            )
          : await nativeFetch(rewrittenUrl, init);

      return response;
    }};
  }}

  const NativeXMLHttpRequest = globalThis.XMLHttpRequest;
  if (NativeXMLHttpRequest) {{
    const nativeOpen = NativeXMLHttpRequest.prototype.open;
    NativeXMLHttpRequest.prototype.open = function(method, url, ...rest) {{
      return nativeOpen.call(this, method, rewriteHttpUrl(url), ...rest);
    }};
  }}

  const NativeWebSocket = globalThis.WebSocket;
  if (NativeWebSocket) {{
    class PatchedWebSocket extends NativeWebSocket {{
      constructor(url, protocols) {{
        super(rewriteWsUrl(url), protocols);
      }}
    }}

    Object.defineProperties(PatchedWebSocket, {{
      CONNECTING: {{ value: NativeWebSocket.CONNECTING }},
      OPEN: {{ value: NativeWebSocket.OPEN }},
      CLOSING: {{ value: NativeWebSocket.CLOSING }},
      CLOSED: {{ value: NativeWebSocket.CLOSED }}
    }});

    PatchedWebSocket.prototype = NativeWebSocket.prototype;
    Object.setPrototypeOf(PatchedWebSocket, NativeWebSocket);
    globalThis.WebSocket = PatchedWebSocket;
  }}
}})();
"#
    ))
}

pub(crate) fn configured_server_url(app: &AppHandle) -> Option<String> {
    let store = app.store(STORE_FILE).ok()?;
    if store.has(LEGACY_KEY_REFRESH_TOKEN) {
        store.delete(LEGACY_KEY_REFRESH_TOKEN);
        let _ = store.save();
    }
    let url = store
        .get(KEY_SERVER_URL)
        .and_then(|v| v.as_str().map(str::to_string))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())?;
    normalize_server_url(&url).ok()
}

fn store_server_url(app: &AppHandle, url: &str) -> Result<String, String> {
    let normalized = normalize_server_url(url)?;
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(
        KEY_SERVER_URL,
        serde_json::Value::String(normalized.clone()),
    );
    store.save().map_err(|e| e.to_string())?;
    Ok(normalized)
}

pub(crate) fn open_or_navigate_main_window(
    app: &AppHandle,
    server_url: &str,
) -> Result<(), String> {
    let parsed = Url::parse(server_url).map_err(|e| e.to_string())?;
    let init_script = remote_main_init_script()?;
    let app_handle = app.clone();
    let download_app_handle = app.clone();
    let data_dir = webview_data_dir(app)?;

    if let Some(main) = app.get_webview_window("main") {
        main.navigate(parsed).map_err(|e| e.to_string())?;
        let _ = main.show();
        let _ = main.set_focus();
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(parsed))
        .on_navigation(move |url| intercept_navigation(&app_handle, url))
        .on_download(move |_webview, event| match event {
            DownloadEvent::Requested { url, destination } => {
                if let Some(path) = default_download_path(&download_app_handle, &url) {
                    *destination = path;
                }
                true
            }
            DownloadEvent::Finished { url, path, success } => {
                if !success {
                    log::warn!("Download failed for {url}");
                } else if let Some(path) = path {
                    log::info!("Downloaded {url} to {}", path.display());
                }
                true
            }
            _ => true,
        })
        .on_page_load({
            let init_script = init_script.clone();
            move |webview, payload| {
                if payload.event() == PageLoadEvent::Finished {
                    let _ = webview.eval(init_script.clone());
                }
            }
        })
        .initialization_script(init_script)
        .disable_drag_drop_handler()
        .title("ex")
        .data_directory(data_dir)
        .inner_size(1280.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(icon) = app.default_window_icon() {
        let _ = window.set_icon(icon.clone());
    }

    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

fn webview_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("webview");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn is_attachment_download_url(url: &Url) -> bool {
    url.query_pairs().any(|(key, value)| {
        key.eq_ignore_ascii_case("response-content-disposition")
            && value.to_ascii_lowercase().contains("attachment")
    }) || (url
        .host_str()
        .is_some_and(|host| host.ends_with(".amazonaws.com"))
        && url.path().contains("/attachments/"))
}

fn default_download_path(app: &AppHandle, url: &Url) -> Option<PathBuf> {
    let download_dir = app.path().download_dir().ok()?;
    let filename = filename_for_download(url, None);
    Some(unique_download_path(download_dir, &filename))
}

fn open_attachment_download_url(app: &AppHandle, url: &Url) -> Result<(), String> {
    app.opener()
        .open_url(url.as_str(), None::<String>)
        .map_err(|e| e.to_string())
}

fn filename_for_download(url: &Url, content_disposition: Option<&str>) -> String {
    content_disposition
        .and_then(filename_from_content_disposition)
        .or_else(|| {
            url.query_pairs()
                .find(|(key, _)| key.eq_ignore_ascii_case("response-content-disposition"))
                .and_then(|(_, value)| filename_from_content_disposition(&value))
        })
        .or_else(|| {
            url.path_segments()
                .and_then(|mut segments| segments.rfind(|segment| !segment.is_empty()))
                .map(str::to_string)
        })
        .map(|filename| sanitize_filename(&filename))
        .filter(|filename| !filename.is_empty())
        .unwrap_or_else(|| "download".to_string())
}

fn filename_from_content_disposition(value: &str) -> Option<String> {
    let mut fallback = None;
    for part in value.split(';').map(str::trim) {
        if let Some(encoded) = part.strip_prefix("filename*=") {
            let encoded = encoded.trim_matches('"');
            let encoded = encoded
                .strip_prefix("UTF-8''")
                .or_else(|| encoded.strip_prefix("utf-8''"))
                .unwrap_or(encoded);
            return Some(percent_decode(encoded));
        }
        if let Some(filename) = part.strip_prefix("filename=") {
            fallback = Some(filename.trim_matches('"').to_string());
        }
    }
    fallback
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[index + 1..index + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    output.push(byte);
                    index += 3;
                    continue;
                }
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|character| match character {
            '/' | '\\' | ':' | '\0' => '_',
            _ => character,
        })
        .collect::<String>()
        .trim()
        .trim_start_matches('.')
        .to_string()
}

fn unique_download_path(download_dir: PathBuf, filename: &str) -> PathBuf {
    let path = download_dir.join(filename);
    if !path.exists() {
        return path;
    }

    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
    let extension = path.extension().and_then(|value| value.to_str());
    for counter in 1.. {
        let candidate_name = match extension {
            Some(extension) if !extension.is_empty() => {
                format!("{stem} ({counter}).{extension}")
            }
            _ => format!("{stem} ({counter})"),
        };
        let candidate = download_dir.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    path
}

fn intercept_navigation(app: &AppHandle, url: &Url) -> bool {
    if is_attachment_download_url(url) {
        if let Err(err) = open_attachment_download_url(app, url) {
            log::warn!("Could not open attachment download URL: {err}");
        }
        return false;
    }

    if url.path() == "/auth/oidc/login" {
        let app = app.clone();
        let login_url = url.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(err) = run_external_oidc_login(app, login_url).await {
                log::warn!("Could not start desktop OIDC flow: {err}");
            }
        });

        return false;
    }

    true
}

async fn run_external_oidc_login(app: AppHandle, login_url: Url) -> Result<(), String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let mut browser_url = login_url.clone();
    let redirect_to = format!("http://localhost:{port}/callback");
    let query_pairs: Vec<(String, String)> = browser_url
        .query_pairs()
        .filter(|(key, _)| key != "redirect_to")
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    browser_url.set_query(None);
    {
        let mut pairs = browser_url.query_pairs_mut();
        for (key, value) in query_pairs {
            pairs.append_pair(&key, &value);
        }
        pairs.append_pair("redirect_to", &redirect_to);
    }

    app.opener()
        .open_url(browser_url.as_str(), None::<String>)
        .map_err(|e| e.to_string())?;

    let (mut stream, _) = listener.accept().await.map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf).await.unwrap_or(0);
    let request = String::from_utf8_lossy(&buf[..n]);
    let query = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|path| path.split_once('?').map(|(_, query)| query))
        .unwrap_or_default();

    let params: std::collections::HashMap<String, String> =
        url::form_urlencoded::parse(query.as_bytes())
            .into_owned()
            .collect();

    if let Some(desktop_code) = params.get("desktop_code") {
        let mut complete_url = login_url;
        complete_url.set_path("/auth/desktop/complete");
        complete_url.set_query(Some(&format!(
            "code={}",
            url::form_urlencoded::byte_serialize(desktop_code.as_bytes()).collect::<String>()
        )));

        if let Some(main) = app.get_webview_window("main") {
            main.navigate(complete_url).map_err(|e| e.to_string())?;
            let _ = main.show();
            let _ = main.set_focus();
        }
    } else if let Some(access_token) = params.get("token") {
        let mut callback_url = login_url;
        callback_url.set_path("/oidc/callback");
        callback_url.set_query(Some(&oidc_callback_query(access_token)));

        if let Some(main) = app.get_webview_window("main") {
            main.navigate(callback_url).map_err(|e| e.to_string())?;
            let _ = main.show();
            let _ = main.set_focus();
        }
    }

    let html = "<html><head><title>Signed in</title></head><body><h2>Sign-in successful. You can close this tab.</h2><script>window.close()</script></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;

    Ok(())
}

fn oidc_callback_query(access_token: &str) -> String {
    format!(
        "token={}",
        url::form_urlencoded::byte_serialize(access_token.as_bytes()).collect::<String>()
    )
}

fn login_url_for_server(server_url: &str) -> Result<Url, String> {
    let mut url = Url::parse(server_url).map_err(|e| e.to_string())?;
    url.set_path("/auth/oidc/login");
    url.set_query(None);
    Ok(url)
}

#[tauri::command]
pub fn get_server_url(app: AppHandle) -> Option<String> {
    configured_server_url(&app)
}

#[tauri::command]
pub fn set_server_url(app: AppHandle, url: String) -> Result<String, String> {
    store_server_url(&app, &url)
}

#[tauri::command]
pub fn save_server_url_and_load(
    app: AppHandle,
    window: WebviewWindow,
    url: String,
) -> Result<String, String> {
    let normalized = store_server_url(&app, &url)?;
    open_or_navigate_main_window(&app, &normalized)?;

    if window.label() != "main" {
        let _ = window.close();
    }

    Ok(normalized)
}

#[tauri::command]
pub fn clear_server_url(app: AppHandle) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.delete(KEY_SERVER_URL);
    store.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_setup_window(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    crate::open_setup_window(&app).map_err(|e| e.to_string())?;
    if window.label() == "main" {
        let _ = window.hide();
    }
    Ok(())
}

#[tauri::command]
pub fn start_relogin(app: AppHandle) -> Result<(), String> {
    let server_url =
        configured_server_url(&app).ok_or_else(|| "No workspace URL is configured.".to_string())?;
    let login_url = login_url_for_server(&server_url)?;
    tauri::async_runtime::spawn(async move {
        if let Err(err) = run_external_oidc_login(app, login_url).await {
            log::warn!("Could not start desktop OIDC relogin flow: {err}");
        }
    });
    Ok(())
}

/// Updates the tray icon and tooltip to reflect the unread message count.
#[tauri::command]
pub fn set_badge_count(app: AppHandle, count: u32) -> Result<(), String> {
    let Some(tray) = app.tray_by_id("main") else {
        return Ok(());
    };
    if count > 0 {
        let icon = Image::from_bytes(include_bytes!("../icons/tray-badge-template.png"))
            .map_err(|e| e.to_string())?;
        tray.set_icon_with_as_template(Some(icon), true)
            .map_err(|e| e.to_string())?;
        tray.set_tooltip(Some(&format!(
            "ex — {} unread message{}",
            count,
            if count == 1 { "" } else { "s" }
        )))
        .map_err(|e| e.to_string())?;
    } else {
        let icon = Image::from_bytes(include_bytes!("../icons/tray-template.png"))
            .map_err(|e| e.to_string())?;
        tray.set_icon_with_as_template(Some(icon), true)
            .map_err(|e| e.to_string())?;
        tray.set_tooltip(Some("ex")).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        filename_for_download, filename_from_content_disposition, is_attachment_download_url,
        login_url_for_server, oidc_callback_query, percent_decode, sanitize_filename,
    };
    use url::Url;

    #[test]
    fn callback_query_contains_only_access_token() {
        assert_eq!(oidc_callback_query("access token"), "token=access+token");
    }

    #[test]
    fn relogin_url_uses_server_origin() {
        assert_eq!(
            login_url_for_server("https://chat.example.com/workspace")
                .unwrap()
                .as_str(),
            "https://chat.example.com/auth/oidc/login"
        );
    }

    #[test]
    fn detects_presigned_attachment_download_urls() {
        let url = Url::parse("https://bucket.s3.eu-north-1.amazonaws.com/attachments/abc?response-content-disposition=attachment%3B%20filename%3D%22file.png%22").unwrap();

        assert!(is_attachment_download_url(&url));
    }

    #[test]
    fn extracts_download_filename_from_content_disposition() {
        assert_eq!(
            filename_from_content_disposition(
                "attachment; filename=\"fallback.png\"; filename*=UTF-8''Screenshot%202026.png"
            ),
            Some("Screenshot 2026.png".to_string())
        );
    }

    #[test]
    fn derives_filename_for_download_from_query_header() {
        let url = Url::parse("https://bucket.s3.eu-north-1.amazonaws.com/attachments/abc?response-content-disposition=attachment%3B%20filename%3D%22file.png%22").unwrap();

        assert_eq!(filename_for_download(&url, None), "file.png");
    }

    #[test]
    fn sanitizes_unsafe_download_filenames() {
        assert_eq!(
            sanitize_filename("../bad/name:image.png"),
            "_bad_name_image.png"
        );
    }

    #[test]
    fn percent_decodes_utf8_filename_values() {
        assert_eq!(
            percent_decode("Screenshot%202026.png"),
            "Screenshot 2026.png"
        );
    }
}
