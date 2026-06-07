use dioxus::prelude::*;
use crate::api;
use crate::models::Rom;
use crate::state::AppState;
use crate::theme::*;

#[component]
pub fn ConsolePage(slug: String) -> Element {
    let mut visible_count = use_signal(|| 36);
    let slug_for_api = slug.clone();

    let roms = use_resource(move || {
        let slug = slug_for_api.clone();
        async move {
            api::fetch_all_sections(&slug).await.unwrap_or_default()
        }
    });

    let console_name = api::consoles()
        .into_iter()
        .find(|c| c.slug == slug)
        .map(|c| c.name)
        .unwrap_or_else(|| slug.clone());

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; background: {BG};",

            div {
                style: "flex-shrink: 0; padding: 24px 24px 16px 24px; gap: 16px; display: flex; flex-direction: column; border-bottom: 1px solid {BORDER}; background: {BG};",
                h2 { style: "color: {TEXT}; margin: 0; font-size: 24px;", "{console_name}" }
            }

            div {
                style: "flex: 1; overflow-y: auto; padding: 16px 24px 24px 24px; min-height: 0;",
                match &*roms.read_unchecked() {
                    Some(list) => {
                        let total = list.len();
                        let count = (*visible_count.read()).min(total);
                        let visible = list[..count].to_vec();
                        rsx! {
                            div {
                                style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px; padding-bottom: 24px;",
                                for rom in visible {
                                    RomCard { rom: rom.clone() }
                                }
                                if count < total {
                                    div {
                                        onvisible: move |_| {
                                            let current = *visible_count.read();
                                            visible_count.set(current + 30);
                                        },
                                        style: "grid-column: 1 / -1; height: 1px;",
                                    }
                                }
                            }
                        }
                    }
                    None => rsx! {
                        div { style: "color: {TEXT_DIM}; padding: 40px;", "Loading..." }
                    }
                }
            }
        }
    }
}

#[component]
fn RomCard(rom: Rom) -> Element {
    let mut state = use_context::<AppState>();
    let nav = use_navigator();
    let rom_id = rom.id.clone();
    let rom_id_hover = rom.id.clone();
    let rom_for_click = rom.clone();

    let image_data = use_resource(move || {
        let id = rom.id.clone();
        async move {
            api::fetch_image_data_url(&id, "box").await.ok()
        }
    });

    rsx! {
        div {
            onclick: move |_| {
                state.current_rom.set(Some(rom_for_click.clone()));
                let _ = nav.push(crate::Route::Game { id: rom_id.clone() });
            },
            onmouseenter: move |_| {
                let id = rom_id_hover.clone();
                spawn(async move {
                    api::preload_game(&id).await;
                });
            },
            style: "cursor: pointer; display: flex; flex-direction: column; gap: 8px;",
            div {
                style: "position: relative; width: 100%; aspect-ratio: 2/3; border-radius: 4px; overflow: hidden; background: {SURFACE}; border: 1px solid {BORDER};",
                match &*image_data.read_unchecked() {
                    Some(Some(data_url)) => rsx! {
                        img {
                            src: data_url.clone(),
                            style: "width: 100%; height: 100%; object-fit: cover; display: block;",
                        }
                    },
                    _ => rsx! {
                        div { style: "width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; color: {TEXT_DIM}; font-size: 12px;", "Loading..." }
                    }
                }
                div {
                    style: "position: absolute; bottom: 0; left: 0; right: 0; padding: 24px 8px 8px 8px; background: linear-gradient(transparent, rgba(0,0,0,0.85));",
                    div { style: "color: #fff; font-size: 12px; font-weight: 600; line-height: 1.3; overflow: hidden; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;", "{rom.name}" }
                }
            }
        }
    }
}
