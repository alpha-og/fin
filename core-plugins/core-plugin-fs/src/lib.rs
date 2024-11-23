mod cache;
mod db;

use plugin_api::Plugin;
use sqlx::Row;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct FsPlugin {
    results: Option<Vec<db::fs::Entry>>,
    db: db::Db,
    cache: cache::Cache,
    client_state: Arc<Mutex<plugin_api::ClientState>>,
}

impl Default for FsPlugin {
    fn default() -> Self {
        Self {
            db: db::Db::default(),
            cache: cache::Cache::default(),
            results: Some(Vec::new()),
            client_state: Arc::new(Mutex::new(plugin_api::ClientState::default())),
        }
    }
}

impl Plugin for FsPlugin {
    fn init(&mut self, client_state_arc: Arc<Mutex<plugin_api::ClientState>>) {
        self.client_state = client_state_arc;
        self.db.init(Some(
            "/Users/athulanoop/.config/fin/cache.sqlite".to_string(),
        ));

        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(self.cache.init(&self.db));

        println!("Calculator plugin initialized!");
    }

    fn start(&mut self) {
        let mut client_state = self
            .client_state
            .lock()
            .expect("Failed to lock client state");

        let query = client_state.get_search_query();
        let result = self.get_files(query);
        if let Ok(result) = result {
            if let Some(existing_result) = &self.results {
                let mut same = true;
                for result in &result {
                    if !existing_result.contains(&result) {
                        same = false;
                    }
                }
                if same {
                    return;
                }
            }
            self.results = Some(result);
        } else {
            self.results = None;
        }
        if let Some(results) = &self.results {
            let existing_results = client_state.get_search_results();
            let mut new_results = Vec::new();
            for result in existing_results {
                new_results.push(result);
            }
            for result in results {
                let icon = match result.kind.as_str() {
                    "application" => None,
                    "directory" => Some(plugin_api::Icon::Folder),
                    "file" => Some(plugin_api::Icon::File),
                    _ => None,
                };
                let action = match result.kind.as_str() {
                    "application" => {
                        Some(plugin_api::Action::LaunchApplication(result.path.clone()))
                    }
                    "directory" | "file" => Some(plugin_api::Action::Open(result.path.clone())),
                    _ => None,
                };
                new_results.push(plugin_api::SearchResult::new(
                    result.name.clone(),
                    None,
                    icon,
                    action,
                    Some(10),
                ));
            }
            client_state.update_search_results(new_results);
        }
    }
    fn get_metadata(&self) -> plugin_api::Metadata {
        plugin_api::Metadata {
            name: "FS Walk".to_string(),
            description: "A plugin to traverse the filesystem".to_string(),
            icon: None,
            url: None,
        }
    }
    fn destroy(&mut self) {
        println!("Calculator plugin destroyed!");
    }
    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(self.clone())
    }

    fn get_config(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
}

impl FsPlugin {
    fn get_files(&self, filter: &str) -> Result<Vec<db::fs::Entry>, String> {
        let filter = format!("%{filter}%");
        let files = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(async {
                let records = sqlx::query(
                    "SELECT * FROM filesystem WHERE name LIKE $1 OR path LIKE $2 ORDER BY CASE WHEN kind = 'application' THEN 0 ELSE 1 END ,atime DESC LIMIT 100",
                )
                .bind(&filter)
                .bind(&filter)
                .fetch_all(self.db.pool.as_ref().expect("SQLite connection pool must be valid to query database"))
                .await
                .unwrap();
                return records
                    .iter()
                    .map(|record| db::fs::Entry{
                        name: record.get("name"),
                        path: record.get("path"),
                        kind: db::fs::EntryKind::from(record.get::<&str, _>("kind")),
                        ctime: record.get("ctime"),
                        mtime: record.get("mtime"),
                        atime: record.get("atime"),
                    })
                    .collect::<Vec<db::fs::Entry>>();

            });
        Ok(files)
    }
}
