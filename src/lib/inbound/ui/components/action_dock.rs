//! The icons for orangui's action dock: a screen's or a sheet's actions in
//! one floating squircle bar at the bottom right.

use dioxus::prelude::*;

/// The add icon: a plus.
pub const PLUS: &str = "M12 5v14M5 12h14";
/// The save icon: a checkmark.
pub const CHECK: &str = "M5 12.5l4.5 4.5L19 7";
/// The Home icon: a house.
pub const HOUSE: &str = "M3 11.5l9-8 9 8M5 10v10h5v-6h4v6h5V10";
/// The Settings icon: a cog, an eight-toothed gear around a hole.
pub const COG: &str = "M19.4 10.3L21.9 10.7L21.9 13.3L19.4 13.7L18.4 16.1L19.9 18.1L18.1 19.9L16.1 18.4L13.7 19.4L13.3 21.9L10.7 21.9L10.3 19.4L7.9 18.4L5.9 19.9L4.1 18.1L5.6 16.1L4.6 13.7L2.1 13.3L2.1 10.7L4.6 10.3L5.6 7.9L4.1 5.9L5.9 4.1L7.9 5.6L10.3 4.6L10.7 2.1L13.3 2.1L13.7 4.6L16.1 5.6L18.1 4.1L19.9 5.9L18.4 7.9zM12 8.8a3.2 3.2 0 1 0 0 6.4 3.2 3.2 0 1 0 0-6.4";
/// The test icon: a play triangle.
pub const PLAY: &str = "M7 5v14l11-7z";
/// The About icon: an i in a circle.
pub const INFO: &str = "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18M12 11v5M12 8v0.01";
/// The Theme icon: a paintbrush.
pub const BRUSH: &str = "M18.4 2.6a2 2 0 0 1 2.8 2.8L12 14.6l-3-3 9.4-9ZM9 11.6 12.4 15c-.4 2.8-2.3 4.6-6.4 5 .9-1.2 1-2.6 1.2-3.9.2-1.8 1-2.8 1.8-4.5Z";

/// One stroke icon for a dock button, drawn from a single SVG path.
#[component]
pub fn DockIcon(path: &'static str) -> Element {
    rsx! {
        svg {
            class: "action-dock-icon",
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
            class: "action-dock-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: "M4 20h4L19 9a2.8 2.8 0 0 0-4-4L4 16v4z" }
            path { d: "M13.5 6.5l4 4" }
        }
    }
}
