//! The Dioxus app: theme signal, shared CSS, router.

pub mod components;
pub mod router;
pub mod screens;

use crate::domain::{
    counter::{CounterId, Store},
    date_format::DateFormat,
    preferences::Preferences,
};
use chrono::{Local, NaiveDate, NaiveDateTime};
use dioxus::prelude::*;
use dioxus_primitives::toast::{Toast, ToastPropsWithOwner, ToastProvider};
use router::Route;
use std::{sync::Arc, time::Duration};
use zwipe_components::{Button, ButtonVariant, COMPONENTS_CSS, THEMES_CSS, ThemeConfig};

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

/// The preferences, provided by [`App`].
pub fn use_prefs() -> Signal<Preferences> {
    use_context::<Signal<Preferences>>()
}

/// The day the app is on: local time, shifted by the rollover hour, so a
/// tap at 00:30 after a late session still lands on the day before. Read
/// at the UI edge so the domain never touches the clock.
pub fn today() -> NaiveDate {
    let prefs = try_use_context::<Signal<Preferences>>()
        .map(|p| p())
        .unwrap_or_default();
    prefs.day_of(Local::now().naive_local())
}

/// The current local wall-clock time, for recording a tap.
pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}

/// Bumped after any write so screens that read the store outside their own
/// state (the shell's title, the home list) re-render. Provided by [`App`].
#[derive(Clone, Copy)]
pub struct StoreVersion(pub Signal<u32>);

/// How long a toast stays up, shared with zwipe so a confirmation does not
/// linger twice as long in one app as the other. Re-exported at this path so
/// call sites keep importing from `inbound::ui`.
///
/// These are paired with `assets/toast.css`, which fades each toast in and out
/// across its own lifetime. The stylesheet keys off `data-type`: info ->
/// quick, success and warning -> normal, error -> the library's own 5s
/// default, which `normal` matches.
///
/// That pairing is why the assertion below exists. The values live in another
/// repo now, so a bump there arriving via `cargo update -p zwipe-components`
/// would otherwise leave the keyframes mistimed and toasts fading out while
/// still up, with nothing to notice it. This fails the build instead.
///
/// Lengthened from 900ms and 1500ms on 2026-09-24, partly because toasts now
/// collapse into a stack you tap to expand and at 900ms one was gone before a
/// thumb could land on it.
pub use zwipe_components::{TOAST_NORMAL, TOAST_QUICK};

const _: () = assert!(
    TOAST_QUICK.as_secs() == 3 && TOAST_NORMAL.as_secs() == 5,
    "toast bands changed in zwipe-components: retune the toast-life-3000 and \
     toast-life-5000 keyframes in assets/toast.css to match, then update this \
     assertion"
);

/// Marks the store as changed.
///
/// `consume_context`, not `use_context`: every caller is an event handler
/// rather than a component body, and a hook there is what `dx check` flags.
pub fn bump_store_version() {
    let mut v = consume_context::<StoreVersion>().0;
    v += 1;
}

/// Root component. The binary opens the store and provides it through launch
/// context, so this crate never decides where the database lives.
#[component]
pub fn App() -> Element {
    // Toasts collapse into a stack and expand on tap, the way grouped
    // notifications do, so three of them cost about one toast of screen.
    let mut toasts_expanded = use_signal(|| false);
    // Set only for the length of a tap-driven toggle. The collapse offset is a
    // margin, and an arriving toast changes that same margin by taking over as
    // `:first-child`, so a permanent transition animated the stack expanding to
    // full height and dropping back every time one landed. CSS cannot tell the
    // two apart; this can, because only the tap sets it.
    let mut toasts_animating = use_signal(|| false);
    let store = use_store();
    let saved = store.theme().ok().flatten();
    let theme = use_signal(move || saved.unwrap_or_default());
    use_context_provider(|| theme);
    use_context_provider(|| StoreVersion(Signal::new(0)));
    let saved_prefs = store.preferences().unwrap_or_default();
    let prefs = use_signal(move || saved_prefs);
    use_context_provider(|| prefs);
    // Persist every change; when the rollover hour moves, rebuild the daily
    // entries from the events so history follows the new day boundary.
    let prefs_store = store.clone();
    let mut last_rollover = use_signal(move || saved_prefs.rollover_hour);
    use_effect(move || {
        let current = prefs();
        let _ = prefs_store.set_preferences(&current);
        if current.rollover_hour != last_rollover() {
            last_rollover.set(current.rollover_hour);
            let _ = prefs_store.rebuild_entries(&current);
        }
    });
    let saved_format = store.date_format().unwrap_or_default();
    let date_format = use_signal(move || saved_format);
    use_context_provider(|| date_format);
    let format_store = store.clone();
    use_effect(move || {
        let _ = format_store.set_date_format(date_format());
    });
    let overlays = components::navigation::overlay_stack::use_overlay_back_stack();
    use_context_provider(|| overlays);
    let dialogs = components::dialog_host::DialogHost(use_signal(|| None));
    use_context_provider(|| dialogs);
    let screen_hint = components::hint::ScreenHint(use_signal(|| None));
    use_context_provider(|| screen_hint);
    let celebrations = components::celebration::CelebrationHost(use_signal(|| None));
    use_context_provider(|| celebrations);

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
        // The theme class sits here as well as on the screen, so the toast
        // container, which mounts above the router, resolves the same
        // palette instead of falling through to unset variables.
        div { class: "theme-root {theme.read().css_class()}",
            ToastProvider {
                max_toasts: 3_usize,
                class: match (toasts_expanded(), toasts_animating()) {
                    (true, true) => "toast-container expanded animating",
                    (true, false) => "toast-container expanded",
                    (false, true) => "toast-container animating",
                    (false, false) => "toast-container",
                },
                // The library owns the container and list DOM and does not pass
                // event handlers through `attributes`, so the tap target has to
                // be the toast itself. `display: contents` keeps this wrapper
                // out of the layout.
                render_toast: move |props: ToastPropsWithOwner| rsx! {
                    div {
                        class: "toast-tap",
                        onclick: move |_| {
                            toasts_expanded.toggle();
                            toasts_animating.set(true);
                            spawn(async move {
                                // Outlasts the 0.2s transition in toast.css.
                                tokio::time::sleep(Duration::from_millis(250)).await;
                                toasts_animating.set(false);
                            });
                        },
                        Toast { ..props }
                    }
                },
                Router::<Route> {}
                // Dialogs draw here, beside the router, so no screen's scroll
                // container can trap their fixed overlay.
                components::dialog_host::DialogHostView {}
                // Same reason as the dialogs: a fixed overlay inside a
                // screen is clipped by the card or the scroll column.
                components::celebration::CelebrationHostView {}
            }
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
        Route::Home {} => "Cairn".to_string(),
        Route::Config {} => "Config".to_string(),
        Route::CounterScreen { id } => store
            .get_counter(CounterId(*id))
            .ok()
            .flatten()
            .map_or_else(|| "Counter".to_string(), |c| c.name.to_string()),
    };

    // Only screens that registered one get a "?", so the corner stays empty
    // rather than offering a dialog with nothing in it.
    let hint = use_context::<components::hint::ScreenHint>().0;

    rsx! {
        div { class: "screen theme-wrapper {css_class}",
            header { class: "page-header",
                h2 { "{title}" }
                if let Some(open) = hint() {
                    Button {
                        variant: ButtonVariant::Util,
                        class: "page-header-corner",
                        onclick: move |_| open.call(()),
                        "?"
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}
