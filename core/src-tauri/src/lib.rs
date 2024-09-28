mod cache;
mod config;
mod db;

use plugin_api::{PluginEventPayload, PluginManager};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Test {
    ok: String,
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Arc::new(Mutex::new(config::Config::default())));
    app.manage(Arc::new(Mutex::new(db::Db::default())));
    app.manage(Arc::new(Mutex::new(cache::Cache::default())));
    app.manage(Arc::new(Mutex::new(PluginManager::default())));
    app.manage(Arc::new(Mutex::new(String::new())));

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let plugin_manager_state = app.state::<Arc<Mutex<plugin_api::PluginManager>>>();
    loop {
        let plugin_manager_guard = plugin_manager_state.try_lock();
        if plugin_manager_guard.is_ok() {
            let mut plugin_manager = plugin_manager_guard.expect("Thread should not be poisoned");
            let calculator_plugin = core_plugin_calculator::CalculatorPlugin::default();
            plugin_manager.register_plugin("calculator", calculator_plugin);

            plugin_manager.init_plugins();
            break;
        }
    }

    let query_state = app.state::<Arc<Mutex<String>>>();
    let query_arc = Arc::clone(&query_state);
    let plugin_manager_arc = Arc::clone(&plugin_manager_state);
    // let app_handle = app.app_handle().clone();
    app.listen("query", move |event| loop {
        let query_guard = query_arc.try_lock();
        if query_guard.is_ok() {
            let mut query = query_guard.expect("Thread should not be poisoned");
            let payload =
                serde_json::from_str::<std::collections::HashMap<String, String>>(event.payload())
                    .expect("should be valid (string,string) hashmap");
            *query = payload
                .get("query")
                .expect("query should be valid key")
                .to_owned();
            let mut plugin_manager = plugin_manager_arc.lock().expect("Should not be poisoned");
            plugin_manager.emit("query", PluginEventPayload::Single(query.to_string()));
            // plugin_manager.listen("response", |plugin_event_payload| {
            //     if let PluginEventPayload::Single(payload) = plugin_event_payload {
            //         let _ = app_handle.emit("response", payload);
            //     }
            // });
            break;
        }
    });

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

fn handle_run_events(app_handle: &tauri::AppHandle, _event: tauri::RunEvent) {
    // TODO: add cleanup code (example: cache app state to persistent sqlite db)
    // if let tauri::RunEvent::ExitRequested { api, .. } = event {
    //     api.prevent_exit();
    // }
    let plugin_manager_state = app_handle.state::<Arc<Mutex<PluginManager>>>();
    let plugin_manager = plugin_manager_state.lock().expect("Should not be poisoned");
    let mut event_bus = plugin_manager
        .event_bus
        .lock()
        .expect("Should not be poisoned");
    let payload = event_bus.get("response");
    if let Some(PluginEventPayload::Single(payload)) = payload {
        app_handle
            .emit("response", payload)
            .expect("Should emit properly");
    }
    event_bus.remove("response");
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

#[tauri::command]
fn query(app_handle: tauri::AppHandle, query: String) -> Result<QueryResponse, String> {
    let query_guard = app_handle.state::<Arc<Mutex<String>>>();
    let _query = query_guard.lock().expect("should not be poisoned");

    // let plugin_manager_guard = app_handle.state::<Arc<Mutex<plugin_api::PluginManager>>>();
    // let plugin_manager = plugin_manager_guard.lock().expect("should not be locked");
    //
    // let (sender, receiver) = std::sync::mpsc::channel();
    // plugin_manager.plugins.get("calculator").unwrap().execute(
    //     sender,
    //     "calculate",
    //     vec![query.clone()],
    // );
    // let calculation = receiver.recv().expect("should receive calculation output");
    // let fs_entries = db::get_files(&app_handle, &query);
    Ok(QueryResponse {
        calculation: Ok(0.0),
        fs_entries: Ok(vec![]),
    })
}
