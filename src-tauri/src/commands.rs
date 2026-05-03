use keyring_core::{Entry, Error as KeyringError};
use std::sync::Once;
use tauri::webview::WebviewWindowBuilder;
use tauri::{image::Image, webview::PageLoadEvent, AppHandle, Manager, WebviewUrl, WebviewWindow};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use url::Url;

use crate::config::normalize_server_url;

const KEYRING_SERVICE: &str = "com.digitaltolk.ex";
const KEYRING_REFRESH_TOKEN: &str = "refresh_token";
const STORE_FILE: &str = "config.json";
const KEY_SERVER_URL: &str = "serverUrl";

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

  async function getNativeRefreshToken() {{
    try {{
      return await globalThis.__TAURI_INTERNALS__?.invoke('get_refresh_token');
    }} catch {{
      return null;
    }}
  }}

  async function clearNativeRefreshToken() {{
    try {{
      await globalThis.__TAURI_INTERNALS__?.invoke('delete_refresh_token');
    }} catch {{
      // ignore
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
        const refreshToken = await getNativeRefreshToken();
        if (refreshToken) {{
          const headers = new Headers(
            input instanceof Request ? input.headers : init?.headers
          );
          headers.set('X-Refresh-Token', refreshToken);

          const response = await nativeFetch(rewrittenUrl, {{
            ...init,
            method: input instanceof Request ? input.method : init?.method,
            body: input instanceof Request ? input.body : init?.body,
            cache: input instanceof Request ? input.cache : init?.cache,
            credentials: input instanceof Request ? input.credentials : init?.credentials,
            integrity: input instanceof Request ? input.integrity : init?.integrity,
            keepalive: input instanceof Request ? input.keepalive : init?.keepalive,
            mode: input instanceof Request ? input.mode : init?.mode,
            redirect: input instanceof Request ? input.redirect : init?.redirect,
            referrer: input instanceof Request ? input.referrer : init?.referrer,
            referrerPolicy: input instanceof Request ? input.referrerPolicy : init?.referrerPolicy,
            signal: input instanceof Request ? input.signal : init?.signal,
            headers,
          }});

          if (response.status === 401) {{
            await clearNativeRefreshToken();
          }}

          return response;
        }}
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
    let url = store
        .get(KEY_SERVER_URL)
        .and_then(|v| v.as_str().map(str::to_string))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())?;
    normalize_server_url(&url).ok()
}

fn refresh_token_entry() -> Result<Entry, String> {
    static KEYRING_INIT: Once = Once::new();
    KEYRING_INIT.call_once(|| {
        let _ = keyring::use_native_store(false);
    });

    Entry::new(KEYRING_SERVICE, KEYRING_REFRESH_TOKEN).map_err(|e| e.to_string())
}

fn delete_refresh_token_internal() -> Result<(), String> {
    match refresh_token_entry() {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(KeyringError::NoEntry) => Ok(()),
            Err(err) => Err(err.to_string()),
        },
        Err(err) => Err(err),
    }
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

    if let Some(main) = app.get_webview_window("main") {
        main.navigate(parsed).map_err(|e| e.to_string())?;
        let _ = main.show();
        let _ = main.set_focus();
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(parsed))
        .on_navigation(move |url| intercept_oidc_navigation(&app_handle, url))
        .on_page_load({
            let init_script = init_script.clone();
            move |webview, payload| {
                if payload.event() == PageLoadEvent::Finished {
                    let _ = webview.eval(init_script.clone());
                }
            }
        })
        .initialization_script(init_script)
        .title("ex")
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

fn intercept_oidc_navigation(app: &AppHandle, url: &Url) -> bool {
    if url.path() != "/auth/oidc/login" {
        return true;
    }

    let app = app.clone();
    let login_url = url.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = run_external_oidc_login(app, login_url).await {
            log::warn!("Could not start desktop OIDC flow: {err}");
        }
    });

    false
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

    if let Some(refresh_token) = params.get("refresh") {
        let _ = refresh_token_entry()
            .and_then(|entry| entry.set_password(refresh_token).map_err(|e| e.to_string()));
    }

    if let Some(access_token) = params.get("token") {
        let mut callback_url = login_url;
        callback_url.set_path("/oidc/callback");
        callback_url.set_query(Some(&format!(
            "token={}",
            url::form_urlencoded::byte_serialize(access_token.as_bytes()).collect::<String>()
        )));

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

#[tauri::command]
pub fn get_server_url(app: AppHandle) -> Option<String> {
    configured_server_url(&app)
}

#[tauri::command]
pub fn set_server_url(app: AppHandle, url: String) -> Result<String, String> {
    let previous = configured_server_url(&app);
    let normalized = store_server_url(&app, &url)?;
    if previous.as_deref() != Some(normalized.as_str()) {
        let _ = delete_refresh_token_internal();
    }
    Ok(normalized)
}

#[tauri::command]
pub fn save_server_url_and_load(
    app: AppHandle,
    window: WebviewWindow,
    url: String,
) -> Result<String, String> {
    let previous = configured_server_url(&app);
    let normalized = store_server_url(&app, &url)?;
    if previous.as_deref() != Some(normalized.as_str()) {
        let _ = delete_refresh_token_internal();
    }
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
    store.save().map_err(|e| e.to_string())?;
    let _ = delete_refresh_token_internal();
    Ok(())
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
pub fn get_refresh_token() -> Option<String> {
    refresh_token_entry()
        .ok()
        .and_then(|entry| entry.get_password().ok())
}

#[tauri::command]
pub fn delete_refresh_token() -> Result<(), String> {
    delete_refresh_token_internal()
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
