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
use chrono::{Local, NaiveDateTime, TimeZone};
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
    // Whether the line is out in full. Only the launch's first card starts it
    // false; every later one shows its quote whole, unless the hour turns
    // while it is on screen.
    let mut typed = use_signal(|| TYPED.swap(true, Ordering::Relaxed));
    // Whether the tags animate in once the line is out.
    let mut arriving = use_signal(|| !*typed.peek());
    let mut tick = use_signal(now);
    use_future(move || async move {
        let mut current = quote_at(&now());
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let at = now();
            let next = quote_at(&at);
            if next != current {
                current = next;
                typed.set(false);
                arriving.set(true);
            }
            tick.set(at);
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

    // The tags hold their place while the line types, then follow it in.
    let tags = match (typed(), arriving()) {
        (false, _) => "quote-tags quote-tags-pending",
        (true, true) => "quote-tags quote-tags-in",
        (true, false) => "quote-tags",
    };

    rsx! {
        div { class: "profile-list quote-card",
            TypedText { key: "{text}", text, typed }
            div { class: tags,
                span { class: "stat-chip stat-chip-goal", "{author}" }
                span { class: "stat-chip stat-chip-derived", "{source}" }
                span { class: "stat-chip",
                    "Next quote "
                    span { class: "quote-tick-value", "{countdown}" }
                }
            }
        }
    }
}

/// The quote for a naive local time, `None` across a daylight saving gap.
fn quote_at(at: &NaiveDateTime) -> Option<&'static Quote> {
    quote::for_time(&Local.from_local_datetime(at).single()?)
}

/// `text`, typed out behind a block cursor that goes away once the typing stops.
///
/// The full text sits underneath, hidden, so the card is its final height
/// from the first frame and nothing below it moves while the line grows.
/// Types only when `typed` is false at mount, and sets it once the last key
/// lands; otherwise it shows the text whole.
#[component]
fn TypedText(text: &'static str, typed: Signal<bool>) -> Element {
    let fresh = use_hook(|| !*typed.peek());
    let mut typed = typed;
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
        typed.set(true);
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
