mod db;
mod keymaps;

use db::Db;

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    keymaps::init(app);
    Db::init(
        app,
        Some("/Users/athulanoop/.config/fin/cache.sqlite".to_string()),
    );
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    Ok(())
}

fn handle_window_events(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::Focused(focused) = event {
        if !*focused {
            window.hide().expect("Window should have closed");
        };
    };
}

fn handle_run_events(_app_handle: &tauri::AppHandle, _event: tauri::RunEvent) {
    // TODO: add cleanup code (example: cache app state to persistent sqlite db)
    // if let tauri::RunEvent::ExitRequested { api, .. } = event {
    //     api.prevent_exit();
    // }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(handle_window_events)
        .plugin(tauri_plugin_shell::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![db::get_files])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(handle_run_events)
}
