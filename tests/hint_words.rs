//! No hint names an icon button by a word it no longer shows: Settings' Change buttons are a
//! pencil and the mode toggle is a sun or moon, so a hint draws the icon, never the word.

#![expect(
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    reason = "a test: slices the source after a match it found and counts brace depth"
)]

use std::fs;
use std::path::Path;

/// The words the icon-only buttons used to show.
const ICON_ONLY: [&str; 3] = ["change", "light", "dark"];

/// Every `.rs` file under `dir`, as its path and text.
fn sources(dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push((
                path.display().to_string(),
                fs::read_to_string(&path).unwrap(),
            ));
        }
    }
}

/// Where the block that `rest` opens inside closes: the byte of its matching `}`.
fn closing(rest: &str) -> usize {
    let mut depth = 0_usize;
    for (at, c) in rest.char_indices() {
        match c {
            '{' => depth += 1,
            '}' if depth == 0 => return at,
            '}' => depth -= 1,
            _ => {}
        }
    }
    rest.len()
}

#[test]
fn no_hint_key_names_a_change_light_or_dark_button_by_word() {
    let mut files = Vec::new();
    sources(Path::new("src"), &mut files);
    assert_ne!(files.len(), 0, "the sources were found");
    for (path, text) in files {
        for (at, _) in text.match_indices("HintKey {") {
            let rest = &text[at + "HintKey {".len()..];
            let body = rest.get(..closing(rest)).unwrap_or_default();
            let label: String = body
                .split('"')
                .skip(1)
                .step_by(2)
                .collect::<Vec<_>>()
                .concat();
            let words = label.to_lowercase();
            for word in ICON_ONLY {
                assert!(
                    !words.split(|c: char| !c.is_alphabetic()).any(|w| w == word),
                    "{path}: a hint key reads {label:?}; draw the button's icon instead"
                );
            }
        }
        let lower = text.to_lowercase();
        for phrase in ["change button", "tap change", "light button", "dark button"] {
            assert!(!lower.contains(phrase), "{path}: copy names {phrase:?}");
        }
    }
}
