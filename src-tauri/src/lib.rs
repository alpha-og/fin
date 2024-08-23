use walkdir::WalkDir;

#[tauri::command]
async fn get_dirs(query: &str) -> Result<Vec<String>, ()> {
    let mut dirs: Vec<String> = Vec::new();
    for entry in WalkDir::new(query)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        if let Some(path) = entry.path().to_str() {
            dirs.push(path.to_string());
        }
    }
    Ok(dirs)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_dirs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
