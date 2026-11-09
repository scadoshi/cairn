//! The counter form body: name, goal with its unit, and step, plus the edit
//! sheet that wraps it. Shared by the create sheet on the list, the edit
//! button on each card, and the counter screen, so the three never drift.
//! The host owns the signals and does the saving.

use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;
use dioxus_primitives::toast::{Toasts, use_toast};
use orangui::{
    ActionDock, BottomSheet, Chip, DockAction, DockPlace, FieldLabel, HintDialog, HintKey,
    HintLine, TextInput, icons,
};

use crate::domain::counter::{CounterId, CounterName, Goal, Step, stats};
use crate::domain::preferences::Celebration;
use crate::inbound::ui::{self, SharedStore, use_prefs, use_store};
use crate::screens::counter_form::{
    Fields, FormErrors, GoalUnit, Mode, ValidCounter, Write, saved,
};
use crate::screens::toast::Toast;

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
        self.set(&Fields::of(name, goal, step, big_step, celebration));
    }

    /// Clears the form for a new counter.
    pub fn clear(&mut self) {
        self.set(&Fields::blank());
    }

    fn set(&mut self, fields: &Fields) {
        self.name.set(fields.name.clone());
        self.amount.set(fields.amount.clone());
        // A counter without a goal leaves the unit as it was.
        if !fields.amount.is_empty() {
            self.unit.set(fields.unit);
        }
        self.step.set(fields.step);
        self.big_step.set(fields.big_step);
        self.celebration.set(fields.celebration);
        self.errors.set(FormErrors::default());
    }

    /// What the form holds now.
    pub fn fields(&self) -> Fields {
        Fields {
            name: (self.name)(),
            amount: (self.amount)(),
            unit: (self.unit)(),
            step: (self.step)(),
            big_step: (self.big_step)(),
            celebration: (self.celebration)(),
        }
    }

    /// Records `message` at the bottom of the form, under every field.
    pub fn fail(&mut self, message: String) {
        self.errors.set(FormErrors::failed(message));
    }

    /// Checks the fields into domain values, or marks every field that is
    /// wrong.
    pub fn validate(&mut self) -> Option<ValidCounter> {
        let checked = self.fields().validate();
        let errors = checked.as_ref().err().cloned().unwrap_or_default();
        self.errors.set(errors);
        checked.ok()
    }
}

/// Saves the form in `mode`: checks it and writes it, and on success
/// toasts "Saved {name}" and tells the app the store changed. Returns
/// whether it saved; a failed write leaves its message at the bottom of the
/// form. A new counter starts on `today`.
pub fn save_form(
    form: &mut CounterFormState,
    mode: Mode,
    store: &SharedStore,
    toasts: Toasts,
    today: NaiveDate,
) -> bool {
    let Some(valid) = form.validate() else {
        return false;
    };
    let name = valid.name.clone();
    let written = match mode.write(valid) {
        Write::Create(v) => store
            .create_counter(&v.name, v.goal, v.step, v.big_step, v.celebration, today)
            .map(|_| ()),
        Write::Update(id, v) => {
            store.update_counter(id, &v.name, v.goal, v.step, v.big_step, v.celebration)
        }
    };
    match written {
        Ok(()) => {
            ui::raise(toasts, &Toast::success(saved(&name)));
            ui::bump_store_version();
            true
        }
        Err(e) => {
            form.fail(e.to_string());
            false
        }
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
    let days = stats::days_in_year(ui::shown_day().year());
    // Only what the preview and the big step chips need, so typing the
    // name does not re-render the form.
    let shown = Fields {
        amount: amount(),
        unit: unit(),
        step: step(),
        ..Fields::blank()
    };
    let preview = shown.preview(days);
    let big_steps = shown.big_steps();
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
                for n in big_steps {
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
                HintLine { HintKey { "Default" } " follows the animation set in Settings" }
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

    let prefs = use_prefs();
    let save = move |()| {
        let today = ui::today(&prefs.peek());
        if save_form(&mut form, Mode::Edit(id), &store, toast, today) {
            on_saved.call(());
            open.set(false);
        }
    };

    rsx! {
        BottomSheet {
            open,
            title: "Edit counter",
            closable: true,
            CounterForm { state: form }
            // Save as the action dock's checkmark, pinned bottom right.
            ActionDock { place: DockPlace::Sheet,
                DockAction { label: "Save", icon: icons::CHECK, onclick: save }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_marks_every_wrong_field_at_once() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let mut state = CounterFormState::default();
            state.name.set("   ".to_string());
            state.step.set(10);
            state.big_step.set(5);
            assert!(state.validate().is_none());
            let errors = (state.errors)();
            assert!(errors.name.is_some(), "{errors:?}");
            assert!(errors.big_step.is_some(), "{errors:?}");
            assert!(errors.goal.is_none(), "{errors:?}");
            assert!(errors.step.is_none(), "{errors:?}");
        });
    }
}
