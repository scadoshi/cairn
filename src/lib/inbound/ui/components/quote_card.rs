//! The quote under the counters, with the countdown to the next one.
//!
//! Which quote shows is a pure function of the hour (`domain::quote`), so
//! this holds no state beyond a clock that ticks once a second to move the
//! countdown along. When the countdown reaches zero the hour has turned and
//! the same tick picks up the new quote. At launch the words type themselves
//! out, with the stumbles of a person at a keyboard (`domain::typing`).

use crate::{
    domain::{
        quote::{self, Quote},
        typing::{self, Key},
    },
    inbound::ui::now,
};
use chrono::{Local, TimeZone};
use dioxus::prelude::*;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// Set once the launch's quote has started typing.
static TYPED: AtomicBool = AtomicBool::new(false);

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
            TypedText { key: "{text}", text }
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

/// `text`, typed out behind a block cursor that goes away once the typing stops.
///
/// The full text sits underneath, hidden, so the card is its final height
/// from the first frame and nothing below it moves while the line grows.
/// Only the first one mounted after launch types; every later one, on
/// coming back to Home or on the hour's new quote, shows the text whole.
#[component]
fn TypedText(text: &'static str) -> Element {
    let fresh = use_hook(|| !TYPED.swap(true, Ordering::Relaxed));
    let mut shown = use_signal(|| {
        if fresh {
            String::new()
        } else {
            text.to_string()
        }
    });
    let mut typing = use_signal(|| fresh);
    use_future(move || async move {
        if !fresh {
            return;
        }
        // Each view types it a little differently.
        let seed = Local::now().timestamp_nanos_opt().unwrap_or_default();
        for stroke in typing::script(text, seed.cast_unsigned()) {
            tokio::time::sleep(Duration::from_millis(u64::from(stroke.wait_ms))).await;
            match stroke.key {
                Key::Char(c) => shown.write().push(c),
                Key::Backspace => {
                    shown.write().pop();
                }
            }
        }
        typing.set(false);
    });

    rsx! {
        p { class: "quote-text typed", aria_label: text,
            span { class: "typed-full", aria_hidden: "true", "{text}" }
            span { class: "typed-live", aria_hidden: "true",
                "{shown}"
                if typing() {
                    span { class: "typed-cursor" }
                }
            }
        }
    }
}
