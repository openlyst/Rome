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
    let already_queued = state.downloads.read().iter().any(|d| d.rom.id == rom.id);

    let rom_for_dl = rom.clone();
    let rom_page = rom.page_url.clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 16px; max-width: 720px;",

            h1 { style: "color: {TEXT}; margin: 0; font-size: 26px; font-weight: 700;", "{rom.name}" }

            if !rom.image_url.is_empty() {
                img {
                    src: rom.image_url.clone(),
                    style: "max-width: 200px; max-height: 280px; object-fit: contain; border-radius: 8px;",
                }
            }

            if !rom.description.is_empty() {
                p { style: "color: {TEXT_DIM}; font-size: 14px; line-height: 1.5; margin: 0;", "{rom.description}" }
            }

            div {
                style: "display: flex; flex-wrap: wrap; gap: 12px; margin-top: 4px;",
                if !rom.region.is_empty() {
                    InfoBadge { label: "Region".to_string(), value: rom.region.clone() }
                }
                if !rom.version.is_empty() {
                    InfoBadge { label: "Version".to_string(), value: rom.version.clone() }
                }
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
fn InfoBadge(label: String, value: String) -> Element {
    rsx! {
        div {
            style: "padding: 8px 14px; background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; display: flex; flex-direction: column; gap: 2px;",
            span { style: "color: {TEXT_DIM}; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px;", "{label}" }
            span { style: "color: {TEXT}; font-size: 13px; font-weight: 600;", "{value}" }
        }
    }
}
