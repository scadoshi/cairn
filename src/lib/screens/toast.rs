//! A toast a screen asks for.

use std::time::Duration;

use orangui_core::toast::{NORMAL, QUICK};

/// Which of the toast stack's looks a toast takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    /// A routine confirmation.
    Info,
    /// Something worth a moment: a goal met.
    Success,
    /// Something went wrong.
    Error,
}

/// A toast a screen asks for: its kind, its text and how long it stays up.
///
/// Built only through [`Toast::info`], [`Toast::success`] and
/// [`Toast::error`], which tie the duration to the kind. Cairn's stylesheet
/// fades a toast on a timer keyed by its kind (`architecture/structure.md`,
/// "One coupling"), so an info toast must be `QUICK` and the rest `NORMAL`,
/// or it vanishes while it is still up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    kind: ToastKind,
    text: String,
    duration: Duration,
}

impl Toast {
    /// A routine confirmation, up for `QUICK`.
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(ToastKind::Info, text.into(), QUICK)
    }

    /// A success, up for `NORMAL`.
    pub fn success(text: impl Into<String>) -> Self {
        Self::new(ToastKind::Success, text.into(), NORMAL)
    }

    /// A failure, up for `NORMAL`.
    pub fn error(text: impl Into<String>) -> Self {
        Self::new(ToastKind::Error, text.into(), NORMAL)
    }

    fn new(kind: ToastKind, text: String, duration: Duration) -> Self {
        Self {
            kind,
            text,
            duration,
        }
    }

    /// Which look it takes.
    pub fn kind(&self) -> ToastKind {
        self.kind
    }

    /// What it says.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// How long it stays up.
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_is_quick_and_the_rest_normal() {
        assert_eq!(Toast::info("a").duration(), QUICK);
        assert_eq!(Toast::success("a").duration(), NORMAL);
        assert_eq!(Toast::error("a").duration(), NORMAL);
        assert_eq!(Toast::success("a").kind(), ToastKind::Success);
        assert_eq!(Toast::error("b").text(), "b");
    }
}
