//! Where Cairn's files live on this platform.

use std::path::PathBuf;

/// The database path: `<data dir>/scadoshi-count/count.db`, with the directory created.
/// `~/Library/Application Support/scadoshi-count/count.db` on macOS. On Android the data
/// dir is the app's private files dir. Falls back to the working directory if the platform
/// has no data dir.
///
/// # Errors
///
/// The I/O error from creating the data directory, such as a permission denied, or on
/// Android from asking for the files dir.
pub fn database() -> std::io::Result<PathBuf> {
    let dir = data_dir()?.join("scadoshi-count");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("count.db"))
}

/// The platform's data dir, or the working directory where it has none.
#[cfg(not(target_os = "android"))]
#[expect(clippy::unnecessary_wraps, reason = "Android's arm can fail")]
fn data_dir() -> std::io::Result<PathBuf> {
    Ok(dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")))
}

/// The app's private files dir: `dirs` has no data dir on Android, and the
/// working directory is the read-only `/`.
#[cfg(target_os = "android")]
fn data_dir() -> std::io::Result<PathBuf> {
    super::android_fs::files_dir()
}

/// Where CSV exports land, created if missing.
///
/// On iOS the sandbox's HOME is the app container, and its Documents folder
/// is the one the Files app shows under "On My iPhone > Count" (Dioxus.toml
/// sets `UIFileSharingEnabled` and `LSSupportsOpeningDocumentsInPlace` for
/// that). The dirs crate has no iOS notion of a documents folder, and the
/// sandbox has no Downloads folder, so this builds the path from HOME.
/// Elsewhere it is the platform Downloads folder, falling back to the data
/// dir, as on Android, which has no Downloads folder in `dirs`.
///
/// # Errors
///
/// The I/O error from creating the exports folder, or from [`database`] when it is the fallback.
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
