use dioxus::prelude::*;
use crate::state::AppState;
use crate::theme::*;
use crate::Route;

#[component]
pub fn AppShell() -> Element {
    use_context_provider(|| AppState::new());

    rsx! {
        div {
            style: "display: flex; height: 100vh; width: 100vw; background: {BG}; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;",
            Sidebar {}
            div {
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn Sidebar() -> Element {
    let current = use_route::<Route>();

    rsx! {
        div {
            style: "width: 220px; min-width: 220px; background: {SURFACE}; border-right: 1px solid {BORDER}; display: flex; flex-direction: column; padding: 16px; gap: 4px;",

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
