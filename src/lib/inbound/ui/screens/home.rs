//! The counter list. Each card shows the headline numbers and carries its own
//! bar with +1 and Open; the screen bar leads to the create form and profile.

use crate::{
    domain::counter::{Counter, stats},
    inbound::ui::{
        components::stat_tile::{StatTile, rate},
        router::Route,
        today, use_store,
    },
};
use dioxus::prelude::*;
use zwipe_components::{ActionBar, Button, ButtonVariant};

/// The counter list.
#[component]
pub fn Home() -> Element {
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
                    nav.push(Route::NewCounter {});
                },
                "New counter"
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

/// One counter on the home list: name and goal up top, the headline numbers,
/// and a bar with +1 and Open.
#[component]
fn CounterCard(counter: Counter, on_bump: EventHandler<()>, on_open: EventHandler<i64>) -> Element {
    let store = use_store();
    let entries = store.entries(counter.id).unwrap_or_default();
    let summary = stats::summarize(&entries, counter.goal, today());
    let id = counter.id;
    let bump_store = store.clone();

    rsx! {
        div { class: "profile-list",
            div { class: "card-header",
                span { class: "card-title", "{counter.name}" }
                if let Some(g) = counter.goal {
                    span { class: "card-subtitle", "goal {g}" }
                }
            }
            div { class: "stat-grid stat-grid-3",
                StatTile { label: "Lifetime", value: summary.lifetime.to_string() }
                StatTile { label: "Today", value: summary.today.to_string() }
                StatTile {
                    label: "This year",
                    value: summary.this_year.total.to_string(),
                    hint: format!("{}/day", rate(summary.this_year.per_day)),
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        if bump_store.adjust(id, today(), 1).is_ok() {
                            on_bump.call(());
                        }
                    },
                    "+1"
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
