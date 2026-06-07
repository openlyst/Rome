use dioxus::prelude::*;
use crate::state::AppState;
use crate::theme::*;

#[component]
pub fn SettingsPage() -> Element {
    let mut state = use_context::<AppState>();
    let settings = state.settings.read().clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: 24px; gap: 20px; overflow-y: auto; max-width: 600px;",

            h2 { style: "color: {TEXT}; margin: 0; font-size: 24px;", "Settings" }

            div {
                style: "display: flex; flex-direction: column; gap: 8px;",
                span { style: "color: {TEXT}; font-size: 14px; font-weight: 600;", "Download Directory" }
                div {
                    style: "display: flex; gap: 8px; align-items: center;",
                    input {
                        r#type: "text",
                        value: "{settings.download_dir}",
                        oninput: move |e| {
                            let mut s = state.settings.read().clone();
                            s.download_dir = e.value();
                            state.settings.set(s);
                        },
                        style: "flex: 1; padding: 10px 12px; background: {CARD}; color: {TEXT}; border: 1px solid {BORDER}; border-radius: 6px; font-size: 14px; outline: none;",
                    }
                    button {
                        onclick: move |_| {
                            let mut state = state;
                            spawn(async move {
                                if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                    let path = folder.path().to_string_lossy().to_string();
                                    let mut s = state.settings.read().clone();
                                    s.download_dir = path;
                                    state.settings.set(s);
                                }
                            });
                        },
                        style: "padding: 10px 12px; background: {CARD}; color: {TEXT}; border: 1px solid {BORDER}; border-radius: 6px; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                        svg {
                            width: "18",
                            height: "18",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
                        }
                    }
                }
            }

            div {
                style: "display: flex; flex-direction: column; gap: 8px;",
                span { style: "color: {TEXT}; font-size: 14px; font-weight: 600;", "Auto Extract" }
                div {
                    style: "display: flex; align-items: center; gap: 10px;",
                    input {
                        r#type: "checkbox",
                        checked: settings.auto_extract,
                        onchange: move |e| {
                            let mut s = state.settings.read().clone();
                            s.auto_extract = e.checked();
                            state.settings.set(s);
                        },
                    }
                    span { style: "color: {TEXT}; font-size: 14px;", "Extract archives after download" }
                }
            }

            div {
                style: "display: flex; flex-direction: column; gap: 8px;",
                span { style: "color: {TEXT}; font-size: 14px; font-weight: 600;", "Version Filter" }
                select {
                    onchange: move |e| {
                        let mut s = state.settings.read().clone();
                        s.version_filter = e.value();
                        state.settings.set(s);
                    },
                    style: "padding: 10px 12px; background: {CARD}; color: {TEXT}; border: 1px solid {BORDER}; border-radius: 6px; font-size: 14px; outline: none; appearance: none; -webkit-appearance: none;",
                    option { value: "new", selected: settings.version_filter == "new", style: "background: {CARD}; color: {TEXT};", "Newest Only" }
                    option { value: "old", selected: settings.version_filter == "old", style: "background: {CARD}; color: {TEXT};", "Oldest Only" }
                    option { value: "all", selected: settings.version_filter == "all", style: "background: {CARD}; color: {TEXT};", "Show All" }
                }
            }

            div {
                style: "margin-top: 8px; padding: 16px; background: {CARD}; border: 1px solid {BORDER}; border-radius: 8px;",
                p { style: "color: {TEXT_DIM}; font-size: 13px; margin: 0; line-height: 1.5;",
                    "Changes are saved automatically in memory. The download directory is used for all future downloads."
                }
            }
        }
    }
}
