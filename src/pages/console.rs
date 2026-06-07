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
            style: "display: flex; flex-direction: column; height: 100%; padding: 24px; gap: 16px; overflow-y: auto;",

            h2 { style: "color: {TEXT}; margin: 0; font-size: 24px;", "{console_name}" }

            SectionBar { section, current_sec }

            match &*roms.read_unchecked() {
                Some(list) => rsx! {
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 10px;",
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
    rsx! {
        div {
            onclick: move |_| {
                state.current_rom.set(Some(rom.clone()));
                let _ = nav.push(crate::Route::Game { id: rom_id.clone() });
            },
            style: "padding: 14px; background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px; cursor: pointer; display: flex; gap: 12px;",
            if !rom.image_url.is_empty() {
                img {
                    src: rom.image_url.clone(),
                    style: "width: 60px; height: 60px; object-fit: contain; border-radius: 4px; flex-shrink: 0;",
                }
            }
            div {
                style: "display: flex; flex-direction: column; gap: 6px;",
                div { style: "color: {TEXT}; font-size: 14px; font-weight: 600; line-height: 1.3;", "{rom.name}" }
                div { style: "color: {TEXT_DIM}; font-size: 12px;", "{rom.region}" }
                div { style: "color: {TEXT_DIM}; font-size: 11px;", "v{rom.version}" }
            }
        }
    }
}
