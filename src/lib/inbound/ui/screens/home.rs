//! The landing screen: the wordmark fading in, a line on today, and the bar
//! to the counters and the profile. Same shape as zwiper's home.

use crate::{
    domain::counter::{format::thousands, stats},
    inbound::ui::{router::Route, today, use_store},
};
use chrono::Datelike;
use dioxus::prelude::*;
use zwipe_components::{ActionBar, Button, ButtonVariant};

const LOGO: &str = include_str!("../../../../../assets/odo.txt");

/// The landing screen.
#[component]
pub fn Home() -> Element {
    let store = use_store();
    let nav = use_navigator();

    // The day at a glance across every counter: reps logged today, how many
    // counters were touched, and the longest live streak.
    let (counters, today_total, active, streak) =
        store.list_counters().map_or((0, 0, 0, 0), |list| {
            let mut total = 0u32;
            let mut active = 0usize;
            let mut streak = 0u32;
            for c in &list {
                let entries = store.entries(c.id).unwrap_or_default();
                let s = stats::summarize(&entries, c.goal, today());
                total = total.saturating_add(s.today);
                if s.today > 0 {
                    active += 1;
                }
                streak = streak.max(s.streak);
            }
            (list.len(), total, active, streak)
        });

    let now = today();
    let date = now.format("%a %-d %b %Y").to_string();
    let day = now.ordinal();
    let week = now.iso_week().week();

    rsx! {
        div { class: "screen-content centered",
            pre { class: "logo", "aria-label": "Odo", "{LOGO}" }
            div { class: "container-sm home-hero content-enter-delayed",
                div { class: "card-header home-hero-head",
                    span { class: "card-title", "{date}" }
                    div { class: "chip-tags",
                        span { class: "stat-chip stat-chip-goal", "day {day}" }
                        span { class: "stat-chip", "week {week}" }
                    }
                }
                if counters == 0 {
                    p { class: "pref-note", style: "padding: 1rem;", "No counters yet. Counters, then New, starts one." }
                } else {
                    div { class: "profile-row",
                        span { class: "profile-row-label hero-label", "Logged today" }
                        div { class: "profile-row-value", "{thousands(today_total)}" }
                    }
                    div { class: "profile-row",
                        span { class: "profile-row-label hero-label", "Counters touched" }
                        div { class: "profile-row-value", "{active} of {counters}" }
                    }
                    div { class: "profile-row",
                        span { class: "profile-row-label hero-label", "Best streak" }
                        div { class: "profile-row-value", "{streak} days" }
                    }
                }
            }
        }
        ActionBar {
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| {
                    nav.push(Route::Counters {});
                },
                "Counters"
            }
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| {
                    nav.push(Route::Profile {});
                },
                "Profile"
            }
        }
    }
}
