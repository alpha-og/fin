use crate::db::{self, fs};
use sqlx::Row;

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
    pub filesystem: CacheEntry,
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
    pub async fn init(&mut self, db: &db::Db) {
        self.update_cache_states(db).await;
        self.cache_file_system(db, false).await;
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
    pub async fn cache_file_system(&mut self, db: &db::Db, overwrite: bool) {
        if !(overwrite || (!overwrite && !self.get_cache_status())) {
            return;
        }

        self.filesystem.status = CacheStatus::Updating;
        dbg!("Caching file system");
        let entries = fs::Fs::index_file_system();
        let pool = db.pool.as_ref().unwrap();
        let query = "INSERT OR REPLACE INTO filesystem (name, path, kind, ctime, mtime, atime) VALUES ($1, $2, $3, $4, $5, $6)";
        dbg!("Creating transactions for file system index");
        let mut tx = pool.begin().await.unwrap();
        for fs::Entry {
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
