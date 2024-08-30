use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri::State;
use walkdir::WalkDir;

#[derive(serde::Serialize, Clone)]
pub struct File {
    pub name: String,
    pub path: String,
}
pub struct Db {
    pub connection: Arc<Mutex<Option<Connection>>>,
}
impl Default for Db {
    fn default() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
        }
    }
}
impl Db {
    pub fn init(app: &mut tauri::App, path: &str) {
        let connection = Connection::open(format!("{path}/file_system_cache.sqlite"))
            .map_or(None, |connection| Some(connection));
        app.manage(Db {
            connection: Arc::new(Mutex::new(connection)),
        });
        let db_state = app.state::<Db>();
        Db::create_tables(&db_state);
        Db::cache_file_system(&db_state);
    }
    fn create_tables(db_state: &State<Db>) {
        let connection_arc = Arc::clone(&db_state.connection);
        std::thread::spawn(move || {
            let mut state_connection = connection_arc.lock().unwrap();
            if let Some(connection) = state_connection.as_mut() {
                let transaction = connection.transaction().unwrap();
                transaction
                    .execute(
                        "CREATE TABLE IF NOT EXISTS file_system(
        name TEXT NOT NULL,
        path TEXT NOT NULL)",
                        (),
                    )
                    .unwrap();
                transaction
                    .execute(
                        "CREATE UNIQUE INDEX IF NOT EXISTS idx_name ON file_system(name);",
                        (),
                    )
                    .unwrap();
                transaction.execute("CREATE TABLE IF NOT EXISTS query_history(query TEXT NOT NULL, created_at DEFAULT CURRENT_TIMESTAMP, modified_at DEFAULT CURRENT_TIMESTAMP)", ()).unwrap();
                transaction
                    .execute(
                        "CREATE UNIQUE INDEX IF NOT EXISTS idx_query ON query_history(query);",
                        (),
                    )
                    .unwrap();

                let _ = transaction.commit();
            } else {
                println!("Database connection could not be acquired");
            }
        });
    }
    fn index_file_system() -> HashMap<String, String> {
        let mut files = HashMap::new();
        for entry in WalkDir::new("/Users/athulanoop/")
            .min_depth(1)
            .max_depth(5)
            .follow_links(true)
            .into_iter()
            .filter_entry(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|file_name| !file_name.starts_with("."))
                    .unwrap_or(false)
            })
            .filter_map(Result::ok)
        {
            files.insert(
                String::from(entry.file_name().to_string_lossy()),
                String::from(entry.path().to_string_lossy()),
            );
        }
        files
    }
    fn cache_file_system(db_state: &State<Db>) {
        let connection_arc = Arc::clone(&db_state.connection);
        std::thread::spawn(move || {
            let files = Self::index_file_system();
            let mut state_connection = connection_arc.lock().unwrap();
            if let Some(connection) = state_connection.as_mut() {
                let transaction = connection.transaction().unwrap();
                for (file_name, file_path) in files.iter() {
                    if let Err(error) = transaction.execute(
                        &format!(
                    "insert or replace into file_system values ('{file_name}', '{file_path}');"
                ),
                        (),
                    ) {
                        println!("{error} | {file_name} | {file_path}");
                    } else {
                    };
                }
                let _ = transaction.commit();
            }
        });
    }
    // fn cache_query_history(connection: &mut Connection, queries: Vec<String>) {
    //     std::thread::spawn(move || {});
    // }
}

#[tauri::command]
pub async fn get_files(state: State<'_, Db>, filter: String) -> Result<Vec<File>, String> {
    let mut state = state.connection.lock().unwrap();
    let connection = state.as_mut().unwrap();
    let mut query = connection.prepare("select * from file_system").unwrap();
    let rows = query
        .query_map((), |row| {
            Ok(File {
                name: row.get(0).unwrap(),
                path: row.get(1).unwrap(),
            })
        })
        .unwrap();
    let mut files = Vec::new();
    for row in rows {
        let row = row.unwrap();
        if row.name.contains(&filter) {
            files.push(row);
        }
    }
    if files.len() > 100 {
        Ok(files[..100].into())
    } else {
        Ok(files)
    }
}
