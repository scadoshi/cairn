//! The shared `NavBar` shell with Odo's brand, links, and the theme picker.

use crate::inbound::ui::router::Route;
use dioxus::prelude::*;
use zwipe_components::{NavBar, ThemeConfig, ThemePicker};

const LOGO: &str = include_str!("../../../../../assets/odo.txt");

/// Top navigation. The theme picker is the same component zwipe and the
/// portfolio use, so the theme list stays identical across surfaces.
#[component]
pub fn Navbar() -> Element {
    let theme = use_context::<Signal<ThemeConfig>>();
    let mut open = use_signal(|| false);

    rsx! {
        NavBar {
            open,
            brand: rsx! {
                Link {
                    to: Route::Home {},
                    class: "nav-brand",
                    onclick: move |_| open.set(false),
                    pre { class: "nav-logo", "aria-label": "Odo", "{LOGO}" }
                }
            },
            links: rsx! {
                li {
                    Link {
                        to: Route::Home {},
                        class: "nav-link",
                        onclick: move |_| open.set(false),
                        "Counters"
                    }
                }
            },
            trailing: rsx! {
                ThemePicker { theme }
            },
        }
    }
}
