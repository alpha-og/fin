use crate::db;
use sqlx::Row;

#[derive(Debug, Clone)]
pub enum CacheType {
    Filesystem,
}

#[derive(Debug, Clone)]
pub enum CacheStatus {
    Outdated,
    Updated,
    Updating,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub r#type: CacheType,
    pub status: CacheStatus,
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub filesystem: CacheEntry,
    pub filesystem_root: Option<String>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            filesystem: CacheEntry {
                r#type: CacheType::Filesystem,
                status: CacheStatus::Outdated,
            },
            filesystem_root: None,
        }
    }
}

impl Cache {
    pub async fn init(&mut self, db: &db::Db) {
        self.update_cache_states(db).await;
        self.cache_file_system(db, false, false).await;
    }
    async fn update_cache_states(&mut self, db: &db::Db) {
        let pool = match db.pool.as_ref() {
            Some(pool) => pool,
            None => return,
        };
        let result = sqlx::query("SELECT EXISTS (SELECT 1 FROM filesystem LIMIT 1)")
            .fetch_one(pool)
            .await
            .map(|row| row.get::<bool, _>(0))
            .unwrap_or(false);
        if result {
            dbg!("File system cache exists");
            self.filesystem.status = CacheStatus::Updated;
        } else {
            dbg!("File system cache does not exists");
        }
    }
    fn get_cache_status(&self) -> bool {
        dbg!(&self);
        if let CacheStatus::Updated = self.filesystem.status {
            true
        } else {
            false
        }
    }
    pub async fn cache_file_system(&mut self, db: &db::Db, upsert: bool, overwrite: bool) {
        if !(upsert || (!upsert && !self.get_cache_status())) {
            return;
        }
        if overwrite {
            dbg!("Overwriting file system cache");
            self.filesystem.status = CacheStatus::Outdated;
            let pool = db.pool.as_ref().unwrap();
            let _ = sqlx::query("DELETE FROM filesystem").execute(pool).await;
        }

        self.filesystem.status = CacheStatus::Updating;
        dbg!("Caching file system");
        let entries;
        if let Some(root) = &self.filesystem_root {
            println!("Indexing with root as {}", root);
            entries = db::fs::Fs::index_file_system(Some(root));
        } else {
            println!("Indexing with default root");
            entries = db::fs::Fs::index_file_system(None);
        }
        let pool = db.pool.as_ref().unwrap();
        let query = "INSERT OR REPLACE INTO filesystem (name, path, kind, ctime, mtime, atime) VALUES ($1, $2, $3, $4, $5, $6)";
        dbg!("Creating transactions for file system index");
        let mut tx = pool.begin().await.unwrap();
        for db::fs::Entry {
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
        self.filesystem.status = CacheStatus::Updated;
        dbg!("Completed caching file system");
    }
}
