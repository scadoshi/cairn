//! The theme sheet and the wiring every host shares: Settings' theme row
//! and the menu's footer both raise it. A kept theme carries the home-screen
//! icon along when the icon follows the theme, and a flipped mode saves at
//! once through the app's theme signal.

use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use orangui::{ThemeConfig, ThemeSheet};

use crate::domain::preferences::{AppIcon, Preferences};
use crate::inbound::ui::{TOAST_NORMAL, TOAST_QUICK, use_prefs};
use crate::outbound::app_icon;

/// Whether the app-wide theme sheet is up. Provided at the root so the menu,
/// which folds as it raises the sheet, can reach it.
#[derive(Debug, Clone, Copy)]
pub struct ThemeSheetOpen(pub Signal<bool>);

/// Redraws the home-screen icon in `kept` when the icon follows the theme.
/// `Ok(true)` when it did.
///
/// # Errors
///
/// The platform's message when the icon could not be switched.
pub fn follow_icon(prefs: &Preferences, kept: &ThemeConfig) -> Result<bool, String> {
    if prefs.app_icon != AppIcon::MatchesTheme {
        return Ok(false);
    }
    app_icon::show(kept).map(|()| true)
}

/// A callback that flips the app's theme between its dark and light
/// palettes, carries the icon along, and says which mode is on. The root
/// effect persists the change.
pub fn use_toggle_dark() -> Callback<()> {
    let mut theme = use_context::<Signal<ThemeConfig>>();
    let toast = use_toast();
    let prefs = use_prefs();
    use_callback(move |()| {
        let prev = theme.read().clone();
        let on = !prev.is_dark;
        let next = ThemeConfig {
            name: prev.name,
            is_dark: on,
        };
        theme.set(next.clone());
        if let Err(e) = follow_icon(&prefs.peek(), &next) {
            toast.error(e, ToastOptions::default());
        }
        toast.info(
            format!("Dark mode {}", if on { "on" } else { "off" }),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    })
}

/// The kit's theme sheet over the app's theme: live preview, the checkmark
/// keeps the pick and the x restores. `on_kept` hears a kept theme after
/// the icon followed it, for a host that shows the icon.
#[component]
pub fn AppThemeSheet(
    open: Signal<bool>,
    #[props(default)] hint: Option<Signal<bool>>,
    #[props(default)] on_kept: Option<EventHandler<ThemeConfig>>,
) -> Element {
    let theme = use_context::<Signal<ThemeConfig>>();
    let prefs = use_prefs();
    let toast = use_toast();
    rsx! {
        ThemeSheet {
            open,
            theme,
            hint,
            on_save: move |kept: ThemeConfig| {
                match follow_icon(&prefs.peek(), &kept) {
                    Ok(true) => {
                        if let Some(on_kept) = on_kept {
                            on_kept.call(kept);
                        }
                    }
                    Ok(false) => {}
                    Err(e) => toast.error(e, ToastOptions::default()),
                }
                toast.success(
                    "Theme saved".to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
            },
            on_unchanged: move |()| {
                toast.info(
                    "Theme unchanged".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            },
        }
    }
}
