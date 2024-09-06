use crate::db::{self, BaseDirs};
use sqlx::Row;
use std::os::unix::fs::MetadataExt;
use std::sync::{Arc, Mutex};
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

#[derive(Debug)]
pub enum CacheType {
    Filesystem,
}

#[derive(Debug)]
pub enum CacheStatus {
    Outdated,
    Updated,
    Updating,
}

#[derive(Debug)]
pub struct CacheEntry {
    pub r#type: CacheType,
    pub status: CacheStatus,
}

#[derive(Debug)]
pub struct Cache {
    filesystem: CacheEntry,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            filesystem: CacheEntry {
                r#type: CacheType::Filesystem,
                status: CacheStatus::Outdated,
            },
        }
    }
}

impl Cache {
    // pub fn init(&self, db: &Arc<db::Db>) {
    //     Self::create_cache_files(&db.connection_url.unwrap());
    //     self.update_cache_states(db);
    // }
    pub fn create_cache_files(database_url: &str) {
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
    pub async fn update_cache_states(db_state: &Arc<db::Db>) {
        let pool_state = db_state.pool.lock().unwrap();
        let pool = pool_state.as_ref().unwrap();
        let result = sqlx::query("SELECT EXISTS (SELECT 1 FROM filesystem LIMIT 1)")
            .fetch_one(pool)
            .await
            .map(|row| row.get::<bool, _>(0))
            .unwrap_or(false);
        if result {
            dbg!("File system cache exists");
            let mut cache_status = db_state.cache.lock().unwrap();
            cache_status.filesystem.status = CacheStatus::Updated;
        } else {
            dbg!("File system cache does not exists");
        }
    }
    pub fn get_cache_status(cache: &Arc<Mutex<Cache>>) -> bool {
        let cache = cache.lock().unwrap();
        dbg!(&cache);
        if let CacheStatus::Updated = cache.filesystem.status {
            true
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
    pub async fn cache_file_system(db_state: &Arc<db::Db>, overwrite: bool) {
        if !(overwrite || (!overwrite && !Self::get_cache_status(&db_state.cache))) {
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
