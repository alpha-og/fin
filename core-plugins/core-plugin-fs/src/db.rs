pub mod fs;

use directories::BaseDirs;
use dotenvy;
use std::env;

#[derive(Clone)]
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

        tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(async {
                dbg!("Connecting to database: {:?}", &self.connection_url);
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
            });
    }

    fn get_database_url(database_url: Option<String>) -> Option<String> {
        if let Some(database_url) = database_url {
            println!("Using custom database URL");
            Some(database_url)
        } else {
            match env::var("DATABASE_URL") {
                Ok(database_url) => Some(database_url),
                Err(..) => {
                    if let Some(base_dirs) = BaseDirs::new() {
                        let home_dir = base_dirs.home_dir();
                        println!("DATABASE_URL not found, using default path");
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
