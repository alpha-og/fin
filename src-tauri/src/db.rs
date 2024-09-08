pub mod fs;

use directories::BaseDirs;
use dotenvy;
use sqlx::Row;
use std::env;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct Db {
    pub connection_url: Option<String>,
    pub pool: Option<sqlx::SqlitePool>,
}
impl Default for Db {
    fn default() -> Self {
        Db {
            connection_url: None,
            pool: None,
        }
    }
}

impl Db {
    pub fn init(&mut self, database_url: Option<String>) {
        dotenvy::dotenv().ok();
        self.connection_url = Self::get_database_url(database_url);
        Self::create_db_files(&self.connection_url.as_ref().unwrap());

        tauri::async_runtime::block_on(async move {
            let pool = sqlx::SqlitePool::connect(&self.connection_url.as_ref().unwrap())
                .await
                .unwrap();
            self.pool = Some(pool);

            Self::run_migrations(
                self.pool
                    .as_ref()
                    .expect("SQLite connection pool should be Some"),
            )
            .await;
        })
    }

    fn get_database_url(database_url: Option<String>) -> Option<String> {
        if let Some(database_url) = database_url {
            Some(database_url)
        } else {
            match env::var("DATABASE_URL") {
                Ok(database_url) => Some(database_url),
                Err(..) => {
                    if let Some(base_dirs) = BaseDirs::new() {
                        let home_dir = base_dirs.home_dir();
                        println!("Default path");
                        std::path::Path::join(home_dir, ".config/fin/cache.sqlite")
                            .to_str()
                            .map(|path| path.to_string())
                    } else {
                        Some(String::from("sqlite:cache.sqlite"))
                    }
                }
            }
        }
    }
    fn create_db_files(database_url: &str) {
        if !std::path::Path::new(database_url).exists() {
            let path = std::path::Path::new(database_url);
            let prefix = path.parent().unwrap();
            std::fs::create_dir_all(prefix).unwrap();
            if let Ok(_file) = std::fs::File::create_new(path) {
                dbg!("created file");
            } else {
                panic!("Unable to create file");
            }
        }
    }

    async fn run_migrations(pool: &sqlx::SqlitePool) {
        if let Ok(()) = sqlx::migrate!().run(pool).await {
            dbg!("Migrations completed");
        } else {
            dbg!("Migrations failed");
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct EntryResponse {
    pub name: String,
    pub path: String,
    pub kind: fs::EntryKind,
}

#[tauri::command]
pub async fn get_files(
    app_handle: tauri::AppHandle,
    filter: String,
) -> Result<Vec<EntryResponse>, String> {
    let db_state = app_handle.state::<Arc<Mutex<Db>>>();
    let db_arc = Arc::clone(&db_state);
    let filter = format!("%{filter}%");
    let files = std::thread::spawn(move || loop{
        let db_guard = db_arc.try_lock();
        if db_guard.is_ok(){
            let db = db_guard.expect("Thread should not be poisoned");
            return tauri::async_runtime::block_on(async {
                let records = sqlx::query(
                    "SELECT * FROM filesystem WHERE name LIKE $1 OR path LIKE $2 ORDER BY CASE WHEN kind = 'application' THEN 0 ELSE 1 END ,atime DESC LIMIT 100",
                )
                .bind(&filter)
                .bind(&filter)
                .fetch_all(db.pool.as_ref().expect("SQLite connection pool must be Some not None to query database"))
                .await
                .unwrap();
                return records
                    .iter()
                    .map(|record| EntryResponse {
                        name: record.get("name"),
                        path: record.get("path"),
                        kind: fs::EntryKind::from(record.get::<&str, _>("kind")),
                    })
                    .collect::<Vec<EntryResponse>>();
            });
        };
    })
    .join()
    .unwrap();
    Ok(files)
}
