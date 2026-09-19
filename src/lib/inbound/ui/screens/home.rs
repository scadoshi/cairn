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
use zwipe_components::{Button, ButtonVariant, Panel};

/// The counter list with a +1 on each card and the create form.
#[component]
pub fn Home() -> Element {
    let store = use_store();
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
        section { class: "content-enter",
            if let Some(e) = error() {
                p { class: "form-error", "{e}" }
            }
            div { class: "counter-grid",
                for c in counters() {
                    CounterCard { counter: c.clone(), on_change: move |()| reload.call(()) }
                }
                Panel {
                    eyebrow: "New",
                    title: "Add a counter",
                    actions: rsx! {
                        Button { variant: ButtonVariant::Small, onclick: create, "Create" }
                    },
                    label { class: "field",
                        span { class: "field-label", "Name" }
                        input {
                            class: "field-input",
                            placeholder: "pull-ups",
                            value: "{name_input}",
                            maxlength: "{CounterName::MAX_LEN}",
                            oninput: move |e| name_input.set(e.value()),
                        }
                    }
                    label { class: "field",
                        span { class: "field-label" , "Yearly goal (optional)" }
                        input {
                            class: "field-input",
                            r#type: "number",
                            min: "1",
                            placeholder: "5000",
                            value: "{goal_input}",
                            oninput: move |e| goal_input.set(e.value()),
                        }
                    }
                }
            }
        }
    }
}

/// One counter on the home grid: name, lifetime, today, and a +1.
#[component]
fn CounterCard(counter: Counter, on_change: EventHandler<()>) -> Element {
    let store = use_store();
    let entries = store.entries(counter.id).unwrap_or_default();
    let summary = stats::summarize(&entries, counter.goal, today());
    let id = counter.id;
    let bump_store = store.clone();

    rsx! {
        Panel {
            eyebrow: "Counter",
            title: counter.name.to_string(),
            actions: rsx! {
                Button {
                    variant: ButtonVariant::Small,
                    onclick: move |_| {
                        if bump_store.adjust(id, today(), 1).is_ok() {
                            on_change.call(());
                        }
                    },
                    "+1 today"
                }
                Link { to: Route::CounterScreen { id: id.0 }, class: "panel-action", "Details" }
            },
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
