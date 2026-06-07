use crate::models::{AppSettings, DownloadTask, Rom};
use dioxus::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct AppState {
    pub search_query: Signal<String>,
    pub selected_console: Signal<Option<String>>,
    pub downloads: Signal<Vec<DownloadTask>>,
    pub settings: Signal<AppSettings>,
    pub current_rom: Signal<Option<Rom>>,
}

impl AppState {
    pub fn new() -> Self {
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VimmsROMs")
            .to_string_lossy()
            .to_string();

        Self {
            search_query: Signal::new(String::new()),
            selected_console: Signal::new(None),
            downloads: Signal::new(Vec::new()),
            settings: Signal::new(AppSettings {
                download_dir,
                auto_extract: true,
                regions: vec![8, 14, 26],
                version_filter: "new".into(),
            }),
            current_rom: Signal::new(None),
        }
    }

    pub fn queue_download(&mut self, rom: Rom) {
        let settings = self.settings.read();
        let path = PathBuf::from(&settings.download_dir)
            .join(format!("{}.zip", rom.id))
            .to_string_lossy()
            .to_string();

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
