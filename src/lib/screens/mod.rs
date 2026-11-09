//! What each screen does, without drawing it.
//!
//! Pure like [`crate::domain`]: no Dioxus, no SQLite and no clock. The day
//! and the time come in as arguments, and anything that reads or writes the
//! store is the caller's to perform. `inbound::ui` draws these and performs
//! what they ask; the shape follows zwipe's screen models
//! (`context/plans/screen_models.md`).

pub mod home;
pub mod saying;
pub mod tap;
pub mod toast;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    /// Every `.rs` file under `dir`, recursively.
    fn sources(dir: &Path, found: &mut Vec<(String, String)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = fs::read_to_string(&path).unwrap();
                found.push((path.display().to_string(), text));
            }
        }
    }

    #[test]
    fn the_screens_import_neither_dioxus_nor_rusqlite() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib/screens");
        let mut found = Vec::new();
        sources(&dir, &mut found);
        assert!(
            found.len() >= 4,
            "found the module's files: {}",
            found.len()
        );
        // Split so this test's own source does not match them.
        let banned = [
            concat!("dioxus", "::"),
            concat!("dioxus_", "primitives"),
            concat!("rusqlite", "::"),
            concat!("crate::", "inbound"),
            concat!("crate::", "outbound"),
        ];
        for (path, text) in found {
            for line in text.lines().map(str::trim_start) {
                if line.starts_with("//") {
                    continue;
                }
                for banned in banned {
                    assert!(!line.contains(banned), "{path} names {banned}: {line}");
                }
            }
        }
    }
}
