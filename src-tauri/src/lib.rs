mod db;
use db::Db;
use tauri::Manager;
use tauri::State;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Db::default())
        .setup(|app| {
            let handle = app.handle();

            let app_state: State<Db> = handle.state();
            *app_state.connection.lock().unwrap() = Db::init("/Users/athulanoop/.config/fin");
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![db::get_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
