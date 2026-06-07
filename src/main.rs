use dioxus::prelude::*;

mod api;
mod app;
mod models;
mod pages;
mod state;
mod theme;

use app::AppShell;

#[derive(Routable, Clone, PartialEq)]
pub enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Home {},
    #[route("/console/:slug")]
    Console { slug: String },
    #[route("/game/:id")]
    Game { id: String },
    #[route("/downloads")]
    Downloads {},
    #[route("/settings")]
    Settings {},
}

#[component]
fn Home() -> Element {
    rsx! { pages::home::HomePage {} }
}

#[component]
fn Console(slug: String) -> Element {
    rsx! { pages::console::ConsolePage { slug } }
}

#[component]
fn Game(id: String) -> Element {
    rsx! { pages::game::GamePage { id } }
}

#[component]
fn Downloads() -> Element {
    rsx! { pages::downloads::DownloadsPage {} }
}

#[component]
fn Settings() -> Element {
    rsx! { pages::settings::SettingsPage {} }
}

fn main() {
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");

    dioxus::LaunchBuilder::desktop()
        .with_cfg(dioxus::desktop::Config::new()
            .with_window(
                dioxus::desktop::WindowBuilder::new()
                    .with_title("Vimm's Lair Downloader")
                    .with_inner_size(dioxus::desktop::LogicalSize::new(1200, 800))
                    .with_decorations(false)
            )
            .with_disable_context_menu(true)
            .with_background_color((13, 13, 18, 255))
            .with_custom_head(r#"<style>body,html{margin:0;padding:0;background:#0d0d12;}</style>"#.to_string())
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
