//! The "!" in the header's left corner, zwiper's support button, opening a
//! note from the developer with a link to scottyfermo.com.

use super::{
    dialog_host::{DialogSpec, use_hosted_dialog},
    hint::HintLine,
    navigation::overlay_stack::use_overlay_back,
};
use dioxus::prelude::*;

const SITE_URL: &str = "https://scottyfermo.com";

/// The corner button and the dialog it opens.
#[component]
pub fn AboutButton() -> Element {
    let mut open = use_signal(|| false);
    use_overlay_back(open);
    use_hosted_dialog(open, move |close| DialogSpec {
        title: "Hey, I'm Scotty".to_string(),
        body: rsx! {
            div { class: "alert-dialog-description hint-body",
                HintLine { "I made Cairn by myself. It started as an app just for me." }
                HintLine {
                    "There's no account and no server, so your counts never leave your phone. If you want to see what else I'm working on, it's all on my site."
                }
            }
        },
        confirm: None,
        link: Some(("scottyfermo.com \u{2197}", SITE_URL)),
        close,
        owner: 0,
    });

    rsx! {
        button {
            class: "util-btn page-header-support",
            r#type: "button",
            aria_label: "About the developer",
            onclick: move |_| open.set(true),
            "!"
        }
    }
}
