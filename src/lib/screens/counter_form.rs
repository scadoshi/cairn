//! The Create and Edit counter sheets' form: what it holds, how it checks,
//! and what a save writes and says.
//!
//! One model in two modes, as zwipe's deck form is. The typed name and goal
//! stay bound to the client's inputs; the client reads them into [`Fields`]
//! before checking, so every rule here is plain data.

use crate::domain::counter::models::{CounterId, CounterName, Goal, Step, ValidationError};
use crate::domain::counter::{self};
use crate::domain::preferences::Celebration;

/// The goal field's message for anything but a whole number of at least 1.
pub const GOAL_NOT_A_NUMBER: &str = "goal must be a whole number of at least 1";

/// The toast after a save.
pub fn saved(name: &CounterName) -> String {
    format!("Saved {name}")
}

/// Which period a typed goal amount is for.
#[expect(missing_docs, reason = "Day, Week and Year say what they are")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GoalUnit {
    #[default]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidCounter {
    /// Display name.
    pub name: CounterName,
    /// Optional target.
    pub goal: Option<Goal>,
    /// How much one tap adds.
    pub step: Step,
    /// The optional second, larger step.
    pub big_step: Option<Step>,
    /// This counter's celebration, or `None` to follow Settings.
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
    /// A failed save's `message`, at the bottom under every field.
    pub fn failed(message: String) -> Self {
        Self {
            form: Some(message),
            ..Self::default()
        }
    }
}

/// What the form holds, as typed and picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fields {
    /// The name as typed.
    pub name: String,
    /// The goal amount as typed; blank means no goal.
    pub amount: String,
    /// Which period the amount is for.
    pub unit: GoalUnit,
    /// The step, one of [`Step::ALLOWED`].
    pub step: u32,
    /// The big step; zero means none, which keeps the bar at three buttons.
    pub big_step: u32,
    /// The counter's own celebration; `None` follows Settings.
    pub celebration: Option<Celebration>,
}

impl Fields {
    /// A new counter's form: no name, no goal, a step of 1.
    pub fn blank() -> Self {
        Self::of("", None, 1, None, None)
    }

    /// The form for a counter with these values.
    pub fn of(
        name: &str,
        goal: Option<Goal>,
        step: u32,
        big_step: Option<u32>,
        celebration: Option<Celebration>,
    ) -> Self {
        let (amount, unit) = match goal {
            Some(g @ (Goal::PerDay(n) | Goal::PerWeek(n) | Goal::PerYear(n))) => {
                (n.to_string(), GoalUnit::of(g))
            }
            None => (String::new(), GoalUnit::Day),
        };
        Self {
            name: name.to_owned(),
            amount,
            unit,
            step,
            big_step: big_step.unwrap_or(0),
            celebration,
        }
    }

    /// The checked counter, or every field that is wrong at once.
    ///
    /// # Errors
    ///
    /// [`FormErrors`] naming each field that does not check out.
    pub fn validate(&self) -> Result<ValidCounter, FormErrors> {
        let mut errors = FormErrors::default();
        let name = CounterName::new(&self.name)
            .map_err(|e| errors.name = Some(e.to_string()))
            .ok();
        let goal = if self.amount.trim().is_empty() {
            Some(None)
        } else if let Some(goal) = self.goal() {
            Some(Some(goal))
        } else {
            errors.goal = Some(GOAL_NOT_A_NUMBER.to_owned());
            None
        };
        let step = Step::new(self.step)
            .map_err(|e| errors.step = Some(e.to_string()))
            .ok();
        // Zero is the form's way of saying none; the domain decides whether
        // what is left is a sensible pair.
        let big = if self.big_step == 0 {
            Some(None)
        } else {
            Step::new(self.big_step)
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
        match (name, goal, step, big_step) {
            (Some(name), Some(goal), Some(step), Some(big_step)) => Ok(ValidCounter {
                name,
                goal,
                step,
                big_step,
                celebration: self.celebration,
            }),
            _ => Err(errors),
        }
    }

    /// The typed amount as a goal in the picked unit, when it is one.
    fn goal(&self) -> Option<Goal> {
        let n = self.amount.trim().parse::<u32>().ok()?;
        self.unit.goal(n).ok()
    }

    /// The goal as a day, a week and a year, the unit typed marked, once
    /// the amount is a goal.
    pub fn preview(&self, days_in_year: u32) -> Option<[(String, bool); 3]> {
        self.goal().map(|g| g.breakdown(days_in_year))
    }

    /// The big steps on offer: those larger than the step.
    pub fn big_steps(&self) -> Vec<u32> {
        Step::ALLOWED
            .into_iter()
            .filter(|n| *n > self.step)
            .collect()
    }
}

/// Which sheet the form is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Create counter.
    Create,
    /// Edit counter, on this counter.
    Edit(CounterId),
}

/// What a checked form writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Write {
    /// A new counter.
    Create(ValidCounter),
    /// An existing counter's new values.
    Update(CounterId, ValidCounter),
}

impl Mode {
    /// The write `valid` becomes in this mode.
    pub fn write(self, valid: ValidCounter) -> Write {
        match self {
            Self::Create => Write::Create(valid),
            Self::Edit(id) => Write::Update(id, valid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(name: &str, amount: &str, step: u32, big_step: u32) -> Fields {
        Fields {
            name: name.to_owned(),
            amount: amount.to_owned(),
            big_step,
            step,
            ..Fields::blank()
        }
    }

    #[test]
    fn the_copy_matches_the_app() {
        assert_eq!(
            saved(&CounterName::new("Pushups").unwrap()),
            "Saved Pushups"
        );
        assert_eq!(
            GOAL_NOT_A_NUMBER,
            "goal must be a whole number of at least 1"
        );
    }

    #[test]
    fn a_blank_form_is_no_name_no_goal_and_a_step_of_one() {
        let blank = Fields::blank();
        assert_eq!((blank.name.as_str(), blank.amount.as_str()), ("", ""));
        assert_eq!(
            (blank.unit, blank.step, blank.big_step, blank.celebration),
            (GoalUnit::Day, 1, 0, None)
        );
    }

    #[test]
    fn a_counter_loads_its_values_with_its_goals_unit() {
        let f = Fields::of(
            "Pages",
            Some(Goal::per_week(70).unwrap()),
            5,
            Some(20),
            Some(Celebration::Pulse),
        );
        assert_eq!(f.name, "Pages");
        assert_eq!((f.amount.as_str(), f.unit), ("70", GoalUnit::Week));
        assert_eq!(
            (f.step, f.big_step, f.celebration),
            (5, 20, Some(Celebration::Pulse))
        );
        let none = Fields::of("Pages", None, 1, None, None);
        assert_eq!((none.amount.as_str(), none.big_step), ("", 0));
    }

    #[test]
    fn validate_marks_every_wrong_field_at_once() {
        let errors = fields("   ", "", 10, 5).validate().unwrap_err();
        assert!(errors.name.is_some(), "{errors:?}");
        assert!(errors.big_step.is_some(), "{errors:?}");
        assert!(errors.goal.is_none(), "{errors:?}");
        assert!(errors.step.is_none(), "{errors:?}");
        assert!(errors.form.is_none(), "{errors:?}");
    }

    #[test]
    fn a_goal_that_is_not_a_whole_number_of_at_least_one_is_wrong() {
        for amount in ["0", "abc", "-3", "1.5"] {
            let errors = fields("Pushups", amount, 1, 0).validate().unwrap_err();
            assert_eq!(
                errors.goal.as_deref(),
                Some(GOAL_NOT_A_NUMBER),
                "{amount:?}"
            );
        }
    }

    #[test]
    fn a_good_form_checks_into_a_counter() {
        let mut f = fields(" Pushups ", " 50 ", 5, 20);
        f.unit = GoalUnit::Year;
        f.celebration = Some(Celebration::Confetti);
        let v = f.validate().unwrap();
        assert_eq!(v.name, CounterName::new("Pushups").unwrap());
        assert_eq!(v.goal, Some(Goal::per_year(50).unwrap()));
        assert_eq!((v.step.get(), v.big_step.map(Step::get)), (5, Some(20)));
        assert_eq!(v.celebration, Some(Celebration::Confetti));
        assert_eq!(
            fields("Pushups", "", 1, 0).validate().unwrap().goal,
            None,
            "blank is no goal"
        );
    }

    #[test]
    fn the_preview_shows_once_the_amount_is_a_goal() {
        let f = fields("Pushups", "10", 1, 0);
        assert_eq!(
            f.preview(365),
            Some(Goal::per_day(10).unwrap().breakdown(365))
        );
        assert_eq!(fields("Pushups", "", 1, 0).preview(365), None);
        assert_eq!(fields("Pushups", "0", 1, 0).preview(365), None);
    }

    #[test]
    fn big_steps_are_larger_than_the_step() {
        assert_eq!(fields("a", "", 25, 0).big_steps(), [30, 40, 50, 100]);
        assert_eq!(fields("a", "", 100, 0).big_steps(), Vec::<u32>::new());
    }

    #[test]
    fn a_failed_save_sits_at_the_bottom_alone() {
        let errors = FormErrors::failed("disk full".to_owned());
        assert_eq!(
            errors,
            FormErrors {
                form: Some("disk full".to_owned()),
                ..FormErrors::default()
            }
        );
    }

    #[test]
    fn create_writes_a_new_counter_and_edit_updates_its_own() {
        let v = fields("Pushups", "", 1, 0).validate().unwrap();
        assert_eq!(Mode::Create.write(v.clone()), Write::Create(v.clone()));
        assert_eq!(
            Mode::Edit(CounterId(4)).write(v.clone()),
            Write::Update(CounterId(4), v)
        );
    }
}
