use dioxus::prelude::*;
use crate::api;
use crate::models::Rom;
use crate::state::AppState;
use crate::theme::*;

#[component]
pub fn GamePage(id: String) -> Element {
    let detail = use_resource(move || {
        let id = id.clone();
        async move {
            api::fetch_game_detail(&id).await.ok()
        }
    });

    let rom = use_context::<AppState>().current_rom.read().clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 24px; gap: 20px; overflow-y: auto;",

            match &*detail.read_unchecked() {
                Some(Some(rom_detail)) => rsx! {
                    GameDetail { rom: rom_detail.clone() }
                },
                Some(None) => match rom {
                    Some(r) => rsx! {
                        GameDetail { rom: r.clone() }
                    },
                    None => rsx! {
                        div { style: "color: {TEXT_DIM}; padding: 40px;", "Could not load game details." }
                    }
                },
                None => rsx! {
                    div { style: "color: {TEXT_DIM}; padding: 40px;", "Loading..." }
                }
            }
        }
    }
}

#[component]
fn GameDetail(rom: Rom) -> Element {
    let mut state = use_context::<AppState>();
    let nav = use_navigator();
    let already_queued = state.downloads.read().iter().any(|d| d.rom.id == rom.id);

    let rom_for_dl = rom.clone();
    let rom_page = rom.page_url.clone();
    let rom_id_images = rom.id.clone();

    let images = use_resource(move || {
        let id = rom_id_images.clone();
        async move {
            let (box_img, screen_img) = tokio::join!(
                api::fetch_image_data_url(&id, "box"),
                api::fetch_image_data_url(&id, "screen"),
            );
            (box_img.ok(), screen_img.ok())
        }
    });

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 20px;",

            div {
                style: "display: flex; align-items: center; gap: 12px;",
                button {
                    onclick: move |_| { let _ = nav.go_back(); },
                    style: "padding: 6px 14px; background: {CARD}; color: {TEXT}; border: 1px solid {BORDER}; border-radius: 6px; font-size: 13px; cursor: pointer; display: inline-flex; align-items: center; gap: 6px;",
                    "\u{2190} Back"
                }
            }

            h1 { style: "color: {TEXT}; margin: 0; font-size: 26px; font-weight: 700;", "{rom.name}" }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 24px; align-items: flex-start;",

                // Info table (left side)
                div {
                    style: "flex: 2 1 280px;",
                    div {
                        style: "background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; overflow: hidden;",

                        InfoRow { label: "Region", value: rom.region.clone() }
                        InfoRow { label: "Players", value: rom.players.clone() }
                        InfoRow { label: "Year", value: rom.year.clone() }
                        if !rom.publisher.is_empty() {
                            InfoRow { label: "Publisher", value: rom.publisher.clone() }
                        }
                        if !rom.serial.is_empty() {
                            InfoRow { label: "Serial #", value: rom.serial.clone() }
                        }
                        if !rom.graphics.is_empty() {
                            InfoRow { label: "Graphics", value: rom.graphics.clone() }
                        }
                        if !rom.sound.is_empty() {
                            InfoRow { label: "Sound", value: rom.sound.clone() }
                        }
                        if !rom.gameplay.is_empty() {
                            InfoRow { label: "Gameplay", value: rom.gameplay.clone() }
                        }
                        if !rom.overall.is_empty() {
                            InfoRow { label: "Overall", value: rom.overall.clone() }
                        }

                        div { style: "border-top: 1px solid {BORDER};" }

                        if !rom.file_name.is_empty() {
                            InfoRow { label: "File", value: rom.file_name.clone() }
                        }
                        if !rom.crc.is_empty() {
                            InfoRow { label: "CRC", value: rom.crc.clone() }
                        }
                        if !rom.md5.is_empty() {
                            InfoRow { label: "MD5", value: rom.md5.clone() }
                        }
                        if !rom.sha1.is_empty() {
                            InfoRow { label: "SHA1", value: rom.sha1.clone() }
                        }
                        if !rom.verified.is_empty() {
                            InfoRow { label: "Verified", value: rom.verified.clone() }
                        }
                        if !rom.version.is_empty() {
                            InfoRow { label: "Version", value: rom.version.clone() }
                        }
                        if !rom.size.is_empty() {
                            InfoRow { label: "Size", value: rom.size.clone() }
                        }
                    }
                }

                // Images (right side)
                div {
                    style: "display: flex; flex-direction: column; gap: 16px; flex: 1 1 220px;",
                    match &*images.read_unchecked() {
                        Some((screen_opt, box_opt)) => rsx! {
                            if let Some(data_url) = screen_opt {
                                div {
                                    style: "background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; overflow: hidden; padding: 8px;",
                                    img {
                                        src: data_url.clone(),
                                        style: "width: 100%; height: auto; object-fit: contain; display: block; border-radius: 4px;",
                                    }
                                    div { style: "text-align: center; color: {TEXT_DIM}; font-size: 11px; padding-top: 6px;", "Title screen" }
                                }
                            }
                            if let Some(data_url) = box_opt {
                                div {
                                    style: "background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; overflow: hidden; padding: 8px;",
                                    img {
                                        src: data_url.clone(),
                                        style: "width: 100%; height: auto; object-fit: contain; display: block; border-radius: 4px;",
                                    }
                                    div { style: "text-align: center; color: {TEXT_DIM}; font-size: 11px; padding-top: 6px;", "Box" }
                                }
                            }
                        },
                        _ => rsx! { "" }
                    }
                }
            }

            if !rom.description.is_empty() {
                p { style: "color: {TEXT_DIM}; font-size: 14px; line-height: 1.5; margin: 0; max-width: 720px;", "{rom.description}" }
            }

            div {
                style: "display: flex; gap: 12px; margin-top: 8px;",
                if already_queued {
                    button {
                        disabled: true,
                        style: "padding: 10px 24px; background: {BORDER}; color: {TEXT_DIM}; border: none; border-radius: 8px; font-size: 14px; cursor: default;",
                        "In Queue"
                    }
                } else {
                    button {
                        onclick: move |_| {
                            state.queue_download(rom_for_dl.clone());
                        },
                        style: "padding: 10px 24px; background: {ACCENT}; color: #fff; border: none; border-radius: 8px; font-size: 14px; cursor: pointer; font-weight: 600;",
                        "Download"
                    }
                }

                a {
                    href: rom_page.clone(),
                    target: "_blank",
                    style: "padding: 10px 24px; background: {CARD}; color: {TEXT}; border: 1px solid {BORDER}; border-radius: 8px; font-size: 14px; text-decoration: none; display: inline-flex; align-items: center;",
                    "Open on Vimm's Lair"
                }
            }
        }
    }
}

#[component]
fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; padding: 8px 12px; border-bottom: 1px solid {BORDER}; font-size: 13px;",
            div { style: "color: {TEXT_DIM}; min-width: 90px; padding-right: 12px;", "{label}" }
            div { style: "color: {TEXT}; flex: 1; word-break: break-word;", "{value}" }
        }
    }
}
