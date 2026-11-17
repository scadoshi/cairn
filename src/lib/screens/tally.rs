//! The note a plain tap leaves, so a tap on the wrong counter or the wrong
//! step is easy to spot: the counter's name, the change and the day's count
//! after it, as "Pushups +1 (now 5)".
//!
//! Taps that come in a run on one counter fold into one note, which the
//! client raises once the run settles, so a burst of taps reads as one toast
//! with the whole change rather than a stack of them.

use crate::domain::counter::format;
use crate::screens::toast::Toast;

/// What a run of taps on one counter changed on its day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tally {
    name: String,
    change: i64,
    now: i64,
}

impl Tally {
    /// One tap on `name` that changed its day by `change`, leaving it at
    /// `now`. `None` for a tap that changed nothing.
    pub fn new(name: &str, change: i64, now: i64) -> Option<Self> {
        (change != 0).then(|| Self {
            name: name.to_owned(),
            change,
            now,
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
        self.now = next.now;
        Ok(())
    }

    /// The note: "Pushups +3 (now 8)", or "Pushups -1 (now 4)".
    pub fn text(&self) -> String {
        let sign = if self.change >= 0 { "+" } else { "-" };
        let change = format::thousands_i64(self.change.saturating_abs());
        let now = format::thousands_i64(self.now);
        format!("{} {sign}{change} (now {now})", self.name)
    }

    /// The note as the routine toast it is raised as.
    pub fn toast(&self) -> Toast {
        Toast::info(self.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tally(name: &str, change: i64, now: i64) -> Tally {
        Tally::new(name, change, now).expect("a tap that changed something")
    }

    #[test]
    fn a_tap_names_its_counter_the_change_and_the_count_after() {
        assert_eq!(tally("Pushups", 1, 5).text(), "Pushups +1 (now 5)");
        assert_eq!(tally("Pushups", -1, 4).text(), "Pushups -1 (now 4)");
        assert_eq!(
            tally("Steps", 2500, 12_000).text(),
            "Steps +2,500 (now 12,000)"
        );
    }

    #[test]
    fn a_tap_that_changed_nothing_leaves_no_note() {
        assert_eq!(Tally::new("Pushups", 0, 0), None);
    }

    #[test]
    fn a_run_on_one_counter_reads_as_its_whole_change() {
        let mut run = tally("Pushups", 1, 5);
        for now in 6..=8 {
            assert_eq!(run.absorb(tally("Pushups", 1, now)), Ok(()));
        }
        assert_eq!(run.absorb(tally("Pushups", -1, 7)), Ok(()));
        assert_eq!(run.text(), "Pushups +3 (now 7)");
    }

    #[test]
    fn a_tap_on_another_counter_starts_its_own_run() {
        let mut run = tally("Pushups", 1, 5);
        let other = tally("Water", 1, 2);
        assert_eq!(run.absorb(other.clone()), Err(other));
        assert_eq!(run.text(), "Pushups +1 (now 5)");
    }

    #[test]
    fn it_is_raised_as_a_routine_toast() {
        assert_eq!(
            tally("Pushups", 1, 5).toast(),
            Toast::info("Pushups +1 (now 5)")
        );
    }
}
