//! The create form: a name, an optional goal, and whether that goal is per
//! day or for the year. Save lives in the screen's bar, zwiper style.

use crate::{
    domain::counter::{CounterName, Goal, Step, stats},
    inbound::ui::{router::Route, today, use_store},
};
use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant, Chip};

/// The create screen.
#[component]
pub fn NewCounter() -> Element {
    let store = use_store();
    let nav = use_navigator();
    let mut name = use_signal(String::new);
    let mut amount = use_signal(String::new);
    let mut per_day = use_signal(|| true);
    let mut step = use_signal(|| 1u32);
    let mut error = use_signal(|| None::<String>);
    let toast = use_toast();

    // What the goal works out to, so "15 a day" shows its yearly figure and
    // "5000 a year" shows the daily rate it implies.
    let days = stats::days_in_year(today().year());
    let preview = amount()
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0)
        .map(|n| {
            if per_day() {
                format!("{n} a day is {} this year", n.saturating_mul(days))
            } else {
                format!(
                    "{n} this year is {:.1} a day",
                    f64::from(n) / f64::from(days)
                )
            }
        });

    let save = move |_| {
        let counter_name = match CounterName::new(&name()) {
            Ok(n) => n,
            Err(e) => return error.set(Some(e.to_string())),
        };
        let raw = amount();
        let goal = if raw.trim().is_empty() {
            None
        } else {
            let parsed = raw.trim().parse::<u32>().ok().map(|n| {
                if per_day() {
                    Goal::per_day(n)
                } else {
                    Goal::per_year(n)
                }
            });
            match parsed {
                Some(Ok(g)) => Some(g),
                _ => {
                    return error.set(Some(
                        "goal must be a whole number of at least 1".to_string(),
                    ));
                }
            }
        };
        let step = match Step::new(step()) {
            Ok(s) => s,
            Err(e) => return error.set(Some(e.to_string())),
        };
        match store.create_counter(&counter_name, goal, step, today()) {
            Ok(c) => {
                toast.success(
                    format!("Saved {}", c.name),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                nav.push(Route::Counters {});
            }
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    rsx! {
        div { class: "screen-content",
            div { class: "profile-sections content-enter",
                if let Some(e) = error() {
                    p { class: "form-error", "{e}" }
                }
                div { class: "profile-list",
                    div { class: "card-header",
                        span { class: "card-title", "Counter" }
                    }
                    div { class: "form-body",
                        input {
                            class: "input",
                            placeholder: "Name, e.g. pull-ups",
                            value: "{name}",
                            maxlength: "{CounterName::MAX_LEN}",
                            oninput: move |e| name.set(e.value()),
                        }
                        input {
                            class: "input",
                            r#type: "number",
                            min: "1",
                            inputmode: "numeric",
                            placeholder: "Goal (optional)",
                            value: "{amount}",
                            oninput: move |e| amount.set(e.value()),
                        }
                        div { class: "chip-row",
                            Chip { selected: per_day(), onclick: move |_| per_day.set(true), "Per day" }
                            Chip { selected: !per_day(), onclick: move |_| per_day.set(false), "Per year" }
                        }
                        if let Some(p) = preview {
                            p { class: "pref-note", "{p}" }
                        }
                        p { class: "field-label", "Each tap adds" }
                        div { class: "chip-row",
                            for n in Step::ALLOWED {
                                Chip { selected: step() == n, onclick: move |_| step.set(n), "{n}" }
                            }
                        }
                    }
                }
            }
        }
        ActionBar {
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| {
                    if nav.can_go_back() {
                        nav.go_back();
                    } else {
                        nav.push(Route::Counters {});
                    }
                },
                "Back"
            }
            Button { variant: ButtonVariant::Util, onclick: save, "Save" }
        }
    }
}
