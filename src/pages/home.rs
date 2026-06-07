use dioxus::prelude::*;
use crate::api;
use crate::models::{Console, Rom};
use crate::state::AppState;
use crate::theme::*;

#[component]
pub fn HomePage() -> Element {
    let mut query = use_signal(|| String::new());
    let results = use_resource(move || {
        let q = query.read().clone();
        async move {
            if q.len() < 2 { return Vec::new(); }
            api::search(&q, None).await.unwrap_or_default()
        }
    });

    let consoles = api::consoles();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 24px; gap: 24px; overflow-y: auto;",

            div {
                style: "display: flex; align-items: center; gap: 12px;",
                h1 { style: "color: {TEXT}; margin: 0; font-size: 28px; font-weight: 700;", "Vimm's Lair" }
                span { style: "color: {TEXT_DIM}; font-size: 14px;", "ROM downloader" }
            }

            div {
                style: "display: flex; gap: 8px;",
                input {
                    r#type: "text",
                    placeholder: "Search for a game...",
                    value: "{query}",
                    oninput: move |e| query.set(e.value()),
                    style: "flex: 1; padding: 12px 16px; background: {CARD}; color: {TEXT}; border: 1px solid {BORDER}; border-radius: 8px; font-size: 15px; outline: none;",
                }
            }

            match &*results.read_unchecked() {
                Some(list) if !list.is_empty() => rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        h3 { style: "color: {TEXT}; margin: 0;", "Search Results" }
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 12px;",
                            for rom in list.clone() {
                                SearchResultCard { rom: rom.clone() }
                            }
                        }
                    }
                },
                _ => rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 16px;",
                        h3 { style: "color: {TEXT}; margin: 0;", "Consoles" }
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 12px;",
                            for c in consoles {
                                ConsoleCard { console: c.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ConsoleCard(console: Console) -> Element {
    let nav = use_navigator();
    rsx! {
        div {
            onclick: move |_| { let _ = nav.push(crate::Route::Console { slug: console.slug.clone() }); },
            style: "padding: 20px; background: {CARD}; border: 1px solid {BORDER}; border-radius: 10px; cursor: pointer;",
            h4 { style: "color: {TEXT}; margin: 0; font-size: 15px; font-weight: 600;", "{console.name}" }
            p { style: "color: {TEXT_DIM}; margin: 6px 0 0 0; font-size: 12px;", "{console.slug}" }
        }
    }
}

#[component]
fn SearchResultCard(rom: Rom) -> Element {
    let mut state = use_context::<AppState>();
    let nav = use_navigator();
    let rom_id = rom.id.clone();
    rsx! {
        div {
            onclick: move |_| {
                state.current_rom.set(Some(rom.clone()));
                let _ = nav.push(crate::Route::Game { id: rom_id.clone() });
            },
            style: "padding: 14px; background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; cursor: pointer;",
            div { style: "color: {TEXT}; font-size: 14px; font-weight: 600;", "{rom.name}" }
            div { style: "color: {TEXT_DIM}; font-size: 12px; margin-top: 4px;", "{rom.region}   v{rom.version}" }
        }
    }
}
