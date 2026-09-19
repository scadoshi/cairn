//! Routes. Two screens: the counter list and one counter.

use super::{
    Shell,
    screens::{counter::CounterScreen, home::Home},
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
}
