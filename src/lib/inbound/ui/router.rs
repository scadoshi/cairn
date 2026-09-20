//! Routes: the landing, the counter list, one counter, and config.

use super::{
    Shell,
    components::navigation::back_handler::BackHandlerLayout,
    screens::{config::Config, counter::CounterScreen, counters::Counters, home::Home},
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
        /// One counter, by store id.
        #[route("/counters/:id")]
        CounterScreen {
            /// `CounterId` as a plain integer, since routes are strings.
            id: i64,
        },
        /// Everything the person can set.
        #[route("/config")]
        Config {},
}
