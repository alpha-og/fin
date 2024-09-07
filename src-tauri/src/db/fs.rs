use directories::BaseDirs;
use std::os::unix::fs::MetadataExt;
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
pub struct Fs {}

impl Fs {
    pub fn index_file_system() -> Vec<Entry> {
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
}
