mod keymaps;

use directories::BaseDirs;
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use tauri::Manager;
use toml;

#[derive(serde::Serialize, Debug)]
pub struct Config {
    configs: HashMap<String, String>,
    source: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let configs = HashMap::new();
        Self {
            configs,
            source: PathBuf::from_str("./assets/default-config.toml")
                .expect("Should be valid path string"),
        }
    }
}

impl Config {
    pub fn init(&mut self, app: &tauri::App) {
        self.load_config(app);
        keymaps::init(app);
        dbg!(self);
    }
    fn set_source(&mut self, app: &tauri::App) {
        self.source = app
            .path()
            .resolve(&self.source, tauri::path::BaseDirectory::Resource)
            .expect("Should be valid path");
        if let Some(base_dirs) = BaseDirs::new() {
            let home_dir = base_dirs.home_dir();
            let source = home_dir.join(".config/fin/fin.toml");
            if source.exists() {
                self.source = source;
            }
        }
    }
    fn load_config(&mut self, app: &tauri::App) {
        self.set_source(app);
        let config_file_contents = std::fs::read_to_string(&self.source)
            .expect(&format!("{} does not exist", self.source.to_str().unwrap()));
        let parsed_config: std::collections::HashMap<String, String> =
            toml::from_str(&config_file_contents).expect("Should be valid toml");
        self.configs = parsed_config;
        return;
    }
}
