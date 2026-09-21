//! Where Crow's files live on this platform.

use std::path::PathBuf;

/// The database path: `<data dir>/scadoshi-count/count.db`, with the directory created.
/// `~/Library/Application Support/scadoshi-count/count.db` on macOS. Falls back to the
/// working directory if the platform has no data dir.
pub fn database() -> std::io::Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("scadoshi-count");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("count.db"))
}

/// Where CSV exports land, created if missing.
///
/// On iOS the sandbox's HOME is the app container, and its Documents folder
/// is the one the Files app shows under "On My iPhone > Count" (Dioxus.toml
/// sets UIFileSharingEnabled and LSSupportsOpeningDocumentsInPlace for
/// that). The dirs crate has no iOS notion of a documents folder, and the
/// sandbox has no Downloads folder, so this builds the path from HOME.
/// Elsewhere it is the platform Downloads folder, falling back to the data
/// dir.
pub fn exports() -> std::io::Result<PathBuf> {
    let preferred = if cfg!(target_os = "ios") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Documents"))
    } else {
        dirs::download_dir()
    };
    let dir = match preferred {
        Some(d) => d,
        None => database()?
            .parent()
            .map_or_else(|| PathBuf::from("."), PathBuf::from),
    };
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
