use std::sync::{Arc, Mutex};

use tauri::Manager;

mod cache;
mod db;
mod keymaps;

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        keymaps::init(app);
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
    let db_state = app.state::<Arc<Mutex<db::Db>>>();
    {
        let db_arc = Arc::clone(&db_state);
        std::thread::spawn(move || {
            let mut db = db_arc
                .lock()
                .expect("Thread poisoned, db state cannot be accessed");

            db.init(Some(
                "/Users/athulanoop/.config/fin/cache.sqlite".to_string(),
            ));
        });
    }
    let cache_state = app.state::<Arc<Mutex<cache::Cache>>>();
    let cache_arc = Arc::clone(&cache_state);
    {
        let db_arc = Arc::clone(&db_state);
        std::thread::spawn(move || {
            let db = db_arc
                .lock()
                .expect("Thread poisoned, db state cannot be accessed");

            let mut cache = cache_arc
                .lock()
                .expect("Thread poisoned, cache state cannot be accessed");
            tauri::async_runtime::block_on(cache.init(&db));
        });
    }
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
        .manage(Arc::new(Mutex::new(db::Db::default())))
        .manage(Arc::new(Mutex::new(cache::Cache::default())))
        .on_window_event(handle_window_events)
        .plugin(tauri_plugin_shell::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![db::get_files])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(handle_run_events)
}
