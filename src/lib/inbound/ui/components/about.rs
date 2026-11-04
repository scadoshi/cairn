//! The two dialogs behind the menu's footer: the "!", zwiper's support
//! button, with a note from the developer and a link to scottyfermo.com, and
//! the "i" with the app's version and a link to its changelog. Links open in
//! the in-app browser.

use dioxus::prelude::*;
use orangui::{AlertDialog, DialogAction, HintLine};

use crate::inbound::ui::APP_VERSION;
use crate::inbound::ui::components::action_dock::{DockIcon, INFO};

const SITE_URL: &str = "https://scottyfermo.com";

/// Cairn's changelog, which the portfolio serves; the app has no other.
const CHANGELOG_URL: &str = "https://scottyfermo.com/projects/cairn/changelog/";

/// The "!" and the dialog it opens.
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
                    label: "scottyfermo.com \u{2197}".into(),
                    href: SITE_URL.into(),
                },
            ],
            div { class: "hint-body",
                HintLine { "I made Cairn as a personal tool. My other projects are at the link below." }
            }
        }
        button {
            class: "btn-action app-nav-support",
            r#type: "button",
            aria_label: "From the developer",
            onclick: move |_| open.set(true),
            "!"
        }
    }
}

/// The "i" and the About dialog it opens: the version and the changelog
/// link. Lit while the dialog is up, like a footer icon on its own screen.
#[component]
pub fn VersionButton() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        AlertDialog {
            open,
            title: "About",
            actions: vec![
                DialogAction::Dismiss("Got it".into()),
                DialogAction::Link {
                    label: "Changelog \u{2197}".into(),
                    href: CHANGELOG_URL.into(),
                },
            ],
            div { class: "hint-body",
                HintLine { "Version v{APP_VERSION}" }
            }
        }
        button {
            class: if open() { "btn-action app-nav-icon active" } else { "btn-action app-nav-icon" },
            r#type: "button",
            aria_label: "About",
            onclick: move |_| open.set(true),
            DockIcon { path: INFO }
        }
    }
}
