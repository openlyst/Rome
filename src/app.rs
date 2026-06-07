use dioxus::prelude::*;
use crate::state::AppState;
use crate::theme::*;
use crate::Route;

#[component]
pub fn AppShell() -> Element {
    use_context_provider(|| AppState::new());
    let mut state = use_context::<AppState>();

    use_effect(move || {
        spawn(async move {
            loop {
                let maybe_task = {
                    let tasks = state.downloads.read();
                    tasks.iter()
                        .find(|t| matches!(t.status, crate::models::DownloadStatus::Queued))
                        .cloned()
                };

                if let Some(task) = maybe_task {
                    let id = task.id;
                    let url = task.rom.download_url.clone();
                    let path = task.save_path.clone();

                    tracing::info!("[queue] starting download id={} name={}", id, task.rom.name);
                    state.update_download_status(id, crate::models::DownloadStatus::Downloading);

                    let page_url = task.rom.page_url.clone();
                    let c = crate::api::client();
                    let result = crate::api::do_download_with_progress(&c, &url, &path, &page_url, move |p| {
                        state.update_download_progress(id, p);
                    }).await;

                    match result {
                        Ok(()) => {
                            tracing::info!("[queue] download completed id={}", id);
                            state.update_download_status(id, crate::models::DownloadStatus::Done);
                            state.mark_downloaded(task.rom.clone());

                            let rom_id = task.rom.id.clone();
                            let rom_name = task.rom.name.clone();
                            let sys = if task.rom.system.is_empty() {
                                "unknown".to_string()
                            } else {
                                task.rom.system.to_lowercase().replace(" ", "").replace("-", "")
                            };
                            let settings = state.settings.read();
                            let base_dir = std::path::PathBuf::from(&settings.download_dir).join(&sys);

                            // Extract zip
                            tracing::info!("[queue] extracting zip for id={}", id);
                            if let Err(e) = crate::api::extract_zip(&task.save_path, &base_dir.to_string_lossy()).await {
                                tracing::warn!("[queue] zip extraction failed for {}: {}", rom_name, e);
                            } else {
                                // Remove zip after extraction
                                let _ = tokio::fs::remove_file(&task.save_path).await;
                            }

                            // Download box art
                            let box_path = base_dir.join(format!("{}_icon.png", rom_id));
                            if let Err(e) = crate::api::download_image(&rom_id, "box", &box_path.to_string_lossy()).await {
                                tracing::warn!("[queue] box art download failed for {}: {}", rom_name, e);
                            }

                            // Save metadata JSON next to extracted files
                            let json_path = base_dir.join(format!("{}_info.json", rom_id));
                            if let Ok(json) = serde_json::to_string_pretty(&task.rom) {
                                if let Err(e) = tokio::fs::write(&json_path, json).await {
                                    tracing::warn!("[queue] metadata save failed for {}: {}", rom_name, e);
                                }
                            }

                            // Update main.json tracker
                            let main_path = std::path::PathBuf::from(&settings.download_dir).join("main.json");
                            let mut tracker = if main_path.exists() {
                                match tokio::fs::read_to_string(&main_path).await {
                                    Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| serde_json::Map::new()),
                                    Err(_) => serde_json::Map::new(),
                                }
                            } else {
                                serde_json::Map::new()
                            };
                            let entry = serde_json::json!({
                                "name": task.rom.name,
                                "system": task.rom.system,
                                "folder": sys,
                                "file_name": task.rom.file_name,
                            });
                            tracker.insert(rom_id.clone(), entry);
                            if let Ok(text) = serde_json::to_string_pretty(&tracker) {
                                if let Err(e) = tokio::fs::write(&main_path, text).await {
                                    tracing::warn!("[queue] main.json update failed for {}: {}", rom_name, e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("[queue] download failed id={} error={}", id, e);
                            state.update_download_status(id, crate::models::DownloadStatus::Failed(e));
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    });

    let mut sidebar_width = use_signal(|| 220_i32);
    let mut is_dragging = use_signal(|| false);

    rsx! {
        div {
            style: "display: flex; height: 100vh; width: 100vw; background: {BG}; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; user-select: none;",
            onmousemove: move |e| {
                if *is_dragging.read() {
                    let x = e.client_coordinates().x as i32;
                    let new_width = x.max(160).min(400);
                    sidebar_width.set(new_width);
                }
            },
            onmouseup: move |_| {
                is_dragging.set(false);
            },

            Sidebar { width: *sidebar_width.read() }

            div {
                style: "width: 4px; cursor: col-resize; background: transparent; position: relative; flex-shrink: 0;",
                onmousedown: move |_| {
                    is_dragging.set(true);
                },
                div {
                    style: "position: absolute; top: 0; bottom: 0; left: 1px; width: 2px; background: {BORDER}; transition: background 0.15s;",
                }
            }

            div {
                class: "main-content",
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                Outlet::<Route> {}
            }
            BottomNav {}
        }
    }
}

#[component]
fn Sidebar(width: i32) -> Element {
    let current = use_route::<Route>();

    rsx! {
        div {
            class: "sidebar",
            style: "width: {width}px; min-width: {width}px; background: {SURFACE}; border-right: 1px solid {BORDER}; display: flex; flex-direction: column; padding: 16px; gap: 4px; box-sizing: border-box; overflow: hidden;",

            div {
                style: "display: flex; align-items: center; gap: 8px; padding: 8px 4px 20px 4px;",
                span { style: "color: {ACCENT}; font-size: 20px; font-weight: 800;", "V" }
                span { style: "color: {TEXT}; font-size: 16px; font-weight: 700;", "imms" }
            }

            NavItem { label: "Home".to_string(), route: Route::Home {}, current: current.clone() }
            NavItem { label: "Downloads".to_string(), route: Route::Downloads {}, current: current.clone() }
            NavItem { label: "Settings".to_string(), route: Route::Settings {}, current: current.clone() }
        }
    }
}

#[component]
fn BottomNav() -> Element {
    let current = use_route::<Route>();

    rsx! {
        div {
            class: "bottom-nav",
            style: "display: none;",
            BottomNavItem { label: "Home".to_string(), icon: "\u{2302}".to_string(), route: Route::Home {}, current: current.clone() }
            BottomNavItem { label: "Downloads".to_string(), icon: "\u{2B73}".to_string(), route: Route::Downloads {}, current: current.clone() }
            BottomNavItem { label: "Settings".to_string(), icon: "\u{2699}".to_string(), route: Route::Settings {}, current: current.clone() }
        }
    }
}

#[component]
fn NavItem(label: String, route: Route, current: Route) -> Element {
    let nav = use_navigator();
    let active = std::mem::discriminant(&current) == std::mem::discriminant(&route);

    rsx! {
        div {
            onclick: move |_| { let _ = nav.push(route.clone()); },
            style: if active {
                format!("padding: 10px 12px; border-radius: 8px; color: {TEXT}; background: {CARD}; font-size: 14px; font-weight: 600; cursor: pointer;")
            } else {
                format!("padding: 10px 12px; border-radius: 8px; color: {TEXT_DIM}; font-size: 14px; cursor: pointer;")
            },
            "{label}"
        }
    }
}

#[component]
fn BottomNavItem(label: String, icon: String, route: Route, current: Route) -> Element {
    let nav = use_navigator();
    let active = std::mem::discriminant(&current) == std::mem::discriminant(&route);
    let color = if active { ACCENT } else { TEXT_DIM };
    let font_weight = if active { "600" } else { "400" };

    rsx! {
        div {
            onclick: move |_| { let _ = nav.push(route.clone()); },
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 2px; flex: 1; height: 100%; cursor: pointer;",
            div { style: "font-size: 20px; color: {color}; line-height: 1;", "{icon}" }
            div { style: "font-size: 10px; color: {color}; font-weight: {font_weight};", "{label}" }
        }
    }
}
