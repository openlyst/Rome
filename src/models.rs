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
    pub screen_url: String,
    pub crc: String,
    pub md5: String,
    pub sha1: String,
    pub graphics: String,
    pub sound: String,
    pub gameplay: String,
    pub overall: String,
    pub publisher: String,
    pub serial: String,
    pub file_name: String,
    pub verified: String,
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
            screen_url: String::new(),
            crc: String::new(),
            md5: String::new(),
            sha1: String::new(),
            graphics: String::new(),
            sound: String::new(),
            gameplay: String::new(),
            overall: String::new(),
            publisher: String::new(),
            serial: String::new(),
            file_name: String::new(),
            verified: String::new(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rom_has_image_url_field() {
        let rom = Rom::new_basic(
            "Super Mario Bros".to_string(),
            "https://vimm.net/vault/12345".to_string(),
            "https://dl.vimm.net/?mediaId=12345".to_string(),
        );
        assert!(rom.image_url.is_empty());
    }

    #[test]
    fn game_box_art_url_format() {
        let id = "34905";
        let url = format!("https://dl.vimm.net/image.php?type=box&id={}", id);
        assert_eq!(url, "https://dl.vimm.net/image.php?type=box&id=34905");
    }
}
