//! Routes: the counter list, one counter, the create form, and the profile.

use super::{
    Shell,
    components::navigation::back_handler::BackHandlerLayout,
    screens::{
        counter::CounterScreen, counters::Counters, home::Home, new_counter::NewCounter,
        profile::Profile,
    },
};
use dioxus::prelude::*;

/// Every screen in the app.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(BackHandlerLayout)]
    #[layout(Shell)]
        /// The landing screen.
        #[route("/")]
        Home {},
        /// The counter list.
        #[route("/counters")]
        Counters {},
        /// Create a counter.
        #[route("/new")]
        NewCounter {},
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
