use std::{collections::HashMap, path::PathBuf};

use directories::BaseDirs;
use toml;

#[derive(serde::Serialize, Debug)]
pub struct Config {
    configs: HashMap<String, String>,
    source: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let configs = HashMap::new();
        Self {
            configs,
            source: None,
        }
    }
}

impl Config {
    pub fn init(&mut self) {
        self.load_config();
        dbg!(self);
    }
    fn load_config(&mut self) {
        if let Some(base_dirs) = BaseDirs::new() {
            let home_dir = base_dirs.home_dir();
            let config_file_path = home_dir.join(".config/fin/fin.toml");
            let config_file_contents = std::fs::read_to_string(&config_file_path).unwrap();
            let parsed_config: std::collections::HashMap<String, String> =
                toml::from_str(&config_file_contents).expect("Should be valid toml");
            self.configs = parsed_config;
            self.source = Some(config_file_path);
        } else {
        }
    }
}
