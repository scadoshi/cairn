//! The quote under the counters, with the countdown to the next one.
//!
//! Which quote shows is a pure function of the hour (`domain::quote`), so
//! this holds no state beyond a clock that ticks once a second to move the
//! countdown along. When the countdown reaches zero the hour has turned and
//! the same tick picks up the new quote.

use crate::{
    domain::quote::{self, Quote},
    inbound::ui::now,
};
use chrono::{Local, TimeZone};
use dioxus::prelude::*;
use std::time::Duration;

/// A quote, its attribution, and how long until it changes.
#[component]
pub fn QuoteCard() -> Element {
    let mut tick = use_signal(now);
    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            tick.set(now());
        }
    });

    // The rotation works in absolute time, so the naive local clock has to be
    // given its offset back before it means anything.
    let at = Local.from_local_datetime(&tick()).single();
    let Some(at) = at else {
        // Ambiguous or skipped local time, an hour either side of a daylight
        // saving change. Rather than guess an offset, sit the hour out.
        return rsx! {};
    };
    let Some(Quote {
        text,
        author,
        source,
    }) = quote::for_time(&at)
    else {
        return rsx! {};
    };
    let left = quote::until_next(&at);
    let countdown = format!("{}:{:02}", left.num_minutes(), left.num_seconds() % 60);

    rsx! {
        div { class: "profile-list quote-card",
            p { class: "quote-text", "{text}" }
            div { class: "quote-foot",
                span { class: "quote-author", "{author}" }
                span { class: "quote-source", "{source}" }
            }
            div { class: "quote-tick",
                span { class: "stat-chip",
                    "Next quote "
                    span { class: "quote-tick-value", "{countdown}" }
                }
            }
        }
    }
}
