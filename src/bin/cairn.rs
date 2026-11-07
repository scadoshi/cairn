//! Cairn's entrypoint: finds the data directory, opens the SQLite store and
//! launches the app on it.

use std::sync::Arc;

use cairn::inbound::ui::{App, SharedStore, rebuild};
use cairn::outbound::paths;
use cairn::outbound::sqlite::SqliteStore;

/// The wordmark, as `assets/cairn.txt` draws it.
const LOGO: &str = include_str!("../../assets/cairn.txt");

/// Prints the wordmark to the terminal that launched the app.
#[expect(clippy::print_stdout, reason = "prints the logo to a CLI's own output")]
fn print_logo() {
    println!("{LOGO}");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_logo();
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
    dioxus::LaunchBuilder::new()
        .with_cfg(rebuild::webview_config())
        .with_context(store)
        .launch(App);
}

#[cfg(all(feature = "mobile", not(feature = "desktop")))]
fn launch(store: SharedStore) {
    dioxus::LaunchBuilder::new()
        .with_cfg(rebuild::webview_config())
        .with_context(store)
        .launch(App);
}
