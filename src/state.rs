use crate::models::{AppSettings, DownloadTask, Rom};
use dioxus::prelude::*;
use std::path::PathBuf;

fn download_base_dir() -> PathBuf {
    dirs::download_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VimmsROMs")
}

fn downloaded_json_path() -> PathBuf {
    download_base_dir().join("downloaded.json")
}

fn main_json_path() -> PathBuf {
    download_base_dir().join("main.json")
}

fn load_downloaded_roms() -> Vec<Rom> {
    let path = downloaded_json_path();
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_downloaded_roms(roms: &[Rom]) {
    let path = downloaded_json_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(roms) {
        let _ = std::fs::write(path, text);
    }
}

fn system_slug(system: &str) -> String {
    system.to_lowercase().replace(" ", "").replace("-", "")
}

fn rom_download_path(download_dir: &str, rom: &Rom) -> String {
    let sys = if rom.system.is_empty() {
        "unknown".to_string()
    } else {
        system_slug(&rom.system)
    };
    PathBuf::from(download_dir)
        .join(&sys)
        .join(format!("{}.zip", rom.id))
        .to_string_lossy()
        .to_string()
}

fn rom_extract_dir(download_dir: &str, rom: &Rom) -> String {
    let sys = if rom.system.is_empty() {
        "unknown".to_string()
    } else {
        system_slug(&rom.system)
    };
    PathBuf::from(download_dir)
        .join(&sys)
        .to_string_lossy()
        .to_string()
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub search_query: Signal<String>,
    pub selected_console: Signal<Option<String>>,
    pub downloads: Signal<Vec<DownloadTask>>,
    pub downloaded_roms: Signal<Vec<Rom>>,
    pub settings: Signal<AppSettings>,
    pub current_rom: Signal<Option<Rom>>,
}

impl AppState {
    pub fn new() -> Self {
        let download_dir = download_base_dir().to_string_lossy().to_string();

        Self {
            search_query: Signal::new(String::new()),
            selected_console: Signal::new(None),
            downloads: Signal::new(Vec::new()),
            downloaded_roms: Signal::new(load_downloaded_roms()),
            settings: Signal::new(AppSettings {
                download_dir,
                regions: vec![8, 14, 26],
                version_filter: "new".into(),
            }),
            current_rom: Signal::new(None),
        }
    }

    pub fn is_downloaded(&self, rom_id: &str) -> bool {
        self.downloaded_roms.read().iter().any(|r| r.id == rom_id)
    }

    pub fn mark_downloaded(&mut self, rom: Rom) {
        if self.is_downloaded(&rom.id) {
            return;
        }
        let mut list = self.downloaded_roms.write();
        list.push(rom);
        save_downloaded_roms(&list);
    }

    pub fn remove_downloaded(&mut self, rom_id: &str) {
        let mut list = self.downloaded_roms.write();
        list.retain(|r| r.id != rom_id);
        save_downloaded_roms(&list);
    }

    pub fn queue_download(&mut self, rom: Rom) {
        if self.is_downloaded(&rom.id) {
            return;
        }
        let settings = self.settings.read();
        let path = rom_download_path(&settings.download_dir, &rom);

        let mut list = self.downloads.write();
        if list.iter().any(|d| d.rom.id == rom.id) {
            return;
        }
        list.push(DownloadTask::new(rom, path));
    }

    pub fn remove_download(&mut self, id: u64) {
        let mut list = self.downloads.write();
        list.retain(|d| d.id != id);
    }

    pub fn update_download_status(&mut self, id: u64, status: crate::models::DownloadStatus) {
        let mut list = self.downloads.write();
        if let Some(task) = list.iter_mut().find(|d| d.id == id) {
            task.status = status;
        }
    }

    pub fn update_download_progress(&mut self, id: u64, progress: f32) {
        let mut list = self.downloads.write();
        if let Some(task) = list.iter_mut().find(|d| d.id == id) {
            task.progress = progress.clamp(0.0, 1.0);
        }
    }
}
