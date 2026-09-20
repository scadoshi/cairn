//! The Dioxus app: theme signal, shared CSS, router.

pub mod components;
pub mod router;
pub mod screens;

use crate::domain::{
    counter::{CounterId, Store},
    date_format::DateFormat,
};
use chrono::{Local, NaiveDate, NaiveDateTime};
use dioxus::prelude::*;
use dioxus_primitives::toast::ToastProvider;
use router::Route;
use std::sync::Arc;
use zwipe_components::{COMPONENTS_CSS, THEMES_CSS, ThemeConfig};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TOAST_CSS: Asset = asset!("/assets/toast.css");
const FONT_JBM_400: Asset = asset!("/assets/fonts/jetbrains-mono-400.woff2");
const FONT_JBM_700: Asset = asset!("/assets/fonts/jetbrains-mono-700.woff2");

/// The store as the UI holds it: one shared handle behind both ports.
pub type SharedStore = Arc<dyn Store + Send + Sync>;

/// The store from context. Provided once by [`App`].
pub fn use_store() -> SharedStore {
    use_context::<SharedStore>()
}

/// The date format preference, provided by [`App`].
pub fn use_date_format() -> Signal<DateFormat> {
    use_context::<Signal<DateFormat>>()
}

/// Today in local time, read at the UI edge so the domain never touches the
/// clock.
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// The current local wall-clock time, for recording a tap.
pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

/// Bumped after any write so screens that read the store outside their own
/// state (the shell's title, the home list) re-render. Provided by [`App`].
#[derive(Clone, Copy)]
pub struct StoreVersion(pub Signal<u32>);

/// Marks the store as changed.
pub fn bump_store_version() {
    let mut v = use_context::<StoreVersion>().0;
    v += 1;
}

/// Root component. The binary opens the store and provides it through launch
/// context, so this crate never decides where the database lives.
#[component]
pub fn App() -> Element {
    let store = use_store();
    let saved = store.theme().ok().flatten();
    let theme = use_signal(move || saved.unwrap_or_default());
    use_context_provider(|| theme);
    use_context_provider(|| StoreVersion(Signal::new(0)));
    let saved_format = store.date_format().unwrap_or_default();
    let date_format = use_signal(move || saved_format);
    use_context_provider(|| date_format);
    let format_store = store.clone();
    use_effect(move || {
        let _ = format_store.set_date_format(date_format());
    });
    let overlays = components::navigation::overlay_stack::use_overlay_back_stack();
    use_context_provider(|| overlays);

    // Persist every theme change. Runs once at mount too, which is harmless:
    // it writes back whatever was just loaded.
    let persist_store = store.clone();
    use_effect(move || {
        let cfg = theme.read().clone();
        let _ = persist_store.set_theme(&cfg);
    });

    rsx! {
        // user-scalable=no kills the double-tap zoom. This is an app, not a
        // page: pinch and double-tap zoom just leave the layout stranded
        // off-center with no way back.
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover",
        }
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
        document::Stylesheet { href: TOAST_CSS }
        ToastProvider { max_toasts: 3_usize, class: "toast-container",
            Router::<Route> {}
        }
    }
}

/// The app shell, laid out like zwiper's: a header naming the screen, then whatever the screen renders
/// (its content and its own action bar) as siblings in the column.
#[component]
pub fn Shell() -> Element {
    let theme = use_context::<Signal<ThemeConfig>>();
    let css_class = theme.read().css_class();
    let route = use_route::<Route>();
    let store = use_store();
    // Read so a rename re-renders the title.
    let _ = use_context::<StoreVersion>().0.read();

    // The counter screen is named after its counter, so the title needs a
    // lookup. The rest are static.
    let title = match &route {
        Route::Home {} => "Home".to_string(),
        Route::Counters {} => "Counters".to_string(),
        Route::Profile {} => "Profile".to_string(),
        Route::CounterScreen { id } => store
            .get_counter(CounterId(*id))
            .ok()
            .flatten()
            .map_or_else(|| "Counter".to_string(), |c| c.name.to_string()),
    };

    rsx! {
        div { class: "screen theme-wrapper {css_class}",
            header { class: "page-header",
                h2 { "{title}" }
            }
            Outlet::<Route> {}
        }
    }
}
