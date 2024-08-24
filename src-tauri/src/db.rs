use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use walkdir::WalkDir;

#[derive(serde::Serialize)]
pub struct File {
    pub name: String,
    pub path: String,
}
pub struct Db {
    pub connection: Mutex<Option<Connection>>,
}
impl Default for Db {
    fn default() -> Self {
        Self {
            connection: Mutex::new(None),
        }
    }
}
impl Db {
    pub fn init(_path: &str) -> Option<Connection> {
        // let connection = Connection::open(format!("{path}/cache.db")).unwrap();
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute(
                "create table if not exists file_system(
        name text not null,
        path text not null)",
                (),
            )
            .unwrap();
        Self::index_files(&connection);
        Some(connection)
    }
    fn index_files(connection: &Connection) {
        let mut files = HashMap::new();
        for entry in WalkDir::new("/Users/athulanoop/.config/")
            .min_depth(1)
            .max_depth(5)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                files.insert(
                    String::from(entry.file_name().to_string_lossy()),
                    String::from(entry.path().to_string_lossy()),
                );
            }
        }
        for (file_name, file_path) in files.iter() {
            connection
                .execute(
                    &format!("insert into file_system values ('{file_name}', '{file_path}');"),
                    (),
                )
                .unwrap();
        }
    }
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
    Ok(files)
}
