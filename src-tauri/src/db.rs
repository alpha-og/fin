use directories::BaseDirs;
use dotenvy;
use sqlx::Row;
use std::collections::HashMap;
use std::env;
use std::os::unix::fs::MetadataExt;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use walkdir::WalkDir;

#[derive(serde::Serialize, Clone)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
    Application,
}

impl From<&str> for EntryKind {
    fn from(value: &str) -> Self {
        match value {
            "file" => Self::File,
            "directory" => Self::Directory,
            "symlink" => Self::Symlink,
            "application" => Self::Application,
            _ => panic!("Failed to parse file kind!"),
        }
    }
}

impl EntryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::Application => "application",
        }
    }
}

#[derive(serde::Serialize, Clone)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
    pub ctime: i64,
    pub mtime: i64,
    pub atime: i64,
}

pub struct Db {
    pub pool: Arc<Mutex<Option<sqlx::SqlitePool>>>,
    pub cache_status: Arc<Mutex<HashMap<String, bool>>>,
}

impl Db {
    pub fn init(app: &mut tauri::App, database_url: Option<String>) {
        dotenvy::dotenv().ok();
        let database_url = Self::get_database_url(database_url);
        Self::create_cache_files(&database_url);
        dbg!(&database_url);

        app.manage(Arc::new(Db {
            pool: Arc::new(Mutex::new(None)),
            cache_status: Arc::new(Mutex::new(HashMap::new())),
        }));

        let db_state = Arc::clone(&app.state::<Arc<Db>>());
        std::thread::spawn(|| {
            tauri::async_runtime::block_on(async move {
                let pool = sqlx::SqlitePool::connect(&database_url).await.unwrap();
                {
                    let mut pool_state = db_state.pool.lock().unwrap();
                    *pool_state = Some(pool);
                }
                Self::run_migrations(&db_state.pool).await;
                Self::update_cache_states(&db_state).await;
                Self::cache_file_system(&db_state, false).await;
            })
        });
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
    fn create_cache_files(database_url: &str) {
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

    async fn run_migrations(pool: &Arc<Mutex<Option<sqlx::SqlitePool>>>) {
        let pool_state = pool.lock().unwrap();
        let pool = pool_state.as_ref().unwrap();
        if let Ok(()) = sqlx::migrate!().run(pool).await {
            dbg!("Migrations completed");
        } else {
            dbg!("Migrations failed");
        }
    }

    async fn update_cache_states(db_state: &Arc<Db>) {
        let pool_state = db_state.pool.lock().unwrap();
        let pool = pool_state.as_ref().unwrap();
        let result = sqlx::query("SELECT EXISTS (SELECT 1 FROM filesystem LIMIT 1)")
            .fetch_one(pool)
            .await
            .map(|row| row.get::<bool, _>(0))
            .unwrap_or(false);
        if result {
            dbg!("File system cache exists");
            let mut cache_status = db_state.cache_status.lock().unwrap();
            cache_status.insert("filesystem".to_string(), true);
        } else {
            dbg!("File system cache does not exists");
        }
    }
    fn check_cache_status(cache_status: &Arc<Mutex<HashMap<String, bool>>>) -> bool {
        dbg!("test");
        let cache_status = cache_status.lock().unwrap();
        dbg!(&cache_status);
        if let Some(filesystem_cache_status) = cache_status.get("filesystem") {
            if *filesystem_cache_status {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn index_file_system() -> Vec<Entry> {
        let mut entries = Vec::new();

        // index files
        for entry in WalkDir::new(BaseDirs::new().unwrap().home_dir())
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
            let metadata = entry.metadata();
            if let Ok(metadata) = metadata {
                let kind = if metadata.is_file() {
                    EntryKind::File
                } else if metadata.is_dir() {
                    EntryKind::Directory
                } else {
                    EntryKind::Symlink
                };
                entries.push(Entry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    kind,
                    ctime: metadata.ctime(),
                    mtime: metadata.mtime(),
                    atime: metadata.atime(),
                })
            }
        }

        // index all applications
        for entry in WalkDir::new("/Applications/")
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|entry| {
                let file_name_substrings: Vec<String> = entry
                    .file_name()
                    .to_string_lossy()
                    .split(".")
                    .into_iter()
                    .map(|substring| substring.to_string())
                    .collect();
                file_name_substrings
                    .get(file_name_substrings.len() - 1)
                    .unwrap()
                    .contains("app")
            })
            .filter_map(Result::ok)
        {
            let metadata = entry.metadata().unwrap();
            entries.push(Entry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                kind: EntryKind::Application,
                ctime: metadata.ctime(),
                mtime: metadata.mtime(),
                atime: metadata.atime(),
            })
        }
        entries
    }
    async fn cache_file_system(db_state: &Arc<Db>, overwrite: bool) {
        if !(overwrite || (!overwrite && !Self::check_cache_status(&db_state.cache_status))) {
            return;
        }

        dbg!("Caching file system");
        let pool_state = db_state.pool.lock().unwrap();
        let entries = Self::index_file_system();
        let pool = pool_state.as_ref().unwrap();
        let query = "INSERT OR REPLACE INTO filesystem (name, path, kind, ctime, mtime, atime) VALUES ($1, $2, $3, $4, $5, $6)";
        dbg!("Creating transactions for file system index");
        let mut tx = pool.begin().await.unwrap();
        for Entry {
            name,
            path,
            kind,
            ctime,
            mtime,
            atime,
        } in entries
        {
            let _ = sqlx::query(query)
                .bind(&name)
                .bind(&path)
                .bind(kind.as_str())
                .bind(&ctime)
                .bind(&mtime)
                .bind(&atime)
                .execute(&mut *tx)
                .await;
        }
        dbg!("Committing transactions for file system index");
        tx.commit().await.unwrap();
        dbg!("Completed caching file system");
    }
}

#[derive(serde::Serialize, Clone)]
pub struct EntryResponse {
    pub name: String,
    pub path: String,
    pub kind: EntryKind,
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
                    kind: EntryKind::from(record.get::<&str, _>("kind")),
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
