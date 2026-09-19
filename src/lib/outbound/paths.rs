//! Where Odo's files live on this platform.

use std::path::PathBuf;

/// The database path: `<data dir>/odo/odo.db`, with the directory created.
/// `~/Library/Application Support/odo/odo.db` on macOS. Falls back to the
/// working directory if the platform has no data dir.
pub fn database() -> std::io::Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("odo");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("odo.db"))
}

/// Where CSV exports land: the platform Downloads folder, else the data dir.
pub fn exports() -> std::io::Result<PathBuf> {
    match dirs::download_dir() {
        Some(d) => Ok(d),
        None => Ok(database()?
            .parent()
            .map_or_else(|| PathBuf::from("."), PathBuf::from)),
    }
}
