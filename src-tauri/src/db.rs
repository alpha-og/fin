use directories::BaseDirs;
use dotenvy::dotenv;
use sqlx::Row;
use std::collections::HashMap;
use std::env;
use std::ops::Deref;
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
    pub pool: Arc<Mutex<sqlx::SqlitePool>>,
}

impl Db {
    pub fn init(app: &mut tauri::App, database_url: Option<String>) {
        dotenv().ok();
        let database_url = Self::get_database_url(database_url);
        let pool =
            tauri::async_runtime::block_on(sqlx::SqlitePool::connect(&database_url)).unwrap();
        app.manage(Db {
            pool: Arc::new(Mutex::new(pool)),
        });
        let db_state = app.state::<Db>();
        let pool_arc = Arc::clone(&db_state.pool);
        std::thread::spawn(move || {
            let pool_state = pool_arc.lock().unwrap();
            let pool = pool_state.deref();
            tauri::async_runtime::block_on(sqlx::migrate!().run(pool)).unwrap();
        });

        let db_state = app.state::<Db>();
        Db::cache_file_system(&db_state);
    }
    fn get_database_url(database_url: Option<String>) -> String {
        if let Some(database_url) = database_url {
            database_url
        } else {
            match env::var("DATABASE_URL") {
                Ok(database_url) => database_url,
                Err(..) => {
                    if let Some(base_dirs) = BaseDirs::new() {
                        let home_dir = base_dirs.home_dir();
                        println!("Default path");
                        String::from(
                            std::path::Path::join(home_dir, ".config/fin/cache.sqlite")
                                .to_str()
                                .unwrap(),
                        )
                    } else {
                        String::from("sqlite:cache.sqlite")
                    }
                }
            }
        }
    }

    fn index_file_system() -> HashMap<String, String> {
        let mut files = HashMap::new();
        for entry in WalkDir::new("/Users/athulanoop/Software Projects/")
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
        dbg!("Caching file system");
        let pool_arc = Arc::clone(&db_state.pool);
        std::thread::spawn(move || {
            let files = Self::index_file_system();
            let pool = pool_arc.lock().unwrap();
            tauri::async_runtime::block_on(async {
                let query = "INSERT OR REPLACE INTO filesystem (name, path) VALUES ($1, $2)";
                dbg!("Creating transactions for file system index");
                let mut tx = pool.begin().await.unwrap();
                for (name, path) in files {
                    let _ = sqlx::query(query)
                        .bind(&name)
                        .bind(&path)
                        .execute(&mut *tx)
                        .await;
                }
                dbg!("Committing transactions for file system index");
                tx.commit().await.unwrap();
                dbg!("Completed caching file system");
            })
        });
    }
    // fn cache_query_history(connection: &mut Connection, queries: Vec<String>) {
    //     std::thread::spawn(move || {});
    // }
}

#[tauri::command]
pub async fn get_files(app_handle: tauri::AppHandle, filter: String) -> Result<Vec<File>, String> {
    let db_state = app_handle.state::<Db>();
    let pool_arc = Arc::clone(&db_state.pool);
    let files = std::thread::spawn(move || {
        let pool_state = pool_arc.lock().unwrap();
        let pool = pool_state.deref();

        tauri::async_runtime::block_on(async {
            let records = sqlx::query(&format!(
                "SELECT * FROM filesystem WHERE name like '%{filter}%' OR path like '%{filter}%' LIMIT 100"
            ))
            .fetch_all(pool)
            .await
            .unwrap();
            records
                .iter()
                .map(|record| File {
                    name: record.get("name"),
                    path: record.get("path"),
                })
                .collect::<Vec<File>>()
        })
    })
    .join()
    .unwrap();
    Ok(files)
}
