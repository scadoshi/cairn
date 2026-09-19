//! Routes: the counter list, one counter, and the profile.

use super::{
    Shell,
    screens::{counter::CounterScreen, home::Home, profile::Profile},
};
use dioxus::prelude::*;

/// Every screen in the app.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        /// The counter list.
        #[route("/")]
        Home {},
        /// One counter, by store id.
        #[route("/counters/:id")]
        CounterScreen {
            /// `CounterId` as a plain integer, since routes are strings.
            id: i64,
        },
        /// Preferences: theme and dark mode.
        #[route("/profile")]
        Profile {},
}
