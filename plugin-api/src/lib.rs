use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex, MutexGuard},
    time,
};

pub struct Metadata {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub url: Option<String>,
}

pub trait Plugin: Send + Sync {
    fn init(&mut self, client_state_arc: Arc<Mutex<ClientState>>);
    fn start(&mut self);
    fn get_metadata(&self) -> Metadata;
    fn destroy(&mut self);
    fn clone_box(&self) -> Box<dyn Plugin>;
}

impl Clone for Box<dyn Plugin> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Clone, serde::Serialize, Debug)]
pub enum Action {
    Open,
    Copy,
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct SearchResult {
    title: String,
    description: Option<String>,
    icon: Option<Icon>,
    action: Option<Action>,
    priority: Option<u8>,
}

#[derive(Clone, serde::Serialize, Debug)]
pub enum Icon {
    File,
    Folder,
    Calculator,
}

impl SearchResult {
    pub fn new(
        title: String,
        description: Option<String>,
        icon: Option<Icon>,
        action: Option<Action>,
        mut priority: Option<u8>,
    ) -> Self {
        if let None = priority {
            priority = Some(0);
        }
        Self {
            title,
            description,
            icon,
            action,
            priority,
        }
    }
}

pub struct ClientState {
    search_query: String,
    search_results: Vec<SearchResult>,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            search_results: vec![],
        }
    }
}

struct Worker {
    id: usize,
    plugin_name: String,
    thread: std::thread::JoinHandle<()>,
}

pub struct PluginManager {
    pub plugins: HashMap<String, Box<dyn Plugin>>,
    client_state: Arc<Mutex<ClientState>>,
    workers: Vec<Worker>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
            client_state: Arc::new(Mutex::new(ClientState::default())),
            workers: vec![],
        }
    }
}

impl PluginManager {
    // pub fn init(&self, plugin_directory: &str) {}
    // pub fn index_third_party_plugins(plugin_directory: &str) {
    //     for entry in walkdir::WalkDir::new(plugin_directory)
    //         .min_depth(1)
    //         .into_iter()
    //         .filter_entry(|entry| {
    //             let file_name_substrings: Vec<String> = entry
    //                 .file_name()
    //                 .to_string_lossy()
    //                 .split(".")
    //                 .into_iter()
    //                 .map(|substring| substring.to_string())
    //                 .collect();
    //             let file_extension = file_name_substrings
    //                 .get(file_name_substrings.len() - 1)
    //                 .unwrap();
    //             file_extension == "so" || file_extension == "lua"
    //         }borrow)
    //         .filter_map(Result::ok)
    //     {
    //         dbg!(entry.path());
    //     }
    // }
    pub fn init(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        for mut plugin in plugins {
            let metadata = plugin.get_metadata();
            plugin.init(Arc::clone(&self.client_state));
            self.plugins.insert(metadata.name.clone(), plugin.clone());
            self.workers.push(Worker {
                id: 0,
                plugin_name: metadata.name.clone(),
                thread: std::thread::spawn(move || loop {
                    plugin.start();
                    std::thread::sleep(time::Duration::from_millis(100));
                }),
            });
            println!("Plugin {} initialized!", metadata.name);
        }
    }
    pub fn get_client_state(&self) -> MutexGuard<ClientState> {
        self.client_state
            .lock()
            .expect("Failed to lock client state")
    }
    pub fn get_client_state_arc(&self) -> Arc<Mutex<ClientState>> {
        Arc::clone(&self.client_state)
    }
    // fn load_plugin(&mut self, plugin_name: &str, plugin: Box<dyn Plugin>) {}
    // pub fn register_plugin(&mut self, plugin_name: &str, plugin: Box<dyn Plugin>) {}
}

impl ClientState {
    pub fn update_search_query(&mut self, query: String) {
        println!("Searching for: {}", query);
        self.search_query = query;
        self.search_results.clear();
    }
    pub fn update_search_results(&mut self, results: Vec<SearchResult>) {
        println!("Search results: {:?}", results);
        self.search_results = results;
    }
    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }
    pub fn get_search_results(&self) -> Vec<SearchResult> {
        self.search_results.clone()
    }
}
