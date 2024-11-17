mod config;

use std::sync::{Arc, Mutex};
use tauri::Manager;

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(Arc::new(Mutex::new(config::Config::default())));
    app.manage(Arc::new(Mutex::new(plugin_api::PluginManager::default())));

    let plugin_manager_state = app.state::<Arc<Mutex<plugin_api::PluginManager>>>();

    loop {
        let plugin_manager_guard = plugin_manager_state.try_lock();
        if plugin_manager_guard.is_ok() {
            let mut plugin_manager = plugin_manager_guard.expect("Thread should not be poisoned");
            plugin_manager.init(vec![
                Box::new(core_plugin_calculator::CalculatorPlugin::default()),
                Box::new(core_plugin_fs::FsPlugin::default()),
            ]);
            break;
        }
    }

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
        .invoke_handler(tauri::generate_handler![
            update_search_query,
            get_search_results
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(handle_run_events)
}

#[tauri::command]
fn update_search_query(app_handle: tauri::AppHandle, query: String) {
    let plugin_manager_state = app_handle.state::<Arc<Mutex<plugin_api::PluginManager>>>();
    loop {
        let plugin_manager_guard = plugin_manager_state.try_lock();
        if plugin_manager_guard.is_ok() {
            let plugin_manager = plugin_manager_guard.expect("Thread should not be poisoned");
            plugin_manager.get_client_state().update_search_query(query);
            break;
        }
    }
}

#[tauri::command]
fn get_search_results(
    app_handle: tauri::AppHandle,
) -> Result<Vec<plugin_api::SearchResult>, String> {
    let plugin_manager_state = app_handle.state::<Arc<Mutex<plugin_api::PluginManager>>>();
    loop {
        let plugin_manager_guard = plugin_manager_state.try_lock();
        if plugin_manager_guard.is_ok() {
            let plugin_manager = plugin_manager_guard.expect("Thread should not be poisoned");
            let search_results = plugin_manager.get_client_state().get_search_results();
            break Ok(search_results);
        }
    }
}
