use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub trait Plugin: Send {
    fn init(&mut self, event_bus: EventBus);
    // fn get_plugin_info(&self);
    fn listen(&mut self, _event_name: &str) {}
    fn emit(&mut self, _event_name: &str, _payload: PluginEventPayload<String>) {}
    fn destroy(&self);
}

#[derive(Clone)]
pub enum PluginEventPayload<T> {
    List(Vec<T>),
    Single(T),
}

pub type EventBus = Arc<Mutex<HashMap<String, PluginEventPayload<String>>>>;

pub struct PluginManager {
    plugins: HashMap<String, Box<Arc<Mutex<dyn Plugin>>>>,
    pub event_bus: EventBus,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
            event_bus: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl PluginManager {
    pub fn init_plugins(&mut self) {
        println!("Initialising plugins");
        for (_identifier, plugin) in self.plugins.iter() {
            let event_bus_arc = self.event_bus.clone();
            let plugin_arc = plugin.clone();
            std::thread::spawn(move || {
                let mut plugin = plugin_arc.lock().expect("Should not be poisoned");
                plugin.init(event_bus_arc);
            });
        }
    }
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
    pub fn register_plugin<U: 'static + Plugin>(&mut self, identifier: &str, plugin: U) {
        self.plugins.insert(
            identifier.to_string(),
            Box::new(Arc::new(Mutex::new(plugin))),
        );
    }
    pub fn emit(&mut self, event_name: &str, payload: PluginEventPayload<String>) {
        let mut event_bus = self.event_bus.lock().expect("Should not be poisoned");
        event_bus.insert(event_name.to_string(), payload);
    }
    pub fn listen<F>(&self, event_name: &str, handler: F)
    where
        F: Fn(&PluginEventPayload<String>),
    {
        let event_bus = self.event_bus.lock().expect("Should not be poisoned");
        if let Some(payload) = event_bus.get(event_name) {
            handler(payload)
        }
    }
}
