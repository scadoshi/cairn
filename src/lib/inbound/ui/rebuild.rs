//! The app rebuilt in a new window when iOS killed its web page, through
//! orangui's `use_page_kill_recovery`.
//!
//! A rebuilt app opens the store it was given and reads everything back from
//! it, as at a launch; only what a screen held in memory is gone.

use std::sync::Arc;

#[cfg(feature = "desktop")]
use dioxus::desktop::{Config, window};
#[cfg(all(feature = "mobile", not(feature = "desktop")))]
use dioxus::mobile::{Config, window};
use dioxus::prelude::*;
use orangui::{BOOT_BG, BOOT_HEAD};

use super::{App, SharedStore, use_store};

/// The webview's launch config: dark before any HTML paints, `#main` hidden
/// until the styles are in. The launch and a rebuilt window share it.
pub fn webview_config() -> Config {
    Config::new()
        .with_background_color(BOOT_BG)
        .with_custom_head(BOOT_HEAD.to_string())
}

/// The rebuild orangui calls after a kill: a new window with a fresh app on
/// the same store, then the dead window closed. orangui calls it on the main
/// thread outside the app's runtime, so it takes the window and the store
/// now. The event loop opens the new window before it closes this one, so
/// the app never runs out of windows.
pub fn use_rebuild() -> impl Fn() + 'static {
    let store: SharedStore = use_store();
    let dead = window();
    move || {
        dioxus::logger::tracing::warn!("the page was killed; rebuilding the app in a new window");
        let dom = VirtualDom::new(App).with_root_context(Arc::clone(&store));
        let _ = dead.new_window(dom, webview_config());
        dead.close();
    }
}
