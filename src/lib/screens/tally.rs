//! The note a plain tap leaves, so a tap on the wrong counter or the wrong
//! step is easy to spot: the counter's name and the change, as "Pushups +1".
//!
//! A tap shows its note at once, in its counter's own live toast; a tap while
//! that toast is up adds to the change it shows and keeps it up, so a burst of
//! taps reads as one toast counting up rather than a stack of them.

use crate::domain::counter::format;
use crate::screens::toast::Toast;

/// What a run of taps on one counter changed on its day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tally {
    name: String,
    change: i64,
}

impl Tally {
    /// One tap on `name` that changed its day by `change`. `None` for a tap
    /// that changed nothing.
    pub fn new(name: &str, change: i64) -> Option<Self> {
        (change != 0).then(|| Self {
            name: name.to_owned(),
            change,
        })
    }

    /// Folds `next` into this run when it is a tap on the same counter, and
    /// hands it back when it is a tap on another one.
    ///
    /// # Errors
    ///
    /// Returns `next` unchanged when it names another counter, which starts a
    /// run of its own.
    pub fn absorb(&mut self, next: Self) -> Result<(), Self> {
        if next.name != self.name {
            return Err(next);
        }
        self.change = self.change.saturating_add(next.change);
        Ok(())
    }

    /// The key its counter's live toast shows under, one per counter.
    pub fn key(&self) -> String {
        format!("tally:{}", self.name)
    }

    /// The change to show after `tap`: added to `showing`, the running change, while the
    /// counter's toast is still `up`, or `tap` alone once it has left or when there is none.
    pub fn after(showing: Option<Self>, up: bool, tap: Self) -> Self {
        match showing.filter(|_| up) {
            Some(mut running) => match running.absorb(tap) {
                Ok(()) => running,
                Err(other) => other,
            },
            None => tap,
        }
    }

    /// The note: "Pushups +3", or "Pushups -1".
    pub fn text(&self) -> String {
        let sign = if self.change >= 0 { "+" } else { "-" };
        let change = format::thousands_i64(self.change.saturating_abs());
        format!("{} {sign}{change}", self.name)
    }

    /// The note as the routine toast it is raised as.
    pub fn toast(&self) -> Toast {
        Toast::info(self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tally(name: &str, change: i64) -> Tally {
        Tally::new(name, change).expect("a tap that changed something")
    }

    #[test]
    fn a_tap_names_its_counter_and_the_change() {
        assert_eq!(tally("Pushups", 1).text(), "Pushups +1");
        assert_eq!(tally("Pushups", -1).text(), "Pushups -1");
        assert_eq!(tally("Steps", 2500).text(), "Steps +2,500");
    }

    #[test]
    fn a_tap_that_changed_nothing_leaves_no_note() {
        assert_eq!(Tally::new("Pushups", 0), None);
    }

    #[test]
    fn a_run_on_one_counter_reads_as_its_whole_change() {
        let mut run = tally("Pushups", 1);
        for _ in 0..3 {
            assert_eq!(run.absorb(tally("Pushups", 1)), Ok(()));
        }
        assert_eq!(run.absorb(tally("Pushups", -1)), Ok(()));
        assert_eq!(run.text(), "Pushups +3");
    }

    #[test]
    fn a_tap_on_another_counter_starts_its_own_run() {
        let mut run = tally("Pushups", 1);
        let other = tally("Water", 1);
        assert_eq!(run.absorb(other.clone()), Err(other));
        assert_eq!(run.text(), "Pushups +1");
    }

    #[test]
    fn a_tap_while_its_toast_is_up_adds_to_the_running_change() {
        let mut shown = Tally::after(None, false, tally("Pull-ups", 1));
        assert_eq!(shown.text(), "Pull-ups +1");
        for _ in 0..4 {
            shown = Tally::after(Some(shown), true, tally("Pull-ups", 1));
        }
        assert_eq!(shown.text(), "Pull-ups +5");
    }

    #[test]
    fn a_tap_after_its_toast_left_starts_over() {
        let shown = tally("Pull-ups", 3);
        assert_eq!(
            Tally::after(Some(shown), false, tally("Pull-ups", 1)).text(),
            "Pull-ups +1"
        );
    }

    #[test]
    fn each_counter_has_its_own_toast() {
        assert_eq!(tally("Pull-ups", 1).key(), "tally:Pull-ups");
        assert_ne!(tally("Pull-ups", 1).key(), tally("Water", 1).key());
    }

    #[test]
    fn it_is_raised_as_a_routine_toast() {
        assert_eq!(tally("Pushups", 1).toast(), Toast::info("Pushups +1"));
    }
}
