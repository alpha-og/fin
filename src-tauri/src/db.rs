mod cache;

use directories::BaseDirs;
use dotenvy;
use sqlx::Row;
use std::env;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct Db {
    pub connection_url: Option<String>,
    pub pool: Arc<Mutex<Option<sqlx::SqlitePool>>>,
    pub cache: Arc<Mutex<cache::Cache>>,
}
impl Default for Db {
    fn default() -> Self {
        Db {
            connection_url: None,
            pool: Arc::new(Mutex::new(None)),
            cache: Arc::new(Mutex::new(cache::Cache::default())),
        }
    }
}

impl Db {
    pub fn init(app: &mut tauri::App, database_url: Option<String>) {
        dotenvy::dotenv().ok();
        let mut db = Self::default();
        db.connection_url = Self::get_database_url(database_url);
        app.manage(Arc::new(db));

        let db_state = Arc::clone(&app.state::<Arc<Db>>());
        cache::Cache::create_cache_files(db_state.connection_url.as_deref().unwrap());
        std::thread::spawn(|| {
            tauri::async_runtime::block_on(async move {
                let pool = sqlx::SqlitePool::connect(db_state.connection_url.as_deref().unwrap())
                    .await
                    .unwrap();
                {
                    let mut pool_state = db_state.pool.lock().unwrap();
                    *pool_state = Some(pool);
                }

                Self::run_migrations(&db_state.pool).await;
                cache::Cache::update_cache_states(&db_state).await;
                cache::Cache::cache_file_system(&db_state, false).await;
            })
        });
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

    async fn run_migrations(pool: &Arc<Mutex<Option<sqlx::SqlitePool>>>) {
        let pool_state = pool.lock().unwrap();
        let pool = pool_state.as_ref().unwrap();
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
    pub kind: cache::EntryKind,
}

#[tauri::command]
pub async fn get_files(
    app_handle: tauri::AppHandle,
    filter: String,
) -> Result<Vec<EntryResponse>, String> {
    let db_state = app_handle.state::<Arc<Db>>();
    let pool_arc = Arc::clone(&db_state.pool);
    let filter = format!("%{filter}%");
    let files = std::thread::spawn(move || {
        let pool_state = pool_arc.lock().unwrap();
        let pool = pool_state.as_ref().unwrap();

        tauri::async_runtime::block_on(async {
            let records = sqlx::query(
                "SELECT * FROM filesystem WHERE name LIKE $1 OR path LIKE $2 ORDER BY CASE WHEN kind = 'application' THEN 0 ELSE 1 END ,atime DESC LIMIT 100",
            )
            .bind(&filter)
            .bind(&filter)
            .fetch_all(pool)
            .await
            .unwrap();
            records
                .iter()
                .map(|record| EntryResponse {
                    name: record.get("name"),
                    path: record.get("path"),
                    kind: cache::EntryKind::from(record.get::<&str, _>("kind")),
                })
                .collect::<Vec<EntryResponse>>()
        })
    })
    .join()
    .unwrap();

    // TODO: Add logic to re-index when no results are returned from search
    // query ONLY if indexing is currently not in progress
    //
    // if files.len() == 0 {
    //     Db::cache_file_system(&db_state);
    // }
    Ok(files)
}
