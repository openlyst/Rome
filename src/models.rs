use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rom {
    pub id: String,
    pub name: String,
    pub page_url: String,
    pub download_url: String,
    pub system: String,
    pub region: String,
    pub version: String,
    pub region_flags: Vec<String>,
    pub year: String,
    pub players: String,
    pub size: String,
    pub rating: String,
    pub description: String,
    pub image_url: String,
}

impl Rom {
    pub fn new_basic(name: String, page_url: String, download_url: String) -> Self {
        Self {
            id: String::new(),
            name,
            page_url,
            download_url,
            system: String::new(),
            region: String::new(),
            version: String::new(),
            region_flags: Vec::new(),
            year: String::new(),
            players: String::new(),
            size: String::new(),
            rating: String::new(),
            description: String::new(),
            image_url: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Console {
    pub name: String,
    pub slug: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Extracting,
    Done,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DownloadTask {
    pub id: u64,
    pub rom: Rom,
    pub status: DownloadStatus,
    pub progress: f32,
    pub save_path: String,
}

impl DownloadTask {
    pub fn new(rom: Rom, save_path: String) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self {
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            rom,
            status: DownloadStatus::Queued,
            progress: 0.0,
            save_path,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AppSettings {
    pub download_dir: String,
    pub auto_extract: bool,
    pub regions: Vec<i32>,
    pub version_filter: String, // "new", "old", "all"
}
