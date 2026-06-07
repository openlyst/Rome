use dioxus::prelude::*;
use crate::api;
use crate::models::Rom;
use crate::state::AppState;
use crate::theme::*;

#[component]
pub fn ConsolePage(slug: String) -> Element {
    let section = use_signal(|| 'A'.to_string());
    let slug_for_api = slug.clone();

    let roms = use_resource(move || {
        let slug = slug_for_api.clone();
        async move {
            let sec = section.read().clone();
            let sec_param = if sec == "#" { "number" } else { &sec };
            api::fetch_section(&slug, sec_param).await.unwrap_or_default()
        }
    });

    let console_name = api::consoles()
        .into_iter()
        .find(|c| c.slug == slug)
        .map(|c| c.name)
        .unwrap_or_else(|| slug.clone());

    let current_sec = section.read().clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; background: {BG};",

            div {
                style: "flex-shrink: 0; padding: 24px 24px 16px 24px; gap: 16px; display: flex; flex-direction: column; border-bottom: 1px solid {BORDER}; background: {BG};",
                h2 { style: "color: {TEXT}; margin: 0; font-size: 24px;", "{console_name}" }
                SectionBar { section, current_sec }
            }

            div {
                style: "flex: 1; overflow-y: auto; padding: 16px 24px 24px 24px; min-height: 0;",
                match &*roms.read_unchecked() {
                    Some(list) => rsx! {
                        div {
                            style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px; padding-bottom: 24px;",
                            for rom in list.clone() {
                                RomCard { rom: rom.clone() }
                            }
                        }
                    },
                    None => rsx! {
                        div { style: "color: {TEXT_DIM}; padding: 40px;", "Loading..." }
                    }
                }
            }
        }
    }
}

#[component]
fn SectionBar(section: Signal<String>, current_sec: String) -> Element {
    let all: Vec<String> = std::iter::once("#".to_string())
        .chain(('A'..='Z').map(|c| c.to_string()))
        .collect();

    rsx! {
        div {
            style: "display: flex; flex-wrap: wrap; gap: 6px;",
            for sec in all {
                SectionButton { section, sec: sec.clone(), current_sec: current_sec.clone() }
            }
        }
    }
}

#[component]
fn SectionButton(section: Signal<String>, sec: String, current_sec: String) -> Element {
    let is_active = sec == current_sec;
    let style = if is_active {
        format!("padding: 6px 12px; background: {ACCENT}; color: #fff; border: none; border-radius: 6px; font-size: 13px; cursor: pointer; font-weight: 600;")
    } else {
        format!("padding: 6px 12px; background: {CARD}; color: {TEXT_DIM}; border: 1px solid {BORDER}; border-radius: 6px; font-size: 13px; cursor: pointer;")
    };
    let sec_clone = sec.clone();
    rsx! {
        button {
            onclick: move |_| section.set(sec_clone.clone()),
            style: "{style}",
            "{sec}"
        }
    }
}

#[component]
fn RomCard(rom: Rom) -> Element {
    let mut state = use_context::<AppState>();
    let nav = use_navigator();
    let rom_id = rom.id.clone();
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
