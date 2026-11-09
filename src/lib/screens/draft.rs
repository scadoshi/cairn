//! A pick held in a sheet until Save: Settings' Mark, Goal animation and
//! Rest days, and the App icon sheet.
//!
//! Each open snapshots what is set. Picking changes only the draft, so
//! tapping through the options to see them costs nothing; Save is off while
//! the draft matches the snapshot; Back, or a tap outside, says the setting
//! was left as it was when a pick is thrown away.

use crate::screens::toast::Toast;

/// Back on the Mark sheet with a pick thrown away.
pub const MARK_UNCHANGED: &str = "Mark unchanged";
/// Back on the Goal animation sheet with a pick thrown away.
pub const ANIMATION_UNCHANGED: &str = "Animation unchanged";
/// Back on the Rest days sheet with a pick thrown away.
pub const REST_DAYS_UNCHANGED: &str = "Rest days unchanged";
/// Back on the App icon sheet with a pick thrown away.
pub const APP_ICON_UNCHANGED: &str = "App icon unchanged";

/// A setting's value as set, and the pick a sheet holds over it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft<T> {
    saved: T,
    shown: T,
    unchanged: &'static str,
}

impl<T: Clone + PartialEq> Draft<T> {
    /// A draft over `saved`; `unchanged` is what Back says when it throws a
    /// pick away.
    pub fn new(saved: T, unchanged: &'static str) -> Self {
        Self {
            shown: saved.clone(),
            saved,
            unchanged,
        }
    }

    /// The sheet opened: start from what is set now.
    pub fn open(&mut self, saved: T) {
        self.shown = saved.clone();
        self.saved = saved;
    }

    /// A tap on an option.
    pub fn pick(&mut self, value: T) {
        self.shown = value;
    }

    /// The pick the sheet shows.
    pub fn shown(&self) -> &T {
        &self.shown
    }

    /// Whether Save does anything.
    pub fn can_save(&self) -> bool {
        self.shown != self.saved
    }

    /// Back or a tap outside: the toast to raise, if a pick was thrown away.
    pub fn back(&self) -> Option<Toast> {
        self.can_save().then(|| Toast::info(self.unchanged))
    }

    /// Save: the value to write, `None` when nothing changed.
    pub fn save(&self) -> Option<T> {
        self.can_save().then(|| self.shown.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copy_matches_the_app() {
        assert_eq!(MARK_UNCHANGED, "Mark unchanged");
        assert_eq!(ANIMATION_UNCHANGED, "Animation unchanged");
        assert_eq!(REST_DAYS_UNCHANGED, "Rest days unchanged");
        assert_eq!(APP_ICON_UNCHANGED, "App icon unchanged");
    }

    #[test]
    fn an_untouched_draft_has_nothing_to_save_and_backs_out_quietly() {
        let d = Draft::new(3, MARK_UNCHANGED);
        assert_eq!(*d.shown(), 3);
        assert!(!d.can_save());
        assert_eq!(d.save(), None);
        assert_eq!(d.back(), None);
    }

    #[test]
    fn a_pick_turns_save_on_and_back_says_unchanged() {
        let mut d = Draft::new(3, REST_DAYS_UNCHANGED);
        d.pick(5);
        assert_eq!(*d.shown(), 5);
        assert!(d.can_save());
        assert_eq!(d.save(), Some(5));
        assert_eq!(d.back(), Some(Toast::info("Rest days unchanged")));
    }

    #[test]
    fn picking_back_to_the_saved_value_is_no_change() {
        let mut d = Draft::new(3, MARK_UNCHANGED);
        d.pick(5);
        d.pick(3);
        assert!(!d.can_save());
        assert_eq!(d.back(), None);
    }

    #[test]
    fn each_open_starts_from_what_is_set() {
        let mut d = Draft::new(3, MARK_UNCHANGED);
        d.pick(5);
        d.open(7);
        assert_eq!(*d.shown(), 7);
        assert!(!d.can_save());
        d.pick(3);
        assert_eq!(d.save(), Some(3), "compared with the new snapshot");
    }

    #[test]
    fn match_theme_is_a_value_like_any_other() {
        let mut d: Draft<Option<String>> = Draft::new(Some("nord".to_owned()), APP_ICON_UNCHANGED);
        d.pick(None);
        assert_eq!(d.save(), Some(None));
        assert_eq!(d.back(), Some(Toast::info("App icon unchanged")));
    }
}
