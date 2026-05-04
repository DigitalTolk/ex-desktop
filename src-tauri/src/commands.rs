use std::path::PathBuf;
use tauri::webview::{DownloadEvent, NewWindowResponse, WebviewWindowBuilder};
use tauri::{image::Image, webview::PageLoadEvent, AppHandle, Manager, WebviewUrl, WebviewWindow};
#[cfg(not(target_os = "macos"))]
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use url::Url;

use crate::config::normalize_server_url;
#[cfg(target_os = "macos")]
use std::sync::Once;

const STORE_FILE: &str = "config.json";
const KEY_SERVER_URL: &str = "serverUrl";
const LEGACY_KEY_REFRESH_TOKEN: &str = "refreshToken";

fn main_webview_devtools_enabled() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn disable_native_context_menu(window: &WebviewWindow) {
    let _ = window;
    filter_wkwebview_context_menu();
}

#[cfg(not(target_os = "macos"))]
fn disable_native_context_menu(_window: &WebviewWindow) {}

#[cfg(target_os = "macos")]
fn native_context_menu_will_open_selector_name() -> &'static std::ffi::CStr {
    c"willOpenMenu:withEvent:"
}

#[cfg(target_os = "macos")]
fn filter_wkwebview_context_menu() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| unsafe {
        use objc2::ffi::{class_getInstanceMethod, method_setImplementation};
        use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};

        unsafe extern "C-unwind" fn filter_context_menu(
            _this: *mut AnyObject,
            _cmd: Sel,
            menu: *mut AnyObject,
            _event: *mut AnyObject,
        ) {
            if menu.is_null() {
                return;
            }
            let menu = &*(menu as *mut objc2_app_kit::NSMenu);
            let mut index = menu.numberOfItems();
            while index > 0 {
                index -= 1;
                let Some(item) = menu.itemAtIndex(index) else {
                    continue;
                };
                let title = item.title().to_string().to_lowercase();
                if title == "back" || title == "reload" {
                    menu.removeItemAtIndex(index);
                }
            }
            if menu.numberOfItems() == 0 {
                menu.cancelTracking();
            }
        }

        let filter_menu_implementation: Imp = std::mem::transmute(
            filter_context_menu
                as unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, *mut AnyObject),
        );
        for class_name in [c"WKWebView", c"WKContentView"] {
            let Some(class) = AnyClass::get(class_name) else {
                log::warn!(
                    "Could not find {} class to disable native context menu",
                    class_name.to_string_lossy()
                );
                continue;
            };
            let method = class_getInstanceMethod(
                class,
                Sel::register(native_context_menu_will_open_selector_name()),
            );
            if method.is_null() {
                log::warn!(
                    "Could not find {} willOpenMenu:withEvent: to filter native context menu",
                    class_name.to_string_lossy()
                );
                continue;
            }
            let _ = method_setImplementation(method, filter_menu_implementation);
        }
    });
}

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
  Object.defineProperty(globalThis, '__EX_DESKTOP__', {{
    value: true,
    writable: false,
    configurable: false
  }});
  Object.defineProperty(globalThis, '__EX_DESKTOP_LINK_DIAGNOSTICS__', {{
    value: [],
    writable: false,
    configurable: false
  }});

  let authRequiredShown = false;

  function recordDesktopLinkDiagnostic(event, detail) {{
    const entry = {{
      event,
      detail,
      href: globalThis.location.href,
      timestamp: new Date().toISOString()
    }};
    try {{
      globalThis.__EX_DESKTOP_LINK_DIAGNOSTICS__.push(entry);
      if (globalThis.__EX_DESKTOP_LINK_DIAGNOSTICS__.length > 50) {{
        globalThis.__EX_DESKTOP_LINK_DIAGNOSTICS__.shift();
      }}
    }} catch {{}}
    try {{
      console.info('[ex-desktop-link]', event, detail);
    }} catch {{}}
  }}

  function unreadCountFromTitle() {{
    const match = /^\\((\\d+)\\)\\s+/.exec(document.title || '');
    if (!match) {{
      return 0;
    }}
    const count = Number.parseInt(match[1], 10);
    return Number.isFinite(count) && count > 0 ? count : 0;
  }}

  function installUnreadBadgeBridge() {{
    let lastCount = -1;
    function syncUnreadBadge() {{
      const count = unreadCountFromTitle();
      if (count === lastCount) {{
        return;
      }}
      lastCount = count;
      try {{
        globalThis.__TAURI_INTERNALS__?.invoke('set_badge_count', {{ count }});
      }} catch {{
        // ignore
      }}
    }}

    function observeTitle() {{
      const title = document.querySelector('title');
      if (!title) {{
        return false;
      }}
      syncUnreadBadge();
      new MutationObserver(syncUnreadBadge).observe(title, {{
        childList: true,
        characterData: true,
        subtree: true
      }});
      return true;
    }}

    if (observeTitle()) {{
      return;
    }}
    let attempts = 0;
    const timer = globalThis.setInterval(() => {{
      attempts += 1;
      if (observeTitle() || attempts >= 20) {{
        globalThis.clearInterval(timer);
      }}
    }}, 250);
  }}

  function selectedContextMenuText() {{
    return String(globalThis.getSelection?.() || '').trim();
  }}

  function contextMenuAnchor(target) {{
    const element = target?.nodeType === Node.ELEMENT_NODE ? target : target?.parentElement;
    return element?.closest?.('a[href]') || null;
  }}

  function eventAnchor(event) {{
    for (const item of event.composedPath?.() || []) {{
      if (item instanceof HTMLAnchorElement && item.href) {{
        return item;
      }}
      if (item instanceof Element) {{
        const anchor = item.closest?.('a[href]');
        if (anchor?.href) {{
          return anchor;
        }}
      }}
    }}
    return contextMenuAnchor(event.target);
  }}

  function desktopExternalURL(href) {{
    try {{
      const url = new URL(href, globalThis.location.href);
      if (url.protocol === 'mailto:') {{
        return url;
      }}
      if (url.protocol !== 'http:' && url.protocol !== 'https:') {{
        return null;
      }}
      return url.origin === globalThis.location.origin ? null : url;
    }} catch {{
      return null;
    }}
  }}

  function openExternalURL(url) {{
    const invoke = globalThis.__TAURI_INTERNALS__?.invoke;
    if (typeof invoke !== 'function') {{
      return Promise.reject(new Error('Tauri IPC invoke is not available'));
    }}
    return invoke('open_external_link', {{ url: url.href }}).catch((appCommandError) => {{
      recordDesktopLinkDiagnostic('open-external-app-command-error', {{
        href: url.href,
        error: String(appCommandError)
      }});
      return invoke('plugin:opener|open_url', {{ url: url.href }});
    }});
  }}

  function installExternalLinkBridge() {{
    const onExternalLinkClick = (event) => {{
      if (
        event.defaultPrevented ||
        event.button !== 0 ||
        event.metaKey ||
        event.ctrlKey ||
        event.altKey ||
        event.shiftKey
      ) {{
        return;
      }}

      const anchor = eventAnchor(event);
      if (!anchor?.href) {{
        recordDesktopLinkDiagnostic('click-no-anchor', {{
          target: event.target?.nodeName || null
        }});
        return;
      }}

      const url = desktopExternalURL(anchor.href);
      if (!url) {{
        recordDesktopLinkDiagnostic('click-internal-or-unsupported', {{
          href: anchor.href
        }});
        return;
      }}

      event.preventDefault();
      event.stopImmediatePropagation?.();
      recordDesktopLinkDiagnostic('click-open-external', {{ href: url.href }});
      void openExternalURL(url).then(
        () => {{
          recordDesktopLinkDiagnostic('open-external-ok', {{ href: url.href }});
        }},
        (error) => {{
          recordDesktopLinkDiagnostic('open-external-error', {{
            href: url.href,
            error: String(error)
          }});
          globalThis.location.href = url.href;
        }}
      );
    }};

    globalThis.addEventListener('click', onExternalLinkClick, true);
    document.addEventListener(
      'click',
      onExternalLinkClick,
      true
    );

    globalThis.__EX_DESKTOP_TEST_EXTERNAL_LINK__ = () => {{
      const anchor = document.createElement('a');
      anchor.href = 'https://example.com/ex-desktop-link-test';
      anchor.textContent = 'ex desktop link test';
      anchor.style.position = 'fixed';
      anchor.style.left = '0';
      anchor.style.top = '0';
      anchor.style.zIndex = '2147483647';
      anchor.setAttribute('data-ex-desktop-test-link', 'true');
      document.body?.appendChild(anchor);
      recordDesktopLinkDiagnostic('test-link-click-dispatch', {{ href: anchor.href }});
      anchor.click();
      globalThis.setTimeout(() => anchor.remove(), 1000);
      return globalThis.__EX_DESKTOP_LINK_DIAGNOSTICS__;
    }};
  }}

  function installWindowOpenBridge() {{
    const originalOpen = globalThis.open?.bind(globalThis);
    globalThis.open = (rawUrl, target, features) => {{
      const url = desktopExternalURL(String(rawUrl || ''));
      if (!url) {{
        return originalOpen?.(rawUrl, target, features) || null;
      }}
      recordDesktopLinkDiagnostic('window-open-external', {{ href: url.href }});
      void openExternalURL(url).then(
        () => {{
          recordDesktopLinkDiagnostic('open-external-ok', {{ href: url.href }});
        }},
        (error) => {{
          recordDesktopLinkDiagnostic('open-external-error', {{
            href: url.href,
            error: String(error)
          }});
          originalOpen?.(url.href, target, features);
        }}
      );
      return null;
    }};
  }}

  function installExternalFormBridge() {{
    document.addEventListener(
      'submit',
      (event) => {{
        const form = event.target;
        if (!(form instanceof HTMLFormElement) || !form.action) {{
          return;
        }}

        const url = desktopExternalURL(form.action);
        if (!url) {{
          return;
        }}

        event.preventDefault();
        recordDesktopLinkDiagnostic('submit-open-external', {{ href: url.href }});
        void openExternalURL(url).then(
          () => {{
            recordDesktopLinkDiagnostic('open-external-ok', {{ href: url.href }});
          }},
          (error) => {{
            recordDesktopLinkDiagnostic('open-external-error', {{
              href: url.href,
              error: String(error)
            }});
            globalThis.location.href = url.href;
          }}
        );
      }},
      true
    );
  }}

  function copyContextMenuText(value) {{
    if (navigator.clipboard?.writeText) {{
      return navigator.clipboard.writeText(value);
    }}
    const input = document.createElement('textarea');
    input.value = value;
    input.setAttribute('readonly', '');
    input.style.position = 'fixed';
    input.style.opacity = '0';
    document.body.appendChild(input);
    input.select();
    document.execCommand('copy');
    input.remove();
    return Promise.resolve();
  }}

  function removeDesktopContextMenu() {{
    document.getElementById('__ex-desktop-context-menu')?.remove();
  }}

  function showDesktopContextMenu(event, items) {{
    removeDesktopContextMenu();
    if (!items.length || !document.body) {{
      return;
    }}

    const menu = document.createElement('div');
    menu.id = '__ex-desktop-context-menu';
    menu.setAttribute('role', 'menu');
    Object.assign(menu.style, {{
      position: 'fixed',
      left: `${{event.clientX}}px`,
      top: `${{event.clientY}}px`,
      zIndex: '2147483647',
      minWidth: '128px',
      padding: '4px 0',
      borderRadius: '6px',
      border: '1px solid rgba(15, 23, 42, 0.14)',
      background: 'rgba(255, 255, 255, 0.98)',
      color: '#0f172a',
      font: '13px/1.2 system-ui, sans-serif',
      boxShadow: '0 12px 36px rgba(15, 23, 42, 0.22)'
    }});

    for (const item of items) {{
      const button = document.createElement('button');
      button.type = 'button';
      button.setAttribute('role', 'menuitem');
      button.textContent = item.label;
      Object.assign(button.style, {{
        display: 'block',
        width: '100%',
        border: '0',
        padding: '7px 12px',
        background: 'transparent',
        color: 'inherit',
        font: 'inherit',
        textAlign: 'left',
        cursor: 'default'
      }});
      button.addEventListener('mouseenter', () => {{
        button.style.background = 'rgba(15, 23, 42, 0.08)';
      }});
      button.addEventListener('mouseleave', () => {{
        button.style.background = 'transparent';
      }});
      button.addEventListener('click', () => {{
        void copyContextMenuText(item.value);
        removeDesktopContextMenu();
      }});
      menu.appendChild(button);
    }}

    document.body.appendChild(menu);
  }}

  function installDesktopContextMenu() {{
    const onContextMenu = (event) => {{
      event.preventDefault();

      const anchor = contextMenuAnchor(event.target);
      if (anchor?.closest?.('[data-ex-desktop-link="true"]')) {{
        return;
      }}

      const selectedText = selectedContextMenuText();
      const items = [];
      if (selectedText) {{
        items.push({{ label: 'Copy', value: selectedText }});
      }}
      if (anchor?.href) {{
        items.push({{ label: 'Copy Link', value: anchor.href }});
      }}
      showDesktopContextMenu(event, items);
    }};

    document.addEventListener('contextmenu', onContextMenu, true);
    document.addEventListener('click', removeDesktopContextMenu, true);
    document.addEventListener('scroll', removeDesktopContextMenu, true);
    document.addEventListener(
      'keydown',
      (event) => {{
        if (event.key === 'Escape') {{
          removeDesktopContextMenu();
        }}
      }},
      true
    );
  }}

  installExternalLinkBridge();
  installWindowOpenBridge();
  installExternalFormBridge();
  installDesktopContextMenu();
  installUnreadBadgeBridge();

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
    let new_window_app_handle = app.clone();
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
        .on_new_window(move |url, _features| {
            if handle_new_window_request(&new_window_app_handle, &url) {
                NewWindowResponse::Deny
            } else {
                NewWindowResponse::Allow
            }
        })
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
        .devtools(main_webview_devtools_enabled())
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
    disable_native_context_menu(&window);

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

fn open_external_url(app: &AppHandle, url: &Url) -> Result<(), String> {
    app.opener()
        .open_url(url.as_str(), None::<String>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_external_link(app: AppHandle, url: String) -> Result<(), String> {
    let url = Url::parse(&url).map_err(|e| e.to_string())?;
    if should_open_externally(&app, &url) {
        open_external_url(&app, &url)
    } else {
        Ok(())
    }
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn is_workspace_url(app: &AppHandle, url: &Url) -> bool {
    configured_server_url(app)
        .and_then(|server_url| Url::parse(&server_url).ok())
        .is_some_and(|server_url| same_origin(&server_url, url))
}

fn should_open_externally(app: &AppHandle, url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https" | "mailto") && !is_workspace_url(app, url)
}

fn handle_new_window_request(app: &AppHandle, url: &Url) -> bool {
    if matches!(url.scheme(), "http" | "https" | "mailto") {
        if let Err(err) = open_external_url(app, url) {
            log::warn!("Could not open new-window URL: {err}");
        }
        return true;
    }
    false
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

    if should_open_externally(app, url) {
        if let Err(err) = open_external_url(app, url) {
            log::warn!("Could not open external URL: {err}");
        }
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

#[tauri::command]
pub fn request_notification_attention(app: AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };
    if window.is_focused().unwrap_or(false) {
        return Ok(());
    }
    window
        .request_user_attention(Some(tauri::UserAttentionType::Informational))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn send_desktop_notification(
    app: AppHandle,
    title: String,
    body: Option<String>,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    send_macos_user_notification(&app, &title, body.as_deref())?;

    #[cfg(not(target_os = "macos"))]
    {
        let mut notification = app.notification().builder().title(title);
        if let Some(body) = body.filter(|body| !body.trim().is_empty()) {
            notification = notification.body(body);
        }
        notification.show().map_err(|e| e.to_string())?;
    }

    request_notification_attention(app)
}

#[cfg(target_os = "macos")]
fn send_macos_user_notification(
    _app: &AppHandle,
    title: &str,
    body: Option<&str>,
) -> Result<(), String> {
    use objc2_foundation::NSString;
    use objc2_user_notifications::{
        UNMutableNotificationContent, UNNotificationRequest, UNNotificationSound,
        UNUserNotificationCenter,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    let content = UNMutableNotificationContent::new();
    content.setTitle(&NSString::from_str(title));
    if let Some(body) = body.filter(|body| !body.trim().is_empty()) {
        content.setBody(&NSString::from_str(body));
    }
    content.setSound(Some(&UNNotificationSound::defaultSound()));

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let identifier = NSString::from_str(&format!("ex-desktop-{timestamp}"));
    let request =
        UNNotificationRequest::requestWithIdentifier_content_trigger(&identifier, &content, None);

    UNUserNotificationCenter::currentNotificationCenter()
        .addNotificationRequest_withCompletionHandler(&request, None);
    Ok(())
}

/// Updates the tray icon and tooltip to reflect the unread message count.
#[tauri::command]
pub fn set_badge_count(app: AppHandle, count: u32) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let title = if count > 0 {
            format!("({count}) ex")
        } else {
            "ex".to_string()
        };
        let _ = window.set_title(&title);
    }

    let Some(tray) = app.tray_by_id("main") else {
        return Ok(());
    };
    if count > 0 {
        let icon = Image::from_bytes(include_bytes!("../icons/tray-badge-template.png"))
            .map_err(|e| e.to_string())?;
        tray.set_icon_with_as_template(Some(icon), true)
            .map_err(|e| e.to_string())?;
        tray.set_tooltip(Some(&format!(
            "ex — {} unread notification{}",
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
    #[cfg(target_os = "macos")]
    use super::native_context_menu_will_open_selector_name;
    use super::{
        filename_for_download, filename_from_content_disposition, is_attachment_download_url,
        login_url_for_server, main_webview_devtools_enabled, oidc_callback_query, percent_decode,
        remote_main_init_script, same_origin, sanitize_filename,
    };
    use url::Url;

    #[test]
    fn callback_query_contains_only_access_token() {
        assert_eq!(oidc_callback_query("access token"), "token=access+token");
    }

    #[test]
    fn remote_init_script_bridges_unread_title_to_badge_count() {
        let script = remote_main_init_script().unwrap();

        assert!(script.contains("'__EX_DESKTOP__'"));
        assert!(script.contains("unreadCountFromTitle"));
        assert!(script.contains("invoke('set_badge_count', { count })"));
        assert!(script.contains("MutationObserver(syncUnreadBadge)"));
        assert!(!script.contains("installMinimalContextMenu"));
        assert!(!script.contains("globalThis.Notification ="));
        assert!(!script.contains("ServiceWorkerRegistration"));
        assert!(!script.contains("prototype.showNotification"));
    }

    #[test]
    fn remote_init_script_opens_external_links_with_system_browser() {
        let script = remote_main_init_script().unwrap();

        assert!(script.contains("installExternalLinkBridge"));
        assert!(script.contains("document.addEventListener(\n      'click'"));
        assert!(script.contains("contextMenuAnchor(event.target)"));
        assert!(script.contains("event.composedPath?.()"));
        assert!(script.contains("url.protocol === 'mailto:'"));
        assert!(script.contains("url.protocol !== 'http:' && url.protocol !== 'https:'"));
        assert!(script.contains("url.origin === globalThis.location.origin ? null : url"));
        assert!(script.contains("Tauri IPC invoke is not available"));
        assert!(script.contains("invoke('open_external_link', { url: url.href })"));
        assert!(script.contains("invoke('plugin:opener|open_url', { url: url.href })"));
        assert!(script.contains("event.preventDefault()"));
        assert!(script.contains("event.stopImmediatePropagation?.()"));
        assert!(script.contains("globalThis.addEventListener('click', onExternalLinkClick, true)"));
        assert!(script.contains("__EX_DESKTOP_LINK_DIAGNOSTICS__"));
        assert!(script.contains("__EX_DESKTOP_TEST_EXTERNAL_LINK__"));
        assert!(script.contains("installWindowOpenBridge"));
        assert!(script.contains("installExternalFormBridge"));
        assert!(script.contains("open-external-app-command-error"));
        assert!(script.contains("open-external-error"));
    }

    #[test]
    fn remote_chat_capability_allows_only_required_injected_commands() {
        use std::str::FromStr;

        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/remote-chat.json")).unwrap();
        let urls = capability
            .pointer("/remote/urls")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        let permissions = capability
            .get("permissions")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        let permissions = permissions
            .iter()
            .map(|permission| permission.as_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(capability["windows"], serde_json::json!(["main"]));
        assert!(urls.iter().any(|url| url == "http://*"));
        assert!(urls.iter().any(|url| url == "http://*:*"));
        assert!(urls.iter().any(|url| url == "https://*"));
        assert!(urls.iter().any(|url| url == "https://*:*"));
        for (pattern, url) in [
            (
                "http://*:*",
                "http://localhost:5173/channel/general?x=1#message",
            ),
            (
                "https://*",
                "https://chat.example.com/channel/general?x=1#message",
            ),
            (
                "https://*:*",
                "https://chat.example.com:8443/channel/general?x=1#message",
            ),
        ] {
            let pattern = tauri::utils::acl::RemoteUrlPattern::from_str(pattern).unwrap();
            assert!(
                pattern.test(&Url::parse(url).unwrap()),
                "remote pattern must match full chat page URLs"
            );
        }
        assert_eq!(
            permissions,
            vec![
                "allow-show-setup-window",
                "allow-start-relogin",
                "allow-set-badge-count",
                "allow-open-external-link",
                "opener:allow-default-urls",
                "opener:allow-open-url",
            ]
        );
        assert!(!permissions.iter().any(|permission| {
            permission.starts_with("store:")
                || permission.starts_with("updater:")
                || permission.starts_with("global-shortcut:")
        }));
    }

    #[test]
    fn app_manifest_generates_permissions_for_remote_injected_commands() {
        let build_script = include_str!("../build.rs");

        for command in [
            "show_setup_window",
            "start_relogin",
            "set_badge_count",
            "open_external_link",
        ] {
            assert!(
                build_script.contains(command),
                "build.rs must generate an allow/deny permission for {command}"
            );
        }
    }

    #[test]
    fn csp_allows_tauri_ipc_connect_sources() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let csp = config
            .pointer("/app/security/csp")
            .and_then(serde_json::Value::as_str)
            .unwrap();

        assert!(csp.contains("connect-src"));
        assert!(csp.contains("ipc:"));
        assert!(csp.contains("http://ipc.localhost"));
    }

    #[test]
    fn remote_init_script_suppresses_native_context_menu() {
        let script = remote_main_init_script().unwrap();

        assert!(script.contains("installDesktopContextMenu"));
        assert!(script.contains("document.addEventListener('contextmenu', onContextMenu, true)"));
        assert!(script.contains("event.preventDefault()"));
        assert!(script.contains("__ex-desktop-context-menu"));
        assert!(script.contains("Copy Link"));
        assert!(script.contains("data-ex-desktop-link"));
        assert!(!script.contains("event.stopPropagation()"));
        assert!(!script.contains("Reload"));
        assert!(!script.contains("Inspect Element"));
    }

    #[test]
    fn release_build_does_not_enable_tauri_devtools_context_menu() {
        let manifest = include_str!("../Cargo.toml");
        let tauri_dependency_line = manifest
            .lines()
            .find(|line| line.trim_start().starts_with("tauri "))
            .expect("tauri dependency should be declared");

        assert!(
            !tauri_dependency_line.contains("\"devtools\""),
            "the tauri devtools feature enables Inspect Element from the macOS WebKit context menu"
        );
        assert!(
            !main_webview_devtools_enabled(),
            "main webview devtools should stay disabled so WebKit does not add Inspect Element"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_context_menu_override_targets_webkit_content_view() {
        let source = include_str!("commands.rs");

        assert!(source.contains("WKWebView"));
        assert!(source.contains("WKContentView"));
        assert!(source.contains("method_setImplementation"));
        assert!(!source.contains(&format!("{}{}", "menuFor", "Event:")));
        assert!(!source.contains(&format!("{}{}", "no_context", "_menu")));
        assert!(!source.contains(&format!("{}{}", "set", "Menu")));
        assert!(!source.contains(&format!("{}{}", "rightMouse", "Down:")));
        assert!(!source.contains(&format!("{}{}", "rightMouse", "Up:")));
        assert_eq!(
            native_context_menu_will_open_selector_name()
                .to_str()
                .unwrap(),
            "willOpenMenu:withEvent:"
        );
        assert!(source.contains("title == \"back\" || title == \"reload\""));
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
    fn origin_comparison_ignores_paths_but_respects_hosts() {
        let app = Url::parse("https://chat.example.com/channel/general").unwrap();
        let same = Url::parse("https://chat.example.com/docs").unwrap();
        let other = Url::parse("https://example.com/docs").unwrap();

        assert!(same_origin(&app, &same));
        assert!(!same_origin(&app, &other));
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
