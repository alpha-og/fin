use std::collections::HashMap;

pub trait Plugin: Send {
    fn init(&self);
    // fn get_plugin_info(&self);
    fn execute(
        &self,
        sender: std::sync::mpsc::Sender<Result<f64, String>>,
        fn_name: &str,
        args: Vec<String>,
    );
    fn destroy(&self);
}

pub struct PluginManager {
    pub plugins: HashMap<String, Box<dyn Plugin>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
}

impl PluginManager {
    // pub fn init(&self, plugin_directory: &str) {}
    pub fn index_third_party_plugins(plugin_directory: &str) {
        for entry in walkdir::WalkDir::new(plugin_directory)
            .min_depth(1)
            .into_iter()
            .filter_entry(|entry| {
                let file_name_substrings: Vec<String> = entry
                    .file_name()
                    .to_string_lossy()
                    .split(".")
                    .into_iter()
                    .map(|substring| substring.to_string())
                    .collect();
                let file_extension = file_name_substrings
                    .get(file_name_substrings.len() - 1)
                    .unwrap();
                file_extension == "so" || file_extension == "lua"
            })
            .filter_map(Result::ok)
        {
            dbg!(entry.path());
        }
    }
    pub fn register_plugin<T: Plugin + 'static>(&mut self, identifier: &str, plugin: T) {
        self.plugins
            .insert(identifier.to_string(), Box::new(plugin));
    }
}
