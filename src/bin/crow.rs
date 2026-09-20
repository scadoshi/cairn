use crow::{
    inbound::ui::{App, SharedStore},
    outbound::{paths, sqlite::SqliteStore},
};
use std::sync::Arc;

// Anti-flash boot styling for the native WebView, same trick as zwiper: the
// default theme's --bg-primary (gruvbox-dark #282828) is painted before any
// HTML so the window never flashes white.
const BOOT_BG: (u8, u8, u8, u8) = (0x28, 0x28, 0x28, 0xff);
const BOOT_HEAD: &str = "<style>html,body{background-color:#282828;}</style>";

fn main() {
    let path = match paths::database() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("crow: cannot locate data directory: {e}");
            return;
        }
    };
    let store = match SqliteStore::open(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("crow: cannot open {}: {e}", path.display());
            return;
        }
    };
    let store: SharedStore = Arc::new(store);
    launch(store);
}

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
