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

/// Which period a typed goal amount is for. Lives here because both the
/// create form and the edit sheet pick it with the same chips.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalUnit {
    Day,
    Week,
    Year,
}

impl GoalUnit {
    /// The goal for `n` in this unit.
    pub fn goal(self, n: u32) -> Result<Goal, crate::domain::counter::ValidationError> {
        match self {
            Self::Day => Goal::per_day(n),
            Self::Week => Goal::per_week(n),
            Self::Year => Goal::per_year(n),
        }
    }

    /// The unit a stored goal was entered in.
    pub fn of(goal: Goal) -> Self {
        match goal {
            Goal::PerDay(_) => Self::Day,
            Goal::PerWeek(_) => Self::Week,
            Goal::PerYear(_) => Self::Year,
        }
    }
}

/// The create screen.
#[component]
pub fn NewCounter() -> Element {
    let store = use_store();
    let nav = use_navigator();
    let mut name = use_signal(String::new);
    let mut amount = use_signal(String::new);
    let mut unit = use_signal(|| GoalUnit::Day);
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
        .map(|n| unit().goal(n).map(|g| g.label(days)).unwrap_or_default());

    let save = move |_| {
        let counter_name = match CounterName::new(&name()) {
            Ok(n) => n,
            Err(e) => return error.set(Some(e.to_string())),
        };
        let raw = amount();
        let goal = if raw.trim().is_empty() {
            None
        } else {
            let parsed = raw.trim().parse::<u32>().ok().map(|n| unit().goal(n));
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
            div { class: "container-sm content-enter",
                form { class: "flex-col text-center", onsubmit: move |e| e.prevent_default(),
                    div { class: "form-section-title",
                        span { class: "card-title", "Counter" }
                    }
                    label { class: "label", r#for: "counter_name", "Name" }
                    input {
                        class: "input",
                        id: "counter_name",
                        placeholder: "Not set",
                        value: "{name}",
                        maxlength: "{CounterName::MAX_LEN}",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        oninput: move |e| name.set(e.value()),
                    }
                    label { class: "label", r#for: "counter_goal", "Goal" }
                    input {
                        class: "input",
                        id: "counter_goal",
                        r#type: "number",
                        min: "1",
                        inputmode: "numeric",
                        placeholder: "Not set",
                        value: "{amount}",
                        oninput: move |e| amount.set(e.value()),
                    }
                    div { class: "chip-row chip-row-center",
                        Chip { selected: unit() == GoalUnit::Day, onclick: move |_| unit.set(GoalUnit::Day), "Per day" }
                        Chip { selected: unit() == GoalUnit::Week, onclick: move |_| unit.set(GoalUnit::Week), "Per week" }
                        Chip { selected: unit() == GoalUnit::Year, onclick: move |_| unit.set(GoalUnit::Year), "Per year" }
                    }
                    if let Some(p) = preview {
                        p { class: "pref-note", "{p}" }
                    }
                    label { class: "label", "Each tap adds" }
                    div { class: "chip-row chip-row-center",
                        for n in Step::ALLOWED {
                            Chip { selected: step() == n, onclick: move |_| step.set(n), "{n}" }
                        }
                    }
                    if let Some(e) = error() {
                        p { class: "form-error", "{e}" }
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
