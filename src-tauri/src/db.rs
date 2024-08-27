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
    pub fn init(app: &mut tauri::App, _path: &str) {
        // let connection = Connection::open(format!("{path}/cache.db")).unwrap();
        app.manage(Db {
            connection: Arc::new(Mutex::new(Some(Connection::open_in_memory().unwrap()))),
        });
        let db_state = app.state::<Db>();
        let arced_connection = Arc::clone(&db_state.connection);
        std::thread::spawn(move || {
            let state_connection = arced_connection.lock().unwrap();
            let connection = state_connection.as_ref().unwrap();
            connection
                .execute(
                    "create table if not exists file_system(
        name text not null,
        path text not null)",
                    (),
                )
                .unwrap();
            Self::index_files(connection);
        });
    }
    fn index_files(connection: &Connection) {
        let mut files = HashMap::new();
        for entry in WalkDir::new("/Users/athulanoop/")
            .min_depth(1)
            .max_depth(5)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
        {
            files.insert(
                String::from(entry.file_name().to_string_lossy()),
                String::from(entry.path().to_string_lossy()),
            );
        }
        for (file_name, file_path) in files.iter() {
            if let Err(error) = connection.execute(
                &format!("insert into file_system values ('{file_name}', '{file_path}');"),
                (),
            ) {
                println!("{error}");
            } else {
            };
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
    if files.len() > 100 {
        Ok(files[..100].into())
    } else {
        Ok(files)
    }
}
