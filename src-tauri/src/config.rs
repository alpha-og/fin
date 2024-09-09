use std::{collections::HashMap, path::PathBuf, str::FromStr};

use directories::BaseDirs;
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
    pub fn init(&mut self) {
        self.load_config();
        dbg!(self);
    }
    fn set_source(&mut self) {
        if let Some(base_dirs) = BaseDirs::new() {
            let home_dir = base_dirs.home_dir();
            let source = home_dir.join(".config/fin/fin.toml");
            if source.exists() {
                self.source = source;
            }
        }
    }
    fn load_config(&mut self) {
        self.set_source();
        let config_file_contents = std::fs::read_to_string(&self.source).unwrap();
        let parsed_config: std::collections::HashMap<String, String> =
            toml::from_str(&config_file_contents).expect("Should be valid toml");
        self.configs = parsed_config;
        return;
    }
}
