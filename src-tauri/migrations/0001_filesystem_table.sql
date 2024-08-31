CREATE TABLE IF NOT EXISTS filesystem (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL, 
    path TEXT UNIQUE NOT NULL
    -- kind TEXT CHECK(type in ('file', 'directory')) NOT NULL ,
    -- ctime DATETIME NOT NULL,
    -- mtime DATETIME NOT NULL,
    -- atime DATETIME NOT NULL,
    -- created_at DEFAULT CURRENT_TIMESTAMP,
    -- modified_at DEFAULT CURRENT_TIMESTAMP
); 
CREATE UNIQUE INDEX IF NOT EXISTS idx_name ON filesystem(path);
