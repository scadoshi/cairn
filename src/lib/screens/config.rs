//! Settings: the preference buttons, the hint topics, and the Data rows'
//! backup, import and CSV export with what each says.
//!
//! Preferences live in the app's signal, which the app persists; a button
//! here asks for the next value. The files are the client's to write and
//! read: an export or a backup is an effect, answered with what happened.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use chrono::Weekday;

use crate::domain::counter::CounterName;
use crate::domain::counter::backup::{Backup, OnConflict, Restored};
use crate::domain::preferences::Preferences;
use crate::screens::text::sentence_case;
use crate::screens::toast::Toast;

/// A Backup Export that worked.
pub const BACKUP_SAVED: &str = "Backup saved";
/// A Backup Export that failed.
pub const BACKUP_FAILED: &str = "Backup failed";
/// A CSV export that failed.
pub const EXPORT_FAILED: &str = "Export failed";
/// A Backup Import that failed.
pub const IMPORT_FAILED: &str = "Import failed";
/// What Replace and Keep mine do, under the conflict dialog's names.
pub const CONFLICT_BODY: &str = "Replace swaps yours, and their history, for the backup's. Keep mine leaves yours and imports the rest.";

/// Which row's hint is showing.
#[expect(missing_docs, reason = "each names its Settings row")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Theme,
    AppIcon,
    Mark,
    Celebration,
    Dates,
    DayStarts,
    WeekStarts,
    RestDays,
    SortCounters,
    ConfirmMinus,
    Backup,
    Export,
}

/// A preference button that steps to its next value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Day starts: midnight, then each later rollover hour, round again.
    DayStarts,
    /// Week starts: Monday or Sunday.
    WeekStarts,
    /// Sort counters: the next order.
    SortCounters,
    /// Confirm minus: on or off.
    ConfirmMinus,
}

/// `prefs` with `step`'s preference moved to its next value.
pub fn stepped(prefs: Preferences, step: Step) -> Preferences {
    let mut next = prefs;
    match step {
        Step::DayStarts => {
            let hours = Preferences::ROLLOVER_HOURS;
            let at = hours
                .iter()
                .position(|h| *h == prefs.rollover_hour)
                .unwrap_or(0);
            next.rollover_hour = at
                .saturating_add(1)
                .checked_rem(hours.len())
                .and_then(|i| hours.get(i))
                .copied()
                .unwrap_or(0);
        }
        Step::WeekStarts => {
            next.week_start = if prefs.week_start == Weekday::Mon {
                Weekday::Sun
            } else {
                Weekday::Mon
            };
        }
        Step::SortCounters => next.counter_order = prefs.counter_order.next(),
        Step::ConfirmMinus => next.confirm_minus = !prefs.confirm_minus,
    }
    next
}

/// "Midnight", "2am", "4am", "6am".
pub fn rollover_label(hour: u32) -> String {
    match hour {
        0 => "Midnight".to_owned(),
        h => format!("{h}am"),
    }
}

/// The Rest days row's value: how many weekdays rest, or None.
pub fn rest_label(mask: u8) -> String {
    match (0..7).filter(|i| mask & (1 << i) != 0).count() {
        0 => "None".to_owned(),
        n => n.to_string(),
    }
}

/// "Exported 1 file", "Exported 3 files".
pub fn exported_line(n: usize) -> String {
    if n == 1 {
        "Exported 1 file".to_owned()
    } else {
        format!("Exported {n} files")
    }
}

/// "1 counter is already here", "3 counters are already here".
pub fn conflict_title(n: usize) -> String {
    if n == 1 {
        "1 counter is already here".to_owned()
    } else {
        format!("{n} counters are already here")
    }
}

/// The shared names, comma separated.
pub fn conflict_names(names: &[CounterName]) -> String {
    names
        .iter()
        .map(CounterName::as_str)
        .collect::<Vec<&str>>()
        .join(", ")
}

/// What an import did, as its toast says it.
pub fn restored_line(done: Restored) -> String {
    let imported = done.added.saturating_add(done.replaced);
    let mut line = match imported {
        1 => "Imported 1 counter".to_owned(),
        n => format!("Imported {n} counters"),
    };
    if done.replaced > 0 {
        let _ = write!(line, ", {} replaced", done.replaced);
    }
    if done.kept > 0 {
        let _ = write!(line, ", kept {} of yours", done.kept);
    }
    line
}

/// "Written to {path}", for a file or the folder files went into.
fn written_to(path: &Path) -> String {
    format!("Written to {}", path.display())
}

/// A backup read and checked, with the names it shares with the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingImport {
    /// The backup.
    pub backup: Backup,
    /// Its counters' names the store already has.
    pub conflicts: Vec<CounterName>,
    /// The file it came from.
    pub file: PathBuf,
}

/// Settings' model.
#[derive(Debug, Default)]
pub struct State {
    notice: Option<String>,
    pending: Option<PendingImport>,
    /// The file a restore under way came from, for the notice after it.
    restoring: Option<PathBuf>,
    topic: Option<Topic>,
}

/// What happened on Settings.
#[derive(Debug, Clone)]
pub enum Action {
    /// A preference button, with the preferences as they are.
    Step(Step, Preferences),
    /// A row's ? or the button that opens its sheet.
    Topic(Topic),
    /// The CSV button.
    ExportCsv,
    /// The client's answer to [`Effect::ExportCsv`]: how many files and the
    /// folder, or what failed.
    Exported(Result<(usize, PathBuf), String>),
    /// Backup's Export.
    SaveBackup,
    /// The client's answer to [`Effect::SaveBackup`]: the file, or what failed.
    BackupSaved(Result<PathBuf, String>),
    /// Backup's Import.
    Import,
    /// The client's answer to [`Effect::ReadBackup`].
    BackupRead(Result<PendingImport, String>),
    /// Keep mine or Replace in the conflict dialog.
    Restore(OnConflict),
    /// The store's answer to [`Effect::Restore`].
    Restored(Result<Restored, String>),
}

/// What Settings asks the client to do.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Set the app's preferences.
    SetPrefs(Preferences),
    /// Write one CSV per counter, answering [`Action::Exported`].
    ExportCsv,
    /// Write the backup file, answering [`Action::BackupSaved`].
    SaveBackup,
    /// Read and check the newest backup, answering [`Action::BackupRead`].
    ReadBackup,
    /// Open the conflict dialog over the pending import.
    AskConflict,
    /// Import the backup, answering [`Action::Restored`].
    Restore(Backup, OnConflict),
    /// Raise a toast.
    Toast(Toast),
    /// Tell the rest of the app the store changed.
    StoreChanged,
}

impl State {
    /// Applies `action` and returns what the client should do, in order.
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Step(step, prefs) => vec![Effect::SetPrefs(stepped(prefs, step))],
            Action::Topic(topic) => {
                self.topic = Some(topic);
                Vec::new()
            }
            Action::ExportCsv => vec![Effect::ExportCsv],
            Action::Exported(Ok((n, dir))) => {
                self.notice = Some(written_to(&dir));
                vec![Effect::Toast(Toast::success(exported_line(n)))]
            }
            Action::Exported(Err(e)) => self.failed(EXPORT_FAILED, e),
            Action::SaveBackup => vec![Effect::SaveBackup],
            Action::BackupSaved(Ok(file)) => {
                self.notice = Some(written_to(&file));
                vec![Effect::Toast(Toast::success(BACKUP_SAVED))]
            }
            Action::BackupSaved(Err(e)) => self.failed(BACKUP_FAILED, e),
            Action::Import => vec![Effect::ReadBackup],
            // With no name in common it imports at once; otherwise it asks.
            Action::BackupRead(Ok(found)) => {
                let ask = !found.conflicts.is_empty();
                self.pending = Some(found);
                if ask {
                    vec![Effect::AskConflict]
                } else {
                    self.restore(OnConflict::Keep)
                }
            }
            Action::BackupRead(Err(e)) => self.failed(IMPORT_FAILED, e),
            Action::Restore(on_conflict) => self.restore(on_conflict),
            Action::Restored(result) => self.restored(result),
        }
    }

    /// The error toast `toast`, with `why` under the Data rows.
    fn failed(&mut self, toast: &str, why: String) -> Vec<Effect> {
        self.notice = Some(why);
        vec![Effect::Toast(Toast::error(toast))]
    }

    /// Takes the pending import into a restore; nothing when none waits.
    fn restore(&mut self, on_conflict: OnConflict) -> Vec<Effect> {
        let Some(import) = self.pending.take() else {
            return Vec::new();
        };
        self.restoring = Some(import.file);
        vec![Effect::Restore(import.backup, on_conflict)]
    }

    fn restored(&mut self, result: Result<Restored, String>) -> Vec<Effect> {
        let Some(file) = self.restoring.take() else {
            return Vec::new();
        };
        match result {
            Ok(done) => {
                self.notice = Some(format!("Imported from {}", file.display()));
                vec![
                    Effect::StoreChanged,
                    Effect::Toast(Toast::success(restored_line(done))),
                ]
            }
            Err(e) => self.failed(
                IMPORT_FAILED,
                format!("{}. Nothing was changed", sentence_case(&e)),
            ),
        }
    }

    /// The line under the Data rows: where a file went, or what failed.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// The import waiting on the conflict dialog.
    pub fn pending(&self) -> Option<&PendingImport> {
        self.pending.as_ref()
    }

    /// The hint topic last asked for.
    pub fn topic(&self) -> Option<Topic> {
        self.topic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::counter::backup;
    use crate::domain::preferences::CounterOrder;
    use crate::screens::toast::ToastKind;

    fn name(n: &str) -> CounterName {
        CounterName::new(n).unwrap()
    }

    fn pending(conflicts: &[&str]) -> PendingImport {
        PendingImport {
            backup: backup::parse(r#"{"format":"cairn-backup","version":1,"counters":[]}"#)
                .unwrap(),
            conflicts: conflicts.iter().map(|c| name(c)).collect(),
            file: PathBuf::from("/exports/cairn-backup.json"),
        }
    }

    fn toast(effects: &[Effect]) -> Option<&Toast> {
        effects.iter().find_map(|e| match e {
            Effect::Toast(t) => Some(t),
            _ => None,
        })
    }

    #[test]
    fn the_copy_matches_the_app() {
        assert_eq!(BACKUP_SAVED, "Backup saved");
        assert_eq!(BACKUP_FAILED, "Backup failed");
        assert_eq!(EXPORT_FAILED, "Export failed");
        assert_eq!(IMPORT_FAILED, "Import failed");
        assert_eq!(
            CONFLICT_BODY,
            "Replace swaps yours, and their history, for the backup's. Keep mine leaves yours and imports the rest."
        );
        assert_eq!(exported_line(0), "Exported 0 files");
        assert_eq!(exported_line(1), "Exported 1 file");
        assert_eq!(exported_line(3), "Exported 3 files");
        assert_eq!(conflict_title(1), "1 counter is already here");
        assert_eq!(conflict_title(2), "2 counters are already here");
        assert_eq!(
            conflict_names(&[name("Pushups"), name("Pages")]),
            "Pushups, Pages"
        );
        assert_eq!(rollover_label(0), "Midnight");
        assert_eq!(rollover_label(4), "4am");
        assert_eq!(rest_label(0), "None");
        assert_eq!(rest_label(0b0110_0000), "2");
    }

    #[test]
    fn an_import_toast_counts_what_it_did() {
        let done = |added, replaced, kept| Restored {
            added,
            replaced,
            kept,
        };
        assert_eq!(restored_line(done(1, 0, 0)), "Imported 1 counter");
        assert_eq!(
            restored_line(done(2, 1, 0)),
            "Imported 3 counters, 1 replaced"
        );
        assert_eq!(
            restored_line(done(2, 0, 4)),
            "Imported 2 counters, kept 4 of yours"
        );
    }

    #[test]
    fn each_button_steps_its_preference() {
        let prefs = Preferences::default();
        let hours = Preferences::ROLLOVER_HOURS;
        let last = *hours.last().unwrap();
        let wrapped = stepped(
            Preferences {
                rollover_hour: last,
                ..prefs
            },
            Step::DayStarts,
        );
        assert_eq!(
            wrapped.rollover_hour, hours[0],
            "round again after the last hour"
        );
        assert_eq!(stepped(prefs, Step::DayStarts).rollover_hour, hours[1]);
        let week = stepped(prefs, Step::WeekStarts).week_start;
        assert_ne!(week, prefs.week_start);
        assert_eq!(
            stepped(
                Preferences {
                    week_start: week,
                    ..prefs
                },
                Step::WeekStarts
            )
            .week_start,
            prefs.week_start
        );
        assert!(matches!(week, Weekday::Mon | Weekday::Sun));
        assert_eq!(
            stepped(prefs, Step::SortCounters).counter_order,
            prefs.counter_order.next()
        );
        assert_eq!(
            stepped(prefs, Step::ConfirmMinus).confirm_minus,
            !prefs.confirm_minus
        );
        let odd = Preferences {
            rollover_hour: 3,
            ..prefs
        };
        assert_eq!(
            stepped(odd, Step::DayStarts).rollover_hour,
            hours[1],
            "an hour not on the list steps from the first"
        );
        let _ = CounterOrder::default();
    }

    #[test]
    fn a_step_sets_the_preferences_and_a_topic_is_held() {
        let mut state = State::default();
        let prefs = Preferences::default();
        let effects = state.update(Action::Step(Step::ConfirmMinus, prefs));
        assert!(
            matches!(&effects[..], [Effect::SetPrefs(p)] if p.confirm_minus != prefs.confirm_minus),
            "{effects:?}"
        );
        assert!(state.update(Action::Topic(Topic::Backup)).is_empty());
        assert_eq!(state.topic(), Some(Topic::Backup));
    }

    #[test]
    fn a_csv_export_says_how_many_and_where() {
        let mut state = State::default();
        assert!(matches!(
            state.update(Action::ExportCsv)[..],
            [Effect::ExportCsv]
        ));
        let effects = state.update(Action::Exported(Ok((2, PathBuf::from("/exports")))));
        assert_eq!(toast(&effects), Some(&Toast::success("Exported 2 files")));
        assert_eq!(state.notice(), Some("Written to /exports"));
        let effects = state.update(Action::Exported(Err("disk full".to_owned())));
        assert_eq!(toast(&effects), Some(&Toast::error("Export failed")));
        assert_eq!(state.notice(), Some("disk full"));
    }

    #[test]
    fn a_backup_says_where_it_went() {
        let mut state = State::default();
        assert!(matches!(
            state.update(Action::SaveBackup)[..],
            [Effect::SaveBackup]
        ));
        let effects = state.update(Action::BackupSaved(Ok(PathBuf::from("/exports/b.json"))));
        assert_eq!(toast(&effects), Some(&Toast::success("Backup saved")));
        assert_eq!(state.notice(), Some("Written to /exports/b.json"));
        let effects = state.update(Action::BackupSaved(Err("no folder".to_owned())));
        assert_eq!(toast(&effects), Some(&Toast::error("Backup failed")));
        assert_eq!(state.notice(), Some("no folder"));
    }

    #[test]
    fn an_import_with_nothing_in_common_restores_at_once() {
        let mut state = State::default();
        assert!(matches!(
            state.update(Action::Import)[..],
            [Effect::ReadBackup]
        ));
        let effects = state.update(Action::BackupRead(Ok(pending(&[]))));
        assert!(
            matches!(&effects[..], [Effect::Restore(_, OnConflict::Keep)]),
            "{effects:?}"
        );
        assert!(state.pending().is_none(), "taken by the restore");
    }

    #[test]
    fn an_import_with_names_in_common_asks_first() {
        let mut state = State::default();
        let effects = state.update(Action::BackupRead(Ok(pending(&["Pushups"]))));
        assert!(matches!(&effects[..], [Effect::AskConflict]), "{effects:?}");
        assert_eq!(state.pending().map(|p| p.conflicts.len()), Some(1));
        let effects = state.update(Action::Restore(OnConflict::Replace));
        assert!(
            matches!(&effects[..], [Effect::Restore(_, OnConflict::Replace)]),
            "{effects:?}"
        );
        assert!(
            state.update(Action::Restore(OnConflict::Keep)).is_empty(),
            "nothing is pending"
        );
    }

    #[test]
    fn a_restore_tells_the_app_toasts_and_names_the_file() {
        let mut state = State::default();
        state.update(Action::BackupRead(Ok(pending(&[]))));
        let done = Restored {
            added: 2,
            replaced: 0,
            kept: 0,
        };
        let effects = state.update(Action::Restored(Ok(done)));
        assert!(
            matches!(&effects[..], [Effect::StoreChanged, Effect::Toast(t)] if *t == Toast::success("Imported 2 counters")),
            "{effects:?}"
        );
        assert_eq!(
            state.notice(),
            Some("Imported from /exports/cairn-backup.json")
        );
    }

    #[test]
    fn a_failed_restore_or_read_says_nothing_changed() {
        let mut state = State::default();
        state.update(Action::BackupRead(Ok(pending(&[]))));
        let effects = state.update(Action::Restored(Err("bad counter".to_owned())));
        assert!(
            matches!(&effects[..], [Effect::Toast(t)] if t.kind() == ToastKind::Error && t.text() == "Import failed"),
            "{effects:?}"
        );
        assert_eq!(state.notice(), Some("Bad counter. Nothing was changed"));
        let effects = state.update(Action::BackupRead(Err("No backup in /exports".to_owned())));
        assert_eq!(toast(&effects), Some(&Toast::error("Import failed")));
        assert_eq!(state.notice(), Some("No backup in /exports"));
    }
}
