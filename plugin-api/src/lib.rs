use std::collections::HashMap;

pub struct Metadata {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub url: Option<String>,
}

#[derive(PartialEq, Eq, Hash, Debug)] // derive for making enum comparisons
pub enum EventType {
    UpdateSearchQuery,
}

pub struct Event {
    pub event_type: EventType,
    pub data: Option<String>,
}

pub enum Response {
    Text(String),
    F64(f64),
}

pub struct BroadcastList {
    event_type: EventType,
    plugins: Vec<String>,
}

pub trait Plugin: Send {
    fn listen(&mut self, event: &Event);
    fn get_metadata(&self) -> Metadata;
    fn get_registered_events(&self) -> Vec<EventType>;
    fn get_response(&self) -> Option<Response>;
}

pub struct PluginManager {
    pub plugins: HashMap<String, Box<dyn Plugin>>,
    pub broadcast_lists: Vec<BroadcastList>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
            broadcast_lists: vec![],
        }
    }
}

impl PluginManager {
    // pub fn init(&self, plugin_directory: &str) {}
    pub fn init(&mut self) {
        self.broadcast_lists.push(BroadcastList {
            event_type: EventType::UpdateSearchQuery,
            plugins: Vec::new(),
        });
    }
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
    //         })
    //         .filter_map(Result::ok)
    //     {
    //         dbg!(entry.path());
    //     }
    // }
    pub fn register_plugin<T: Plugin + 'static>(&mut self, identifier: &str, plugin: T) {
        // register events requested by plugin
        let registered_events = plugin.get_registered_events();
        registered_events.iter().for_each(|event| {
            self.broadcast_lists
                .iter_mut()
                .filter(|broadcast_list| broadcast_list.event_type == *event)
                .collect::<Vec<_>>()
                .get_mut(0)
                .map_or_else(
                    || {},
                    |broadcast_list| {
                        broadcast_list.plugins.push(identifier.to_string());
                    },
                );
        });
        // register plugin
        self.plugins
            .insert(identifier.to_string(), Box::new(plugin));
        println!("Plugin {identifier} registered");
    }

    // function to broadcast an event to all plugins that are listening to the same event type
    pub fn broadcast(&mut self, event: Event) {
        self.broadcast_lists
            .iter_mut()
            .filter(|broadcast_list| broadcast_list.event_type == event.event_type)
            .collect::<Vec<_>>()
            .get_mut(0)
            .map_or_else(
                || {},
                |broadcast_list| {
                    broadcast_list
                        .plugins
                        .iter_mut()
                        .for_each(|plugin_identifier| {
                            if let Some(plugin) = self.plugins.get_mut(plugin_identifier) {
                                plugin.listen(&event);
                            }
                        });
                },
            );
    }

    pub fn get_responses(&mut self) -> Vec<Response> {
        let mut responses: Vec<Response> = vec![];
        self.plugins
            .iter()
            .for_each(|(_plugin_identifier, plugin)| {
                let response = plugin.get_response();
                if let Some(response) = response {
                    responses.push(response);
                }
            });
        responses
    }
}
