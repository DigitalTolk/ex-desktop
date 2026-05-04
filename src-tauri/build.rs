fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_server_url",
            "set_server_url",
            "save_server_url_and_load",
            "clear_server_url",
            "show_setup_window",
            "start_relogin",
            "request_notification_attention",
            "send_desktop_notification",
            "set_badge_count",
            "open_external_link",
        ]),
    ))
    .expect("failed to build Tauri app");
}
