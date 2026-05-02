mod commands;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_deep_link::DeepLinkExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // On Wayland, WebKitGTK's EGL/DMA-BUF renderer fails on some GPU driver
    // combinations ("Could not create default EGL display: EGL_BAD_PARAMETER").
    // Force X11/XWayland so GTK uses GLX instead of EGL — XWayland is always
    // present on GNOME/KDE. Also disable the DMA-BUF renderer as a belt-and-
    // suspenders measure.
    // SAFETY: called before any threads are spawned by Tauri or WebKit.
    #[cfg(target_os = "linux")]
    unsafe {
        if std::env::var("WAYLAND_DISPLAY").is_ok()
            && std::env::var("GDK_BACKEND").is_err()
        {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        if std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER").is_err() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // A second instance was launched (e.g. by an ex:// deep link).
            // Bring the existing window forward; the deep-link plugin fires
            // on_open_url after single-instance forwards the args, which then
            // emits the 'deep-link' event to the frontend.
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
            // Belt-and-suspenders: also emit directly in case the deep-link
            // plugin's on_open_url doesn't fire (plugin init order race).
            let handle = app.clone();
            let urls: Vec<String> = argv.into_iter()
                .filter(|a| a.starts_with("ex://"))
                .collect();
            if !urls.is_empty() {
                tauri::async_runtime::spawn(async move {
                    // Emit twice: once after a short delay (window focus) and
                    // once after a longer delay (React may still be loading if
                    // this is a fresh instance launched by the OS for the link).
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    for url in &urls {
                        let _ = handle.emit("deep-link", url.clone());
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    for url in &urls {
                        let _ = handle.emit("deep-link", url.clone());
                    }
                });
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Global shortcut: Ctrl+Shift+E shows and focuses the window.
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyE);
            let handle = app.handle().clone();
            if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
                if let Some(w) = handle.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.set_focus();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }) {
                log::warn!("Could not register global shortcut Ctrl+Shift+E (already in use by OS?): {e}");
            }

            let open_i = MenuItem::with_id(app, "open", "Open ex", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &sep, &quit_i])?;

            let tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("ex")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Keep the tray alive for the app's lifetime.
            app.manage(tray);

            // Check for app updates in the background after a short delay.
            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_updater::UpdaterExt;
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    if let Ok(updater) = handle.updater() {
                        if let Ok(Some(_)) = updater.check().await {
                            let _ = handle.emit("update-available", ());
                        }
                    }
                });
            }

            // Set the window icon explicitly (needed for Linux taskbar/switcher in dev mode).
            if let Some(w) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon() {
                    let _ = w.set_icon(icon.clone());
                }
            }

            // Register the ex:// URL scheme with the OS.
            app.deep_link().register("ex").ok();

            // Handle ex:// deep links — bring the window forward and forward
            // the URL to the frontend so it can navigate.
            let handle = app.handle().clone();
            app.deep_link().on_open_url(move |event: tauri_plugin_deep_link::OpenUrlEvent| {
                if let Some(w) = handle.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
                for url in event.urls() {
                    let _ = handle.emit("deep-link", url.to_string());
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide to tray instead of closing when user presses X.
            // Quit is available from the tray menu.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::get_server_url,
            commands::set_server_url,
            commands::get_autostart,
            commands::set_autostart,
            commands::set_badge_count,
            commands::get_refresh_token,
            commands::set_refresh_token,
            commands::delete_refresh_token,
            commands::start_oauth_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ex desktop");
}
