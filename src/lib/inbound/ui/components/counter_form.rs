//! The counter form body: name, goal with its unit, and step, plus the edit
//! sheet that wraps it. Shared by the create sheet on the list, the edit
//! button on each card, and the counter screen, so the three never drift.
//! The host owns the signals and does the saving.

use crate::domain::counter::{
    CounterId, CounterName, Goal, Step, ValidationError, check_big_step, stats,
};
use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{Button, ButtonVariant, Chip};

use crate::inbound::ui::{
    bump_store_version,
    components::{
        bottom_sheet::BottomSheet,
        hint::{HintBullet, HintBullets, HintChip, HintDialog, HintLine},
    },
    today, use_store,
};

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
    /// A second, larger step. Zero means none, which keeps the counter's
    /// bar at three buttons.
    pub big_step: Signal<u32>,
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
            big_step: Signal::new(0),
            error: Signal::new(None),
        }
    }
}

impl CounterFormState {
    /// Loads an existing counter's values.
    pub fn load(&mut self, name: &str, goal: Option<Goal>, step: u32, big_step: Option<u32>) {
        self.name.set(name.to_string());
        match goal {
            Some(g @ (Goal::PerDay(n) | Goal::PerWeek(n) | Goal::PerYear(n))) => {
                self.amount.set(n.to_string());
                self.unit.set(GoalUnit::of(g));
            }
            None => self.amount.set(String::new()),
        }
        self.step.set(step);
        self.big_step.set(big_step.unwrap_or(0));
        self.error.set(None);
    }

    /// Validates the fields into domain values, or records the error.
    pub fn validate(&mut self) -> Option<(CounterName, Option<Goal>, Step, Option<Step>)> {
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
        // Zero is the form's way of saying "none"; the domain decides
        // whether what is left is a sensible pair.
        let raw_big = (self.big_step)();
        let big = if raw_big == 0 {
            None
        } else {
            match Step::new(raw_big) {
                Ok(s) => Some(s),
                Err(e) => {
                    self.error.set(Some(e.to_string()));
                    return None;
                }
            }
        };
        let big_step = match check_big_step(step, big) {
            Ok(b) => b,
            Err(e) => {
                self.error.set(Some(e.to_string()));
                return None;
            }
        };
        self.error.set(None);
        Some((name, goal, step, big_step))
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
        mut big_step,
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
            label { class: "label", "Step" }
            div { class: "chip-row chip-row-center",
                for n in Step::ALLOWED {
                    Chip { selected: step() == n, onclick: move |_| step.set(n), "{n}" }
                }
            }
            label { class: "label", "Big step" }
            div { class: "chip-row chip-row-center",
                // Zero is "none", and it comes first so the default reads as
                // the absence of a second button rather than a size.
                Chip {
                    selected: big_step() == 0,
                    onclick: move |_| big_step.set(0),
                    "None"
                }
                for n in Step::ALLOWED.into_iter().filter(|n| *n > step()) {
                    Chip { selected: big_step() == n, onclick: move |_| big_step.set(n), "{n}" }
                }
            }
            if let Some(e) = error() {
                p { class: "form-error", "{e}" }
            }
        }
    }
}

/// Name, goal, and step in a sheet, the same form the create sheet uses.
///
/// Lives here rather than on either screen because both reach it: the card
/// on the list and the counter screen itself, so editing works wherever you
/// happen to be looking at a counter.
#[component]
pub fn EditSheet(
    open: Signal<bool>,
    id: CounterId,
    current_name: String,
    current_goal: Option<Goal>,
    current_step: u32,
    current_big_step: Option<u32>,
    on_saved: EventHandler<()>,
) -> Element {
    let mut open = open;
    let store = use_store();
    let toast = use_toast();
    let mut form = use_hook(CounterFormState::default);
    let hint_open = use_signal(|| false);

    let seed_name = current_name.clone();
    use_effect(move || {
        if open() {
            form.load(&seed_name, current_goal, current_step, current_big_step);
        }
    });

    let save = move |_| {
        let Some((name, goal, step, big_step)) = form.validate() else {
            return;
        };
        match store.update_counter(id, &name, goal, step, big_step) {
            Ok(()) => {
                toast.success(
                    format!("Saved {name}"),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
                bump_store_version();
                on_saved.call(());
                open.set(false);
            }
            Err(e) => form.error.set(Some(e.to_string())),
        }
    };

    rsx! {
        HintDialog { open: hint_open, title: "Edit counter",
            HintLine { "Changing these does not touch anything already logged." }
            HintBullets {
                HintBullet { "Goal is optional. Clear it and the counter just totals up." }
                HintBullet { "A yearly goal still shows a daily share, so " HintChip { class: "stat-chip-goal", "1,000/year" } " asks for 3 a day." }
                HintBullet { "Step is how much one tap adds, on this screen and on the list." }
                HintBullet { "Big step adds a larger pair outside the first, for the days you do more at once. It has to be bigger than the step." }
            }
        }
        BottomSheet {
            open,
            title: "Edit counter",
            hint: hint_open,
            footer: rsx! {
                Button { variant: ButtonVariant::Util, onclick: move |_| open.set(false), "Back" }
                Button { variant: ButtonVariant::Util, onclick: save, "Save" }
            },
            CounterForm { state: form }
        }
    }
}
