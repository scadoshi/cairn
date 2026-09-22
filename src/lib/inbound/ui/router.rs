//! Routes: the home list, one counter, and config. The list used to be a
//! screen of its own; it is the landing screen now.

use super::{
    Shell,
    components::navigation::back_handler::BackHandlerLayout,
    screens::{config::Config, counter::CounterScreen, home::Home},
};
use dioxus::prelude::*;

/// Every screen in the app.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(BackHandlerLayout)]
    #[layout(Shell)]
        /// The landing screen: today, the counters, and a quote.
        #[route("/")]
        Home {},
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
