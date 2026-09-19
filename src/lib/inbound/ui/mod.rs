//! The Dioxus app: theme signal, shared CSS, router.

pub mod components;
pub mod router;
pub mod screens;

use crate::domain::counter::Store;
use chrono::{Local, NaiveDate};
use dioxus::prelude::*;
use router::Route;
use std::sync::Arc;
use zwipe_components::{COMPONENTS_CSS, THEMES_CSS, ThemeConfig};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const FONT_JBM_400: Asset = asset!("/assets/fonts/jetbrains-mono-400.woff2");
const FONT_JBM_700: Asset = asset!("/assets/fonts/jetbrains-mono-700.woff2");

/// The store as the UI holds it: one shared handle behind both ports.
pub type SharedStore = Arc<dyn Store + Send + Sync>;

/// The store from context. Provided once by [`App`].
pub fn use_store() -> SharedStore {
    use_context::<SharedStore>()
}

/// Today in local time, read at the UI edge so the domain never touches the
/// clock.
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Root component. The binary opens the store and provides it through launch
/// context, so this crate never decides where the database lives.
#[component]
pub fn App() -> Element {
    let store = use_store();
    let saved = store.theme().ok().flatten();
    let theme = use_signal(move || saved.unwrap_or_default());
    use_context_provider(|| theme);

    // Persist every theme change. Runs once at mount too, which is harmless:
    // it writes back whatever was just loaded.
    let persist_store = store.clone();
    use_effect(move || {
        let cfg = theme.read().clone();
        let _ = persist_store.set_theme(&cfg);
    });

    rsx! {
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" }
        document::Meta { name: "darkreader-lock" }
        // Fonts ship in the bundle via asset!(), so the @font-face has to be
        // built here where the hashed URLs are known (same trick as zwiper).
        document::Style {
            "@font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:400;font-display:swap;src:url({FONT_JBM_400}) format('woff2');}}\
             @font-face{{font-family:'JetBrains Mono';font-style:normal;font-weight:700;font-display:swap;src:url({FONT_JBM_700}) format('woff2');}}"
        }
        // Order matters: themes -> components -> app, so app rules can
        // override component rules at equal specificity.
        document::Style { {THEMES_CSS} }
        document::Style { {COMPONENTS_CSS} }
        document::Stylesheet { href: MAIN_CSS }
        Router::<Route> {}
    }
}

/// Wraps every route in the theme class, the nav bar, and the content column.
#[component]
pub fn Shell() -> Element {
    let theme = use_context::<Signal<ThemeConfig>>();
    let css_class = theme.read().css_class();
    rsx! {
        div { class: "theme-wrapper {css_class}",
            components::navbar::Navbar {}
            main { class: "content",
                Outlet::<Route> {}
            }
        }
    }
}
