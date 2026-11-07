//! Where Cairn's files live on this platform.

use std::path::{Path, PathBuf};

use chrono::NaiveDate;

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

/// The prefix every full backup's file name starts with.
const BACKUP_PREFIX: &str = "cairn-backup-";

/// Where a full backup made on `day` goes: `cairn-backup-YYYY-MM-DD.json` in
/// `dir`. A second backup the same day overwrites the first.
pub fn backup_file(dir: &Path, day: NaiveDate) -> PathBuf {
    dir.join(format!("{BACKUP_PREFIX}{}.json", day.format("%Y-%m-%d")))
}

/// The newest full backup in `dir`, by the date in its name, or `None` when
/// there is none.
///
/// # Errors
///
/// The I/O error from reading `dir`.
pub fn latest_backup(dir: &Path) -> std::io::Result<Option<PathBuf>> {
    let mut newest: Option<(String, PathBuf)> = None;
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
            continue;
        };
        let json = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("json"));
        if !(json && name.starts_with(BACKUP_PREFIX)) {
            continue;
        }
        if newest.as_ref().is_none_or(|(best, _)| name > *best) {
            newest = Some((name, path));
        }
    }
    Ok(newest.map(|(_, path)| path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_latest_backup_is_the_newest_by_name() {
        let dir = std::env::temp_dir().join(format!("cairn-paths-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(latest_backup(&dir).unwrap(), None);
        let day = |m, d| NaiveDate::from_ymd_opt(2026, m, d).unwrap();
        for (m, d) in [(3, 9), (11, 2), (6, 30)] {
            std::fs::write(backup_file(&dir, day(m, d)), "{}").unwrap();
        }
        std::fs::write(dir.join("count-pages.csv"), "day,count\n").unwrap();
        std::fs::write(dir.join("cairn-backup-notes.txt"), "").unwrap();
        assert_eq!(
            latest_backup(&dir).unwrap(),
            Some(backup_file(&dir, day(11, 2)))
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
