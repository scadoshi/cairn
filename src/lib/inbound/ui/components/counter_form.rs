//! The counter form body: name, goal with its unit, and step, plus the edit
//! sheet that wraps it. Shared by the create sheet on the list, the edit
//! button on each card, and the counter screen, so the three never drift.
//! The host owns the signals and does the saving.

use chrono::Datelike;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use orangui::{BottomSheet, Chip, FieldLabel, HintDialog, HintKey, HintLine, TextInput};

use crate::domain::counter::{self, CounterId, CounterName, Goal, Step, ValidationError, stats};
use crate::domain::preferences::Celebration;
use crate::inbound::ui::components::action_dock::{self, DockIcon};
use crate::inbound::ui::{self, TOAST_NORMAL, use_store};

/// Which period a typed goal amount is for.
#[expect(missing_docs, reason = "Day, Week and Year say what they are")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalUnit {
    Day,
    Week,
    Year,
}

impl GoalUnit {
    /// The goal for `n` in this unit.
    ///
    /// # Errors
    ///
    /// [`ValidationError::ZeroGoal`] when `n` is zero.
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

/// What a filled-in form becomes once every field checks out.
#[derive(Debug)]
pub struct ValidCounter {
    /// Display name.
    pub name: CounterName,
    /// Optional target.
    pub goal: Option<Goal>,
    /// How much one tap adds.
    pub step: Step,
    /// The optional second, larger step.
    pub big_step: Option<Step>,
    /// This counter's celebration, or `None` to follow Config.
    pub celebration: Option<Celebration>,
}

/// What is wrong with the form, by field.
///
/// Each field's message sits under that field; `form` holds anything that
/// belongs to no single field, such as a failed save, and sits at the
/// bottom. Messages are the domain's lowercase lines; the form sentence-cases
/// them where they show.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormErrors {
    /// Under the name field.
    pub name: Option<String>,
    /// Under the goal field.
    pub goal: Option<String>,
    /// Under the step chips.
    pub step: Option<String>,
    /// Under the big step chips.
    pub big_step: Option<String>,
    /// At the bottom of the form.
    pub form: Option<String>,
}

impl FormErrors {
    /// Whether no field and not the form carries a message.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
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
    /// This counter's celebration. `None` follows the app-wide setting.
    pub celebration: Signal<Option<Celebration>>,
    /// What the last validation or save found wrong.
    pub errors: Signal<FormErrors>,
}

impl std::fmt::Debug for CounterFormState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CounterFormState").finish_non_exhaustive()
    }
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
            celebration: Signal::new(None),
            errors: Signal::new(FormErrors::default()),
        }
    }
}

impl CounterFormState {
    /// Loads an existing counter's values.
    pub fn load(
        &mut self,
        name: &str,
        goal: Option<Goal>,
        step: u32,
        big_step: Option<u32>,
        celebration: Option<Celebration>,
    ) {
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
        self.celebration.set(celebration);
        self.errors.set(FormErrors::default());
    }

    /// Records `message` at the bottom of the form, under every field.
    pub fn fail(&mut self, message: String) {
        self.errors.set(FormErrors {
            form: Some(message),
            ..FormErrors::default()
        });
    }

    /// Validates the fields into domain values, or marks every field that
    /// is wrong.
    pub fn validate(&mut self) -> Option<ValidCounter> {
        let mut errors = FormErrors::default();
        let name = CounterName::new(&(self.name)())
            .map_err(|e| errors.name = Some(e.to_string()))
            .ok();
        let raw = (self.amount)();
        let goal = if raw.trim().is_empty() {
            Some(None)
        } else {
            let parsed = raw
                .trim()
                .parse::<u32>()
                .ok()
                .map(|n| (self.unit)().goal(n));
            if let Some(Ok(g)) = parsed {
                Some(Some(g))
            } else {
                errors.goal = Some("goal must be a whole number of at least 1".to_string());
                None
            }
        };
        let step = Step::new((self.step)())
            .map_err(|e| errors.step = Some(e.to_string()))
            .ok();
        // Zero is the form's way of saying "none"; the domain decides
        // whether what is left is a sensible pair.
        let raw_big = (self.big_step)();
        let big = if raw_big == 0 {
            Some(None)
        } else {
            Step::new(raw_big)
                .map(Some)
                .map_err(|e| errors.big_step = Some(e.to_string()))
                .ok()
        };
        let big_step = match (step, big) {
            (Some(step), Some(big)) => counter::check_big_step(step, big)
                .map_err(|e| errors.big_step = Some(e.to_string()))
                .ok(),
            _ => None,
        };
        self.errors.set(errors);
        Some(ValidCounter {
            name: name?,
            goal: goal?,
            step: step?,
            big_step: big_step?,
            celebration: (self.celebration)(),
        })
    }
}

/// The form body, the deck form's shape: labels over inputs, chips centered.
#[component]
pub fn CounterForm(state: CounterFormState) -> Element {
    let CounterFormState {
        name,
        amount,
        mut unit,
        mut step,
        mut big_step,
        mut celebration,
        errors,
    } = state;
    let mut hint = use_signal(|| None::<FormHint>);
    let mut hint_open = use_signal(|| false);
    let days = stats::days_in_year(ui::today().year());
    let preview = amount()
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0)
        .and_then(|n| unit().goal(n).ok())
        .map(|g| g.breakdown(days));
    let errors = errors();
    let field_error = |message: &Option<String>| message.as_deref().map(ui::sentence_case);

    rsx! {
        form { class: "flex-col text-center", onsubmit: move |e| e.prevent_default(),
            TextInput {
                value: name,
                id: "counter_name",
                label: "Name",
                placeholder: "Not set",
                maxlength: CounterName::MAX_LEN,
                error: field_error(&errors.name),
                on_info: move |_| { hint.set(Some(FormHint::Name)); hint_open.set(true); },
            }
            // A text field, not type=number: iOS WebKit drops keystrokes when a
            // number input's value is rewritten mid-typing, which a bound value
            // does on every key. inputmode still brings up the number pad, and
            // the filter keeps only digits.
            TextInput {
                value: amount,
                id: "counter_goal",
                label: "Goal",
                placeholder: "Not set",
                inputmode: "numeric",
                filter: move |typed: String| typed.chars().filter(char::is_ascii_digit).collect::<String>(),
                error: field_error(&errors.goal),
                on_info: move |_| { hint.set(Some(FormHint::Goal)); hint_open.set(true); },
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
                            class: if entered { "stat-chip stat-chip-p4" } else { "stat-chip stat-chip-muted" },
                            "{text}"
                        }
                    }
                }
            }
            FieldLabel {
                label: "Step",
                on_info: move |_| { hint.set(Some(FormHint::Step)); hint_open.set(true); },
            }
            div { class: "chip-row chip-row-center",
                for n in Step::ALLOWED {
                    Chip { selected: step() == n, onclick: move |_| step.set(n), "{n}" }
                }
            }
            if let Some(e) = field_error(&errors.step) {
                div { class: "message-error", "{e}" }
            }
            FieldLabel {
                label: "Big step",
                on_info: move |_| { hint.set(Some(FormHint::BigStep)); hint_open.set(true); },
            }
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
            if let Some(e) = field_error(&errors.big_step) {
                div { class: "message-error", "{e}" }
            }
            FieldLabel {
                label: "Goal animation",
                on_info: move |_| { hint.set(Some(FormHint::Celebration)); hint_open.set(true); },
            }
            div { class: "chip-row chip-row-center",
                // Default first: most counters should follow the app-wide
                // setting rather than each carrying its own opinion.
                Chip {
                    selected: celebration().is_none(),
                    onclick: move |_| celebration.set(None),
                    "Default"
                }
                for c in Celebration::ALL {
                    Chip {
                        selected: celebration() == Some(c),
                        onclick: move |_| celebration.set(Some(c)),
                        "{c.label()}"
                    }
                }
            }
            if let Some(e) = field_error(&errors.form) {
                p { class: "form-error", "{e}" }
            }
        }
        FormHintDialog { open: hint_open, which: hint() }
    }
}

/// Which field's hint is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormHint {
    Name,
    Goal,
    Step,
    BigStep,
    Celebration,
}

/// One dialog for the whole form, its content picked by `which`. The same
/// shape Config uses: a "?" per field beats one dialog trying to explain
/// every field at once.
#[component]
fn FormHintDialog(open: Signal<bool>, which: Option<FormHint>) -> Element {
    let Some(which) = which else {
        return rsx! {};
    };
    match which {
        FormHint::Name => rsx! {
            HintDialog { open, title: "Name",
                HintLine { "Names the counter and its CSV export" }
            }
        },
        FormHint::Goal => rsx! {
            HintDialog { open, title: "Goal",
                HintLine { "Optional. Any unit becomes what today has to clear" }
                HintLine { "Changing it leaves logged days alone" }
            }
        },
        FormHint::Step => rsx! {
            HintDialog { open, title: "Step",
                HintLine { "How much one tap adds, and minus takes off" }
            }
        },
        FormHint::BigStep => rsx! {
            HintDialog { open, title: "Big step",
                HintLine { "An optional second pair of buttons, outside the first" }
                HintLine { "Has to be larger than the step" }
            }
        },
        FormHint::Celebration => rsx! {
            HintDialog { open, title: "Goal animation",
                HintLine { "What plays when this counter finishes its day" }
                HintLine { HintKey { "Default" } " follows the animation set in Config" }
            }
        },
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
    current_celebration: Option<Celebration>,
    on_saved: EventHandler<()>,
) -> Element {
    let mut open = open;
    let store = use_store();
    let toast = use_toast();
    let mut form = use_hook(CounterFormState::default);

    let seed_name = current_name;
    use_effect(move || {
        if open() {
            form.load(
                &seed_name,
                current_goal,
                current_step,
                current_big_step,
                current_celebration,
            );
        }
    });

    let save = move |_| {
        let Some(v) = form.validate() else {
            return;
        };
        match store.update_counter(id, &v.name, v.goal, v.step, v.big_step, v.celebration) {
            Ok(()) => {
                toast.success(
                    format!("Saved {}", v.name),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                ui::bump_store_version();
                on_saved.call(());
                open.set(false);
            }
            Err(e) => form.fail(e.to_string()),
        }
    };

    rsx! {
        BottomSheet {
            open,
            title: "Edit counter",
            closable: true,
            CounterForm { state: form }
            // Save as the action dock's checkmark, pinned bottom right.
            div { class: "action-dock action-dock-right sheet-dock",
                button {
                    class: "action-dock-btn",
                    "aria-label": "Save",
                    onclick: save,
                    DockIcon { path: action_dock::CHECK }
                }
            }
        }
    }
}
