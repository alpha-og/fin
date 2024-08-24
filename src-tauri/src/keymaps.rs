use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn init(app: &mut tauri::App) {
    let global_shortcut = Shortcut::new(Some(Modifiers::ALT.union(Modifiers::SHIFT)), Code::Space);
    app.handle()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    let main_window = app.get_webview_window("main").unwrap();
                    if shortcut == &global_shortcut && event.state == ShortcutState::Pressed {
                        if main_window.is_visible().unwrap() {
                            let _ = main_window.hide();
                        } else {
                            let _ = main_window.show();
                            let _ = main_window.set_focus();
                        }
                    }
                })
                .build(),
        )
        .unwrap();
    app.global_shortcut().register(global_shortcut).unwrap();
}

#[tauri::command]
pub fn hide_app(window: tauri::Window) {
    let _ = window.hide();
}
