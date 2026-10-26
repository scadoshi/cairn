//! Cairn's entrypoint: finds the data directory, opens the SQLite store and
//! launches the app on it.

use std::sync::Arc;

use cairn::inbound::ui::{App, SharedStore};
use cairn::outbound::paths;
use cairn::outbound::sqlite::SqliteStore;
use orangui::{BOOT_BG, BOOT_HEAD};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = paths::database().map_err(|e| format!("cannot locate data directory: {e}"))?;
    let store =
        SqliteStore::open(&path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    let store: SharedStore = Arc::new(store);
    launch(store);
    Ok(())
}

// The WebView paints dark and keeps #main hidden until the styles are in;
// App mounts orangui's BootReveal last among its head elements.
#[cfg(feature = "desktop")]
fn launch(store: SharedStore) {
    use dioxus::desktop::Config;
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_background_color(BOOT_BG)
                .with_custom_head(BOOT_HEAD.to_string()),
        )
        .with_context(store)
        .launch(App);
}

#[cfg(all(feature = "mobile", not(feature = "desktop")))]
fn launch(store: SharedStore) {
    use dioxus::mobile::Config;
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_background_color(BOOT_BG)
                .with_custom_head(BOOT_HEAD.to_string()),
        )
        .with_context(store)
        .launch(App);
}
