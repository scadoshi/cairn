//! The "!" in the header's left corner, zwiper's support button, opening a
//! note from the developer with links to Cairn's changelog and
//! scottyfermo.com.

use dioxus::prelude::*;
use orangui::{AlertDialog, DialogAction, HintLine};

const SITE_URL: &str = "https://scottyfermo.com";

/// Cairn's changelog, which the portfolio serves; the app has no other.
const CHANGELOG_URL: &str = "https://scottyfermo.com/projects/cairn/changelog/";

/// The corner button and the dialog it opens.
#[component]
pub fn AboutButton() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        AlertDialog {
            open,
            title: "Hello!",
            actions: vec![
                DialogAction::Dismiss("Got it".into()),
                DialogAction::Link {
                    label: "Changelog \u{2197}".into(),
                    href: CHANGELOG_URL.into(),
                },
                DialogAction::Link {
                    label: "scottyfermo.com \u{2197}".into(),
                    href: SITE_URL.into(),
                },
            ],
            div { class: "hint-body",
                HintLine { "I made Cairn as a personal tool. What's new in each version, and my other projects, are at the links below." }
            }
        }
        button {
            class: "btn-action page-header-support",
            r#type: "button",
            aria_label: "About the developer",
            onclick: move |_| open.set(true),
            "!"
        }
    }
}
