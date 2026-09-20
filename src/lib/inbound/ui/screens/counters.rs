//! The counter list. Each card shows the headline numbers and carries its own
//! bar with +1 and Open; the screen bar goes back home or to the create form.

use crate::{
    domain::counter::{Counter, format::thousands, stats, stats::days_in_year},
    inbound::ui::{
        components::stat_tile::{StatTile, rate},
        now,
        router::Route,
        today, use_store,
    },
};
use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// The counter list.
#[component]
pub fn Counters() -> Element {
    let store = use_store();
    let nav = use_navigator();
    let mut counters = use_signal(Vec::<Counter>::new);
    let mut error = use_signal(|| None::<String>);

    let load_store = store.clone();
    let reload = use_callback(move |()| match load_store.list_counters() {
        Ok(list) => counters.set(list),
        Err(e) => error.set(Some(e.to_string())),
    });
    use_effect(move || reload.call(()));

    rsx! {
        div { class: "screen-content",
            div { class: "profile-sections content-enter",
                if let Some(e) = error() {
                    p { class: "form-error", "{e}" }
                }
                if counters().is_empty() {
                    p { class: "pref-note", "No counters yet. New counter starts one." }
                }
                for c in counters() {
                    CounterCard {
                        counter: c.clone(),
                        on_bump: move |()| reload.call(()),
                        on_open: move |id: i64| {
                            nav.push(Route::CounterScreen { id });
                        },
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
                        nav.push(Route::Home {});
                    }
                },
                "Back"
            }
            Button {
                variant: ButtonVariant::Util,
                onclick: move |_| {
                    nav.push(Route::NewCounter {});
                },
                "Create"
            }
        }
    }
}

/// One counter on the home list: name and goal up top, the headline numbers,
/// and a bar with +1 and Open.
#[component]
fn CounterCard(counter: Counter, on_bump: EventHandler<()>, on_open: EventHandler<i64>) -> Element {
    let store = use_store();
    let entries = store.entries(counter.id).unwrap_or_default();
    let summary = stats::summarize(&entries, counter.goal, today());
    let id = counter.id;
    let bump_store = store.clone();
    let toast = use_toast();
    let name = counter.name.to_string();
    let step = i64::from(counter.step.get());
    let bump = use_callback(
        move |delta: i64| match bump_store.adjust(id, now(), delta) {
            Ok(total) => {
                toast.info(
                    format!("{name}: {} today", thousands(total)),
                    ToastOptions::default().duration(Duration::from_millis(900)),
                );
                on_bump.call(());
            }
            Err(e) => toast.error(e.to_string(), ToastOptions::default()),
        },
    );

    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "{counter.name}" }
                if let Some(g) = counter.goal {
                    span { class: "stat-chip stat-chip-goal", "{g.label(days_in_year(today().year()))}" }
                }
            }
            div { class: "stat-grid stat-grid-3",
                StatTile { label: "Lifetime", value: thousands(summary.lifetime) }
                StatTile { label: "Today", value: thousands(summary.today) }
                StatTile {
                    label: "This year",
                    value: thousands(summary.this_year.total),
                    hint: format!("{}/day", rate(summary.this_year.per_day)),
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| bump.call(-step),
                    "-{step}"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| bump.call(step),
                    "+{step}"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| on_open.call(id.0),
                    "Open"
                }
            }
        }
    }
}
