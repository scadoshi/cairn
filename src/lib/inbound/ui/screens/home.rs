//! The counter list, a +1 on each, and the create form.

use crate::{
    domain::counter::{Counter, CounterName, Goal, stats},
    inbound::ui::{
        components::stat_tile::{StatTile, rate},
        router::Route,
        today, use_store,
    },
};
use dioxus::prelude::*;
use zwipe_components::{Button, ButtonVariant};

const LOGO: &str = include_str!("../../../../../assets/odo.txt");

/// The counter list with a +1 on each card and the create form.
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

    let mut name_input = use_signal(String::new);
    let mut goal_input = use_signal(String::new);

    let create_store = store.clone();
    let create = move |_| {
        let name = match CounterName::new(&name_input()) {
            Ok(n) => n,
            Err(e) => return error.set(Some(e.to_string())),
        };
        let goal_text = goal_input();
        let goal = if goal_text.trim().is_empty() {
            None
        } else {
            match goal_text.trim().parse::<u32>().ok().map(Goal::per_year) {
                Some(Ok(g)) => Some(g),
                _ => {
                    return error.set(Some(
                        "goal must be a whole number of at least 1".to_string(),
                    ));
                }
            }
        };
        match create_store.create_counter(&name, goal, today()) {
            Ok(_) => {
                name_input.set(String::new());
                goal_input.set(String::new());
                error.set(None);
                reload.call(());
            }
            Err(e) => error.set(Some(e.to_string())),
        }
    };

    rsx! {
        div { class: "profile-sections content-enter",
            pre { class: "logo", "aria-label": "Odo", "{LOGO}" }
            if let Some(e) = error() {
                p { class: "form-error", "{e}" }
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
            div { class: "profile-list",
                div { class: "card-header",
                    span { class: "card-title", "New counter" }
                }
                div { class: "form-body",
                    input {
                        class: "input",
                        placeholder: "Name, e.g. pull-ups",
                        value: "{name_input}",
                        maxlength: "{CounterName::MAX_LEN}",
                        oninput: move |e| name_input.set(e.value()),
                    }
                    input {
                        class: "input",
                        r#type: "number",
                        min: "1",
                        inputmode: "numeric",
                        placeholder: "Yearly goal (optional)",
                        value: "{goal_input}",
                        oninput: move |e| goal_input.set(e.value()),
                    }
                    Button { variant: ButtonVariant::Util, onclick: create, "Create" }
                }
            }
        }
    }
}

/// One counter on the home list: name, lifetime, today, and a +1. Tapping the
/// card opens it.
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
                div { class: "card-header-actions",
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
            div { class: "stat-grid stat-grid-3",
                StatTile { label: "Lifetime", value: summary.lifetime.to_string() }
                StatTile { label: "Today", value: summary.today.to_string() }
                StatTile {
                    label: "This year",
                    value: summary.this_year.total.to_string(),
                    hint: format!("{}/day", rate(summary.this_year.per_day)),
                }
            }
        }
    }
}
