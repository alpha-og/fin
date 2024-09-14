mod cache;
mod config;
mod db;
mod plugins;

use libloading::{Library, Symbol};
use std::{
    ffi::{CStr, CString},
    sync::{Arc, Mutex},
};
use tauri::Manager;

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Arc::new(Mutex::new(config::Config::default())));
    app.manage(Arc::new(Mutex::new(db::Db::default())));
    app.manage(Arc::new(Mutex::new(cache::Cache::default())));

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let config_state = app.state::<Arc<Mutex<config::Config>>>();
    loop {
        let config_guard = config_state.try_lock();
        if config_guard.is_ok() {
            let mut config = config_guard.expect("Thread should not be poisoned");
            config.init(app);
            break;
        }
    }

    let db_state = app.state::<Arc<Mutex<db::Db>>>();
    {
        let db_arc = Arc::clone(&db_state);
        std::thread::spawn(move || loop {
            let db_guard = db_arc.try_lock();
            if db_guard.is_ok() {
                let mut db = db_guard.expect("Thread should not be poisoned");
                db.init(Some(
                    "/Users/athulanoop/.config/fin/cache.sqlite".to_string(),
                ));
                break;
            }
        });
    }

    let cache_state = app.state::<Arc<Mutex<cache::Cache>>>();
    let cache_arc = Arc::clone(&cache_state);
    {
        let db_arc = Arc::clone(&db_state);
        std::thread::spawn(move || loop {
            let db_guard = db_arc.try_lock();

            let cache_guard = cache_arc.try_lock();
            if db_guard.is_ok() && cache_guard.is_ok() {
                let mut cache = cache_guard.expect("Thread should not be poisoned");
                let db = db_guard.expect("Thread should not be poisoned");
                tauri::async_runtime::block_on(cache.init(&db));
                break;
            }
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
        .on_window_event(handle_window_events)
        .plugin(tauri_plugin_shell::init())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![query])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(handle_run_events)
}
#[derive(serde::Serialize, Clone)]
struct QueryResponse {
    calculation: Result<f64, String>,
    fs_entries: Result<Vec<db::fs::Entry>, String>,
}
#[repr(C)]
pub struct ResultWrapper {
    pub is_ok: bool,
    pub value: f64,
    pub error: *const libc::c_char,
}
#[tauri::command]
fn query(app_handle: tauri::AppHandle, query: String) -> Result<QueryResponse, String> {
    let calculation;
    unsafe {
        let lib = Library::new(
            "/Users/athulanoop/software_projects/Rust/fin/core-plugins/core-plugin-calculator/target/release/libcore_plugin_calculator.dylib",
        )
        .unwrap();

        let calculate: Symbol<unsafe extern "C" fn(*const libc::c_char) -> ResultWrapper> = lib
            .get(b"calculate")
            .expect("calculate method should exist");
        let free_error_string: Symbol<unsafe extern "C" fn(*const libc::c_char)> = lib
            .get(b"free_error_string")
            .expect("free_error_string method should exist");
        let result = calculate(
            CString::new(query.clone())
                .expect("Should be valid query")
                .into_raw(),
        );

        if result.is_ok {
            calculation = Ok(result.value);
        } else {
            calculation = Err(String::from(CStr::from_ptr(result.error).to_string_lossy()));
            free_error_string(result.error);
        }
        let _ = lib.close();
    }
    let fs_entries = db::get_files(app_handle, &query);
    Ok(QueryResponse {
        calculation,
        fs_entries,
    })
}
