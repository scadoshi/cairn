//! Routes: the home list, one counter, and config. The list is the landing
//! screen rather than a screen of its own.

use dioxus::prelude::*;

use super::Shell;
use super::components::navigation::back_handler::BackHandlerLayout;
use super::screens::config::Config;
use super::screens::counter::CounterScreen;
use super::screens::home::Home;

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
