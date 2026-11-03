//! The action dock, zwiper's: a screen's or a sheet's icon actions in one
//! floating squircle bar at the bottom right. Styled by the "action dock"
//! block of `main.css`.

use dioxus::prelude::*;

/// The add icon: a plus.
pub const PLUS: &str = "M12 5v14M5 12h14";
/// The save icon: a checkmark.
pub const CHECK: &str = "M5 12.5l4.5 4.5L19 7";
/// The test icon: a play triangle.
pub const PLAY: &str = "M7 5v14l11-7z";

/// One stroke icon for a dock button, drawn from a single SVG path.
#[component]
pub fn DockIcon(path: &'static str) -> Element {
    rsx! {
        svg {
            class: "icon-pill-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: path }
        }
    }
}

/// The edit icon: a pencil.
#[component]
pub fn EditIcon() -> Element {
    rsx! {
        svg {
            class: "icon-pill-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: "M4 20h4L19 9a2.8 2.8 0 0 0-4-4L4 16v4z" }
            path { d: "M13.5 6.5l4 4" }
        }
    }
}
