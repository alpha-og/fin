use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
// pub struct KeyBinding {
//     description: String,
//     modifiers: Modifiers,
//     code: Code,
// }
// pub struct KeyBindings {
//
// }

pub fn init(app: &tauri::App) {
    let global_shortcut = Shortcut::new(Some(Modifiers::ALT.union(Modifiers::SHIFT)), Code::Space);
    app.handle()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if shortcut == &global_shortcut && event.state == ShortcutState::Pressed {
                        let main_window = match app.get_webview_window("main") {
                            Some(main_window) => main_window,
                            None => {
                                let main_window = tauri::WebviewWindowBuilder::from_config(
                                    app,
                                    app.config().app.windows.get(0).unwrap(),
                                )
                                .unwrap()
                                .build()
                                .unwrap();
                                let _ = main_window.hide();
                                main_window
                            }
                        };

                        if main_window.is_focused().unwrap() {
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
