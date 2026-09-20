//! The counter form body: name, goal with its unit, and step. Shared by the
//! create sheet on the list and the edit sheet on the counter screen so the
//! two never drift. The host owns the signals and does the saving.

use crate::domain::counter::{CounterName, Goal, Step, ValidationError, stats};
use chrono::Datelike;
use dioxus::prelude::*;
use zwipe_components::Chip;

use crate::inbound::ui::today;

/// Which period a typed goal amount is for.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalUnit {
    Day,
    Week,
    Year,
}

impl GoalUnit {
    /// The goal for `n` in this unit.
    pub fn goal(self, n: u32) -> Result<Goal, ValidationError> {
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

/// The form's state, owned by the host sheet.
#[derive(Clone, Copy, PartialEq)]
pub struct CounterFormState {
    /// Counter name as typed.
    pub name: Signal<String>,
    /// Goal amount as typed; blank means no goal.
    pub amount: Signal<String>,
    /// Which period the amount is for.
    pub unit: Signal<GoalUnit>,
    /// Step, one of `Step::ALLOWED`.
    pub step: Signal<u32>,
    /// Validation message to show, if any.
    pub error: Signal<Option<String>>,
}

impl Default for CounterFormState {
    /// Fresh state for a new counter.
    fn default() -> Self {
        Self {
            name: Signal::new(String::new()),
            amount: Signal::new(String::new()),
            unit: Signal::new(GoalUnit::Day),
            step: Signal::new(1),
            error: Signal::new(None),
        }
    }
}

impl CounterFormState {
    /// Loads an existing counter's values.
    pub fn load(&mut self, name: &str, goal: Option<Goal>, step: u32) {
        self.name.set(name.to_string());
        match goal {
            Some(g @ (Goal::PerDay(n) | Goal::PerWeek(n) | Goal::PerYear(n))) => {
                self.amount.set(n.to_string());
                self.unit.set(GoalUnit::of(g));
            }
            None => self.amount.set(String::new()),
        }
        self.step.set(step);
        self.error.set(None);
    }

    /// Validates the fields into domain values, or records the error.
    pub fn validate(&mut self) -> Option<(CounterName, Option<Goal>, Step)> {
        let name = match CounterName::new(&(self.name)()) {
            Ok(n) => n,
            Err(e) => {
                self.error.set(Some(e.to_string()));
                return None;
            }
        };
        let raw = (self.amount)();
        let goal = if raw.trim().is_empty() {
            None
        } else {
            let parsed = raw
                .trim()
                .parse::<u32>()
                .ok()
                .map(|n| (self.unit)().goal(n));
            let Some(Ok(g)) = parsed else {
                self.error.set(Some(
                    "goal must be a whole number of at least 1".to_string(),
                ));
                return None;
            };
            Some(g)
        };
        let step = match Step::new((self.step)()) {
            Ok(s) => s,
            Err(e) => {
                self.error.set(Some(e.to_string()));
                return None;
            }
        };
        self.error.set(None);
        Some((name, goal, step))
    }
}

/// The form body, the deck form's shape: labels over inputs, chips centred.
#[component]
pub fn CounterForm(state: CounterFormState) -> Element {
    let CounterFormState {
        mut name,
        mut amount,
        mut unit,
        mut step,
        error,
    } = state;
    let days = stats::days_in_year(today().year());
    let preview = amount()
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0)
        .and_then(|n| unit().goal(n).ok())
        .map(|g| g.breakdown(days));

    rsx! {
        form { class: "flex-col text-center", onsubmit: move |e| e.prevent_default(),
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
            // A text field, not type=number: iOS WebKit drops keystrokes when a
            // number input's value is rewritten mid-typing, which a bound value
            // does on every key. inputmode still brings up the number pad, and
            // the handler keeps only digits.
            input {
                class: "input",
                id: "counter_goal",
                r#type: "text",
                inputmode: "numeric",
                pattern: "[0-9]*",
                placeholder: "Not set",
                value: "{amount}",
                oninput: move |e| amount.set(e.value().chars().filter(char::is_ascii_digit).collect()),
            }
            div { class: "chip-row chip-row-center",
                Chip { selected: unit() == GoalUnit::Day, onclick: move |_| unit.set(GoalUnit::Day), "Per day" }
                Chip { selected: unit() == GoalUnit::Week, onclick: move |_| unit.set(GoalUnit::Week), "Per week" }
                Chip { selected: unit() == GoalUnit::Year, onclick: move |_| unit.set(GoalUnit::Year), "Per year" }
            }
            if let Some(parts) = preview {
                div { class: "chip-tags chip-tags-center",
                    for (text, entered) in parts {
                        span {
                            class: if entered { "stat-chip stat-chip-goal" } else { "stat-chip stat-chip-derived" },
                            "{text}"
                        }
                    }
                }
            }
            label { class: "label", "Tap increments" }
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
