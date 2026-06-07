use dioxus::prelude::*;
use crate::models::DownloadStatus;
use crate::state::AppState;
use crate::theme::*;

#[component]
pub fn DownloadsPage() -> Element {
    let state = use_context::<AppState>();
    let tasks = state.downloads.read().clone();
    let downloaded = state.downloaded_roms.read().clone();
    let has_active = !tasks.is_empty();
    let has_downloaded = !downloaded.is_empty();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 24px; gap: 16px; overflow-y: auto;",

            h2 { style: "color: {TEXT}; margin: 0; font-size: 24px;", "Downloads" }

            if !has_active && !has_downloaded {
                div {
                    style: "padding: 60px; text-align: center; color: {TEXT_DIM}; font-size: 14px;",
                    "No downloads yet. Search for a game and hit Download."
                }
            }

            if has_active {
                div {
                    style: "display: flex; flex-direction: column; gap: 10px;",
                    for task in tasks {
                        DownloadRow { task: task.clone() }
                    }
                }
            }

            if has_downloaded {
                div {
                    style: "display: flex; flex-direction: column; gap: 10px;",
                    h3 { style: "color: {TEXT}; margin: 0; font-size: 16px;", "Downloaded" }
                    for rom in downloaded {
                        DownloadedRow { rom: rom.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn DownloadRow(task: crate::models::DownloadTask) -> Element {
    let mut state = use_context::<AppState>();
    let status_color = match &task.status {
        DownloadStatus::Queued => TEXT_DIM,
        DownloadStatus::Downloading => ACCENT,
        DownloadStatus::Extracting => WARNING,
        DownloadStatus::Done => SUCCESS,
        DownloadStatus::Failed(_) => ERROR,
    };

    let status_text = match &task.status {
        DownloadStatus::Queued => "Queued",
        DownloadStatus::Downloading => "Downloading",
        DownloadStatus::Extracting => "Extracting",
        DownloadStatus::Done => "Done",
        DownloadStatus::Failed(e) => &format!("Failed: {e}"),
    };

    let is_done = matches!(&task.status, DownloadStatus::Done);
    let task_id = task.id;
    let display_path = std::path::Path::new(&task.save_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| task.save_path.clone());

    rsx! {
        div {
            style: "padding: 14px; background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; display: flex; flex-direction: column; gap: 8px;",

            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                div {
                    style: "display: flex; flex-direction: column; gap: 2px;",
                    span { style: "color: {TEXT}; font-size: 14px; font-weight: 600;", "{task.rom.name}" }
                    span { style: "color: {TEXT_DIM}; font-size: 12px;", "{display_path}" }
                }
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    span { style: "color: {status_color}; font-size: 12px; font-weight: 600;", "{status_text}" }
                    if is_done {
                        button {
                            onclick: move |_| state.remove_download(task_id),
                            style: "padding: 4px 10px; background: transparent; color: {TEXT_DIM}; border: 1px solid {BORDER}; border-radius: 4px; font-size: 11px; cursor: pointer;",
                            "Remove"
                        }
                    }
                }
            }

            if !is_done && !matches!(&task.status, DownloadStatus::Failed(_)) {
                div {
                    style: "width: 100%; height: 4px; background: {BORDER}; border-radius: 2px; overflow: hidden;",
                    div {
                        style: "height: 100%; width: {task.progress * 100.0}%; background: {ACCENT}; border-radius: 2px;",
                    }
                }
            }
        }
    }
}

#[component]
fn DownloadedRow(rom: crate::models::Rom) -> Element {
    let mut state = use_context::<AppState>();
    let rom_id = rom.id.clone();
    let rom_name = rom.name.clone();

    rsx! {
        div {
            style: "padding: 14px; background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; display: flex; justify-content: space-between; align-items: center;",
            div {
                style: "display: flex; flex-direction: column; gap: 2px;",
                span { style: "color: {TEXT}; font-size: 14px; font-weight: 600;", "{rom_name}" }
                span { style: "color: {SUCCESS}; font-size: 12px; font-weight: 600;", "Downloaded" }
            }
            button {
                onclick: move |_| state.remove_downloaded(&rom_id),
                style: "padding: 4px 10px; background: transparent; color: {TEXT_DIM}; border: 1px solid {BORDER}; border-radius: 4px; font-size: 11px; cursor: pointer;",
                "Remove"
            }
        }
    }
}
